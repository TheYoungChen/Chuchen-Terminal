import { workspaceMocks } from '../mocks/workspaces'
import type {
  PaneNode,
  PaneTerminalSession,
  ProviderProfile,
  ProviderToolTarget,
  TerminalEntry,
  WorkspaceCard,
  WorkspaceSnapshot,
  WorkspaceTab,
} from '../types/workspace'

const STORAGE_KEY = 'chuchen-terminal.workspace-data.v1'
let lastSavedPayload = ''

type WorkspaceStorageRecord = {
  version: 1
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
    version: 1,
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
  baseUrl: string
  apiKey: string
  apiFormat: ProviderProfile['apiFormat']
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  color?: string | null
  note?: string | null
  isDefault?: boolean
}): ProviderProfile {
  const now = new Date().toISOString()

  return {
    id: createId('provider'),
    workspaceId: input.workspaceId,
    name: input.name,
    providerKind: input.providerKind,
    baseUrl: input.baseUrl,
    apiKey: input.apiKey,
    apiFormat: input.apiFormat,
    defaultModel: input.defaultModel,
    toolTargets: input.toolTargets,
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
  providerBindingMode?: TerminalEntry['providerBindingMode']
  providerProfileId?: string | null
  modelId?: string | null
  environmentVariablesText?: string | null
  mcpPolicy?: TerminalEntry['mcpPolicy']
  skillPolicy?: TerminalEntry['skillPolicy']
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
    providerBindingMode: input.providerBindingMode ?? 'inherit',
    providerProfileId: input.providerProfileId ?? null,
    modelId: input.modelId ?? null,
    environmentVariablesText: input.environmentVariablesText ?? null,
    mcpPolicy: input.mcpPolicy ?? 'inherit',
    skillPolicy: input.skillPolicy ?? 'inherit',
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
  const tabs = (workspace.tabs || []).map((tab, tabIndex) => normalizeTab(tab, workspaceId, tabIndex, createdAt, usedTabIds, usedPaneIds, usedSessionIds))
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
    providerProfiles: (workspace.providerProfiles || []).map((profile) => normalizeProviderProfile(profile, workspaceId, createdAt)),
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
  const tabsState = (snapshot.tabsState || []).map((tab, tabIndex) => normalizeTab(tab, workspaceId, tabIndex, createdAt, usedTabIds, usedPaneIds, usedSessionIds))
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
    providerBindingMode: entry.providerBindingMode ?? 'inherit',
    providerProfileId: entry.providerProfileId ?? null,
    modelId: entry.modelId ?? null,
    environmentVariablesText: entry.environmentVariablesText ?? null,
    mcpPolicy: entry.mcpPolicy ?? 'inherit',
    skillPolicy: entry.skillPolicy ?? 'inherit',
    runtimeNote: entry.runtimeNote ?? null,
    tags: entry.tags || [],
    note: entry.note ?? null,
    createdAt: entry.createdAt || fallbackDate,
    updatedAt: entry.updatedAt || entry.createdAt || fallbackDate,
  }
}

function normalizeProviderProfile(profile: ProviderProfile, workspaceId: string, fallbackDate: string): ProviderProfile {
  return {
    id: profile.id || createId('provider'),
    workspaceId: profile.workspaceId || workspaceId,
    name: profile.name,
    providerKind: profile.providerKind || 'openai-compatible',
    baseUrl: profile.baseUrl || '',
    apiKey: profile.apiKey || '',
    apiFormat: profile.apiFormat || 'openai',
    defaultModel: profile.defaultModel || '',
    toolTargets: Array.from(new Set((profile.toolTargets || []).filter(Boolean))) as ProviderToolTarget[],
    color: profile.color ?? null,
    note: profile.note ?? null,
    isDefault: Boolean(profile.isDefault),
    createdAt: profile.createdAt || fallbackDate,
    updatedAt: profile.updatedAt || profile.createdAt || fallbackDate,
  }
}

function normalizeTab(
  tab: WorkspaceTab,
  workspaceId: string,
  tabIndex: number,
  fallbackDate: string,
  usedTabIds: Set<string>,
  usedPaneIds: Set<string>,
  usedSessionIds: Set<string>,
): WorkspaceTab {
  const tabId = ensureUniqueId(tab.id, 'tab', usedTabIds)
  const normalizedPanes = (tab.panes || []).map((pane, paneIndex) => normalizePane(pane, tabId, paneIndex, usedPaneIds, usedSessionIds))
  const inferredPaneSequence = normalizedPanes.reduce((maxValue, pane) => {
    const match = /^Pane\s+(\d+)$/i.exec((pane.name || '').trim())
    if (!match) return maxValue
    return Math.max(maxValue, Number.parseInt(match[1] || '0', 10))
  }, 0)
  return {
    id: tabId,
    workspaceId: tab.workspaceId || workspaceId,
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
  tabId: string,
  paneIndex: number,
  usedPaneIds: Set<string>,
  usedSessionIds: Set<string>,
): PaneNode {
  const paneId = ensureUniqueId(pane.id, 'pane', usedPaneIds)
  const sessions = normalizePaneSessions(pane, paneId, usedSessionIds)

  return {
    id: paneId,
    tabId: pane.tabId || tabId,
    name: pane.name,
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
    children: pane.children?.map((child, childIndex) => normalizePane(child, tabId, childIndex, usedPaneIds, usedSessionIds)) || [],
  }
}

function normalizePaneSessions(pane: PaneNode, paneId: string, usedSessionIds: Set<string>): PaneTerminalSession[] {
  if (pane.sessions?.length) {
    return pane.sessions.map((session, index) => {
      const nextId = ensureUniqueId(session.id, 'session', usedSessionIds)

      return {
        id: nextId,
        name: sanitizeSessionName(session.name, index === 0 ? (pane.name || 'Pane') : `${pane.name || 'Pane'} · ${index + 1}`),
        pathLabel: session.pathLabel || pane.pathLabel || '',
        terminalEntryId: session.terminalEntryId ?? pane.terminalEntryId ?? null,
        status: (session.status || 'idle') as PaneTerminalSession['status'],
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


