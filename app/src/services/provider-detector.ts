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

export type DetectedProviderRequestLog = Omit<ProviderRequestLog, 'providerProfileId'>

export interface DetectedProviderUsageStats {
  summary: ProviderUsageStats['summary']
  trends: ProviderUsageTrendPoint[]
  requestLogs: DetectedProviderRequestLog[]
}

export interface DetectedProviderProfile {
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
  detectedSource: string
  configExists: boolean
  usageStats?: DetectedProviderUsageStats
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

export function fallbackDetectedProviderProfiles(): DetectedProviderProfile[] {
  return [
    createFallbackDetection({
      providerKind: 'codex',
      name: 'Codex CLI 本地配置',
      profileName: 'default',
      configPath: '~/.codex/config.toml',
      defaultModel: 'gpt-5',
      toolTargets: ['codex'],
      color: '#4b83ff',
    }),
    createFallbackDetection({
      providerKind: 'claude-code',
      name: 'Claude Code 本地配置',
      profileName: 'default',
      configPath: '~/.claude.json',
      defaultModel: 'claude-sonnet-4',
      toolTargets: ['claude'],
      color: '#d97706',
    }),
    createFallbackDetection({
      providerKind: 'gemini-cli',
      name: 'Gemini CLI 本地配置',
      profileName: 'default',
      configPath: '~/.gemini/settings.json',
      defaultModel: 'gemini-2.5-pro',
      toolTargets: ['gemini'],
      color: '#0f9f6e',
    }),
    createFallbackDetection({
      providerKind: 'opencode',
      name: 'OpenCode 本地配置',
      profileName: 'default',
      configPath: '~/.config/opencode/opencode.json',
      defaultModel: '',
      toolTargets: ['opencode'],
      color: '#2563eb',
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
    providerKind: input.providerKind,
    name: input.name,
    profileName: input.profileName,
    configPath: input.configPath,
    configScope: 'global',
    managedBy: 'manual',
    authSource: '浏览器预览模式，未读取本机配置',
    switchCommand: fallbackSwitchCommand(input.providerKind, input.profileName),
    defaultModel: input.defaultModel,
    toolTargets: input.toolTargets,
    status: 'missing',
    isActive: false,
    color: input.color,
    note: '当前页面运行在浏览器模式，无法直接扫描用户目录；Tauri 桌面模式会只读检测真实配置文件。',
    detectedSource: 'browser-preview',
    configExists: false,
  }
}

function fallbackSwitchCommand(kind: ProviderProfile['providerKind'], profileName: string) {
  if (kind === 'claude-code') return `cc-switch claude use ${profileName}`
  if (kind === 'gemini-cli') return `cc-switch gemini use ${profileName}`
  if (kind === 'opencode') return `cc-switch opencode use ${profileName}`
  if (kind === 'custom-cli') return `cc-switch use ${profileName}`
  return `cc-switch codex use ${profileName}`
}
