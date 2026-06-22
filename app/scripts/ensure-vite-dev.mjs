import { spawn } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

const DEV_URL = 'http://127.0.0.1:6173/'
const LOG_PATH = path.join(os.tmpdir(), 'chuchen-terminal-vite-managed.log')
const ERROR_LOG_PATH = path.join(os.tmpdir(), 'chuchen-terminal-vite-managed.err.log')
const WAIT_TIMEOUT_MS = 20000
const POLL_INTERVAL_MS = 500
const DEV_MARKERS = [
  '/@vite/client',
  'vite/client',
  '__VITE_IS_MODERN__',
]

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function isDevServerReady() {
  try {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 1200)
    const response = await fetch(DEV_URL, { signal: controller.signal })
    clearTimeout(timer)
    if (!response.ok) return false
    const html = await response.text()
    return html.includes('<div id="app"></div>') && DEV_MARKERS.some((marker) => html.includes(marker))
  } catch {
    return false
  }
}

function killProcessOnPort(port) {
  if (process.platform !== 'win32') return
  try {
    const output = execFileSync('powershell', [
      '-NoProfile',
      '-Command',
      `$line = (netstat -ano | Select-String ":${port}" | Select-String "LISTENING" | Select-Object -First 1).Line; ` +
        `if ($line) { ($line -split '\\s+')[-1] }`,
    ], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    }).trim()
    if (!output) return
    const pid = Number(output.split(/\s+/)[0])
    if (!Number.isFinite(pid) || pid <= 0) return
    execFileSync('taskkill', ['/PID', String(pid), '/F'], { stdio: 'ignore' })
  } catch {
    // Best-effort cleanup only.
  }
}

function startDetachedVite() {
  const out = fs.openSync(LOG_PATH, 'a')
  const err = fs.openSync(ERROR_LOG_PATH, 'a')
  const isWindows = process.platform === 'win32'
  const command = isWindows ? 'cmd.exe' : 'npm'
  const args = isWindows
    ? ['/d', '/c', 'npm.cmd run dev -- --host 127.0.0.1 --port 6173']
    : ['run', 'dev', '--', '--host', '127.0.0.1', '--port', '6173']

  const child = spawn(command, args, {
    cwd: process.cwd(),
    detached: true,
    shell: false,
    stdio: ['ignore', out, err],
    windowsHide: true,
  })

  child.unref()
}

async function main() {
  if (await isDevServerReady()) {
    console.log(`Reusing existing dev server at ${DEV_URL}`)
    return
  }

  killProcessOnPort(6173)
  startDetachedVite()

  const startedAt = Date.now()
  while (Date.now() - startedAt < WAIT_TIMEOUT_MS) {
    if (await isDevServerReady()) {
      console.log(`Dev server ready at ${DEV_URL}`)
      return
    }
    await sleep(POLL_INTERVAL_MS)
  }

  console.error(`Dev server did not become ready within ${WAIT_TIMEOUT_MS}ms`)
  process.exitCode = 1
}

await main()
