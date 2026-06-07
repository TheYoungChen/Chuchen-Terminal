<template>
  <div class="snapshot-preview" :class="{ 'snapshot-preview--compact': compact }">
    <div class="snapshot-preview__tabs">
      <article
        v-for="tab in visibleTabs"
        :key="tab.id"
        class="snapshot-preview__tab"
        :class="{ 'snapshot-preview__tab--active': tab.id === activeTabId }"
      >
        <header class="snapshot-preview__tab-head">
          <strong>{{ tab.name }}</strong>
          <small>{{ countLeafPanes(tab.panes) }} Pane</small>
        </header>
        <div
          class="snapshot-preview__tab-canvas"
          :class="{
            'snapshot-preview__tab-canvas--horizontal': tab.layoutMode === 'horizontal',
            'snapshot-preview__tab-canvas--vertical': tab.layoutMode === 'vertical',
          }"
        >
          <SnapshotMiniPane
            v-for="pane in tab.panes"
            :key="pane.id"
            :pane="pane"
          />
        </div>
      </article>
    </div>
    <div v-if="hiddenTabCount > 0" class="snapshot-preview__more">
      +{{ hiddenTabCount }} 个项目
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SnapshotMiniPane from './SnapshotMiniPane.vue'
import type { PaneNode, WorkspaceTab } from '../types/workspace'

const props = withDefaults(defineProps<{
  tabs: WorkspaceTab[]
  activeTabId?: string | null
  compact?: boolean
}>(), {
  activeTabId: null,
  compact: false,
})

const visibleTabs = computed(() => props.tabs.slice(0, props.compact ? 2 : 3))
const hiddenTabCount = computed(() => Math.max(0, props.tabs.length - visibleTabs.value.length))

function countLeafPanes(panes: PaneNode[]): number {
  return flattenLeafPanes(panes).length
}

function countPaneSessions(pane: PaneNode): number {
  const own = pane.sessions?.length ?? 0
  if (!pane.children?.length) return own
  return pane.children.reduce((count, child) => count + countPaneSessions(child), own)
}

function flattenLeafPanes(panes: PaneNode[]): PaneNode[] {
  return panes.flatMap((pane) => pane.children?.length ? flattenLeafPanes(pane.children) : [pane])
}
</script>

<style scoped>
.snapshot-preview {
  display: grid;
  gap: 10px;
}

.snapshot-preview__tabs {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 10px;
}

.snapshot-preview__tab {
  display: grid;
  gap: 8px;
  padding: 10px;
  border-radius: 14px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.7), rgba(246, 248, 251, 0.95));
  border: 1px solid rgba(17, 24, 39, 0.08);
}

.snapshot-preview__tab--active {
  border-color: rgba(79, 124, 255, 0.28);
  box-shadow: 0 10px 20px rgba(79, 124, 255, 0.08);
}

.snapshot-preview__tab-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.snapshot-preview__tab-head strong,
.snapshot-preview__tab-head small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.snapshot-preview__tab-head strong {
  font-size: 11px;
}

.snapshot-preview__tab-head small,
.snapshot-preview__more {
  color: #76879b;
  font-size: 10px;
}

.snapshot-preview__tab-canvas {
  min-height: 74px;
  display: grid;
  grid-auto-flow: row;
  grid-template-columns: 1fr;
  gap: 6px;
}

.snapshot-preview__tab-canvas--horizontal {
  grid-auto-flow: column;
  grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
}

.snapshot-preview__tab-canvas--vertical {
  grid-template-columns: 1fr;
}

.snapshot-preview--compact .snapshot-preview__tabs {
  grid-template-columns: repeat(auto-fit, minmax(108px, 1fr));
}

.snapshot-preview--compact .snapshot-preview__tab {
  padding: 8px;
}

.snapshot-preview--compact .snapshot-preview__tab-canvas {
  min-height: 64px;
}
</style>
