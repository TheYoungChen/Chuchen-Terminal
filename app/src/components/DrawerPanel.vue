<template>
  <teleport to="body">
    <transition name="overlay-fade">
      <div v-if="open" class="drawer-layer" @pointerdown.self="handleLayerPointerDown" @pointerup.self="handleLayerPointerUp">
        <transition :name="side === 'right' ? 'drawer-slide-right' : 'drawer-slide-left'">
          <aside v-if="open" class="drawer-panel" :class="`drawer-panel--${side}`">
            <header class="drawer-panel__header">
              <div>
                <h3>{{ title }}</h3>
                <p v-if="description">{{ description }}</p>
              </div>
              <button type="button" class="icon-btn" @click="$emit('close')">×</button>
            </header>
            <div class="drawer-panel__body">
              <slot />
            </div>
          </aside>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'

withDefaults(defineProps<{
  open: boolean
  title: string
  description?: string
  side?: 'left' | 'right'
}>(), {
  side: 'right',
})

const emit = defineEmits<{ close: [] }>()
const layerPressed = ref(false)

function handleLayerPointerDown() {
  layerPressed.value = true
}

function handleLayerPointerUp() {
  if (!layerPressed.value) return
  layerPressed.value = false
  emit('close')
}
</script>
