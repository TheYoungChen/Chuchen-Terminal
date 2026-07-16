<template>
  <ModalShell
    :open="open"
    :title="tr('模型价格', 'Model Pricing')"
    :description="tr(
      '内置官方参考价，可按厂商筛选与编辑。桌面态写入后端计费表（~/.chuchen-terminal/usage.db），并立即影响 Usage 成本。',
      'Built-in official reference prices. Desktop mode writes the backend pricing table (~/.chuchen-terminal/usage.db) and affects usage cost immediately.',
    )"
    icon="dollar"
    size="lg"
    @close="emit('close')"
  >
    <div class="pricing-modal">
      <div class="pricing-modal__toolbar">
        <div class="pricing-modal__filters">
          <input
            v-model.trim="query"
            class="pricing-modal__search"
            type="search"
            :placeholder="tr('搜索模型 ID / 名称', 'Search model id / name')"
          />
          <div class="pricing-modal__vendors">
            <button
              v-for="vendor in MODEL_PRICING_VENDORS"
              :key="vendor.id"
              type="button"
              class="pricing-modal__chip"
              :class="{ 'pricing-modal__chip--active': vendorFilter === vendor.id }"
              @click="vendorFilter = vendor.id"
            >{{ locale === 'en-US' ? vendor.labelEn : vendor.labelZh }}</button>
          </div>
        </div>
        <div class="pricing-modal__actions">
          <div class="pricing-modal__unit">
            <span>{{ tr('单位', 'Unit') }}</span>
            <button type="button" class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--primary': unit === 'per_million' }" @click="setUnit('per_million')">$/1M</button>
            <button type="button" class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--primary': unit === 'per_thousand' }" @click="setUnit('per_thousand')">$/1K</button>
          </div>
          <button type="button" class="ghost-btn ghost-btn--small" @click="beginCreate()">
            <AppIcon name="plus" :size="13" />
            <span>{{ tr('新增', 'Add') }}</span>
          </button>
          <button type="button" class="ghost-btn ghost-btn--small" @click="handleResetAll()">
            <span>{{ tr('恢复内置', 'Reset seeds') }}</span>
          </button>
        </div>
      </div>

      <p class="pricing-modal__hint">
        {{ tr(
          `列表 ${filteredEntries.length} 条 · 价格按 ${unitLabel} 显示 · 输入/输出/缓存读写用于 Usage 成本估算；补全/图/视频/音频为预留字段`,
          `${filteredEntries.length} rows · shown as ${unitLabel} · input/output/cache used for usage cost; completion/image/video/audio are reserved`,
        ) }}
      </p>

      <div class="pricing-modal__table-wrap">
        <table class="pricing-modal__table">
          <thead>
            <tr>
              <th>{{ tr('模型', 'Model') }}</th>
              <th>{{ tr('厂商', 'Vendor') }}</th>
              <th class="is-num">{{ tr('输入', 'Input') }}</th>
              <th class="is-num">{{ tr('输出', 'Output') }}</th>
              <th class="is-num">{{ tr('缓存读', 'Cache R') }}</th>
              <th class="is-num">{{ tr('缓存写', 'Cache W') }}</th>
              <th class="is-num">{{ tr('补全', 'Compl.') }}</th>
              <th class="is-num">{{ tr('图', 'Image') }}</th>
              <th class="is-num">{{ tr('视频', 'Video') }}</th>
              <th class="is-num">{{ tr('音频', 'Audio') }}</th>
              <th>{{ tr('来源', 'Source') }}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entry in filteredEntries" :key="entry.modelId">
              <td>
                <strong>{{ entry.displayName }}</strong>
                <small>{{ entry.modelId }}</small>
              </td>
              <td>{{ vendorLabel(entry.vendor) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.inputCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.outputCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.cacheReadCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.cacheCreationCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.completionCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.imageCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.videoCostPerMillion)) }}</td>
              <td class="is-num">{{ formatPriceUsd(displayPrice(entry.audioCostPerMillion)) }}</td>
              <td>
                <span class="pricing-modal__source" :class="entry.source === 'user' ? 'is-user' : 'is-seed'">
                  {{ entry.source === 'user' ? tr('自定义', 'Custom') : tr('内置', 'Built-in') }}
                </span>
              </td>
              <td class="pricing-modal__row-actions">
                <button type="button" class="ghost-btn ghost-btn--small" @click="beginEdit(entry)">{{ tr('编辑', 'Edit') }}</button>
                <button
                  v-if="entry.source === 'user'"
                  type="button"
                  class="ghost-btn ghost-btn--small"
                  @click="handleResetOne(entry.modelId)"
                >{{ tr('还原', 'Reset') }}</button>
              </td>
            </tr>
            <tr v-if="!filteredEntries.length">
              <td colspan="12" class="pricing-modal__empty">{{ tr('无匹配模型', 'No matching models') }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="draftOpen" class="pricing-modal__editor">
        <div class="pricing-modal__editor-head">
          <strong>{{ draftIsNew ? tr('新增模型价格', 'Add model pricing') : tr('编辑模型价格', 'Edit model pricing') }}</strong>
          <button type="button" class="icon-btn" @click="closeDraft()">×</button>
        </div>
        <form class="pricing-modal__form" @submit.prevent="saveDraft()">
          <label class="form-field">
            <span>{{ tr('模型 ID', 'Model ID') }}</span>
            <input v-model.trim="draft.modelId" type="text" :disabled="!draftIsNew" required placeholder="claude-sonnet-4-6" @change="onModelIdChange" />
          </label>
          <label class="form-field">
            <span>{{ tr('显示名称', 'Display name') }}</span>
            <input v-model.trim="draft.displayName" type="text" required placeholder="Claude Sonnet 4.6" />
          </label>
          <label class="form-field">
            <span>{{ tr('厂商', 'Vendor') }}</span>
            <select v-model="draft.vendor">
              <option v-for="vendor in editableVendors" :key="vendor.id" :value="vendor.id">{{ locale === 'en-US' ? vendor.labelEn : vendor.labelZh }}</option>
            </select>
          </label>
          <label class="form-field">
            <span>{{ tr('输入价格', 'Input') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.inputCost" type="number" min="0" step="0.0001" required />
          </label>
          <label class="form-field">
            <span>{{ tr('输出价格', 'Output') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.outputCost" type="number" min="0" step="0.0001" required />
          </label>
          <label class="form-field">
            <span>{{ tr('缓存读取', 'Cache read') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.cacheReadCost" type="number" min="0" step="0.0001" required />
          </label>
          <label class="form-field">
            <span>{{ tr('缓存创建', 'Cache create') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.cacheCreationCost" type="number" min="0" step="0.0001" required />
          </label>
          <label class="form-field">
            <span>{{ tr('补全价格', 'Completion') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.completionCost" type="number" min="0" step="0.0001" />
          </label>
          <label class="form-field">
            <span>{{ tr('生图价格', 'Image') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.imageCost" type="number" min="0" step="0.0001" />
          </label>
          <label class="form-field">
            <span>{{ tr('生视频价格', 'Video') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.videoCost" type="number" min="0" step="0.0001" />
          </label>
          <label class="form-field">
            <span>{{ tr('音频价格', 'Audio') }} ({{ unitLabel }})</span>
            <input v-model.number="draft.audioCost" type="number" min="0" step="0.0001" />
          </label>
          <div class="form-actions">
            <button type="button" class="ghost-btn" @click="closeDraft()">{{ tr('取消', 'Cancel') }}</button>
            <button type="submit" class="ghost-btn ghost-btn--primary" :disabled="draftSaving || loading">
              {{ draftSaving ? tr('保存中…', 'Saving…') : tr('保存', 'Save') }}
            </button>
          </div>
        </form>
        <p v-if="draftError" class="pricing-modal__error">{{ draftError }}</p>
      </div>
    </div>
  </ModalShell>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from './AppIcon.vue'
import ModalShell from './ModalShell.vue'
import {
  MODEL_PRICING_VENDORS,
  convertDisplayedPrice,
  draftToPerMillion,
  emptyDraft,
  entryToDraft,
  fetchModelPricing,
  filterModelPricing,
  formatPriceUsd,
  inferVendor,
  listModelPricing,
  resetAllModelPricing,
  resetModelPricingToSeed,
  upsertModelPricing,
  type ModelPriceUnit,
  type ModelPricingDraft,
  type ModelPricingEntry,
  type ModelPricingVendor,
} from '../services/model-pricing'
import { pickLocaleText } from '../services/runtime-locale'

const props = defineProps<{
  open: boolean
  locale?: string
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const query = ref('')
const vendorFilter = ref<ModelPricingVendor | 'all'>('all')
const unit = ref<ModelPriceUnit>('per_million')
const entries = ref<ModelPricingEntry[]>(listModelPricing())
const loading = ref(false)
const draftOpen = ref(false)
const draftIsNew = ref(false)
const draft = ref<ModelPricingDraft>(emptyDraft())
const draftError = ref('')
const draftSaving = ref(false)

const locale = computed(() => props.locale || 'zh-CN')

function tr(zh: string, en: string) {
  return pickLocaleText(zh, en)
}

const unitLabel = computed(() => (unit.value === 'per_thousand' ? '$/1K tokens' : '$/1M tokens'))
const editableVendors = computed(() => MODEL_PRICING_VENDORS.filter((item) => item.id !== 'all') as Array<{ id: ModelPricingVendor; labelZh: string; labelEn: string }>)

const filteredEntries = computed(() => filterModelPricing(entries.value, {
  vendor: vendorFilter.value,
  query: query.value,
}))

async function reload() {
  loading.value = true
  try {
    entries.value = await fetchModelPricing()
  } catch (error) {
    draftError.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

watch(() => props.open, (open) => {
  if (!open) {
    closeDraft()
    return
  }
  void reload()
})

function displayPrice(perMillion: number) {
  return unit.value === 'per_thousand' ? convertDisplayedPrice(perMillion, 'per_million', 'per_thousand') : perMillion
}

function vendorLabel(vendor: ModelPricingVendor) {
  const found = MODEL_PRICING_VENDORS.find((item) => item.id === vendor)
  if (!found) return vendor
  return locale.value === 'en-US' ? found.labelEn : found.labelZh
}

function setUnit(next: ModelPriceUnit) {
  if (unit.value === next) return
  if (draftOpen.value) {
    const factor = unit.value === 'per_million' && next === 'per_thousand' ? 0.001 : 1000
    draft.value = {
      ...draft.value,
      unit: next,
      inputCost: round(draft.value.inputCost * factor),
      outputCost: round(draft.value.outputCost * factor),
      cacheReadCost: round(draft.value.cacheReadCost * factor),
      cacheCreationCost: round(draft.value.cacheCreationCost * factor),
      completionCost: round(draft.value.completionCost * factor),
      imageCost: round(draft.value.imageCost * factor),
      videoCost: round(draft.value.videoCost * factor),
      audioCost: round(draft.value.audioCost * factor),
    }
  }
  unit.value = next
}

function beginCreate() {
  draftIsNew.value = true
  draftError.value = ''
  draft.value = emptyDraft(unit.value)
  draftOpen.value = true
}

function beginEdit(entry: ModelPricingEntry) {
  draftIsNew.value = false
  draftError.value = ''
  draft.value = entryToDraft(entry, unit.value)
  draftOpen.value = true
}

function closeDraft() {
  draftOpen.value = false
  draftError.value = ''
}

function onModelIdChange() {
  if (!draftIsNew.value) return
  if (!draft.value.displayName) draft.value.displayName = draft.value.modelId
  draft.value.vendor = inferVendor(draft.value.modelId)
}

async function saveDraft() {
  draftError.value = ''
  const modelId = draft.value.modelId.trim()
  if (!modelId) {
    draftError.value = tr('模型 ID 不能为空', 'Model ID is required')
    return
  }
  if (!draft.value.displayName.trim()) {
    draftError.value = tr('显示名称不能为空', 'Display name is required')
    return
  }
  const costs = draftToPerMillion(draft.value)
  const fields = Object.values(costs)
  if (fields.some((value) => !Number.isFinite(value) || value < 0)) {
    draftError.value = tr('价格必须为非负数', 'Prices must be non-negative')
    return
  }
  draftSaving.value = true
  try {
    entries.value = await upsertModelPricing({
      modelId,
      displayName: draft.value.displayName,
      vendor: draft.value.vendor,
      ...costs,
    })
    closeDraft()
    emit('saved')
  } catch (error) {
    draftError.value = error instanceof Error ? error.message : String(error)
  } finally {
    draftSaving.value = false
  }
}

async function handleResetOne(modelId: string) {
  draftError.value = ''
  loading.value = true
  try {
    entries.value = await resetModelPricingToSeed(modelId)
    emit('saved')
  } catch (error) {
    draftError.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

async function handleResetAll() {
  draftError.value = ''
  loading.value = true
  try {
    entries.value = await resetAllModelPricing()
    closeDraft()
    emit('saved')
  } catch (error) {
    draftError.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

function round(value: number) {
  return Math.round(value * 1_000_000) / 1_000_000
}
</script>

<style scoped>
.pricing-modal {
  display: grid;
  gap: 12px;
  min-height: 0;
}

.pricing-modal__toolbar {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 12px;
}

.pricing-modal__filters {
  display: grid;
  gap: 8px;
  min-width: min(100%, 520px);
  flex: 1;
}

.pricing-modal__search {
  height: 34px;
  width: 100%;
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 0 12px;
  background: rgba(255, 255, 255, 0.7);
  color: var(--text-primary);
  font: inherit;
}

.pricing-modal__vendors {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.pricing-modal__chip {
  min-height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  border: 1px solid rgba(17, 24, 39, 0.08);
  background: rgba(255, 255, 255, 0.55);
  color: var(--text-secondary);
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
}

.pricing-modal__chip--active {
  border-color: rgba(15, 23, 42, 0.22);
  background: rgba(15, 23, 42, 0.08);
  color: var(--text-primary);
}

.pricing-modal__actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.pricing-modal__unit {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.pricing-modal__hint {
  margin: 0;
  font-size: 11.5px;
  color: var(--text-tertiary);
}

.pricing-modal__table-wrap {
  max-height: min(48vh, 420px);
  overflow: auto;
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.48);
}

.pricing-modal__table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  font-size: 12px;
}

.pricing-modal__table th,
.pricing-modal__table td {
  padding: 9px 10px;
  border-bottom: 1px solid rgba(17, 24, 39, 0.06);
  white-space: nowrap;
  text-align: left;
  vertical-align: middle;
}

.pricing-modal__table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: rgba(248, 250, 252, 0.96);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 700;
}

.pricing-modal__table td strong {
  display: block;
  color: var(--text-primary);
  font-size: 12px;
}

.pricing-modal__table td small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 10.5px;
}

.pricing-modal__table .is-num {
  text-align: right;
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
}

.pricing-modal__source {
  display: inline-flex;
  min-height: 22px;
  align-items: center;
  padding: 0 8px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 700;
}

.pricing-modal__source.is-seed {
  background: rgba(59, 130, 246, 0.1);
  color: #2563eb;
}

.pricing-modal__source.is-user {
  background: rgba(16, 185, 129, 0.12);
  color: #047857;
}

.pricing-modal__row-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.pricing-modal__empty {
  text-align: center !important;
  color: var(--text-tertiary);
  padding: 28px 12px !important;
}

.pricing-modal__editor {
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 12px;
  background: rgba(248, 250, 252, 0.72);
  display: grid;
  gap: 10px;
}

.pricing-modal__editor-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.pricing-modal__form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 12px;
}

.pricing-modal__form .form-actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.pricing-modal__error {
  margin: 0;
  color: #b91c1c;
  font-size: 12px;
}

@media (max-width: 860px) {
  .pricing-modal__form {
    grid-template-columns: 1fr;
  }
}
</style>
