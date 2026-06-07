import path from 'node:path'
import os from 'node:os'
import { spawn, spawnSync } from 'node:child_process'

const rootDir = path.resolve(import.meta.dirname)
const tauriDriverPath = process.env.TAURI_DRIVER_PATH || 'tauri-driver'
const nativeDriverPath = process.env.WEBDRIVER_PATH || 'msedgedriver'
const tauriBinaryPath = path.resolve(rootDir, 'src-tauri/target/debug/app.exe')

let tauriDriverProcess
let shuttingDown = false

function closeTauriDriver() {
  shuttingDown = true
  if (tauriDriverProcess && !tauriDriverProcess.killed) {
    tauriDriverProcess.kill()
  }
}

export const config = {
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',
  specs: ['./tests/desktop/**/*.spec.mjs'],
  maxInstances: 1,
  runner: 'local',
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  capabilities: [{
    maxInstances: 1,
    browserName: 'MicrosoftEdge',
    'tauri:options': {
      application: tauriBinaryPath,
    },
  }],
  onPrepare() {
    spawnSync('npm', ['run', 'tauri', 'build', '--', '--debug', '--no-bundle'], {
      cwd: rootDir,
      stdio: 'inherit',
      shell: true,
    })
  },
  beforeSession() {
    tauriDriverProcess = spawn(tauriDriverPath, ['--native-driver', nativeDriverPath], {
      cwd: rootDir,
      stdio: ['ignore', 'pipe', 'pipe'],
      windowsHide: true,
      env: process.env,
    })

    tauriDriverProcess.stdout?.on('data', (chunk) => {
      process.stdout.write(`[tauri-driver] ${chunk}`)
    })

    tauriDriverProcess.stderr?.on('data', (chunk) => {
      process.stderr.write(`[tauri-driver] ${chunk}`)
    })

    tauriDriverProcess.on('exit', (code) => {
      if (!shuttingDown) {
        console.error(`tauri-driver exited unexpectedly: ${code}`)
        process.exit(1)
      }
    })
  },
  afterSession() {
    closeTauriDriver()
  },
  onComplete() {
    closeTauriDriver()
  },
}
