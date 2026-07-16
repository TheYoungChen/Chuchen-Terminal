import { invoke } from '@tauri-apps/api/core'
import type {
  ProviderConfigScope,
  ProviderProfile,
  ProviderProfileSource,
  ProviderProfileStatus,
  ProviderRequestLog,
  ProviderToolTarget,
  ProviderUsageStats,
  ProviderUsageTrendPoint,
} from '../types/workspace'
import { pickLocaleText } from './runtime-locale'

export type DetectedProviderRequestLog = Omit<ProviderRequestLog, 'providerProfileId'>

export interface DetectedProviderUsageStats {
  summary: ProviderUsageStats['summary']
  trends: ProviderUsageTrendPoint[]
  requestLogs: DetectedProviderRequestLog[]
}

export interface DetectedProviderProfile {
  /** 后端稳定身份键；可能短暂缺席（浏览器 fallback） */
  identityKey?: string
  providerKind: ProviderProfile['providerKind']
  name: string
  profileName: string
  configPath: string
  configScope: ProviderConfigScope
  managedBy: ProviderProfileSource
  authSource: string
  switchCommand: string
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  status: ProviderProfileStatus
  isActive: boolean
  color: string
  note: string
  homepageUrl?: string | null
  requestBaseUrl?: string | null
  configPayload?: string | null
  authPayload?: string | null
  detectedSource: string
  configExists: boolean
  usageStats?: DetectedProviderUsageStats
}

export interface AppliedProviderProfileResult {
  providerKind: ProviderProfile['providerKind']
  profileName: string
  appliedCommand: string
  stdout: string
  stderr: string
  requiresRestart: boolean
  restartHint: string
  affectedSessionCount: number
}

export async function detectLocalProviderProfiles(): Promise<DetectedProviderProfile[]> {
  try {
    const detected = await invoke<DetectedProviderProfile[]>('detect_local_provider_profiles')
    if (Array.isArray(detected) && detected.length) {
      return detected
    }
  } catch {
    // Browser-only dev mode cannot access the Tauri command. Keep a deterministic
    // preview so the Provider page remains testable without touching real files.
  }

  return fallbackDetectedProviderProfiles()
}

export async function readCurrentCodexProfile(): Promise<DetectedProviderProfile> {
  return invoke<DetectedProviderProfile>('read_current_codex_profile')
}

export async function readCurrentGeminiProfile(): Promise<DetectedProviderProfile> {
  return invoke<DetectedProviderProfile>('read_current_gemini_profile')
}

export async function readCurrentClaudeProfile(): Promise<DetectedProviderProfile> {
  return invoke<DetectedProviderProfile>('read_current_claude_profile')
}

export async function readCurrentHermesProfile(): Promise<DetectedProviderProfile> {
  return invoke<DetectedProviderProfile>('read_current_hermes_profile')
}

export async function applyProviderProfile(input: {
  providerKind: ProviderProfile['providerKind']
  profileName: string
  switchCommand?: string
  configPayload?: string | null
  authPayload?: string | null
}): Promise<AppliedProviderProfileResult> {
  return invoke<AppliedProviderProfileResult>('apply_provider_profile', {
    providerKind: input.providerKind,
    profileName: input.profileName,
    switchCommand: input.switchCommand ?? null,
    configPayload: input.configPayload ?? null,
    authPayload: input.authPayload ?? null,
  })
}

/** 后端已有的多维用量查询（native CLI + CC Switch proxy_request_logs） */
export interface ManagedUsageQueryFilters {
  appType?: string | null
  providerName?: string | null
  /** 对应 Profile.identityKey 列表；优先于 providerName */
  providerIds?: string[] | null
  model?: string | null
  dataSource?: string | null
  startAt?: string | null
  endAt?: string | null
  bucket?: string | null
  cursor?: string | null
  limit?: number | null
}

export interface ManagedUsagePricing {
  inputCostPerMillion: number
  outputCostPerMillion: number
  cacheReadCostPerMillion: number
  cacheCreationCostPerMillion: number
}

export interface ManagedUsageRequestLog {
  id: string
  /** 契约：对应 Profile.identityKey，前端不得拆解 */
  providerId: string
  providerName: string
  appType: string
  model: string
  pricingModel: string
  inputTokens: number
  outputTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  inputCostUsd: number
  outputCostUsd: number
  cacheReadCostUsd: number
  cacheCreationCostUsd: number
  totalCostUsd: number
  firstTokenMs?: number | null
  durationMs?: number | null
  statusCode: number
  dataSource: string
  createdAt: string
  pricing?: ManagedUsagePricing | null
}

export interface ManagedUsageTrendPoint {
  timestamp: string
  requestCount: number
  inputTokens: number
  outputTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  costUsd: number
}

export interface ManagedUsageProviderStat {
  providerId: string
  providerName: string
  appType: string
  totalRequests: number
  totalCostUsd: number
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheReadTokens: number
  totalCacheCreationTokens: number
  cacheHitRate: number
  modelCount: number
  models: string[]
  dataSources: string[]
  lastActivityAt: string
}

/** 后端全量模型聚合（含 rollup）；不受 requestLogs limit/cursor 影响 */
export interface ManagedUsageModelStat {
  model: string
  totalRequests: number
  totalCostUsd: number
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheReadTokens: number
  totalCacheCreationTokens: number
  providerCount: number
  providerNames: string[]
  appTypes: string[]
  lastActivityAt: string
}

export interface ManagedUsageQueryResult {
  summary: ProviderUsageStats['summary']
  channels: Array<{
    channelId: string
    appType: string
    dataSource: string
    totalRequests: number
    totalCostUsd: number
    totalInputTokens: number
    totalOutputTokens: number
  }>
  trends: ManagedUsageTrendPoint[]
  providerStats: ManagedUsageProviderStat[]
  /** 全量模型排行；前端禁止用分页 logs 重算 KPI */
  modelStats: ManagedUsageModelStat[]
  requestLogs: ManagedUsageRequestLog[]
  nextCursor?: string | null
  hasMore?: boolean
  total?: number
  updatedAt: string
}

export async function queryManagedUsage(filters: ManagedUsageQueryFilters = {}): Promise<ManagedUsageQueryResult> {
  return invoke<ManagedUsageQueryResult>('query_managed_usage', {
    filters: {
      appType: filters.appType ?? null,
      providerName: filters.providerName ?? null,
      // 保留空数组语义：有些调用方用 [] 表示“已选但无有效 identityKey”，不能再折叠成 null（全量）
      providerIds: filters.providerIds === undefined ? null : filters.providerIds,
      model: filters.model ?? null,
      dataSource: filters.dataSource ?? null,
      startAt: filters.startAt ?? null,
      endAt: filters.endAt ?? null,
      bucket: filters.bucket ?? null,
      cursor: filters.cursor ?? null,
      limit: filters.limit ?? null,
    },
  })
}

export function fallbackDetectedProviderProfiles(): DetectedProviderProfile[] {
  return [
    createFallbackDetection({
      providerKind: 'codex',
      name: pickLocaleText('Codex CLI 本地配置', 'Codex CLI local config'),
      profileName: 'default',
      configPath: '~/.codex/config.toml',
      defaultModel: 'gpt-5',
      toolTargets: ['codex'],
      color: '#4b83ff',
    }),
    createFallbackDetection({
      providerKind: 'claude-code',
      name: pickLocaleText('Claude Code 本地配置', 'Claude Code local config'),
      profileName: 'default',
      configPath: '~/.claude.json',
      defaultModel: 'claude-sonnet-4',
      toolTargets: ['claude'],
      color: '#d97706',
    }),
    createFallbackDetection({
      providerKind: 'gemini-cli',
      name: pickLocaleText('Gemini CLI 本地配置', 'Gemini CLI local config'),
      profileName: 'default',
      configPath: '~/.gemini/settings.json',
      defaultModel: 'gemini-2.5-pro',
      toolTargets: ['gemini'],
      color: '#0f9f6e',
    }),
    createFallbackDetection({
      providerKind: 'opencode',
      name: pickLocaleText('OpenCode 本地配置', 'OpenCode local config'),
      profileName: 'default',
      configPath: '~/.config/opencode/opencode.json',
      defaultModel: '',
      toolTargets: ['opencode'],
      color: '#2563eb',
    }),
    createFallbackDetection({
      providerKind: 'hermes',
      name: pickLocaleText('Hermes 本地配置', 'Hermes local config'),
      profileName: 'default',
      configPath: '~/.hermes/config.yaml',
      defaultModel: 'anthropic/claude-opus-4-8',
      // Hermes 独立 kind；不伪装成 Codex
      toolTargets: ['generic'],
      color: '#0891b2',
    }),
  ]
}

function createFallbackDetection(input: {
  providerKind: ProviderProfile['providerKind']
  name: string
  profileName: string
  configPath: string
  defaultModel: string
  toolTargets: ProviderToolTarget[]
  color: string
}): DetectedProviderProfile {
  return {
    identityKey: `browser:${input.providerKind}:${input.profileName}`,
    providerKind: input.providerKind,
    name: input.name,
    profileName: input.profileName,
    configPath: input.configPath,
    configScope: 'global',
    managedBy: 'manual',
    authSource: pickLocaleText('浏览器预览模式，未读取本机配置', 'Browser preview mode, local config not read'),
    switchCommand: '',
    defaultModel: input.defaultModel,
    toolTargets: input.toolTargets,
    status: 'missing',
    isActive: false,
    color: input.color,
    note: pickLocaleText('当前页面运行在浏览器模式，无法直接扫描用户目录；Tauri 桌面模式会只读检测真实配置文件。', 'This page is running in browser mode, so user directories cannot be scanned directly; Tauri desktop mode will read real config files in read-only mode.'),
    homepageUrl: null,
    requestBaseUrl: null,
    configPayload: null,
    authPayload: null,
    detectedSource: 'browser-preview',
    configExists: false,
  }
}
