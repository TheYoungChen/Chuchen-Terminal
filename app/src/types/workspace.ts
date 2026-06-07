export type AppSection = 'home' | 'workspace' | 'recent' | 'templates' | 'search' | 'settings'
export type WorkspaceView = 'overview' | 'detail' | 'runtime'
export type LaunchMode = 'open-only' | 'prefill' | 'execute' | 'switch-or-create'
export type SplitDirection = 'none' | 'horizontal' | 'vertical'
export type TabLayoutMode = 'grid' | 'horizontal' | 'vertical'
export type ShellType = 'pwsh7' | 'custom'
export type ProviderKind = 'openai-compatible' | 'anthropic' | 'gemini' | 'custom'
export type ProviderBindingMode = 'inherit' | 'explicit' | 'disabled'
export type ProviderToolTarget = 'claude' | 'codex' | 'gemini' | 'opencode' | 'generic'
export type ProviderApiFormat = 'openai' | 'anthropic' | 'gemini' | 'custom'
export type SupervisorMode = 'off' | 'watch' | 'auto-resume'
export type SupervisorState = 'idle' | 'watching' | 'stalled' | 'needs-human' | 'completed'
export type SessionAttentionState = 'fresh' | 'running' | 'waiting' | 'needs-input' | 'completed' | 'error' | 'stalled' | 'idle'

export interface PaneTerminalSession {
  id: string
  name: string
  pathLabel: string
  terminalEntryId: string | null
  status: 'idle' | 'running'
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
  baseUrl: string
  apiKey: string
  apiFormat: ProviderApiFormat
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  color?: string | null
  note?: string | null
  isDefault?: boolean
  createdAt: string
  updatedAt: string
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
  providerBindingMode?: ProviderBindingMode
  providerProfileId?: string | null
  modelId?: string | null
  environmentVariablesText?: string | null
  mcpPolicy?: 'inherit' | 'workspace' | 'none' | 'custom'
  skillPolicy?: 'inherit' | 'workspace' | 'none' | 'custom'
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
