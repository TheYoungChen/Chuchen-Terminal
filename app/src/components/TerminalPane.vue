<template>
  <div class="terminal-pane" @click="focusTerminal">
    <div ref="terminalHost" class="terminal-pane__host"></div>
    <div v-if="!runtimeBridgeReady" class="terminal-pane__bridge-badge">
      <span></span>
      {{ t('terminal.bridgePending') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import '@xterm/xterm/css/xterm.css'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { focusTerminalRuntime, mountTerminalRuntime, unmountTerminalRuntime, updateTerminalRuntimeActivity } from '../services/terminal-runtime'

const props = withDefaults(defineProps<{
  sessionId: string
  sessionName: string
  workingDirectory: string
  shellLabel?: string
  fontFamily?: string
  fontSize?: number
  bridgeReady?: boolean
  active?: boolean
  onCommandCommitted?: (sessionId: string, command: string) => void
  onSessionStateChange?: (sessionId: string, status: 'idle' | 'running') => void
  onOutputChunk?: (sessionId: string, chunk: string) => void
  onSessionExit?: (sessionId: string, exitCode: number) => void
}>(), {
  shellLabel: 'PowerShell 7',
  fontFamily: 'JetBrains Mono, Cascadia Code, Consolas, monospace',
  fontSize: 13,
  bridgeReady: false,
})

const terminalHost = ref<HTMLDivElement | null>(null)
const runtimeBridgeReady = ref(props.bridgeReady || (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window))
const { t } = useI18n()

async function mountCurrentSession() {
  await nextTick()
  if (!terminalHost.value) return

  await mountTerminalRuntime(terminalHost.value, {
    sessionId: props.sessionId,
    sessionName: props.sessionName,
    workingDirectory: props.workingDirectory,
    shellLabel: props.shellLabel,
    fontFamily: props.fontFamily,
    fontSize: props.fontSize,
    bridgeReady: runtimeBridgeReady.value,
    active: props.active,
    onCommandCommitted: props.onCommandCommitted,
    onSessionStateChange: props.onSessionStateChange,
    onOutputChunk: props.onOutputChunk,
    onSessionExit: props.onSessionExit,
  })
}

function focusTerminal() {
  focusTerminalRuntime(props.sessionId)
}

onMounted(async () => {
  await mountCurrentSession()
})

onBeforeUnmount(() => {
  unmountTerminalRuntime(props.sessionId)
})

watch(() => props.sessionId, async () => {
  await mountCurrentSession()
})

watch(() => [props.fontFamily, props.fontSize] as const, async () => {
  await mountCurrentSession()
})

watch(() => props.bridgeReady, async (value) => {
  runtimeBridgeReady.value = value || (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)
  await mountCurrentSession()
})

watch(() => props.active, (value) => {
  updateTerminalRuntimeActivity(props.sessionId, Boolean(value))
})
</script>

<style scoped>
.terminal-pane {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  width: 100%;
  height: 100%;
  background: #050608;
  overflow: hidden;
}

.terminal-pane__host {
  width: 100%;
  height: 100%;
  min-height: 0;
  padding: 4px 8px 4px;
  box-sizing: border-box;
}

.terminal-pane__bridge-badge {
  position: absolute;
  right: 10px;
  bottom: 10px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(5, 6, 8, 0.78);
  color: rgba(219, 228, 239, 0.62);
  font-size: 11px;
  pointer-events: none;
}

.terminal-pane__bridge-badge span {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: #f59e0b;
  box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.12);
}

.terminal-pane :deep(.xterm) {
  height: 100%;
}

.terminal-pane :deep(.xterm-viewport) {
  height: 100% !important;
  overflow-y: auto !important;
  overflow-x: hidden !important;
  background: transparent !important;
  padding-bottom: 6px;
  box-sizing: border-box;
}

.terminal-pane :deep(.xterm-screen),
.terminal-pane :deep(.xterm-helper-textarea) {
  min-height: 0;
}

.terminal-pane :deep(.xterm-viewport::-webkit-scrollbar) {
  width: 8px;
}

.terminal-pane :deep(.xterm-viewport::-webkit-scrollbar-track) {
  background: transparent;
}

.terminal-pane :deep(.xterm-viewport::-webkit-scrollbar-thumb) {
  border-radius: 999px;
  border: 2px solid transparent;
  background: rgba(255, 255, 255, 0.16);
  background-clip: padding-box;
}
</style>
