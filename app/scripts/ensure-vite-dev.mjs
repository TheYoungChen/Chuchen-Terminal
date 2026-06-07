import { spawn } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

const DEV_URL = 'http://127.0.0.1:6173/'
const LOG_PATH = path.resolve(process.cwd(), '.tmp-vite-managed.log')
const ERROR_LOG_PATH = path.resolve(process.cwd(), '.tmp-vite-managed.err.log')
const WAIT_TIMEOUT_MS = 20000
const POLL_INTERVAL_MS = 500

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
    return html.includes('<div id="app"></div>')
  } catch {
    return false
  }
}

function startDetachedVite() {
  const out = fs.openSync(LOG_PATH, 'a')
  const err = fs.openSync(ERROR_LOG_PATH, 'a')
  const command = process.platform === 'win32'
    ? 'npx vite --host 127.0.0.1 --configLoader native'
    : 'npx vite --host 127.0.0.1 --configLoader native'

  const child = spawn(command, {
    cwd: process.cwd(),
    detached: true,
    shell: true,
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
