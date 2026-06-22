import { workspaceMocks } from '../mocks/workspaces'
import type {
  AiCliKind,
  PaneNode,
  PaneTerminalSession,
  ProviderConfigScope,
  ProviderProfile,
  ProviderProfileSource,
  ProviderProfileStatus,
  ProviderToolTarget,
  TerminalEntry,
  WorkspaceCard,
  WorkspaceSnapshot,
  WorkspaceTab,
} from '../types/workspace'

const STORAGE_KEY = 'chuchen-terminal.workspace-data.v1'
let lastSavedPayload = ''

type WorkspaceStorageRecord = {
  version: 2
  workspaces: WorkspaceCard[]
}

export function loadWorkspaces(): WorkspaceCard[] {
  if (typeof window === 'undefined' || !window.localStorage) {
    return normalizeWorkspaces(workspaceMocks)
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      const seeded = normalizeWorkspaces(workspaceMocks)
      saveWorkspaces(seeded)
      return seeded
    }

    const parsed = JSON.parse(raw) as WorkspaceStorageRecord
    const workspaces = normalizeWorkspaces(parsed.workspaces)
    saveWorkspaces(workspaces)
    return workspaces
  } catch {
    const fallback = normalizeWorkspaces(workspaceMocks)
    saveWorkspaces(fallback)
    return fallback
  }
}

export function saveWorkspaces(workspaces: WorkspaceCard[]) {
  if (typeof window === 'undefined' || !window.localStorage) {
    return
  }

  const payload: WorkspaceStorageRecord = {
    version: 2,
    workspaces: toPersistedWorkspaces(workspaces),
  }

  const serialized = JSON.stringify(payload)
  if (serialized === lastSavedPayload) {
    return
  }

  lastSavedPayload = serialized
  window.localStorage.setItem(STORAGE_KEY, serialized)
}

export function createId(prefix: string) {
  const randomPart = typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID().slice(0, 8)
    : Math.random().toString(36).slice(2, 10)

  return `${prefix}-${randomPart}`
}

export function createWorkspaceRecord(input: {
  name: string
  rootPath: string
  description: string
  tags: string[]
}): WorkspaceCard {
  const now = new Date().toISOString()
  const workspaceId = createId('workspace')
  const defaultTabId = createId('tab')

  return {
    id: workspaceId,
    name: input.name,
    description: input.description,
    rootPath: input.rootPath,
    tags: input.tags,
    lastOpenedAt: now,
    createdAt: now,
    updatedAt: now,
    defaultSnapshotId: null,
    workspaceType: 'default',
    providerProfiles: [],
    providerQuotas: [],
    providerUsageStats: [],
    terminalEntries: [],
    tabs: [
      {
        id: defaultTabId,
        workspaceId,
        name: '默认标签页',
        order: 0,
        layoutMode: 'grid',
        paneSequence: 0,
        panes: [],
        createdAt: now,
        updatedAt: now,
      },
    ],
  }
}

export function createProviderProfileRecord(input: {
  workspaceId: string
  name: string
  providerKind: ProviderProfile['providerKind']
  profileName: string
  configPath: string
  configScope: ProviderConfigScope
  managedBy: ProviderProfileSource
  authSource: string
  switchCommand: string
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  status?: ProviderProfileStatus
  color?: string | null
  note?: string | null
  isDefault?: boolean
  isActive?: boolean
}): ProviderProfile {
  const now = new Date().toISOString()

  return {
    id: createId('provider'),
    workspaceId: input.workspaceId,
    name: input.name,
    providerKind: input.providerKind,
    profileName: input.profileName,
    configPath: input.configPath,
    configScope: input.configScope,
    managedBy: input.managedBy,
    authSource: input.authSource,
    switchCommand: input.switchCommand,
    defaultModel: input.defaultModel,
    toolTargets: input.toolTargets,
    status: input.status ?? (input.isActive ? 'active' : 'available'),
    isActive: Boolean(input.isActive),
    lastDetectedAt: now,
    color: input.color ?? null,
    note: input.note ?? null,
    isDefault: Boolean(input.isDefault),
    createdAt: now,
    updatedAt: now,
  }
}

export function createTerminalEntryRecord(input: {
  workspaceId: string
  name: string
  workingDirectory: string
  defaultCommand: string
  launchMode: TerminalEntry['launchMode']
  environmentVariablesText?: string | null
  tags: string[]
  note?: string | null
}): TerminalEntry {
  const now = new Date().toISOString()

  return {
    id: createId('entry'),
    workspaceId: input.workspaceId,
    name: input.name,
    shellType: 'pwsh7',
    workingDirectory: input.workingDirectory,
    defaultCommand: input.defaultCommand,
    lastCommand: input.defaultCommand,
    commandHistory: input.defaultCommand ? [input.defaultCommand] : [],
    favoriteCommands: [],
    status: 'idle' as const,
    launchMode: input.launchMode,
    environmentVariablesText: input.environmentVariablesText ?? null,
    runtimeNote: null,
    tags: input.tags,
    note: input.note ?? null,
    createdAt: now,
    updatedAt: now,
  }
}

export function createWorkspaceSnapshotRecord(input: {
  workspace: WorkspaceCard
  name: string
  activeTabId: string | null
  activePaneId: string | null
  activePaneIdsByTab?: Record<string, string>
  activeSessionIdsByPane?: Record<string, string>
}): WorkspaceSnapshot {
  const now = new Date().toISOString()
  const tabsState = snapshotTabs(input.workspace.tabs)

  return {
    id: createId('snapshot'),
    workspaceId: input.workspace.id,
    name: input.name,
    tabsState,
    panesState: tabsState.reduce<Record<string, PaneNode[]>>((result, tab) => {
      result[tab.id] = tab.panes
      return result
    }, {}),
    activeTabId: input.activeTabId,
    activePaneId: input.activePaneId,
    activePaneIdsByTab: input.activePaneIdsByTab ?? {},
    activeSessionIdsByPane: input.activeSessionIdsByPane ?? {},
    createdAt: now,
    updatedAt: now,
  }
}


export function relativeTimeLabel(iso: string) {
  const date = new Date(iso)
  const diffMs = Date.now() - date.getTime()

  if (!Number.isFinite(diffMs) || Number.isNaN(diffMs)) {
    return '未知时间'
  }

  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour

  if (diffMs < hour) {
    const minutes = Math.max(1, Math.round(diffMs / minute))
    return `${minutes} 分钟前`
  }

  if (diffMs < day) {
    const hours = Math.max(1, Math.round(diffMs / hour))
    return `${hours} 小时前`
  }

  if (diffMs < day * 2) {
    return '昨天'
  }

  const days = Math.max(2, Math.round(diffMs / day))
  return `${days} 天前`
}


function sanitizeSessionName(name: string | undefined, fallback: string) {
  const value = (name || '').trim()
  if (!value || /^\?+/.test(value)) return fallback
  return value
}

function normalizeSearchText(value: string | undefined | null) {
  return (value || '').trim().toLocaleLowerCase()
}

function inferAiCliKindFromText(...parts: Array<string | null | undefined>): AiCliKind | null {
  const text = normalizeSearchText(parts.filter(Boolean).join(' '))
  if (!text) return null
  if (/(^|[\s>])(codex|cx)(?=$|[\s/:-])|openai codex|gpt-/.test(text)) return 'codex'
  if (/(^|[\s>])(claude|cc)(?=$|[\s/:-])|claude code|anthropic|sonnet|opus/.test(text)) return 'claude-code'
  if (/(^|[\s>])gemini(?=$|[\s/:-])|google ai|google-ai/.test(text)) return 'gemini-cli'
  if (/(^|[\s>])deepseek(?=$|[\s/:-])|deep seek/.test(text)) return 'deepseek-cli'
  if (/opencode|open code/.test(text)) return 'opencode'
  if (/\b(ai\s*cli|agent|assistant|llm)\b/.test(text)) return 'generic-ai'
  return null
}

function aiCliDisplayName(kind: AiCliKind | null | undefined) {
  if (kind === 'codex') return 'Codex'
  if (kind === 'claude-code') return 'Claude'
  if (kind === 'gemini-cli') return 'Gemini'
  if (kind === 'deepseek-cli') return 'DeepSeek'
  if (kind === 'opencode') return 'OpenCode'
  if (kind === 'generic-ai') return 'AI 会话'
  return '终端'
}

function isGenericTerminalSessionName(name: string | null | undefined) {
  const text = normalizeSearchText(name)
  if (!text) return true
  return text === 'powershell'
    || text === 'powershell 7'
    || text === 'pwsh'
    || text === 'terminal'
    || text === 'shell'
    || text === 'cmd'
    || text === 'command prompt'
    || text === 'bash'
    || text === 'zsh'
    || /^pane\s*\d+$/.test(text)
}

function inferPaneAiCliKind(
  workspace: WorkspaceCard,
  tab: WorkspaceTab,
  pane: PaneNode,
  session: Partial<PaneTerminalSession> | undefined,
  terminalEntries: TerminalEntry[],
) {
  const entry = terminalEntries.find((item) => item.id === (session?.terminalEntryId ?? pane.terminalEntryId))
  const exact = inferAiCliKindFromText(
    entry?.defaultCommand,
    entry?.lastCommand,
  )
  if (exact && exact !== 'generic-ai') return exact
  return exact
}

function ensureUniqueId(candidate: string | undefined, prefix: string, usedIds: Set<string>) {
  let nextId = (candidate || '').trim()
  if (!nextId || usedIds.has(nextId)) {
    nextId = createId(prefix)
    while (usedIds.has(nextId)) {
      nextId = createId(prefix)
    }
  }
  usedIds.add(nextId)
  return nextId
}

function normalizeWorkspaces(workspaces: WorkspaceCard[]) {
  return workspaces.map((workspace, workspaceIndex) => normalizeWorkspace(workspace, workspaceIndex))
}

function toPersistedWorkspaces(workspaces: WorkspaceCard[]) {
  return normalizeWorkspaces(workspaces).map((workspace) => ({
    ...workspace,
    providerProfiles: (workspace.providerProfiles || []).map((profile) => ({
      ...normalizeProviderProfile(profile, workspace.id, workspace.createdAt),
    })),
    providerQuotas: (workspace.providerQuotas || []).map((quota) => ({
      usdRemaining: quota.usdRemaining ?? null,
      requestsToday: quota.requestsToday ?? null,
      lastCheckedAt: quota.lastCheckedAt ?? null,
    })),
    providerUsageStats: (workspace.providerUsageStats || []).map((stats) => ({
      providerProfileId: stats.providerProfileId,
      summary: {
        totalRequests: stats.summary.totalRequests,
        totalCostUsd: stats.summary.totalCostUsd,
        totalInputTokens: stats.summary.totalInputTokens,
        totalOutputTokens: stats.summary.totalOutputTokens,
        totalCacheReadTokens: stats.summary.totalCacheReadTokens,
        totalCacheCreationTokens: stats.summary.totalCacheCreationTokens,
        cacheHitRate: stats.summary.cacheHitRate,
      },
      trends: (stats.trends || []).map((point) => ({ ...point })),
      requestLogs: (stats.requestLogs || []).map((log) => ({ ...log })),
    })),
    terminalEntries: workspace.terminalEntries.map((entry) => ({
      ...entry,
      status: 'idle' as const,
    })),
    tabs: workspace.tabs.map((tab) => ({
      ...tab,
      panes: tab.panes.map((pane) => stripRuntimePane(pane)),
    })),
    snapshots: normalizeSnapshots(workspace.snapshots ?? [], workspace.id, workspace.createdAt).slice(0, 8),
  }))
}

function snapshotTabs(tabs: WorkspaceTab[]) {
  return tabs.map((tab) => ({
    ...tab,
    panes: tab.panes.map((pane) => stripRuntimePane(pane)),
  }))
}

function stripRuntimePane(pane: PaneNode): PaneNode {
  const children = pane.children?.map((child) => stripRuntimePane(child)) || []
  const sessions = (pane.sessions || []).map((session) => ({
    ...session,
    status: 'idle' as const,
    aiCliKind: null,
  }))

  return {
    ...pane,
    activeSessionId: children.length
      ? null
      : (pane.activeSessionId && sessions.some((session) => session.id === pane.activeSessionId)
        ? pane.activeSessionId
        : sessions[0]?.id ?? null),
    sessions,
    children,
  }
}

function sanitizeGitBranch(branch?: string | null) {
  const value = (branch || '').trim()
  if (!value) return null
  if (/^[?？]+$/.test(value)) return null
  return value
}

function normalizeWorkspace(workspace: WorkspaceCard, workspaceIndex: number): WorkspaceCard {
  const now = new Date().toISOString()
  const workspaceId = workspace.id || createId('workspace')
  const createdAt = workspace.createdAt || now
  const updatedAt = workspace.updatedAt || createdAt
  const lastOpenedAt = workspace.lastOpenedAt || updatedAt
  const usedTabIds = new Set<string>()
  const usedPaneIds = new Set<string>()
  const usedSessionIds = new Set<string>()

  const terminalEntries = (workspace.terminalEntries || []).map((entry) => normalizeTerminalEntry(entry, workspaceId, createdAt))
  const draftWorkspace: WorkspaceCard = {
    ...(workspace as WorkspaceCard),
    id: workspaceId,
    providerProfiles: (workspace.providerProfiles || []).map((profile) => normalizeProviderProfile(profile, workspaceId, createdAt)),
  }
  const tabs = (workspace.tabs || []).map((tab, tabIndex) => normalizeTab(tab, draftWorkspace, tabIndex, createdAt, usedTabIds, usedPaneIds, usedSessionIds, terminalEntries))
  const snapshots = normalizeSnapshots(workspace.snapshots ?? [], workspaceId, createdAt)
  const defaultSnapshotId = workspace.defaultSnapshotId && snapshots.some((snapshot) => snapshot.id === workspace.defaultSnapshotId)
    ? workspace.defaultSnapshotId
    : snapshots[0]?.id ?? null

  return {
    id: workspaceId,
    name: workspace.name,
    description: workspace.description || '',
    rootPath: workspace.rootPath,
    gitBranch: sanitizeGitBranch(workspace.gitBranch),
    tags: workspace.tags || [],
    lastOpenedAt,
    createdAt,
    updatedAt,
    defaultSnapshotId,
    workspaceType: workspace.workspaceType ?? 'default',
    providerProfiles: draftWorkspace.providerProfiles,
    providerQuotas: (workspace.providerQuotas || []).map((quota) => ({
      usdRemaining: quota.usdRemaining ?? null,
      requestsToday: quota.requestsToday ?? null,
      lastCheckedAt: quota.lastCheckedAt ?? null,
    })),
    providerUsageStats: (workspace.providerUsageStats || []).map((stats) => ({
      providerProfileId: stats.providerProfileId,
      summary: {
        totalRequests: stats.summary?.totalRequests ?? 0,
        totalCostUsd: stats.summary?.totalCostUsd ?? 0,
        totalInputTokens: stats.summary?.totalInputTokens ?? 0,
        totalOutputTokens: stats.summary?.totalOutputTokens ?? 0,
        totalCacheReadTokens: stats.summary?.totalCacheReadTokens ?? 0,
        totalCacheCreationTokens: stats.summary?.totalCacheCreationTokens ?? 0,
        cacheHitRate: stats.summary?.cacheHitRate ?? 0,
      },
      trends: (stats.trends || []).map((point) => ({ ...point })),
      requestLogs: (stats.requestLogs || []).map((log) => ({ ...log })),
    })),
    terminalEntries,
    tabs,
    snapshots,
  }
}

function normalizeSnapshots(snapshots: WorkspaceSnapshot[], workspaceId: string, fallbackDate: string): WorkspaceSnapshot[] {
  const usedSnapshotIds = new Set<string>()

  return snapshots
    .map((snapshot, snapshotIndex) => normalizeSnapshot(snapshot, workspaceId, fallbackDate, snapshotIndex, usedSnapshotIds))
    .sort((left, right) => new Date(right.updatedAt).getTime() - new Date(left.updatedAt).getTime())
}

function normalizeSnapshot(
  snapshot: WorkspaceSnapshot,
  workspaceId: string,
  fallbackDate: string,
  snapshotIndex: number,
  usedSnapshotIds: Set<string>,
): WorkspaceSnapshot {
  const usedTabIds = new Set<string>()
  const usedPaneIds = new Set<string>()
  const usedSessionIds = new Set<string>()
  const createdAt = snapshot.createdAt || fallbackDate
  const updatedAt = snapshot.updatedAt || createdAt
  const snapshotWorkspace = {
    id: workspaceId,
    name: '',
    description: '',
    rootPath: '',
    tags: [],
    lastOpenedAt: createdAt,
    createdAt,
    updatedAt,
    tabs: [],
    terminalEntries: [],
    providerProfiles: [],
  } as WorkspaceCard
  const snapshotTerminalEntries: TerminalEntry[] = []
  const tabsState = (snapshot.tabsState || []).map((tab, tabIndex) =>
    normalizeTab(tab, snapshotWorkspace, tabIndex, createdAt, usedTabIds, usedPaneIds, usedSessionIds, snapshotTerminalEntries),
  )
  const validTabIds = new Set(tabsState.map((tab) => tab.id))
  const validPaneIds = new Set(tabsState.flatMap((tab) => flattenSnapshotPanes(tab.panes).map((pane) => pane.id)))
  const validSessionByPane = new Map<string, Set<string>>()

  tabsState.forEach((tab) => {
    flattenSnapshotPanes(tab.panes).forEach((pane) => {
      validSessionByPane.set(pane.id, new Set((pane.sessions || []).map((session) => session.id)))
    })
  })

  const activeTabId = snapshot.activeTabId && validTabIds.has(snapshot.activeTabId)
    ? snapshot.activeTabId
    : tabsState[0]?.id ?? null
  const activePaneId = snapshot.activePaneId && validPaneIds.has(snapshot.activePaneId)
    ? snapshot.activePaneId
    : flattenSnapshotPanes(tabsState.find((tab) => tab.id === activeTabId)?.panes ?? [])[0]?.id ?? null

  return {
    id: ensureUniqueId(snapshot.id, `snapshot-${snapshotIndex + 1}`, usedSnapshotIds),
    workspaceId: snapshot.workspaceId || workspaceId,
    name: snapshot.name || `布局快照 ${snapshotIndex + 1}`,
    tabsState,
    panesState: tabsState.reduce<Record<string, PaneNode[]>>((result, tab) => {
      result[tab.id] = tab.panes
      return result
    }, {}),
    activeTabId,
    activePaneId,
    activePaneIdsByTab: filterPaneMap(snapshot.activePaneIdsByTab ?? {}, validPaneIds),
    activeSessionIdsByPane: filterSessionMap(snapshot.activeSessionIdsByPane ?? {}, validSessionByPane),
    createdAt,
    updatedAt,
  }
}

function flattenSnapshotPanes(panes: PaneNode[]): PaneNode[] {
  return panes.flatMap((pane) => pane.children?.length ? flattenSnapshotPanes(pane.children) : [pane])
}

function filterPaneMap(source: Record<string, string>, validPaneIds: Set<string>) {
  return Object.fromEntries(Object.entries(source).filter(([, paneId]) => validPaneIds.has(paneId)))
}

function filterSessionMap(source: Record<string, string>, validSessionByPane: Map<string, Set<string>>) {
  return Object.fromEntries(Object.entries(source).filter(([paneId, sessionId]) => validSessionByPane.get(paneId)?.has(sessionId)))
}

function normalizeTerminalEntry(entry: TerminalEntry, workspaceId: string, fallbackDate: string): TerminalEntry {
  const commandHistory = Array.from(new Set((entry.commandHistory || []).map((item) => item.trim()).filter(Boolean))).slice(0, 20)
  const favoriteCommands = Array.from(new Set((entry.favoriteCommands || []).map((item) => item.trim()).filter(Boolean))).slice(0, 12)
  return {
    id: entry.id || createId('entry'),
    workspaceId: entry.workspaceId || workspaceId,
    name: entry.name,
    shellType: entry.shellType || 'pwsh7',
    workingDirectory: entry.workingDirectory,
    defaultCommand: entry.defaultCommand || '',
    lastCommand: entry.lastCommand || entry.defaultCommand || '',
    commandHistory,
    favoriteCommands,
    status: (entry.status || 'idle') as TerminalEntry['status'],
    launchMode: entry.launchMode || 'open-only',
    environmentVariablesText: entry.environmentVariablesText ?? null,
    runtimeNote: entry.runtimeNote ?? null,
    tags: entry.tags || [],
    note: entry.note ?? null,
    createdAt: entry.createdAt || fallbackDate,
    updatedAt: entry.updatedAt || entry.createdAt || fallbackDate,
  }
}

type LegacyProviderSource = ProviderProfileSource | 'openai' | 'anthropic' | 'gemini' | 'custom'

type LegacyProviderProfile = Omit<Partial<ProviderProfile>, 'providerKind' | 'managedBy'> & {
  providerKind?: ProviderProfile['providerKind'] | 'openai-compatible' | 'anthropic' | 'gemini' | 'custom'
  managedBy?: ProviderProfileSource
  apiFormat?: LegacyProviderSource
  baseUrl?: string
  apiKey?: string
}

function normalizeProviderKind(kind: LegacyProviderProfile['providerKind']): ProviderProfile['providerKind'] {
  if (kind === 'anthropic') return 'claude-code'
  if (kind === 'gemini') return 'gemini-cli'
  if (kind === 'openai-compatible') return 'codex'
  if (kind === 'custom') return 'custom-cli'
  if (kind === 'codex' || kind === 'claude-code' || kind === 'gemini-cli' || kind === 'opencode' || kind === 'custom-cli') return kind
  return 'codex'
}

function normalizeProviderSource(source: LegacyProviderSource | undefined): ProviderProfileSource {
  if (source === 'cc-switch' || source === 'oauth' || source === 'env' || source === 'script' || source === 'manual' || source === 'cli-config') return source
  if (source === 'anthropic' || source === 'openai' || source === 'gemini') return 'cli-config'
  return 'manual'
}

function normalizeProviderScope(scope: LegacyProviderProfile['configScope']): ProviderConfigScope {
  if (scope === 'workspace' || scope === 'project' || scope === 'global') return scope
  return 'global'
}

function normalizeProviderStatus(status: LegacyProviderProfile['status'], isActive?: boolean): ProviderProfileStatus {
  if (status === 'active' || status === 'available' || status === 'missing' || status === 'disabled') return status
  return isActive ? 'active' : 'available'
}

function normalizeProviderProfile(profile: LegacyProviderProfile, workspaceId: string, fallbackDate: string): ProviderProfile {
  const providerKind = normalizeProviderKind(profile.providerKind)
  const managedBy = normalizeProviderSource(profile.managedBy ?? profile.apiFormat)
  const isActive = Boolean(profile.isActive || profile.isDefault)
  const legacySource = profile.baseUrl?.trim() || ''
  const legacyAuth = profile.apiKey?.trim() || ''
  const migrationNote = legacySource || legacyAuth
    ? '旧版 API URL/Key 字段已迁移为本地 CLI 档案记录，Chuchen 不再保存请求地址或密钥。'
    : ''

  return {
    id: profile.id || createId('provider'),
    workspaceId: profile.workspaceId || workspaceId,
    name: profile.name || '本地 CLI 配置',
    providerKind,
    profileName: profile.profileName || profile.name || 'default',
    configPath: profile.configPath || legacySource || defaultConfigPathForProvider(providerKind),
    configScope: normalizeProviderScope(profile.configScope),
    managedBy,
    authSource: profile.authSource || (legacyAuth ? '旧版密钥字段已清空' : defaultAuthSourceForProvider(providerKind, managedBy)),
    switchCommand: profile.switchCommand || defaultSwitchCommandForProvider(providerKind, profile.profileName || profile.name || 'default'),
    defaultModel: profile.defaultModel || '',
    toolTargets: Array.from(new Set((profile.toolTargets || []).filter(Boolean))) as ProviderToolTarget[],
    status: normalizeProviderStatus(profile.status, isActive),
    isActive,
    lastDetectedAt: profile.lastDetectedAt ?? null,
    color: profile.color ?? null,
    note: profile.note || migrationNote || null,
    isDefault: Boolean(profile.isDefault),
    createdAt: profile.createdAt || fallbackDate,
    updatedAt: profile.updatedAt || profile.createdAt || fallbackDate,
  }
}

function defaultConfigPathForProvider(kind: ProviderProfile['providerKind']) {
  if (kind === 'codex') return '~/.codex/config.toml'
  if (kind === 'claude-code') return '~/.claude.json'
  if (kind === 'gemini-cli') return '~/.gemini/settings.json'
  if (kind === 'opencode') return '~/.config/opencode/opencode.json'
  return '本地 CLI 配置文件'
}

function defaultAuthSourceForProvider(kind: ProviderProfile['providerKind'], source: ProviderProfileSource) {
  if (source === 'oauth') return 'CLI OAuth 登录态'
  if (kind === 'claude-code') return 'Claude Code 本地配置'
  if (kind === 'gemini-cli') return 'Gemini CLI 登录态'
  return 'CLI 本地配置'
}

function defaultSwitchCommandForProvider(kind: ProviderProfile['providerKind'], profileName: string) {
  if (kind === 'codex') return `cc-switch codex use ${profileName}`
  if (kind === 'claude-code') return `cc-switch claude use ${profileName}`
  if (kind === 'gemini-cli') return `cc-switch gemini use ${profileName}`
  if (kind === 'opencode') return `cc-switch opencode use ${profileName}`
  return `cc-switch use ${profileName}`
}

function normalizeTab(
  tab: WorkspaceTab,
  workspace: WorkspaceCard,
  tabIndex: number,
  fallbackDate: string,
  usedTabIds: Set<string>,
  usedPaneIds: Set<string>,
  usedSessionIds: Set<string>,
  terminalEntries: TerminalEntry[],
): WorkspaceTab {
  const tabId = ensureUniqueId(tab.id, 'tab', usedTabIds)
  const normalizedPanes = (tab.panes || []).map((pane, paneIndex) => normalizePane(pane, { ...tab, id: tabId, workspaceId: workspace.id }, workspace, paneIndex, usedPaneIds, usedSessionIds, terminalEntries))
  const inferredPaneSequence = normalizedPanes.reduce((maxValue, pane) => {
    const match = /^Pane\s+(\d+)$/i.exec((pane.name || '').trim())
    if (!match) return maxValue
    return Math.max(maxValue, Number.parseInt(match[1] || '0', 10))
  }, 0)
  return {
    id: tabId,
    workspaceId: tab.workspaceId || workspace.id,
    name: tab.name,
    order: typeof tab.order === 'number' ? tab.order : tabIndex,
    layoutMode: tab.layoutMode || 'grid',
    paneSequence: Math.max(tab.paneSequence || 0, inferredPaneSequence),
    panes: normalizedPanes,
    createdAt: tab.createdAt || fallbackDate,
    updatedAt: tab.updatedAt || tab.createdAt || fallbackDate,
  }
}

function normalizePane(
  pane: PaneNode,
  tab: WorkspaceTab,
  workspace: WorkspaceCard,
  paneIndex: number,
  usedPaneIds: Set<string>,
  usedSessionIds: Set<string>,
  terminalEntries: TerminalEntry[],
): PaneNode {
  const paneId = ensureUniqueId(pane.id, 'pane', usedPaneIds)
  const normalizedPane = {
    ...pane,
    id: paneId,
    tabId: pane.tabId || tab.id,
    name: pane.name,
  }
  const sessions = normalizePaneSessions(normalizedPane, paneId, usedSessionIds, workspace, tab, terminalEntries)

  return {
    id: paneId,
    tabId: normalizedPane.tabId,
    name: normalizedPane.name,
    pathLabel: pane.pathLabel || '',
    terminalEntryId: pane.terminalEntryId ?? null,
    splitDirection: pane.splitDirection || 'none',
    parentPaneId: pane.parentPaneId ?? null,
    order: typeof pane.order === 'number' ? pane.order : paneIndex,
    sizeRatio: pane.sizeRatio || 1,
    activeSessionId: pane.activeSessionId && sessions.some((session) => session.id === pane.activeSessionId)
      ? pane.activeSessionId
      : sessions[0]?.id ?? null,
    sessions,
    children: pane.children?.map((child, childIndex) => normalizePane(child, tab, workspace, childIndex, usedPaneIds, usedSessionIds, terminalEntries)) || [],
  }
}

function normalizePaneSessions(
  pane: PaneNode,
  paneId: string,
  usedSessionIds: Set<string>,
  workspace: WorkspaceCard,
  tab: WorkspaceTab,
  terminalEntries: TerminalEntry[],
): PaneTerminalSession[] {
  if (pane.sessions?.length) {
    return pane.sessions.map((session, index) => {
      const nextId = ensureUniqueId(session.id, 'session', usedSessionIds)
      const inferredKind = session.lastAiCliKind
        ?? session.aiCliKind
        ?? inferPaneAiCliKind(workspace, tab, pane, session, terminalEntries)

      return {
        id: nextId,
        name: sanitizeSessionName(session.name, index === 0 ? (pane.name || 'Pane') : `${pane.name || 'Pane'} · ${index + 1}`),
        pathLabel: session.pathLabel || pane.pathLabel || '',
        terminalEntryId: session.terminalEntryId ?? pane.terminalEntryId ?? null,
        status: (session.status || 'idle') as PaneTerminalSession['status'],
        aiCliKind: session.status === 'running' ? (session.aiCliKind ?? inferredKind ?? null) : null,
        lastAiCliKind: inferredKind ?? null,
        hasUserCommand: Boolean(session.hasUserCommand),
        lastCommandAt: session.lastCommandAt ?? null,
        lastOutputAt: session.lastOutputAt ?? null,
        lastExitCode: session.lastExitCode ?? null,
        supervisorMode: session.supervisorMode ?? 'off',
        supervisorState: session.supervisorState ?? 'idle',
        expectedDoneSignal: session.expectedDoneSignal ?? null,
        lastHeartbeatAt: session.lastHeartbeatAt ?? null,
        lastActivityAt: session.lastActivityAt ?? null,
        supervisorNote: session.supervisorNote ?? null,
      }
    })
  }

  return [
    {
      id: ensureUniqueId(`${paneId}-session-main`, 'session', usedSessionIds),
      name: sanitizeSessionName(pane.name, '终端'),
      pathLabel: pane.pathLabel || '',
      terminalEntryId: pane.terminalEntryId ?? null,
      status: 'idle' as const,
      aiCliKind: null,
      lastAiCliKind: inferPaneAiCliKind(workspace, tab, pane, undefined, terminalEntries),
      hasUserCommand: false,
      lastCommandAt: null,
      lastOutputAt: null,
      lastExitCode: null,
      supervisorMode: 'off',
      supervisorState: 'idle',
      expectedDoneSignal: null,
      lastHeartbeatAt: null,
      lastActivityAt: null,
      supervisorNote: null,
    },
  ]
}


