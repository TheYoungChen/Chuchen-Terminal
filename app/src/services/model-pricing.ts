/** 模型价格配置
 *
 * 对齐 CC Switch model_pricing 字段主结构 + NewAPI 扩展字段（图/视频/音频预留）。
 * 桌面态：后端 get/update/delete_model_pricing（~/.chuchen-terminal/usage.db）为 SSOT。
 * 浏览器 / 无 Tauri：回退 localStorage，仅便于 UI 预览。
 */

import { invoke } from '@tauri-apps/api/core'

export type ModelPricingVendor =
  | 'claude'
  | 'openai'
  | 'gemini'
  | 'grok'
  | 'deepseek'
  | 'glm'
  | 'minimax'
  | 'kimi'
  | 'qwen'
  | 'other'

export type ModelPriceUnit = 'per_million' | 'per_thousand'

export interface ModelPricingEntry {
  modelId: string
  displayName: string
  vendor: ModelPricingVendor
  /** 统一内部存 per_million USD */
  inputCostPerMillion: number
  outputCostPerMillion: number
  cacheReadCostPerMillion: number
  cacheCreationCostPerMillion: number
  /** NewAPI 风格扩展，默认 0；当前计费未用 */
  completionCostPerMillion: number
  imageCostPerMillion: number
  videoCostPerMillion: number
  audioCostPerMillion: number
  /** seed = 内置；user = 用户新增/改过 */
  source: 'seed' | 'user'
  updatedAt: string
}

export interface ModelPricingDraft {
  modelId: string
  displayName: string
  vendor: ModelPricingVendor
  unit: ModelPriceUnit
  inputCost: number
  outputCost: number
  cacheReadCost: number
  cacheCreationCost: number
  completionCost: number
  imageCost: number
  videoCost: number
  audioCost: number
}

const STORAGE_KEY = 'chuchen.model-pricing.v1'

type SeedRow = {
  modelId: string
  displayName: string
  vendor: ModelPricingVendor
  inputCostPerMillion: number
  outputCostPerMillion: number
  cacheReadCostPerMillion: number
  cacheCreationCostPerMillion: number
}

const SEED: SeedRow[] = [
  { modelId: 'claude-opus-4-8', displayName: 'Claude Opus 4.8', vendor: 'claude', inputCostPerMillion: 5, outputCostPerMillion: 25, cacheReadCostPerMillion: 0.5, cacheCreationCostPerMillion: 6.25 },
  { modelId: 'claude-opus-4-6', displayName: 'Claude Opus 4.6', vendor: 'claude', inputCostPerMillion: 5, outputCostPerMillion: 25, cacheReadCostPerMillion: 0.5, cacheCreationCostPerMillion: 6.25 },
  { modelId: 'claude-sonnet-4-6', displayName: 'Claude Sonnet 4.6', vendor: 'claude', inputCostPerMillion: 3, outputCostPerMillion: 15, cacheReadCostPerMillion: 0.3, cacheCreationCostPerMillion: 3.75 },
  { modelId: 'claude-sonnet-4-5', displayName: 'Claude Sonnet 4.5', vendor: 'claude', inputCostPerMillion: 3, outputCostPerMillion: 15, cacheReadCostPerMillion: 0.3, cacheCreationCostPerMillion: 3.75 },
  { modelId: 'claude-haiku-4-5', displayName: 'Claude Haiku 4.5', vendor: 'claude', inputCostPerMillion: 1, outputCostPerMillion: 5, cacheReadCostPerMillion: 0.1, cacheCreationCostPerMillion: 1.25 },
  { modelId: 'gpt-5.5', displayName: 'GPT-5.5', vendor: 'openai', inputCostPerMillion: 5, outputCostPerMillion: 30, cacheReadCostPerMillion: 0.5, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-5.4', displayName: 'GPT-5.4', vendor: 'openai', inputCostPerMillion: 2.5, outputCostPerMillion: 15, cacheReadCostPerMillion: 0.25, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-5.4-mini', displayName: 'GPT-5.4 Mini', vendor: 'openai', inputCostPerMillion: 0.75, outputCostPerMillion: 4.5, cacheReadCostPerMillion: 0.075, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-5.4-nano', displayName: 'GPT-5.4 Nano', vendor: 'openai', inputCostPerMillion: 0.2, outputCostPerMillion: 1.25, cacheReadCostPerMillion: 0.02, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-5.2', displayName: 'GPT-5.2', vendor: 'openai', inputCostPerMillion: 1.75, outputCostPerMillion: 14, cacheReadCostPerMillion: 0.175, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-5.2-codex', displayName: 'GPT-5.2 Codex', vendor: 'openai', inputCostPerMillion: 1.75, outputCostPerMillion: 14, cacheReadCostPerMillion: 0.175, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-4o', displayName: 'GPT-4o', vendor: 'openai', inputCostPerMillion: 2.5, outputCostPerMillion: 10, cacheReadCostPerMillion: 1.25, cacheCreationCostPerMillion: 0 },
  { modelId: 'gpt-4o-mini', displayName: 'GPT-4o Mini', vendor: 'openai', inputCostPerMillion: 0.15, outputCostPerMillion: 0.6, cacheReadCostPerMillion: 0.075, cacheCreationCostPerMillion: 0 },
  { modelId: 'gemini-3.5-flash', displayName: 'Gemini 3.5 Flash', vendor: 'gemini', inputCostPerMillion: 1.5, outputCostPerMillion: 9, cacheReadCostPerMillion: 0.15, cacheCreationCostPerMillion: 0 },
  { modelId: 'gemini-3-pro-preview', displayName: 'Gemini 3 Pro Preview', vendor: 'gemini', inputCostPerMillion: 2, outputCostPerMillion: 12, cacheReadCostPerMillion: 0.2, cacheCreationCostPerMillion: 0 },
  { modelId: 'gemini-2.5-pro', displayName: 'Gemini 2.5 Pro', vendor: 'gemini', inputCostPerMillion: 1.25, outputCostPerMillion: 10, cacheReadCostPerMillion: 0.125, cacheCreationCostPerMillion: 0 },
  { modelId: 'gemini-2.5-flash', displayName: 'Gemini 2.5 Flash', vendor: 'gemini', inputCostPerMillion: 0.3, outputCostPerMillion: 2.5, cacheReadCostPerMillion: 0.03, cacheCreationCostPerMillion: 0 },
  { modelId: 'gemini-2.5-flash-lite', displayName: 'Gemini 2.5 Flash-Lite', vendor: 'gemini', inputCostPerMillion: 0.1, outputCostPerMillion: 0.4, cacheReadCostPerMillion: 0.01, cacheCreationCostPerMillion: 0 },
  { modelId: 'grok-4', displayName: 'Grok 4', vendor: 'grok', inputCostPerMillion: 3, outputCostPerMillion: 15, cacheReadCostPerMillion: 0.75, cacheCreationCostPerMillion: 0 },
  { modelId: 'grok-3', displayName: 'Grok 3', vendor: 'grok', inputCostPerMillion: 3, outputCostPerMillion: 15, cacheReadCostPerMillion: 0.75, cacheCreationCostPerMillion: 0 },
  { modelId: 'grok-3-mini', displayName: 'Grok 3 Mini', vendor: 'grok', inputCostPerMillion: 0.3, outputCostPerMillion: 0.5, cacheReadCostPerMillion: 0.075, cacheCreationCostPerMillion: 0 },
  { modelId: 'deepseek-v4-pro', displayName: 'DeepSeek V4 Pro', vendor: 'deepseek', inputCostPerMillion: 1.0, outputCostPerMillion: 2.0, cacheReadCostPerMillion: 0.2, cacheCreationCostPerMillion: 0 },
  { modelId: 'deepseek-chat', displayName: 'DeepSeek Chat', vendor: 'deepseek', inputCostPerMillion: 0.27, outputCostPerMillion: 1.1, cacheReadCostPerMillion: 0.07, cacheCreationCostPerMillion: 0 },
  { modelId: 'deepseek-reasoner', displayName: 'DeepSeek Reasoner', vendor: 'deepseek', inputCostPerMillion: 0.55, outputCostPerMillion: 2.19, cacheReadCostPerMillion: 0.14, cacheCreationCostPerMillion: 0 },
  { modelId: 'glm-5.2', displayName: 'GLM-5.2', vendor: 'glm', inputCostPerMillion: 1.4, outputCostPerMillion: 4.4, cacheReadCostPerMillion: 0.26, cacheCreationCostPerMillion: 0 },
  { modelId: 'glm-5.1', displayName: 'GLM-5.1', vendor: 'glm', inputCostPerMillion: 1.4, outputCostPerMillion: 4.4, cacheReadCostPerMillion: 0.26, cacheCreationCostPerMillion: 0 },
  { modelId: 'glm-4.6', displayName: 'GLM-4.6', vendor: 'glm', inputCostPerMillion: 0.6, outputCostPerMillion: 2.2, cacheReadCostPerMillion: 0.11, cacheCreationCostPerMillion: 0 },
  { modelId: 'minimax-m2.5', displayName: 'MiniMax M2.5', vendor: 'minimax', inputCostPerMillion: 0.3, outputCostPerMillion: 1.2, cacheReadCostPerMillion: 0.03, cacheCreationCostPerMillion: 0 },
  { modelId: 'kimi-k2.5', displayName: 'Kimi K2.5', vendor: 'kimi', inputCostPerMillion: 0.6, outputCostPerMillion: 2.5, cacheReadCostPerMillion: 0.15, cacheCreationCostPerMillion: 0 },
  { modelId: 'qwen3-max', displayName: 'Qwen3 Max', vendor: 'qwen', inputCostPerMillion: 1.2, outputCostPerMillion: 6.0, cacheReadCostPerMillion: 0.3, cacheCreationCostPerMillion: 0 },
]

export const MODEL_PRICING_VENDORS: Array<{ id: ModelPricingVendor | 'all'; labelZh: string; labelEn: string }> = [
  { id: 'all', labelZh: '全部厂商', labelEn: 'All vendors' },
  { id: 'claude', labelZh: 'Claude', labelEn: 'Claude' },
  { id: 'openai', labelZh: 'OpenAI / GPT', labelEn: 'OpenAI / GPT' },
  { id: 'gemini', labelZh: 'Gemini', labelEn: 'Gemini' },
  { id: 'grok', labelZh: 'Grok', labelEn: 'Grok' },
  { id: 'deepseek', labelZh: 'DeepSeek', labelEn: 'DeepSeek' },
  { id: 'glm', labelZh: 'GLM', labelEn: 'GLM' },
  { id: 'minimax', labelZh: 'MiniMax', labelEn: 'MiniMax' },
  { id: 'kimi', labelZh: 'Kimi', labelEn: 'Kimi' },
  { id: 'qwen', labelZh: 'Qwen', labelEn: 'Qwen' },
  { id: 'other', labelZh: '其他', labelEn: 'Other' },
]

function nowIso() {
  return new Date().toISOString()
}

function num(value: unknown) {
  const n = typeof value === 'number' ? value : Number(value)
  if (!Number.isFinite(n) || n < 0) return 0
  return n
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export function inferVendor(modelId: string): ModelPricingVendor {
  const id = modelId.toLowerCase()
  if (id.includes('claude')) return 'claude'
  if (id.includes('gpt') || id.includes('o1') || id.includes('o3') || id.includes('o4') || id.startsWith('openai')) return 'openai'
  if (id.includes('gemini')) return 'gemini'
  if (id.includes('grok')) return 'grok'
  if (id.includes('deepseek')) return 'deepseek'
  if (id.includes('glm') || id.includes('chatglm')) return 'glm'
  if (id.includes('minimax') || id.includes('abab')) return 'minimax'
  if (id.includes('kimi') || id.includes('moonshot')) return 'kimi'
  if (id.includes('qwen') || id.includes('qwq')) return 'qwen'
  return 'other'
}

function normalizeVendor(value: unknown, modelId: string): ModelPricingVendor {
  const raw = String(value || '').trim().toLowerCase()
  const allowed: ModelPricingVendor[] = [
    'claude', 'openai', 'gemini', 'grok', 'deepseek', 'glm', 'minimax', 'kimi', 'qwen', 'other',
  ]
  if (allowed.includes(raw as ModelPricingVendor)) return raw as ModelPricingVendor
  return inferVendor(modelId)
}

function normalizeEntry(raw: Partial<ModelPricingEntry> & Pick<ModelPricingEntry, 'modelId'>): ModelPricingEntry {
  const modelId = String(raw.modelId || '').trim()
  return {
    modelId,
    displayName: String(raw.displayName || modelId).trim() || modelId,
    vendor: normalizeVendor(raw.vendor, modelId),
    inputCostPerMillion: num(raw.inputCostPerMillion),
    outputCostPerMillion: num(raw.outputCostPerMillion),
    cacheReadCostPerMillion: num(raw.cacheReadCostPerMillion),
    cacheCreationCostPerMillion: num(raw.cacheCreationCostPerMillion),
    completionCostPerMillion: num(raw.completionCostPerMillion),
    imageCostPerMillion: num(raw.imageCostPerMillion),
    videoCostPerMillion: num(raw.videoCostPerMillion),
    audioCostPerMillion: num(raw.audioCostPerMillion),
    source: raw.source === 'user' ? 'user' : 'seed',
    updatedAt: raw.updatedAt || nowIso(),
  }
}

function seedEntries(): ModelPricingEntry[] {
  return SEED.map((item) => normalizeEntry({
    ...item,
    completionCostPerMillion: 0,
    imageCostPerMillion: 0,
    videoCostPerMillion: 0,
    audioCostPerMillion: 0,
    source: 'seed',
    updatedAt: '1970-01-01T00:00:00.000Z',
  }))
}

function readLocalOverrides(): ModelPricingEntry[] {
  if (typeof window === 'undefined' || !window.localStorage) return []
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter((item): item is Partial<ModelPricingEntry> & { modelId: string } => Boolean(item && typeof item === 'object' && (item as { modelId?: unknown }).modelId))
      .map((item) => normalizeEntry({ ...item, source: 'user' }))
  } catch {
    return []
  }
}

function writeLocalOverrides(entries: ModelPricingEntry[]) {
  if (typeof window === 'undefined' || !window.localStorage) return
  const userOnly = entries.filter((item) => item.source === 'user')
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(userOnly))
}

function sortEntries(entries: ModelPricingEntry[]) {
  return [...entries].sort((a, b) => {
    if (a.vendor !== b.vendor) return a.vendor.localeCompare(b.vendor)
    return a.displayName.localeCompare(b.displayName)
  })
}

function mergeLocalEntries(): ModelPricingEntry[] {
  const map = new Map<string, ModelPricingEntry>()
  for (const seed of seedEntries()) map.set(seed.modelId.toLowerCase(), seed)
  for (const override of readLocalOverrides()) map.set(override.modelId.toLowerCase(), override)
  return sortEntries(Array.from(map.values()))
}

function normalizeRemoteEntries(raw: unknown): ModelPricingEntry[] {
  if (!Array.isArray(raw)) return []
  return sortEntries(
    raw
      .filter((item): item is Record<string, unknown> => Boolean(item && typeof item === 'object'))
      .map((item) => normalizeEntry({
        modelId: String(item.modelId ?? ''),
        displayName: String(item.displayName ?? ''),
        vendor: item.vendor as ModelPricingVendor,
        inputCostPerMillion: num(item.inputCostPerMillion),
        outputCostPerMillion: num(item.outputCostPerMillion),
        cacheReadCostPerMillion: num(item.cacheReadCostPerMillion),
        cacheCreationCostPerMillion: num(item.cacheCreationCostPerMillion),
        completionCostPerMillion: num(item.completionCostPerMillion),
        imageCostPerMillion: num(item.imageCostPerMillion),
        videoCostPerMillion: num(item.videoCostPerMillion),
        audioCostPerMillion: num(item.audioCostPerMillion),
        source: item.source === 'user' ? 'user' : 'seed',
        updatedAt: String(item.updatedAt || nowIso()),
      }))
      .filter((item) => item.modelId),
  )
}

/** 同步读取：仅 localStorage 兜底（浏览器预览 / 弹窗初始空态） */
export function listModelPricing(): ModelPricingEntry[] {
  return mergeLocalEntries()
}

/** 桌面态走后端 SSOT；浏览器回退本地 */
export async function fetchModelPricing(): Promise<ModelPricingEntry[]> {
  if (!isTauriRuntime()) return mergeLocalEntries()
  try {
    const remote = await invoke<unknown>('get_model_pricing')
    return normalizeRemoteEntries(remote)
  } catch {
    return mergeLocalEntries()
  }
}

export function getModelPricing(modelId: string): ModelPricingEntry | null {
  const key = modelId.trim().toLowerCase()
  if (!key) return null
  return listModelPricing().find((item) => item.modelId.toLowerCase() === key) ?? null
}

export async function upsertModelPricing(input: {
  modelId: string
  displayName: string
  vendor?: ModelPricingVendor
  inputCostPerMillion: number
  outputCostPerMillion: number
  cacheReadCostPerMillion: number
  cacheCreationCostPerMillion: number
  completionCostPerMillion?: number
  imageCostPerMillion?: number
  videoCostPerMillion?: number
  audioCostPerMillion?: number
}): Promise<ModelPricingEntry[]> {
  const modelId = input.modelId.trim()
  if (!modelId) throw new Error('modelId required')
  const displayName = input.displayName.trim() || modelId
  const vendor = input.vendor || inferVendor(modelId)

  if (isTauriRuntime()) {
    const remote = await invoke<unknown>('update_model_pricing', {
      modelId,
      displayName,
      vendor,
      inputCostPerMillion: num(input.inputCostPerMillion),
      outputCostPerMillion: num(input.outputCostPerMillion),
      cacheReadCostPerMillion: num(input.cacheReadCostPerMillion),
      cacheCreationCostPerMillion: num(input.cacheCreationCostPerMillion),
      completionCostPerMillion: num(input.completionCostPerMillion ?? 0),
      imageCostPerMillion: num(input.imageCostPerMillion ?? 0),
      videoCostPerMillion: num(input.videoCostPerMillion ?? 0),
      audioCostPerMillion: num(input.audioCostPerMillion ?? 0),
    })
    return normalizeRemoteEntries(remote)
  }

  const entry = normalizeEntry({
    modelId,
    displayName,
    vendor,
    inputCostPerMillion: input.inputCostPerMillion,
    outputCostPerMillion: input.outputCostPerMillion,
    cacheReadCostPerMillion: input.cacheReadCostPerMillion,
    cacheCreationCostPerMillion: input.cacheCreationCostPerMillion,
    completionCostPerMillion: input.completionCostPerMillion ?? 0,
    imageCostPerMillion: input.imageCostPerMillion ?? 0,
    videoCostPerMillion: input.videoCostPerMillion ?? 0,
    audioCostPerMillion: input.audioCostPerMillion ?? 0,
    source: 'user',
    updatedAt: nowIso(),
  })
  const overrides = readLocalOverrides().filter((item) => item.modelId.toLowerCase() !== modelId.toLowerCase())
  overrides.push(entry)
  writeLocalOverrides(overrides)
  return mergeLocalEntries()
}

export async function deleteModelPricing(modelId: string): Promise<ModelPricingEntry[]> {
  const key = modelId.trim()
  if (!key) return fetchModelPricing()

  if (isTauriRuntime()) {
    const remote = await invoke<unknown>('delete_model_pricing', { modelId: key })
    return normalizeRemoteEntries(remote)
  }

  const overrides = readLocalOverrides().filter((item) => item.modelId.toLowerCase() !== key.toLowerCase())
  writeLocalOverrides(overrides)
  return mergeLocalEntries()
}

/** 还原单条到 seed：删除用户覆盖（后端 delete 后 seed 重新露出） */
export async function resetModelPricingToSeed(modelId: string): Promise<ModelPricingEntry[]> {
  return deleteModelPricing(modelId)
}

/** 清除全部用户覆盖；后端无 bulk reset，逐条 delete user 源 */
export async function resetAllModelPricing(): Promise<ModelPricingEntry[]> {
  const current = await fetchModelPricing()
  const userIds = current.filter((item) => item.source === 'user').map((item) => item.modelId)
  if (!userIds.length) return current

  if (isTauriRuntime()) {
    let last = current
    for (const id of userIds) {
      last = await deleteModelPricing(id)
    }
    return last
  }

  if (typeof window !== 'undefined' && window.localStorage) {
    window.localStorage.removeItem(STORAGE_KEY)
  }
  return mergeLocalEntries()
}

export function entryToDraft(entry: ModelPricingEntry, unit: ModelPriceUnit = 'per_million'): ModelPricingDraft {
  const factor = unit === 'per_thousand' ? 0.001 : 1
  return {
    modelId: entry.modelId,
    displayName: entry.displayName,
    vendor: entry.vendor,
    unit,
    inputCost: roundPrice(entry.inputCostPerMillion * factor),
    outputCost: roundPrice(entry.outputCostPerMillion * factor),
    cacheReadCost: roundPrice(entry.cacheReadCostPerMillion * factor),
    cacheCreationCost: roundPrice(entry.cacheCreationCostPerMillion * factor),
    completionCost: roundPrice(entry.completionCostPerMillion * factor),
    imageCost: roundPrice(entry.imageCostPerMillion * factor),
    videoCost: roundPrice(entry.videoCostPerMillion * factor),
    audioCost: roundPrice(entry.audioCostPerMillion * factor),
  }
}

export function emptyDraft(unit: ModelPriceUnit = 'per_million'): ModelPricingDraft {
  return {
    modelId: '',
    displayName: '',
    vendor: 'other',
    unit,
    inputCost: 0,
    outputCost: 0,
    cacheReadCost: 0,
    cacheCreationCost: 0,
    completionCost: 0,
    imageCost: 0,
    videoCost: 0,
    audioCost: 0,
  }
}

export function draftToPerMillion(draft: ModelPricingDraft) {
  const factor = draft.unit === 'per_thousand' ? 1000 : 1
  return {
    inputCostPerMillion: num(draft.inputCost) * factor,
    outputCostPerMillion: num(draft.outputCost) * factor,
    cacheReadCostPerMillion: num(draft.cacheReadCost) * factor,
    cacheCreationCostPerMillion: num(draft.cacheCreationCost) * factor,
    completionCostPerMillion: num(draft.completionCost) * factor,
    imageCostPerMillion: num(draft.imageCost) * factor,
    videoCostPerMillion: num(draft.videoCost) * factor,
    audioCostPerMillion: num(draft.audioCost) * factor,
  }
}

export function convertDisplayedPrice(value: number, from: ModelPriceUnit, to: ModelPriceUnit) {
  if (from === to) return value
  if (from === 'per_million' && to === 'per_thousand') return value / 1000
  return value * 1000
}

export function formatPriceUsd(value: number, digits = 4) {
  if (!Number.isFinite(value)) return '—'
  if (value === 0) return '$0'
  if (value >= 1) return `$${value.toFixed(Math.min(digits, 2))}`
  return `$${value.toFixed(digits)}`
}

function roundPrice(value: number) {
  if (!Number.isFinite(value)) return 0
  return Math.round(value * 1_000_000) / 1_000_000
}

export function filterModelPricing(
  entries: ModelPricingEntry[],
  options: { vendor?: ModelPricingVendor | 'all'; query?: string } = {},
) {
  const vendor = options.vendor || 'all'
  const query = (options.query || '').trim().toLowerCase()
  return entries.filter((entry) => {
    if (vendor !== 'all' && entry.vendor !== vendor) return false
    if (!query) return true
    return (
      entry.modelId.toLowerCase().includes(query)
      || entry.displayName.toLowerCase().includes(query)
      || entry.vendor.includes(query)
    )
  })
}
