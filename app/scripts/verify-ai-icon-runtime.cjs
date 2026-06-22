const { spawn } = require('node:child_process')
const http = require('node:http')
const path = require('node:path')

async function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function waitForServer(url, timeoutMs = 45000) {
  const started = Date.now()
  while (Date.now() - started < timeoutMs) {
    const ok = await new Promise((resolve) => {
      const req = http.get(url, (res) => {
        const good = res.statusCode === 200
        res.resume()
        resolve(good)
      })
      req.on('error', () => resolve(false))
      req.setTimeout(3000, () => {
        req.destroy()
        resolve(false)
      })
    })
    if (ok) return
    await wait(500)
  }
  throw new Error(`Server not ready: ${url}`)
}

async function main() {
  const appCwd = path.resolve(__dirname, '..')
  console.log('[step] spawn vite')
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

  try {
    console.log('[step] wait server')
    await waitForServer('http://127.0.0.1:6173/')
    console.log('[step] server ready')

    const puppeteer = require('puppeteer-core')
    console.log('[step] launch browser')
    const browser = await puppeteer.launch({
      executablePath: 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
      headless: 'new',
      args: ['--no-first-run', '--no-default-browser-check'],
    })
    console.log('[step] browser launched')

    console.log('[step] new page')
    const page = await browser.newPage()
    const consoleMessages = []
    page.on('console', (msg) => {
      consoleMessages.push({ type: msg.type(), text: msg.text() })
    })
    page.on('pageerror', (error) => {
      consoleMessages.push({ type: 'pageerror', text: error.message })
    })
    console.log('[step] goto initial')
    await page.goto('http://127.0.0.1:6173/', { waitUntil: 'commit', timeout: 60000 })
    console.log('[step] initial loaded')

    console.log('[step] seed storage')
    await page.evaluate(() => {
      const payload = {
        version: 2,
        workspaces: [{
          id: 'verify-workspace',
          name: '终端测试',
          description: '验证 AI 图标',
          rootPath: 'E:\\StudyFile\\Chuchen-Terminal',
          gitBranch: 'main',
          tags: ['AI'],
          lastOpenedAt: new Date().toISOString(),
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          defaultSnapshotId: null,
          workspaceType: 'default',
          providerProfiles: [],
          providerQuotas: [],
          providerUsageStats: [],
          terminalEntries: [{
            id: 'entry-ai',
            workspaceId: 'verify-workspace',
            name: 'AI CLI',
            shellType: 'pwsh7',
            workingDirectory: 'E:\\StudyFile\\Chuchen-Terminal',
            defaultCommand: '',
            lastCommand: 'cd ../',
            commandHistory: ['cd ../', 'ls'],
            favoriteCommands: [],
            status: 'idle',
            launchMode: 'open-only',
            environmentVariablesText: null,
            runtimeNote: null,
            tags: ['AI'],
            note: 'AI CLI 工作流模板生成',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          }],
          tabs: [{
            id: 'tab-ai',
            workspaceId: 'verify-workspace',
            name: 'AI',
            order: 0,
            layoutMode: 'horizontal',
            paneSequence: 1,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            panes: [{
              id: 'pane-ai',
              tabId: 'tab-ai',
              name: 'PowerShell 7',
              pathLabel: 'E:\\StudyFile\\Chuchen-Terminal',
              terminalEntryId: 'entry-ai',
              splitDirection: 'none',
              parentPaneId: null,
              order: 0,
              sizeRatio: 1,
              activeSessionId: 'session-ai',
              sessions: [{
                id: 'session-ai',
                name: 'PowerShell 7',
                pathLabel: 'E:\\StudyFile\\Chuchen-Terminal',
                terminalEntryId: 'entry-ai',
                status: 'idle',
                aiCliKind: null,
                hasUserCommand: true,
                lastCommandAt: new Date().toISOString(),
                lastOutputAt: new Date().toISOString(),
                lastExitCode: null,
                supervisorMode: 'off',
                supervisorState: 'idle',
                expectedDoneSignal: null,
                lastHeartbeatAt: new Date().toISOString(),
                lastActivityAt: new Date().toISOString(),
                supervisorNote: null,
              }],
              children: [],
            }],
          }],
          snapshots: [],
        }],
      }
      localStorage.setItem('chuchen-terminal.workspace-data.v1', JSON.stringify(payload))
      localStorage.setItem('chuchen-terminal.workbench-restore-state.v1', JSON.stringify({
        version: 1,
        selectedWorkspaceId: 'verify-workspace',
        selectedOverviewWorkspaceId: 'verify-workspace',
        openedWorkspaceIds: ['verify-workspace'],
        collapsedWorkspaceIds: [],
        workspaceFocus: {
          'verify-workspace': {
            activeTabId: 'tab-ai',
            activePaneId: 'pane-ai',
            activePaneIdsByTab: { 'tab-ai': 'pane-ai' },
            activeSessionIdsByPane: { 'pane-ai': 'session-ai' },
            collapsedTreeTabIds: [],
          },
        },
        updatedAt: new Date().toISOString(),
      }))
    })
    console.log('[step] storage seeded')

    console.log('[step] reload')
    await page.reload({ waitUntil: 'commit', timeout: 60000 })
    console.log('[step] reloaded')
    await wait(2000)
    console.log('[step] inspect dom')

    const result = await page.evaluate(() => {
      const tabTexts = Array.from(document.querySelectorAll('.terminal-window-tab span')).map((el) => el.textContent?.trim()).filter(Boolean)
      const explorerTexts = Array.from(document.querySelectorAll('.explorer-pane__title-row strong')).map((el) => el.textContent?.trim()).filter(Boolean)
      const brandImages = Array.from(document.querySelectorAll('.terminal-window-tab img.ai-brand-icon, .explorer-ai-badge img.ai-brand-icon')).map((el) => ({
        cls: el.getAttribute('class'),
        src: el.getAttribute('src'),
      }))
      return {
        title: document.title,
        tabTexts,
        explorerTexts,
        brandImages,
        bodySample: document.body.innerText.slice(0, 1200),
      }
    })

    console.log('[step] result ready')
    console.log(JSON.stringify({ result, consoleMessages }, null, 2))
    console.log('[step] close browser')
    await browser.close()
  } finally {
    console.log('[step] cleanup')
    vite.kill('SIGTERM')
    await wait(500)
    if (vite.exitCode == null) vite.kill('SIGKILL')
  }
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
