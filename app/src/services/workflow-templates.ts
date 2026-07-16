import type { WorkflowTemplate } from '../types/workspace'
import { createId } from './workspace-storage'
import { pickLocaleText, type RuntimeLocale } from './runtime-locale'

const USER_TEMPLATES_STORAGE_KEY = 'chuchen-terminal.workflow-templates.v1'
const SYSTEM_TEMPLATE_CREATED_AT = '2026-06-02T00:00:00.000Z'

type WorkflowTemplateStorageRecord = {
  version: 1
  templates: WorkflowTemplate[]
}

export function getSystemWorkflowTemplates(locale?: RuntimeLocale): WorkflowTemplate[] {
  const tr = (zh: string, en: string) => pickLocaleText(zh, en, locale)

  return [
    {
      id: 'system-ai-cli',
      kind: 'system',
      category: 'ai-cli',
      name: 'AI CLI',
      description: tr('适合单独跑 Codex / Claude / Gemini 等 AI CLI，会创建一个 AI 协作终端。', 'Best for running a single Codex / Claude / Gemini style AI CLI session. It creates one AI collaboration terminal.'),
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
      name: tr('前端', 'Frontend'),
      description: tr('适合只需要一个前端开发终端的项目，例如 Vite / Vue / React / Tauri 前端。', 'Best for projects that only need a frontend development terminal, such as Vite / Vue / React / Tauri frontend work.'),
      tags: [tr('前端', 'Frontend')],
      panes: [
        {
          name: tr('前端', 'Frontend'),
          shellType: 'pwsh7',
          workingDirectoryHint: '.',
          defaultCommand: 'npm run dev',
          tags: [tr('前端', 'Frontend')],
        },
      ],
      createdAt: SYSTEM_TEMPLATE_CREATED_AT,
      updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
    },
    {
      id: 'system-backend',
      kind: 'system',
      category: 'backend',
      name: tr('后端', 'Backend'),
      description: tr('适合 Java / Go / Node 等后端项目，会创建一个后端服务终端。', 'Best for Java / Go / Node and similar backend projects. It creates one backend service terminal.'),
      tags: [tr('后端', 'Backend')],
      panes: [
        {
          name: tr('后端', 'Backend'),
          shellType: 'pwsh7',
          workingDirectoryHint: '.',
          defaultCommand: '',
          tags: [tr('后端', 'Backend')],
        },
      ],
      createdAt: SYSTEM_TEMPLATE_CREATED_AT,
      updatedAt: SYSTEM_TEMPLATE_CREATED_AT,
    },
    {
      id: 'system-fullstack-ai',
      kind: 'system',
      category: 'fullstack',
      name: tr('前端 + 后端 + AI CLI', 'Frontend + Backend + AI CLI'),
      description: tr('适合 AI 应用开发工作流，一次创建前端、后端和 AI 协作三个终端。', 'Best for AI application workflows. It creates frontend, backend, and AI collaboration terminals in one step.'),
      tags: [tr('前端', 'Frontend'), tr('后端', 'Backend'), 'AI'],
      panes: [
        {
          name: tr('前端', 'Frontend'),
          shellType: 'pwsh7',
          workingDirectoryHint: 'frontend',
          defaultCommand: 'npm run dev',
          tags: [tr('前端', 'Frontend')],
        },
        {
          name: tr('后端', 'Backend'),
          shellType: 'pwsh7',
          workingDirectoryHint: 'backend',
          defaultCommand: '',
          tags: [tr('后端', 'Backend')],
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
}

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
