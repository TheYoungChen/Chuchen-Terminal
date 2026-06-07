<template>
  <div
    v-if="open"
    class="popover-menu"
    :style="popoverStyle"
    @click.stop
    @mousedown.stop
    @mouseup.stop
    @pointerdown.stop
    @pointerup.stop
    @contextmenu.stop.prevent
  >
    <button
      v-for="item in items"
      :key="`${item.label}-${item.shortcut || ''}`"
      type="button"
      class="popover-menu__item"
      :class="{
        'popover-menu__item--active': item.active,
        'popover-menu__item--danger': item.danger,
      }"
      @click.stop="item.onClick?.()"
      @mousedown.stop
      @mouseup.stop
      @pointerdown.stop
      @pointerup.stop
    >
      <AppIcon v-if="item.icon" :name="item.icon" :size="14" />
      <span class="popover-menu__item-content">
        <span class="popover-menu__item-label">{{ item.label }}</span>
        <span v-if="item.badges?.length" class="popover-menu__badges">
          <span v-for="badge in item.badges" :key="`${item.label}-${badge}`" class="popover-menu__badge">{{ badge }}</span>
        </span>
        <small v-if="item.description" class="popover-menu__item-description">{{ item.description }}</small>
      </span>
      <small v-if="item.shortcut" class="popover-menu__item-shortcut">{{ item.shortcut }}</small>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { CSSProperties } from 'vue'
import { computed } from 'vue'
import AppIcon from './AppIcon.vue'

export interface PopoverItem {
  label: string
  icon?: string
  badges?: string[]
  description?: string
  shortcut?: string
  active?: boolean
  danger?: boolean
  onClick?: () => void
}

const props = defineProps<{
  open: boolean
  items: PopoverItem[]
  position?: { x: number; y: number } | null
}>()

const popoverStyle = computed<CSSProperties | undefined>(() => {
  if (!props.position) return undefined

  return {
    position: 'fixed',
    left: `${props.position.x}px`,
    top: `${props.position.y}px`,
    right: 'auto',
    transformOrigin: 'top left',
  }
})
</script>
