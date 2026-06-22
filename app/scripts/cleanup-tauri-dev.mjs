import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
import path from 'node:path'

const appExePath = path.resolve(process.cwd(), 'src-tauri', 'target', 'debug', 'app.exe')
const cargoLockPath = path.resolve(process.cwd(), 'src-tauri', 'target', 'debug', '.cargo-lock')
const DEV_PORT = 6173

function escapePowerShell(value) {
  return value.replace(/'/g, "''")
}

function killProcessOnPort(port) {
  try {
    const output = execFileSync(
      'powershell',
      [
        '-NoProfile',
        '-Command',
        `$line = (netstat -ano | Select-String ":${port}" | Select-String "LISTENING" | Select-Object -First 1).Line; ` +
          `if ($line) { ($line -split '\\s+')[-1] }`,
      ],
      {
        encoding: 'utf8',
        stdio: ['ignore', 'pipe', 'pipe'],
      },
    ).trim()

    if (!output) return
    const pid = Number(output.split(/\s+/)[0])
    if (!Number.isFinite(pid) || pid <= 0) return
    execFileSync('taskkill', ['/PID', String(pid), '/F'], { stdio: 'ignore' })
  } catch {
    // Best-effort cleanup only.
  }
}

try {
  execFileSync(
    'powershell',
    [
      '-NoProfile',
      '-Command',
      `$target='${escapePowerShell(appExePath)}'; ` +
      `Get-Process app -ErrorAction SilentlyContinue | ` +
      `Where-Object { $_.Path -eq $target } | ` +
      `Stop-Process -Force -ErrorAction SilentlyContinue`,
    ],
    { stdio: 'ignore' },
  )
} catch {
  // Best-effort cleanup. If nothing is running or PowerShell returns non-zero,
  // Tauri dev can still continue and surface the real error.
}

killProcessOnPort(DEV_PORT)

try {
  if (fs.existsSync(cargoLockPath)) {
    fs.rmSync(cargoLockPath, { force: true })
  }
} catch {
  // Best-effort cleanup only. Cargo will recreate the lock if needed.
}
