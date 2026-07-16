export type AppSection = 'home' | 'workspace' | 'recent' | 'templates' | 'providers' | 'usage' | 'search' | 'settings'
export type WorkspaceView = 'overview' | 'detail' | 'runtime'
export type LaunchMode = 'open-only' | 'prefill' | 'execute' | 'switch-or-create'
export type SplitDirection = 'none' | 'horizontal' | 'vertical'
export type TabLayoutMode = 'grid' | 'horizontal' | 'vertical'
export type ShellType = 'pwsh7' | 'custom'
export type ProviderKind = 'codex' | 'claude-code' | 'gemini-cli' | 'deepseek-cli' | 'opencode' | 'hermes' | 'custom-cli'
export type ProviderToolTarget = 'claude' | 'codex' | 'gemini' | 'opencode' | 'generic'
export type ProviderProfileSource = 'cli-config' | 'cc-switch' | 'oauth' | 'env' | 'script' | 'manual'
export type ProviderConfigScope = 'global' | 'workspace' | 'project'
export type ProviderProfileStatus = 'active' | 'available' | 'missing' | 'disabled'
export type SupervisorMode = 'off' | 'watch' | 'auto-resume'
export type SupervisorState = 'idle' | 'watching' | 'stalled' | 'needs-human' | 'completed'
export type SessionAttentionState = 'fresh' | 'running' | 'waiting' | 'needs-input' | 'completed' | 'error' | 'stalled' | 'idle'
export type AiCliKind = 'codex' | 'claude-code' | 'gemini-cli' | 'deepseek-cli' | 'opencode' | 'custom-cli' | 'generic-ai'

export interface PaneTerminalSession {
  id: string
  name: string
  pathLabel: string
  terminalEntryId: string | null
  status: 'idle' | 'running'
  aiCliKind?: AiCliKind | null
  lastAiCliKind?: AiCliKind | null
  hasUserCommand?: boolean
  lastCommandAt?: string | null
  lastOutputAt?: string | null
  lastExitCode?: number | null
  supervisorMode?: SupervisorMode
  supervisorState?: SupervisorState
  expectedDoneSignal?: string | null
  lastHeartbeatAt?: string | null
  lastActivityAt?: string | null
  supervisorNote?: string | null
}

export interface ProviderProfile {
  id: string
  workspaceId: string
  name: string
  providerKind: ProviderKind
  profileName: string
  configPath: string
  configScope: ProviderConfigScope
  managedBy: ProviderProfileSource
  authSource: string
  switchCommand: string
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  status: ProviderProfileStatus
  isActive?: boolean
  /** 后端 detect 返回的稳定身份键；导入/刷新优先按此匹配，避免同名多档串档 */
  identityKey?: string | null
  lastDetectedAt?: string | null
  color?: string | null
  note?: string | null
  homepageUrl?: string | null
  requestBaseUrl?: string | null
  configPayload?: string | null
  authPayload?: string | null
  isDefault?: boolean
  createdAt: string
  updatedAt: string
}

export interface ProviderQuotaSnapshot {
  providerProfileId: string
  usdRemaining?: number | null
  requestsToday?: number | null
  lastCheckedAt?: string | null
}

export interface ProviderUsageStats {
  providerProfileId: string
  summary: ProviderUsageSummary
  trends: ProviderUsageTrendPoint[]
  requestLogs: ProviderRequestLog[]
}

export interface ProviderUsageSummary {
  totalRequests: number
  totalCostUsd: number
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheReadTokens: number
  totalCacheCreationTokens: number
  cacheHitRate: number
}

export interface ProviderUsageTrendPoint {
  timestamp: string
  inputTokens: number
  outputTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  costUsd: number
}

export type ProviderUsageAppType = 'claude' | 'codex' | 'gemini' | 'opencode' | 'hermes'

export interface ProviderRequestLog {
  id: string
  providerProfileId: string
  appType: ProviderUsageAppType
  model: string
  inputTokens: number
  outputTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  costUsd: number
  /** 官方分项计费（来自 query_managed_usage；无则仅用 costUsd） */
  inputCostUsd?: number
  outputCostUsd?: number
  cacheReadCostUsd?: number
  cacheCreationCostUsd?: number
  /** 首 token 延迟（毫秒）；native/部分 proxy 可能缺省 */
  firstTokenMs?: number | null
  statusCode: number
  durationMs: number
  dataSource: string
  createdAt: string
  /**
   * 后端 ManagedUsage 的 providerId，契约上等于 Profile.identityKey。
   * 仅用于关联展示，禁止前端拆解/推断。
   */
  managedProviderId?: string
  /** 后端返回的 providerName，未匹配到本地 Profile 时作展示回退 */
  managedProviderName?: string
  pricingModel?: string
}

export interface TerminalEntry {
  id: string
  workspaceId: string
  name: string
  shellType: ShellType
  workingDirectory: string
  defaultCommand: string
  lastCommand: string
  commandHistory?: string[]
  favoriteCommands?: string[]
  status: 'idle' | 'running'
  launchMode: LaunchMode
  environmentVariablesText?: string | null
  runtimeNote?: string | null
  tags: string[]
  note?: string | null
  createdAt: string
  updatedAt: string
}

export interface PaneNode {
  id: string
  tabId: string
  name: string
  pathLabel: string
  terminalEntryId: string | null
  splitDirection: SplitDirection
  parentPaneId?: string | null
  order: number
  sizeRatio: number
  activeSessionId?: string | null
  sessions?: PaneTerminalSession[]
  children?: PaneNode[]
}

export interface WorkspaceTab {
  id: string
  workspaceId: string
  name: string
  order: number
  layoutMode?: TabLayoutMode
  paneSequence?: number
  panes: PaneNode[]
  createdAt: string
  updatedAt: string
}

export interface WorkspaceSnapshot {
  id: string
  workspaceId: string
  name: string
  tabsState: WorkspaceTab[]
  panesState?: Record<string, PaneNode[]> | null
  activeTabId: string | null
  activePaneId: string | null
  activePaneIdsByTab?: Record<string, string>
  activeSessionIdsByPane?: Record<string, string>
  createdAt: string
  updatedAt: string
}

export interface WorkspaceCard {
  id: string
  name: string
  description: string
  rootPath: string
  gitBranch?: string | null
  tags: string[]
  lastOpenedAt: string
  createdAt: string
  updatedAt: string
  providerProfiles?: ProviderProfile[]
  providerQuotas?: ProviderQuotaSnapshot[]
  providerUsageStats?: ProviderUsageStats[]
  tabs: WorkspaceTab[]
  terminalEntries: TerminalEntry[]
  snapshots?: WorkspaceSnapshot[]
  defaultSnapshotId?: string | null
  workspaceType?: string
}

export type WorkflowTemplateKind = 'system' | 'user'
export type WorkflowTemplateCategory = 'ai-cli' | 'frontend' | 'backend' | 'fullstack' | 'custom'

export interface WorkflowTemplatePane {
  name: string
  shellType: ShellType
  workingDirectoryHint: string
  defaultCommand: string
  tags: string[]
}

export interface WorkflowTemplate {
  id: string
  kind: WorkflowTemplateKind
  category: WorkflowTemplateCategory
  name: string
  description: string
  tags: string[]
  panes: WorkflowTemplatePane[]
  createdAt: string
  updatedAt: string
}
