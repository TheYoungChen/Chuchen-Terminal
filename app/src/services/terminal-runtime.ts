import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { FitAddon } from '@xterm/addon-fit'
import { Terminal } from '@xterm/xterm'
import { pickLocaleText } from './runtime-locale'

export type TerminalSessionOpened = {
  sessionId: string
  shellLabel: string
  workingDirectory: string
}

export type TerminalOutputPayload = {
  sessionId: string
  chunk: string
}

export type TerminalExitPayload = {
  sessionId: string
  exitCode: number
}

export type TerminalRuntimeOptions = {
  sessionId: string
  sessionName: string
  workingDirectory: string
  shellLabel: string
  fontFamily: string
  fontSize: number
  bridgeReady: boolean
  active?: boolean
  onCommandCommitted?: (sessionId: string, command: string) => void
  onSessionStateChange?: (sessionId: string, status: 'idle' | 'running') => void
  onOutputChunk?: (sessionId: string, chunk: string) => void
  onSessionExit?: (sessionId: string, exitCode: number) => void
}

type RuntimeEntry = {
  terminal: Terminal
  fitAddon: FitAddon
  outputUnlisten: UnlistenFn | null
  exitUnlisten: UnlistenFn | null
  resizeObserver: ResizeObserver | null
  dataDisposable: { dispose: () => void } | null
  host: HTMLDivElement | null
  container: HTMLDivElement | null
  bridgeReady: boolean
  sessionOpened: boolean
  options: TerminalRuntimeOptions
  inputBuffer: string
  fitRafId: number | null
  fitTimer: number | null
  resizeDebounceTimer: number | null
  lastResizeCols: number
  lastResizeRows: number
  outputQueue: string[]
  outputFlushRafId: number | null
  outputFlushTimer: number | null
  inputQueue: string[]
  inputFlushTimer: number | null
  inputFlushInFlight: boolean
  openingSession: Promise<void> | null
}

declare global {
  var __ctTerminalRuntimeStore: Map<string, RuntimeEntry> | undefined
}

const MAX_OUTPUT_QUEUE_LENGTH = 256_000
const MAX_VISIBLE_OUTPUT_FLUSH_CHARS = 64_000
const MAX_BACKGROUND_OUTPUT_FLUSH_CHARS = 16_000
const BACKGROUND_OUTPUT_FLUSH_DELAY_MS = 180
const INPUT_FLUSH_DELAY_MS = 24
const BACKGROUND_FIT_DELAY_MS = 120

const runtimeStore = globalThis.__ctTerminalRuntimeStore ??= new Map<string, RuntimeEntry>()

function createTerminal(options: TerminalRuntimeOptions) {
  const terminal = new Terminal({
    allowTransparency: false,
    convertEol: false,
    cursorBlink: true,
    cursorStyle: 'bar',
    fontFamily: options.fontFamily,
    fontSize: options.fontSize,
    lineHeight: 1.35,
    letterSpacing: 0,
    scrollback: 1500,
    tabStopWidth: 4,
    theme: {
      background: '#050608',
      foreground: '#dbe4ef',
      cursor: '#7ddbd3',
      cursorAccent: '#061012',
      selectionBackground: '#264b55',
      black: '#111827',
      red: '#ef4444',
      green: '#22c55e',
      yellow: '#f59e0b',
      blue: '#60a5fa',
      magenta: '#c084fc',
      cyan: '#2dd4bf',
      white: '#e5e7eb',
      brightBlack: '#64748b',
      brightRed: '#f87171',
      brightGreen: '#4ade80',
      brightYellow: '#fbbf24',
      brightBlue: '#93c5fd',
      brightMagenta: '#d8b4fe',
      brightCyan: '#5eead4',
      brightWhite: '#f8fafc',
    },
  })

  const fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)

  return { terminal, fitAddon }
}

function terminalHostVisible(entry: RuntimeEntry) {
  if (typeof document === 'undefined') return false
  return Boolean(entry.host?.isConnected && document.contains(entry.host) && !document.hidden)
}

function clearOutputFlushHandles(entry: RuntimeEntry) {
  if (entry.outputFlushRafId) {
    cancelAnimationFrame(entry.outputFlushRafId)
    entry.outputFlushRafId = null
  }
  if (entry.outputFlushTimer) {
    window.clearTimeout(entry.outputFlushTimer)
    entry.outputFlushTimer = null
  }
}

function flushOutputQueue(entry: RuntimeEntry, visible: boolean) {
  try {
    if (!entry.outputQueue.length) return
    const chunk = takeOutputFlushChunk(entry, visible ? MAX_VISIBLE_OUTPUT_FLUSH_CHARS : MAX_BACKGROUND_OUTPUT_FLUSH_CHARS)
    entry.terminal.write(chunk)
  } finally {
    clearOutputFlushHandles(entry)
    if (entry.outputQueue.length) {
      scheduleOutputFlush(entry)
    }
  }
}

function scheduleOutputFlush(entry: RuntimeEntry) {
  const visible = terminalHostVisible(entry)
  if (visible) {
    if (entry.outputFlushRafId) return
    if (entry.outputFlushTimer) {
      window.clearTimeout(entry.outputFlushTimer)
      entry.outputFlushTimer = null
    }
    entry.outputFlushRafId = window.requestAnimationFrame(() => {
      flushOutputQueue(entry, true)
    })
    return
  }

  if (entry.outputFlushTimer) return
  if (entry.outputFlushRafId) {
    cancelAnimationFrame(entry.outputFlushRafId)
    entry.outputFlushRafId = null
  }
  entry.outputFlushTimer = window.setTimeout(() => {
    flushOutputQueue(entry, false)
  }, BACKGROUND_OUTPUT_FLUSH_DELAY_MS)
}

function outputQueueLength(entry: RuntimeEntry) {
  return entry.outputQueue.reduce((total, chunk) => total + chunk.length, 0)
}

function enqueueOutputChunk(entry: RuntimeEntry, chunk: string) {
  if (!chunk) return

  entry.outputQueue.push(chunk)

  let overflow = outputQueueLength(entry) - MAX_OUTPUT_QUEUE_LENGTH
  while (overflow > 0 && entry.outputQueue.length > 1) {
    const first = entry.outputQueue[0]
    if (first.length <= overflow) {
      overflow -= first.length
      entry.outputQueue.shift()
      continue
    }

    entry.outputQueue[0] = first.slice(overflow)
    overflow = 0
  }

  if (overflow > 0 && entry.outputQueue.length === 1) {
    entry.outputQueue[0] = entry.outputQueue[0].slice(-MAX_OUTPUT_QUEUE_LENGTH)
  }
}

function takeOutputFlushChunk(entry: RuntimeEntry, maxChars: number) {
  let remaining = maxChars
  const chunks: string[] = []

  while (entry.outputQueue.length && remaining > 0) {
    const first = entry.outputQueue[0]
    if (first.length <= remaining) {
      chunks.push(first)
      remaining -= first.length
      entry.outputQueue.shift()
      continue
    }

    chunks.push(first.slice(0, remaining))
    entry.outputQueue[0] = first.slice(remaining)
    remaining = 0
  }

  return chunks.join('')
}

async function flushInputQueue(entry: RuntimeEntry) {
  if (entry.inputFlushInFlight || !entry.bridgeReady || !entry.sessionOpened || !entry.inputQueue.length) {
    return
  }
  entry.inputFlushInFlight = true
  const input = entry.inputQueue.join('')
  entry.inputQueue = []
  if (!input) {
    entry.inputFlushInFlight = false
    return
  }

  try {
    await invoke('write_terminal_input', {
      sessionId: entry.options.sessionId,
      input,
    })
  } catch {
    entry.inputQueue = [input, ...entry.inputQueue]
  } finally {
    entry.inputFlushInFlight = false
    if (entry.inputQueue.length) {
      scheduleInputFlush(entry)
    }
  }
}

function scheduleInputFlush(entry: RuntimeEntry, immediate = false) {
  if (!entry.bridgeReady || !entry.sessionOpened || !entry.inputQueue.length) {
    return
  }

  if (immediate) {
    if (entry.inputFlushTimer) {
      window.clearTimeout(entry.inputFlushTimer)
      entry.inputFlushTimer = null
    }
    void flushInputQueue(entry)
    return
  }

  if (entry.inputFlushTimer) return

  entry.inputFlushTimer = window.setTimeout(async () => {
    entry.inputFlushTimer = null
    await flushInputQueue(entry)
  }, INPUT_FLUSH_DELAY_MS)
}

function scheduleFit(entry: RuntimeEntry) {
  if (entry.fitRafId) {
    cancelAnimationFrame(entry.fitRafId)
  }
  if (entry.fitTimer) {
    window.clearTimeout(entry.fitTimer)
    entry.fitTimer = null
  }

  const runFit = async () => {
    try {
      if (!entry.host || !entry.container) return
      const hostRect = entry.host.getBoundingClientRect()
      if (hostRect.width < 40 || hostRect.height < 40) return

      entry.fitAddon.fit()
      const nextCols = entry.terminal.cols
      const nextRows = entry.terminal.rows

      if (entry.bridgeReady && entry.sessionOpened && (nextCols !== entry.lastResizeCols || nextRows !== entry.lastResizeRows)) {
        if (entry.resizeDebounceTimer) {
          window.clearTimeout(entry.resizeDebounceTimer)
        }

        entry.resizeDebounceTimer = window.setTimeout(async () => {
          entry.lastResizeCols = nextCols
          entry.lastResizeRows = nextRows
          entry.resizeDebounceTimer = null
          await invoke('resize_terminal', {
            sessionId: entry.options.sessionId,
            cols: nextCols,
            rows: nextRows,
          })
        }, entry.options.active ? 48 : 120)
      }
    } catch {
    } finally {
      entry.fitRafId = null
      entry.fitTimer = null
    }
  }

  if (entry.options.active) {
    entry.fitRafId = window.requestAnimationFrame(() => {
      void runFit()
    })
    return
  }

  entry.fitTimer = window.setTimeout(() => {
    void runFit()
  }, BACKGROUND_FIT_DELAY_MS)
}

async function bindEvents(entry: RuntimeEntry) {
  if (!entry.bridgeReady) {
    return
  }

  if (!entry.outputUnlisten) {
    entry.outputUnlisten = await listen<TerminalOutputPayload>('terminal://output', (event) => {
      if (event.payload.sessionId !== entry.options.sessionId) return
      enqueueOutputChunk(entry, event.payload.chunk)
      entry.options.onOutputChunk?.(entry.options.sessionId, event.payload.chunk)
      scheduleOutputFlush(entry)
    })
  }

  if (!entry.exitUnlisten) {
    entry.exitUnlisten = await listen<TerminalExitPayload>('terminal://exit', (event) => {
      if (event.payload.sessionId !== entry.options.sessionId) return
      const line = `\r\n\x1b[38;2;135;147;161m[\u4f1a\u8bdd\u7ed3\u675f\uff0c\u9000\u51fa\u7801 ${event.payload.exitCode}]\x1b[0m\r\n`
      enqueueOutputChunk(entry, line)
      scheduleOutputFlush(entry)
      entry.sessionOpened = false
      entry.inputBuffer = ''
      entry.inputQueue = []
      if (entry.inputFlushTimer) {
        window.clearTimeout(entry.inputFlushTimer)
        entry.inputFlushTimer = null
      }
      entry.options.onSessionExit?.(entry.options.sessionId, event.payload.exitCode)
      entry.options.onSessionStateChange?.(entry.options.sessionId, 'idle')
    })
  }

  if (!entry.dataDisposable) {
    entry.dataDisposable = entry.terminal.onData((data) => {
      if (!entry.bridgeReady || !entry.sessionOpened) return
      entry.inputQueue.push(data)
      scheduleInputFlush(entry, data === '\r')

      if (data === '\r') {
        const command = entry.inputBuffer.trim()
        if (command) {
          entry.options.onCommandCommitted?.(entry.options.sessionId, command)
        }
        entry.inputBuffer = ''
        return
      }

      if (data === '\u007f') {
        entry.inputBuffer = entry.inputBuffer.slice(0, -1)
        return
      }

      if (data >= ' ' && data !== '\u001b') {
        entry.inputBuffer += data
      }
    })
  }
}

async function ensureSession(entry: RuntimeEntry) {
  if (!entry.bridgeReady || entry.sessionOpened) return
  if (entry.openingSession) {
    await entry.openingSession
    return
  }

  entry.openingSession = (async () => {
    try {
      const opened = await invoke<TerminalSessionOpened>('open_terminal_session', {
        sessionId: entry.options.sessionId,
        workingDirectory: entry.options.workingDirectory,
        cols: entry.terminal.cols || 120,
        rows: entry.terminal.rows || 32,
      })

      entry.options.workingDirectory = opened.workingDirectory
      entry.sessionOpened = true
      entry.options.onSessionStateChange?.(entry.options.sessionId, 'running')
      scheduleFit(entry)
      entry.terminal.scrollToBottom()
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      if (message.includes('终端会话已存在') || message.toLowerCase().includes('terminal session already exists')) {
        entry.sessionOpened = true
        entry.options.onSessionStateChange?.(entry.options.sessionId, 'running')
        scheduleFit(entry)
        entry.terminal.scrollToBottom()
        return
      }
      entry.terminal.writeln(`\x1b[38;2;240;114;114m${pickLocaleText('终端启动失败：', 'Terminal startup failed: ')}${message}\x1b[0m`)
    } finally {
      entry.openingSession = null
    }
  })()

  await entry.openingSession
}
function attachHost(entry: RuntimeEntry, host: HTMLDivElement) {
  if (entry.host === host && host.firstChild) {
    scheduleFit(entry)
    scheduleOutputFlush(entry)
    return
  }

  entry.host = host
  host.replaceChildren()
  if (entry.container) {
    host.appendChild(entry.container)
  } else {
    const mount = document.createElement('div')
    mount.style.width = '100%'
    mount.style.height = '100%'
    mount.style.minHeight = '0'
    entry.container = mount
    host.appendChild(mount)
    entry.terminal.open(mount)
  }

  if (entry.resizeObserver) {
    entry.resizeObserver.disconnect()
  }
  entry.resizeObserver = new ResizeObserver(() => scheduleFit(entry))
  entry.resizeObserver.observe(host)
  scheduleFit(entry)
  scheduleOutputFlush(entry)
}

export async function mountTerminalRuntime(host: HTMLDivElement, options: TerminalRuntimeOptions) {
  let entry = runtimeStore.get(options.sessionId)

  if (!entry) {
    const { terminal, fitAddon } = createTerminal(options)
    entry = {
      terminal,
      fitAddon,
      outputUnlisten: null,
      exitUnlisten: null,
      resizeObserver: null,
      dataDisposable: null,
      host: null,
      container: null,
      bridgeReady: options.bridgeReady,
      sessionOpened: false,
      options: { ...options },
      inputBuffer: '',
      fitRafId: null,
      fitTimer: null,
      resizeDebounceTimer: null,
      lastResizeCols: 0,
      lastResizeRows: 0,
      outputQueue: [],
      outputFlushRafId: null,
      outputFlushTimer: null,
      inputQueue: [],
      inputFlushTimer: null,
      inputFlushInFlight: false,
      openingSession: null,
    }
    runtimeStore.set(options.sessionId, entry)
  }
  const currentEntry = entry

  currentEntry.options = { ...currentEntry.options, ...options }
  currentEntry.bridgeReady = options.bridgeReady
  await bindEvents(currentEntry)
  currentEntry.terminal.options.fontFamily = options.fontFamily
  currentEntry.terminal.options.fontSize = options.fontSize
  attachHost(currentEntry, host)
  await ensureSession(currentEntry)
  if (currentEntry.options.active) {
    currentEntry.terminal.focus()
  }
}

export function updateTerminalRuntimeActivity(sessionId: string, active: boolean) {
  const entry = runtimeStore.get(sessionId)
  if (!entry) return
  if (entry.options.active === active) return
  entry.options = {
    ...entry.options,
    active,
  }
  scheduleFit(entry)
  scheduleOutputFlush(entry)
  if (active) {
    entry.terminal.focus()
  }
}

export function focusTerminalRuntime(sessionId: string) {
  runtimeStore.get(sessionId)?.terminal.focus()
}

export function unmountTerminalRuntime(sessionId: string) {
  const entry = runtimeStore.get(sessionId)
  if (!entry) return
  entry.resizeObserver?.disconnect()
  entry.resizeObserver = null
  if (entry.fitRafId) {
    cancelAnimationFrame(entry.fitRafId)
    entry.fitRafId = null
  }
  if (entry.fitTimer) {
    window.clearTimeout(entry.fitTimer)
    entry.fitTimer = null
  }
  if (entry.outputFlushRafId) {
    cancelAnimationFrame(entry.outputFlushRafId)
    entry.outputFlushRafId = null
  }
  if (entry.outputFlushTimer) {
    window.clearTimeout(entry.outputFlushTimer)
    entry.outputFlushTimer = null
  }
  if (entry.inputFlushTimer) {
    window.clearTimeout(entry.inputFlushTimer)
    entry.inputFlushTimer = null
  }
  if (entry.resizeDebounceTimer) {
    window.clearTimeout(entry.resizeDebounceTimer)
    entry.resizeDebounceTimer = null
  }
  entry.host = null
}

export async function destroyTerminalRuntime(sessionId: string) {
  const entry = runtimeStore.get(sessionId)
  if (!entry) return

  runtimeStore.delete(sessionId)
  entry.resizeObserver?.disconnect()
  entry.resizeObserver = null
  if (entry.fitRafId) {
    cancelAnimationFrame(entry.fitRafId)
    entry.fitRafId = null
  }
  if (entry.fitTimer) {
    window.clearTimeout(entry.fitTimer)
    entry.fitTimer = null
  }
  clearOutputFlushHandles(entry)
  if (entry.inputFlushTimer) {
    window.clearTimeout(entry.inputFlushTimer)
    entry.inputFlushTimer = null
  }
  if (entry.resizeDebounceTimer) {
    window.clearTimeout(entry.resizeDebounceTimer)
    entry.resizeDebounceTimer = null
  }
  entry.outputUnlisten?.()
  entry.outputUnlisten = null
  entry.exitUnlisten?.()
  entry.exitUnlisten = null
  entry.dataDisposable?.dispose()
  entry.dataDisposable = null

  entry.host = null
  entry.container = null

  entry.sessionOpened = false
  entry.options.onSessionStateChange?.(sessionId, 'idle')
  try {
    void invoke('close_terminal_session', { sessionId })
  } catch {
  }
  entry.terminal.dispose()
}

export async function writeTerminalText(sessionId: string, text: string) {
  const entry = runtimeStore.get(sessionId)
  if (!entry || !entry.bridgeReady || !entry.sessionOpened) {
    throw new Error('terminal_not_ready')
  }

  entry.terminal.focus()
  await invoke('write_terminal_input', {
    sessionId,
    input: text,
  })

  entry.inputBuffer += text
}

export async function ensureTerminalReady(sessionId: string) {
  const entry = runtimeStore.get(sessionId)
  if (!entry || !entry.bridgeReady) {
    throw new Error('terminal_not_ready')
  }

  await ensureSession(entry)

  if (!entry.sessionOpened) {
    throw new Error('terminal_not_ready')
  }
}

export function getTerminalRuntimeState(sessionId: string) {
  const entry = runtimeStore.get(sessionId)
  return {
    mounted: Boolean(entry),
    bridgeReady: Boolean(entry?.bridgeReady),
    sessionOpened: Boolean(entry?.sessionOpened),
  }
}
