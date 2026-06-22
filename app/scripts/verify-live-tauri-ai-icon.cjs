const { spawn } = require('node:child_process')
const http = require('node:http')
const path = require('node:path')

async function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function poll(label, fn, timeoutMs = 30000, intervalMs = 500) {
  const started = Date.now()
  while (Date.now() - started < timeoutMs) {
    try {
      const value = await fn()
      if (value) return value
    } catch {
    }
    await wait(intervalMs)
  }
  throw new Error(`Timeout waiting for ${label}`)
}

async function httpJson(url) {
  return new Promise((resolve, reject) => {
    const req = http.get(url, (res) => {
      let data = ''
      res.setEncoding('utf8')
      res.on('data', (chunk) => { data += chunk })
      res.on('end', () => {
        try {
          resolve(JSON.parse(data))
        } catch (error) {
          reject(error)
        }
      })
    })
    req.on('error', reject)
    req.setTimeout(1500, () => {
      req.destroy(new Error(`timeout ${url}`))
    })
  })
}

async function isHttpReady(url) {
  return new Promise((resolve) => {
    const req = http.get(url, (res) => {
      res.resume()
      resolve(res.statusCode === 200)
    })
    req.on('error', () => resolve(false))
    req.setTimeout(1000, () => {
      req.destroy()
      resolve(false)
    })
  })
}

async function main() {
  const appCwd = path.resolve(__dirname, '..')

  const vite = spawn('cmd.exe', ['/d', '/c', 'npm.cmd run dev -- --host 127.0.0.1 --port 6173'], {
    cwd: appCwd,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
    env: {
      ...process.env,
      CI: '1',
    },
  })
  let viteOut = ''
  let viteErr = ''
  vite.stdout.on('data', (d) => { viteOut += d.toString() })
  vite.stderr.on('data', (d) => { viteErr += d.toString() })

  const cargo = spawn('cargo.exe', ['run', '--no-default-features', '--color', 'always', '--'], {
    cwd: path.join(appCwd, 'src-tauri'),
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
    env: {
      ...process.env,
      CARGO_TARGET_DIR: path.join(process.env.TEMP || process.env.TMP || appCwd, 'chuchen-terminal-cargo-target'),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9222',
    },
  })
  let cargoOut = ''
  let cargoErr = ''
  cargo.stdout.on('data', (d) => { cargoOut += d.toString() })
  cargo.stderr.on('data', (d) => { cargoErr += d.toString() })

  try {
    await poll('vite dev server', () => isHttpReady('http://127.0.0.1:6173/'))
    const targets = await poll('webview2 debug target', async () => {
      const list = await httpJson('http://127.0.0.1:9222/json/list')
      return Array.isArray(list) && list.length ? list : null
    }, 45000, 1000)

    const pageTarget = targets.find((item) => typeof item.url === 'string' && item.url.startsWith('http://127.0.0.1:6173/'))
    if (!pageTarget) {
      throw new Error(`No 6173 target in 9222 list: ${JSON.stringify(targets, null, 2)}`)
    }

    const puppeteer = require('puppeteer-core')
    const browser = await puppeteer.connect({ browserURL: 'http://127.0.0.1:9222' })
    const pages = await browser.pages()
    const page = pages.find((item) => item.url().startsWith('http://127.0.0.1:6173/'))
    if (!page) throw new Error('No page object for 6173 target')

    await page.waitForTimeout(2000)

    const result = await page.evaluate(() => {
      const storageRaw = localStorage.getItem('chuchen-terminal.workspace-data.v1')
      const restoreRaw = localStorage.getItem('chuchen-terminal.workbench-restore-state.v1')
      return {
        title: document.title,
        url: location.href,
        tabTexts: Array.from(document.querySelectorAll('.terminal-window-tab span')).map((el) => el.textContent?.trim()).filter(Boolean),
        explorerTexts: Array.from(document.querySelectorAll('.explorer-pane__title-row strong')).map((el) => el.textContent?.trim()).filter(Boolean),
        brandImages: Array.from(document.querySelectorAll('.terminal-window-tab img.ai-brand-icon, .explorer-ai-badge img.ai-brand-icon')).map((el) => ({
          cls: el.getAttribute('class'),
          src: el.getAttribute('src'),
        })),
        bodySample: document.body.innerText.slice(0, 1500),
        storagePreview: storageRaw ? storageRaw.slice(0, 4000) : null,
        restorePreview: restoreRaw ? restoreRaw.slice(0, 1200) : null,
      }
    })

    console.log(JSON.stringify({ result, targets }, null, 2))
    await browser.disconnect()
  } finally {
    vite.kill('SIGTERM')
    cargo.kill('SIGTERM')
    await wait(1000)
    if (vite.exitCode == null) vite.kill('SIGKILL')
    if (cargo.exitCode == null) cargo.kill('SIGKILL')
  }
}

main().catch((error) => {
  console.error(JSON.stringify({
    error: error instanceof Error ? error.message : String(error),
  }, null, 2))
  process.exit(1)
})
