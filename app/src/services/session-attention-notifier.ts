import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import type { SessionAttentionState } from '../types/workspace'
import { pickLocaleText } from './runtime-locale'

export type SessionAttentionNotification = {
  workspaceName: string
  tabName: string
  sessionName: string
  state: SessionAttentionState
}

let permissionChecked = false
let permissionGranted = false

function stateTitle(state: SessionAttentionState) {
  if (state === 'needs-input') return pickLocaleText('终端等待输入', 'Terminal waiting for input')
  if (state === 'stalled') return pickLocaleText('终端疑似停滞', 'Terminal may be stalled')
  if (state === 'completed') return pickLocaleText('终端任务完成', 'Terminal task completed')
  if (state === 'error') return pickLocaleText('终端异常退出', 'Terminal exited unexpectedly')
  return pickLocaleText('终端提醒', 'Terminal alert')
}

function stateBody(state: SessionAttentionState) {
  if (state === 'needs-input') return pickLocaleText('需要你回到终端继续输入或确认。', 'Return to the terminal to continue input or confirm the next step.')
  if (state === 'stalled') return pickLocaleText('终端长时间无活动，建议人工查看。', 'The terminal has been inactive for a long time. Manual review is recommended.')
  if (state === 'completed') return pickLocaleText('终端任务已完成，建议查看输出结果。', 'The terminal task is complete. Review the output when convenient.')
  if (state === 'error') return pickLocaleText('终端异常退出，建议查看错误输出。', 'The terminal exited unexpectedly. Check the error output.')
  return pickLocaleText('终端状态已发生变化。', 'The terminal state has changed.')
}

async function ensurePermission() {
  if (permissionChecked) return permissionGranted

  try {
    permissionGranted = await isPermissionGranted()
    if (!permissionGranted) {
      permissionGranted = (await requestPermission()) === 'granted'
    }
  } catch {
    permissionGranted = false
  }

  permissionChecked = true
  return permissionGranted
}

export async function sendSessionAttentionNotification(payload: SessionAttentionNotification) {
  const granted = await ensurePermission()
  if (!granted) return false

  sendNotification({
    title: `${stateTitle(payload.state)} · ${payload.workspaceName}`,
    body: `${payload.tabName} / ${payload.sessionName}\n${stateBody(payload.state)}`,
  })

  return true
}
