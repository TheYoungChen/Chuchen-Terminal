const { spawn } = require('node:child_process')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const http = require('node:http')

const appCwd = path.resolve(__dirname, '..')
const logDir = path.join(os.tmpdir(), 'chuchen-terminal-runner')
fs.mkdirSync(logDir, { recursive: true })
const outPath = path.join(logDir, 'vite-detached.out.log')
const errPath = path.join(logDir, 'vite-detached.err.log')

const out = fs.openSync(outPath, 'a')
const err = fs.openSync(errPath, 'a')

const viteCli = path.join(appCwd, 'node_modules', 'vite', 'bin', 'vite.js')
const child = spawn(process.execPath, [viteCli, '--configLoader', 'native', '--host', '127.0.0.1', '--port', '6173'], {
  cwd: appCwd,
  detached: true,
  stdio: ['ignore', out, err],
  windowsHide: true,
  env: {
    ...process.env,
    CI: '1',
  },
})

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function checkReady() {
  return new Promise((resolve) => {
    const req = http.get('http://127.0.0.1:6173/', (res) => {
      resolve(res.statusCode === 200)
      res.resume()
    })
    req.on('error', () => resolve(false))
    req.setTimeout(1200, () => {
      req.destroy()
      resolve(false)
    })
  })
}

async function main() {
  for (let i = 0; i < 20; i += 1) {
    if (await checkReady()) {
      child.unref()
      process.stdout.write(String(child.pid))
      return
    }
    await wait(500)
  }
  child.unref()
  process.stdout.write(`NOT_READY:${child.pid}`)
}

main().catch(() => {
  child.unref()
  process.stdout.write(`ERROR:${child.pid}`)
})
