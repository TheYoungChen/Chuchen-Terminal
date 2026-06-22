import fs from 'node:fs'
import { spawn, execFileSync } from 'node:child_process'
import os from 'node:os'
import path from 'node:path'

const APP_CWD = process.cwd()
const DEV_URL = 'http://127.0.0.1:6173/'
const WAIT_TIMEOUT_MS = 20000
const POLL_INTERVAL_MS = 400
const DEV_MARKERS = [
  '/@vite/client',
  'vite/client',
  '__VITE_IS_MODERN__',
]

const cargoTargetDir = path.join(APP_CWD, 'src-tauri', 'target')
const legacyAppExePath = path.join(APP_CWD, 'src-tauri', 'target', 'debug', 'app.exe')
const tempAppExePath = path.join(cargoTargetDir, 'debug', 'app.exe')
const runnerTempDir = path.join(os.tmpdir(), 'chuchen-terminal-runner')
const runnerLogPath = path.join(runnerTempDir, 'tauri-dev-runner.log')

let viteChild = null
let tauriChild = null
let shuttingDown = false
let logStream = null

function ensureRunnerLog() {
  if (!fs.existsSync(runnerTempDir)) {
    fs.mkdirSync(runnerTempDir, { recursive: true })
  }
  if (!logStream) {
    logStream = fs.createWriteStream(runnerLogPath, { flags: 'a' })
  }
}

function logLine(message) {
  ensureRunnerLog()
  const line = `[${new Date().toISOString()}] ${message}\n`
  logStream.write(line)
}

function mirrorChildOutput(prefix, stream, target = process.stdout) {
  stream?.on('data', (chunk) => {
    const text = chunk.toString()
    target.write(text)
    logLine(`${prefix}${text.replace(/\r?\n$/, '')}`)
  })
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function escapePowerShell(value) {
  return value.replace(/'/g, "''")
}

function killPidTree(pid) {
  if (!pid || Number.isNaN(pid)) return
  try {
    execFileSync('taskkill', ['/PID', String(pid), '/T', '/F'], { stdio: 'ignore' })
  } catch {
    // Best effort.
  }
}

function killProcessOnPort(port) {
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
    killPidTree(pid)
  } catch {
    // Best effort.
  }
}

function cleanupKnownAppProcesses() {
  try {
    execFileSync('powershell', [
      '-NoProfile',
      '-Command',
      `$legacy='${escapePowerShell(legacyAppExePath)}'; ` +
        `$temp='${escapePowerShell(tempAppExePath)}'; ` +
        `Get-Process app -ErrorAction SilentlyContinue | ` +
        `Where-Object { $_.Path -eq $legacy -or $_.Path -eq $temp } | ` +
        `Stop-Process -Force -ErrorAction SilentlyContinue`,
    ], { stdio: 'ignore' })
  } catch {
    // Best effort.
  }
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

async function waitForDevServer() {
  const startedAt = Date.now()
  while (Date.now() - startedAt < WAIT_TIMEOUT_MS) {
    if (await isDevServerReady()) return
    await sleep(POLL_INTERVAL_MS)
  }
  throw new Error(`Dev server did not become ready within ${WAIT_TIMEOUT_MS}ms`)
}

function spawnVite() {
  logLine('Starting Vite dev server')
  viteChild = spawn('cmd.exe', ['/d', '/c', 'npm.cmd run dev -- --host 127.0.0.1 --port 6173'], {
    cwd: APP_CWD,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
    env: {
      ...process.env,
      CI: '1',
    },
  })
  logLine(`Vite pid=${viteChild.pid ?? 'unknown'}`)
  mirrorChildOutput('[vite] ', viteChild.stdout, process.stdout)
  mirrorChildOutput('[vite:err] ', viteChild.stderr, process.stderr)
  viteChild.on('exit', (code) => {
    logLine(`Vite exited with code ${code ?? 'unknown'}`)
    if (!shuttingDown && code !== 0) {
      console.error(`Vite dev server exited early with code ${code ?? 'unknown'}`)
      void shutdown(code ?? 1)
    }
  })
}

function spawnTauri() {
  logLine('Starting Tauri dev')
  tauriChild = spawn('cmd.exe', ['/d', '/c', 'npx.cmd tauri dev'], {
    cwd: APP_CWD,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
    env: {
      ...process.env,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9222',
      CARGO_TARGET_DIR: cargoTargetDir,
    },
  })
  logLine(`Tauri pid=${tauriChild.pid ?? 'unknown'}`)
  mirrorChildOutput('[tauri] ', tauriChild.stdout, process.stdout)
  mirrorChildOutput('[tauri:err] ', tauriChild.stderr, process.stderr)
  tauriChild.on('exit', (code) => {
    logLine(`Tauri exited with code ${code ?? 'unknown'}`)
    void shutdown(code ?? 0)
  })
}

async function shutdown(exitCode = 0) {
  if (shuttingDown) return
  shuttingDown = true
  logLine(`Shutdown requested exitCode=${exitCode}`)
  if (tauriChild?.pid) killPidTree(tauriChild.pid)
  if (viteChild?.pid) killPidTree(viteChild.pid)
  cleanupKnownAppProcesses()
  killProcessOnPort(6173)
  logStream?.end()
  process.exit(exitCode)
}

process.on('SIGINT', () => { void shutdown(0) })
process.on('SIGTERM', () => { void shutdown(0) })
process.on('exit', () => {
  shuttingDown = true
  if (tauriChild?.pid) killPidTree(tauriChild.pid)
  if (viteChild?.pid) killPidTree(viteChild.pid)
})

async function main() {
  ensureRunnerLog()
  logLine('Runner boot')
  cleanupKnownAppProcesses()
  killProcessOnPort(6173)
  spawnVite()
  await waitForDevServer()
  logLine('Vite ready')
  spawnTauri()
}

await main().catch(async (error) => {
  console.error(error instanceof Error ? error.message : String(error))
  await shutdown(1)
})
