import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import type { SessionAttentionState } from '../types/workspace'

export type SessionAttentionNotification = {
  workspaceName: string
  tabName: string
  sessionName: string
  state: SessionAttentionState
}

let permissionChecked = false
let permissionGranted = false

function stateTitle(state: SessionAttentionState) {
  if (state === 'needs-input') return '终端等待输入'
  if (state === 'stalled') return '终端疑似停滞'
  if (state === 'completed') return '终端任务完成'
  if (state === 'error') return '终端异常退出'
  return '终端提醒'
}

function stateBody(state: SessionAttentionState) {
  if (state === 'needs-input') return '需要你回到终端继续输入或确认。'
  if (state === 'stalled') return '终端长时间无活动，建议人工查看。'
  if (state === 'completed') return '终端任务已完成，建议查看输出结果。'
  if (state === 'error') return '终端异常退出，建议查看错误输出。'
  return '终端状态已发生变化。'
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
