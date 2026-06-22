import type { WorkflowTemplate } from '../types/workspace'
import { createId } from './workspace-storage'

const USER_TEMPLATES_STORAGE_KEY = 'chuchen-terminal.workflow-templates.v1'
const SYSTEM_TEMPLATE_CREATED_AT = '2026-06-02T00:00:00.000Z'

type WorkflowTemplateStorageRecord = {
  version: 1
  templates: WorkflowTemplate[]
}

export const systemWorkflowTemplates: WorkflowTemplate[] = [
  {
    id: 'system-ai-cli',
    kind: 'system',
    category: 'ai-cli',
    name: 'AI CLI',
    description: '适合单独跑 Codex / Claude / Gemini 等 AI CLI，会创建一个 AI 协作终端。',
    tags: ['AI', 'CLI'],
    panes: [
      {
        name: 'Codex',
        shellType: 'pwsh7',
        workingDirectoryHint: '.',
        defaultCommand: 'codex',
        tags: ['AI', 'Codex'],
      },
    ],
    createdAt: SYSTEM_TEMPLATE_CREATED_AT,
    updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
  },
  {
    id: 'system-frontend',
    kind: 'system',
    category: 'frontend',
    name: '前端',
    description: '适合只需要一个前端开发终端的项目，例如 Vite / Vue / React / Tauri 前端。',
    tags: ['前端'],
    panes: [
      {
        name: '前端',
        shellType: 'pwsh7',
        workingDirectoryHint: '.',
        defaultCommand: 'npm run dev',
        tags: ['前端'],
      },
    ],
    createdAt: SYSTEM_TEMPLATE_CREATED_AT,
    updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
  },
  {
    id: 'system-backend',
    kind: 'system',
    category: 'backend',
    name: '后端',
    description: '适合 Java / Go / Node 等后端项目，会创建一个后端服务终端。',
    tags: ['后端'],
    panes: [
      {
        name: '后端',
        shellType: 'pwsh7',
        workingDirectoryHint: '.',
        defaultCommand: '',
        tags: ['后端'],
      },
    ],
    createdAt: SYSTEM_TEMPLATE_CREATED_AT,
    updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
  },
  {
    id: 'system-fullstack-ai',
    kind: 'system',
    category: 'fullstack',
    name: '前端 + 后端 + AI CLI',
    description: '适合 AI 应用开发工作流，一次创建前端、后端和 AI 协作三个终端。',
    tags: ['前端', '后端', 'AI'],
    panes: [
      {
        name: '前端',
        shellType: 'pwsh7',
        workingDirectoryHint: 'frontend',
        defaultCommand: 'npm run dev',
        tags: ['前端'],
      },
      {
        name: '后端',
        shellType: 'pwsh7',
        workingDirectoryHint: 'backend',
        defaultCommand: '',
        tags: ['后端'],
      },
      {
        name: 'Codex',
        shellType: 'pwsh7',
        workingDirectoryHint: '.',
        defaultCommand: 'codex',
        tags: ['AI', 'Codex'],
      },
    ],
    createdAt: SYSTEM_TEMPLATE_CREATED_AT,
    updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
  },
]

export function loadUserWorkflowTemplates(): WorkflowTemplate[] {
  if (typeof window === 'undefined' || !window.localStorage) {
    return []
  }

  try {
    const raw = window.localStorage.getItem(USER_TEMPLATES_STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as WorkflowTemplateStorageRecord
    if (!Array.isArray(parsed.templates)) return []
    return parsed.templates
      .filter((template) => template.kind === 'user' && template.id && template.name)
      .map(normalizeUserTemplate)
  } catch {
    return []
  }
}

export function saveUserWorkflowTemplates(templates: WorkflowTemplate[]) {
  if (typeof window === 'undefined' || !window.localStorage) {
    return
  }

  const payload: WorkflowTemplateStorageRecord = {
    version: 1,
    templates: templates.filter((template) => template.kind === 'user').map(normalizeUserTemplate),
  }

  window.localStorage.setItem(USER_TEMPLATES_STORAGE_KEY, JSON.stringify(payload))
}

export function createWorkflowTemplateFromInput(input: {
  name: string
  description: string
  tags: string[]
  panes: WorkflowTemplate['panes']
}): WorkflowTemplate {
  const now = new Date().toISOString()

  return {
    id: createId('template'),
    kind: 'user',
    category: 'custom',
    name: input.name.trim(),
    description: input.description.trim(),
    tags: input.tags,
    panes: input.panes,
    createdAt: now,
    updatedAt: now,
  }
}

function normalizeUserTemplate(template: WorkflowTemplate): WorkflowTemplate {
  const now = new Date().toISOString()
  return {
    ...template,
    kind: 'user',
    category: template.category ?? 'custom',
    description: template.description ?? '',
    tags: Array.isArray(template.tags) ? template.tags : [],
    panes: Array.isArray(template.panes) ? template.panes.map((pane) => ({
      name: pane.name || 'PowerShell 7',
      shellType: pane.shellType ?? 'pwsh7',
      workingDirectoryHint: pane.workingDirectoryHint || '.',
      defaultCommand: pane.defaultCommand || '',
      tags: Array.isArray(pane.tags) ? pane.tags : [],
    })) : [],
    createdAt: template.createdAt || now,
    updatedAt: template.updatedAt || template.createdAt || now,
  }
}
