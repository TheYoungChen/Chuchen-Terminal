<template>
  <teleport to="body">
    <transition name="overlay-fade">
      <div v-if="open" class="overlay-mask" @pointerdown.self="handleMaskPointerDown" @pointerup.self="handleMaskPointerUp">
        <transition name="overlay-scale">
          <div v-if="open" class="overlay-modal" :class="sizeClass" role="dialog" aria-modal="true" @click.stop>
            <header class="overlay-modal__header">
              <div class="overlay-modal__title-wrap">
                <div v-if="icon" class="overlay-modal__icon-badge">
                  <AppIcon :name="icon" :size="16" />
                </div>
                <div class="overlay-modal__head-copy">
                <h3>{{ title }}</h3>
                <p v-if="description">{{ description }}</p>
                </div>
              </div>
              <button type="button" class="icon-btn" @click="$emit('close')">×</button>
            </header>
            <div class="overlay-modal__body">
              <slot />
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from './AppIcon.vue'

const props = withDefaults(defineProps<{
  open: boolean
  title: string
  description?: string
  icon?: string
  size?: 'sm' | 'md' | 'lg'
}>(), {
  size: 'md',
})

const emit = defineEmits<{ close: [] }>()

const sizeClass = computed(() => `overlay-modal--${props.size}`)
const maskPressed = ref(false)

function handleMaskPointerDown() {
  maskPressed.value = true
}

function handleMaskPointerUp() {
  if (!maskPressed.value) return
  maskPressed.value = false
  emit('close')
}
</script>
