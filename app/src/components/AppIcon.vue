<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    class="app-icon"
  >
    <template v-if="name === 'workspace'">
      <rect x="4" y="4" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.6" />
      <rect x="13" y="4" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.6" />
      <rect x="4" y="13" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.6" />
      <rect x="13" y="13" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.6" />
    </template>

    <template v-else-if="name === 'home'">
      <path d="M4.5 11.5L12 5L19.5 11.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M6.5 10.5V18.2C6.5 18.92 7.08 19.5 7.8 19.5H16.2C16.92 19.5 17.5 18.92 17.5 18.2V10.5" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M10 19V14H14V19" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'recent'">
      <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 8V12L15 14" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'template'">
      <path d="M4 6.5C4 5.67 4.67 5 5.5 5H18.5C19.33 5 20 5.67 20 6.5V9.5C20 10.33 19.33 11 18.5 11H5.5C4.67 11 4 10.33 4 9.5V6.5Z" stroke="currentColor" stroke-width="1.6" />
      <path d="M4 14.5C4 13.67 4.67 13 5.5 13H10.5C11.33 13 12 13.67 12 14.5V17.5C12 18.33 11.33 19 10.5 19H5.5C4.67 19 4 18.33 4 17.5V14.5Z" stroke="currentColor" stroke-width="1.6" />
      <path d="M14 14.5C14 13.67 14.67 13 15.5 13H18.5C19.33 13 20 13.67 20 14.5V17.5C20 18.33 19.33 19 18.5 19H15.5C14.67 19 14 18.33 14 17.5V14.5Z" stroke="currentColor" stroke-width="1.6" />
    </template>

    <template v-else-if="name === 'search'">
      <circle cx="11" cy="11" r="5.5" stroke="currentColor" stroke-width="1.6" />
      <path d="M15.5 15.5L19 19" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'theme'">
      <path d="M12 4.5C7.86 4.5 4.5 7.86 4.5 12C4.5 16.14 7.86 19.5 12 19.5C13.38 19.5 14.33 18.76 14.33 17.63C14.33 16.97 14.03 16.45 14.03 15.94C14.03 15.15 14.69 14.5 15.49 14.5H16.75C18.82 14.5 19.5 12.97 19.5 11.26C19.5 7.57 16.43 4.5 12 4.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <circle cx="8.25" cy="10" r="0.9" fill="currentColor" />
      <circle cx="10.5" cy="7.7" r="0.9" fill="currentColor" />
      <circle cx="13.8" cy="8.2" r="0.9" fill="currentColor" />
    </template>

    <template v-else-if="name === 'settings'">
      <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 3.8V6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M12 18V20.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M20.2 12H18" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M6 12H3.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M17.8 6.2L16.2 7.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M7.8 16.2L6.2 17.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M17.8 17.8L16.2 16.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M7.8 7.8L6.2 6.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'chevron-right'">
      <path d="M9 6L15 12L9 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'chevron-down'">
      <path d="M6 9L12 15L18 9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'plus'">
      <path d="M12 5V19" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      <path d="M5 12H19" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'detail'">
      <path d="M5 6H19" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M5 12H19" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M5 18H19" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <circle cx="7" cy="6" r="1" fill="currentColor" />
      <circle cx="11" cy="12" r="1" fill="currentColor" />
      <circle cx="15" cy="18" r="1" fill="currentColor" />
    </template>

    <template v-else-if="name === 'runtime'">
      <rect x="4" y="5" width="16" height="14" rx="2" stroke="currentColor" stroke-width="1.6" />
      <path d="M8 10L10.8 12L8 14" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M13.2 14H16" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'folder'">
      <path d="M4 8.5C4 7.67 4.67 7 5.5 7H9L10.5 8.5H18.5C19.33 8.5 20 9.17 20 10V17.5C20 18.33 19.33 19 18.5 19H5.5C4.67 19 4 18.33 4 17.5V8.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'copy'">
      <rect x="9" y="8" width="9" height="11" rx="1.8" stroke="currentColor" stroke-width="1.6" />
      <path d="M7 15H6.2C5.54 15 5 14.46 5 13.8V6.2C5 5.54 5.54 5 6.2 5H12.8C13.46 5 14 5.54 14 6.2V7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'download'">
      <path d="M12 4.5V14" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M8.3 10.8L12 14.5L15.7 10.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M5.5 18.5H18.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'pane'">
      <rect x="4" y="5" width="16" height="14" rx="2" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 5V19" stroke="currentColor" stroke-width="1.6" />
    </template>

    <template v-else-if="name === 'tab'">
      <path d="M4 8.5C4 7.67 4.67 7 5.5 7H9L10.2 8.5H18.5C19.33 8.5 20 9.17 20 10V16.5C20 17.33 19.33 18 18.5 18H5.5C4.67 18 4 17.33 4 16.5V8.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'play'">
      <path d="M9 7.5L17 12L9 16.5V7.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'terminal'">
      <rect x="4" y="5" width="16" height="14" rx="2" stroke="currentColor" stroke-width="1.6" />
      <path d="M8 10L11 12L8 14" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M13 14H16.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'codex'">
      <circle cx="12" cy="12" r="1.4" fill="currentColor" />
      <circle cx="12" cy="6.1" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <circle cx="17.4" cy="9.1" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <circle cx="17.4" cy="14.9" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <circle cx="12" cy="17.9" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <circle cx="6.6" cy="14.9" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <circle cx="6.6" cy="9.1" r="1.4" stroke="currentColor" stroke-width="1.4" />
      <path d="M12 7.5V10.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M15.8 10.2L13.6 11.4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M15.8 13.8L13.6 12.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M12 16.5V13.4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M8.2 13.8L10.4 12.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M8.2 10.2L10.4 11.4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'claude'">
      <path d="M16.8 7.4C15.85 6.5 14.55 6 13.1 6H10.9C7.64 6 5 8.64 5 11.9V12.1C5 15.36 7.64 18 10.9 18H13.1C14.55 18 15.85 17.5 16.8 16.6" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M18.2 8.6C17.1 7.35 15.7 6.58 14.15 6.28" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      <path d="M18.2 15.4C17.1 16.65 15.7 17.42 14.15 17.72" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'gemini'">
      <path d="M12 4.8L13.9 10.1L19.2 12L13.9 13.9L12 19.2L10.1 13.9L4.8 12L10.1 10.1L12 4.8Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M17.2 5.2L17.9 7.1L19.8 7.8L17.9 8.5L17.2 10.4L16.5 8.5L14.6 7.8L16.5 7.1L17.2 5.2Z" fill="currentColor" />
    </template>

    <template v-else-if="name === 'opencode'">
      <path d="M9.2 7.4L5 12L9.2 16.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M14.8 7.4L19 12L14.8 16.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M13.3 6L10.7 18" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'sparkle'">
      <path d="M12 5.5L13.5 10.5L18.5 12L13.5 13.5L12 18.5L10.5 13.5L5.5 12L10.5 10.5L12 5.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'more'">
      <circle cx="7" cy="12" r="1.4" fill="currentColor" />
      <circle cx="12" cy="12" r="1.4" fill="currentColor" />
      <circle cx="17" cy="12" r="1.4" fill="currentColor" />
    </template>

    <template v-else-if="name === 'edit'">
      <path d="M5 16.5V19H7.5L17.8 8.7L15.3 6.2L5 16.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M13.9 7.6L16.4 10.1" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M4.5 19H19.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'trash'">
      <path d="M5.5 7.5H18.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M9 5.5H15" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M7 7.5L7.8 18.1C7.86 18.92 8.54 19.55 9.35 19.55H14.65C15.46 19.55 16.14 18.92 16.2 18.1L17 7.5" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M10 11V16" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M14 11V16" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'close'">
      <path d="M8 8L16 16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      <path d="M16 8L8 16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'info'">
      <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 10.5V16" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <circle cx="12" cy="7.6" r="1" fill="currentColor" />
    </template>

    <template v-else-if="name === 'check'">
      <path d="M7 12.5L10.2 15.5L17 8.8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'bolt'">
      <path d="M13 3L6 13H11L10 21L18 10.5H13.5L13 3Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'refresh'">
      <path d="M19 8A7 7 0 1 0 20 12" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      <path d="M20 5V9H16" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'star'">
      <path d="M12 4.5L14.25 9.05L19.25 9.78L15.62 13.31L16.48 18.3L12 15.95L7.52 18.3L8.38 13.31L4.75 9.78L9.75 9.05L12 4.5Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'activity'">
      <polyline points="22 12 18 12 15 20 9 4 6 12 2 12" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'dollar'">
      <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 7V8.5M12 15.5V17M14.5 9.8C14.5 9.1 13.38 8.5 12 8.5C10.62 8.5 9.5 9.1 9.5 9.8C9.5 10.5 10.62 11 12 11C13.38 11 14.5 11.5 14.5 12.2C14.5 12.9 13.38 13.5 12 13.5C10.62 13.5 9.5 12.9 9.5 12.2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'bar-chart'">
      <rect x="4" y="12" width="4" height="8" rx="1" stroke="currentColor" stroke-width="1.6" />
      <rect x="10" y="7" width="4" height="13" rx="1" stroke="currentColor" stroke-width="1.6" />
      <rect x="16" y="4" width="4" height="16" rx="1" stroke="currentColor" stroke-width="1.6" />
    </template>

    <template v-else-if="name === 'package'">
      <path d="M20 7L12 3L4 7V17L12 21L20 17V7Z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M4 7L12 11L20 7" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
      <path d="M12 11V21" stroke="currentColor" stroke-width="1.6" />
      <path d="M8 5L16 9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'coin'">
      <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.6" />
      <path d="M9.5 14.5C9.5 14.5 10.2 15.5 12 15.5C13.8 15.5 14.5 14.5 14.5 13.5C14.5 12.5 13.5 12 12 11.5C10.5 11 9.5 10.5 9.5 9.5C9.5 8.5 10.2 8 12 8C13.8 8 14.5 8.8 14.5 8.8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <path d="M12 7V8M12 16V17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    </template>

    <template v-else>
      <circle cx="12" cy="12" r="7" stroke="currentColor" stroke-width="1.6" />
    </template>
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  name: string
  size?: number
}>(), {
  size: 16,
})
</script>

<style scoped>
.app-icon {
  display: block;
  flex: none;
}
</style>
