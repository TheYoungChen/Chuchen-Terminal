import { execFileSync } from 'node:child_process'
import path from 'node:path'

const appExePath = path.resolve(process.cwd(), 'src-tauri', 'target', 'debug', 'app.exe')

function escapePowerShell(value) {
  return value.replace(/'/g, "''")
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
