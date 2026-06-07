<template>
  <div
    class="snapshot-preview__pane"
    :class="[
      hasChildren ? 'snapshot-preview__pane--group' : 'snapshot-preview__pane--leaf',
      pane.splitDirection === 'horizontal' ? 'snapshot-preview__pane--horizontal' : '',
      pane.splitDirection === 'vertical' ? 'snapshot-preview__pane--vertical' : '',
    ]"
  >
    <template v-if="hasChildren">
      <SnapshotMiniPane
        v-for="child in pane.children"
        :key="child.id"
        :pane="child"
      />
    </template>
    <template v-else>
      <span class="snapshot-preview__pane-name">{{ pane.name }}</span>
      <small class="snapshot-preview__pane-meta">{{ countPaneSessions(pane) }} 终端</small>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PaneNode } from '../types/workspace'

const props = defineProps<{
  pane: PaneNode
}>()

const hasChildren = computed(() => Boolean(props.pane.children?.length))

function countPaneSessions(pane: PaneNode): number {
  const own = pane.sessions?.length ?? 0
  if (!pane.children?.length) return own
  return pane.children.reduce((count, child) => count + countPaneSessions(child), own)
}
</script>

<style scoped>
.snapshot-preview__pane {
  min-width: 0;
  min-height: 0;
  display: grid;
  gap: 6px;
  border-radius: 10px;
}

.snapshot-preview__pane--group {
  padding: 6px;
  background: rgba(79, 124, 255, 0.06);
  border: 1px dashed rgba(79, 124, 255, 0.18);
}

.snapshot-preview__pane--horizontal {
  grid-auto-flow: column;
  grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
}

.snapshot-preview__pane--vertical {
  grid-template-columns: 1fr;
}

.snapshot-preview__pane--leaf {
  align-content: space-between;
  min-height: 54px;
  padding: 8px;
  background: linear-gradient(180deg, rgba(233, 246, 255, 0.9), rgba(245, 250, 255, 0.96));
  border: 1px solid rgba(79, 124, 255, 0.14);
}

.snapshot-preview__pane-name,
.snapshot-preview__pane-meta {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.snapshot-preview__pane-name {
  color: #18212b;
  font-size: 10px;
  font-weight: 700;
}

.snapshot-preview__pane-meta {
  color: #76879b;
  font-size: 9px;
}
</style>
