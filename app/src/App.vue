<template>
  <div class="app-shell" :class="appShellClasses" :style="appThemeVars">
    <header class="titlebar">
      <div class="titlebar__brand">
        <div class="titlebar__logo">CT</div>
        <div>
          <strong>Chuchen-Terminal</strong>
          <p>Windows-first 本地终端工作台</p>
        </div>
      </div>
      <div class="titlebar__meta titlebar__statusbar">
        <button
          type="button"
          class="titlebar__status-meta"
          :class="{ 'titlebar__status-meta--button': systemRefreshInterval === 'manual' }"
          :disabled="systemRefreshInterval !== 'manual' && !systemStatusRefreshing"
          @click="systemRefreshInterval === 'manual' ? refreshSystemStatus() : undefined"
        >
          <AppIcon :name="systemStatusRefreshing ? 'refresh' : 'recent'" :size="13" :class="{ 'is-spinning': systemStatusRefreshing }" />
          <span>{{ systemRefreshLabel }}</span>
        </button>
        <span class="titlebar__status-pill"><AppIcon name="runtime" :size="13" /> <strong>CPU</strong> {{ systemStatus.cpu }}</span>
        <span class="titlebar__status-pill"><AppIcon name="workspace" :size="13" /> <strong>内存</strong> {{ systemStatus.memory }}</span>
        <span class="titlebar__status-pill"><AppIcon name="theme" :size="13" /> <strong>GPU</strong> {{ systemStatus.gpu }}</span>
        <span v-if="attentionSessionCount" class="titlebar__status-pill titlebar__status-pill--attention">
          <AppIcon name="recent" :size="13" />
          <strong>待处理</strong> {{ attentionSessionCount }}
        </span>
        <button type="button" class="titlebar__status-action" @click="openThemePanel('system')">
          <AppIcon name="settings" :size="13" />
          <span>系统</span>
        </button>
      </div>
    </header>

    <div class="layout" :class="{ 'layout--rail-collapsed': railCollapsed }">
      <aside class="rail" :class="{ 'rail--collapsed': railCollapsed }">
        <div class="rail__group rail__group--top">
          <div
            v-for="item in primaryRailItems"
            :key="item.id"
            class="rail__entry"
            @mouseenter="activeRailTooltipId = item.id"
            @mouseleave="activeRailTooltipId = null"
          >
            <button
              class="rail__item rail__item--wide"
              :class="{ 'rail__item--active': item.active }"
              @click="item.action()"
            >
              <AppIcon :name="item.icon" :size="18" />
              <span v-if="!railCollapsed">{{ item.label }}</span>
            </button>

            <div v-if="activeRailTooltipId === item.id" class="rail-tooltip">
              <div class="rail-tooltip__content">
                <strong>{{ item.label }}</strong>
                <p>{{ item.summary }}</p>
              </div>
              <button type="button" class="rail-tooltip__help" @click.stop="openHelpTopic(item.id)">
                <AppIcon name="info" :size="14" />
              </button>
            </div>
          </div>
        </div>

        <div class="rail__spacer"></div>

        <div class="rail__group rail__group--bottom">
          <div
            v-for="item in secondaryRailItems"
            :key="item.id"
            class="rail__entry"
            @mouseenter="activeRailTooltipId = item.id"
            @mouseleave="activeRailTooltipId = null"
          >
            <button
              class="rail__item rail__item--wide"
              :class="{ 'rail__item--active': item.active }"
              @click="item.action()"
            >
              <AppIcon :name="item.icon" :size="18" />
              <span v-if="!railCollapsed">{{ item.label }}</span>
            </button>

            <div v-if="activeRailTooltipId === item.id" class="rail-tooltip">
              <div class="rail-tooltip__content">
                <strong>{{ item.label }}</strong>
                <p>{{ item.summary }}</p>
              </div>
              <button type="button" class="rail-tooltip__help" @click.stop="openHelpTopic(item.id)">
                <AppIcon name="info" :size="14" />
              </button>
            </div>
          </div>

          <button class="rail__collapse" :title="railCollapsed ? '展开菜单' : '收起菜单'" @click="toggleRailCollapsed()">
            <AppIcon name="chevron-right" :size="15" />
            <span v-if="!railCollapsed">{{ railCollapsed ? '展开菜单' : '收起菜单' }}</span>
          </button>
        </div>
      </aside>

      <main class="workspace">
        <section v-if="!isWorkspaceWorkbench" class="topbar">
          <div class="breadcrumbs breadcrumbs--pixel">
            <button
              v-for="(item, index) in breadcrumbItems"
              :key="item.key"
              class="breadcrumb"
              :class="{ 'breadcrumb--current': index === breadcrumbItems.length - 1 }"
              @click="item.onClick?.()"
            >
              <AppIcon :name="item.icon" :size="16" />
              <span>{{ item.label }}</span>
              <AppIcon v-if="index < breadcrumbItems.length - 1" name="chevron-right" :size="12" />
            </button>
          </div>

          <div class="topbar__actions">
            <template v-if="isWorkspaceWorkbench">
              <div class="topbar__view-actions">
                <button class="ghost-btn ghost-btn--small" @click="openTerminalEntriesModal = true">
                  <AppIcon name="terminal" :size="14" />
                  <span>运行配置</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="openHelpTopic('layout')">
                  <AppIcon name="info" :size="14" />
                  <span>结构说明</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="goWorkspaceOverview">
                  <AppIcon name="workspace" :size="14" />
                  <span>返回卡片</span>
                </button>
              </div>
            </template>
          </div>
        </section>

        <section class="home-dashboard" v-if="appSection === 'home'">
          <section class="home-hero panel">
            <div class="home-hero__copy">
              <h2>欢迎回来</h2>
              <p>从最近项目、环境状态和系统资源开始，快速进入终端工作台。</p>
            </div>
            <div class="home-hero__meta">
              <span>最近使用：{{ recentHomeWorkspaces[0]?.name || '暂无' }}</span>
              <span>默认 Shell：PowerShell 7</span>
              <span>桌面壳：Tauri 2 已接入</span>
            </div>
            <div class="home-hero__stats">
              <div class="home-stat">
                <strong>{{ totalWorkspaceCount }}</strong>
                <span>工作区</span>
              </div>
              <div class="home-stat">
                <strong>{{ totalPaneCount }}</strong>
                <span>终端 Pane</span>
              </div>
              <div class="home-stat home-stat--running">
                <strong>{{ totalRunningCount }}</strong>
                <span>运行中</span>
              </div>
            </div>
            <div class="home-hero__quick">
              <button class="ghost-btn ghost-btn--primary" @click="openWorkspaceCreateModal()">
                <AppIcon name="workspace" :size="15" />
                <span>新建工作区</span>
              </button>
              <button class="ghost-btn" @click="openQuickSearch()">
                <AppIcon name="search" :size="15" />
                <span>搜索</span>
              </button>
              <button class="ghost-btn" @click="openThemePanel('theme')">
                <AppIcon name="theme" :size="15" />
                <span>主题</span>
              </button>
            </div>
          </section>

          <section class="home-grid">
            <article class="home-card home-card--recent">
              <div class="home-card__header">
                <div>
                  <h3>最近工作区</h3>
                  <p>优先展示最近进入的项目容器。</p>
                </div>
                <button class="ghost-btn ghost-btn--small" @click="goWorkspaceOverview">
                  <AppIcon name="workspace" :size="14" />
                  <span>全部</span>
                </button>
              </div>
              <div class="home-recent-list">
                <button
                  v-for="workspace in recentHomeWorkspaces"
                  :key="workspace.id"
                  type="button"
                  class="home-recent-item"
                  @click="openWorkspace(workspace.id)"
                >
                  <div>
                    <strong>{{ workspace.name }}</strong>
                    <span>{{ workspace.rootPath }}</span>
                  </div>
                  <small>{{ workspace.tabs.length }} 项目 · {{ totalPanes(workspace) }} Pane · {{ totalWorkspaceSessions(workspace) }} 终端</small>
                </button>
              </div>
            </article>

            <article class="home-card home-card--env">
              <div class="home-card__header">
                <div>
                  <h3>环境与资源</h3>
                  <p>把本地环境、项目依赖和系统资源集中在一块展示。</p>
                </div>
                <button type="button" class="ghost-btn ghost-btn--small" @click="openThemePanel('system')">
                  <AppIcon name="settings" :size="14" />
                  <span>系统</span>
                </button>
              </div>
              <div class="home-check-list">
                <div v-for="item in visibleEnvironmentChecks" :key="item.name" class="home-check-item" :class="`home-check-item--${item.status}`">
                  <span class="home-env-icon" :class="{ 'home-env-icon--mono': item.monochrome }">
                    <img v-if="item.iconSrc" :src="item.iconSrc" :alt="`${item.name} 图标`" />
                    <span v-else>{{ item.icon }}</span>
                  </span>
                  <div>
                    <strong>{{ item.name }}</strong>
                    <small>{{ item.note }}</small>
                  </div>
                  <span class="home-check-badge">{{ item.value }}</span>
                </div>
              </div>
            </article>
          </section>
        </section>

        <section class="workspace-overview" v-else-if="appSection === 'workspace' && workspaceView === 'overview'">
          <aside class="workspace-overview__list panel">
            <div class="workspace-overview__list-head">
              <div>
                <h2>工作区总览</h2>
                <span>{{ workspaces.length }} 个工作区 · {{ openedWorkspaces.length }} 个已打开</span>
              </div>
              <button class="icon-btn" title="新建工作区" @click="openWorkspaceCreateModal()">
                <AppIcon name="plus" :size="15" />
              </button>
            </div>
            <div class="workspace-overview__items">
              <div
              v-for="workspace in workspaces"
              :key="workspace.id"
              class="workspace-overview__item"
              :class="{ 'workspace-overview__item--active': activeOverviewWorkspace?.id === workspace.id }"
            >
              <button
                type="button"
                class="workspace-overview__item-main"
                @click="selectOverviewWorkspace(workspace.id)"
              >
                <div>
                  <strong>{{ workspace.name }}</strong>
                  <span>{{ workspace.rootPath }}</span>
                </div>
                <small>{{ runningCount(workspace) }} 运行 · {{ workspace.tabs.length }} 项目 · {{ totalPanes(workspace) }} 终端 · {{ workspaceGitBranchLabel(workspace) }}</small>
              </button>
              <div class="workspace-overview__item-actions">
                <button
                  type="button"
                  class="icon-btn icon-btn--mini workspace-overview__item-action workspace-overview__item-action--edit"
                  title="编辑工作区"
                  @click.stop="openWorkspaceEditModal(workspace.id)"
                >
                  <AppIcon name="edit" :size="13" />
                </button>
                <button
                  type="button"
                  class="icon-btn icon-btn--mini workspace-overview__item-action workspace-overview__item-action--delete"
                  title="删除工作区"
                  @click.stop="removeWorkspace(workspace.id)"
                >
                  <AppIcon name="trash" :size="13" />
                </button>
              </div>
            </div>
            </div>
          </aside>

          <section class="workspace-carousel panel">
            <div class="workspace-carousel__header">
              <div>
                <h2>工作区轮播</h2>
                <span>通过左侧列表或箭头快速定位工作区。</span>
              </div>
              <div class="workspace-carousel__actions">
                <span class="meta-badge meta-badge--soft">{{ overviewWorkspaceIndex + 1 }} / {{ workspaces.length || 1 }}</span>
                <button class="ghost-btn ghost-btn--small" @click="openQuickSearch()">
                  <AppIcon name="search" :size="14" />
                  <span>搜索</span>
                </button>
                <button class="ghost-btn ghost-btn--small ghost-btn--primary" @click="openWorkspaceCreateModal()">
                  <AppIcon name="workspace" :size="14" />
                  <span>新建</span>
                </button>
              </div>
            </div>

            <div class="workspace-carousel__stage" v-if="activeOverviewWorkspace">
              <transition name="carousel-ghost" mode="out-in">
                <button
                  v-if="previousOverviewWorkspace"
                  :key="`previous-${previousOverviewWorkspace.id}`"
                  type="button"
                  class="workspace-carousel__ghost workspace-carousel__ghost--previous"
                  @click="shiftOverviewWorkspace('previous')"
                >
                  <strong>{{ previousOverviewWorkspace.name }}</strong>
                  <span>{{ previousOverviewWorkspace.rootPath }}</span>
                </button>
              </transition>

              <transition name="carousel-card" mode="out-in">
              <article :key="activeOverviewWorkspace.id" class="workspace-carousel__card">
                <div class="workspace-carousel__card-top">
                  <div>
                    <h3>{{ activeOverviewWorkspace.name }}</h3>
                    <p>{{ activeOverviewWorkspace.description || '暂无描述' }}</p>
                  </div>
                  <span class="meta-badge">{{ relativeTimeLabel(activeOverviewWorkspace.lastOpenedAt) }}</span>
                </div>
                <div class="path-row workspace-carousel__path">{{ activeOverviewWorkspace.rootPath }}</div>
                <div class="workspace-carousel__metrics">
                  <span>{{ activeOverviewWorkspace.tabs.length }} 项目</span>
                  <span>{{ totalPanes(activeOverviewWorkspace) }} 终端</span>
                  <span class="meta-running">运行中 {{ runningCount(activeOverviewWorkspace) }}</span>
                  <span>{{ workspaceGitBranchLabel(activeOverviewWorkspace) }}</span>
                </div>
                <div class="workspace-card__tags">
                  <span v-for="tag in activeOverviewWorkspace.tags" :key="tag" class="tag-chip">{{ tag }}</span>
                  <span v-if="!activeOverviewWorkspace.tags.length" class="tag-chip tag-chip--soft">未设置标签</span>
                </div>
                <div class="workspace-carousel__preview">
                  <div v-for="tab in activeOverviewWorkspace.tabs.slice(0, 3)" :key="tab.id" class="workspace-carousel__tab-row">
                    <strong>{{ tab.name }}</strong>
                    <span>{{ countLeafPanes(tab.panes) }} Pane · {{ countPaneSessions(tab.panes) }} 终端</span>
                    <div class="mini-runtime-panes">
                      <span
                        v-for="pane in flattenLeafPanes(tab.panes).slice(0, 4)"
                        :key="pane.id"
                        class="mini-pane"
                        :class="{ 'mini-pane--running': workspacePaneRunning(activeOverviewWorkspace, pane) }"
                      ></span>
                      <span v-if="!countLeafPanes(tab.panes)" class="mini-pane mini-pane--empty"></span>
                    </div>
                  </div>
                </div>
                <div class="workspace-carousel__buttons">
                  <button type="button" class="card-action-btn card-action-btn--primary" @click="openWorkspace(activeOverviewWorkspace.id)">打开工作区</button>
                  <button type="button" class="card-action-btn" @click="openWorkspaceEditModal(activeOverviewWorkspace.id)">编辑工作区</button>
                  <button type="button" class="card-action-btn card-action-btn--danger" @click="removeWorkspace(activeOverviewWorkspace.id)">删除工作区</button>
                </div>
              </article>
              </transition>

              <transition name="carousel-ghost" mode="out-in">
                <button
                  v-if="nextOverviewWorkspace"
                  :key="`next-${nextOverviewWorkspace.id}`"
                  type="button"
                  class="workspace-carousel__ghost workspace-carousel__ghost--next"
                  @click="shiftOverviewWorkspace('next')"
                >
                  <strong>{{ nextOverviewWorkspace.name }}</strong>
                  <span>{{ nextOverviewWorkspace.rootPath }}</span>
                </button>
              </transition>

              <button v-if="previousOverviewWorkspace" type="button" class="workspace-carousel__arrow workspace-carousel__arrow--left" @click="shiftOverviewWorkspace('previous')">
                ‹
              </button>
              <button v-if="nextOverviewWorkspace" type="button" class="workspace-carousel__arrow workspace-carousel__arrow--right" @click="shiftOverviewWorkspace('next')">
                ›
              </button>
            </div>
          </section>
        </section>
        <section v-else-if="isWorkspaceWorkbench && selectedWorkspace" class="workbench-shell" :style="workbenchShellStyle">
          <aside class="workbench-sidebar">
            <section class="explorer-shell">
              <div class="explorer-shell__header">
                <div>
                  <h3>工作区 Explorer</h3>
                </div>
              </div>

              <div class="explorer-list">
                <article
                  v-for="workspace in openedWorkspaces"
                  :key="workspace.id"
                  class="explorer-workspace"
                  :class="{ 'explorer-workspace--active': workspace.id === selectedWorkspaceId }"
                >
                  <div class="explorer-workspace__summary">
                    <button type="button" class="explorer-workspace__button" @click="handleWorkspaceSummaryClick(workspace.id)">
                      <AppIcon
                        class="explorer-workspace__chevron"
                        :class="{ 'explorer-workspace__chevron--open': workspace.id === selectedWorkspaceId && !isWorkspaceCollapsed(workspace.id) }"
                        name="chevron-right"
                        :size="13"
                      />
                      <div class="explorer-workspace__content">
                        <div class="explorer-workspace__title-row">
                          <strong>{{ workspace.name }}</strong>
                        </div>
                        <div class="explorer-workspace__path">
                          <AppIcon name="folder" :size="13" />
                          <span>{{ workspace.rootPath }}</span>
                        </div>
                        <div class="explorer-workspace__stats">
                          <span><AppIcon name="recent" :size="12" />{{ relativeTimeLabel(workspace.lastOpenedAt) }}</span>
                          <span><AppIcon name="tab" :size="12" />{{ workspace.tabs.length }} 项目</span>
                          <span><AppIcon name="pane" :size="12" />{{ totalPanes(workspace) }} 终端</span>
                        </div>
                        <div class="explorer-workspace__branch">
                          <AppIcon name="copy" :size="12" />
                          <span>{{ workspaceGitBranchLabel(workspace) }}</span>
                        </div>
                      </div>
                    </button>

                    <div class="explorer-workspace__menu-wrap">
                      <button class="icon-btn icon-btn--mini" title="工作区操作" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleWorkspaceMenu(workspace.id, $event)">
                        <AppIcon name="more" :size="14" />
                      </button>
                      <PopoverMenu :open="activeWorkspaceMenu === workspace.id" :items="workspaceMenuItems(workspace)" :position="activeWorkspaceMenuPosition" />
                    </div>
                  </div>

                  <div v-if="workspace.id === selectedWorkspaceId && !isWorkspaceCollapsed(workspace.id)" class="explorer-project-list">
                    <article
                      v-for="tab in workspace.tabs"
                      :key="tab.id"
                      class="explorer-project"
                      :class="[
                        { 'explorer-project--active': activeRuntimeTabId === tab.id },
                        explorerProjectAttentionClass(workspace, tab),
                      ]"
                    >
                      <div class="explorer-project__header">
                        <button
                          type="button"
                          class="explorer-project__toggle"
                          :title="isTreeTabCollapsed(tab.id) ? '展开项目' : '收起项目'"
                          @click.stop="toggleTreeTab(tab.id)"
                        >
                          <AppIcon
                            class="explorer-project__chevron"
                            :class="{ 'explorer-project__chevron--open': !isTreeTabCollapsed(tab.id) }"
                            name="chevron-right"
                            :size="12"
                          />
                        </button>
                        <div
                          role="button"
                          tabindex="0"
                          class="explorer-project__button"
                          @click="openProjectWorkspace(workspace.id, tab.id)"
                          @keydown.enter.prevent="openProjectWorkspace(workspace.id, tab.id)"
                          @keydown.space.prevent="openProjectWorkspace(workspace.id, tab.id)"
                          @contextmenu.prevent.stop="toggleExplorerProjectMenu(workspace.id, tab.id, $event)"
                        >
                          <div class="explorer-project__summary">
                            <div class="explorer-project__title">
                              <AppIcon name="tab" :size="14" />
                              <strong>{{ tab.name }}</strong>
                              <span
                                v-if="explorerProjectAttentionBadge(workspace, tab)"
                                class="explorer-attention-badge"
                                :class="`explorer-attention-badge--${explorerProjectAttentionState(workspace, tab)}`"
                              >
                                {{ explorerProjectAttentionBadge(workspace, tab) }}
                              </span>
                            </div>
                            <div class="explorer-project__stats">
                              <span>{{ countPaneSessions(tab.panes) }} 个终端</span>
                              <span>{{ tabRunningCount(workspace, tab) }} 运行中</span>
                            </div>
                          </div>
                          <div class="explorer-project__actions">
                            <button
                              type="button"
                              class="icon-btn icon-btn--mini explorer-project__action"
                              title="重命名项目"
                              @pointerdown="handleMenuTriggerPointerDown"
                              @click.stop="openExplorerTabRename(workspace.id, tab.id)"
                            >
                              <AppIcon name="edit" :size="13" />
                            </button>
                            <button
                              type="button"
                              class="icon-btn icon-btn--mini explorer-project__action"
                              title="项目操作"
                              @pointerdown="handleMenuTriggerPointerDown"
                              @click.stop="toggleExplorerProjectMenu(workspace.id, tab.id, $event)"
                            >
                              <AppIcon name="more" :size="13" />
                            </button>
                          </div>
                        </div>
                      </div>
                      <PopoverMenu :open="activeExplorerProjectMenuId === tab.id && activeExplorerProjectWorkspaceId === workspace.id" :items="explorerProjectMenuItems" :position="activeExplorerProjectMenuPosition" />

                      <ul v-if="!isTreeTabCollapsed(tab.id)" class="explorer-pane-list">
                        <li
                          v-for="item in explorerSessionItems(workspace, tab)"
                          :key="`${item.pane.id}-${item.session.id}`"
                          class="explorer-pane-item"
                        >
                          <button
                            type="button"
                            class="explorer-pane"
                            :class="[
                              { 'explorer-pane--active': activeRuntimePaneId === item.pane.id && currentActiveRuntimeSessionId() === item.session.id },
                              { 'explorer-pane--supervised': sessionIsSupervised(item.session) },
                              explorerSessionAttentionClass(item.session),
                            ]"
                            @click="openWorkspaceTerminalSession(workspace.id, tab.id, item.pane.id, item.session.id)"
                          >
                            <span class="status-dot" :class="`status-dot--${explorerSessionTone(item.session)}`"></span>
                            <div class="explorer-pane__body">
                              <div class="explorer-pane__title-row">
                                <strong>{{ item.session.name }}</strong>
                                <span class="explorer-pane__status">{{ explorerSessionLabel(item.session) }}</span>
                              </div>
                              <span class="explorer-pane__path">{{ workspaceEntryById(workspace, item.session.terminalEntryId)?.workingDirectory || item.session.pathLabel || item.pane.pathLabel }}</span>
                            </div>
                            <div v-if="(workspaceEntryById(workspace, item.session.terminalEntryId)?.tags ?? []).length" class="explorer-pane__tags">
                              <span
                                v-for="tag in (workspaceEntryById(workspace, item.session.terminalEntryId)?.tags ?? []).slice(0, 2)"
                                :key="`${item.session.id}-${tag}`"
                                class="tag-chip tag-chip--soft"
                              >
                                {{ tag }}
                              </span>
                            </div>
                          </button>
                        </li>
                        <li v-if="!explorerSessionItems(workspace, tab).length" class="tree-empty tree-empty--compact">当前项目还没有终端。</li>
                      </ul>
                    </article>

                    <button type="button" class="explorer-list-action explorer-list-action--project" @click="createTab()">
                      <AppIcon name="tab" :size="14" />
                      <span>新建项目</span>
                    </button>
                  </div>
                </article>

                <button type="button" class="explorer-list-action explorer-list-action--workspace" @click="openWorkspaceCreateModal()">
                  <AppIcon name="workspace" :size="14" />
                  <span>新建工作区</span>
                </button>
              </div>
            </section>
          </aside>

          <div
            class="workbench-resizer"
            title="拖拽调整 Explorer 宽度"
            @pointerdown="startWorkbenchResize"
          ></div>

          <section class="workbench-main">
            <div class="runtime-header runtime-header--workbench">
              <div class="runtime-header__workspace">
                <div class="runtime-header__copy">
                  <h2>{{ selectedWorkspace.name }}</h2>
                  <p>{{ selectedWorkspace.description || selectedWorkspace.rootPath }}</p>
                </div>
              </div>

              <div class="runtime-tabs runtime-tabs--embedded">
                <button
                  v-for="tab in selectedWorkspace.tabs"
                  :key="tab.id"
                  class="tab"
                  :class="{ 'tab--active': activeRuntimeTabId === tab.id }"
                  @click="openProjectWorkspace(selectedWorkspace.id, tab.id)"
                  @contextmenu.prevent.stop="toggleRuntimeTabMenu(tab.id, $event)"
                >
                  <AppIcon name="tab" :size="15" />
                  <span>{{ runtimeTabLabel(tab) }}</span>
                  <small class="tab-badge">{{ countPaneSessions(tab.panes) }}</small>
                </button>
                <button type="button" class="tab tab--create" title="新建项目" @click="createTab()">
                  <AppIcon name="plus" :size="14" />
                </button>
                <PopoverMenu :open="Boolean(activeRuntimeTabMenuId)" :items="runtimeTabMenuItems" :position="activeRuntimeTabMenuPosition" />
                
              </div>
            </div>

            <div ref="runtimeCanvasRef" class="runtime-canvas runtime-canvas--tree">
              <template v-if="activeRuntimeTab && countLeafPanes(activeRuntimeTab.panes)">
                <template v-for="pane in activeRuntimeTab.panes" :key="pane.id">
                  <PaneTreeNode :pane="pane" />
                </template>
              </template>

              <section v-else class="pane pane--empty pane--empty-full">
                <div class="pane__body pane__body--empty-state">
                  <div class="terminal-ready-state">
                    <div class="terminal-ready-state__icon">
                      <AppIcon name="terminal" :size="28" />
                    </div>
                    <strong>准备就绪</strong>
                    <span>当前项目还没有终端，可先在左侧切换项目，或创建一个新的终端窗口。</span>
                    <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" @click="createPane()">
                      <AppIcon name="pane" :size="14" />
                      <span>新建终端</span>
                    </button>
                  </div>
                </div>
              </section>
            </div>

            <div class="runtime-footer">
              <div class="runtime-footer__meta">
                <div>当前工作区：{{ selectedWorkspace.name }}</div>
                <div>当前项目：{{ activeRuntimeTab?.name || '未选择' }}</div>
                <div>终端数：{{ countPaneSessions(activeRuntimeTab?.panes || []) }}</div>
                <div v-if="defaultWorkspaceSnapshot">最近现场：{{ defaultWorkspaceSnapshot.name }}</div>
              </div>
              <div class="runtime-footer__actions">
                <div class="restore-strategy" role="group" aria-label="恢复策略">
                  <button
                    v-for="option in restoreCommandStrategyOptions"
                    :key="option.value"
                    type="button"
                    class="restore-strategy__item"
                    :class="{ 'restore-strategy__item--active': restoreCommandStrategy === option.value }"
                    :title="option.description"
                    @click="restoreCommandStrategy = option.value"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <button type="button" class="ghost-btn ghost-btn--small" @click="saveCurrentWorkspaceSnapshot()">
                  <AppIcon name="copy" :size="14" />
                  <span>保存现场</span>
                </button>
                <button
                  type="button"
                  class="ghost-btn ghost-btn--small"
                  :disabled="!activeRuntimeTab || !countLeafPanes(activeRuntimeTab.panes)"
                  @click="saveActiveTabAsWorkflowTemplate()"
                >
                  <AppIcon name="template" :size="14" />
                  <span>保存为模板</span>
                </button>
                <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" :disabled="!defaultWorkspaceSnapshot" @click="restoreDefaultWorkspaceSnapshot()">
                  <AppIcon name="refresh" :size="14" />
                  <span>恢复现场</span>
                </button>
              </div>
            </div>
          </section>
        </section>
        <section class="detail-layout" v-else-if="appSection === 'workspace' && workspaceView === 'detail' && selectedWorkspace">
          <aside class="detail-side">
            <div class="side-card side-card--summary">
              <div class="summary-hero">
                <div class="summary-hero__head">
                  <span class="meta-badge meta-badge--soft summary-badge">工作区</span>
                  <h3>{{ selectedWorkspace.name }}</h3>
                  <div class="summary-path">
                    <AppIcon name="folder" :size="15" />
                    <span>{{ selectedWorkspace.rootPath }}</span>
                  </div>
                </div>
                <div class="summary-metrics">
                  <div class="summary-metric">
                    <span class="summary-metric__label"><AppIcon name="recent" :size="14" />最近使用</span>
                    <strong>{{ relativeTimeLabel(selectedWorkspace.lastOpenedAt) }}</strong>
                  </div>
                  <div class="summary-metric">
                    <span class="summary-metric__label"><AppIcon name="tab" :size="14" />Tab</span>
                    <strong>{{ selectedWorkspace.tabs.length }}</strong>
                  </div>
                  <div class="summary-metric">
                    <span class="summary-metric__label"><AppIcon name="pane" :size="14" />Pane</span>
                    <strong>{{ totalPanes(selectedWorkspace) }}</strong>
                  </div>
                </div>
                <div v-if="selectedWorkspace.tags.length" class="summary-tags">
                  <span v-for="tag in selectedWorkspace.tags" :key="tag" class="tag-chip">{{ tag }}</span>
                </div>
              </div>
              <div class="side-actions">
                <button class="ghost-btn ghost-btn--small" @click="openTerminalEntriesModal = true">
                  <AppIcon name="terminal" :size="14" />
                  <span>运行配置</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="openWorkspaceEditModal(selectedWorkspace.id)">编辑工作区</button>
                <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeWorkspace(selectedWorkspace.id)">删除工作区</button>
              </div>
            </div>

            <div class="side-card">
              <div class="side-card__header side-card__header--tree">
                <h3>层级结构</h3>
                <div class="tree-actions">
                  <button class="ghost-btn ghost-btn--small" @click="expandAllTreeTabs()">全部展开</button>
                  <button class="ghost-btn ghost-btn--small" @click="collapseAllTreeTabs()">全部收起</button>
                </div>
              </div>
              <ul class="tree-list">
                <li v-for="tab in selectedWorkspace.tabs" :key="tab.id" class="tree-node tree-node--tab">
                  <button type="button" class="tree-node__row tree-node__row--tab" @click="toggleTreeTab(tab.id)">
                    <AppIcon class="tree-node__chevron" :class="{ 'tree-node__chevron--open': !isTreeTabCollapsed(tab.id) }" name="chevron-right" :size="12" />
                    <AppIcon name="tab" :size="15" />
                    <strong>{{ tab.name }}</strong>
                    <span class="tree-meta">{{ countLeafPanes(tab.panes) }} 个 Pane</span>
                  </button>
                  <ul v-if="countLeafPanes(tab.panes) && !isTreeTabCollapsed(tab.id)" class="tree-list tree-list--nested">
                    <li
                      v-for="pane in flattenLeafPanes(tab.panes)"
                      :key="pane.id"
                      class="tree-node tree-node--pane"
                      :class="{ 'tree-node--running': paneHasRunningSession(pane) }"
                    >
                      <button type="button" class="tree-node__row tree-node__row--pane" @click="openRuntimeFromPane(tab.id, pane.id)">
                        <AppIcon name="pane" :size="15" />
                        <span>{{ pane.name }}</span>
                        <span class="status-dot" :class="{ 'status-dot--running': paneHasRunningSession(pane) }"></span>
                        <span class="tree-status">{{ paneHasRunningSession(pane) ? '运行中' : '待命' }}</span>
                      </button>
                    </li>
                  </ul>
                  <div v-else-if="!isTreeTabCollapsed(tab.id)" class="empty-inline">当前 Tab 暂无 Pane。</div>
                </li>
              </ul>
            </div>

          </aside>

          <div class="detail-main">
            <article v-for="tab in selectedWorkspace.tabs" :key="tab.id" class="tab-block">
              <div class="tab-block__head">
                <div class="tab-block__title">
                  <AppIcon name="tab" :size="16" />
                  <h3>{{ tab.name }}</h3>
                  <span class="tab-label">{{ countLeafPanes(tab.panes) }} 个 Pane · {{ countPaneSessions(tab.panes) }} 个终端</span>
                </div>
                <div class="tab-block__actions">
                  <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeTab(tab.id)">删除 Tab</button>
                </div>
              </div>

              <div class="pane-lines pane-lines--stacked" v-if="countLeafPanes(tab.panes)">
                <div
                  v-for="pane in flattenLeafPanes(tab.panes)"
                  :key="pane.id"
                  class="pane-line"
                  :class="{ 'pane-line--running': paneHasRunningSession(pane), 'pane-line--interactive': true, 'pane-line--selected': activeRuntimePaneId === pane.id }"
                  @click="openRuntimeFromPane(tab.id, pane.id)"
                >
                  <div class="pane-line__head">
                    <div class="pane-line__title">
                      <AppIcon name="pane" :size="15" />
                      <strong>{{ pane.name }}</strong>
                      <span class="status-badge" :class="{ 'status-badge--running': paneHasRunningSession(pane) }">
                        {{ paneHasRunningSession(pane) ? '运行中' : '待命' }}
                      </span>
                    </div>
                    <span class="pane-line__path">{{ entryById(pane.terminalEntryId)?.workingDirectory || pane.pathLabel }}</span>
                  </div>
                  <div class="pane-line__meta">
                    <span class="meta-inline">
                      <span class="meta-inline__label">Provider</span>
                      <span class="meta-inline__value meta-inline__value--accent">{{ providerSummaryLabel(entryById(pane.terminalEntryId)) }}</span>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">模型</span>
                      <span class="meta-inline__value meta-inline__value--muted">{{ entryById(pane.terminalEntryId)?.modelId || runtimeProfileResolvedProvider(entryById(pane.terminalEntryId) || undefined)?.defaultModel || '未设置' }}</span>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">默认命令</span>
                      <code class="meta-inline__value">{{ entryById(pane.terminalEntryId)?.defaultCommand || '未设置' }}</code>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">最后命令</span>
                      <code class="meta-inline__value meta-inline__value--muted">{{ entryById(pane.terminalEntryId)?.lastCommand || '无' }}</code>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">启动模式</span>
                      <span class="meta-inline__value meta-inline__value--accent">{{ launchModeLabel(entryById(pane.terminalEntryId)?.launchMode) }}</span>
                    </span>
                    <span v-if="entryById(pane.terminalEntryId)?.tags?.length" class="meta-inline meta-inline--tags">
                      <span class="meta-inline__label">标签</span>
                      <span class="meta-inline__value meta-inline__value--muted">{{ entryById(pane.terminalEntryId)?.tags.join(' · ') }}</span>
                    </span>
                  </div>
                </div>
              </div>

              <div v-else class="empty-inline empty-inline--block">当前 Tab 暂无 Pane，可在运行态中新增。</div>
            </article>

          </div>
        </section>

        <section class="runtime-layout" v-else-if="appSection === 'workspace' && workspaceView === 'runtime' && selectedWorkspace">
          <div class="runtime-header">
            <div>
              <h2>运行态</h2>
              <p>工作区 > Tab > Pane > 终端实例。统一管理项目视图、分屏布局与终端会话。</p>
            </div>
            <div class="runtime-header__actions">
              <div class="layout-toggle-group">
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'grid' }" @click="setActiveTabLayout('grid')">
                  <AppIcon name="workspace" :size="14" />
                  <span>网格</span>
                </button>
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'horizontal' }" @click="setActiveTabLayout('horizontal')">
                  <AppIcon name="copy" :size="14" />
                  <span>横向</span>
                </button>
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'vertical' }" @click="setActiveTabLayout('vertical')">
                  <AppIcon name="pane" :size="14" />
                  <span>纵向</span>
                </button>
              </div>
              <button class="ghost-btn" @click="createTab()">
                <AppIcon name="tab" :size="15" />
                <span>新建标签页</span>
              </button>
              <button class="ghost-btn" @click="createPane()">
                <AppIcon name="pane" :size="15" />
                <span>新增分屏 Pane</span>
              </button>
              <button class="ghost-btn" :disabled="!activeRuntimeTab || !countLeafPanes(activeRuntimeTab?.panes || [])" @click="saveActiveTabAsWorkflowTemplate()">
                <AppIcon name="copy" :size="15" />
                <span>保存当前项目为模板</span>
              </button>
              <button class="ghost-btn" @click="openHelpDrawer = true">
                <AppIcon name="info" :size="15" />
                <span>布局说明</span>
              </button>
            </div>
          </div>

          <div class="runtime-tabs">
            <button
              v-for="tab in selectedWorkspace.tabs"
              :key="tab.id"
              class="tab"
              :class="{ 'tab--active': activeRuntimeTabId === tab.id }"
              @click="activeRuntimeTabId = tab.id"
            >
              <AppIcon name="tab" :size="15" />
              <span>{{ runtimeTabLabel(tab) }}</span>
              <small class="tab-badge">{{ countPaneSessions(tab.panes) }}</small>
            </button>
          </div>

          <div class="runtime-canvas" :class="runtimeCanvasClass">
            <section
              v-for="pane in activeRuntimeTab?.panes || []"
              :key="pane.id"
              class="pane"
              :class="{ 'pane--running': paneHasRunningSession(pane), 'pane--selected': activeRuntimePaneId === pane.id }"
              @click="activeRuntimePaneId = pane.id"
            >
              <div class="pane__header">
                <div class="pane__title-wrap">
                  <strong>{{ pane.name }}</strong>
                  <span>{{ entryById(pane.terminalEntryId)?.workingDirectory || pane.pathLabel }}</span>
                </div>
                <div class="pane__actions">
                  <button class="icon-btn" title="打开目录" @click="openPaneDirectory(pane)">
                    <AppIcon name="folder" :size="15" />
                  </button>
                  <button class="icon-btn" title="复制路径" @click="copyPanePath(pane)">
                    <AppIcon name="copy" :size="15" />
                  </button>
                  <button class="icon-btn" title="拆分到右侧" @click="splitLeafPane(pane.id, 'horizontal')">
                    <AppIcon name="pane" :size="15" />
                  </button>
                  <div class="pane__more-wrap">
                    <button class="icon-btn" title="更多操作" @click.stop="togglePaneMenu(pane.id)">
                      <AppIcon name="more" :size="15" />
                    </button>
                    <PopoverMenu :open="activePaneMenu === pane.id" :items="paneMenuItems(pane)" />
                  </div>
                </div>
              </div>

                <div class="pane__statusbar">
                  <div class="pane__status-meta">
                    <span class="status-badge" :class="{ 'status-badge--running': paneHasRunningSession(pane) }">
                      {{ paneHasRunningSession(pane) ? '运行中' : '待命' }}
                    </span>
                    <span>{{ splitLabel(pane.splitDirection) }}</span>
                  </div>
                  <div v-if="entryById(pane.terminalEntryId)?.tags?.length" class="pane__tags">
                    <span v-for="tag in entryById(pane.terminalEntryId)?.tags || []" :key="`${pane.id}-${tag}`" class="tag-chip tag-chip--soft">{{ tag }}</span>
                  </div>
                  <div class="pane__binding">
                    <span>运行配置</span>
                    <div class="pane__binding-wrap">
                    <button
                      type="button"
                      class="binding-trigger"
                      :class="{ 'binding-trigger--active': !!pane.terminalEntryId }"
                      @click.stop="togglePaneBindingMenu(pane.id)"
                    >
                      <span>{{ entryById(pane.terminalEntryId)?.name || '未绑定运行配置' }}</span>
                      <AppIcon name="chevron-right" :size="14" />
                    </button>
                    <PopoverMenu :open="activePaneBindingMenu === pane.id" :items="paneBindingItems(pane)" />
                  </div>
                </div>
              </div>

              <div class="pane__body pane__body--terminal">
                <div class="terminal-config-card" :style="terminalPreviewStyle">
                  <div class="terminal-config-card__head">
                    <div>
                      <strong>运行配置</strong>
                      <span>{{ entryById(pane.terminalEntryId)?.name || '当前 Pane 未绑定运行配置' }}</span>
                    </div>
                    <button type="button" class="icon-btn icon-btn--bolt" title="快捷命令" @click.stop="activeCommandPanelPaneId = activeCommandPanelPaneId === pane.id ? null : pane.id">
                      <AppIcon name="bolt" :size="15" />
                    </button>
                  </div>
                  <div v-if="activeCommandPanelPaneId === pane.id" class="command-panel">
                    <div class="command-panel__section">
                      <span class="command-panel__title">默认命令</span>
                      <code>{{ entryById(pane.terminalEntryId)?.defaultCommand || '未配置' }}</code>
                    </div>
                    <div class="command-panel__section">
                      <span class="command-panel__title">最后命令</span>
                      <code>{{ entryById(pane.terminalEntryId)?.lastCommand || '未记录' }}</code>
                    </div>
                    <div class="command-panel__section" v-if="recentCommandsByPane[pane.id]?.length">
                      <span class="command-panel__title">最近命令</span>
                      <button v-for="command in recentCommandsByPane[pane.id]" :key="`${pane.id}-${command}`" type="button" class="command-chip" @click.stop="insertPaneText(pane, command)">
                        <span>{{ command }}</span>
                      </button>
                    </div>
                  </div>
                </div>
                <div class="terminal-lines" :style="terminalPreviewStyle">
                  <div>
                    <span class="terminal-key"># 绑定运行配置</span>
                    <span class="terminal-value terminal-value--accent">{{ entryById(pane.terminalEntryId)?.name || '未绑定' }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># 默认命令</span>
                    <span class="terminal-value">{{ entryById(pane.terminalEntryId)?.defaultCommand || '未设置' }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># 最后命令</span>
                    <span class="terminal-value terminal-value--muted">{{ entryById(pane.terminalEntryId)?.lastCommand || '无' }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># 启动模式</span>
                    <span class="terminal-value terminal-value--accent">{{ launchModeLabel(entryById(pane.terminalEntryId)?.launchMode) }}</span>
                  </div>
                  <div class="terminal-space"></div>
                  <div v-if="pane.terminalEntryId" class="terminal-hint">
                    <AppIcon name="info" :size="14" />
                    <span>当前 Pane 已绑定运行配置，可复用该配置的目录、命令、Provider 与环境变量。</span>
                  </div>
                  <div v-else class="terminal-hint">
                    <AppIcon name="info" :size="14" />
                    <span>当前 Pane 还未绑定运行配置，可先绑定一个运行配置，或直接使用闪电按钮查看快捷命令。</span>
                  </div>
                </div>
              </div>
            </section>

            <section v-if="!countLeafPanes(activeRuntimeTab?.panes || [])" class="pane pane--empty">
              <div class="pane__body">当前 Tab 暂无 Pane，可使用“新增分屏 Pane”创建。</div>
            </section>
          </div>

          <div class="runtime-footer">
            <div>当前工作区：{{ selectedWorkspace.name }}</div>
            <div>当前 Tab：{{ activeRuntimeTab?.name || '未选择' }}</div>
            <div>Pane 数：{{ countLeafPanes(activeRuntimeTab?.panes || []) }}</div>
          </div>
        </section>

        <section class="p45-page p45-page--recent" v-else-if="appSection === 'recent'">
          <header class="p45-hero p45-hero--recent panel">
            <div class="p45-hero__copy">
              <h2>最近</h2>
              <p>汇总最近工作区、项目、终端、命令和布局快照，减少重复查找。</p>
            </div>
            <div class="p45-hero__stats">
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="recent" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters[0]?.count ?? 0 }}</strong>
                  <small>记录</small>
                </div>
              </div>
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="bolt" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters.find((item) => item.id === 'command')?.count ?? 0 }}</strong>
                  <small>历史命令</small>
                </div>
              </div>
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="copy" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters.find((item) => item.id === 'snapshot')?.count ?? 0 }}</strong>
                  <small>现场</small>
                </div>
              </div>
            </div>
          </header>

          <div class="p45-toolbar panel">
            <div class="segmented-control">
              <button
                v-for="filter in recentFilters"
                :key="filter.id"
                type="button"
                :class="{ 'segmented-control__item--active': recentFilter === filter.id }"
                class="segmented-control__item"
                @click="recentFilter = filter.id"
              >
                <AppIcon :name="filter.icon" :size="13" />
                <span>{{ filter.label }}</span>
                <small>{{ filter.count }}</small>
              </button>
            </div>
            <div class="filter-dropdown">
              <button
                type="button"
                class="filter-dropdown__trigger"
                :class="{ 'filter-dropdown__trigger--open': openRecentWorkspaceFilterMenu }"
                @pointerdown="handleMenuTriggerPointerDown"
                @click.stop="toggleRecentWorkspaceFilterMenu()"
              >
                <span class="filter-dropdown__label">
                  <span class="filter-dropdown__icon"><AppIcon name="workspace" :size="13" /></span>
                  <span>工作区</span>
                </span>
                <span class="filter-dropdown__value">{{ recentWorkspaceFilterLabel }}</span>
                <AppIcon name="chevron-down" :size="14" />
              </button>
              <transition name="dropdown-fade">
                <div v-if="openRecentWorkspaceFilterMenu" class="filter-dropdown__menu" @click.stop>
                  <button
                    v-for="workspace in recentWorkspaceOptions"
                    :key="workspace.id"
                    type="button"
                    class="filter-dropdown__option"
                    :class="{ 'filter-dropdown__option--active': recentWorkspaceFilter === workspace.id }"
                    @click="selectRecentWorkspaceFilter(workspace.id)"
                  >
                    <span class="filter-dropdown__option-main">
                      <span class="filter-dropdown__option-icon"><AppIcon :name="workspace.id === 'all' ? 'recent' : 'workspace'" :size="13" /></span>
                      <span>{{ workspace.label }}</span>
                    </span>
                    <AppIcon v-if="recentWorkspaceFilter === workspace.id" name="check" :size="14" />
                  </button>
                </div>
              </transition>
            </div>
            <button
              v-if="hiddenRecentItemCount"
              type="button"
              class="ghost-btn ghost-btn--small"
              @click="openRecentRecycleBinModal = true"
            >
              <AppIcon name="trash" :size="13" />
              <span>回收站 {{ hiddenRecentItemCount }}</span>
            </button>
          </div>

          <section class="p45-list panel" :class="recentFilter === 'snapshot' ? 'p45-list--snapshots' : 'p45-list--records'">
            <div v-if="recentFilter === 'command'" class="p45-hint-card">
              <div class="p45-hint-card__icon">
                <AppIcon name="bolt" :size="16" />
              </div>
              <div class="p45-hint-card__body">
                <strong>这里是历史命令回顾区</strong>
                <p>点击整行会回到命令原本所在的终端；`复制命令` 适合复用，`回填当前输入框` 只会把命令写回你当前选中的终端输入框，不会自动发送。</p>
              </div>
            </div>
            <article
              v-for="item in filteredRecentItems"
              :key="item.id"
              type="button"
              role="button"
              tabindex="0"
              class="p45-row"
              :class="{ 'p45-row--snapshot': item.type === 'snapshot', 'p45-row--pinned': recentItemIsPinned(item.id), 'p45-row--command': item.type === 'command' }"
              @click="item.onOpen?.()"
              @keydown.enter.prevent="item.onOpen?.()"
              @keydown.space.prevent="item.onOpen?.()"
            >
              <template v-if="item.type === 'snapshot' && item.previewTabs?.length">
                <span class="p45-row__snapshot-info">
                  <span class="p45-row__icon p45-row__icon--snapshot"><AppIcon :name="item.icon" :size="15" /></span>
                  <span class="p45-row__snapshot-copy">
                    <span v-if="recentItemIsPinned(item.id)" class="p45-row__pin-label"><AppIcon name="star" :size="11" />已置顶</span>
                    <strong>{{ item.title }}</strong>
                    <small>{{ item.description }}</small>
                    <em>{{ item.meta }}</em>
                    <span class="p45-row__snapshot-metrics">
                      <span class="meta-badge meta-badge--soft">{{ item.previewTabs.length }} 项目</span>
                      <span class="meta-badge meta-badge--soft">{{ item.previewTabs.reduce((count, tab) => count + countPaneSessions(tab.panes), 0) }} 终端</span>
                    </span>
                  </span>
                </span>
                <span class="p45-row__preview-shell">
                  <SnapshotMiniPreview
                    class="p45-row__preview"
                    :tabs="item.previewTabs"
                    :active-tab-id="item.previewActiveTabId"
                    compact
                  />
                </span>
                <span class="p45-row__actions p45-row__actions--snapshot">
                  <button type="button" class="ghost-btn ghost-btn--small" @click.stop="togglePinRecentItem(item.id)">
                    <AppIcon :name="recentItemIsPinned(item.id) ? 'star' : 'recent'" :size="13" />
                    <span>{{ recentItemIsPinned(item.id) ? '取消置顶' : '置顶' }}</span>
                  </button>
                  <span class="meta-badge meta-badge--soft">{{ item.badge }}</span>
                  <button type="button" class="ghost-btn ghost-btn--danger ghost-btn--small" @click.stop="removeWorkspaceSnapshot(item.workspaceId || '', item.snapshotId || '')">
                    <AppIcon name="trash" :size="13" />
                    <span>删除现场</span>
                  </button>
                </span>
              </template>
              <template v-else>
                <span class="p45-row__icon"><AppIcon :name="item.icon" :size="15" /></span>
                <span class="p45-row__body">
                  <span v-if="recentItemIsPinned(item.id)" class="p45-row__pin-label"><AppIcon name="star" :size="11" />已置顶</span>
                  <strong>{{ item.title }}</strong>
                  <small>{{ item.description }}</small>
                  <em>{{ item.meta }}</em>
                  <span v-if="item.command && item.sourceSessionLabel" class="p45-row__meta-badges">
                    <span class="meta-badge meta-badge--soft">来源 {{ item.sourceSessionLabel }}</span>
                  </span>
                </span>
                <span class="p45-row__actions">
                  <button type="button" class="ghost-btn ghost-btn--small" @click.stop="togglePinRecentItem(item.id)">
                    <AppIcon :name="recentItemIsPinned(item.id) ? 'star' : 'recent'" :size="13" />
                    <span>{{ recentItemIsPinned(item.id) ? '取消置顶' : '置顶' }}</span>
                  </button>
                  <span class="meta-badge meta-badge--soft">{{ item.badge }}</span>
                  <button v-if="item.type !== 'workspace'" type="button" class="ghost-btn ghost-btn--small" @click.stop="hideRecentItem(item.id)">
                    <AppIcon name="close" :size="13" />
                    <span>移除</span>
                  </button>
                  <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="copyCommandText(item.command || '')">
                    <AppIcon name="copy" :size="13" />
                    <span>复制命令</span>
                  </button>
                  <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="openRecentCommandTarget(item.workspaceId, item.entryId, item.command)">
                    <AppIcon name="recent" :size="13" />
                    <span>回到来源终端</span>
                  </button>
                  <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="insertRecentCommand(item)">
                    <AppIcon name="terminal" :size="13" />
                    <span>回填当前输入框</span>
                  </button>
                </span>
              </template>
            </article>

            <div v-if="!filteredRecentItems.length" class="empty-state">
              <div class="empty-state__icon"><AppIcon name="recent" :size="18" /></div>
              <div class="empty-state__body">
                <strong>暂无匹配的最近记录</strong>
                <p>{{ recentFilter === 'command' ? '当前没有可回顾的历史命令。先在终端里执行过命令后，这里才会出现可复制、可回填的历史记录。' : '切换筛选条件，或先打开工作区、创建终端和保存现场。' }}</p>
              </div>
            </div>
          </section>
        </section>

        <section class="p45-page p45-page--templates" v-else-if="appSection === 'templates'">
          <header class="p45-hero p45-hero--templates panel">
            <div class="p45-hero__copy">
              <h2>模板</h2>
              <p>系统默认提供 AI CLI、前端、后端和全栈模板；需要保存当前项目时，请回到工作区底部操作栏使用“保存为模板”。</p>
              <small v-if="templateApplyTargetWorkspace" class="p45-hero__hint">默认应用目标：{{ templateApplyTargetWorkspace.name }} · {{ templateApplyTargetWorkspace.rootPath }}</small>
            </div>
          </header>

          <div class="p45-toolbar panel">
            <div class="segmented-control">
              <button
                v-for="filter in workflowTemplateFilters"
                :key="filter.id"
                type="button"
                :class="{ 'segmented-control__item--active': templateFilter === filter.id }"
                class="segmented-control__item"
                @click="templateFilter = filter.id"
              >
                <AppIcon :name="filter.icon" :size="13" />
                <span>{{ filter.label }}</span>
                <small>{{ filter.count }}</small>
              </button>
            </div>
            <div class="segmented-control segmented-control--secondary">
              <button
                v-for="tag in workflowTemplateTagFilters"
                :key="`template-tag-${tag}`"
                type="button"
                class="segmented-control__item segmented-control__item--tag"
                :class="{ 'segmented-control__item--active': templateTagFilter === tag }"
                @click="templateTagFilter = tag"
              >
                <span>{{ tag === 'all' ? '全部标签' : tag }}</span>
              </button>
            </div>
          </div>

          <section class="template-grid">
            <article v-for="template in filteredWorkflowTemplates" :key="template.id" class="workflow-template-card panel">
              <div class="workflow-template-card__head" :class="{ 'workflow-template-card__head--system': template.kind === 'system', 'workflow-template-card__head--user': template.kind === 'user' }">
                <div>
                  <span class="meta-badge" :class="{ 'meta-badge--soft': template.kind === 'user' }">{{ template.kind === 'system' ? '系统模板' : '我的模板' }}</span>
                  <h3>{{ template.name }}</h3>
                  <p>{{ template.description }}</p>
                </div>
                <span class="workflow-template-card__count">{{ template.panes.length }} 终端</span>
              </div>
              <div class="workflow-template-card__panes">
                <div v-for="pane in template.panes" :key="`${template.id}-${pane.name}`" class="template-pane-line">
                  <AppIcon name="terminal" :size="14" />
                  <span>{{ pane.name }}</span>
                  <code>{{ pane.defaultCommand || '不自动执行命令' }}</code>
                </div>
              </div>
              <div class="workflow-template-card__tags">
                <span v-for="tag in template.tags" :key="`${template.id}-${tag}`" class="tag-chip tag-chip--soft">{{ tag }}</span>
              </div>
              <div class="workflow-template-card__actions">
                <button type="button" class="ghost-btn ghost-btn--primary ghost-btn--small" @click="applyWorkflowTemplate(template)">
                  <AppIcon name="plus" :size="13" />
                  <span>使用模板</span>
                </button>
                <button type="button" class="ghost-btn ghost-btn--small" @click="duplicateWorkflowTemplate(template.id)">
                  <AppIcon name="copy" :size="13" />
                  <span>复制模板</span>
                </button>
                <button v-if="template.kind === 'user'" type="button" class="ghost-btn ghost-btn--small" @click="openWorkflowTemplateEditModal(template.id)">
                  <AppIcon name="edit" :size="13" />
                  <span>编辑</span>
                </button>
                <button v-if="template.kind === 'user'" type="button" class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeWorkflowTemplate(template.id)">
                  <AppIcon name="trash" :size="13" />
                  <span>删除</span>
                </button>
              </div>
            </article>
          </section>
        </section>

        <section class="p45-page p45-page--search" v-else-if="appSection === 'search'">
          <section class="app-search-panel panel">
            <label class="app-search-input">
              <AppIcon name="search" :size="16" />
              <input data-app-search-input v-model.trim="appSearchQuery" type="search" placeholder="搜索工作区、项目、终端、路径、配置、最近命令..." @keydown="handleSearchInputKeydown" />
            </label>
            <div v-if="pageSearchResults.length" class="app-search-meta">
              <span>当前选中 {{ Math.max(activeSearchResultIndex() + 1, 1) }} / {{ pageSearchResults.length }}</span>
              <span :class="{ 'app-search-meta__hint--active': searchLoopHint }">{{ searchLoopHint || '↑ ↓ 切换 · Enter 打开' }}</span>
            </div>
            <transition name="search-loop-hint">
              <div v-if="searchLoopHint" class="search-loop-hint" role="status">
                <AppIcon name="recent" :size="14" />
                <span>{{ searchLoopHint }}</span>
              </div>
            </transition>

            <div v-if="groupedSearchResults.length" class="search-group-list">
              <section v-for="group in groupedSearchResults" :key="group.key" class="search-group">
                <h3>{{ group.title }}</h3>
                <button v-for="result in group.items" :key="result.id" type="button" class="p45-row" :class="{ 'p45-row--selected': searchResultIsActive(result.id) }" :data-search-result-id="result.id" @mouseenter="searchResultActiveId = result.id" @click="result.onOpen()">
                  <span class="p45-row__icon"><AppIcon :name="result.icon" :size="15" /></span>
                  <span class="p45-row__body">
                    <strong>
                      <template v-for="(part, index) in highlightedText(result.title, appSearchQuery)" :key="`${result.id}-title-${index}`">
                        <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                        <template v-else>{{ part.text }}</template>
                      </template>
                    </strong>
                    <small>
                      <template v-for="(part, index) in highlightedText(result.description, appSearchQuery)" :key="`${result.id}-desc-${index}`">
                        <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                        <template v-else>{{ part.text }}</template>
                      </template>
                    </small>
                    <em>
                      <template v-for="(part, index) in highlightedText(result.meta, appSearchQuery)" :key="`${result.id}-meta-${index}`">
                        <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                        <template v-else>{{ part.text }}</template>
                      </template>
                    </em>
                  </span>
                  <span class="p45-row__actions">
                    <span class="meta-badge meta-badge--soft">{{ result.actionLabel }}</span>
                    <button v-if="result.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="copyCommandText(result.command || '')">
                      <AppIcon name="copy" :size="13" />
                      <span>复制命令</span>
                    </button>
                    <button v-if="result.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="insertCommandToActivePane(result.command || '')">
                      <AppIcon name="terminal" :size="13" />
                      <span>回填当前输入框</span>
                    </button>
                  </span>
                </button>
              </section>
            </div>

            <div v-else class="empty-state">
              <div class="empty-state__icon"><AppIcon name="search" :size="18" /></div>
              <div class="empty-state__body">
                <strong>没有找到匹配结果</strong>
                <p>当前搜索不会扫描电脑磁盘文件，只检索 Chuchen-Terminal 已保存的数据。</p>
              </div>
            </div>
          </section>
        </section>

        <section class="p45-page p45-page--settings" v-else-if="appSection === 'settings'">
          <header class="p45-hero panel">
            <div class="p45-hero__copy">
              <h2>设置</h2>
              <p>集中管理外观、终端、系统检测、通知、任务监督和数据能力。</p>
            </div>
          </header>

          <section class="settings-layout">
            <aside class="settings-nav panel">
              <button
                v-for="group in settingsGroups"
                :key="group.id"
                type="button"
                class="settings-nav__item"
                :class="{ 'settings-nav__item--active': activeSettingsSection === group.id }"
                @click="activeSettingsSection = group.id"
              >
                <AppIcon :name="group.icon" :size="15" />
                <span>{{ group.title }}</span>
                <small>{{ group.badge }}</small>
              </button>
            </aside>

            <div class="settings-main panel">
              <article
                v-for="group in settingsGroups"
                v-show="activeSettingsSection === group.id"
                :key="group.id"
                class="settings-section"
                :class="`settings-section--${group.id}`"
              >
                <div class="settings-section__head">
                  <div class="settings-section__lead">
                    <div class="settings-section__eyebrow">
                      <span class="settings-section__eyebrow-icon"><AppIcon :name="group.icon" :size="13" /></span>
                      <span>{{ group.badge }}</span>
                    </div>
                    <div>
                      <h3>{{ group.title }}</h3>
                      <p>{{ group.description }}</p>
                    </div>
                  </div>
                  <div class="settings-section__icon-shell">
                    <AppIcon :name="group.icon" :size="18" />
                  </div>
                </div>

                <div class="settings-list">
                  <div v-for="item in group.items" :key="`${group.id}-${item}`" class="settings-row">
                    <span>{{ splitSettingsItem(item).label }}</span>
                    <small>{{ splitSettingsItem(item).value }}</small>
                  </div>
                </div>

                <div v-if="group.id === 'appearance'" class="settings-actions">
                  <button type="button" class="ghost-btn ghost-btn--primary" @click="openThemePanel('theme')">
                    <AppIcon name="theme" :size="14" />
                    <span>打开主题面板</span>
                  </button>
                  <button type="button" class="ghost-btn" @click="toggleRailCollapsed()">
                    <AppIcon name="workspace" :size="14" />
                    <span>{{ railCollapsed ? '默认展开 Rail' : '默认收起 Rail' }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-preview-card">
                  <div class="settings-preview-card__head">
                    <strong>{{ selectedThemePreset.name }}</strong>
                    <span>{{ activeThemeDescription }}</span>
                  </div>
                  <div class="settings-preview-card__swatches">
                    <span v-for="color in activeThemeSwatches" :key="`settings-${color}`" class="settings-preview-card__swatch" :style="{ background: color }"></span>
                  </div>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-chip-group">
                  <button
                    v-for="theme in [...systemThemes, ...importedThemes]"
                    :key="`settings-theme-${theme.id}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': activeThemeId === theme.id }"
                    @click="applyThemeFromSettings(theme.id)"
                  >
                    <span class="settings-chip__swatch" :style="{ background: theme.accent }"></span>
                    <span>{{ theme.name }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-chip-group">
                  <button
                    v-for="size in [11, 13, 16]"
                    :key="`font-size-${size}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': terminalFontSize === size }"
                    @click="applyTerminalFontSizeFromSettings(size)"
                  >
                    <span>终端字号 {{ size }}px</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance' && activeThemeId === 'default'" class="settings-inline-field">
                  <label class="settings-inline-field__label" for="settings-accent-color">主题主色</label>
                  <input id="settings-accent-color" type="color" :value="customAccentHex" @input="onAccentColorInput(($event.target as HTMLInputElement).value); showToast('主题主色已更新', ($event.target as HTMLInputElement).value)" />
                </div>

                <div v-if="group.id === 'terminal'" class="settings-actions">
                  <button
                    v-for="option in restoreCommandStrategyOptions"
                    :key="option.value"
                    type="button"
                    class="ghost-btn"
                    :class="{ 'ghost-btn--active': restoreCommandStrategy === option.value }"
                    @click="applyRestoreStrategyFromSettings(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <div v-if="group.id === 'terminal'" class="settings-preview-card">
                  <div class="settings-preview-card__head">
                    <strong>{{ terminalFontFamily }}</strong>
                    <span>当前恢复策略：{{ restoreCommandStrategyLabel(restoreCommandStrategy) }}</span>
                  </div>
                  <code class="settings-preview-card__terminal" :style="terminalPreviewStyle">PS&gt; echo "Hello Chuchen-Terminal"</code>
                  <div class="settings-control-row">
                    <label class="settings-control-row__label" for="settings-font-size">终端字号</label>
                    <input id="settings-font-size" type="range" min="8" max="24" :value="terminalFontSize" @input="applyTerminalFontSizeFromSettings(Number(($event.target as HTMLInputElement).value))" />
                    <span class="settings-control-row__value">{{ terminalFontSize }}px</span>
                  </div>
                </div>
                <div v-if="group.id === 'terminal'" class="settings-chip-group">
                  <button
                    v-for="font in fontOptions"
                    :key="`font-family-${font}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': terminalFontFamily === font }"
                    @click="applyTerminalFontFamilyFromSettings(font)"
                  >
                    <span>{{ font }}</span>
                  </button>
                </div>

                <div v-if="group.id === 'system'" class="settings-actions">
                  <button type="button" class="ghost-btn ghost-btn--primary" @click="openThemePanel('system')">
                    <AppIcon name="runtime" :size="14" />
                    <span>打开系统面板</span>
                  </button>
                  <button type="button" class="ghost-btn" :disabled="environmentChecksRefreshing" @click="refreshEnvironmentChecks(true)">
                    <AppIcon name="refresh" :size="14" :class="{ 'is-spinning': environmentChecksRefreshing }" />
                    <span>{{ environmentChecksRefreshing ? '检测中' : '重新检测环境' }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'system'" class="settings-chip-group">
                  <button
                    v-for="option in ['manual', '5s', '10s', '30s'] as const"
                    :key="`refresh-${option}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': systemRefreshInterval === option }"
                    @click="applySystemRefreshModeFromSettings(option)"
                  >
                    <span>{{ option === 'manual' ? '手动刷新' : `${option} 刷新` }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'system'" class="settings-chip-group settings-chip-group--wrap">
                  <button
                    v-for="item in homeEnvironmentChecks"
                    :key="`settings-env-${item.name}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': !hiddenEnvironmentItems.includes(item.name) }"
                    @click="toggleEnvironmentVisibilityFromSettings(item.name)"
                  >
                    <span>{{ item.name }}</span>
                  </button>
                </div>

                <div v-if="group.id === 'notifications'" class="settings-chip-group settings-chip-group--wrap">
                  <button
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': systemNotificationsEnabled }"
                    @click="systemNotificationsEnabled = !systemNotificationsEnabled"
                  >
                    <span>{{ systemNotificationsEnabled ? '系统通知已开启' : '系统通知已关闭' }}</span>
                  </button>
                  <button
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': windowAttentionEnabled }"
                    @click="windowAttentionEnabled = !windowAttentionEnabled"
                  >
                    <span>{{ windowAttentionEnabled ? '任务栏提醒已开启' : '任务栏提醒已关闭' }}</span>
                  </button>
                </div>

                <div v-if="group.id === 'supervisor' && isDevBuild" class="settings-chip-group settings-chip-group--wrap">
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('completed')"><span>模拟完成</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('needs-input')"><span>模拟等待</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('error')"><span>模拟异常</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('stalled')"><span>模拟停滞</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('clear')"><span>清除提醒</span></button>
                </div>

                <div v-if="group.id === 'data'" class="settings-note">
                  <strong>cc-switch 配置导入</strong>
                  <p>用于承接常见 AI CLI 配置迁移流程。导入前会先预览 Provider、模型和敏感字段。</p>
                </div>
                <div v-if="group.id === 'data'" class="settings-actions">
                  <button type="button" class="ghost-btn" @click="clearDiagnosticsCaches()">
                    <AppIcon name="trash" :size="14" />
                    <span>重新加载系统数据</span>
                  </button>
                </div>
              </article>
            </div>
          </section>
        </section>

        <section class="panel panel--placeholder" v-else>
          <div class="panel__header">
            <div>
              <h2>{{ placeholderTitle }}</h2>
              <span>该模块暂未纳入当前阶段的主开发闭环，先保留结构入口。</span>
            </div>
          </div>
          <div class="placeholder-body">{{ placeholderDescription }}</div>
        </section>
      </main>
    </div>

    <ModalShell
      :open="openWorkspaceEditorModal"
      :title="workspaceEditorMode === 'create' ? '新建工作区' : '编辑工作区'"
      description="保存工作区名称、根目录、描述与标签，作为首页卡片和后续运行配置的归属容器。"
      icon="workspace"
      size="md"
      @close="closeWorkspaceEditorModal()"
    >
      <form class="editor-form editor-form--pixel" @submit.prevent="submitWorkspaceForm()">
        <label class="form-field">
          <span>工作区名称</span>
          <input v-model.trim="workspaceForm.name" type="text" placeholder="例如：demo-workspace / app-suite" />
        </label>
        <label class="form-field">
          <span>根目录</span>
          <div class="path-field">
            <input v-model.trim="workspaceForm.rootPath" type="text" placeholder="例如：D:\\Projects\\demo-workspace" />
            <button type="button" class="ghost-btn ghost-btn--small path-field__action" @click.stop="pickWorkspaceRootPath()">选择目录</button>
          </div>
        </label>
        <label class="form-field">
          <span>工作区描述</span>
          <textarea v-model.trim="workspaceForm.description" rows="3" placeholder="简单描述这个工作区主要承担的用途"></textarea>
        </label>
        <label class="form-field">
          <span>标签</span>
          <input v-model.trim="workspaceForm.tagsText" type="text" placeholder="使用中文逗号、英文逗号或空格分隔，例如：前端, 后端, AI" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeWorkspaceEditorModal()">取消</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">保存工作区</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTerminalEntryEditorModal"
      :title="terminalEntryEditorMode === 'create' ? '新建运行配置' : '编辑运行配置'"
      description="运行配置保存目录、命令、Provider 绑定和环境变量，是 Pane 恢复与运行时解析的基础对象。"
      icon="terminal"
      size="md"
      @close="closeTerminalEntryEditorModal()"
    >
      <form class="editor-form" @submit.prevent="submitTerminalEntryForm()">
        <label class="form-field">
          <span>运行配置名称</span>
          <input v-model.trim="terminalEntryForm.name" type="text" placeholder="例如：前端 / 后端 / Codex" />
        </label>
        <label class="form-field">
          <span>工作目录</span>
          <div class="path-field">
            <input v-model.trim="terminalEntryForm.workingDirectory" type="text" placeholder="例如：D:\\Projects\\demo-frontend" />
            <button type="button" class="ghost-btn ghost-btn--small path-field__action" @click.stop="pickTerminalEntryWorkingDirectory()">选择目录</button>
          </div>
        </label>
        <label class="form-field">
          <span>默认命令</span>
          <input v-model.trim="terminalEntryForm.defaultCommand" type="text" placeholder="例如：pnpm dev / go run ./cmd/server" />
        </label>
        <label class="form-field">
          <span>启动模式</span>
          <div class="pane__binding-wrap form-select-wrap">
            <button type="button" class="binding-trigger form-select-trigger" @click.stop="toggleLaunchModeMenu()">
              <span>{{ launchModeLabel(terminalEntryForm.launchMode) }}</span>
              <AppIcon name="chevron-right" :size="14" />
            </button>
            <PopoverMenu :open="openLaunchModeMenu" :items="launchModeItems" />
          </div>
        </label>
        <label class="form-field">
          <span>Provider 绑定</span>
          <select v-model="terminalEntryForm.providerBindingMode">
            <option value="inherit">继承默认 Provider</option>
            <option value="explicit">显式指定 Provider</option>
            <option value="disabled">不注入 Provider</option>
          </select>
        </label>
        <label v-if="terminalEntryForm.providerBindingMode === 'explicit'" class="form-field">
          <span>Provider</span>
          <select v-model="terminalEntryForm.providerProfileId">
            <option value="">请选择 Provider</option>
            <option v-for="provider in selectedWorkspaceProviders" :key="provider.id" :value="provider.id">{{ provider.name }}</option>
          </select>
        </label>
        <label class="form-field">
          <span>模型</span>
          <input v-model.trim="terminalEntryForm.modelId" type="text" placeholder="例如：gpt-5 / claude-sonnet-4 / gemini-2.5-pro" />
        </label>
        <label class="form-field">
          <span>MCP 策略</span>
          <select v-model="terminalEntryForm.mcpPolicy">
            <option value="inherit">继承工作区</option>
            <option value="workspace">强制使用工作区</option>
            <option value="none">不注入 MCP</option>
            <option value="custom">后续自定义</option>
          </select>
        </label>
        <label class="form-field">
          <span>Skill 策略</span>
          <select v-model="terminalEntryForm.skillPolicy">
            <option value="inherit">继承工作区</option>
            <option value="workspace">强制使用工作区</option>
            <option value="none">不注入 Skill</option>
            <option value="custom">后续自定义</option>
          </select>
        </label>
        <label class="form-field">
          <span>环境变量</span>
          <textarea v-model.trim="terminalEntryForm.environmentVariablesText" rows="4" placeholder="每行一个，例如：&#10;OPENAI_API_KEY=***&#10;HTTP_PROXY=http://127.0.0.1:7890"></textarea>
        </label>
        <label class="form-field">
          <span>标签</span>
          <input v-model.trim="terminalEntryForm.tagsText" type="text" placeholder="例如：前端, 常用, AI" />
        </label>
        <label class="form-field">
          <span>备注</span>
          <textarea v-model.trim="terminalEntryForm.note" rows="3" placeholder="记录这个配置的场景说明或注意事项"></textarea>
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeTerminalEntryEditorModal()">取消</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">保存配置</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTemplateEditorModal"
      :title="templateEditorMode === 'create' ? '新建模板' : '编辑模板'"
      description="只编辑模板的名称、说明和标签；模板内的 Pane 结构沿用保存时的内容。"
      icon="template"
      size="md"
      @close="closeWorkflowTemplateEditorModal()"
    >
      <form class="editor-form" @submit.prevent="submitWorkflowTemplateForm()">
        <label class="form-field">
          <span>模板名称</span>
          <input v-model.trim="workflowTemplateForm.name" type="text" placeholder="例如：前后端联调 / AI CLI 工作流" />
        </label>
        <label class="form-field">
          <span>模板说明</span>
          <textarea v-model.trim="workflowTemplateForm.description" rows="3" placeholder="简单描述这个模板适合什么场景"></textarea>
        </label>
        <label class="form-field">
          <span>标签</span>
          <input v-model.trim="workflowTemplateForm.tagsText" type="text" placeholder="例如：前端, 后端, AI" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeWorkflowTemplateEditorModal()">取消</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">保存模板</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openProviderEditorModal"
      :title="providerEditorMode === 'create' ? '新建 Provider' : '编辑 Provider'"
      description="Provider 保存 Base URL、认证方式、默认模型和适用工具，是运行配置解析的上游对象。"
      icon="settings"
      size="md"
      @close="closeProviderEditorModal()"
    >
      <form class="editor-form" @submit.prevent="submitProviderForm()">
        <label class="form-field">
          <span>Provider 名称</span>
          <input v-model.trim="providerForm.name" type="text" placeholder="例如：OpenAI Main / Claude Proxy" />
        </label>
        <label class="form-field">
          <span>Provider 类型</span>
          <select v-model="providerForm.providerKind">
            <option value="openai-compatible">OpenAI Compatible</option>
            <option value="anthropic">Anthropic</option>
            <option value="gemini">Gemini</option>
            <option value="custom">Custom</option>
          </select>
        </label>
        <label class="form-field">
          <span>Base URL</span>
          <input v-model.trim="providerForm.baseUrl" type="text" placeholder="例如：https://api.openai.com/v1" />
        </label>
        <label class="form-field">
          <span>认证密钥</span>
          <input v-model.trim="providerForm.apiKey" type="password" placeholder="留空，运行前在本机填写" />
        </label>
        <label class="form-field">
          <span>API 格式</span>
          <select v-model="providerForm.apiFormat">
            <option value="openai">OpenAI</option>
            <option value="anthropic">Anthropic</option>
            <option value="gemini">Gemini</option>
            <option value="custom">Custom</option>
          </select>
        </label>
        <label class="form-field">
          <span>默认模型</span>
          <input v-model.trim="providerForm.defaultModel" type="text" placeholder="例如：gpt-5 / claude-sonnet-4" />
        </label>
        <label class="form-field">
          <span>适用工具</span>
          <input v-model.trim="providerForm.toolTargetsText" type="text" placeholder="例如：codex, claude, generic" />
        </label>
        <label class="form-field">
          <span>标识颜色</span>
          <input v-model.trim="providerForm.color" type="text" placeholder="例如：#4b83ff" />
        </label>
        <label class="form-field">
          <span>备注</span>
          <textarea v-model.trim="providerForm.note" rows="3" placeholder="说明认证来源、适用项目或限额提示"></textarea>
        </label>
        <label class="form-field form-field--inline">
          <input v-model="providerForm.isDefault" type="checkbox" />
          <span>设为当前工作区默认 Provider</span>
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeProviderEditorModal()">取消</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">保存 Provider</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTerminalEntriesModal"
      title="运行配置"
      description="运行配置负责工作目录、命令、Provider 绑定与环境变量；Provider 则是独立的供应商配置对象。"
      icon="terminal"
      size="lg"
      @close="openTerminalEntriesModal = false"
    >
      <div class="entry-modal">
        <div class="entry-modal__toolbar">
          <div class="entry-modal__intro">
            <strong>{{ selectedWorkspace?.name }}</strong>
            <span>当前工作区的运行配置与 Provider 都在这里管理。</span>
          </div>
          <div class="entry-modal__toolbar-actions">
            <button class="ghost-btn" @click="openProviderCreateModal()">
              <AppIcon name="settings" :size="15" />
              <span>新建 Provider</span>
            </button>
            <button class="ghost-btn ghost-btn--primary" @click="openTerminalEntryCreateModal()">
              <AppIcon name="terminal" :size="15" />
              <span>新建运行配置</span>
            </button>
          </div>
        </div>

        <div v-if="selectedWorkspaceProviders.length" class="entry-list entry-list--providers">
          <article v-for="provider in selectedWorkspaceProviders" :key="provider.id" class="entry-card entry-card--provider">
            <div class="entry-card__head">
              <div>
                <strong>{{ provider.name }}</strong>
                <span>{{ provider.baseUrl || '未设置 Base URL' }}</span>
              </div>
              <div class="entry-card__head-meta">
                <span class="meta-badge">{{ provider.providerKind }}</span>
                <span v-if="provider.isDefault" class="meta-badge meta-badge--soft">默认 Provider</span>
              </div>
            </div>
            <div class="entry-card__body">
              <span>API 格式：{{ provider.apiFormat }}</span>
              <span>默认模型：{{ provider.defaultModel || '未设置' }}</span>
              <span>适用工具：{{ provider.toolTargets.length ? provider.toolTargets.join(' · ') : 'generic' }}</span>
            </div>
            <div class="entry-card__actions">
              <button class="ghost-btn ghost-btn--small" @click="openProviderEditModal(provider.id)">编辑</button>
              <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeProviderProfile(provider.id)">删除</button>
            </div>
          </article>
        </div>

        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="settings" :size="18" />
          </div>
          <div class="empty-state__body">
            <strong>还没有 Provider</strong>
            <p>先创建 Provider，运行配置才能选择“显式指定”或“继承默认”。</p>
          </div>
        </div>

        <div class="entry-modal__divider"></div>

        <div v-if="selectedWorkspaceEntries.length" class="entry-list">
          <article v-for="entry in selectedWorkspaceEntries" :key="entry.id" class="entry-card">
            <div class="entry-card__head">
              <div>
                <strong>{{ entry.name }}</strong>
                <span>{{ entry.workingDirectory }}</span>
              </div>
              <div class="entry-card__head-meta">
                <span class="meta-badge">{{ launchModeLabel(entry.launchMode) }}</span>
                <span class="meta-badge meta-badge--soft">已绑定 {{ entryUsageCount(entry.id) }} 个 Pane</span>
              </div>
            </div>
            <div class="entry-card__body">
              <span>Provider：{{ providerSummaryLabel(entry) }}</span>
              <span>模型：{{ entry.modelId || runtimeProfileResolvedProvider(entry)?.defaultModel || '未设置' }}</span>
              <span>默认命令：{{ entry.defaultCommand || '未设置' }}</span>
              <span>环境变量：{{ entry.environmentVariablesText ? '已配置' : '未配置' }}</span>
              <span>MCP / Skill：{{ entry.mcpPolicy || 'inherit' }} / {{ entry.skillPolicy || 'inherit' }}</span>
              <span>最近命令：{{ entry.lastCommand || '未记录' }}</span>
            </div>
            <div class="entry-card__actions">
              <button class="ghost-btn ghost-btn--small" @click="openTerminalEntryEditModal(entry.id)">编辑</button>
              <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeTerminalEntry(entry.id)">删除</button>
            </div>
          </article>
        </div>

        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="terminal" :size="15" />
          </div>
          <div class="empty-state__body">
            <strong>还没有运行配置</strong>
            <p>你可以先创建运行配置，再到“运行态”为某个 Pane 绑定它。</p>
          </div>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openSplitActionModal"
      :title="splitActionState.mode === 'create' ? '新增分屏 Pane' : '复制当前 Pane'"
      description="选择这次新增分屏的布局方向。"
      icon="pane"
      size="md"
      @close="openSplitActionModal = false"
    >
      <div class="split-action-dialog">
        <div class="split-action-grid">
          <button type="button" class="split-choice-card" @click="submitSplitAction('grid')">
            <AppIcon name="workspace" :size="18" />
            <strong>网格</strong>
            <span>更适合多个 Pane 同时总览。</span>
          </button>
          <button type="button" class="split-choice-card" @click="submitSplitAction('horizontal')">
            <AppIcon name="copy" :size="18" />
            <strong>横向</strong>
            <span>让 Pane 以左右并排的方式展示。</span>
          </button>
          <button type="button" class="split-choice-card" @click="submitSplitAction('vertical')">
            <AppIcon name="pane" :size="18" />
            <strong>纵向</strong>
            <span>让 Pane 以上下堆叠的方式展示。</span>
          </button>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openThemeModal"
      title="主题与字体"
      description="高频配置使用居中 Modal；主题与字体在同一个面板中切换，不强制关闭整个面板。"
      size="lg"
      @close="openThemeModal = false"
    >
      <div class="modal-tabs">
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'theme' }" @click.stop="themePanelTab = 'theme'">
          <AppIcon name="theme" :size="15" />
          <span>主题</span>
        </button>
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'font' }" @click.stop="themePanelTab = 'font'">
          <AppIcon name="terminal" :size="15" />
          <span>字体</span>
        </button>
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'system' }" @click.stop="themePanelTab = 'system'">
          <AppIcon name="runtime" :size="15" />
          <span>系统</span>
        </button>
      </div>

      <div v-if="themePanelTab === 'theme'" class="theme-panel">
        <div class="theme-panel__summary">
          <div>
            <strong>{{ activeThemeLabel }}</strong>
            <span>{{ activeThemeDescription }}</span>
          </div>
          <div class="theme-swatches">
            <span v-for="color in activeThemeSwatches" :key="color" class="theme-swatches__dot" :style="{ background: color }"></span>
          </div>
        </div>

        <section class="theme-section">
          <div class="theme-section__head">
            <h4>系统主题</h4>
            <span>默认主题支持自由调主色；彩色预设可一键切换。</span>
          </div>
          <div class="theme-grid">
            <button
              type="button"
              v-for="theme in systemThemes"
              :key="theme.id"
              class="theme-card"
              :class="{ 'theme-card--active': activeThemeId === theme.id }"
              @click.stop="selectTheme(theme.id)"
            >
              <span class="theme-card__swatches">
                <span v-for="color in theme.swatches" :key="color" class="theme-card__dot" :style="{ background: color }"></span>
              </span>
              <span class="theme-card__body">
                <strong>{{ theme.name }}</strong>
                <span>{{ theme.description }}</span>
              </span>
              <small>{{ theme.kind }}</small>
            </button>
          </div>
        </section>

        <section class="theme-section">
          <div class="theme-section__head">
            <h4>常见主题风格</h4>
            <span>先内置几套常见主题，后续可扩展下载与导入。</span>
          </div>
          <div class="theme-grid theme-grid--compact">
            <button
              type="button"
              v-for="theme in importedThemes"
              :key="theme.id"
              class="theme-card"
              :class="{ 'theme-card--active': activeThemeId === theme.id }"
              @click.stop="selectTheme(theme.id)"
            >
              <span class="theme-card__swatches">
                <span v-for="color in theme.swatches" :key="color" class="theme-card__dot" :style="{ background: color }"></span>
              </span>
              <span class="theme-card__body">
                <strong>{{ theme.name }}</strong>
                <span>{{ theme.description }}</span>
              </span>
              <small>{{ theme.kind }}</small>
            </button>
          </div>
        </section>

        <section class="theme-customizer" v-if="activeThemeId === 'default'">
          <div class="theme-section__head">
            <h4>默认主题调色</h4>
            <span>默认主题允许按你的喜好自定义主色，支持色盘、HEX 与 RGB。</span>
          </div>
          <div class="color-editor">
            <label class="color-input-card">
              <span>主色</span>
              <input type="color" :value="customAccentHex" @input="onAccentColorInput(($event.target as HTMLInputElement).value)" />
            </label>
            <label class="text-input-card text-input-card--wide">
              <span>HEX</span>
              <input type="text" :value="customAccentHex" @change="onAccentHexChange(($event.target as HTMLInputElement).value)" />
            </label>
            <label class="text-input-card">
              <span>R</span>
              <input type="number" min="0" max="255" :value="customAccentRgb.r" @change="onAccentRgbChange('r', ($event.target as HTMLInputElement).value)" />
            </label>
            <label class="text-input-card">
              <span>G</span>
              <input type="number" min="0" max="255" :value="customAccentRgb.g" @change="onAccentRgbChange('g', ($event.target as HTMLInputElement).value)" />
            </label>
            <label class="text-input-card">
              <span>B</span>
              <input type="number" min="0" max="255" :value="customAccentRgb.b" @change="onAccentRgbChange('b', ($event.target as HTMLInputElement).value)" />
            </label>
          </div>
        </section>
      </div>

      <div v-else-if="themePanelTab === 'font'" class="font-panel">
        <div class="font-preview" :style="terminalPreviewStyle">
          <div><span class="font-preview__prompt">?</span> echo "Hello, Chuchen-Terminal"</div>
          <div class="font-preview__muted">Hello, Chuchen-Terminal</div>
          <div><span class="font-preview__prompt">?</span> pnpm dev</div>
          <div class="font-preview__muted">ready in 812 ms  ?  Local: http://localhost:5173/</div>
          <div class="font-preview__divider"></div>
          <div class="font-preview__chars">特殊字符：│ ├ └ ┐ ─ ? ? ↖ ↗ ? √ △</div>
        </div>

        <div class="font-presets">
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 11 }" @click.stop="setTerminalFontSize(11)">
            <strong>小</strong>
            <span>11px</span>
          </button>
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 13 }" @click.stop="setTerminalFontSize(13)">
            <strong>中</strong>
            <span>13px</span>
          </button>
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 16 }" @click.stop="setTerminalFontSize(16)">
            <strong>大</strong>
            <span>16px</span>
          </button>
        </div>

        <div class="font-controls">
          <div class="font-control">
            <div class="font-control__head">
              <strong>终端字号</strong>
              <span>{{ terminalFontSize }}px</span>
            </div>
            <div class="font-control__body">
              <input type="range" min="8" max="24" :value="terminalFontSize" @input="setTerminalFontSize(Number(($event.target as HTMLInputElement).value))" />
              <input class="font-number" type="number" min="8" max="24" :value="terminalFontSize" @change="setTerminalFontSize(Number(($event.target as HTMLInputElement).value))" />
            </div>
          </div>

          <div class="font-control">
            <div class="font-control__head">
              <strong>终端字体</strong>
              <span>{{ fontOptions.length }} 个候选</span>
            </div>
            <div class="font-list">
              <button
                type="button"
                v-for="font in fontOptions"
                :key="font"
                class="font-family-card"
                :class="{ 'font-family-card--active': terminalFontFamily === font }"
                @click.stop="terminalFontFamily = font"
              >
                <span class="font-family-card__body">
                  <strong>{{ font }}</strong>
                  <span>The quick brown fox jumps</span>
                </span>
                <small>终端</small>
              </button>
            </div>
          </div>
        </div>

        <div class="font-note">后续可扩展：在线检索常见终端字体、导入本地 Nerd Font、与 xterm.js 配置联动。</div>
      </div>

      <div v-else class="font-panel">
        <section class="theme-section">
          <div class="theme-section__head">
            <h4>系统资源刷新</h4>
            <span>自动刷新默认开启，也可以切为手动刷新，降低系统负担。</span>
          </div>
          <div class="theme-section__actions">
            <button type="button" class="ghost-btn ghost-btn--small" @click.stop="refreshSystemStatus()">
              <AppIcon name="runtime" :size="14" />
              <span>立即刷新</span>
            </button>
          </div>
          <div class="theme-grid theme-grid--compact">
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === 'manual' }" @click.stop="systemRefreshInterval = 'manual'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>手动刷新</strong><span>仅在打开应用时读取一次</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '5s' }" @click.stop="systemRefreshInterval = '5s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>5 秒</strong><span>更实时，但开销更大</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '10s' }" @click.stop="systemRefreshInterval = '10s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>10 秒</strong><span>默认建议值</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '30s' }" @click.stop="systemRefreshInterval = '30s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>30 秒</strong><span>更省资源</span></span></button>
          </div>
        </section>

        <section class="theme-section">
          <div class="theme-section__head">
            <h4>环境项显示</h4>
            <span>常见技术栈默认展示，可以按你的习惯隐藏不关心的项。</span>
          </div>
          <div class="theme-section__actions">
            <button type="button" class="ghost-btn ghost-btn--small" :disabled="environmentChecksRefreshing" @click.stop="refreshEnvironmentChecks(true)">
              <AppIcon name="refresh" :size="14" :class="{ 'is-spinning': environmentChecksRefreshing }" />
              <span>{{ environmentChecksRefreshing ? '检测中' : '重新检测环境' }}</span>
            </button>
          </div>
          <div class="theme-grid theme-grid--compact theme-grid--env-switches">
            <button
              type="button"
              v-for="item in homeEnvironmentChecks"
              :key="item.name"
              class="theme-card theme-card--env-switch"
              :class="{ 'theme-card--active': !hiddenEnvironmentItems.includes(item.name) }"
              @click.stop="hiddenEnvironmentItems = hiddenEnvironmentItems.includes(item.name) ? hiddenEnvironmentItems.filter((name) => name !== item.name) : [...hiddenEnvironmentItems, item.name]"
            >
              <span class="theme-card__icon">
                <img v-if="item.iconSrc" :src="item.iconSrc" :alt="`${item.name} 图标`" />
                <span v-else>{{ item.icon }}</span>
              </span>
              <span class="theme-card__body">
                <strong>{{ item.name }}</strong>
                <span>{{ hiddenEnvironmentItems.includes(item.name) ? '已隐藏' : '已显示' }}</span>
              </span>
            </button>
          </div>
        </section>

        <section v-if="isDevBuild" class="theme-section">
          <div class="theme-section__head">
            <h4>提醒测试（开发环境）</h4>
            <span>只在开发环境显示，用于直接模拟终端提醒状态，验证高亮、通知、待处理数与任务栏提醒。</span>
          </div>
          <div class="theme-panel__summary theme-panel__summary--devtest">
            <div>
              <strong>{{ currentActiveRuntimeSessionMeta?.sessionName || '未选中终端' }}</strong>
              <span>{{ currentActiveRuntimeSessionMeta ? `${currentActiveRuntimeSessionMeta.workspaceName} / ${currentActiveRuntimeSessionMeta.tabName}` : '请先在右侧选中一个终端标签' }}</span>
            </div>
            <span class="meta-badge meta-badge--soft">DEV ONLY</span>
          </div>
          <div class="theme-grid theme-grid--compact">
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('completed')">
              <span class="theme-card__body">
                <strong>模拟完成</strong>
                <span>触发已完成高亮、通知与待处理数。</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('needs-input')">
              <span class="theme-card__body">
                <strong>模拟等待输入</strong>
                <span>触发待处理高亮与系统提醒。</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('error')">
              <span class="theme-card__body">
                <strong>模拟异常退出</strong>
                <span>触发异常高亮、通知与待处理数。</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('stalled')">
              <span class="theme-card__body">
                <strong>模拟疑似停滞</strong>
                <span>触发黄色提醒，用于测试人工介入场景。</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('clear')">
              <span class="theme-card__body">
                <strong>清除提醒</strong>
                <span>把当前终端恢复为无提醒状态。</span>
              </span>
            </button>
          </div>
        </section>
      </div>
    </ModalShell>

    <ModalShell
      :open="openRenameModal"
      :title="renameTarget.title"
      description="支持手动调整名称，不影响内部编号与布局状态。"
      icon="edit"
      size="sm"
      @close="closeRenameModal()"
    >
      <form class="editor-form" @submit.prevent="submitRenameModal()">
        <label class="form-field">
          <span>{{ renameTarget.kind === 'tab' ? '项目名称' : renameTarget.kind === 'session' ? '终端名称' : 'Pane 名称' }}</span>
          <input ref="renameInputRef" v-model.trim="renameTarget.value" type="text" :placeholder="renameTarget.placeholder" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeRenameModal()">取消</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">保存名称</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openSearchModal"
      title="快速搜索"
      description="双击 Shift 或 Ctrl+K 随时唤起，快速打开工作区、项目、终端、配置和命令。"
      size="lg"
      @close="openSearchModal = false"
    >
      <div class="quick-search-panel">
        <label class="app-search-input app-search-input--modal">
          <AppIcon name="search" :size="16" />
          <input data-quick-search-input v-model.trim="appSearchQuery" type="search" placeholder="输入工作区、项目、终端、配置、命令..." @keydown="handleSearchInputKeydown" />
        </label>
        <div class="quick-search-panel__meta">
          <span class="meta-badge meta-badge--soft">Shift Shift</span>
          <span class="meta-badge meta-badge--soft">Ctrl+K</span>
          <span v-if="searchLoopHint" class="meta-badge meta-badge--notice">{{ searchLoopHint }}</span>
          <button type="button" class="ghost-btn ghost-btn--small" @click="openSearchModal = false; openSearchPage()">
            <AppIcon name="search" :size="13" />
            <span>进入搜索页</span>
          </button>
        </div>
        <div v-if="groupedSearchResults.length" class="quick-search-results">
          <section v-for="group in groupedSearchResults" :key="`quick-${group.key}`" class="search-group">
            <h3>{{ group.title }}</h3>
            <button v-for="result in group.items.slice(0, 5)" :key="`quick-${result.id}`" type="button" class="p45-row p45-row--compact" :class="{ 'p45-row--selected': searchResultIsActive(result.id) }" :data-search-result-id="result.id" @mouseenter="searchResultActiveId = result.id" @click="openSearchModal = false; result.onOpen()">
              <span class="p45-row__icon"><AppIcon :name="result.icon" :size="15" /></span>
              <span class="p45-row__body">
                <strong>
                  <template v-for="(part, index) in highlightedText(result.title, appSearchQuery)" :key="`quick-${result.id}-title-${index}`">
                    <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                    <template v-else>{{ part.text }}</template>
                  </template>
                </strong>
                <small>
                  <template v-for="(part, index) in highlightedText(result.description, appSearchQuery)" :key="`quick-${result.id}-desc-${index}`">
                    <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                    <template v-else>{{ part.text }}</template>
                  </template>
                </small>
                <em>
                  <template v-for="(part, index) in highlightedText(result.meta, appSearchQuery)" :key="`quick-${result.id}-meta-${index}`">
                    <mark v-if="part.match" class="search-highlight">{{ part.text }}</mark>
                    <template v-else>{{ part.text }}</template>
                  </template>
                </em>
              </span>
              <span class="p45-row__actions">
                <span class="meta-badge meta-badge--soft">{{ result.actionLabel }}</span>
              </span>
            </button>
          </section>
        </div>
        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="search" :size="18" />
          </div>
          <div class="empty-state__body">
            <strong>没有找到匹配结果</strong>
            <p>这里只检索 Chuchen-Terminal 内部数据，不会扫描整个电脑磁盘。</p>
          </div>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openRecentRecycleBinModal"
      title="最近记录回收站"
      description="这里保存你从最近页移除的记录。可以单条恢复，也可以全部恢复。"
      icon="trash"
      size="md"
      @close="openRecentRecycleBinModal = false"
    >
      <div class="entry-modal">
        <div class="entry-modal__toolbar">
          <div class="entry-modal__intro">
            <strong>{{ removedRecentItems.length }} 条记录</strong>
            <span>恢复后会重新出现在最近页中。</span>
          </div>
          <div class="entry-modal__toolbar-actions">
            <button class="ghost-btn" @click="clearHiddenRecentItems()">
              <AppIcon name="trash" :size="14" />
              <span>清空回收站</span>
            </button>
            <button class="ghost-btn ghost-btn--primary" @click="restoreHiddenRecentItems()">
              <AppIcon name="refresh" :size="14" />
              <span>全部恢复</span>
            </button>
          </div>
        </div>

        <div v-if="removedRecentItems.length" class="entry-list">
          <article v-for="item in removedRecentItems" :key="`recycle-${item.id}`" class="entry-card">
            <div class="entry-card__head">
              <div>
                <strong>{{ item.title }}</strong>
                <span>{{ item.description }}</span>
              </div>
              <div class="entry-card__head-meta">
                <span class="meta-badge meta-badge--soft">{{ item.badge }}</span>
              </div>
            </div>
            <div class="entry-card__body">
              <span>{{ item.meta }}</span>
            </div>
            <div class="entry-card__actions">
              <button class="ghost-btn ghost-btn--small" @click="restoreHiddenRecentItem(item.id)">恢复</button>
            </div>
          </article>
        </div>

        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="trash" :size="18" />
          </div>
          <div class="empty-state__body">
            <strong>回收站为空</strong>
            <p>当前没有被移除的最近记录。</p>
          </div>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openConfirmModal"
      :title="confirmDialog.title"
      :description="confirmDialog.description"
      size="sm"
      @close="closeConfirmModal()"
    >
      <div class="confirm-dialog" :class="`confirm-dialog--${confirmDialog.variant}`">
        <div class="confirm-dialog__body">
          <p>{{ confirmDialog.description }}</p>
          <ul v-if="confirmDialog.details.length" class="confirm-dialog__details">
            <li v-for="detail in confirmDialog.details" :key="detail">{{ detail }}</li>
          </ul>
        </div>
        <div class="modal-actions modal-actions--end">
          <button type="button" class="ghost-btn" @click="closeConfirmModal()">取消</button>
          <button type="button" class="ghost-btn" :class="confirmDialog.variant === 'danger' ? 'ghost-btn--danger' : 'ghost-btn--primary'" @click="submitConfirmModal()">{{ confirmDialog.confirmLabel }}</button>
        </div>
      </div>
    </ModalShell>

    <DrawerPanel
      :open="openHelpDrawer"
      :title="activeHelpContent.title"
      :description="activeHelpContent.description"
      side="right"
      @close="openHelpDrawer = false"
    >
      <div class="drawer-content">
        <section v-for="section in activeHelpContent.sections" :key="section.title">
          <h4>{{ section.title }}</h4>
          <p v-if="section.body">{{ section.body }}</p>
          <ul v-if="section.items?.length">
            <li v-for="item in section.items" :key="item">{{ item }}</li>
          </ul>
        </section>
      </div>
    </DrawerPanel>

    <transition name="toast-slide">
      <div v-if="uiToast" class="top-toast" :key="uiToast.id">
        <div class="top-toast__content">
          <div>
            <strong>{{ uiToast.title }}</strong>
            <p>{{ uiToast.message }}</p>
          </div>
          <button type="button" class="icon-btn" @click="dismissToast">×</button>
        </div>
        <span class="top-toast__progress" :style="{ '--toast-duration': `${uiToast.duration}ms` } as Record<string, string>"></span>
      </div>
    </transition>
  </div>


  <teleport to="body">
    <div
      v-if="draggingSession?.active && draggingSession?.moved"
      class="terminal-drag-ghost"
      :style="{ left: `${draggingSession.currentX - draggingSession.offsetX}px`, top: `${draggingSession.currentY - draggingSession.offsetY}px`, width: `${draggingSession.width}px`, height: `${draggingSession.height}px` }"
    >
      <AppIcon name="terminal" :size="13" />
      <span>{{ draggingSession.sessionName }}</span>
    </div>

    <div
      v-if="dragDropTarget && dragDropTarget.kind !== 'tabbar'"
      class="terminal-drop-preview"
      :style="{
        left: `${dragDropTarget.rect.left}px`,
        top: `${dragDropTarget.rect.top}px`,
        width: `${dragDropTarget.rect.width}px`,
        height: `${dragDropTarget.rect.height}px`,
      }"
    ></div>

    <div
      v-if="dragDropTarget?.kind === 'tabbar'"
      class="terminal-drop-insert-marker"
      :style="{
        left: `${dragDropTarget.marker.left}px`,
        top: `${dragDropTarget.marker.top}px`,
        height: `${dragDropTarget.marker.height}px`,
      }"
    ></div>
  </teleport>

</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, defineComponent, h, nextTick, onBeforeUnmount, onMounted, reactive, ref, shallowRef, watch, type PropType, type VNode } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window'
import AppIcon from './components/AppIcon.vue'
import DrawerPanel from './components/DrawerPanel.vue'
import ModalShell from './components/ModalShell.vue'
import PopoverMenu, { type PopoverItem } from './components/PopoverMenu.vue'
import SnapshotMiniPreview from './components/SnapshotMiniPreview.vue'
const TerminalPane = defineAsyncComponent(() => import('./components/TerminalPane.vue'))
import githubIcon from './assets/env-icons/github.svg'
import goIcon from './assets/env-icons/go.svg'
import javaIcon from './assets/env-icons/java.svg'
import nodejsIcon from './assets/env-icons/nodejs.svg'
import powershellIcon from './assets/env-icons/powershell.svg'
import pythonIcon from './assets/env-icons/python.svg'
import rustIcon from './assets/env-icons/Rust-icon.svg'
import tauriIcon from './assets/env-icons/tauri.svg'
import tsIcon from './assets/env-icons/ts.svg'
import vueIcon from './assets/env-icons/vue3.svg'
import {
  createProviderProfileRecord,
  createId,
  createTerminalEntryRecord,
  createWorkspaceSnapshotRecord,
  createWorkspaceRecord,
  loadWorkspaces,
  relativeTimeLabel,
  saveWorkspaces,
} from './services/workspace-storage'
import {
  createWorkflowTemplateFromInput,
  loadUserWorkflowTemplates,
  saveUserWorkflowTemplates,
  systemWorkflowTemplates,
} from './services/workflow-templates'
import { destroyTerminalRuntime, ensureTerminalReady, getTerminalRuntimeState, writeTerminalText } from './services/terminal-runtime'
import { sendSessionAttentionNotification } from './services/session-attention-notifier'
import type {
  AppSection,
  PaneNode,
  PaneTerminalSession,
  ProviderProfile,
  ProviderToolTarget,
  SessionAttentionState,
  SplitDirection,
  TabLayoutMode,
  TerminalEntry,
  WorkflowTemplate,
  WorkflowTemplateCategory,
  WorkspaceCard,
  WorkspaceSnapshot,
  WorkspaceTab,
  WorkspaceView,
} from './types/workspace'

type ThemePanelTab = 'theme' | 'font' | 'system'
type RestoreCommandStrategy = 'layout-only' | 'prefill' | 'execute'
type SettingsSection = 'appearance' | 'terminal' | 'system' | 'notifications' | 'supervisor' | 'data'
type RecentFilter = 'all' | 'workspace' | 'project' | 'session' | 'command' | 'snapshot'
type WorkflowTemplateFilter = WorkflowTemplateCategory | 'all' | 'user'
type SearchResultType = 'workspace' | 'project' | 'session' | 'command' | 'config' | 'snapshot' | 'setting'

type RecentItem = {
  id: string
  type: Exclude<RecentFilter, 'all'>
  icon: string
  title: string
  description: string
  meta: string
  badge: string
  timestamp: string
  command?: string
  workspaceId?: string
  tabId?: string
  paneId?: string
  sessionId?: string
  snapshotId?: string
  entryId?: string
  previewTabs?: WorkspaceTab[]
  previewActiveTabId?: string | null
  sourceSessionLabel?: string
  onOpen?: () => void
}

type SearchResultItem = {
  id: string
  type: SearchResultType
  icon: string
  title: string
  description: string
  meta: string
  actionLabel: string
  score?: number
  command?: string
  onOpen: () => void
}

type DropEdge = 'left' | 'right' | 'top' | 'bottom'

type DragInsertMarker = {
  left: number
  top: number
  height: number
}

type DragPreviewRect = {
  left: number
  top: number
  width: number
  height: number
}

type DragDropTarget =
  | {
      kind: 'tabbar'
      paneId: string
      index: number
      marker: DragInsertMarker
    }
  | {
      kind: 'pane-split'
      paneId: string
      edge: DropEdge
      rect: DragPreviewRect
    }
  | {
      kind: 'global-split'
      edge: DropEdge
      rect: DragPreviewRect
    }
  | null

type SessionDragState = {
  sourcePaneId: string
  sourceSessionId: string
  sessionName: string
  pointerId: number
  active: boolean
  moved: boolean
  startX: number
  startY: number
  currentX: number
  currentY: number
  offsetX: number
  offsetY: number
  width: number
  height: number
  tabBarHoverKey: string | null
  tabBarHoverSince: number
}

type PendingTabPress = {
  paneId: string
  sessionId: string
  pointerId: number
  startX: number
  startY: number
}

type SplitResizeState = {
  splitPaneId: string
  direction: SplitDirection
  leftChildId: string
  rightChildId: string
  startX: number
  startY: number
  containerWidth: number
  containerHeight: number
  leftRatio: number
  rightRatio: number
}

type CachedDropZone = {
  type: 'tabbar' | 'pane'
  paneId: string
  element: HTMLElement
  rect: DOMRect
}

type WorkspaceFocusState = {
  activeTabId: string | null
  activePaneId: string | null
  activePaneIdsByTab: Record<string, string>
  activeSessionIdsByPane: Record<string, string>
  collapsedTreeTabIds: string[]
}

type WorkbenchRestoreState = {
  version: 1
  selectedWorkspaceId: string | null
  selectedOverviewWorkspaceId: string | null
  openedWorkspaceIds: string[]
  collapsedWorkspaceIds: string[]
  workspaceFocus: Record<string, WorkspaceFocusState>
  updatedAt: string
}


type ThemePreset = {
  id: string
  name: string
  kind: string
  description: string
  accent: string
  background: string
  background2: string
  panel: string
  panelElevated: string
  accentBlue: string
  textPrimary: string
  textSecondary: string
  swatches: string[]
  scheme?: 'light' | 'dark'
}

type RailItem = {
  id: string
  label: string
  icon: string
  summary: string
  active: boolean
  action: () => void
}

type HelpSection = {
  title: string
  body?: string
  items?: string[]
}

type HelpContent = {
  title: string
  description: string
  sections: HelpSection[]
}

const themePresets: ThemePreset[] = [
  {
    id: 'default',
    name: '\u660e\u4eae\u9ed8\u8ba4',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u6e29\u548c\u4e2d\u6027\u8272\u57fa\u5e95\uff0c\u9002\u5408\u4f5c\u4e3a\u9ed8\u8ba4\u5de5\u4f5c\u4e3b\u9898\u3002',
    accent: '#4f7cff',
    background: '#f6f8fb',
    background2: '#eef2f7',
    panel: '#ffffff',
    panelElevated: '#f3f6fb',
    accentBlue: '#5b8cff',
    textPrimary: '#18212b',
    textSecondary: '#5f7286',
    swatches: ['#f6f8fb', '#ffffff', '#4f7cff', '#5b8cff'],
    scheme: 'light',
  },
  {
    id: 'orange',
    name: '\u660e\u4eae\u6a59',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u6696\u767d\u5e95\u914d\u6a59\u8272\u5f3a\u8c03\uff0c\u4fe1\u606f\u6e05\u6670\u800c\u4e0d\u523a\u773c\u3002',
    accent: '#eb8a2f',
    background: '#fbf7f1',
    background2: '#f5eee4',
    panel: '#ffffff',
    panelElevated: '#f7f1e8',
    accentBlue: '#ef9f49',
    textPrimary: '#211b15',
    textSecondary: '#716356',
    swatches: ['#fbf7f1', '#ffffff', '#eb8a2f', '#ef9f49'],
    scheme: 'light',
  },
  {
    id: 'blue',
    name: '\u660e\u4eae\u84dd',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u51b7\u767d\u5e95\u914d\u84dd\u8272\u5f3a\u8c03\uff0c\u66f4\u63a5\u8fd1\u4e13\u4e1a\u684c\u9762\u5de5\u5177\u3002',
    accent: '#4b83ff',
    background: '#f4f8fd',
    background2: '#ebf1f8',
    panel: '#ffffff',
    panelElevated: '#f0f5fb',
    accentBlue: '#6a9cff',
    textPrimary: '#172233',
    textSecondary: '#607488',
    swatches: ['#f4f8fd', '#ffffff', '#4b83ff', '#6a9cff'],
    scheme: 'light',
  },
  {
    id: 'purple',
    name: '\u660e\u4eae\u7d2b',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u67d4\u548c\u7d2b\u8272\u5f3a\u8c03\uff0c\u4fdd\u7559\u4e00\u70b9\u4e2a\u6027\u4f46\u6574\u4f53\u4ecd\u504f\u6e05\u723d\u3002',
    accent: '#8b6cff',
    background: '#f7f5fd',
    background2: '#efebfa',
    panel: '#ffffff',
    panelElevated: '#f3effb',
    accentBlue: '#a084ff',
    textPrimary: '#201a2f',
    textSecondary: '#6f6685',
    swatches: ['#f7f5fd', '#ffffff', '#8b6cff', '#a084ff'],
    scheme: 'light',
  },
  {
    id: 'pink',
    name: '\u660e\u4eae\u7c89',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u96fe\u7c89\u5f3a\u8c03\uff0c\u6574\u4f53\u66f4\u8f7b\u4f46\u4fdd\u6301\u8db3\u591f\u5bf9\u6bd4\u3002',
    accent: '#d974a5',
    background: '#fcf7fa',
    background2: '#f5eef4',
    panel: '#ffffff',
    panelElevated: '#f8f1f6',
    accentBlue: '#e18bb5',
    textPrimary: '#241b22',
    textSecondary: '#7a6975',
    swatches: ['#fcf7fa', '#ffffff', '#d974a5', '#e18bb5'],
    scheme: 'light',
  },
  {
    id: 'dark-default',
    name: '暗色经典',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '面向长时间终端工作的低亮度主题，和浅色主题共用同一套组件语义。',
    accent: '#6aa6ff',
    background: '#0f141d',
    background2: '#141b26',
    panel: '#192230',
    panelElevated: '#202b3a',
    accentBlue: '#7db5ff',
    textPrimary: '#edf3fb',
    textSecondary: '#9aa9ba',
    swatches: ['#0f141d', '#192230', '#6aa6ff', '#7db5ff'],
    scheme: 'dark',
  },
  {
    id: 'ayu-mirage',
    name: 'Ayu Mirage',
    kind: '\u5e38\u89c1\u4e3b\u9898',
    description: '\u4fdd\u7559\u4e00\u4e2a\u6210\u719f\u6697\u8272\u4e3b\u9898\u4f5c\u4e3a\u53ef\u9009\u9879\u3002',
    accent: '#ffb454',
    background: '#1f2430',
    background2: '#242936',
    panel: '#2a3140',
    panelElevated: '#323b4d',
    accentBlue: '#59c2ff',
    textPrimary: '#dfe7f1',
    textSecondary: '#95a3b3',
    swatches: ['#1f2430', '#2a3140', '#ffb454', '#59c2ff'],
    scheme: 'dark',
  },
  {
    id: 'arc-dark',
    name: 'Arc Dark',
    kind: '\u5e38\u89c1\u4e3b\u9898',
    description: '\u66f4\u504f\u684c\u9762\u5de5\u5177\u7684\u4e2d\u6027\u6697\u8272\u3002',
    accent: '#5294e2',
    background: '#2f343f',
    background2: '#373d49',
    panel: '#404552',
    panelElevated: '#4a5160',
    accentBlue: '#7ab0ef',
    textPrimary: '#f2f4f8',
    textSecondary: '#bcc4cf',
    swatches: ['#2f343f', '#404552', '#5294e2', '#7ab0ef'],
    scheme: 'dark',
  },
  {
    id: 'catppuccin-latte',
    name: 'Catppuccin Latte',
    kind: '\u5e38\u89c1\u4e3b\u9898',
    description: '\u67d4\u548c\u6d45\u8272\u4e3b\u9898\uff0c\u9002\u5408\u4f5c\u4e3a\u5907\u7528\u660e\u4eae\u98ce\u683c\u3002',
    accent: '#7287fd',
    background: '#eff1f5',
    background2: '#e6e9ef',
    panel: '#ffffff',
    panelElevated: '#dce0e8',
    accentBlue: '#8caaee',
    textPrimary: '#4c4f69',
    textSecondary: '#6c6f85',
    swatches: ['#eff1f5', '#ffffff', '#7287fd', '#dd7878'],
    scheme: 'light',
  },
]

const appSection = ref<AppSection>('home')
const workspaceView = ref<WorkspaceView>('overview')
const themePanelTab = ref<ThemePanelTab>('theme')
const openThemeModal = ref(false)
const openSearchModal = ref(false)
const openRecentRecycleBinModal = ref(false)
const recentFilter = ref<RecentFilter>('all')
const recentWorkspaceFilter = ref('all')
const pinnedRecentItemIds = ref<string[]>([])
const hiddenRecentItemIds = ref<string[]>([])
const templateFilter = ref<'all' | 'system' | 'user'>('all')
const templateTagFilter = ref('all')
const appSearchQuery = ref('')
const searchResultActiveId = ref<string | null>(null)
const searchLoopHint = ref('')
const activeSettingsSection = ref<SettingsSection>('appearance')
const terminalReloadVersions = shallowRef<Record<string, number>>({})
const userWorkflowTemplates = ref<WorkflowTemplate[]>(loadUserWorkflowTemplates())
const openHelpDrawer = ref(false)
const openWorkspaceEditorModal = ref(false)
const openTerminalEntryEditorModal = ref(false)
const openProviderEditorModal = ref(false)
const openTemplateEditorModal = ref(false)
const openTerminalEntriesModal = ref(false)
const activePaneMenu = ref<string | null>(null)
const activePaneHeaderMenu = ref<string | null>(null)
const activePaneBindingMenu = ref<string | null>(null)
const activePaneSessionMenu = ref<{ paneId: string; sessionId: string } | null>(null)
const activeCommandPanelPaneId = ref<string | null>(null)
const activeWorkspaceMenu = ref<string | null>(null)
const activeWorkspaceMenuPosition = ref<{ x: number; y: number } | null>(null)
const activePaneMenuPosition = ref<{ x: number; y: number } | null>(null)
const activePaneHeaderMenuPosition = ref<{ x: number; y: number } | null>(null)
const activePaneBindingMenuPosition = ref<{ x: number; y: number } | null>(null)
const activePaneSessionMenuPosition = ref<{ x: number; y: number } | null>(null)
const activeRuntimeTabMenuId = ref<string | null>(null)
const activeRuntimeTabMenuPosition = ref<{ x: number; y: number } | null>(null)
const activeExplorerProjectMenuId = ref<string | null>(null)
const activeExplorerProjectWorkspaceId = ref<string | null>(null)
const activeExplorerProjectMenuPosition = ref<{ x: number; y: number } | null>(null)
const activeRailTooltipId = ref<string | null>(null)
const openRecentWorkspaceFilterMenu = ref(false)
const openLaunchModeMenu = ref(false)
const openSplitActionModal = ref(false)
const openConfirmModal = ref(false)
const railCollapsed = ref(false)
const activeThemeId = ref('default')
const isDevBuild = import.meta.env.DEV
const activeHelpTopicId = ref('layout')
const customAccentHex = ref('#3dd6c6')
const terminalFontSize = ref(13)
const terminalFontFamily = ref('Cascadia Code')
const systemRefreshInterval = ref<'manual' | '5s' | '10s' | '30s'>('manual')
const restoreCommandStrategy = ref<RestoreCommandStrategy>('layout-only')
const hiddenEnvironmentItems = ref<string[]>([])
const systemNotificationsEnabled = ref(true)
const windowAttentionEnabled = ref(true)
const systemStatusRefreshing = ref(false)
const environmentChecksRefreshing = ref(false)
const hasSystemStatusCache = ref(false)
const hasEnvironmentCheckCache = ref(false)
const systemRefreshCountdown = ref(0)
const systemStatusCacheUpdatedAt = ref(0)
const environmentCheckCacheUpdatedAt = ref(0)
const uiToast = ref<{ id: number; title: string; message: string; progress: number; duration: number } | null>(null)
let saveUiPreferencesTimer: ReturnType<typeof setTimeout> | null = null
let searchLoopHintTimer: ReturnType<typeof setTimeout> | null = null
const sessionAttentionNotificationState = new Map<string, SessionAttentionState>()
const sessionOutputTailBySession = new Map<string, string>()
let lastAttentionBadgeCount = -1
const startupPerf = reactive({
  appMountedAt: 0,
  firstHomePaintAt: 0,
  workbenchEnteredAt: 0,
  systemStatusResolvedAt: 0,
  environmentResolvedAt: 0,
})
const confirmDialog = reactive({
  title: '',
  description: '',
  confirmLabel: '确认',
  variant: 'danger' as 'primary' | 'danger',
  details: [] as string[],
})
const splitActionState = reactive<{
  mode: 'create' | 'duplicate'
  paneId: string | null
}>({
  mode: 'create',
  paneId: null,
})
const openRenameModal = ref(false)
const renameTarget = reactive<{ kind: 'tab' | 'pane' | 'session'; id: string; title: string; placeholder: string; value: string }>({
  kind: 'tab',
  id: '',
  title: '',
  placeholder: '',
  value: '',
})
const renameInputRef = ref<HTMLInputElement | null>(null)
const workspaces = ref<WorkspaceCard[]>(loadWorkspaces())
const initialWorkbenchRestoreState = loadWorkbenchRestoreState(workspaces.value)
const initialSelectedWorkspaceId = resolveRestoredWorkspaceId(workspaces.value, initialWorkbenchRestoreState.selectedWorkspaceId)
const initialSelectedWorkspace = workspaces.value.find((workspace) => workspace.id === initialSelectedWorkspaceId)
const initialWorkspaceFocus = initialSelectedWorkspaceId ? initialWorkbenchRestoreState.workspaceFocus[initialSelectedWorkspaceId] : undefined
const initialActiveTabId = resolveRestoredTabId(initialSelectedWorkspace, initialWorkspaceFocus?.activeTabId)
const initialActiveTab = initialSelectedWorkspace?.tabs.find((tab) => tab.id === initialActiveTabId)
const initialActivePaneId = resolveRestoredPaneId(initialActiveTab, initialWorkspaceFocus?.activePaneId)
const openedWorkspaceIds = ref<string[]>(resolveOpenedWorkspaceIds(workspaces.value, initialWorkbenchRestoreState.openedWorkspaceIds, initialSelectedWorkspaceId))
const selectedOverviewWorkspaceId = ref(resolveRestoredWorkspaceId(workspaces.value, initialWorkbenchRestoreState.selectedOverviewWorkspaceId))
const selectedWorkspaceId = ref(initialSelectedWorkspaceId)
const activeRuntimeTabId = ref(initialActiveTabId)
const activeRuntimePaneId = ref(initialActivePaneId)
const runtimeActiveSessionIds = shallowRef<Record<string, string>>(resolveRestoredSessionIds(initialSelectedWorkspace, initialWorkspaceFocus?.activeSessionIdsByPane))
const workspaceFocusCache = shallowRef<Record<string, WorkspaceFocusState>>(initialWorkbenchRestoreState.workspaceFocus)

const runtimeCanvasRef = ref<HTMLElement | null>(null)
const draggingSession = ref<SessionDragState | null>(null)
const dragDropTarget = ref<DragDropTarget>(null)
const pendingTabPress = ref<PendingTabPress | null>(null)
const splitResizeState = ref<SplitResizeState | null>(null)
const suppressSessionClickUntil = ref(0)
let dragPointerCleanup: (() => void) | null = null
let splitResizeCleanup: (() => void) | null = null
let dragRafId: number | null = null
let cachedDropZones: CachedDropZone[] = []

const collapsedWorkspaceIds = ref<string[]>(resolveCollapsedWorkspaceIds(workspaces.value, initialWorkbenchRestoreState.collapsedWorkspaceIds))
const collapsedTreeTabIds = ref<string[]>(resolveRestoredCollapsedTreeTabIds(initialSelectedWorkspace, initialWorkspaceFocus?.collapsedTreeTabIds))
const workbenchSidebarWidth = ref(loadWorkbenchSidebarWidth())
let workbenchResizeCleanup: (() => void) | null = null
let saveWorkspacesTimer: number | null = null
let saveWorkspacesIdleHandle: number | null = null
let saveWorkbenchRestoreStateTimer: ReturnType<typeof setTimeout> | null = null
let nextWorkspacePersistenceMode: 'persist' | 'transient' = 'persist'
let systemRefreshTimer: number | null = null
let systemRefreshTickTimer: number | null = null
let environmentRefreshRunId = 0
let supervisorScanTimer: number | null = null
let suppressFloatingMenuCloseUntil = 0
const tauriViewportWidth = ref<number | null>(null)
let unlistenWindowResize: (() => void) | null = null
let lastStandaloneShiftAt = 0
const workspaceEditorMode = ref<'create' | 'edit'>('create')
const terminalEntryEditorMode = ref<'create' | 'edit'>('create')
const providerEditorMode = ref<'create' | 'edit'>('create')
const templateEditorMode = ref<'create' | 'edit'>('create')
const editingWorkspaceId = ref<string | null>(null)
const editingTerminalEntryId = ref<string | null>(null)
const editingProviderId = ref<string | null>(null)
const editingTemplateId = ref<string | null>(null)

const workspaceForm = reactive({
  name: '',
  rootPath: '',
  description: '',
  tagsText: '',
})

const terminalEntryForm = reactive({
  name: '',
  workingDirectory: '',
  defaultCommand: '',
  launchMode: 'open-only' as TerminalEntry['launchMode'],
  providerBindingMode: 'inherit' as NonNullable<TerminalEntry['providerBindingMode']>,
  providerProfileId: '',
  modelId: '',
  environmentVariablesText: '',
  mcpPolicy: 'inherit' as NonNullable<TerminalEntry['mcpPolicy']>,
  skillPolicy: 'inherit' as NonNullable<TerminalEntry['skillPolicy']>,
  tagsText: '',
  note: '',
})

const providerForm = reactive({
  name: '',
  providerKind: 'openai-compatible' as ProviderProfile['providerKind'],
  baseUrl: '',
  apiKey: '',
  apiFormat: 'openai' as ProviderProfile['apiFormat'],
  defaultModel: '',
  toolTargetsText: 'codex, generic',
  color: '#4b83ff',
  note: '',
  isDefault: true,
})

const workflowTemplateForm = reactive({
  name: '',
  description: '',
  tagsText: '',
})

const SYSTEM_STATUS_CACHE_KEY = 'chuchen-terminal.system-status-cache.v1'
const ENVIRONMENT_CHECK_CACHE_KEY = 'chuchen-terminal.environment-check-cache.v1'
const WORKBENCH_RESTORE_STATE_KEY = 'chuchen-terminal.workbench-restore-state.v1'
const TAURI_ENVIRONMENT_CHECK_NAMES = ['Node.js', 'Python', 'Java', 'Go', 'Rust', 'PowerShell', 'Git / GitHub'] as const
const SYSTEM_STATUS_CACHE_TTL_MS = 1000 * 60 * 10
const ENVIRONMENT_CHECK_CACHE_TTL_MS = 1000 * 60 * 60 * 6
const INITIAL_SYSTEM_REFRESH_DELAY_MS = 8000
const INITIAL_SYSTEM_CACHE_REFRESH_DELAY_MS = 16000
const INITIAL_ENVIRONMENT_REFRESH_DELAY_MS = 18000
const INITIAL_ENVIRONMENT_CACHE_REFRESH_DELAY_MS = 28000
const ENVIRONMENT_CHECK_GAP_MS = 1400
const SUPERVISOR_SCAN_INTERVAL_MS = 30000
const SUPERVISOR_STALLED_THRESHOLD_MS = 1000 * 60 * 2

const recentCommandsByPane = computed<Record<string, string[]>>(() => {
  const result: Record<string, string[]> = {}
  selectedWorkspace.value?.tabs.forEach((tab) => {
    flattenLeafPanes(tab.panes).forEach((pane) => {
      const commands = paneSessions(pane)
        .map((session) => entryById(session.terminalEntryId)?.lastCommand?.trim() || '')
        .filter(Boolean)
      result[pane.id] = Array.from(new Set(commands)).slice(0, 6)
    })
  })
  return result
})

const launchModeOptions: Array<{ value: TerminalEntry['launchMode']; label: string; description: string }> = [
  { value: 'open-only', label: '仅打开窗口', description: '只打开终端，不自动填充和发送命令。' },
  { value: 'prefill', label: '预填命令', description: '将命令填入终端输入区，由用户确认后发送。' },
  { value: 'execute', label: '立即执行', description: '打开终端后按配置发送默认命令，适合可信的本地开发命令。' },
  { value: 'switch-or-create', label: '切换或创建', description: '优先切换已有终端，没有再创建新的终端。' },
]

const restoreCommandStrategyOptions: Array<{ value: RestoreCommandStrategy; label: string; description: string }> = [
  { value: 'layout-only', label: '仅布局', description: '只恢复项目、Pane 和终端标签，不写入命令。' },
  { value: 'prefill', label: '预填', description: '按运行配置预填默认命令，但不自动回车。' },
  { value: 'execute', label: '执行', description: '仅对启动模式为“立即执行”的配置发送命令。' },
]

let confirmAction: (() => void) | null = null

const fontOptions = ['Cascadia Code', 'FiraCode Nerd Font', 'JetBrains Mono', 'Consolas', 'Geist Mono']

const helpContentMap: Record<string, HelpContent> = {
  layout: {
    title: '布局说明',
    description: '抽屉用于持续浏览的辅助信息，不打断主终端画布。',
    sections: [
      {
        title: '双层结构',
        body: '保留工作区卡片首页作为总览入口；进入后切换到左控右画布的工作台壳层。',
      },
      {
        title: '核心层级',
        items: ['工作区卡片是最高一级入口。', '工作区内部以 Tab 组织页面单元。', 'Tab 内部用 Pane 承载真正的终端实例。'],
      },
      {
        title: '终端能力',
        items: ['已接入本地终端运行时。', '支持工作现场保存、恢复与任务提醒。'],
      },
    ],
  },
  workspace: {
    title: '工作区',
    description: '工作区菜单负责卡片首页、进入工作台，以及已打开工作区之间的切换。',
    sections: [
      {
        title: '为什么保留卡片首页',
        items: ['目录一多时，纯列表不利于总览。', '卡片更适合展示 Tab / Pane 数量、最近使用时间和标签。'],
      },
      {
        title: '进入后如何工作',
        items: ['左侧切换已打开工作区。', '左侧项目树负责 Tab / Pane 层级导航。', '右侧区域专注于终端工作台本身。'],
      },
    ],
  },
  recent: {
    title: '最近',
    description: '后续这里会收纳最近恢复的工作区、最近打开的会话以及高频命令入口。',
    sections: [
      {
        title: '规划用途',
        items: ['最近打开的工作区。', '最近恢复的布局快照。', '最近使用的运行配置与命令。'],
      },
    ],
  },
  templates: {
    title: '模板',
    description: '模板用于沉淀常用工作区蓝图，而不是只保存某一次运行态。',
    sections: [
      {
        title: '规划用途',
        items: ['常用前后端双 Pane 模板。', 'AI CLI 工作台模板。', '项目初始化与测试脚本模板。'],
      },
    ],
  },
  search: {
    title: '搜索',
    description: '搜索入口会跨工作区检索名称、路径、运行配置和高频命令。',
    sections: [
      {
        title: '建议范围',
        items: ['工作区名称与标签。', '路径与配置名。', '默认命令与最近命令。'],
      },
    ],
  },
  theme: {
    title: '主题与字体',
    description: '主题、字体和界面密度属于高频配置，适合居中 Modal。',
    sections: [
      {
        title: '设计原则',
        items: ['默认提供系统主题预设。', '默认主题允许自由调整主色。', '字体与主题统一在一个面板中切换。'],
      },
    ],
  },
  settings: {
    title: '设置',
    description: '设置用于放置较低频的全局偏好，例如默认 Shell、启动行为和桌面端选项。',
    sections: [
      {
        title: '后续规划',
        items: ['默认 Shell 与启动方式。', '窗口关闭、托盘与最小化策略。', 'Tauri 级别的路径与权限偏好。'],
      },
    ],
  },
}

const systemThemes = computed(() => themePresets.filter((theme) => theme.kind === '系统主题'))
const importedThemes = computed(() => themePresets.filter((theme) => theme.kind === '常见主题'))
const selectedThemePreset = computed(() => themePresets.find((theme) => theme.id === activeThemeId.value) ?? themePresets[0])
const selectedWorkspace = computed(() => workspaces.value.find((workspace) => workspace.id === selectedWorkspaceId.value))
const selectedWorkspaceSnapshots = computed(() => selectedWorkspace.value?.snapshots ?? [])
const defaultWorkspaceSnapshot = computed(() => {
  const workspace = selectedWorkspace.value
  if (!workspace) return undefined
  return selectedWorkspaceSnapshots.value.find((snapshot) => snapshot.id === workspace.defaultSnapshotId)
    ?? selectedWorkspaceSnapshots.value[0]
})
const openedWorkspaces = computed(() => openedWorkspaceIds.value
  .map((workspaceId) => workspaces.value.find((workspace) => workspace.id === workspaceId))
  .filter((workspace): workspace is WorkspaceCard => Boolean(workspace)))
const isWorkspaceWorkbench = computed(() => appSection.value === 'workspace' && workspaceView.value !== 'overview')
const activeHelpContent = computed(() => helpContentMap[activeHelpTopicId.value] ?? helpContentMap.layout)
const selectedWorkspaceEntries = computed(() => selectedWorkspace.value?.terminalEntries ?? [])
const selectedWorkspaceProviders = computed(() => selectedWorkspace.value?.providerProfiles ?? [])
const referencedTerminalEntryIds = computed(() => {
  const ids = selectedWorkspace.value?.tabs.flatMap((tab) => flattenLeafPanes(tab.panes).map((pane) => pane.terminalEntryId).filter((entryId): entryId is string => Boolean(entryId))) ?? []
  return new Set(ids)
})
const recentHomeWorkspaces = computed(() => [...workspaces.value]
  .sort((left, right) => new Date(right.lastOpenedAt).getTime() - new Date(left.lastOpenedAt).getTime())
  .slice(0, 4))
const totalWorkspaceCount = computed(() => workspaces.value.length)
const totalPaneCount = computed(() => workspaces.value.reduce((count, workspace) => count + totalPanes(workspace), 0))
const totalRunningCount = computed(() => workspaces.value.reduce((count, workspace) => count + runningCount(workspace), 0))
const visibleEnvironmentChecks = computed(() => homeEnvironmentChecks.filter((item) => !hiddenEnvironmentItems.value.includes(item.name)))
const allWorkflowTemplates = computed(() => [...systemWorkflowTemplates, ...userWorkflowTemplates.value])
const workflowTemplateFilters = computed<Array<{ id: 'all' | 'system' | 'user'; label: string; count: number; icon: string }>>(() => [
  { id: 'all', label: '全部', count: allWorkflowTemplates.value.length, icon: 'template' },
  { id: 'system', label: '系统模板', count: allWorkflowTemplates.value.filter((template) => template.kind === 'system').length, icon: 'workspace' },
  { id: 'user', label: '我的模板', count: userWorkflowTemplates.value.length, icon: 'edit' },
])
const workflowTemplateTagFilters = computed(() => {
  const tags = Array.from(new Set(allWorkflowTemplates.value.flatMap((template) => template.tags))).sort((a, b) => a.localeCompare(b, 'zh-CN'))
  return ['all', ...tags]
})
const filteredWorkflowTemplates = computed(() => {
  let templates = allWorkflowTemplates.value
  if (templateFilter.value !== 'all') {
    templates = templates.filter((template) => template.kind === templateFilter.value)
  }
  if (templateTagFilter.value !== 'all') {
    templates = templates.filter((template) => template.tags.includes(templateTagFilter.value))
  }
  return templates
})
const recentWorkspaceOptions = computed(() => [
  { id: 'all', label: '全部工作区' },
  ...workspaces.value.map((workspace) => ({ id: workspace.id, label: workspace.name })),
])
const hiddenRecentItemCount = computed(() => hiddenRecentItemIds.value.length)
const removedRecentItems = computed(() =>
  allRecentItems.value.filter((item) => recentItemIsHidden(item.id)),
)
const recentWorkspaceFilterLabel = computed(() => recentWorkspaceOptions.value.find((workspace) => workspace.id === recentWorkspaceFilter.value)?.label ?? '全部工作区')
const recentFilters = computed<Array<{ id: RecentFilter; label: string; count: number; icon: string }>>(() => {
  const items = allRecentItems.value
  return [
    { id: 'all', label: '全部', count: items.length, icon: 'recent' },
    { id: 'workspace', label: '工作区', count: items.filter((item) => item.type === 'workspace').length, icon: 'workspace' },
    { id: 'project', label: '项目', count: items.filter((item) => item.type === 'project').length, icon: 'tab' },
    { id: 'session', label: '终端', count: items.filter((item) => item.type === 'session').length, icon: 'terminal' },
    { id: 'command', label: '历史命令', count: items.filter((item) => item.type === 'command').length, icon: 'bolt' },
    { id: 'snapshot', label: '布局', count: items.filter((item) => item.type === 'snapshot').length, icon: 'copy' },
  ]
})
const allRecentItems = computed<RecentItem[]>(() => {
  const items: RecentItem[] = []

  workspaces.value.forEach((workspace) => {
    items.push({
      id: `workspace-${workspace.id}`,
      type: 'workspace',
      icon: 'workspace',
      title: workspace.name,
      description: workspace.rootPath,
      meta: `${workspace.tabs.length} 项目 · ${totalWorkspaceSessions(workspace)} 终端`,
      badge: relativeTimeLabel(workspace.lastOpenedAt),
      timestamp: workspace.lastOpenedAt,
      workspaceId: workspace.id,
      onOpen: () => openWorkspace(workspace.id),
    })

    workspace.tabs.forEach((tab) => {
      items.push({
        id: `project-${workspace.id}-${tab.id}`,
        type: 'project',
        icon: 'tab',
        title: tab.name,
        description: `${workspace.name} · ${workspace.rootPath}`,
        meta: `${countLeafPanes(tab.panes)} Pane · ${countPaneSessions(tab.panes)} 终端`,
        badge: relativeTimeLabel(tab.updatedAt || workspace.updatedAt),
        timestamp: tab.updatedAt || workspace.updatedAt,
        workspaceId: workspace.id,
        tabId: tab.id,
        onOpen: () => openProjectWorkspace(workspace.id, tab.id),
      })

      flattenLeafPanes(tab.panes).forEach((pane) => {
        paneSessions(pane).forEach((session) => {
          const entry = workspaceEntryById(workspace, session.terminalEntryId)
          items.push({
            id: `session-${workspace.id}-${tab.id}-${pane.id}-${session.id}`,
            type: 'session',
            icon: 'terminal',
            title: session.name || pane.name,
            description: entry?.workingDirectory || session.pathLabel || pane.pathLabel,
            meta: `${workspace.name} / ${tab.name}`,
            badge: explorerSessionLabel(session),
            timestamp: session.lastActivityAt || session.lastOutputAt || session.lastCommandAt || entry?.updatedAt || workspace.updatedAt,
            workspaceId: workspace.id,
            tabId: tab.id,
            paneId: pane.id,
            sessionId: session.id,
            onOpen: () => openWorkspaceTerminalSession(workspace.id, tab.id, pane.id, session.id),
          })
        })
      })
    })

    workspace.terminalEntries.forEach((entry) => {
      uniqueCommandList([entry.lastCommand, ...(entry.commandHistory ?? []), entry.defaultCommand], 6).forEach((command, commandIndex) => {
        items.push({
          id: `command-${workspace.id}-${entry.id}-${commandIndex}`,
          type: 'command',
          icon: 'bolt',
          title: command,
          description: entry.workingDirectory,
          meta: `${workspace.name} / ${entry.name}`,
          badge: entry.tags[0] || '历史命令',
          timestamp: entry.updatedAt,
          command,
          workspaceId: workspace.id,
          entryId: entry.id,
          sourceSessionLabel: entry.name,
          onOpen: () => openRecentCommandTarget(workspace.id, entry.id, command),
        })
      })
    })

    ;(workspace.snapshots ?? []).forEach((snapshot) => {
      items.push({
        id: `snapshot-${workspace.id}-${snapshot.id}`,
        type: 'snapshot',
        icon: 'copy',
        title: snapshot.name,
        description: workspace.rootPath,
        meta: `${workspace.name} · ${snapshot.tabsState.length} 项目现场`,
        badge: relativeTimeLabel(snapshot.updatedAt),
        timestamp: snapshot.updatedAt,
        workspaceId: workspace.id,
        snapshotId: snapshot.id,
        previewTabs: snapshot.tabsState,
        previewActiveTabId: snapshot.activeTabId,
        onOpen: () => restoreWorkspaceSnapshotFromRecent(workspace.id, snapshot.id),
      })
    })
  })

  return items.sort((left, right) => {
    const pinDelta = Number(recentItemIsPinned(right.id)) - Number(recentItemIsPinned(left.id))
    if (pinDelta !== 0) return pinDelta
    return recentItemTimestamp(right) - recentItemTimestamp(left)
  })
})
const filteredRecentItems = computed(() => allRecentItems.value
  .filter((item) => !recentItemIsHidden(item.id))
  .filter((item) => recentFilter.value === 'all' || item.type === recentFilter.value)
  .filter((item) => recentWorkspaceFilter.value === 'all' || item.workspaceId === recentWorkspaceFilter.value)
  .slice(0, 60))
const searchResults = computed<SearchResultItem[]>(() => {
  const query = normalizeSearchText(appSearchQuery.value)
  const results: SearchResultItem[] = []

  workspaces.value.forEach((workspace) => {
    pushSearchResult(results, query, {
      id: `workspace-${workspace.id}`,
      type: 'workspace',
      icon: 'workspace',
      title: workspace.name,
      description: workspace.rootPath,
      meta: `${workspace.tabs.length} 项目 · ${workspace.tags.join(' · ') || '未设置标签'}`,
      actionLabel: '打开工作区',
      onOpen: () => openWorkspace(workspace.id),
    }, [workspace.name, workspace.rootPath, workspace.description, workspace.tags.join(' ')])

    workspace.tabs.forEach((tab) => {
      pushSearchResult(results, query, {
        id: `project-${workspace.id}-${tab.id}`,
        type: 'project',
        icon: 'tab',
        title: tab.name,
        description: workspace.rootPath,
        meta: `${workspace.name} · ${countPaneSessions(tab.panes)} 终端`,
        actionLabel: '定位项目',
        onOpen: () => openProjectWorkspace(workspace.id, tab.id),
      }, [workspace.name, workspace.rootPath, tab.name])

      flattenLeafPanes(tab.panes).forEach((pane) => {
        paneSessions(pane).forEach((session) => {
          const entry = workspaceEntryById(workspace, session.terminalEntryId)
          pushSearchResult(results, query, {
            id: `session-${workspace.id}-${tab.id}-${pane.id}-${session.id}`,
            type: 'session',
            icon: 'terminal',
            title: session.name || pane.name,
            description: entry?.workingDirectory || session.pathLabel || pane.pathLabel,
            meta: `${workspace.name} / ${tab.name}`,
            actionLabel: '打开终端',
            onOpen: () => openWorkspaceTerminalSession(workspace.id, tab.id, pane.id, session.id),
          }, [workspace.name, tab.name, pane.name, session.name, entry?.workingDirectory ?? '', entry?.defaultCommand ?? '', entry?.lastCommand ?? ''])
        })
      })
    })

    workspace.terminalEntries.forEach((entry) => {
      pushSearchResult(results, query, {
        id: `config-${workspace.id}-${entry.id}`,
        type: 'config',
        icon: 'settings',
        title: entry.name,
        description: entry.workingDirectory,
        meta: `${workspace.name} · ${launchModeLabel(entry.launchMode)}`,
        actionLabel: '查看配置',
        onOpen: () => openWorkspaceTerminalEntries(workspace.id),
      }, [entry.name, entry.workingDirectory, entry.defaultCommand, entry.lastCommand, entry.tags.join(' ')])

      uniqueCommandList([entry.lastCommand, ...(entry.commandHistory ?? []), entry.defaultCommand], 8).forEach((command, index) => {
        pushSearchResult(results, query, {
          id: `command-${workspace.id}-${entry.id}-${index}`,
          type: 'command',
          icon: 'bolt',
          title: command,
          description: entry.workingDirectory,
          meta: `${workspace.name} / ${entry.name}`,
          actionLabel: '回到来源终端',
          command,
          onOpen: () => openRecentCommandTarget(workspace.id, entry.id, command),
        }, [command, entry.workingDirectory, entry.name, workspace.name])
      })
    })

    ;(workspace.snapshots ?? []).forEach((snapshot) => {
      pushSearchResult(results, query, {
        id: `snapshot-${workspace.id}-${snapshot.id}`,
        type: 'snapshot',
        icon: 'copy',
        title: snapshot.name,
        description: workspace.name,
        meta: `${snapshot.tabsState.length} 项目 · ${relativeTimeLabel(snapshot.updatedAt)}`,
        actionLabel: '恢复现场',
        onOpen: () => restoreWorkspaceSnapshotFromRecent(workspace.id, snapshot.id),
      }, [snapshot.name, workspace.name])
    })
  })

  settingsGroups.value.forEach((group) => {
    pushSearchResult(results, query, {
      id: `setting-${group.id}`,
      type: 'setting',
      icon: group.icon,
      title: group.title,
      description: group.description,
      meta: '设置入口',
      actionLabel: '打开设置',
      onOpen: () => {
        appSection.value = 'settings'
        activeSettingsSection.value = group.id
      },
    }, [group.title, group.description, group.items.join(' ')])
  })

  return results
    .sort((left, right) => (right.score ?? 0) - (left.score ?? 0))
    .slice(0, 80)
})
const groupedSearchResults = computed(() => {
  const groups: Array<{ key: string; type: SearchResultType; title: string; items: SearchResultItem[] }> = [
    { key: 'best', type: 'workspace', title: '最佳匹配', items: [] },
    { key: 'workspace', type: 'workspace', title: '工作区', items: [] },
    { key: 'project', type: 'project', title: '项目', items: [] },
    { key: 'session', type: 'session', title: '终端', items: [] },
    { key: 'command', type: 'command', title: '历史命令', items: [] },
    { key: 'config', type: 'config', title: '配置', items: [] },
    { key: 'snapshot', type: 'snapshot', title: '布局快照', items: [] },
    { key: 'setting', type: 'setting', title: '设置入口', items: [] },
  ]

  if (normalizeSearchText(appSearchQuery.value)) {
    const bestMatchGroup = groups[0]
    searchResults.value.slice(0, 5).forEach((result) => {
      bestMatchGroup.items.push(result)
    })
  }

  searchResults.value.forEach((result) => {
    groups.slice(1).find((group) => group.type === result.type)?.items.push(result)
  })

  return groups.filter((group) => group.items.length)
})
const pageSearchResults = computed(() => groupedSearchResults.value.flatMap((group) => group.items))
const quickSearchResults = computed(() => groupedSearchResults.value.flatMap((group) => group.items.slice(0, 5)))
const activeSearchResultList = computed(() => openSearchModal.value ? quickSearchResults.value : pageSearchResults.value)
const settingsGroups = computed<Array<{ id: SettingsSection; title: string; icon: string; badge: string; description: string; items: string[] }>>(() => [
  {
    id: 'appearance',
    title: '外观',
    icon: 'theme',
    badge: '即时预览',
    description: '主题、字体、界面密度和 Rail 记忆。',
    items: [`当前主题：${selectedThemePreset.value.name}`, `终端字号：${terminalFontSize.value}px`, `Rail：${railCollapsed.value ? '默认收起' : '默认展开'}`],
  },
  {
    id: 'terminal',
    title: '终端',
    icon: 'terminal',
    badge: '运行态生效',
    description: '默认 Shell、启动行为和命令恢复策略。',
    items: ['默认 Shell：PowerShell 7', `命令恢复：${restoreCommandStrategyLabel(restoreCommandStrategy.value)}`, '自动执行：默认禁用'],
  },
  {
    id: 'system',
    title: '系统与环境',
    icon: 'runtime',
    badge: '检测与缓存',
    description: '系统资源刷新、环境检测缓存与手动检测。',
    items: [`资源刷新：${systemRefreshLabel.value}`, `环境项：${visibleEnvironmentChecks.value.length} 个展示`, hasEnvironmentCheckCache.value ? '环境缓存：已加载' : '环境缓存：待生成'],
  },
  {
    id: 'notifications',
    title: '通知',
    icon: 'recent',
    badge: '提醒与角标',
    description: '系统通知、任务栏提醒和待处理计数。',
    items: [`待处理终端：${countAttentionSessions()}`, `系统通知：${systemNotificationsEnabled.value ? '已开启' : '已关闭'}`, `任务栏提醒：${windowAttentionEnabled.value ? '已开启' : '已关闭'}`],
  },
  {
    id: 'supervisor',
    title: '任务监督',
    icon: 'bolt',
    badge: '任务提醒',
    description: '完成、等待输入、异常和停滞识别。',
    items: [`停滞阈值：${Math.round(SUPERVISOR_STALLED_THRESHOLD_MS / 60000)} 分钟`, '提醒状态：Explorer / 顶栏 / 系统通知共用', isDevBuild ? '调试入口：可用' : '调试入口：隐藏'],
  },
  {
    id: 'data',
    title: '数据',
    icon: 'copy',
    badge: '本地持久化',
    description: '本地数据、导入导出和配置迁移。',
    items: ['工作区数据：本地持久化', '配置导入：预览后确认', '导出数据：自动排除敏感字段'],
  },
])
const systemRefreshLabel = computed(() => {
  if (systemStatusRefreshing.value) return '刷新中'
  if (systemRefreshInterval.value === 'manual') return '手动刷新'
  if (systemRefreshCountdown.value > 0) return `${systemRefreshCountdown.value}s 后刷新`
  return `${systemRefreshInterval.value} 自动刷新`
})
const overviewWorkspaceIndex = computed(() => {
  const index = workspaces.value.findIndex((workspace) => workspace.id === selectedOverviewWorkspaceId.value)
  return index >= 0 ? index : 0
})
const activeOverviewWorkspace = computed(() => workspaces.value[overviewWorkspaceIndex.value])
const templateApplyTargetWorkspace = computed(() => selectedWorkspace.value ?? activeOverviewWorkspace.value ?? workspaces.value[0] ?? null)
const previousOverviewWorkspace = computed(() => {
  if (workspaces.value.length < 2) return undefined
  const index = (overviewWorkspaceIndex.value - 1 + workspaces.value.length) % workspaces.value.length
  return workspaces.value[index]
})
const nextOverviewWorkspace = computed(() => {
  if (workspaces.value.length < 2) return undefined
  const index = (overviewWorkspaceIndex.value + 1) % workspaces.value.length
  return workspaces.value[index]
})
const systemStatus = reactive({
  cpu: '检测中',
  memory: '检测中',
  gpu: '检测中',
})
const homeEnvironmentChecks = reactive([
  { name: 'Node.js', value: '待检测', status: 'pending', note: 'node -v / npm -v', icon: 'N', iconSrc: nodejsIcon },
  { name: 'Python', value: '待检测', status: 'pending', note: 'python --version', icon: 'PY', iconSrc: pythonIcon },
  { name: 'Java', value: '待检测', status: 'pending', note: 'java -version', icon: 'J', iconSrc: javaIcon },
  { name: 'Go', value: '待检测', status: 'pending', note: 'go version', icon: 'GO', iconSrc: goIcon },
  { name: 'Rust', value: '待检测', status: 'pending', note: 'rustc --version', icon: 'R', iconSrc: rustIcon },
  { name: 'PowerShell', value: '待检测', status: 'pending', note: '$PSVersionTable', icon: 'PS', iconSrc: powershellIcon },
  { name: 'Git / GitHub', value: '待检测', status: 'pending', note: 'git --version', icon: 'G', iconSrc: githubIcon, monochrome: true },
  { name: 'Vue 3', value: '项目依赖', status: 'ok', note: '前端 UI 框架', icon: 'V', iconSrc: vueIcon },
  { name: 'TypeScript', value: '项目依赖', status: 'ok', note: '类型系统', icon: 'TS', iconSrc: tsIcon },
  { name: 'Tauri 2', value: '已接入', status: 'ok', note: '桌面壳运行时', icon: 'T', iconSrc: tauriIcon },
  { name: 'xterm.js', value: '已接入', status: 'ok', note: '真实终端渲染', icon: 'XT', iconSrc: powershellIcon },
])
const primaryRailItems = computed<RailItem[]>(() => [
  {
    id: 'home',
    label: '首页',
    icon: 'home',
    summary: '最近项目、环境检测和系统状态总览。',
    active: appSection.value === 'home',
    action: () => {
      appSection.value = 'home'
      workspaceView.value = 'overview'
    },
  },
  {
    id: 'workspace',
    label: '工作区',
    icon: 'workspace',
    summary: '工作区卡片总览与工作台入口。',
    active: appSection.value === 'workspace',
    action: () => goWorkspaceOverview(),
  },
  {
    id: 'recent',
    label: '最近',
    icon: 'recent',
    summary: '最近恢复、最近会话和高频入口。',
    active: appSection.value === 'recent',
    action: () => { appSection.value = 'recent' },
  },
  {
    id: 'templates',
    label: '模板',
    icon: 'template',
    summary: '常用布局、运行配置和启动蓝图。',
    active: appSection.value === 'templates',
    action: () => { appSection.value = 'templates' },
  },
])
const secondaryRailItems = computed<RailItem[]>(() => [
  {
    id: 'search',
    label: '搜索',
    icon: 'search',
    summary: '跨工作区搜索目录、配置与命令。',
    active: appSection.value === 'search',
    action: () => openSearchPage(),
  },
  {
    id: 'theme',
    label: '主题',
    icon: 'theme',
    summary: '切换主题、字体和视觉密度。',
    active: openThemeModal.value,
    action: () => openThemePanel('theme'),
  },
  {
    id: 'settings',
    label: '设置',
    icon: 'settings',
    summary: '全局 Shell、桌面端行为和低频偏好。',
    active: appSection.value === 'settings',
    action: () => { appSection.value = 'settings' },
  },
])

function emptyWorkspaceFocusState(): WorkspaceFocusState {
  return {
    activeTabId: null,
    activePaneId: null,
    activePaneIdsByTab: {},
    activeSessionIdsByPane: {},
    collapsedTreeTabIds: [],
  }
}

function loadWorkbenchRestoreState(knownWorkspaces: WorkspaceCard[]): WorkbenchRestoreState {
  const fallback: WorkbenchRestoreState = {
    version: 1,
    selectedWorkspaceId: knownWorkspaces[0]?.id ?? null,
    selectedOverviewWorkspaceId: knownWorkspaces[0]?.id ?? null,
    openedWorkspaceIds: knownWorkspaces[0] ? [knownWorkspaces[0].id] : [],
    collapsedWorkspaceIds: [],
    workspaceFocus: {},
    updatedAt: new Date().toISOString(),
  }

  if (typeof window === 'undefined' || !window.localStorage) return fallback

  try {
    const raw = window.localStorage.getItem(WORKBENCH_RESTORE_STATE_KEY)
    if (!raw) return fallback
    const parsed = JSON.parse(raw) as Partial<WorkbenchRestoreState>
    const workspaceIds = new Set(knownWorkspaces.map((workspace) => workspace.id))
    const workspaceFocus = Object.fromEntries(
      Object.entries(parsed.workspaceFocus ?? {})
        .filter(([workspaceId]) => workspaceIds.has(workspaceId))
        .map(([workspaceId, focus]) => [workspaceId, normalizeWorkspaceFocus(knownWorkspaces.find((workspace) => workspace.id === workspaceId), focus)]),
    )

    return {
      version: 1,
      selectedWorkspaceId: typeof parsed.selectedWorkspaceId === 'string' ? parsed.selectedWorkspaceId : fallback.selectedWorkspaceId,
      selectedOverviewWorkspaceId: typeof parsed.selectedOverviewWorkspaceId === 'string' ? parsed.selectedOverviewWorkspaceId : fallback.selectedOverviewWorkspaceId,
      openedWorkspaceIds: Array.isArray(parsed.openedWorkspaceIds) ? parsed.openedWorkspaceIds.filter((workspaceId) => workspaceIds.has(workspaceId)) : fallback.openedWorkspaceIds,
      collapsedWorkspaceIds: Array.isArray(parsed.collapsedWorkspaceIds) ? parsed.collapsedWorkspaceIds.filter((workspaceId) => workspaceIds.has(workspaceId)) : [],
      workspaceFocus,
      updatedAt: typeof parsed.updatedAt === 'string' ? parsed.updatedAt : fallback.updatedAt,
    }
  } catch {
    return fallback
  }
}

function normalizeWorkspaceFocus(workspace: WorkspaceCard | undefined, focus: Partial<WorkspaceFocusState> | undefined): WorkspaceFocusState {
  if (!workspace || !focus) return emptyWorkspaceFocusState()
  const activeTabId = resolveRestoredTabId(workspace, focus.activeTabId ?? null)
  const activeTab = workspace.tabs.find((tab) => tab.id === activeTabId)
  const activePaneId = resolveRestoredPaneId(activeTab, focus.activePaneId ?? null)
  const validTabIds = new Set(workspace.tabs.map((tab) => tab.id))
  const validPaneIds = new Set(workspace.tabs.flatMap((tab) => flattenLeafPanes(tab.panes).map((pane) => pane.id)))
  const validSessionsByPane = new Map<string, Set<string>>()

  workspace.tabs.forEach((tab) => {
    flattenLeafPanes(tab.panes).forEach((pane) => {
      validSessionsByPane.set(pane.id, new Set(paneSessions(pane).map((session) => session.id)))
    })
  })

  return {
    activeTabId,
    activePaneId,
    activePaneIdsByTab: Object.fromEntries(
      Object.entries(focus.activePaneIdsByTab ?? {}).filter(([tabId, paneId]) => validTabIds.has(tabId) && validPaneIds.has(paneId)),
    ),
    activeSessionIdsByPane: Object.fromEntries(
      Object.entries(focus.activeSessionIdsByPane ?? {}).filter(([paneId, sessionId]) => validSessionsByPane.get(paneId)?.has(sessionId)),
    ),
    collapsedTreeTabIds: resolveRestoredCollapsedTreeTabIds(workspace, focus.collapsedTreeTabIds),
  }
}

function resolveRestoredWorkspaceId(knownWorkspaces: WorkspaceCard[], preferredId?: string | null) {
  return knownWorkspaces.some((workspace) => workspace.id === preferredId)
    ? preferredId || ''
    : knownWorkspaces[0]?.id ?? ''
}

function resolveOpenedWorkspaceIds(knownWorkspaces: WorkspaceCard[], restoredIds: string[], selectedId: string) {
  const workspaceIds = new Set(knownWorkspaces.map((workspace) => workspace.id))
  const ids = Array.from(new Set([...restoredIds.filter((workspaceId) => workspaceIds.has(workspaceId)), selectedId].filter(Boolean)))
  return ids.length ? ids : knownWorkspaces[0] ? [knownWorkspaces[0].id] : []
}

function resolveCollapsedWorkspaceIds(knownWorkspaces: WorkspaceCard[], restoredIds: string[]) {
  const workspaceIds = new Set(knownWorkspaces.map((workspace) => workspace.id))
  return Array.from(new Set(restoredIds.filter((workspaceId) => workspaceIds.has(workspaceId))))
}

function resolveRestoredTabId(workspace: WorkspaceCard | undefined, preferredId?: string | null) {
  if (!workspace) return ''
  return workspace.tabs.some((tab) => tab.id === preferredId)
    ? preferredId || ''
    : workspace.tabs[0]?.id ?? ''
}

function resolveRestoredPaneId(tab: WorkspaceTab | undefined, preferredId?: string | null) {
  if (!tab) return ''
  const leafPanes = flattenLeafPanes(tab.panes)
  return leafPanes.some((pane) => pane.id === preferredId)
    ? preferredId || ''
    : leafPanes[0]?.id ?? ''
}

function resolveRestoredSessionIds(workspace: WorkspaceCard | undefined, restoredIds?: Record<string, string>) {
  if (!workspace) return {}
  const next: Record<string, string> = {}

  workspace.tabs.forEach((tab) => {
    flattenLeafPanes(tab.panes).forEach((pane) => {
      const restoredSessionId = restoredIds?.[pane.id]
      if (restoredSessionId && paneSessions(pane).some((session) => session.id === restoredSessionId)) {
        next[pane.id] = restoredSessionId
        return
      }

      const fallbackSessionId = paneSessions(pane).find((session) => session.id === pane.activeSessionId)?.id
        ?? paneSessions(pane)[0]?.id
      if (fallbackSessionId) {
        next[pane.id] = fallbackSessionId
      }
    })
  })

  return next
}

function resolveRestoredCollapsedTreeTabIds(workspace: WorkspaceCard | undefined, restoredIds?: string[]) {
  if (!workspace || !Array.isArray(restoredIds)) return []
  const tabIds = new Set(workspace.tabs.map((tab) => tab.id))
  return Array.from(new Set(restoredIds.filter((tabId) => tabIds.has(tabId))))
}

function commitWorkspaces(
  updater: (current: WorkspaceCard[]) => WorkspaceCard[],
  mode: 'persist' | 'transient' = 'persist',
) {
  nextWorkspacePersistenceMode = mode
  workspaces.value = updater(workspaces.value)
}

function captureWorkspaceFocus(workspaceId: string): WorkspaceFocusState {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  const previousFocus = workspaceFocusCache.value[workspaceId] ?? emptyWorkspaceFocusState()
  if (!workspace) return previousFocus

  const activeTabId = workspace.tabs.some((tab) => tab.id === activeRuntimeTabId.value)
    ? activeRuntimeTabId.value
    : workspace.tabs[0]?.id ?? null
  const activeTab = workspace.tabs.find((tab) => tab.id === activeTabId)
  const activePaneId = activeTab && findPaneById(activeTab.panes, activeRuntimePaneId.value)
    ? activeRuntimePaneId.value
    : activeTab ? findFirstLeafPaneId(activeTab.panes) || null : null
  const activePaneIdsByTab = { ...previousFocus.activePaneIdsByTab }

  if (activeTabId && activePaneId) {
    activePaneIdsByTab[activeTabId] = activePaneId
  }

  return normalizeWorkspaceFocus(workspace, {
    activeTabId,
    activePaneId,
    activePaneIdsByTab,
    activeSessionIdsByPane: runtimeActiveSessionIds.value,
    collapsedTreeTabIds: collapsedTreeTabIds.value,
  })
}

function rememberWorkspaceFocus(workspaceId = selectedWorkspaceId.value) {
  if (!workspaceId) return
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace) return
  workspaceFocusCache.value = {
    ...workspaceFocusCache.value,
    [workspaceId]: captureWorkspaceFocus(workspaceId),
  }
}

function applyWorkspaceFocus(workspaceId: string, preferredTabId?: string | null, preferredPaneId?: string | null) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace) {
    activeRuntimeTabId.value = ''
    activeRuntimePaneId.value = ''
    runtimeActiveSessionIds.value = {}
    collapsedTreeTabIds.value = []
    return
  }

  const focus = normalizeWorkspaceFocus(workspace, workspaceFocusCache.value[workspaceId])
  const tabId = resolveRestoredTabId(workspace, preferredTabId ?? focus.activeTabId)
  const tab = workspace.tabs.find((item) => item.id === tabId)
  const paneId = resolveRestoredPaneId(tab, preferredPaneId ?? focus.activePaneIdsByTab[tabId] ?? focus.activePaneId)

  activeRuntimeTabId.value = tabId
  activeRuntimePaneId.value = paneId
  runtimeActiveSessionIds.value = resolveRestoredSessionIds(workspace, focus.activeSessionIdsByPane)
  collapsedTreeTabIds.value = resolveRestoredCollapsedTreeTabIds(workspace, focus.collapsedTreeTabIds)
}

function buildWorkbenchRestoreState(): WorkbenchRestoreState {
  rememberWorkspaceFocus()
  const workspaceIds = new Set(workspaces.value.map((workspace) => workspace.id))

  return {
    version: 1,
    selectedWorkspaceId: selectedWorkspaceId.value || null,
    selectedOverviewWorkspaceId: selectedOverviewWorkspaceId.value || null,
    openedWorkspaceIds: openedWorkspaceIds.value.filter((workspaceId) => workspaceIds.has(workspaceId)),
    collapsedWorkspaceIds: collapsedWorkspaceIds.value.filter((workspaceId) => workspaceIds.has(workspaceId)),
    workspaceFocus: Object.fromEntries(
      Object.entries(workspaceFocusCache.value)
        .filter(([workspaceId]) => workspaceIds.has(workspaceId))
        .map(([workspaceId, focus]) => [workspaceId, normalizeWorkspaceFocus(workspaces.value.find((workspace) => workspace.id === workspaceId), focus)]),
    ),
    updatedAt: new Date().toISOString(),
  }
}

function saveWorkbenchRestoreState() {
  if (typeof window === 'undefined' || !window.localStorage) return
  window.localStorage.setItem(WORKBENCH_RESTORE_STATE_KEY, JSON.stringify(buildWorkbenchRestoreState()))
}

function queueSaveWorkbenchRestoreState() {
  if (saveWorkbenchRestoreStateTimer) {
    clearTimeout(saveWorkbenchRestoreStateTimer)
  }
  saveWorkbenchRestoreStateTimer = setTimeout(() => {
    saveWorkbenchRestoreStateTimer = null
    saveWorkbenchRestoreState()
  }, 120)
}

watch(workspaces, (nextWorkspaces) => {
  const persistenceMode = nextWorkspacePersistenceMode
  nextWorkspacePersistenceMode = 'persist'

  if (persistenceMode === 'persist') {
    if (saveWorkspacesTimer) {
      window.clearTimeout(saveWorkspacesTimer)
    }
    if (saveWorkspacesIdleHandle && 'cancelIdleCallback' in window) {
      ;(window as Window & { cancelIdleCallback: (handle: number) => void }).cancelIdleCallback(saveWorkspacesIdleHandle)
      saveWorkspacesIdleHandle = null
    }
    saveWorkspacesTimer = window.setTimeout(() => {
      saveWorkspacesTimer = null
      const flushSave = () => {
        saveWorkspaces(nextWorkspaces)
        saveWorkspacesIdleHandle = null
      }
      if ('requestIdleCallback' in window) {
        saveWorkspacesIdleHandle = (window as Window & { requestIdleCallback: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number }).requestIdleCallback(() => flushSave(), { timeout: 600 })
        return
      }
      flushSave()
    }, 900)
  }

  if (persistenceMode === 'transient') {
    return
  }

  openedWorkspaceIds.value = openedWorkspaceIds.value.filter((workspaceId) => nextWorkspaces.some((workspace) => workspace.id === workspaceId))
  collapsedWorkspaceIds.value = collapsedWorkspaceIds.value.filter((workspaceId) => nextWorkspaces.some((workspace) => workspace.id === workspaceId))

  if (!nextWorkspaces.some((workspace) => workspace.id === selectedWorkspaceId.value)) {
    selectedWorkspaceId.value = nextWorkspaces[0]?.id ?? ''
  }

  if (!nextWorkspaces.some((workspace) => workspace.id === selectedOverviewWorkspaceId.value)) {
    selectedOverviewWorkspaceId.value = nextWorkspaces[0]?.id ?? ''
  }

  if (!openedWorkspaceIds.value.length && nextWorkspaces[0]) {
    openedWorkspaceIds.value = [nextWorkspaces[0].id]
  }

  workspaceFocusCache.value = Object.fromEntries(
    Object.entries(workspaceFocusCache.value)
      .filter(([workspaceId]) => nextWorkspaces.some((workspace) => workspace.id === workspaceId))
      .map(([workspaceId, focus]) => [workspaceId, normalizeWorkspaceFocus(nextWorkspaces.find((workspace) => workspace.id === workspaceId), focus)]),
  )
}, { flush: 'post' })

watch(userWorkflowTemplates, (templates) => {
  saveUserWorkflowTemplates(templates)
}, { deep: true })

watch([appSearchQuery, groupedSearchResults, openSearchModal], () => {
  if (!activeSearchResultList.value.length) {
    searchResultActiveId.value = null
    return
  }
  if (!searchResultActiveId.value || !activeSearchResultList.value.some((result) => result.id === searchResultActiveId.value)) {
    searchResultActiveId.value = activeSearchResultList.value[0]?.id ?? null
  }
}, { flush: 'post' })

watch(searchResultActiveId, async (resultId) => {
  if (!resultId) return
  await nextTick()
  document.querySelector<HTMLElement>(`[data-search-result-id="${resultId}"]`)?.scrollIntoView({
    block: 'nearest',
    inline: 'nearest',
  })
}, { flush: 'post' })

watch(openRenameModal, async (open) => {
  if (!open) return
  await nextTick()
  renameInputRef.value?.focus()
  renameInputRef.value?.select()
})

watch(selectedWorkspace, (workspace) => {
  if (!workspace) {
    activeRuntimeTabId.value = ''
    activeRuntimePaneId.value = ''
    return
  }

  if (!workspace.tabs.some((tab) => tab.id === activeRuntimeTabId.value)) {
    activeRuntimeTabId.value = workspace.tabs[0]?.id ?? ''
  }

  collapsedTreeTabIds.value = collapsedTreeTabIds.value.filter((tabId) => workspace.tabs.some((tab) => tab.id === tabId))
}, { immediate: true })

function closeFloatingMenus() {
  activePaneMenu.value = null
  activePaneHeaderMenu.value = null
  activePaneBindingMenu.value = null
  activePaneSessionMenu.value = null
  activeCommandPanelPaneId.value = null
  activeWorkspaceMenu.value = null
  activeWorkspaceMenuPosition.value = null
  activeRuntimeTabMenuId.value = null
  activeRuntimeTabMenuPosition.value = null
  activeExplorerProjectMenuId.value = null
  activeExplorerProjectWorkspaceId.value = null
  activeExplorerProjectMenuPosition.value = null
  activePaneMenuPosition.value = null
  activePaneHeaderMenuPosition.value = null
  activePaneBindingMenuPosition.value = null
  activePaneSessionMenuPosition.value = null
  openRecentWorkspaceFilterMenu.value = false
  openLaunchModeMenu.value = false
  activeRailTooltipId.value = null
}

function handleGlobalClick() {
  if (Date.now() < suppressFloatingMenuCloseUntil) return
  closeFloatingMenus()
}

function handleGlobalPointerDown() {
  if (Date.now() < suppressFloatingMenuCloseUntil) return
  closeFloatingMenus()
}

function handleMenuTriggerPointerDown(event: MouseEvent | PointerEvent) {
  event.stopPropagation()
  suppressFloatingMenuCloseUntil = Date.now() + 120
}

function eventTargetsEditable(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null
  if (!target) return false
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true
  if (target.isContentEditable) return true
  return Boolean(target.closest('[contenteditable="true"]'))
}

function handleGlobalKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k' && !event.altKey) {
    event.preventDefault()
    openQuickSearch()
    return
  }

  if (!event.repeat && event.key === 'Shift' && !event.ctrlKey && !event.metaKey && !event.altKey && !eventTargetsEditable(event)) {
    const now = Date.now()
    if (now - lastStandaloneShiftAt <= 380) {
      lastStandaloneShiftAt = 0
      event.preventDefault()
      openQuickSearch()
      return
    }
    lastStandaloneShiftAt = now
  }

  if (event.key === 'Escape') {
    if (splitResizeState.value) {
      splitResizeState.value = null
      cleanupSplitResizeListeners()
      document.body.classList.remove('is-resizing-split-pane')
      return
    }
    if (draggingSession.value) {
      endSessionDrag({ cancelled: true })
      return
    }
    closeFloatingMenus()
    if (openHelpDrawer.value) {
      openHelpDrawer.value = false
    }
    if (openConfirmModal.value) {
      closeConfirmModal()
    }
  }
}

onMounted(() => {
  startupPerf.appMountedAt = performance.now()
  loadUiPreferences()
  loadDiagnosticsCache()
  scheduleSystemRefresh()
  queueInitialDiagnostics()
  startSupervisorScan()
  void refreshTauriViewportWidth()
  window.addEventListener('click', handleGlobalClick)
  window.addEventListener('pointerdown', handleGlobalPointerDown)
  window.addEventListener('keydown', handleGlobalKeydown)
  window.addEventListener('resize', refreshCachedDropZones)
  if ('__TAURI_INTERNALS__' in window) {
    void getCurrentWindow().onResized(() => {
      void refreshTauriViewportWidth()
    }).then((unlisten) => {
      unlistenWindowResize = unlisten
    }).catch(() => undefined)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleGlobalClick)
  window.removeEventListener('pointerdown', handleGlobalPointerDown)
  window.removeEventListener('keydown', handleGlobalKeydown)
  window.removeEventListener('resize', refreshCachedDropZones)
  unlistenWindowResize?.()
  unlistenWindowResize = null
  saveWorkbenchRestoreState()
  if (saveWorkspacesTimer) {
    window.clearTimeout(saveWorkspacesTimer)
    saveWorkspacesTimer = null
  }
  if (saveWorkspacesIdleHandle && 'cancelIdleCallback' in window) {
    ;(window as Window & { cancelIdleCallback: (handle: number) => void }).cancelIdleCallback(saveWorkspacesIdleHandle)
    saveWorkspacesIdleHandle = null
  }
  if (saveWorkbenchRestoreStateTimer) {
    clearTimeout(saveWorkbenchRestoreStateTimer)
    saveWorkbenchRestoreStateTimer = null
  }
  if (searchLoopHintTimer) {
    clearTimeout(searchLoopHintTimer)
    searchLoopHintTimer = null
  }
  if (systemRefreshTimer) {
    window.clearInterval(systemRefreshTimer)
    systemRefreshTimer = null
  }
  if (systemRefreshTickTimer) {
    window.clearInterval(systemRefreshTickTimer)
    systemRefreshTickTimer = null
  }
  if (supervisorScanTimer) {
    window.clearInterval(supervisorScanTimer)
    supervisorScanTimer = null
  }
  dragPointerCleanup?.()
  cleanupSplitResizeListeners()
  workbenchResizeCleanup?.()
})

const activeRuntimeTab = computed(() => {
  if (!selectedWorkspace.value) return undefined
  return selectedWorkspace.value.tabs.find((tab) => tab.id === activeRuntimeTabId.value) ?? selectedWorkspace.value.tabs[0]
})

const workbenchShellStyle = computed(() => ({
  '--workbench-sidebar-width': `${workbenchSidebarWidth.value}px`,
}))

watch(activeRuntimeTab, (tab) => {
  if (!tab) {
    activeRuntimePaneId.value = ''
    return
  }

  if (!findPaneById(tab.panes, activeRuntimePaneId.value)) {
    activeRuntimePaneId.value = findFirstLeafPaneId(tab.panes)
  }
}, { immediate: true })

watch([
  selectedWorkspaceId,
  selectedOverviewWorkspaceId,
  openedWorkspaceIds,
  activeRuntimeTabId,
  activeRuntimePaneId,
  runtimeActiveSessionIds,
  collapsedWorkspaceIds,
  collapsedTreeTabIds,
], () => {
  rememberWorkspaceFocus()
  queueSaveWorkbenchRestoreState()
}, { deep: true, flush: 'post' })

watch(() => appSection.value, (section) => {
  if (section === 'home' && !startupPerf.firstHomePaintAt) {
    startupPerf.firstHomePaintAt = performance.now()
  }
})

watch(() => workspaceView.value, (view) => {
  if (view === 'runtime' && !startupPerf.workbenchEnteredAt) {
    startupPerf.workbenchEnteredAt = performance.now()
  }
})

watch(workspaces, () => {
  queueSessionAttentionNotifications()
  void applyWindowAttentionBadge()
}, { deep: true, flush: 'post' })

watch([activeThemeId, customAccentHex, terminalFontSize, terminalFontFamily], () => {
  queueSaveUiPreferences()
})

watch([systemRefreshInterval, hiddenEnvironmentItems], () => {
  queueSaveUiPreferences()
  scheduleSystemRefresh()
}, { deep: true })

watch([restoreCommandStrategy, railCollapsed, systemNotificationsEnabled, windowAttentionEnabled, pinnedRecentItemIds, hiddenRecentItemIds], () => {
  queueSaveUiPreferences()
}, { deep: true })

const runtimeCanvasClass = computed(() => {
  if (!activeRuntimeTab.value) return ''
  if (activeRuntimeTab.value.layoutMode === 'vertical') return 'runtime-canvas--vertical'
  if (activeRuntimeTab.value.layoutMode === 'horizontal') return 'runtime-canvas--horizontal'
  return 'runtime-canvas--grid'
})

const workbenchRuntimeCanvasStyle = computed(() => {
  const count = Math.max(countLeafPanes(activeRuntimeTab.value?.panes || []), 1)

  let columns = 1
  if (count === 2) columns = 2
  else if (count <= 4) columns = 2
  else if (count <= 9) columns = 3
  else columns = Math.ceil(Math.sqrt(count))

  const rows = Math.max(Math.ceil(count / columns), 1)

  return {
    '--workbench-pane-cols': String(columns),
    '--workbench-pane-rows': String(rows),
    '--workbench-pane-max-width': count <= 1 ? '100%' : 'minmax(0, 1fr)',
  }
})

const appliedTheme = computed(() => {
  const preset = selectedThemePreset.value
  if (preset.id !== 'default') return preset

  return {
    ...preset,
    accent: customAccentHex.value,
    swatches: [preset.background, preset.panel, customAccentHex.value, preset.accentBlue],
  }
})

const activeThemeLabel = computed(() => `${appliedTheme.value.name} · ${appliedTheme.value.kind}`)
const activeThemeDescription = computed(() => appliedTheme.value.description)
const activeThemeSwatches = computed(() => appliedTheme.value.swatches)
const customAccentRgb = computed(() => hexToRgb(customAccentHex.value))
const isDarkTheme = computed(() => appliedTheme.value.scheme === 'dark' || colorLuminance(appliedTheme.value.background) < 0.36)
const appShellClasses = computed(() => ({
  'app-shell--tauri-wide': typeof window !== 'undefined'
    && '__TAURI_INTERNALS__' in window
    && (tauriViewportWidth.value ?? window.innerWidth) >= 1200,
  'app-shell--dark': isDarkTheme.value,
}))

const appThemeVars = computed(() => {
  const theme = appliedTheme.value
  const dark = isDarkTheme.value
  const accentRgb = hexToRgb(theme.accent)
  const accentStrong = lightenColor(theme.accent, dark ? 18 : -12)

  return {
    '--bg-root': theme.background,
    '--bg-1': theme.background2,
    '--bg-2': theme.panel,
    '--bg-3': theme.panelElevated,
    '--panel-glass': rgba(theme.panel, dark ? 0.9 : 0.84),
    '--panel-solid': theme.panel,
    '--panel-elevated-solid': theme.panelElevated,
    '--floating-surface': theme.panelElevated,
    '--floating-surface-strong': dark ? lightenColor(theme.panelElevated, 4) : '#ffffff',
    '--text-primary': theme.textPrimary,
    '--text-secondary': theme.textSecondary,
    '--text-tertiary': rgba(theme.textSecondary, 0.75),
    '--border-subtle': rgba(theme.textPrimary, 0.06),
    '--border-default': rgba(theme.textPrimary, 0.1),
    '--border-strong': `rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.36)`,
    '--surface-hover': rgba(theme.textPrimary, dark ? 0.08 : 0.07),
    '--surface-press': rgba(theme.textPrimary, dark ? 0.12 : 0.13),
    '--surface-overlay': dark ? 'rgba(0, 0, 0, 0.48)' : 'rgba(15, 23, 42, 0.26)',
    '--terminal-canvas-bg': dark ? '#050608' : `color-mix(in srgb, ${theme.background} 88%, #050608 12%)`,
    '--accent-rgb': `${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}`,
    '--accent': theme.accent,
    '--accent-strong': accentStrong,
    '--accent-soft': `rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.14)`,
    '--accent-blue': theme.accentBlue,
    '--success': '#34d27a',
    '--success-soft': 'rgba(52, 210, 122, 0.14)',
    '--danger': '#f07272',
    '--danger-soft': 'rgba(240, 114, 114, 0.14)',
  } as Record<string, string>
})

watch([appThemeVars, isDarkTheme], ([vars, dark]) => {
  if (typeof document === 'undefined') return
  Object.entries(vars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value)
  })
  document.documentElement.dataset.themeScheme = dark ? 'dark' : 'light'
  document.documentElement.style.colorScheme = dark ? 'dark' : 'light'
}, { immediate: true })

const terminalPreviewStyle = computed(() => ({
  fontFamily: `'${terminalFontFamily.value}', 'Geist Mono', 'Cascadia Code', monospace`,
  fontSize: `${terminalFontSize.value}px`,
}))

const breadcrumbItems = computed(() => {
  const items: Array<{ key: string; label: string; icon: string; onClick?: () => void }> = [
    { key: 'home-root', label: '首页', icon: 'home', onClick: goHome },
  ]

  if (appSection.value === 'home') {
    return items
  }

  if (appSection.value !== 'workspace') {
    items.push({ key: appSection.value, label: placeholderTitle.value, icon: appSection.value === 'recent' ? 'recent' : appSection.value === 'templates' ? 'template' : appSection.value === 'search' ? 'search' : 'settings' })
    return items
  }

  items.push({ key: 'workspace-root', label: '工作区', icon: 'workspace', onClick: goWorkspaceOverview })

  if (workspaceView.value === 'overview') {
    return items
  }

  if (selectedWorkspace.value) {
    const currentWorkspaceId = selectedWorkspace.value.id
    items.push({
      key: currentWorkspaceId,
      label: selectedWorkspace.value.name,
      icon: 'folder',
      onClick: () => openWorkspace(currentWorkspaceId),
    })
  }

  return items
})

const placeholderTitle = computed(() => {
  if (appSection.value === 'home') return '首页'
  if (appSection.value === 'recent') return '最近'
  if (appSection.value === 'templates') return '模板'
  if (appSection.value === 'search') return '搜索'
  if (appSection.value === 'settings') return '设置'
  return '占位页面'
})

const placeholderDescription = computed(() => {
  if (appSection.value === 'recent') return '这里会放最近打开的工作区、最近恢复的布局，以及高频命令入口。'
  if (appSection.value === 'templates') return '这里会放预设模板与常用工作区蓝图。'
  if (appSection.value === 'search') return '这里会放工作区、项目、终端、配置、命令和设置入口的本地搜索。'
  if (appSection.value === 'settings') return '这里会放主题、Shell 默认值、字体与布局等配置。'
  return '当前页面仍在持续完善中。'
})

function paneCommandPreview(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) return '未绑定运行配置'
  const command = mode === 'default' ? entry.defaultCommand?.trim() : entry.lastCommand?.trim()
  if (!command) return mode === 'default' ? '未设置默认命令' : '还没有最后命令'
  return command.length > 72 ? `${command.slice(0, 72)}…` : command
}

function paneMenuItems(pane: PaneNode): PopoverItem[] {
  const items: PopoverItem[] = [
    {
      label: '重命名 Pane',
      icon: 'edit',
      description: '修改当前 Pane 分组名称与其组内标签前缀。',
      shortcut: 'F2',
      onClick: () => {
        activePaneMenu.value = null
        openPaneRenameModal(pane.id)
      },
    },
    {
      label: '选择运行配置',
      icon: 'folder',
      description: '将当前 Pane 绑定到一个已有运行配置。',
      shortcut: 'Alt+T',
      onClick: () => {
        activePaneBindingMenuPosition.value = activePaneMenuPosition.value
        activePaneMenu.value = null
        activePaneBindingMenu.value = pane.id
      },
    },
    {
      label: '打开工作目录',
      icon: 'folder',
      description: '打开当前终端对应的工作目录。',
      shortcut: 'Alt+O',
      onClick: () => {
        activePaneMenu.value = null
        openPaneDirectory(pane)
      },
    },
    {
      label: '复制路径',
      icon: 'copy',
      description: '复制当前 Pane 使用的工作目录路径。',
      shortcut: 'Alt+C',
      onClick: () => {
        activePaneMenu.value = null
        copyPanePath(pane)
      },
    },
    {
      label: '复制默认命令',
      icon: 'copy',
      description: `预览：${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneMenu.value = null
        copyPaneCommand(pane, 'default')
      },
    },
    {
      label: '插入默认命令',
      icon: 'terminal',
      description: `预览：${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneMenu.value = null
        insertPaneCommand(pane, 'default')
      },
    },
    {
      label: '复制最后命令',
      icon: 'copy',
      description: `预览：${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneMenu.value = null
        copyPaneCommand(pane, 'last')
      },
    },
    {
      label: '插入最后命令',
      icon: 'terminal',
      description: `预览：${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneMenu.value = null
        insertPaneCommand(pane, 'last')
      },
    },
    {
      label: '拆分到右侧',
      icon: 'pane',
      description: '在当前 Pane 右侧拆出一个新的分组。',
      shortcut: 'Alt+→',
      onClick: () => {
        activePaneMenu.value = null
        splitLeafPane(pane.id, 'horizontal')
      },
    },
    {
      label: '拆分到下方',
      icon: 'pane',
      description: '在当前 Pane 下方拆出一个新的分组。',
      shortcut: 'Alt+↓',
      onClick: () => {
        activePaneMenu.value = null
        splitLeafPane(pane.id, 'vertical')
      },
    },
  ]

  if (pane.terminalEntryId) {
    items.push({
      label: '解除配置绑定',
      icon: 'close',
      description: '将当前 Pane 恢复为独立空白终端。',
      shortcut: 'Alt+U',
      onClick: () => {
        activePaneMenu.value = null
        assignEntryToPane(pane.id, null)
      },
    })
  }

  items.push({
    label: '删除 Pane',
    icon: 'trash',
    description: '删除当前 Pane 分组及其中的所有终端。',
    shortcut: 'Del',
    danger: true,
    onClick: () => {
      activePaneMenu.value = null
      removePane(pane.id)
    },
  })

  return items
}

function paneSessionMenuItems(pane: PaneNode, session: PaneTerminalSession): PopoverItem[] {
  const supervisorEnabled = session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume'
  const items: PopoverItem[] = [
    {
      label: '重命名终端',
      icon: 'edit',
      description: '修改当前终端标签名称。',
      shortcut: 'F2',
      onClick: () => {
        activePaneSessionMenu.value = null
        renameTarget.kind = 'session'
        renameTarget.id = session.id
        renameTarget.title = '重命名终端'
        renameTarget.placeholder = '例如：PowerShell 7 / 测试终端 / 构建命令'
        renameTarget.value = session.name
        openRenameModal.value = true
      },
    },
    {
      label: '选择运行配置',
      icon: 'folder',
      description: '将当前 Pane 绑定到一个已有运行配置。',
      shortcut: 'Alt+T',
      onClick: () => {
        activePaneBindingMenuPosition.value = activePaneSessionMenuPosition.value
        activePaneSessionMenu.value = null
        activePaneBindingMenu.value = pane.id
      },
    },
  ]

  items.push(
    {
      label: supervisorEnabled ? '关闭任务监督' : '开启任务监督',
      icon: supervisorEnabled ? 'close' : 'recent',
      description: supervisorEnabled
        ? '停止对当前终端任务的完成、停滞和待输入状态进行提醒。'
        : '监控当前终端任务，长时间无输出或命令完成时给出提醒。',
      onClick: () => {
        activePaneSessionMenu.value = null
        setSessionSupervisorMode(session.id, supervisorEnabled ? 'off' : 'watch')
      },
    },
    {
      label: '打开工作目录',
      icon: 'folder',
      description: '打开当前终端对应的工作目录。',
      shortcut: 'Alt+O',
      onClick: () => {
        activePaneSessionMenu.value = null
        openPaneDirectory(pane)
      },
    },
    {
      label: '复制路径',
      icon: 'copy',
      description: '复制当前 Pane 使用的工作目录路径。',
      shortcut: 'Alt+C',
      onClick: () => {
        activePaneSessionMenu.value = null
        copyPanePath(pane)
      },
    },
    {
      label: '复制默认命令',
      icon: 'copy',
      description: `预览：${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneSessionMenu.value = null
        copyPaneCommand(pane, 'default')
      },
    },
    {
      label: '插入默认命令',
      icon: 'terminal',
      description: `预览：${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneSessionMenu.value = null
        insertPaneCommand(pane, 'default')
      },
    },
    {
      label: '复制最后命令',
      icon: 'copy',
      description: `预览：${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneSessionMenu.value = null
        copyPaneCommand(pane, 'last')
      },
    },
    {
      label: '插入最后命令',
      icon: 'terminal',
      description: `预览：${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneSessionMenu.value = null
        insertPaneCommand(pane, 'last')
      },
    },
    {
      label: '拆分到右侧',
      icon: 'pane',
      description: '在当前 Pane 右侧拆出一个新的分组。',
      shortcut: 'Alt+→',
      onClick: () => {
        activePaneSessionMenu.value = null
        splitLeafPane(pane.id, 'horizontal')
      },
    },
    {
      label: '拆分到下方',
      icon: 'pane',
      description: '在当前 Pane 下方拆出一个新的分组。',
      shortcut: 'Alt+↓',
      onClick: () => {
        activePaneSessionMenu.value = null
        splitLeafPane(pane.id, 'vertical')
      },
    },
  )

  if (pane.terminalEntryId) {
    items.push({
      label: '解除配置绑定',
      icon: 'close',
      description: '将当前 Pane 恢复为独立空白终端。',
      shortcut: 'Alt+U',
      onClick: () => {
        activePaneSessionMenu.value = null
        assignEntryToPane(pane.id, null)
      },
    })
  }

  items.push({
    label: '关闭终端',
    icon: 'trash',
    description: '仅关闭当前终端标签，不删除整个 Pane 分组。',
    shortcut: 'Del',
    danger: true,
    onClick: () => {
      activePaneSessionMenu.value = null
      removePaneSession(pane.id, session.id)
    },
  })

  return items
}

function paneHeaderMenuItems(pane: PaneNode): PopoverItem[] {
  return [
    {
      label: '新建终端',
      description: '在当前 Pane 分组中新建一个终端标签。',
      onClick: () => {
        activePaneHeaderMenu.value = null
        createPaneSession(pane.id)
      },
    },
  ]
}

function workspaceMenuItems(workspace: WorkspaceCard): PopoverItem[] {
  const items: PopoverItem[] = [
    {
      label: '切换到此工作区',
      description: '在右侧终端画布中打开这个工作区',
      onClick: () => {
        activeWorkspaceMenu.value = null
        switchOpenedWorkspace(workspace.id)
      },
    },
    {
      label: '运行配置',
      description: '管理该工作区下可复用的终端模板',
      onClick: () => {
        activeWorkspaceMenu.value = null
        openWorkspaceTerminalEntries(workspace.id)
      },
    },
    {
      label: '编辑工作区',
      description: '修改名称、路径、描述与标签',
      onClick: () => {
        activeWorkspaceMenu.value = null
        openWorkspaceEditModal(workspace.id)
      },
    },
  ]

  if (openedWorkspaces.value.length > 1) {
    items.push({
      label: '从常用区移除',
      description: '仅移出左侧常用工作区列表，不删除数据',
      onClick: () => {
        activeWorkspaceMenu.value = null
        closeOpenedWorkspace(workspace.id)
      },
    })
  }

  items.push({
    label: '删除工作区',
    description: '删除该工作区及其项目、终端与配置模板',
    danger: true,
    onClick: () => {
      activeWorkspaceMenu.value = null
      removeWorkspace(workspace.id)
    },
  })

  return items
}

function paneBindingItems(pane: PaneNode): PopoverItem[] {
  const currentEntryId = pane.terminalEntryId ?? ''
  const items: PopoverItem[] = [
    {
      label: '未绑定配置',
      description: '保持当前 Pane 为空白独立终端',
      active: currentEntryId === '',
      onClick: () => assignEntryToPane(pane.id, null),
    },
  ]

  selectedWorkspaceEntries.value.forEach((entry) => {
    const commandText = entry.defaultCommand ? `命令：${entry.defaultCommand}` : '未设置默认命令'
    items.push({
      label: entry.name,
      badges: entry.tags,
      description: commandText,
      active: currentEntryId === entry.id,
      onClick: () => assignEntryToPane(pane.id, entry.id),
    })
  })

  return items
}

function isTreeTabCollapsed(tabId: string) {
  return collapsedTreeTabIds.value.includes(tabId)
}

function toggleTreeTab(tabId: string) {
  collapsedTreeTabIds.value = isTreeTabCollapsed(tabId)
    ? collapsedTreeTabIds.value.filter((item) => item !== tabId)
    : [...collapsedTreeTabIds.value, tabId]
}

function collapseAllTreeTabs() {
  if (!selectedWorkspace.value) return
  collapsedTreeTabIds.value = selectedWorkspace.value.tabs.map((tab) => tab.id)
}

function expandAllTreeTabs() {
  collapsedTreeTabIds.value = []
}

function openRuntimeFromPane(tabId: string, paneId: string) {
  activeRuntimeTabId.value = tabId
  activeRuntimePaneId.value = paneId
  workspaceView.value = 'runtime'
}

function openTabRenameModal(tabId: string) {
  if (!selectedWorkspace.value) return
  const tab = selectedWorkspace.value.tabs.find((item) => item.id === tabId)
  if (!tab) return
  closeFloatingMenus()
  renameTarget.kind = 'tab'
  renameTarget.id = tabId
  renameTarget.title = '重命名项目'
  renameTarget.placeholder = '例如：默认标签页 / 直播测试'
  renameTarget.value = tab.name
  openRenameModal.value = true
}

function openPaneRenameModal(paneId: string) {
  if (!activeRuntimeTab.value) return
  const pane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!pane) return
  closeFloatingMenus()
  renameTarget.kind = 'pane'
  renameTarget.id = paneId
  renameTarget.title = '重命名 Pane'
  renameTarget.placeholder = '例如：直播 / 终端测试 / PowerShell 7'
  renameTarget.value = pane.name
  openRenameModal.value = true
}

function openExplorerTabRename(workspaceId: string, tabId: string) {
  closeFloatingMenus()
  if (selectedWorkspace.value?.id !== workspaceId) {
    openWorkspace(workspaceId)
  }
  nextTick(() => openTabRenameModal(tabId))
}

function nextPaneName(tab: WorkspaceTab) {
  return 'PowerShell 7'
}

function nextSessionName(pane: PaneNode, sessions: PaneTerminalSession[] = paneSessions(pane)) {
  return 'PowerShell 7'
}

function cleanupSplitResizeListeners() {
  splitResizeCleanup?.()
  splitResizeCleanup = null
}

function updateSplitPaneChildRatios(splitPaneId: string, leftChildId: string, rightChildId: string, leftRatio: number, rightRatio: number) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== splitPaneId || !pane.children?.length) return pane
              return {
                ...pane,
                children: pane.children.map((child) => {
                  if (child.id === leftChildId) return { ...child, sizeRatio: leftRatio }
                  if (child.id === rightChildId) return { ...child, sizeRatio: rightRatio }
                  return child
                }),
              }
            }),
          }
        : tab,
    ),
  }))
}

function startSplitResize(splitPaneId: string, direction: SplitDirection, leftChildId: string, rightChildId: string, event: PointerEvent) {
  if (!activeRuntimeTab.value) return

  const splitPane = findPaneById(activeRuntimeTab.value.panes, splitPaneId)
  if (!splitPane?.children?.length) return
  const leftChild = splitPane.children.find((child) => child.id === leftChildId)
  const rightChild = splitPane.children.find((child) => child.id === rightChildId)
  if (!leftChild || !rightChild) return

  const container = (event.currentTarget as HTMLElement | null)?.parentElement
  if (!container) return

  const rect = container.getBoundingClientRect()
  const containerWidth = rect.width
  const containerHeight = rect.height
  const totalRatio = splitPane.children.reduce((sum, child) => sum + (child.sizeRatio || 1), 0) || 1

  splitResizeState.value = {
    splitPaneId,
    direction,
    leftChildId,
    rightChildId,
    startX: event.clientX,
    startY: event.clientY,
    containerWidth,
    containerHeight,
    leftRatio: leftChild.sizeRatio || 1,
    rightRatio: rightChild.sizeRatio || 1,
  }

  const handleMove = (moveEvent: PointerEvent) => {
    const state = splitResizeState.value
    if (!state) return

    const dimension = state.direction === 'horizontal' ? state.containerWidth : state.containerHeight
    if (dimension <= 0) return

    const delta = state.direction === 'horizontal'
      ? moveEvent.clientX - state.startX
      : moveEvent.clientY - state.startY

    const totalPairRatio = state.leftRatio + state.rightRatio
    const totalChildrenRatio = totalRatio
    const deltaRatio = (delta / dimension) * totalChildrenRatio
    const minRatio = Math.max((120 / dimension) * totalChildrenRatio, 0.12)

    let nextLeftRatio = state.leftRatio + deltaRatio
    let nextRightRatio = state.rightRatio - deltaRatio

    if (nextLeftRatio < minRatio) {
      nextLeftRatio = minRatio
      nextRightRatio = totalPairRatio - minRatio
    }
    if (nextRightRatio < minRatio) {
      nextRightRatio = minRatio
      nextLeftRatio = totalPairRatio - minRatio
    }

    updateSplitPaneChildRatios(state.splitPaneId, state.leftChildId, state.rightChildId, nextLeftRatio, nextRightRatio)
  }

  const handleUp = () => {
    splitResizeState.value = null
    cleanupSplitResizeListeners()
    document.body.classList.remove('is-resizing-split-pane')
  }

  cleanupSplitResizeListeners()
  document.body.classList.add('is-resizing-split-pane')
  window.addEventListener('pointermove', handleMove)
  window.addEventListener('pointerup', handleUp)
  splitResizeCleanup = () => {
    window.removeEventListener('pointermove', handleMove)
    window.removeEventListener('pointerup', handleUp)
  }

  event.preventDefault()
  event.stopPropagation()
}

function findSessionLocation(workspace: WorkspaceCard, entryId: string) {
  for (const tab of workspace.tabs) {
    for (const pane of flattenLeafPanes(tab.panes)) {
      const session = paneSessions(pane).find((item) => item.terminalEntryId === entryId)
      if (session) {
        return { tabId: tab.id, paneId: pane.id, sessionId: session.id }
      }
    }
  }
  return null
}

function isSameSessionLocation(
  existing: { tabId: string; paneId: string; sessionId: string } | null,
  tabId: string,
  paneId: string,
  sessionId: string | null,
) {
  if (!existing || !sessionId) return false
  return existing.tabId === tabId && existing.paneId === paneId && existing.sessionId === sessionId
}

function edgeFromPoint(clientX: number, clientY: number, rect: DOMRect, threshold: number): DropEdge | null {
  const distances = [
    { edge: 'left' as const, distance: Math.abs(clientX - rect.left) },
    { edge: 'right' as const, distance: Math.abs(rect.right - clientX) },
    { edge: 'top' as const, distance: Math.abs(clientY - rect.top) },
    { edge: 'bottom' as const, distance: Math.abs(rect.bottom - clientY) },
  ].filter((item) => item.distance <= threshold)

  if (!distances.length) return null
  distances.sort((left, right) => left.distance - right.distance)
  return distances[0]?.edge ?? null
}

function buildGlobalSplitPreview(edge: DropEdge, rect: DOMRect): DragPreviewRect {
  const inset = 8
  if (edge === 'left' || edge === 'right') {
    const width = Math.max(160, Math.round(rect.width * 0.3))
    return {
      left: edge === 'left' ? rect.left + inset : rect.right - width - inset,
      top: rect.top + inset,
      width: width - inset,
      height: rect.height - inset * 2,
    }
  }

  const height = Math.max(120, Math.round(rect.height * 0.3))
  return {
    left: rect.left + inset,
    top: edge === 'top' ? rect.top + inset : rect.bottom - height - inset,
    width: rect.width - inset * 2,
    height: height - inset,
  }
}

function buildPaneSplitPreview(edge: DropEdge, rect: DOMRect): DragPreviewRect {
  const inset = 8
  if (edge === 'left' || edge === 'right') {
    const width = Math.min(88, Math.max(64, Math.round(rect.width * 0.28)))
    return {
      left: edge === 'left' ? rect.left + inset : rect.right - width - inset,
      top: rect.top + inset,
      width,
      height: Math.max(48, rect.height - inset * 2),
    }
  }

  const height = Math.min(72, Math.max(52, Math.round(rect.height * 0.28)))
  return {
    left: rect.left + inset,
    top: edge === 'top' ? rect.top + inset : rect.bottom - height - inset,
    width: Math.max(72, rect.width - inset * 2),
    height,
  }
}

function findTabBarInsertIndex(tabBarElement: HTMLElement, clientX: number) {
  const tabs = Array.from(tabBarElement.querySelectorAll<HTMLElement>('.terminal-window-tab[data-session-id]'))
  if (!tabs.length) return 0

  for (let index = 0; index < tabs.length; index += 1) {
    const rect = tabs[index].getBoundingClientRect()
    if (clientX < rect.left + rect.width / 2) {
      return index
    }
  }

  return tabs.length
}

function refreshCachedDropZones() {
  cachedDropZones = []

  document.querySelectorAll<HTMLElement>('[data-tabbar-pane-id]').forEach((element) => {
    const paneId = element.dataset.tabbarPaneId
    if (!paneId) return
    cachedDropZones.push({
      type: 'tabbar',
      paneId,
      element,
      rect: element.getBoundingClientRect(),
    })
  })

  document.querySelectorAll<HTMLElement>('.pane[data-pane-id]').forEach((element) => {
    const paneId = element.dataset.paneId
    if (!paneId) return
    cachedDropZones.push({
      type: 'pane',
      paneId,
      element,
      rect: element.getBoundingClientRect(),
    })
  })
}

function findCachedZoneAtPoint(type: CachedDropZone['type'], clientX: number, clientY: number) {
  const matches = cachedDropZones.filter((zone) => zone.type === type
    && clientX >= zone.rect.left
    && clientX <= zone.rect.right
    && clientY >= zone.rect.top
    && clientY <= zone.rect.bottom)

  if (!matches.length) return undefined

  return matches.sort((left, right) => {
    const leftArea = left.rect.width * left.rect.height
    const rightArea = right.rect.width * right.rect.height
    return leftArea - rightArea
  })[0]
}

function buildTabBarInsertMarker(tabBarElement: HTMLElement, index: number): DragInsertMarker {
  const tabs = Array.from(tabBarElement.querySelectorAll<HTMLElement>('.terminal-window-tab[data-session-id]'))
  const barRect = tabBarElement.getBoundingClientRect()
  let left = barRect.left + 10

  if (tabs.length) {
    if (index <= 0) {
      left = tabs[0].getBoundingClientRect().left
    } else if (index >= tabs.length) {
      left = tabs[tabs.length - 1].getBoundingClientRect().right
    } else {
      left = tabs[index].getBoundingClientRect().left
    }
  }

  return {
    left: left - 2,
    top: barRect.top + 6,
    height: Math.max(18, barRect.height - 12),
  }
}

function normalizePaneList(panes: PaneNode[], parentPaneId: string | null = null): PaneNode[] {
  return panes.map((pane, index) => {
    const children = pane.children?.length ? normalizePaneList(pane.children, pane.id) : []
    const sessions = children.length ? [] : (pane.sessions ?? [])
    return {
      ...pane,
      parentPaneId,
      order: index,
      sizeRatio: pane.sizeRatio || 1,
      activeSessionId: children.length
        ? null
        : (pane.activeSessionId && sessions.some((session) => session.id === pane.activeSessionId)
          ? pane.activeSessionId
          : sessions[0]?.id ?? null),
      sessions,
      children,
    }
  })
}

function removeSessionFromPaneTree(panes: PaneNode[], paneId: string, sessionId: string): PaneNode[] {
  const sourcePane = findPaneById(panes, paneId)
  if (!sourcePane) return panes

  const sessions = paneSessions(sourcePane)
  if (sessions.length <= 1) {
    return collapsePaneBranch(paneId, panes)
  }

  return normalizePaneList(visitPaneTree(panes, (pane) => {
    if (pane.id !== paneId) return pane
    const remainingSessions = paneSessions(pane).filter((session) => session.id !== sessionId)
    return {
      ...pane,
      sessions: remainingSessions,
      activeSessionId: remainingSessions[0]?.id ?? null,
    }
  }))
}

function insertSessionIntoPaneTree(panes: PaneNode[], targetPaneId: string, session: PaneTerminalSession, targetIndex?: number) {
  return normalizePaneList(visitPaneTree(panes, (pane) => {
    if (pane.id !== targetPaneId) return pane
    const sessions = [...paneSessions(pane)]
    const insertAt = typeof targetIndex === 'number'
      ? Math.max(0, Math.min(targetIndex, sessions.length))
      : sessions.length
    sessions.splice(insertAt, 0, { ...session })
    return {
      ...pane,
      sessions,
      activeSessionId: session.id,
    }
  }))
}

function createPaneFromSession(sourcePane: PaneNode, session: PaneTerminalSession): PaneNode {
  const paneId = createId('pane')
  return {
    id: paneId,
    tabId: activeRuntimeTab.value?.id || sourcePane.tabId,
    name: session.name || sourcePane.name || 'PowerShell 7',
    pathLabel: session.pathLabel || sourcePane.pathLabel,
    terminalEntryId: session.terminalEntryId ?? sourcePane.terminalEntryId,
    splitDirection: 'none',
    parentPaneId: null,
    order: 0,
    sizeRatio: 1,
    activeSessionId: session.id,
    sessions: [{ ...session }],
    children: [],
  }
}

function insertSiblingPaneAtTarget(
  panes: PaneNode[],
  targetPaneId: string,
  newPane: PaneNode,
  direction: SplitDirection,
  position: 'before' | 'after',
): PaneNode[] {
  const walk = (nodes: PaneNode[], parent: PaneNode | null): PaneNode[] => {
    const result: PaneNode[] = []

    nodes.forEach((node) => {
      if (node.id === targetPaneId) {
        const splitId = createId('split')
        const currentNode = {
          ...node,
          parentPaneId: splitId,
          order: position === 'before' ? 1 : 0,
          sizeRatio: 1,
        }
        const insertedNode = {
          ...newPane,
          parentPaneId: splitId,
          order: position === 'before' ? 0 : 1,
          sizeRatio: 1,
        }
        result.push({
          id: splitId,
          tabId: node.tabId,
          name: node.name,
          pathLabel: node.pathLabel,
          terminalEntryId: null,
          splitDirection: direction,
          parentPaneId: parent?.id ?? null,
          order: node.order,
          sizeRatio: node.sizeRatio || 1,
          activeSessionId: null,
          sessions: [],
          children: position === 'before' ? [insertedNode, currentNode] : [currentNode, insertedNode],
        })
        return
      }

      if (node.children?.length) {
        result.push({
          ...node,
          children: walk(node.children, node),
        })
        return
      }

      result.push(node)
    })

    return normalizePaneList(result, parent?.id ?? null)
  }

  return normalizePaneList(walk(panes, null), null)
}

function insertPaneAtGlobalEdge(
  panes: PaneNode[],
  newPane: PaneNode,
  direction: SplitDirection,
  position: 'before' | 'after',
): PaneNode[] {
  if (!panes.length) {
    return normalizePaneList([{ ...newPane }], null)
  }

  if (panes.length === 1 && isSplitPane(panes[0]) && panes[0].splitDirection === direction) {
    const root = panes[0]
    const nextChildren = position === 'before'
      ? [{ ...newPane }, ...paneChildren(root)]
      : [...paneChildren(root), { ...newPane }]

    return normalizePaneList([
      {
        ...root,
        children: normalizePaneList(nextChildren, root.id),
      },
    ], null)
  }

  const rootId = createId('split')
  const compositeId = createId('split')
  const normalizedExisting = normalizePaneList(panes, null)
  const existingNode = normalizedExisting.length === 1
    ? {
        ...normalizedExisting[0],
        parentPaneId: rootId,
        order: position === 'before' ? 1 : 0,
        sizeRatio: 1,
      }
    : {
        id: compositeId,
        tabId: activeRuntimeTab.value?.id || normalizedExisting[0]?.tabId || newPane.tabId,
        name: 'Workspace Root',
        pathLabel: selectedWorkspace.value?.rootPath || newPane.pathLabel,
        terminalEntryId: null,
        splitDirection: direction,
        parentPaneId: rootId,
        order: position === 'before' ? 1 : 0,
        sizeRatio: 1,
        activeSessionId: null,
        sessions: [],
        children: normalizePaneList(normalizedExisting, compositeId),
      }

  return normalizePaneList([
    {
      id: rootId,
      tabId: activeRuntimeTab.value?.id || newPane.tabId,
      name: 'Workspace Root',
      pathLabel: selectedWorkspace.value?.rootPath || newPane.pathLabel,
      terminalEntryId: null,
      splitDirection: direction,
      parentPaneId: null,
      order: 0,
      sizeRatio: 1,
      activeSessionId: null,
      sessions: [],
      children: position === 'before'
        ? [{ ...newPane, parentPaneId: rootId }, existingNode]
        : [existingNode, { ...newPane, parentPaneId: rootId }],
    },
  ], null)
}

function reorderSessionsInPane(paneId: string, sessionId: string, targetIndex: number) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const targetPane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!targetPane) return

  const sessions = [...paneSessions(targetPane)]
  const currentIndex = sessions.findIndex((session) => session.id === sessionId)
  if (currentIndex === -1) return

  const boundedIndex = Math.max(0, Math.min(targetIndex, sessions.length))
  const adjustedIndex = boundedIndex > currentIndex ? boundedIndex - 1 : boundedIndex
  if (currentIndex === adjustedIndex) return

  const [movedSession] = sessions.splice(currentIndex, 1)
  sessions.splice(adjustedIndex, 0, movedSession)
  const now = new Date().toISOString()

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: normalizePaneTree(visitPaneTree(tab.panes, (pane) =>
              pane.id === paneId
                ? {
                    ...pane,
                    sessions,
                    activeSessionId: movedSession.id,
                  }
                : pane,
            )),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = paneId
}

function moveSessionBetweenPanes(sourcePaneId: string, sourceSessionId: string, targetPaneId: string, targetIndex?: number) {
  if (!activeRuntimeTab.value || !selectedWorkspace.value) return
  if (sourcePaneId === targetPaneId) {
    reorderSessionsInPane(sourcePaneId, sourceSessionId, targetIndex ?? Number.MAX_SAFE_INTEGER)
    return
  }

  const sourcePane = findPaneById(activeRuntimeTab.value.panes, sourcePaneId)
  const targetPane = findPaneById(activeRuntimeTab.value.panes, targetPaneId)
  if (!sourcePane || !targetPane) return

  const movedSession = paneSessions(sourcePane).find((session) => session.id === sourceSessionId)
  if (!movedSession) return

  const now = new Date().toISOString()

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) => {
      if (tab.id !== activeRuntimeTab.value?.id) return tab

      let nextPanes = removeSessionFromPaneTree(tab.panes, sourcePaneId, sourceSessionId)
      nextPanes = insertSessionIntoPaneTree(nextPanes, targetPaneId, movedSession, targetIndex)

      activeRuntimePaneId.value = targetPaneId

      return {
        ...tab,
        panes: normalizePaneTree(nextPanes),
        updatedAt: now,
      }
    }),
    updatedAt: now,
  }))
}

function moveSessionToPaneSplit(sourcePaneId: string, sourceSessionId: string, targetPaneId: string, edge: DropEdge) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const sourcePane = findPaneById(activeRuntimeTab.value.panes, sourcePaneId)
  const targetPane = findPaneById(activeRuntimeTab.value.panes, targetPaneId)
  if (!sourcePane || !targetPane) return
  if (sourcePaneId === targetPaneId && paneSessions(sourcePane).length <= 1) return

  const movedSession = paneSessions(sourcePane).find((session) => session.id === sourceSessionId)
  if (!movedSession) return

  const direction = edge === 'left' || edge === 'right' ? 'horizontal' : 'vertical'
  const position = edge === 'left' || edge === 'top' ? 'before' : 'after'
  const now = new Date().toISOString()

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) => {
      if (tab.id !== activeRuntimeTab.value?.id) return tab

      let nextPanes = removeSessionFromPaneTree(tab.panes, sourcePaneId, sourceSessionId)
      const freshTargetPane = findPaneById(nextPanes, targetPaneId)
      if (!freshTargetPane) return { ...tab, panes: normalizePaneTree(nextPanes), updatedAt: now }

      const nextPane = createPaneFromSession(sourcePane, movedSession)
      nextPanes = insertSiblingPaneAtTarget(nextPanes, targetPaneId, nextPane, direction, position)
      activeRuntimePaneId.value = nextPane.id

      return {
        ...tab,
        panes: normalizePaneTree(nextPanes),
        updatedAt: now,
      }
    }),
    updatedAt: now,
  }))
}

function moveSessionToGlobalSplit(sourcePaneId: string, sourceSessionId: string, edge: DropEdge) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const sourcePane = findPaneById(activeRuntimeTab.value.panes, sourcePaneId)
  if (!sourcePane) return

  const movedSession = paneSessions(sourcePane).find((session) => session.id === sourceSessionId)
  if (!movedSession) return

  const direction = edge === 'left' || edge === 'right' ? 'horizontal' : 'vertical'
  const position = edge === 'left' || edge === 'top' ? 'before' : 'after'
  const now = new Date().toISOString()

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) => {
      if (tab.id !== activeRuntimeTab.value?.id) return tab

      let nextPanes = removeSessionFromPaneTree(tab.panes, sourcePaneId, sourceSessionId)
      const nextPane = createPaneFromSession(sourcePane, movedSession)
      nextPanes = insertPaneAtGlobalEdge(nextPanes, nextPane, direction, position)
      activeRuntimePaneId.value = nextPane.id

      return {
        ...tab,
        panes: normalizePaneTree(nextPanes),
        updatedAt: now,
      }
    }),
    updatedAt: now,
  }))
}

function clearDragDropPreview() {
  dragDropTarget.value = null
}

function cleanupDragPointerListeners() {
  dragPointerCleanup?.()
  dragPointerCleanup = null
  if (dragRafId) {
    cancelAnimationFrame(dragRafId)
    dragRafId = null
  }
}

function attachDragPointerListeners() {
  cleanupDragPointerListeners()

  const handleMove = (event: PointerEvent) => handleGlobalPointerMove(event)
  const handleUp = (event: PointerEvent) => handleGlobalPointerUp(event)
  const handleCancel = () => handleGlobalPointerCancel()

  window.addEventListener('pointermove', handleMove, { passive: false })
  window.addEventListener('pointerup', handleUp, { passive: false })
  window.addEventListener('pointercancel', handleCancel, { passive: true })

  dragPointerCleanup = () => {
    window.removeEventListener('pointermove', handleMove)
    window.removeEventListener('pointerup', handleUp)
    window.removeEventListener('pointercancel', handleCancel)
  }
}

function beginSessionDrag(paneId: string, sessionId: string, event: PointerEvent | MouseEvent) {
  if ('button' in event && event.button !== 0) return

  const sourcePane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, paneId) : undefined
  const sourceSession = sourcePane ? paneSessions(sourcePane).find((session) => session.id === sessionId) : undefined
  if (!sourcePane || !sourceSession) return

  pendingTabPress.value = {
    paneId,
    sessionId,
    pointerId: 'pointerId' in event ? Number((event as PointerEvent).pointerId) : -1,
    startX: event.clientX,
    startY: event.clientY,
  }

  attachDragPointerListeners()
}

function activatePendingTabPress() {
  const pending = pendingTabPress.value
  if (!pending) return

  const sourcePane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, pending.paneId) : undefined
  const sourceSession = sourcePane ? paneSessions(sourcePane).find((session) => session.id === pending.sessionId) : undefined
  const targetElement = document.elementFromPoint(pending.startX, pending.startY) as HTMLElement | null
  const tabElement = targetElement?.closest?.('[data-session-id]') as HTMLElement | null
  const tabRect = tabElement?.getBoundingClientRect()
  if (!sourcePane || !sourceSession) {
    pendingTabPress.value = null
    return
  }

  draggingSession.value = {
    sourcePaneId: pending.paneId,
    sourceSessionId: pending.sessionId,
    sessionName: sourceSession.name || 'PowerShell 7',
    pointerId: pending.pointerId,
    active: true,
    moved: false,
    startX: pending.startX,
    startY: pending.startY,
    currentX: pending.startX,
    currentY: pending.startY,
    offsetX: 14,
    offsetY: 10,
    width: tabRect?.width || 160,
    height: tabRect?.height || 34,
    tabBarHoverKey: null,
    tabBarHoverSince: 0,
  }

  pendingTabPress.value = null
  refreshCachedDropZones()
  attachDragPointerListeners()
}

function endSessionDrag(options: { cancelled?: boolean } = {}) {
  const wasMoved = draggingSession.value?.moved
  draggingSession.value = null
  pendingTabPress.value = null
  clearDragDropPreview()
  cleanupDragPointerListeners()
  document.body.classList.remove('is-dragging-terminal-tab')

  window.setTimeout(() => {
    clearDragDropPreview()
  }, 0)

  cachedDropZones = []

  if (!options.cancelled && wasMoved) {
    suppressSessionClickUntil.value = Date.now() + 220
  }
}

function resolveTabBarDropTarget(element: HTMLElement | null, clientX: number, clientY: number): Extract<DragDropTarget, { kind: 'tabbar' }> | null {
  const drag = draggingSession.value
  if (!drag) return null

  const tabBarElement = (findCachedZoneAtPoint('tabbar', clientX, clientY)?.element
    || element?.closest?.('[data-tabbar-pane-id]')) as HTMLElement | null
  if (!tabBarElement) {
    if (drag.tabBarHoverKey || drag.tabBarHoverSince) {
      draggingSession.value = { ...drag, tabBarHoverKey: null, tabBarHoverSince: 0 }
    }
    return null
  }

  const paneId = tabBarElement.dataset.tabbarPaneId
  if (!paneId) return null

  const sourcePane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, drag.sourcePaneId) : undefined
  if (!sourcePane) return null

  const index = findTabBarInsertIndex(tabBarElement, clientX)
  const hoverKey = `${paneId}:${index}`
  const now = performance.now()

  if (drag.tabBarHoverKey !== hoverKey) {
    draggingSession.value = { ...drag, tabBarHoverKey: hoverKey, tabBarHoverSince: now }
    return null
  }

  if (now - drag.tabBarHoverSince < 220) {
    return null
  }

  if (paneId === drag.sourcePaneId && paneSessions(sourcePane).length <= 1) {
    return null
  }

  return {
    kind: 'tabbar',
    paneId,
    index,
    marker: buildTabBarInsertMarker(tabBarElement, index),
  }
}

function detectDragDropTarget(event: PointerEvent): DragDropTarget {
  const drag = draggingSession.value
  if (!drag) return null

  const element = document.elementFromPoint(event.clientX, event.clientY) as HTMLElement | null

  const tabBarTarget = resolveTabBarDropTarget(element, event.clientX, event.clientY)
  if (tabBarTarget) {
    return tabBarTarget
  }

  if (drag.tabBarHoverKey || drag.tabBarHoverSince) {
    draggingSession.value = { ...drag, tabBarHoverKey: null, tabBarHoverSince: 0 }
  }

  const cachedPaneZone = findCachedZoneAtPoint('pane', event.clientX, event.clientY)
  const paneElement = (cachedPaneZone?.element || element?.closest?.('.pane')) as HTMLElement | null
  const paneId = cachedPaneZone?.paneId || paneElement?.dataset?.paneId
  if (!paneElement || !paneId) {
    return null
  }

  const edge = edgeFromPoint(event.clientX, event.clientY, paneElement.getBoundingClientRect(), 30)
  if (!edge) return null

  const paneRect = paneElement.getBoundingClientRect()

  if (paneId === drag.sourcePaneId) {
    const sourcePane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, drag.sourcePaneId) : undefined
    if (!sourcePane || paneSessions(sourcePane).length <= 1) {
      return null
    }
  }

  const paneTarget = {
    kind: 'pane-split' as const,
    paneId,
    edge,
    rect: buildPaneSplitPreview(edge, paneRect),
  }

  const runtimeRect = runtimeCanvasRef.value?.getBoundingClientRect()
  if (runtimeRect) {
    const withinRuntime = event.clientX >= runtimeRect.left && event.clientX <= runtimeRect.right && event.clientY >= runtimeRect.top && event.clientY <= runtimeRect.bottom
    if (withinRuntime) {
      const globalEdge = edgeFromPoint(event.clientX, event.clientY, runtimeRect, 50)
      if (globalEdge) {
        return paneTarget
      }
    }
  }

  return paneTarget
}

function syncDragDropPreview(event: PointerEvent) {
  const nextTarget = detectDragDropTarget(event)
  dragDropTarget.value = nextTarget
}

function applySessionDragDrop() {
  const drag = draggingSession.value
  const target = dragDropTarget.value
  if (!drag || !target) return

  if (target.kind === 'tabbar') {
    moveSessionBetweenPanes(drag.sourcePaneId, drag.sourceSessionId, target.paneId, target.index)
    return
  }

  if (target.kind === 'pane-split') {
    moveSessionToPaneSplit(drag.sourcePaneId, drag.sourceSessionId, target.paneId, target.edge)
    return
  }

  if (target.kind === 'global-split') {
    moveSessionToGlobalSplit(drag.sourcePaneId, drag.sourceSessionId, target.edge)
  }
}

function handleGlobalPointerMove(event: PointerEvent) {
  if (!draggingSession.value && pendingTabPress.value) {
    const pending = pendingTabPress.value
    if (Math.hypot(event.clientX - pending.startX, event.clientY - pending.startY) >= 5) {
      activatePendingTabPress()
    } else {
      return
    }
  }

  const drag = draggingSession.value
  if (!drag) return

  if (!cachedDropZones.length) {
    refreshCachedDropZones()
  }

  const moved = drag.moved || Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) >= 5

  if (dragRafId) {
    cancelAnimationFrame(dragRafId)
  }

  dragRafId = requestAnimationFrame(() => {
    draggingSession.value = {
      ...drag,
      moved,
      currentX: event.clientX,
      currentY: event.clientY,
    }

    if (!moved) return

    document.body.classList.add('is-dragging-terminal-tab')
    syncDragDropPreview(event)
  })

  if (!moved) return

  event.preventDefault()
}

function handleGlobalPointerUp(event: PointerEvent) {
  if (!draggingSession.value && pendingTabPress.value) {
    pendingTabPress.value = null
    return
  }

  if (!draggingSession.value) return

  if (draggingSession.value.moved) {
    event.preventDefault()
    applySessionDragDrop()
    event.stopPropagation()
  }

  endSessionDrag()
}

function handleGlobalPointerCancel() {
  if (!draggingSession.value) return
  endSessionDrag({ cancelled: true })
}

function renameTab(tabId: string, nextName: string) {
  if (!selectedWorkspace.value) return
  const now = new Date().toISOString()
  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === tabId
        ? { ...tab, name: nextName, updatedAt: now }
        : tab,
    ),
    updatedAt: now,
  }))
}


function renamePane(paneId: string, nextName: string) {
  if (!activeRuntimeTab.value) return
  const now = new Date().toISOString()
  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== paneId) return pane
              return {
                ...pane,
                name: nextName,
              }
            }),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))
}

function closeRenameModal() {
  openRenameModal.value = false
}

function submitRenameModal() {
  const value = renameTarget.value.trim()
  if (!value) return

  if (renameTarget.kind === 'tab') {
    renameTab(renameTarget.id, value)
    showToast('项目已重命名', `已更新为：${value}`)
  } else if (renameTarget.kind === 'session') {
    renameSession(renameTarget.id, value)
    showToast('终端已重命名', `已更新为：${value}`)
  } else {
    renamePane(renameTarget.id, value)
    showToast('Pane 已重命名', `已更新为：${value}`)
  }

  closeRenameModal()
}

function openProjectWorkspace(workspaceId: string, tabId: string) {
  switchOpenedWorkspace(workspaceId)

  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  const tab = workspace?.tabs.find((item) => item.id === tabId)
  const focus = workspaceFocusCache.value[workspaceId]

  activeRuntimeTabId.value = tabId
  activeRuntimePaneId.value = resolveRestoredPaneId(tab, focus?.activePaneIdsByTab?.[tabId])
  workspaceView.value = 'runtime'
}

function isWorkspaceCollapsed(workspaceId: string) {
  return collapsedWorkspaceIds.value.includes(workspaceId)
}

function handleWorkspaceSummaryClick(workspaceId: string) {
  if (selectedWorkspaceId.value !== workspaceId) {
    switchOpenedWorkspace(workspaceId)
    collapsedWorkspaceIds.value = collapsedWorkspaceIds.value.filter((item) => item !== workspaceId)
    return
  }

  collapsedWorkspaceIds.value = isWorkspaceCollapsed(workspaceId)
    ? collapsedWorkspaceIds.value.filter((item) => item !== workspaceId)
    : [...collapsedWorkspaceIds.value, workspaceId]
}

function openWorkspacePane(workspaceId: string, tabId: string, paneId: string) {
  switchOpenedWorkspace(workspaceId)
  openRuntimeFromPane(tabId, paneId)
}

function openWorkspaceTerminalSession(workspaceId: string, tabId: string, paneId: string, sessionId: string) {
  switchOpenedWorkspace(workspaceId)
  openRuntimeFromPane(tabId, paneId)
  activatePaneSession(paneId, sessionId)
}

function setActiveTabLayout(layoutMode: 'grid' | 'horizontal' | 'vertical') {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const now = new Date().toISOString()
  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            layoutMode,
            panes: visitPaneTree(tab.panes, (pane) => ({
              ...pane,
              splitDirection: layoutMode === 'horizontal' ? 'horizontal' : layoutMode === 'vertical' ? 'vertical' : pane.splitDirection,
            })),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  showToast('布局已切换', `当前标签页已切换为${layoutMode === 'grid' ? '网格' : layoutMode === 'horizontal' ? '横向' : '纵向'}布局`)
}

const launchModeItems = computed<PopoverItem[]>(() => {
  return launchModeOptions.map((option) => ({
    label: option.label,
    description: option.description,
    active: terminalEntryForm.launchMode === option.value,
    onClick: () => {
      terminalEntryForm.launchMode = option.value
      openLaunchModeMenu.value = false
    },
  }))
})

function toggleLaunchModeMenu() {
  const shouldOpen = !openLaunchModeMenu.value
  closeFloatingMenus()
  openLaunchModeMenu.value = shouldOpen
}

function toggleRailCollapsed() {
  railCollapsed.value = !railCollapsed.value
}

function openHelpTopic(topicId: string) {
  activeHelpTopicId.value = topicId
  openHelpDrawer.value = true
}

function goHome() {
  appSection.value = 'home'
  workspaceView.value = 'overview'
}

function goWorkspaceOverview() {
  appSection.value = 'workspace'
  workspaceView.value = 'overview'
}

function recentItemTimestamp(item: RecentItem) {
  const time = new Date(item.timestamp).getTime()
  return Number.isFinite(time) ? time : 0
}

function normalizeSearchText(value: string) {
  return value.trim().toLocaleLowerCase()
}

function highlightedText(text: string, query: string) {
  const normalizedQuery = query.trim()
  if (!normalizedQuery) return [{ text, match: false }]

  const lowerText = text.toLocaleLowerCase()
  const lowerQuery = normalizedQuery.toLocaleLowerCase()
  const index = lowerText.indexOf(lowerQuery)
  if (index < 0) return [{ text, match: false }]

  const result: Array<{ text: string; match: boolean }> = []
  if (index > 0) {
    result.push({ text: text.slice(0, index), match: false })
  }
  result.push({ text: text.slice(index, index + normalizedQuery.length), match: true })
  if (index + normalizedQuery.length < text.length) {
    result.push({ text: text.slice(index + normalizedQuery.length), match: false })
  }
  return result
}

function recentItemIsPinned(itemId: string) {
  return pinnedRecentItemIds.value.includes(itemId)
}

function recentItemIsHidden(itemId: string) {
  return hiddenRecentItemIds.value.includes(itemId)
}

function togglePinRecentItem(itemId: string) {
  pinnedRecentItemIds.value = recentItemIsPinned(itemId)
    ? pinnedRecentItemIds.value.filter((id) => id !== itemId)
    : [itemId, ...pinnedRecentItemIds.value.filter((id) => id !== itemId)]
}

function hideRecentItem(itemId: string) {
  if (recentItemIsHidden(itemId)) return
  hiddenRecentItemIds.value = [...hiddenRecentItemIds.value, itemId]
  showToast('已移入回收站', '这条最近记录已移除，可在回收站恢复。')
}

function restoreHiddenRecentItems() {
  const count = hiddenRecentItemIds.value.length
  hiddenRecentItemIds.value = []
  showToast('最近记录已恢复', `已恢复 ${count} 条最近记录。`)
}

function restoreHiddenRecentItem(itemId: string) {
  hiddenRecentItemIds.value = hiddenRecentItemIds.value.filter((id) => id !== itemId)
  showToast('最近记录已恢复', '已恢复这条最近记录。')
}

function clearHiddenRecentItems() {
  hiddenRecentItemIds.value = []
  openRecentRecycleBinModal.value = false
  showToast('回收站已清空', '最近记录回收站已清空。')
}

function activeSearchResultIndex() {
  return activeSearchResultList.value.findIndex((result) => result.id === searchResultActiveId.value)
}

function searchResultIsActive(resultId: string) {
  return searchResultActiveId.value === resultId
}

function showSearchLoopHint(message: string) {
  searchLoopHint.value = message
  if (searchLoopHintTimer) {
    clearTimeout(searchLoopHintTimer)
  }
  searchLoopHintTimer = setTimeout(() => {
    searchLoopHint.value = ''
    searchLoopHintTimer = null
  }, 1200)
}

function moveSearchSelection(direction: 'next' | 'previous') {
  if (!activeSearchResultList.value.length) {
    searchResultActiveId.value = null
    searchLoopHint.value = ''
    return
  }

  const currentIndex = activeSearchResultIndex()
  const lastIndex = activeSearchResultList.value.length - 1
  const didLoopToStart = direction === 'next' && currentIndex === lastIndex
  const didLoopToEnd = direction === 'previous' && currentIndex === 0
  const nextIndex = currentIndex < 0
    ? 0
    : direction === 'next'
      ? (currentIndex + 1) % activeSearchResultList.value.length
      : (currentIndex - 1 + activeSearchResultList.value.length) % activeSearchResultList.value.length

  searchResultActiveId.value = activeSearchResultList.value[nextIndex]?.id ?? null
  if (didLoopToStart) {
    showSearchLoopHint('已回到第一条')
  } else if (didLoopToEnd) {
    showSearchLoopHint('已跳到最后一条')
  } else if (searchLoopHint.value) {
    searchLoopHint.value = ''
  }
}

function openActiveSearchResult() {
  const target = activeSearchResultList.value.find((result) => result.id === searchResultActiveId.value) ?? activeSearchResultList.value[0]
  if (!target) return
  if (openSearchModal.value) {
    openSearchModal.value = false
  }
  target.onOpen()
}

function handleSearchInputKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    moveSearchSelection('next')
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    moveSearchSelection('previous')
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    openActiveSearchResult()
  }
}

function pushSearchResult(results: SearchResultItem[], query: string, item: SearchResultItem, haystack: string[]) {
  const normalizedParts = haystack.filter(Boolean).map((part) => part.toLocaleLowerCase())
  const normalizedHaystack = normalizedParts.join(' ')

  if (!query) {
    results.push({ ...item, score: 0 })
    return
  }

  if (!normalizedHaystack.includes(query)) return

  const exact = normalizedParts.some((part) => part === query)
  const prefix = normalizedParts.some((part) => part.startsWith(query))
  const titlePrefix = item.title.toLocaleLowerCase().startsWith(query)
  const score = exact ? 120 : titlePrefix ? 100 : prefix ? 80 : 60
  results.push({ ...item, score })
}

function restoreCommandStrategyLabel(strategy: RestoreCommandStrategy) {
  return restoreCommandStrategyOptions.find((option) => option.value === strategy)?.label ?? '仅布局'
}

function countAttentionSessions() {
  let count = 0
  for (const workspace of workspaces.value) {
    for (const tab of workspace.tabs) {
      for (const pane of flattenLeafPanes(tab.panes)) {
        for (const session of paneSessions(pane)) {
          if (shouldCountAttentionState(sessionAttentionState(session))) {
            count += 1
          }
        }
      }
    }
  }
  return count
}

function canInsertIntoCurrentTerminal() {
  return appSection.value === 'workspace'
    && workspaceView.value === 'runtime'
    && Boolean(activeRuntimePaneId.value)
}

function splitSettingsItem(item: string) {
  const index = item.indexOf('：')
  if (index < 0) {
    return {
      label: item,
      value: '已配置',
    }
  }
  return {
    label: item.slice(0, index),
    value: item.slice(index + 1),
  }
}

function openRecentCommandTarget(workspaceId?: string, entryId?: string | null, command?: string) {
  if (workspaceId && entryId) {
    const workspace = workspaces.value.find((candidate) => candidate.id === workspaceId)
    if (workspace) {
      const located = findSessionLocation(workspace, entryId)
      if (located) {
        switchOpenedWorkspace(workspaceId)
        openRuntimeFromPane(located.tabId, located.paneId)
        activatePaneSession(located.paneId, located.sessionId)
        return
      }
      openWorkspace(workspaceId)
      return
    }
  }

  if (command) {
    void copyCommandText(command)
    showToast('未找到来源终端', '已先复制命令。你也可以在运行态里回填到当前输入框。')
  }
}

function openSearchPage() {
  closeFloatingMenus()
  appSection.value = 'search'
  workspaceView.value = 'overview'
  nextTick(() => {
    document.querySelector<HTMLInputElement>('[data-app-search-input]')?.focus()
  })
}

function openQuickSearch() {
  closeFloatingMenus()
  openSearchModal.value = true
  nextTick(() => {
    document.querySelector<HTMLInputElement>('[data-quick-search-input]')?.focus()
  })
}

function toggleRecentWorkspaceFilterMenu() {
  if (openRecentWorkspaceFilterMenu.value) {
    openRecentWorkspaceFilterMenu.value = false
    return
  }
  closeFloatingMenus()
  openRecentWorkspaceFilterMenu.value = true
}

function selectRecentWorkspaceFilter(workspaceId: string) {
  recentWorkspaceFilter.value = workspaceId
  openRecentWorkspaceFilterMenu.value = false
}

function restoreWorkspaceSnapshotFromRecent(workspaceId: string, snapshotId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  const snapshot = workspace?.snapshots?.find((item) => item.id === snapshotId)
  if (!workspace || !snapshot) return

  requestConfirm({
    title: `恢复现场「${snapshot.name}」`,
    description: '将恢复该现场保存时的项目结构、Pane 分栏和终端焦点，不会自动执行命令。',
    confirmLabel: '恢复现场',
    variant: 'primary',
    details: [
      `工作区：${workspace.name}`,
      `项目数：${snapshot.tabsState.length}`,
      `终端数：${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`,
    ],
    action: () => {
      switchOpenedWorkspace(workspaceId)
      nextTick(() => restoreWorkspaceSnapshot(snapshotId))
    },
  })
}

const pendingTemplateEntries: TerminalEntry[] = []

function applyWorkflowTemplate(template: WorkflowTemplate) {
  const workspace = templateApplyTargetWorkspace.value
  if (!workspace) {
    showToast('暂无工作区', '请先创建一个工作区，再使用工作流模板。')
    return
  }

  requestConfirm({
    title: `使用模板「${template.name}」`,
    description: `将向工作区「${workspace.name}」新增一个项目，包含 ${template.panes.length} 个终端分组。不会自动执行命令；如果目标不是你想要的工作区，请先切换工作区后再使用。`,
    confirmLabel: '创建项目',
    variant: 'primary',
    details: [
      `目标工作区：${workspace.name}`,
      `根目录：${workspace.rootPath}`,
      `将创建终端：${template.panes.length} 个`,
      `预置命令：${template.panes.filter((pane) => pane.defaultCommand.trim()).length} 条`,
      `模板标签：${template.tags.length ? template.tags.join(' / ') : '无'}`,
    ],
    action: () => applyWorkflowTemplateToWorkspace(template, workspace.id),
  })
}

function applyWorkflowTemplateToWorkspace(template: WorkflowTemplate, workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId) ?? templateApplyTargetWorkspace.value
  if (!workspace) {
    showToast('工作区不存在', '当前模板应用目标已失效，请重新选择工作区。')
    return
  }

  const now = new Date().toISOString()
  const tabId = createId('tab')
  const panes = template.panes.map((templatePane, index): PaneNode => {
    const entry = createTerminalEntryRecord({
      workspaceId: workspace.id,
      name: templatePane.name,
      workingDirectory: resolveTemplateWorkingDirectory(workspace.rootPath, templatePane.workingDirectoryHint),
      defaultCommand: templatePane.defaultCommand,
      launchMode: 'open-only',
      tags: templatePane.tags,
      note: `${template.name} 工作流模板生成`,
    })
    const paneId = createId('pane')
    const sessionId = createId('session')

    pendingTemplateEntries.push(entry)

    return {
      id: paneId,
      tabId,
      name: templatePane.name,
      pathLabel: entry.workingDirectory,
      terminalEntryId: entry.id,
      splitDirection: template.panes.length > 1 ? 'vertical' : 'none',
      parentPaneId: null,
      order: index,
      sizeRatio: 1,
      activeSessionId: sessionId,
      sessions: [{
        id: sessionId,
        name: templatePane.name,
        pathLabel: entry.workingDirectory,
        terminalEntryId: entry.id,
        status: 'idle',
      }],
      children: [],
    }
  })
  const nextTab: WorkspaceTab = {
    id: tabId,
    workspaceId: workspace.id,
    name: template.name,
    order: workspace.tabs.length,
    layoutMode: template.panes.length > 2 ? 'grid' : template.panes.length === 2 ? 'horizontal' : 'grid',
    paneSequence: panes.length,
    panes,
    createdAt: now,
    updatedAt: now,
  }
  const templateEntries = [...pendingTemplateEntries]
  pendingTemplateEntries.length = 0

  commitWorkspaces((current) => current.map((item) =>
    item.id === workspace.id
      ? {
          ...item,
          terminalEntries: [...item.terminalEntries, ...templateEntries],
          tabs: [...item.tabs, nextTab],
          updatedAt: now,
        }
      : item,
  ))

  if (!openedWorkspaceIds.value.includes(workspace.id)) {
    openedWorkspaceIds.value = [...openedWorkspaceIds.value, workspace.id]
  }
  selectedWorkspaceId.value = workspace.id
  activeRuntimeTabId.value = tabId
  activeRuntimePaneId.value = panes[0]?.id ?? ''
  appSection.value = 'workspace'
  workspaceView.value = 'runtime'
  showToast('工作流模板已应用', `${template.name} · 已加入 ${workspace.name}，创建了 ${panes.length} 个终端，不会自动执行命令。`)
}

function resolveTemplateWorkingDirectory(rootPath: string, hint: string) {
  const value = hint.trim()
  if (!value || value === '.') return rootPath
  if (/^[a-zA-Z]:[\\/]/.test(value) || value.startsWith('\\\\')) return value
  return `${rootPath.replace(/[\\/]+$/, '')}\\${value.replace(/^[\\/]+/, '')}`
}

function saveActiveTabAsWorkflowTemplate() {
  const workspace = selectedWorkspace.value
  const tab = activeRuntimeTab.value
  if (!workspace || !tab) {
    showToast('无法保存模板', '请先打开一个工作区项目。')
    return
  }

  const panes = flattenLeafPanes(tab.panes)
  if (!panes.length) {
    showToast('无法保存模板', '当前项目还没有终端 Pane。')
    return
  }

  requestConfirm({
    title: `保存项目「${tab.name}」为模板`,
    description: `将从工作区「${workspace.name}」的当前项目结构生成一条“我的模板”，包含 ${panes.length} 个 Pane，不会立即创建新终端。`,
    confirmLabel: '保存模板',
    variant: 'primary',
    details: [
      `工作区：${workspace.name}`,
      `项目：${tab.name}`,
      `Pane 数：${panes.length}`,
      `终端数：${countPaneSessions(tab.panes)}`,
    ],
    action: () => {
      const template = createWorkflowTemplateFromInput({
        name: `${tab.name} 模板`,
        description: `从 ${workspace.name} / ${tab.name} 保存的工作流模板。`,
        tags: ['我的模板'],
        panes: panes.map((pane) => {
          const entry = workspaceEntryById(workspace, pane.terminalEntryId)
          const workingDirectory = entry?.workingDirectory || pane.pathLabel || workspace.rootPath
          return {
            name: pane.name,
            shellType: entry?.shellType ?? 'pwsh7',
            workingDirectoryHint: relativeTemplateWorkingDirectory(workspace.rootPath, workingDirectory),
            defaultCommand: entry?.defaultCommand ?? '',
            tags: entry?.tags ?? [],
          }
        }),
      })

      userWorkflowTemplates.value = [template, ...userWorkflowTemplates.value].slice(0, 40)
      showToast('模板已保存', `${template.name} · 来自 ${workspace.name} / ${tab.name}`)
    },
  })
}

function openWorkflowTemplateEditModal(templateId: string) {
  const template = userWorkflowTemplates.value.find((item) => item.id === templateId)
  if (!template) return
  closeFloatingMenus()
  templateEditorMode.value = 'edit'
  editingTemplateId.value = templateId
  workflowTemplateForm.name = template.name
  workflowTemplateForm.description = template.description
  workflowTemplateForm.tagsText = template.tags.join(', ')
  openTemplateEditorModal.value = true
}

function closeWorkflowTemplateEditorModal() {
  openTemplateEditorModal.value = false
  editingTemplateId.value = null
}

function submitWorkflowTemplateForm() {
  if (!workflowTemplateForm.name.trim()) {
    showToast('信息未完整', '模板名称不能为空。')
    return
  }

  if (templateEditorMode.value === 'edit' && editingTemplateId.value) {
    const now = new Date().toISOString()
    userWorkflowTemplates.value = userWorkflowTemplates.value.map((template) =>
      template.id === editingTemplateId.value
        ? {
            ...template,
            name: workflowTemplateForm.name.trim(),
            description: workflowTemplateForm.description.trim(),
            tags: parseTags(workflowTemplateForm.tagsText),
            updatedAt: now,
          }
        : template,
    )
    showToast('模板已更新', workflowTemplateForm.name.trim())
  }

  closeWorkflowTemplateEditorModal()
}

function relativeTemplateWorkingDirectory(rootPath: string, workingDirectory: string) {
  const normalizedRoot = rootPath.replace(/[\\/]+$/, '').toLocaleLowerCase()
  const normalizedDir = workingDirectory.replace(/[\\/]+$/, '').toLocaleLowerCase()
  if (normalizedDir === normalizedRoot) return '.'
  if (normalizedDir.startsWith(`${normalizedRoot}\\`)) {
    return workingDirectory.slice(rootPath.replace(/[\\/]+$/, '').length + 1)
  }
  return workingDirectory
}

function removeWorkflowTemplate(templateId: string) {
  const template = userWorkflowTemplates.value.find((item) => item.id === templateId)
  if (!template) return

  requestConfirm({
    title: `删除模板「${template.name}」`,
    description: '只会删除这条用户模板本身，不会影响已经基于它创建出来的工作区、项目、Pane 和终端。',
    confirmLabel: '删除模板',
    variant: 'danger',
    action: () => {
      userWorkflowTemplates.value = userWorkflowTemplates.value.filter((item) => item.id !== templateId)
      showToast('模板已删除', template.name)
    },
  })
}

function duplicateWorkflowTemplate(templateId: string) {
  const template = allWorkflowTemplates.value.find((item) => item.id === templateId)
  if (!template) return

  const copy = createWorkflowTemplateFromInput({
    name: `${template.name} 副本`,
    description: template.description,
    tags: template.tags,
    panes: template.panes.map((pane) => ({
      name: pane.name,
      shellType: pane.shellType,
      workingDirectoryHint: pane.workingDirectoryHint,
      defaultCommand: pane.defaultCommand,
      tags: pane.tags,
    })),
  })

  userWorkflowTemplates.value = [copy, ...userWorkflowTemplates.value].slice(0, 40)
  showToast('模板已复制', copy.name)
}

function removeWorkspaceSnapshot(workspaceId: string, snapshotId: string) {
  if (!workspaceId || !snapshotId) return
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  const snapshot = workspace?.snapshots?.find((item) => item.id === snapshotId)
  if (!workspace || !snapshot) return

  requestConfirm({
    title: `删除现场「${snapshot.name}」`,
    description: '删除后，这条现场记录将从最近页和工作区快照列表中移除。',
    confirmLabel: '删除现场',
    variant: 'danger',
    action: () => {
      commitWorkspaces((current) => current.map((item) => {
        if (item.id !== workspaceId) return item
        const nextSnapshots = (item.snapshots ?? []).filter((entry) => entry.id !== snapshotId)
        return {
          ...item,
          snapshots: nextSnapshots,
          defaultSnapshotId: item.defaultSnapshotId === snapshotId ? nextSnapshots[0]?.id ?? null : item.defaultSnapshotId,
          updatedAt: new Date().toISOString(),
        }
      }))

      showToast('现场已删除', snapshot.name)
    },
  })
}

function selectOverviewWorkspace(workspaceId: string) {
  selectedOverviewWorkspaceId.value = workspaceId
}

function shiftOverviewWorkspace(direction: 'previous' | 'next') {
  if (!workspaces.value.length) return

  const offset = direction === 'next' ? 1 : -1
  const nextIndex = (overviewWorkspaceIndex.value + offset + workspaces.value.length) % workspaces.value.length
  selectedOverviewWorkspaceId.value = workspaces.value[nextIndex].id
}

function openWorkspaceTerminalEntries(workspaceId: string) {
  closeFloatingMenus()
  switchOpenedWorkspace(workspaceId)
  openTerminalEntriesModal.value = true
}

function switchOpenedWorkspace(workspaceId: string) {
  rememberWorkspaceFocus()
  if (!openedWorkspaceIds.value.includes(workspaceId)) {
    openedWorkspaceIds.value = [...openedWorkspaceIds.value, workspaceId]
  }

  collapsedWorkspaceIds.value = collapsedWorkspaceIds.value.filter((item) => item !== workspaceId)
  selectedWorkspaceId.value = workspaceId
  appSection.value = 'workspace'
  workspaceView.value = 'runtime'

  void refreshWorkspaceGitInfo(workspaceId)
  applyWorkspaceFocus(workspaceId)
}

function closeOpenedWorkspace(workspaceId: string) {
  openedWorkspaceIds.value = openedWorkspaceIds.value.filter((item) => item !== workspaceId)

  if (selectedWorkspaceId.value === workspaceId) {
    const fallbackId = openedWorkspaceIds.value[0] ?? workspaces.value[0]?.id ?? ''
    if (!fallbackId) {
      goWorkspaceOverview()
      return
    }

    switchOpenedWorkspace(fallbackId)
  }
}

function toggleWorkspaceMenu(workspaceId: string, event?: MouseEvent) {
  const shouldOpen = activeWorkspaceMenu.value !== workspaceId
  const nextPosition = event ? menuPositionFromEvent(event, { offsetX: 12, menuWidth: 280, menuHeight: 220 }) : activeWorkspaceMenuPosition.value
  closeFloatingMenus()
  if (shouldOpen) {
    activeWorkspaceMenu.value = workspaceId
    activeWorkspaceMenuPosition.value = nextPosition
  }
}

function openWorkspaceCreateModal() {
  closeFloatingMenus()
  workspaceEditorMode.value = 'create'
  editingWorkspaceId.value = null
  workspaceForm.name = ''
  workspaceForm.rootPath = ''
  workspaceForm.description = ''
  workspaceForm.tagsText = ''
  openWorkspaceEditorModal.value = true
}

function openWorkspaceEditModal(workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace) return
  closeFloatingMenus()

  workspaceEditorMode.value = 'edit'
  editingWorkspaceId.value = workspaceId
  workspaceForm.name = workspace.name
  workspaceForm.rootPath = workspace.rootPath
  workspaceForm.description = workspace.description
  workspaceForm.tagsText = workspace.tags.join(', ')
  openWorkspaceEditorModal.value = true
}

function closeWorkspaceEditorModal() {
  openWorkspaceEditorModal.value = false
}

function submitWorkspaceForm() {
  if (!workspaceForm.name.trim() || !workspaceForm.rootPath.trim()) {
    showToast('信息未完整', '工作区名称和根目录不能为空。')
    return
  }

  const tags = parseTags(workspaceForm.tagsText)

  if (workspaceEditorMode.value === 'create') {
    const nextWorkspace = createWorkspaceRecord({
      name: workspaceForm.name.trim(),
      rootPath: workspaceForm.rootPath.trim(),
      description: workspaceForm.description.trim(),
      tags,
    })

    workspaces.value = [nextWorkspace, ...workspaces.value]
    openWorkspace(nextWorkspace.id)
    showToast('工作区已创建', `已新增工作区：${nextWorkspace.name}`)
  } else if (editingWorkspaceId.value) {
    workspaces.value = workspaces.value.map((workspace) => {
      if (workspace.id !== editingWorkspaceId.value) return workspace

      const now = new Date().toISOString()
      return {
        ...workspace,
        name: workspaceForm.name.trim(),
        rootPath: workspaceForm.rootPath.trim(),
        description: workspaceForm.description.trim(),
        tags,
        updatedAt: now,
      }
    })

    showToast('工作区已更新', `已保存：${workspaceForm.name.trim()}`)
  }

  closeWorkspaceEditorModal()
}

function removeWorkspace(workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace) return

  requestConfirm({
    title: `删除工作区「${workspace.name}」`,
    description: '该操作会同时移除工作区内的标签页、Pane 布局、运行配置和 Provider。',
    confirmLabel: '确认删除',
    action: () => {
      const sessionIds = workspace.tabs.flatMap((tab) => collectSessionIdsFromPanes(tab.panes))
      const nextRuntimeActiveSessionIds = { ...runtimeActiveSessionIds.value }
      workspace.tabs.flatMap((tab) => flattenLeafPanes(tab.panes)).forEach((pane) => {
        delete nextRuntimeActiveSessionIds[pane.id]
      })
      workspaces.value = workspaces.value.filter((item) => item.id !== workspaceId)
      openedWorkspaceIds.value = openedWorkspaceIds.value.filter((item) => item !== workspaceId)
      selectedWorkspaceId.value = workspaces.value[0]?.id ?? ''
      activeRuntimeTabId.value = workspaces.value[0]?.tabs[0]?.id ?? ''
      workspaceView.value = 'overview'
      appSection.value = 'workspace'
      runtimeActiveSessionIds.value = nextRuntimeActiveSessionIds
      sessionIds.forEach((sessionId) => {
        void destroyTerminalRuntime(sessionId).catch(() => undefined)
      })
      showToast('工作区已删除', `已移除：${workspace.name}`)
    },
  })
}

function openTerminalEntryCreateModal() {
  if (!selectedWorkspace.value) return
  closeFloatingMenus()

  openTerminalEntriesModal.value = false
  terminalEntryEditorMode.value = 'create'
  editingTerminalEntryId.value = null
  terminalEntryForm.name = ''
  terminalEntryForm.workingDirectory = selectedWorkspace.value.rootPath
  terminalEntryForm.defaultCommand = ''
  terminalEntryForm.launchMode = 'open-only'
  terminalEntryForm.providerBindingMode = 'inherit'
  terminalEntryForm.providerProfileId = ''
  terminalEntryForm.modelId = ''
  terminalEntryForm.environmentVariablesText = ''
  terminalEntryForm.mcpPolicy = 'inherit'
  terminalEntryForm.skillPolicy = 'inherit'
  terminalEntryForm.tagsText = ''
  terminalEntryForm.note = ''
  openTerminalEntryEditorModal.value = true
}

function openTerminalEntryEditModal(entryId: string) {
  const entry = selectedWorkspace.value?.terminalEntries.find((item) => item.id === entryId)
  if (!entry) return
  closeFloatingMenus()

  openTerminalEntriesModal.value = false
  terminalEntryEditorMode.value = 'edit'
  editingTerminalEntryId.value = entryId
  terminalEntryForm.name = entry.name
  terminalEntryForm.workingDirectory = entry.workingDirectory
  terminalEntryForm.defaultCommand = entry.defaultCommand
  terminalEntryForm.launchMode = entry.launchMode
  terminalEntryForm.providerBindingMode = entry.providerBindingMode ?? 'inherit'
  terminalEntryForm.providerProfileId = entry.providerProfileId ?? ''
  terminalEntryForm.modelId = entry.modelId ?? ''
  terminalEntryForm.environmentVariablesText = entry.environmentVariablesText ?? ''
  terminalEntryForm.mcpPolicy = entry.mcpPolicy ?? 'inherit'
  terminalEntryForm.skillPolicy = entry.skillPolicy ?? 'inherit'
  terminalEntryForm.tagsText = entry.tags.join(', ')
  terminalEntryForm.note = entry.note ?? ''
  openTerminalEntryEditorModal.value = true
}

function closeTerminalEntryEditorModal() {
  openTerminalEntryEditorModal.value = false
  openLaunchModeMenu.value = false
}

function submitTerminalEntryForm() {
  if (!selectedWorkspace.value) return

  if (!terminalEntryForm.name.trim() || !terminalEntryForm.workingDirectory.trim()) {
    showToast('信息未完整', '配置名称和工作目录不能为空。')
    return
  }

  const tags = parseTags(terminalEntryForm.tagsText)

  if (terminalEntryEditorMode.value === 'create') {
    const nextEntry = createTerminalEntryRecord({
      workspaceId: selectedWorkspace.value.id,
      name: terminalEntryForm.name.trim(),
      workingDirectory: terminalEntryForm.workingDirectory.trim(),
      defaultCommand: terminalEntryForm.defaultCommand.trim(),
      launchMode: terminalEntryForm.launchMode,
      providerBindingMode: terminalEntryForm.providerBindingMode,
      providerProfileId: terminalEntryForm.providerBindingMode === 'explicit' ? (terminalEntryForm.providerProfileId || null) : null,
      modelId: terminalEntryForm.modelId.trim() || null,
      environmentVariablesText: terminalEntryForm.environmentVariablesText.trim() || null,
      mcpPolicy: terminalEntryForm.mcpPolicy,
      skillPolicy: terminalEntryForm.skillPolicy,
      tags,
      note: terminalEntryForm.note.trim() || null,
    })

    patchSelectedWorkspace((workspace) => ({
      ...workspace,
      terminalEntries: [...workspace.terminalEntries, nextEntry],
      updatedAt: new Date().toISOString(),
    }))

    showToast('运行配置已创建', `已新增配置：${nextEntry.name}`)
  } else if (editingTerminalEntryId.value) {
    patchSelectedWorkspace((workspace) => ({
      ...workspace,
      terminalEntries: workspace.terminalEntries.map((entry) => {
        if (entry.id !== editingTerminalEntryId.value) return entry
        const now = new Date().toISOString()
        return {
          ...entry,
          name: terminalEntryForm.name.trim(),
          workingDirectory: terminalEntryForm.workingDirectory.trim(),
          defaultCommand: terminalEntryForm.defaultCommand.trim(),
          launchMode: terminalEntryForm.launchMode,
          providerBindingMode: terminalEntryForm.providerBindingMode,
          providerProfileId: terminalEntryForm.providerBindingMode === 'explicit' ? (terminalEntryForm.providerProfileId || null) : null,
          modelId: terminalEntryForm.modelId.trim() || null,
          environmentVariablesText: terminalEntryForm.environmentVariablesText.trim() || null,
          mcpPolicy: terminalEntryForm.mcpPolicy,
          skillPolicy: terminalEntryForm.skillPolicy,
          tags,
          note: terminalEntryForm.note.trim() || null,
          updatedAt: now,
        }
      }),
      updatedAt: new Date().toISOString(),
    }))

    showToast('运行配置已更新', `已保存：${terminalEntryForm.name.trim()}`)
  }

  closeTerminalEntryEditorModal()
}

function removeTerminalEntry(entryId: string) {
  if (!selectedWorkspace.value) return
  const entry = selectedWorkspace.value.terminalEntries.find((item) => item.id === entryId)
  if (!entry) return

  if (referencedTerminalEntryIds.value.has(entryId)) {
    showToast('无法删除配置', '该配置仍被某个 Pane 引用，请先解除绑定或修改布局。')
    return
  }

  requestConfirm({
    title: `删除运行配置「${entry.name}」`,
    description: '删除后，该运行配置的目录、命令、Provider 绑定和启动模式会从当前工作区移除。',
    confirmLabel: '确认删除',
    action: () => {
      patchSelectedWorkspace((workspace) => ({
        ...workspace,
        terminalEntries: workspace.terminalEntries.filter((item) => item.id !== entryId),
        updatedAt: new Date().toISOString(),
      }))

      showToast('运行配置已删除', `已移除：${entry.name}`)
    },
  })
}

function createTab() {
  if (!selectedWorkspace.value) return

  const now = new Date().toISOString()
  const nextIndex = selectedWorkspace.value.tabs.length + 1
  const nextTab = {
    id: createId('tab'),
    workspaceId: selectedWorkspace.value.id,
    name: `标签页 ${nextIndex}`,
    order: selectedWorkspace.value.tabs.length,
    layoutMode: 'grid' as TabLayoutMode,
    paneSequence: 0,
    panes: [],
    createdAt: now,
    updatedAt: now,
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: [...workspace.tabs, nextTab],
    updatedAt: now,
  }))

  activeRuntimeTabId.value = nextTab.id
  activeRuntimePaneId.value = ''
  workspaceView.value = 'runtime'
  showToast('已新建标签页', `当前标签页：${nextTab.name}`)
}

function createPane() {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  if (activeRuntimePaneId.value) {
    createPaneSession(activeRuntimePaneId.value)
    return
  }

  const now = new Date().toISOString()
  const paneId = createId('pane')
  const sessionId = createId('session')
  const paneName = 'PowerShell 7'
  const nextPane: PaneNode = {
    id: paneId,
    tabId: activeRuntimeTab.value.id,
    name: paneName,
    pathLabel: selectedWorkspace.value.rootPath,
    terminalEntryId: null,
    splitDirection: 'none',
    parentPaneId: null,
    order: activeRuntimeTab.value.panes.length,
    sizeRatio: 1,
    activeSessionId: sessionId,
    sessions: [{
      id: sessionId,
      name: 'PowerShell 7',
      pathLabel: selectedWorkspace.value.rootPath,
      terminalEntryId: null,
      status: 'idle',
    }],
    children: [],
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? { ...tab, paneSequence: (tab.paneSequence || 0) + 1, panes: [...tab.panes, nextPane], updatedAt: now }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = paneId
  setRuntimeActiveSessionId(paneId, sessionId)
  showToast('终端已创建', `当前标签页新增：${paneName}`)
}

function openProviderCreateModal() {
  if (!selectedWorkspace.value) return
  closeFloatingMenus()
  openTerminalEntriesModal.value = false
  providerEditorMode.value = 'create'
  editingProviderId.value = null
  providerForm.name = ''
  providerForm.providerKind = 'openai-compatible'
  providerForm.baseUrl = ''
  providerForm.apiKey = ''
  providerForm.apiFormat = 'openai'
  providerForm.defaultModel = ''
  providerForm.toolTargetsText = 'codex, generic'
  providerForm.color = '#4b83ff'
  providerForm.note = ''
  providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
  openProviderEditorModal.value = true
}

function openProviderEditModal(providerId: string) {
  const provider = selectedWorkspaceProviders.value.find((item) => item.id === providerId)
  if (!provider) return
  closeFloatingMenus()
  openTerminalEntriesModal.value = false
  providerEditorMode.value = 'edit'
  editingProviderId.value = providerId
  providerForm.name = provider.name
  providerForm.providerKind = provider.providerKind
  providerForm.baseUrl = provider.baseUrl
  providerForm.apiKey = provider.apiKey
  providerForm.apiFormat = provider.apiFormat
  providerForm.defaultModel = provider.defaultModel
  providerForm.toolTargetsText = provider.toolTargets.join(', ')
  providerForm.color = provider.color || '#4b83ff'
  providerForm.note = provider.note || ''
  providerForm.isDefault = Boolean(provider.isDefault)
  openProviderEditorModal.value = true
}

function closeProviderEditorModal() {
  openProviderEditorModal.value = false
  editingProviderId.value = null
}

function submitProviderForm() {
  if (!selectedWorkspace.value) return
  if (!providerForm.name.trim()) {
    showToast('信息未完整', 'Provider 名称不能为空。')
    return
  }

  const toolTargets = parseProviderToolTargets(providerForm.toolTargetsText)

  if (providerEditorMode.value === 'create') {
    const provider = createProviderProfileRecord({
      workspaceId: selectedWorkspace.value.id,
      name: providerForm.name.trim(),
      providerKind: providerForm.providerKind,
      baseUrl: providerForm.baseUrl.trim(),
      apiKey: providerForm.apiKey.trim(),
      apiFormat: providerForm.apiFormat,
      defaultModel: providerForm.defaultModel.trim(),
      toolTargets,
      color: providerForm.color.trim() || null,
      note: providerForm.note.trim() || null,
      isDefault: providerForm.isDefault,
    })

    patchSelectedWorkspace((workspace) => ({
      ...workspace,
      providerProfiles: [
        ...(workspace.providerProfiles ?? []).map((item) => ({ ...item, isDefault: provider.isDefault ? false : item.isDefault })),
        provider,
      ],
      updatedAt: new Date().toISOString(),
    }))

    showToast('Provider 已创建', provider.name)
  } else if (editingProviderId.value) {
    patchSelectedWorkspace((workspace) => ({
      ...workspace,
      providerProfiles: (workspace.providerProfiles ?? []).map((provider) => {
        if (provider.id === editingProviderId.value) {
          return {
            ...provider,
            name: providerForm.name.trim(),
            providerKind: providerForm.providerKind,
            baseUrl: providerForm.baseUrl.trim(),
            apiKey: providerForm.apiKey.trim(),
            apiFormat: providerForm.apiFormat,
            defaultModel: providerForm.defaultModel.trim(),
            toolTargets,
            color: providerForm.color.trim() || null,
            note: providerForm.note.trim() || null,
            isDefault: providerForm.isDefault,
            updatedAt: new Date().toISOString(),
          }
        }
        return {
          ...provider,
          isDefault: providerForm.isDefault ? false : provider.isDefault,
        }
      }),
      updatedAt: new Date().toISOString(),
    }))

    showToast('Provider 已更新', providerForm.name.trim())
  }

  closeProviderEditorModal()
}

function removeProviderProfile(providerId: string) {
  if (!selectedWorkspace.value) return
  const provider = selectedWorkspaceProviders.value.find((item) => item.id === providerId)
  if (!provider) return

  const inUse = selectedWorkspaceEntries.value.some((entry) => entry.providerBindingMode === 'explicit' && entry.providerProfileId === providerId)
  if (inUse) {
    showToast('无法删除 Provider', '仍有运行配置显式绑定该 Provider，请先调整绑定关系。')
    return
  }

  requestConfirm({
    title: `删除 Provider「${provider.name}」`,
    description: '删除后，当前工作区中与它相关的显式选择将失效。',
    confirmLabel: '删除 Provider',
    action: () => {
      patchSelectedWorkspace((workspace) => ({
        ...workspace,
        providerProfiles: (workspace.providerProfiles ?? []).filter((item) => item.id !== providerId),
        updatedAt: new Date().toISOString(),
      }))
      showToast('Provider 已删除', provider.name)
    },
  })
}

function snapshotName(workspaceName = '工作区') {
  const date = new Date()
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${workspaceName} · 现场 ${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

function saveCurrentWorkspaceSnapshot() {
  const workspace = selectedWorkspace.value
  if (!workspace) return

  rememberWorkspaceFocus(workspace.id)
  const focus = captureWorkspaceFocus(workspace.id)
  const snapshot = createWorkspaceSnapshotRecord({
    workspace,
    name: snapshotName(workspace.name),
    activeTabId: focus.activeTabId,
    activePaneId: focus.activePaneId,
    activePaneIdsByTab: focus.activePaneIdsByTab,
    activeSessionIdsByPane: focus.activeSessionIdsByPane,
  })
  const now = new Date().toISOString()

  patchSelectedWorkspace((current) => ({
    ...current,
    snapshots: [snapshot, ...(current.snapshots ?? [])].slice(0, 8),
    defaultSnapshotId: snapshot.id,
    updatedAt: now,
  }))

  showToast('现场已保存', `${snapshot.name} · ${workspace.tabs.length} 个项目 · ${totalWorkspaceSessions(workspace)} 个终端`)
}

function restoreDefaultWorkspaceSnapshot() {
  const snapshot = defaultWorkspaceSnapshot.value
  if (!snapshot) {
    showToast('暂无现场快照', '请先保存一次当前工作现场。')
    return
  }

  requestConfirm({
    title: `恢复现场「${snapshot.name}」`,
    description: '将恢复该快照中的项目、Pane 分栏、终端标签与焦点信息，不会自动执行任何命令。',
    confirmLabel: '恢复现场',
    variant: 'primary',
    details: [
      `工作区：${selectedWorkspace.value?.name || snapshot.workspaceId}`,
      `项目数：${snapshot.tabsState.length}`,
      `终端数：${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`,
    ],
    action: () => restoreWorkspaceSnapshot(snapshot.id),
  })
}

function restoreActiveTabFromDefaultSnapshot() {
  if (!activeRuntimeTab.value) return
  const snapshot = defaultWorkspaceSnapshot.value
  if (!snapshot) {
    showToast('暂无现场快照', '请先保存一次当前工作现场。')
    return
  }

  const snapshotTab = snapshot.tabsState.find((tab) => tab.id === activeRuntimeTab.value?.id)
  if (!snapshotTab) {
    showToast('无法恢复当前项目', '最近现场中没有找到同名项目。')
    return
  }

  requestConfirm({
    title: `恢复项目「${snapshotTab.name}」`,
    description: '只恢复当前项目的 Pane 分栏和终端标签，不影响同工作区内其他项目。',
    confirmLabel: '恢复项目',
    variant: 'primary',
    details: [
      `工作区：${selectedWorkspace.value?.name || snapshot.workspaceId}`,
      `项目：${snapshotTab.name}`,
      `Pane 数：${countLeafPanes(snapshotTab.panes)}`,
      `终端数：${countPaneSessions(snapshotTab.panes)}`,
    ],
    action: () => restoreTabSnapshot(snapshot.id, snapshotTab.id),
  })
}

function restoreWorkspaceSnapshot(snapshotId: string) {
  const workspace = selectedWorkspace.value
  const snapshot = workspace?.snapshots?.find((item) => item.id === snapshotId)
  if (!workspace || !snapshot) return

  const previousTabs = workspace.tabs
  const now = new Date().toISOString()
  const nextTabs = cloneSnapshotTabs(snapshot.tabsState, workspace.id, now)
  const restoredWorkspace: WorkspaceCard = { ...workspace, tabs: nextTabs }
  const focus = normalizeWorkspaceFocus(restoredWorkspace, {
    activeTabId: snapshot.activeTabId,
    activePaneId: snapshot.activePaneId,
    activePaneIdsByTab: snapshot.activePaneIdsByTab,
    activeSessionIdsByPane: snapshot.activeSessionIdsByPane,
    collapsedTreeTabIds: collapsedTreeTabIds.value,
  })

  patchSelectedWorkspace((current) => ({
    ...current,
    tabs: nextTabs,
    terminalEntries: applyTerminalEntryStatuses(current.terminalEntries, nextTabs),
    defaultSnapshotId: snapshot.id,
    updatedAt: now,
  }))

  activeRuntimeTabId.value = focus.activeTabId ?? ''
  activeRuntimePaneId.value = focus.activePaneId ?? ''
  runtimeActiveSessionIds.value = resolveRestoredSessionIds(restoredWorkspace, focus.activeSessionIdsByPane)
  workspaceFocusCache.value = {
    ...workspaceFocusCache.value,
    [workspace.id]: focus,
  }
  destroySessionsMissingAfterRestore(previousTabs, nextTabs)
  showToast('现场已恢复', `${snapshot.name} · 已恢复整个工作区布局`)
  void applyCommandRestoreStrategy('工作区现场', { silentNoop: true })
}

function restoreTabSnapshot(snapshotId: string, tabId: string) {
  const workspace = selectedWorkspace.value
  const snapshot = workspace?.snapshots?.find((item) => item.id === snapshotId)
  const snapshotTab = snapshot?.tabsState.find((tab) => tab.id === tabId)
  if (!workspace || !snapshot || !snapshotTab) return

  const currentTab = workspace.tabs.find((tab) => tab.id === tabId)
  const previousTabs = workspace.tabs
  const now = new Date().toISOString()
  const restoredTab = cloneSnapshotTab(snapshotTab, workspace.id, now)
  const nextTabs = currentTab
    ? workspace.tabs.map((tab) => tab.id === tabId ? { ...restoredTab, order: tab.order } : tab)
    : [...workspace.tabs, { ...restoredTab, order: workspace.tabs.length }]
  const restoredWorkspace: WorkspaceCard = { ...workspace, tabs: nextTabs }
  const preferredPaneId = snapshot.activePaneIdsByTab?.[tabId] ?? (snapshot.activeTabId === tabId ? snapshot.activePaneId : null)
  const focus = normalizeWorkspaceFocus(restoredWorkspace, {
    ...workspaceFocusCache.value[workspace.id],
    activeTabId: tabId,
    activePaneId: preferredPaneId,
    activePaneIdsByTab: {
      ...(workspaceFocusCache.value[workspace.id]?.activePaneIdsByTab ?? {}),
      [tabId]: preferredPaneId ?? '',
    },
    activeSessionIdsByPane: {
      ...runtimeActiveSessionIds.value,
      ...(snapshot.activeSessionIdsByPane ?? {}),
    },
  })

  patchSelectedWorkspace((current) => ({
    ...current,
    tabs: nextTabs,
    terminalEntries: applyTerminalEntryStatuses(current.terminalEntries, nextTabs),
    defaultSnapshotId: snapshot.id,
    updatedAt: now,
  }))

  activeRuntimeTabId.value = tabId
  activeRuntimePaneId.value = focus.activePaneIdsByTab[tabId] ?? focus.activePaneId ?? findFirstLeafPaneId(restoredTab.panes)
  runtimeActiveSessionIds.value = resolveRestoredSessionIds(restoredWorkspace, focus.activeSessionIdsByPane)
  workspaceFocusCache.value = {
    ...workspaceFocusCache.value,
    [workspace.id]: focus,
  }
  destroySessionsMissingAfterRestore(previousTabs, nextTabs)
  showToast('项目布局已恢复', `已恢复：${snapshotTab.name}`)
  void applyCommandRestoreStrategy('当前项目', { silentNoop: true })
}

function cloneSnapshotTabs(tabs: WorkspaceTab[], workspaceId: string, updatedAt: string) {
  return tabs.map((tab, index) => ({ ...cloneSnapshotTab(tab, workspaceId, updatedAt), order: index }))
}

function cloneSnapshotTab(tab: WorkspaceTab, workspaceId: string, updatedAt: string): WorkspaceTab {
  return {
    ...tab,
    workspaceId,
    panes: clonePaneTreeForRestore(tab.panes),
    updatedAt,
  }
}

function clonePaneSessionForRestore(
  pane: PaneNode,
  session: PaneTerminalSession | undefined,
  fallbackIndex = 0,
): PaneTerminalSession {
  const fallbackName = fallbackIndex === 0 ? pane.name : `${pane.name} · ${fallbackIndex + 1}`
  return {
    id: createId('session'),
    name: session?.name || fallbackName,
    pathLabel: session?.pathLabel || pane.pathLabel,
    terminalEntryId: session?.terminalEntryId ?? pane.terminalEntryId ?? null,
    status: 'idle',
    hasUserCommand: false,
    lastCommandAt: null,
    lastOutputAt: null,
    lastExitCode: null,
    supervisorMode: 'off',
    supervisorState: 'idle',
    expectedDoneSignal: null,
    lastHeartbeatAt: null,
    lastActivityAt: null,
    supervisorNote: null,
  }
}

function clonePaneTreeForRestore(panes: PaneNode[]): PaneNode[] {
  return panes.map((pane) => {
    const nextChildren = pane.children?.length ? clonePaneTreeForRestore(pane.children) : []
    const nextSessions = nextChildren.length
      ? []
      : (pane.sessions?.length
          ? pane.sessions.map((session, index) => clonePaneSessionForRestore(pane, session, index))
          : [clonePaneSessionForRestore(pane, undefined, 0)])

    return {
      ...pane,
      sessions: nextSessions,
      activeSessionId: nextChildren.length ? null : nextSessions[0]?.id ?? null,
      children: nextChildren,
    }
  })
}

function renameSession(sessionId: string, nextName: string) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const now = new Date().toISOString()

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) =>
                  session.id === sessionId
                    ? { ...session, name: nextName }
                    : session,
                ),
              }
            }),
            updatedAt: now,
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      updatedAt: now,
    }
  }))
}

function destroySessionsMissingAfterRestore(previousTabs: WorkspaceTab[], nextTabs: WorkspaceTab[]) {
  const nextSessionIds = new Set(nextTabs.flatMap((tab) => collectSessionIdsFromPanes(tab.panes)))
  previousTabs
    .flatMap((tab) => collectSessionIdsFromPanes(tab.panes))
    .filter((sessionId) => !nextSessionIds.has(sessionId))
    .forEach((sessionId) => {
      void destroyTerminalRuntime(sessionId).catch(() => undefined)
    })
}

function restoreCommandForEntry(entry: TerminalEntry) {
  return entry.defaultCommand?.trim() || entry.lastCommand?.trim() || ''
}

function restoreCommandActionForEntry(entry: TerminalEntry, strategy: RestoreCommandStrategy): 'prefill' | 'execute' | null {
  if (strategy === 'layout-only') return null
  if (strategy === 'prefill') {
    return entry.launchMode === 'prefill' || entry.launchMode === 'execute' ? 'prefill' : null
  }
  if (entry.launchMode === 'execute') return 'execute'
  if (entry.launchMode === 'prefill') return 'prefill'
  return null
}

function waitForTerminalRuntimeMount(sessionId: string) {
  return new Promise<boolean>((resolve) => {
    let attempts = 0
    const check = () => {
      const state = getTerminalRuntimeState(sessionId)
      if (state.mounted && state.bridgeReady) {
        resolve(true)
        return
      }
      attempts += 1
      if (attempts >= 16) {
        resolve(false)
        return
      }
      window.setTimeout(check, 80)
    }
    check()
  })
}

async function applyCommandRestoreStrategy(scopeLabel: string, options: { silentNoop?: boolean } = {}) {
  const strategy = restoreCommandStrategy.value
  if (strategy === 'layout-only') return

  await nextTick()

  const workspace = selectedWorkspace.value
  const tab = activeRuntimeTab.value
  if (!workspace || !tab) return

  const targets = flattenLeafPanes(tab.panes).flatMap((pane) => {
    const session = activePaneSession(pane)
    const entry = session?.terminalEntryId
      ? workspace.terminalEntries.find((item) => item.id === session.terminalEntryId)
      : null
    if (!session || !entry) return []

    const command = restoreCommandForEntry(entry)
    const action = command ? restoreCommandActionForEntry(entry, strategy) : null
    if (!command || !action) return []

    return [{ pane, session, entry, command, action }]
  })

  if (!targets.length) {
    if (!options.silentNoop) {
      showToast('恢复命令未触发', '当前可见项目中没有符合启动策略的默认命令。')
    }
    return
  }

  let successCount = 0
  let failedCount = 0

  for (const target of targets) {
    try {
      const mounted = await waitForTerminalRuntimeMount(target.session.id)
      if (!mounted) throw new Error('terminal_not_mounted')
      await ensureTerminalReady(target.session.id)
      await writeTerminalText(target.session.id, target.action === 'execute' ? `${target.command}\r` : target.command)
      if (target.action === 'execute') {
        recordSessionCommand(target.session.id, target.command)
      }
      successCount += 1
    } catch {
      failedCount += 1
    }
  }

  if (successCount) {
    showToast(
      strategy === 'execute' ? '恢复命令已处理' : '恢复命令已预填',
      `${scopeLabel}：成功 ${successCount} 个${failedCount ? `，失败 ${failedCount} 个` : ''}`,
    )
    return
  }

  if (failedCount) {
    showToast('恢复命令失败', '终端尚未准备完成，稍后可用闪电入口手动插入。')
  }
}

function requestSplitAction(mode: 'create' | 'duplicate', paneId: string | null = null) {
  splitActionState.mode = mode
  splitActionState.paneId = paneId
  openSplitActionModal.value = true
}

function submitSplitAction(layoutMode: TabLayoutMode) {
  const mode = splitActionState.mode
  const paneId = splitActionState.paneId
  openSplitActionModal.value = false

  if (mode === 'duplicate' && paneId) {
    duplicatePaneWithLayout(paneId, layoutMode)
    return
  }

  createPaneWithLayout(layoutMode)
}

function createPaneWithLayout(layoutMode: TabLayoutMode) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const now = new Date().toISOString()
  const nextPaneId = createId('pane')
  const nextSessionId = createId('session')
  const nextPaneNameValue = 'PowerShell 7'
  const nextPane: PaneNode = {
    id: nextPaneId,
    tabId: activeRuntimeTab.value.id,
    name: nextPaneNameValue,
    pathLabel: selectedWorkspace.value.rootPath,
    terminalEntryId: null,
    splitDirection: layoutMode === 'horizontal' ? 'horizontal' : layoutMode === 'vertical' ? 'vertical' : 'none',
    parentPaneId: null,
    order: activeRuntimeTab.value.panes.length,
    sizeRatio: 1,
    activeSessionId: nextSessionId,
    sessions: [
      {
        id: nextSessionId,
        name: 'PowerShell 7',
        pathLabel: selectedWorkspace.value.rootPath,
        terminalEntryId: null,
        status: 'idle',
      },
    ],
    children: [],
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            layoutMode,
            paneSequence: (tab.paneSequence || 0) + 1,
            panes: [
              ...visitPaneTree(tab.panes, (pane) => ({
                ...pane,
                splitDirection: layoutMode === 'horizontal' ? 'horizontal' : layoutMode === 'vertical' ? 'vertical' : pane.splitDirection,
              })),
              nextPane,
            ],
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = nextPane.id
  setRuntimeActiveSessionId(nextPane.id, nextSessionId)
  workspaceView.value = 'runtime'
  showToast('已新增分屏 Pane', `当前标签页新增：${nextPane.name}`)
}

function duplicatePane(paneId: string) {
  splitLeafPane(paneId, 'horizontal')
}


function createPaneSession(paneId: string) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const targetPane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!targetPane) return

  const boundEntry = targetPane.terminalEntryId
    ? selectedWorkspace.value.terminalEntries.find((entry) => entry.id === targetPane.terminalEntryId)
    : null
  if (boundEntry?.launchMode === 'switch-or-create') {
    const existing = findSessionLocation(selectedWorkspace.value, boundEntry.id)
    const currentSessionId = activePaneSession(targetPane)?.id ?? targetPane.activeSessionId ?? paneSessions(targetPane)[0]?.id ?? null
    if (existing && !isSameSessionLocation(existing, activeRuntimeTab.value.id, paneId, currentSessionId)) {
      activeRuntimeTabId.value = existing.tabId
      activeRuntimePaneId.value = existing.paneId
      activatePaneSession(existing.paneId, existing.sessionId)
      showToast('已切换到现有终端', `当前配置已定位：${boundEntry.name}`)
      return
    }
  }

  const now = new Date().toISOString()
  const nextSession: PaneTerminalSession = {
    id: createId('session'),
    name: 'PowerShell 7',
    pathLabel: terminalSessionWorkingDirectory(targetPane),
    terminalEntryId: targetPane.terminalEntryId,
    status: 'idle',
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) =>
              pane.id === paneId
                ? {
                    ...pane,
                    sessions: [...paneSessions(pane), nextSession],
                    activeSessionId: nextSession.id,
                  }
                : pane,
            ),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = paneId
  setRuntimeActiveSessionId(paneId, nextSession.id)
  showToast('终端已新增', `已在当前分组中新建：${nextSession.name}`)
}

function removePaneSession(paneId: string, sessionId: string) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const targetPane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!targetPane) return

  const sessions = paneSessions(targetPane)
  const targetSession = sessions.find((session) => session.id === sessionId)
  if (!targetSession) return

  requestConfirm({
    title: `关闭终端「${targetSession.name}」`,
    description: '关闭后将结束当前终端会话，Pane 分组会保留。',
    confirmLabel: '确认关闭',
    action: () => {
      const now = new Date().toISOString()
      const shouldRemovePane = sessions.length <= 1

      const removeSessionFromTree = (panes: PaneNode[]): PaneNode[] => {
        const visit = (pane: PaneNode): PaneNode => {
          if (pane.children?.length) {
            return {
              ...pane,
              children: pane.children.map((child) => visit(child)),
            }
          }

          if (pane.id !== paneId) return pane

          const remainingSessions = paneSessions(pane).filter((session) => session.id !== sessionId)
          return {
            ...pane,
            sessions: remainingSessions,
            activeSessionId: remainingSessions[0]?.id ?? null,
          }
        }

        return normalizePaneTree(panes.map((pane) => visit(pane)))
      }

      patchSelectedWorkspace((workspace) => {
        const nextTabs = workspace.tabs.map((tab) => {
          if (tab.id !== activeRuntimeTab.value?.id) return tab

          const nextPanes = shouldRemovePane
            ? collapsePaneBranch(paneId, tab.panes)
            : removeSessionFromTree(tab.panes)

          activeRuntimePaneId.value = shouldRemovePane
            ? findFirstLeafPaneId(nextPanes)
            : paneId

          return {
            ...tab,
            panes: normalizePaneTree(nextPanes),
            updatedAt: now,
          }
        })

        return {
          ...workspace,
          tabs: nextTabs,
          terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
          updatedAt: now,
        }
      })

      if (shouldRemovePane) {
        setRuntimeActiveSessionId(paneId, null)
      } else {
        const remainingSessionIds = sessions.filter((session) => session.id !== sessionId).map((session) => session.id)
        setRuntimeActiveSessionId(paneId, remainingSessionIds[0] ?? null)
      }

      void destroyTerminalRuntime(sessionId).catch(() => undefined)
      showToast('终端已关闭', `已关闭：${targetSession.name}`)
    },
  })
}

function reloadPaneSession(paneId: string, sessionId: string) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return

  setRuntimeActiveSessionId(paneId, sessionId)
  activeRuntimePaneId.value = paneId
  sessionOutputTailBySession.delete(sessionId)
  sessionAttentionNotificationState.delete(sessionId)
  clearSessionAttention(sessionId, { silent: true })

  terminalReloadVersions.value = {
    ...terminalReloadVersions.value,
    [sessionId]: (terminalReloadVersions.value[sessionId] ?? 0) + 1,
  }

  void destroyTerminalRuntime(sessionId)
    .catch(() => undefined)
    .finally(() => showToast('终端已重新加载', located.session.name))
}

const runtimeTabMenuItems = computed<PopoverItem[]>(() => {
  if (!activeRuntimeTabMenuId.value || !selectedWorkspace.value) return []

  const tab = selectedWorkspace.value.tabs.find((item) => item.id === activeRuntimeTabMenuId.value)
  if (!tab) return []

  return [
    {
      label: '保存工作现场',
      icon: 'copy',
      description: '保存当前工作区的 Tab、Pane、终端标签与焦点。',
      onClick: () => {
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
        saveCurrentWorkspaceSnapshot()
      },
    },
    {
      label: '恢复当前项目布局',
      icon: 'refresh',
      description: defaultWorkspaceSnapshot.value ? `从最近现场恢复：${defaultWorkspaceSnapshot.value.name}` : '暂无可恢复的现场快照。',
      onClick: () => {
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
        restoreActiveTabFromDefaultSnapshot()
      },
    },
    {
      label: '重命名项目',
      icon: 'edit',
      description: '修改当前项目名称。',
      onClick: () => {
        openTabRenameModal(tab.id)
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
      },
    },
    {
      label: '删除项目',
      icon: 'trash',
      description: '删除当前项目及其下所有 Pane。',
      danger: true,
      onClick: () => {
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
        removeTab(tab.id)
      },
    },
  ]
})

const explorerProjectMenuItems = computed<PopoverItem[]>(() => {
  if (!activeExplorerProjectMenuId.value || !activeExplorerProjectWorkspaceId.value) return []

  const workspace = workspaces.value.find((item) => item.id === activeExplorerProjectWorkspaceId.value)
  const tab = workspace?.tabs.find((item) => item.id === activeExplorerProjectMenuId.value)
  if (!workspace || !tab) return []

  return [
    {
      label: '保存工作现场',
      icon: 'copy',
      description: '保存当前工作区的完整布局快照。',
      onClick: () => {
        activeExplorerProjectMenuId.value = null
        activeExplorerProjectWorkspaceId.value = null
        activeExplorerProjectMenuPosition.value = null
        if (selectedWorkspace.value?.id !== workspace.id) {
          openWorkspace(workspace.id)
        }
        window.setTimeout(() => saveCurrentWorkspaceSnapshot(), 0)
      },
    },
    {
      label: '恢复该项目布局',
      icon: 'refresh',
      description: (workspace.snapshots ?? []).length ? '从最近现场恢复这个项目。' : '暂无可恢复的现场快照。',
      onClick: () => {
        activeExplorerProjectMenuId.value = null
        activeExplorerProjectWorkspaceId.value = null
        activeExplorerProjectMenuPosition.value = null
        if (selectedWorkspace.value?.id !== workspace.id) {
          openWorkspace(workspace.id)
        }
        activeRuntimeTabId.value = tab.id
        window.setTimeout(() => restoreActiveTabFromDefaultSnapshot(), 0)
      },
    },
    {
      label: '重命名项目',
      icon: 'edit',
      description: '修改当前项目名称。',
      onClick: () => {
        openExplorerTabRename(workspace.id, tab.id)
        activeExplorerProjectMenuId.value = null
        activeExplorerProjectWorkspaceId.value = null
        activeExplorerProjectMenuPosition.value = null
      },
    },
    {
      label: '删除项目',
      icon: 'trash',
      description: '删除当前项目及其下所有 Pane。',
      danger: true,
      onClick: () => {
        activeExplorerProjectMenuId.value = null
        activeExplorerProjectWorkspaceId.value = null
        activeExplorerProjectMenuPosition.value = null
        if (selectedWorkspace.value?.id !== workspace.id) {
          openWorkspace(workspace.id)
        }
        removeTab(tab.id)
      },
    },
  ]
})

function menuPositionFromEvent(
  event: MouseEvent,
  options: { offsetX?: number; offsetY?: number; menuWidth?: number; menuHeight?: number } = {},
) {
  const menuWidth = options.menuWidth ?? 280
  const menuHeight = options.menuHeight ?? 220
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const offsetX = options.offsetX ?? 0
  const offsetY = options.offsetY ?? 0

  return {
    x: Math.min(event.clientX + offsetX, Math.max(16, viewportWidth - menuWidth - 16)),
    y: Math.min(event.clientY + offsetY, Math.max(16, viewportHeight - menuHeight - 16)),
  }
}

function toggleRuntimeTabMenu(tabId: string, event: MouseEvent) {
  closeFloatingMenus()
  activeRuntimeTabMenuId.value = tabId
  activeRuntimeTabMenuPosition.value = menuPositionFromEvent(event)
}

function toggleExplorerProjectMenu(workspaceId: string, tabId: string, event: MouseEvent) {
  const shouldOpen = event.type === 'contextmenu'
    || activeExplorerProjectMenuId.value !== tabId
    || activeExplorerProjectWorkspaceId.value !== workspaceId
  closeFloatingMenus()
  if (!shouldOpen) return
  activeExplorerProjectWorkspaceId.value = workspaceId
  activeExplorerProjectMenuId.value = tabId
  activeExplorerProjectMenuPosition.value = menuPositionFromEvent(event)
}

function duplicatePaneWithLayout(paneId: string, layoutMode: TabLayoutMode) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const sourcePane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!sourcePane) return

  const now = new Date().toISOString()
  const sourceSession = activePaneSession(sourcePane)
  const sourceSessions = paneSessions(sourcePane)
  const nextSession: PaneTerminalSession = {
    id: createId('session'),
    name: nextSessionName(sourcePane, sourceSessions),
    pathLabel: sourceSession?.pathLabel || sourcePane.pathLabel,
    terminalEntryId: sourceSession?.terminalEntryId ?? sourcePane.terminalEntryId,
    status: sourceSession?.status ?? 'idle',
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== paneId) return pane

              const sessions = paneSessions(pane)
              const sourceSessionIndex = Math.max(0, sessions.findIndex((session) => session.id === sourceSession?.id))
              const nextSessions = [
                ...sessions.slice(0, sourceSessionIndex + 1),
                nextSession,
                ...sessions.slice(sourceSessionIndex + 1),
              ]

              return {
                ...pane,
                sessions: nextSessions,
                activeSessionId: nextSession.id,
              }
            }),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = paneId
  setRuntimeActiveSessionId(paneId, nextSession.id)
  showToast('终端标签已复制', `已在当前 Pane 内新增：${nextSession.name}`)
}

function removePane(paneId: string) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const pane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!pane) return

  requestConfirm({
    title: `删除 Pane「${pane.name}」`,
    description: '删除后会移除当前 Pane 分组及其中的所有终端会话。',
    confirmLabel: '确认删除',
    action: () => {
      const sessionsToClose = paneSessions(pane).map((session) => session.id)
      const now = new Date().toISOString()
      patchSelectedWorkspace((workspace) => {
        const nextTabs = workspace.tabs.map((tab) => {
          if (tab.id !== activeRuntimeTab.value?.id) return tab
          const nextPanes = collapsePaneBranch(paneId, tab.panes)
          activeRuntimePaneId.value = findFirstLeafPaneId(nextPanes)
          return {
            ...tab,
            panes: normalizePaneTree(nextPanes),
            updatedAt: now,
          }
        })

        return {
          ...workspace,
          tabs: nextTabs,
          terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
          updatedAt: now,
        }
      })

      sessionsToClose.forEach((sessionId) => {
        void destroyTerminalRuntime(sessionId).catch(() => undefined)
      })
      setRuntimeActiveSessionId(paneId, null)

      showToast('Pane 已删除', `已移除：${pane.name}`)
    },
  })
}

function assignEntryToPane(paneId: string, entryId: string | null) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const now = new Date().toISOString()
  const targetEntry = entryId ? selectedWorkspace.value.terminalEntries.find((entry) => entry.id === entryId) : null

  if (targetEntry?.launchMode === 'switch-or-create') {
    const existing = findSessionLocation(selectedWorkspace.value, targetEntry.id)
    const currentPane = findPaneById(activeRuntimeTab.value.panes, paneId)
    const currentSessionId = currentPane ? (activePaneSession(currentPane)?.id ?? currentPane.activeSessionId ?? paneSessions(currentPane)[0]?.id ?? null) : null
    if (existing && !isSameSessionLocation(existing, activeRuntimeTab.value.id, paneId, currentSessionId)) {
      activePaneBindingMenu.value = null
      activeRuntimeTabId.value = existing.tabId
      activeRuntimePaneId.value = existing.paneId
      activatePaneSession(existing.paneId, existing.sessionId)
      showToast('已切换到现有终端', `当前配置已定位：${targetEntry.name}`)
      return
    }
  }

  patchSelectedWorkspace((workspace) => {
    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) =>
              pane.id === paneId
                ? {
                    ...pane,
                    terminalEntryId: entryId,
                    pathLabel: targetEntry?.workingDirectory || workspace.rootPath,
                    sessions: paneSessions(pane).map((session) => ({
                      ...session,
                      terminalEntryId: entryId,
                      pathLabel: targetEntry?.workingDirectory || workspace.rootPath,
                      status: (session.status === 'running' ? 'running' : 'idle') as PaneTerminalSession['status'],
                    })),
                  }
                : pane,
            ),
            updatedAt: now,
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
      updatedAt: now,
    }
  })

  activePaneBindingMenu.value = null

  if (targetEntry) {
    showToast('Pane 已绑定配置', `当前 Pane 已绑定：${targetEntry.name}`)
    return
  }

  showToast('Pane 已清空绑定', '当前 Pane 已恢复为独立空白终端。')
}

function openPaneDirectory(pane: PaneNode) {
  const path = terminalSessionWorkingDirectory(pane)
  void invoke('open_directory', { path })
    .then(() => {
      showToast('目录已打开', path)
    })
    .catch((error) => {
      const message = error instanceof Error ? error.message : String(error)
      showToast('打开失败', message)
    })
}

async function copyPanePath(pane: PaneNode) {
  const path = terminalSessionWorkingDirectory(pane)

  try {
    await navigator.clipboard.writeText(path)
    showToast('路径已复制', path)
  } catch {
    showToast('复制失败', '当前浏览器环境未允许写入剪贴板。')
  }
}

async function copyPaneCommand(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) {
    showToast('未绑定运行配置', '当前 Pane 还没有绑定运行配置，请先绑定一个运行配置。')
    return
  }
  const command = mode === 'default'
    ? entry.defaultCommand?.trim()
    : entry.lastCommand?.trim()

  if (!command) {
    showToast('暂无命令', mode === 'default' ? '当前 Pane 未设置默认命令。' : '当前 Pane 还没有历史命令。')
    return
  }

  try {
    await navigator.clipboard.writeText(command)
    showToast(mode === 'default' ? '默认命令已复制' : '最后命令已复制', command)
  } catch {
    showToast('复制失败', '当前浏览器环境未允许写入剪贴板。')
  }
}

async function insertPaneCommand(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) {
    showToast('未绑定运行配置', '当前 Pane 还没有绑定运行配置，请先绑定一个运行配置。')
    return
  }
  const command = mode === 'default'
    ? entry.defaultCommand?.trim()
    : entry.lastCommand?.trim()

  if (!command) {
    showToast('暂无命令', mode === 'default' ? '当前 Pane 未设置默认命令。' : '当前 Pane 还没有历史命令。')
    return
  }

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast('终端未就绪', '当前 Pane 还没有可写入的终端会话。')
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, command)
    showToast('命令已插入', command)
  } catch {
    showToast('插入失败', '当前终端尚未准备完成，稍后再试。')
  }
}

async function insertPaneText(pane: PaneNode, text: string) {
  const command = text.trim()
  if (!command) return

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast('终端未就绪', '当前 Pane 还没有可写入的终端会话。')
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, command)
    showToast('命令已插入', command)
  } catch {
    showToast('插入失败', '当前终端尚未准备完成，稍后再试。')
  }
}

async function executePaneText(pane: PaneNode, text: string) {
  const command = text.trim()
  if (!command) return

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast('终端未就绪', '当前 Pane 还没有可执行的终端会话。')
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, `${command}\r`)
    recordSessionCommand(sessionId, command)
    showToast('命令已执行', command)
  } catch {
    showToast('执行失败', '当前终端尚未准备完成，稍后再试。')
  }
}

async function copyCommandText(command: string) {
  const value = command.trim()
  if (!value) return

  try {
    await navigator.clipboard.writeText(value)
    showToast('命令已复制', value)
  } catch {
    showToast('复制失败', '当前环境未允许写入剪贴板。')
  }
}

async function insertCommandToActivePane(command: string) {
  const value = command.trim()
  if (!value) return
  if (!(appSection.value === 'workspace' && workspaceView.value === 'runtime')) {
    await copyCommandText(value)
    showToast('已先复制命令', '请先进入运行态并选中一个终端，再回填到当前输入框。')
    return
  }
  const pane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, activeRuntimePaneId.value) : null

  if (!pane) {
    await copyCommandText(value)
    showToast('已先复制命令', '当前没有选中终端。进入运行态后可手动粘贴。')
    return
  }

  await insertPaneText(pane, value)
}

async function insertRecentCommand(item: RecentItem) {
  const value = item.command?.trim() || ''
  if (!value) return

  if (appSection.value === 'workspace' && workspaceView.value === 'runtime' && activeRuntimePaneId.value) {
    await insertCommandToActivePane(value)
    return
  }

  await copyCommandText(value)
  showToast('已先复制命令', '当前没有选中终端。若要回填，请先进入运行态并选中一个终端。')
}

function commandEntryForPane(pane: PaneNode) {
  const session = activePaneSession(pane)
  return entryById(session?.terminalEntryId ?? pane.terminalEntryId)
}

function uniqueCommandList(commands: string[], limit = 8) {
  return Array.from(new Set(commands.map((command) => command.trim()).filter(Boolean))).slice(0, limit)
}

function workspaceRecentCommands(limit = 10) {
  return uniqueCommandList(
    selectedWorkspaceEntries.value.flatMap((entry) => [
      entry.lastCommand,
      ...(entry.commandHistory ?? []),
      entry.defaultCommand,
    ]),
    limit,
  )
}

function commandPanelSections(pane: PaneNode) {
  const entry = commandEntryForPane(pane)
  const used = new Set<string>()
  const sections: Array<{ key: string; title: string; commands: string[]; favoriteScope: boolean }> = []
  const pushSection = (key: string, title: string, commands: string[], favoriteScope = true) => {
    const nextCommands = uniqueCommandList(commands, 8).filter((command) => {
      if (used.has(command)) return false
      used.add(command)
      return true
    })
    if (nextCommands.length) {
      sections.push({ key, title, commands: nextCommands, favoriteScope })
    }
  }

  if (entry) {
    pushSection('default', '默认命令', [entry.defaultCommand])
    pushSection('last', '最后命令', [entry.lastCommand])
    pushSection('favorite', '收藏命令', entry.favoriteCommands ?? [])
    pushSection('history', '最近命令', entry.commandHistory ?? [])
  }

  pushSection('workspace', entry ? '工作区最近' : '最近命令', workspaceRecentCommands(), Boolean(entry))
  return sections
}

function isFavoriteCommand(pane: PaneNode, command: string) {
  const entry = commandEntryForPane(pane)
  if (!entry) return false
  return (entry.favoriteCommands ?? []).some((item) => item.trim() === command.trim())
}

function toggleFavoriteCommand(pane: PaneNode, command: string) {
  const value = command.trim()
  const entry = commandEntryForPane(pane)
  if (!entry || !value) {
    showToast('无法收藏命令', '请先给当前终端绑定一个运行配置。')
    return
  }

  const now = new Date().toISOString()
  const exists = isFavoriteCommand(pane, value)
  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    terminalEntries: workspace.terminalEntries.map((item) => {
      if (item.id !== entry.id) return item
      const nextFavorites = exists
        ? (item.favoriteCommands ?? []).filter((favorite) => favorite.trim() !== value)
        : [value, ...(item.favoriteCommands ?? []).filter((favorite) => favorite.trim() !== value)].slice(0, 12)

      return {
        ...item,
        favoriteCommands: nextFavorites,
        commandHistory: [value, ...(item.commandHistory ?? []).filter((history) => history.trim() !== value)].slice(0, 20),
        updatedAt: now,
      }
    }),
    updatedAt: now,
  }))

  showToast(exists ? '已取消收藏' : '命令已收藏', value)
}

function parseProviderToolTargets(value: string): ProviderToolTarget[] {
  return Array.from(new Set(
    value
      .split(/[，,\s]+/)
      .map((item) => item.trim().toLowerCase())
      .filter(Boolean)
      .map((item) => {
        if (item === 'claude' || item === 'codex' || item === 'gemini' || item === 'opencode' || item === 'generic') {
          return item
        }
        return 'generic'
      }),
  )) as ProviderToolTarget[]
}

function workspaceProviderById(workspace: WorkspaceCard | undefined, providerId?: string | null) {
  return workspace?.providerProfiles?.find((provider) => provider.id === providerId) ?? null
}

function selectedWorkspaceProviderById(providerId?: string | null) {
  return workspaceProviderById(selectedWorkspace.value, providerId)
}

function defaultWorkspaceProvider(workspace: WorkspaceCard | undefined = selectedWorkspace.value) {
  return workspace?.providerProfiles?.find((provider) => provider.isDefault) ?? workspace?.providerProfiles?.[0] ?? null
}

function runtimeProfileResolvedProvider(entry: TerminalEntry | undefined | null, workspace: WorkspaceCard | undefined = selectedWorkspace.value) {
  if (!entry) return null
  if (entry.providerBindingMode === 'disabled') return null
  if (entry.providerBindingMode === 'explicit') {
    return workspaceProviderById(workspace, entry.providerProfileId)
  }
  return defaultWorkspaceProvider(workspace)
}

function providerBindingModeLabel(mode?: TerminalEntry['providerBindingMode']) {
  if (mode === 'explicit') return '显式指定'
  if (mode === 'disabled') return '不注入'
  return '继承默认'
}

function providerSummaryLabel(entry: TerminalEntry | undefined | null, workspace: WorkspaceCard | undefined = selectedWorkspace.value) {
  if (!entry) return '未绑定运行配置'
  if (entry.providerBindingMode === 'disabled') return '不注入 Provider'
  const provider = runtimeProfileResolvedProvider(entry, workspace)
  if (!provider) return entry.providerBindingMode === 'explicit' ? '未选择 Provider' : '继承但未设置默认 Provider'
  return provider.name
}

function removeTab(tabId: string) {
  if (!selectedWorkspace.value) return

  if (selectedWorkspace.value.tabs.length <= 1) {
    showToast('无法删除标签页', '工作区至少需要保留一个 Tab。')
    return
  }

  const tab = selectedWorkspace.value.tabs.find((item) => item.id === tabId)
  if (!tab) return

  requestConfirm({
    title: `删除标签页「${tab.name}」`,
    description: '该操作会移除当前 Tab 及其下所有 Pane 布局。',
    confirmLabel: '确认删除',
    action: () => {
      const now = new Date().toISOString()
      const sessionIds = collectSessionIdsFromPanes(tab.panes)
      patchSelectedWorkspace((workspace) => {
        const nextTabs = workspace.tabs
          .filter((item) => item.id !== tabId)
          .map((item, index) => ({ ...item, order: index }))

        return {
          ...workspace,
          tabs: nextTabs,
          terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
          updatedAt: now,
        }
      })

      if (activeRuntimeTabId.value === tabId) {
        const fallbackTab = selectedWorkspace.value?.tabs.find((item) => item.id !== tabId)
        activeRuntimeTabId.value = fallbackTab?.id ?? ''
      }

      const nextRuntimeActiveSessionIds = { ...runtimeActiveSessionIds.value }
      flattenLeafPanes(tab.panes).forEach((pane) => {
        delete nextRuntimeActiveSessionIds[pane.id]
      })
      runtimeActiveSessionIds.value = nextRuntimeActiveSessionIds

      sessionIds.forEach((sessionId) => {
        void destroyTerminalRuntime(sessionId).catch(() => undefined)
      })

      showToast('标签页已删除', `已移除：${tab.name}`)
    },
  })
}

function requestConfirm(options: {
  title: string
  description: string
  confirmLabel?: string
  variant?: 'primary' | 'danger'
  details?: string[]
  action: () => void
}) {
  confirmDialog.title = options.title
  confirmDialog.description = options.description
  confirmDialog.confirmLabel = options.confirmLabel ?? '确认'
  confirmDialog.variant = options.variant ?? 'danger'
  confirmDialog.details = options.details ?? []
  confirmAction = options.action
  openConfirmModal.value = true
}

function closeConfirmModal() {
  openConfirmModal.value = false
  confirmAction = null
  confirmDialog.details = []
}

function loadWorkbenchSidebarWidth() {
  if (typeof window === 'undefined') return 320
  const raw = window.localStorage.getItem('chuchen-terminal.workbench-sidebar-width')
  const parsed = raw ? Number(raw) : NaN
  return Number.isFinite(parsed) ? Math.min(480, Math.max(260, parsed)) : 320
}

function saveWorkbenchSidebarWidth(width: number) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem('chuchen-terminal.workbench-sidebar-width', String(width))
}

async function pickDirectory(defaultPath?: string) {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    showToast('仅桌面端可用', '当前目录选择器只在 Tauri 桌面端可用。')
    return null
  }

  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: defaultPath?.trim() || undefined,
    })
    return typeof selected === 'string' ? selected : null
  } catch {
    showToast('目录选择失败', '未能打开系统目录选择框。')
    return null
  }
}

async function pickWorkspaceRootPath() {
  const selected = await pickDirectory(workspaceForm.rootPath || selectedWorkspace.value?.rootPath)
  if (selected) {
    workspaceForm.rootPath = selected
  }
}

async function pickTerminalEntryWorkingDirectory() {
  const selected = await pickDirectory(terminalEntryForm.workingDirectory || selectedWorkspace.value?.rootPath)
  if (selected) {
    terminalEntryForm.workingDirectory = selected
  }
}

function loadUiPreferences() {
  if (typeof window === 'undefined') return

  const themeId = window.localStorage.getItem('chuchen-terminal.theme-id')
  const accentHex = window.localStorage.getItem('chuchen-terminal.accent-hex')
  const fontSize = window.localStorage.getItem('chuchen-terminal.font-size')
  const fontFamily = window.localStorage.getItem('chuchen-terminal.font-family')
  const refreshInterval = window.localStorage.getItem('chuchen-terminal.system-refresh-interval')
  const commandRestoreStrategy = window.localStorage.getItem('chuchen-terminal.restore-command-strategy')
  const hiddenItems = window.localStorage.getItem('chuchen-terminal.hidden-environment-items')
  const railCollapsedValue = window.localStorage.getItem('chuchen-terminal.rail-collapsed')
  const notificationsEnabledValue = window.localStorage.getItem('chuchen-terminal.notifications-enabled')
  const windowAttentionEnabledValue = window.localStorage.getItem('chuchen-terminal.window-attention-enabled')
  const pinnedRecentItems = window.localStorage.getItem('chuchen-terminal.pinned-recent-items')
  const hiddenRecentItems = window.localStorage.getItem('chuchen-terminal.hidden-recent-items')

  if (themeId && themePresets.some((theme) => theme.id === themeId)) {
    activeThemeId.value = themeId
  }

  if (accentHex) {
    customAccentHex.value = normalizeHex(accentHex, customAccentHex.value)
  }

  const parsedFontSize = fontSize ? Number(fontSize) : NaN
  if (Number.isFinite(parsedFontSize)) {
    terminalFontSize.value = clamp(parsedFontSize, 8, 24)
  }

  if (fontFamily) {
    terminalFontFamily.value = fontFamily
  }

  if (refreshInterval === 'manual' || refreshInterval === '5s' || refreshInterval === '10s' || refreshInterval === '30s') {
    systemRefreshInterval.value = refreshInterval
  }

  if (commandRestoreStrategy === 'layout-only' || commandRestoreStrategy === 'prefill' || commandRestoreStrategy === 'execute') {
    restoreCommandStrategy.value = commandRestoreStrategy
  }

  if (hiddenItems) {
    try {
      const parsed = JSON.parse(hiddenItems)
      if (Array.isArray(parsed)) {
        hiddenEnvironmentItems.value = parsed.filter((item): item is string => typeof item === 'string')
      }
    } catch {
    }
  }

  if (railCollapsedValue === '1' || railCollapsedValue === '0') {
    railCollapsed.value = railCollapsedValue === '1'
  }

  if (notificationsEnabledValue === '1' || notificationsEnabledValue === '0') {
    systemNotificationsEnabled.value = notificationsEnabledValue === '1'
  }

  if (windowAttentionEnabledValue === '1' || windowAttentionEnabledValue === '0') {
    windowAttentionEnabled.value = windowAttentionEnabledValue === '1'
  }

  if (pinnedRecentItems) {
    try {
      const parsed = JSON.parse(pinnedRecentItems)
      if (Array.isArray(parsed)) {
        pinnedRecentItemIds.value = parsed.filter((item): item is string => typeof item === 'string')
      }
    } catch {
    }
  }

  if (hiddenRecentItems) {
    try {
      const parsed = JSON.parse(hiddenRecentItems)
      if (Array.isArray(parsed)) {
        hiddenRecentItemIds.value = parsed.filter((item): item is string => typeof item === 'string')
      }
    } catch {
    }
  }
}

function saveUiPreferences() {
  if (typeof window === 'undefined') return
  window.localStorage.setItem('chuchen-terminal.theme-id', activeThemeId.value)
  window.localStorage.setItem('chuchen-terminal.accent-hex', customAccentHex.value)
  window.localStorage.setItem('chuchen-terminal.font-size', String(terminalFontSize.value))
  window.localStorage.setItem('chuchen-terminal.font-family', terminalFontFamily.value)
  window.localStorage.setItem('chuchen-terminal.system-refresh-interval', systemRefreshInterval.value)
  window.localStorage.setItem('chuchen-terminal.restore-command-strategy', restoreCommandStrategy.value)
  window.localStorage.setItem('chuchen-terminal.hidden-environment-items', JSON.stringify(hiddenEnvironmentItems.value))
  window.localStorage.setItem('chuchen-terminal.rail-collapsed', railCollapsed.value ? '1' : '0')
  window.localStorage.setItem('chuchen-terminal.notifications-enabled', systemNotificationsEnabled.value ? '1' : '0')
  window.localStorage.setItem('chuchen-terminal.window-attention-enabled', windowAttentionEnabled.value ? '1' : '0')
  window.localStorage.setItem('chuchen-terminal.pinned-recent-items', JSON.stringify(pinnedRecentItemIds.value))
  window.localStorage.setItem('chuchen-terminal.hidden-recent-items', JSON.stringify(hiddenRecentItemIds.value))
}

function queueSaveUiPreferences() {
  if (saveUiPreferencesTimer) {
    clearTimeout(saveUiPreferencesTimer)
  }
  saveUiPreferencesTimer = setTimeout(() => {
    saveUiPreferencesTimer = null
    saveUiPreferences()
  }, 160)
}

async function refreshTauriViewportWidth() {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return
  try {
    const size = await getCurrentWindow().innerSize()
    tauriViewportWidth.value = size.width || null
  } catch {
  }
}

function loadDiagnosticsCache() {
  if (typeof window === 'undefined') return

  try {
    const rawSystemStatus = window.localStorage.getItem(SYSTEM_STATUS_CACHE_KEY)
    if (rawSystemStatus) {
      const parsed = JSON.parse(rawSystemStatus) as Partial<typeof systemStatus> & { updatedAt?: number }
      systemStatus.cpu = parsed.cpu || systemStatus.cpu
      systemStatus.memory = parsed.memory || systemStatus.memory
      systemStatus.gpu = parsed.gpu || systemStatus.gpu
      hasSystemStatusCache.value = Boolean(parsed.cpu || parsed.memory || parsed.gpu)
      systemStatusCacheUpdatedAt.value = typeof parsed.updatedAt === 'number' ? parsed.updatedAt : Date.now()
    }
  } catch {
  }

  try {
    const rawEnvironmentChecks = window.localStorage.getItem(ENVIRONMENT_CHECK_CACHE_KEY)
    if (!rawEnvironmentChecks) return

    const parsedPayload = JSON.parse(rawEnvironmentChecks) as
      | Array<{ name: string; value: string; status: 'ok' | 'pending' }>
      | { updatedAt?: number; items?: Array<{ name: string; value: string; status: 'ok' | 'pending' }> }
    const parsed = Array.isArray(parsedPayload) ? parsedPayload : (parsedPayload.items || [])
    hasEnvironmentCheckCache.value = parsed.length > 0
    environmentCheckCacheUpdatedAt.value = Array.isArray(parsedPayload)
      ? Date.now()
      : typeof parsedPayload.updatedAt === 'number'
        ? parsedPayload.updatedAt
        : Date.now()
    parsed.forEach((cachedItem) => {
      const target = homeEnvironmentChecks.find((item) => item.name === cachedItem.name)
      if (!target) return
      target.value = cachedItem.value
      target.status = cachedItem.status
    })
  } catch {
  }
}

function persistSystemStatusCache() {
  if (typeof window === 'undefined') return
  const updatedAt = Date.now()
  hasSystemStatusCache.value = true
  systemStatusCacheUpdatedAt.value = updatedAt
  window.localStorage.setItem(SYSTEM_STATUS_CACHE_KEY, JSON.stringify({
    cpu: systemStatus.cpu,
    memory: systemStatus.memory,
    gpu: systemStatus.gpu,
    updatedAt,
  }))
}

function persistEnvironmentCheckCache() {
  if (typeof window === 'undefined') return
  const updatedAt = Date.now()
  hasEnvironmentCheckCache.value = true
  environmentCheckCacheUpdatedAt.value = updatedAt
  window.localStorage.setItem(ENVIRONMENT_CHECK_CACHE_KEY, JSON.stringify(
    {
      updatedAt,
      items: homeEnvironmentChecks
        .filter((item) => TAURI_ENVIRONMENT_CHECK_NAMES.includes(item.name as typeof TAURI_ENVIRONMENT_CHECK_NAMES[number]))
        .map((item) => ({
          name: item.name,
          value: item.value,
          status: item.status,
        })),
    },
  ))
}

function delay(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms))
}

function isSystemStatusCacheStale() {
  return !hasSystemStatusCache.value || (Date.now() - systemStatusCacheUpdatedAt.value) > SYSTEM_STATUS_CACHE_TTL_MS
}

function isEnvironmentCheckCacheStale() {
  return !hasEnvironmentCheckCache.value || (Date.now() - environmentCheckCacheUpdatedAt.value) > ENVIRONMENT_CHECK_CACHE_TTL_MS
}

function scheduleIdleRefresh(task: () => void, delayMs: number) {
  if (typeof window === 'undefined') return

  window.setTimeout(() => {
    if ('requestIdleCallback' in window) {
      ;(window as Window & { requestIdleCallback: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number }).requestIdleCallback(() => task(), { timeout: 1200 })
      return
    }
    task()
  }, delayMs)
}

function queueInitialDiagnostics() {
  if (typeof window === 'undefined') return

  if (isSystemStatusCacheStale()) {
    scheduleIdleRefresh(() => {
      void refreshSystemStatus()
    }, hasSystemStatusCache.value ? INITIAL_SYSTEM_CACHE_REFRESH_DELAY_MS : INITIAL_SYSTEM_REFRESH_DELAY_MS)
  }

  if (isEnvironmentCheckCacheStale()) {
    scheduleIdleRefresh(() => {
      void refreshEnvironmentChecks()
    }, hasEnvironmentCheckCache.value ? INITIAL_ENVIRONMENT_CACHE_REFRESH_DELAY_MS : INITIAL_ENVIRONMENT_REFRESH_DELAY_MS)
  }
}

async function refreshSystemStatus() {
  if (typeof window === 'undefined') return

  if (systemStatusRefreshing.value) return
  systemStatusRefreshing.value = true

  if (!("__TAURI_INTERNALS__" in window)) {
    systemStatus.cpu = '浏览器模式'
    systemStatus.memory = '浏览器模式'
    systemStatus.gpu = '浏览器模式'
    systemStatusRefreshing.value = false
    return
  }

  try {
    const status = await invoke<{ cpu: string; memory: string; gpu: string }>('read_system_status')
    systemStatus.cpu = status.cpu
    systemStatus.memory = status.memory
    systemStatus.gpu = status.gpu
    startupPerf.systemStatusResolvedAt = performance.now()
    persistSystemStatusCache()
  } catch {
    systemStatus.cpu = '读取失败'
    systemStatus.memory = '读取失败'
    systemStatus.gpu = '读取失败'
  } finally {
    systemStatusRefreshing.value = false
  }
}

function scheduleSystemRefresh() {
  if (systemRefreshTimer) {
    window.clearInterval(systemRefreshTimer)
    systemRefreshTimer = null
  }
  if (systemRefreshTickTimer) {
    window.clearInterval(systemRefreshTickTimer)
    systemRefreshTickTimer = null
  }

  if (systemRefreshInterval.value === 'manual') {
    systemRefreshCountdown.value = 0
    return
  }

  const intervalMs = systemRefreshInterval.value === '5s'
    ? 5000
    : systemRefreshInterval.value === '10s'
      ? 10000
      : 30000
  const intervalSeconds = Math.ceil(intervalMs / 1000)

  systemRefreshTimer = window.setInterval(() => {
    systemRefreshCountdown.value = intervalSeconds
    void refreshSystemStatus()
  }, intervalMs)
  systemRefreshCountdown.value = intervalSeconds
  systemRefreshTickTimer = window.setInterval(() => {
    systemRefreshCountdown.value = systemRefreshCountdown.value <= 1
      ? intervalSeconds
      : systemRefreshCountdown.value - 1
  }, 1000)
}

async function refreshEnvironmentChecks(force = false) {
  if (typeof window === 'undefined' || !("__TAURI_INTERNALS__" in window)) {
    return
  }
  if (environmentChecksRefreshing.value && !force) return

  const runId = ++environmentRefreshRunId
  environmentChecksRefreshing.value = true

  for (const name of TAURI_ENVIRONMENT_CHECK_NAMES) {
    if (runId !== environmentRefreshRunId) {
      environmentChecksRefreshing.value = false
      return
    }

    const target = homeEnvironmentChecks.find((item) => item.name === name)
    if (!target) continue

    if (target.value === '待检测' || target.value === '检测失败') {
      target.value = '检测中'
      target.status = 'pending'
    }

    try {
      const nextItem = await invoke<{ name: string; value: string; status: string }>('read_environment_check', { name })
      if (runId !== environmentRefreshRunId) {
        environmentChecksRefreshing.value = false
        return
      }
      target.value = nextItem.value
      target.status = nextItem.status as 'ok' | 'pending'
    } catch {
      target.value = '检测失败'
      target.status = 'pending'
    }

    await delay(ENVIRONMENT_CHECK_GAP_MS)
  }

  startupPerf.environmentResolvedAt = performance.now()
  persistEnvironmentCheckCache()
  environmentChecksRefreshing.value = false
}

function startWorkbenchResize(event: PointerEvent) {
  event.preventDefault()
  event.stopPropagation()
  ;(event.currentTarget as HTMLElement | null)?.setPointerCapture?.(event.pointerId)
  const startX = event.clientX
  const startWidth = workbenchSidebarWidth.value

  const handleMove = (moveEvent: PointerEvent) => {
    const next = startWidth + (moveEvent.clientX - startX)
    workbenchSidebarWidth.value = Math.min(480, Math.max(260, next))
  }

  const handleUp = () => {
    saveWorkbenchSidebarWidth(workbenchSidebarWidth.value)
    window.removeEventListener('pointermove', handleMove)
    window.removeEventListener('pointerup', handleUp)
    document.body.classList.remove('is-resizing-workbench')
    workbenchResizeCleanup = null
  }

  document.body.classList.add('is-resizing-workbench')
  window.addEventListener('pointermove', handleMove)
  window.addEventListener('pointerup', handleUp)
  workbenchResizeCleanup = handleUp
}

function submitConfirmModal() {
  const nextAction = confirmAction
  openConfirmModal.value = false
  confirmAction = null
  nextAction?.()
}

function patchSelectedWorkspace(updater: (workspace: WorkspaceCard) => WorkspaceCard) {
  if (!selectedWorkspace.value) return

  commitWorkspaces((current) => current.map((workspace) =>
    workspace.id === selectedWorkspace.value?.id ? updater(workspace) : workspace,
  ))
}

function parseTags(value: string) {
  return value
    .split(/[，,\s]+/)
    .map((item) => item.trim())
    .filter(Boolean)
}

function totalPanes(workspace: WorkspaceCard) {
  return workspace.tabs.reduce((count, tab) => count + countLeafPanes(tab.panes), 0)
}

function totalWorkspaceSessions(workspace: WorkspaceCard) {
  return workspace.tabs.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)
}

function explorerSessionItems(workspace: WorkspaceCard, tab: WorkspaceTab) {
  return flattenLeafPanes(tab.panes).flatMap((pane) =>
    paneSessions(pane).map((session) => ({ pane, session })),
  )
}

function workspacePaneTagCount(workspace: WorkspaceCard) {
  const tagSet = new Set<string>()

  workspace.tabs.forEach((tab) => {
    flattenLeafPanes(tab.panes).forEach((pane) => {
      workspaceEntryById(workspace, pane.terminalEntryId)?.tags.forEach((tag) => tagSet.add(tag))
    })
  })

  return tagSet.size
}

function workspaceEntryById(workspace: WorkspaceCard, entryId?: string | null) {
  return workspace.terminalEntries.find((entry) => entry.id === entryId)
}

function entryUsageCount(entryId: string) {
  if (!selectedWorkspace.value) return 0

  return selectedWorkspace.value.tabs.reduce((count, tab) => {
    return count + flattenLeafPanes(tab.panes).filter((pane) => pane.terminalEntryId === entryId).length
  }, 0)
}

function runningCount(workspace: WorkspaceCard) {
  return workspace.tabs.reduce((count, tab) => count + tabRunningCount(workspace, tab), 0)
}

async function refreshWorkspaceGitInfo(workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace?.rootPath?.trim()) return

  if (typeof window === 'undefined' || !("__TAURI_INTERNALS__" in window)) {
    return
  }

  try {
    const info = await invoke<{ isRepo: boolean; branch?: string | null }>('read_workspace_git_info', { path: workspace.rootPath })
    const nextBranch = info.isRepo ? (info.branch?.trim() || 'HEAD') : '非 Git 仓库'
    if ((workspace.gitBranch?.trim() || '') === nextBranch) return

    commitWorkspaces((current) => current.map((item) =>
      item.id === workspaceId
        ? {
            ...item,
            gitBranch: nextBranch,
          }
        : item,
    ), 'transient')
  } catch {
    const nextBranch = workspace.gitBranch?.trim() || '未检测分支'
    if ((workspace.gitBranch?.trim() || '') === nextBranch) return

    commitWorkspaces((current) => current.map((item) =>
      item.id === workspaceId
        ? {
            ...item,
            gitBranch: nextBranch,
          }
        : item,
    ), 'transient')
  }
}

function workspaceGitBranchLabel(workspace: WorkspaceCard) {
  return workspace.gitBranch?.trim() || '未检测分支'
}

function tabRunningCount(workspace: WorkspaceCard, tab: WorkspaceTab) {
  return flattenLeafPanes(tab.panes).filter((pane) => workspacePaneRunning(workspace, pane)).length
}

function flattenLeafPanes(panes: PaneNode[]): PaneNode[] {
  const result: PaneNode[] = []
  const visit = (pane: PaneNode) => {
    if (pane.children?.length) {
      pane.children.forEach((child) => visit(child))
      return
    }
    result.push(pane)
  }
  panes.forEach((pane) => visit(pane))
  return result
}

function collectSessionIdsFromPanes(panes: PaneNode[]) {
  return flattenLeafPanes(panes).flatMap((pane) => paneSessions(pane).map((session) => session.id))
}

function countLeafPanes(panes: PaneNode[]) {
  return flattenLeafPanes(panes).length
}

function countPaneSessions(panes: PaneNode[]) {
  return flattenLeafPanes(panes).reduce((count, pane) => count + paneSessions(pane).length, 0)
}

function entryById(entryId?: string | null) {
  return selectedWorkspace.value?.terminalEntries.find((entry) => entry.id === entryId)
}

function sessionById(workspace: WorkspaceCard | undefined, sessionId: string) {
  if (!workspace) return null
  for (const tab of workspace.tabs) {
    for (const pane of flattenLeafPanes(tab.panes)) {
      const session = paneSessions(pane).find((item) => item.id === sessionId)
      if (session) {
        return { tab, pane, session }
      }
    }
  }
  return null
}

function locateSessionAcrossWorkspaces(sessionId: string) {
  for (const workspace of workspaces.value) {
    const located = sessionById(workspace, sessionId)
    if (located) {
      return { workspace, ...located }
    }
  }
  return null
}

function applyTerminalEntryStatuses(entries: TerminalEntry[], tabs: WorkspaceTab[]) {
  return entries.map((entry) => {
    const running = tabs.some((tab) =>
      flattenLeafPanes(tab.panes).some((pane) =>
        paneSessions(pane).some((session) => session.terminalEntryId === entry.id && session.status === 'running'),
      ),
    )

    if ((entry.status === 'running') === running) {
      return entry
    }

    return {
      ...entry,
      status: (running ? 'running' : 'idle') as TerminalEntry['status'],
    }
  })
}

function paneHasRunningSession(pane: PaneNode) {
  return paneSessions(pane).some((session) => session.status === 'running')
}

function sessionAttentionState(session: PaneTerminalSession) {
  if (session.supervisorState === 'needs-human') return 'needs-input'
  if (session.supervisorState === 'stalled') return 'stalled'
  if (session.supervisorState === 'completed') return 'completed'
  if (session.status === 'running') {
    return session.hasUserCommand ? 'running' : 'fresh'
  }
  if (session.lastExitCode != null) {
    if (session.lastExitCode !== 0) return 'error'
    return session.hasUserCommand ? 'completed' : 'idle'
  }
  if (session.expectedDoneSignal?.trim() && session.hasUserCommand) return 'waiting'
  if (!session.hasUserCommand) return 'fresh'
  return 'idle'
}

function explorerSessionTone(session: PaneTerminalSession) {
  const state = sessionAttentionState(session)
  if (state === 'running') return 'running'
  if (state === 'completed') return 'completed'
  if (state === 'error') return 'error'
  if (state === 'waiting' || state === 'needs-input' || state === 'stalled') return 'waiting'
  return 'idle'
}

function explorerSessionLabel(session: PaneTerminalSession) {
  const state = sessionAttentionState(session)
  if (state === 'fresh') return '未开始'
  if (state === 'running') return '运行中'
  if (state === 'needs-input') return '等待输入'
  if (state === 'waiting') return '等待中'
  if (state === 'stalled') return '疑似停滞'
  if (state === 'completed') return '已完成'
  if (state === 'error') return '异常退出'
  return '空闲'
}

function attentionPriority(state: SessionAttentionState) {
  if (state === 'error') return 6
  if (state === 'needs-input') return 5
  if (state === 'stalled') return 4
  if (state === 'completed') return 3
  if (state === 'running') return 2
  if (state === 'waiting') return 1
  return 0
}

function explorerProjectAttentionState(workspace: WorkspaceCard, tab: WorkspaceTab): SessionAttentionState {
  let best: SessionAttentionState = 'idle'
  for (const item of explorerSessionItems(workspace, tab)) {
    const state = sessionAttentionState(item.session)
    if (attentionPriority(state) > attentionPriority(best)) {
      best = state
    }
  }
  return best
}

function explorerProjectAttentionClass(workspace: WorkspaceCard, tab: WorkspaceTab) {
  const state = explorerProjectAttentionState(workspace, tab)
  if (state === 'error') return 'explorer-project--attention-error'
  if (state === 'needs-input' || state === 'stalled') return 'explorer-project--attention-waiting'
  if (state === 'completed') return 'explorer-project--attention-completed'
  return ''
}

function explorerProjectAttentionBadge(workspace: WorkspaceCard, tab: WorkspaceTab) {
  const state = explorerProjectAttentionState(workspace, tab)
  if (state === 'error') return '异常'
  if (state === 'needs-input') return '待处理'
  if (state === 'stalled') return '停滞'
  if (state === 'completed') return '完成'
  return ''
}

function sessionIsSupervised(session: PaneTerminalSession) {
  return session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume'
}

function explorerSessionAttentionClass(session: PaneTerminalSession) {
  const state = sessionAttentionState(session)
  if (state === 'error') return 'explorer-pane--attention-error'
  if (state === 'needs-input' || state === 'stalled') return 'explorer-pane--attention-waiting'
  if (state === 'completed') return 'explorer-pane--attention-completed'
  if (state === 'fresh') return 'explorer-pane--attention-fresh'
  return ''
}

function shouldNotifyAttentionState(state: SessionAttentionState) {
  return state === 'needs-input' || state === 'stalled' || state === 'completed' || state === 'error'
}

function shouldCountAttentionState(state: SessionAttentionState) {
  return state === 'needs-input' || state === 'stalled' || state === 'completed' || state === 'error'
}

const attentionSessionCount = computed(() => {
  let count = 0
  for (const workspace of workspaces.value) {
    for (const tab of workspace.tabs) {
      for (const pane of flattenLeafPanes(tab.panes)) {
        for (const session of paneSessions(pane)) {
          if (shouldCountAttentionState(sessionAttentionState(session))) {
            count += 1
          }
        }
      }
    }
  }
  return count
})

const currentActiveRuntimeSessionMeta = computed(() => {
  const sessionId = currentActiveRuntimeSessionId()
  if (!sessionId) return null
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return null
  return {
    workspaceName: located.workspace.name,
    tabName: located.tab.name,
    sessionName: located.session.name,
    sessionId: located.session.id,
  }
})

function resolveAttentionOverlayIconPath(count: number) {
  if (count <= 0) return undefined
  const label = count > 9 ? '99+' : String(count)
  return `/overlay-badges/attention-${label}.png`
}

async function applyWindowAttentionBadge(force = false) {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return

  if (!windowAttentionEnabled.value) {
    lastAttentionBadgeCount = attentionSessionCount.value
    const currentWindow = getCurrentWindow()
    try {
      await currentWindow.setBadgeCount(undefined)
    } catch {
    }
    try {
      await currentWindow.setOverlayIcon(undefined)
    } catch {
    }
    return
  }

  const count = attentionSessionCount.value
  if (!force && count === lastAttentionBadgeCount) return

  const previous = lastAttentionBadgeCount
  lastAttentionBadgeCount = count

  const currentWindow = getCurrentWindow()

  try {
    await currentWindow.setBadgeCount(count > 0 ? count : undefined)
  } catch {
  }

  try {
    if (count > 0) {
      const overlay = resolveAttentionOverlayIconPath(count)
      await currentWindow.setOverlayIcon(overlay)
    } else {
      await currentWindow.setOverlayIcon(undefined)
    }
  } catch {
  }

  if (count > 0 && count > previous) {
    try {
      await currentWindow.requestUserAttention(UserAttentionType.Informational)
    } catch {
    }
  }
}

function setSessionSupervisorMode(sessionId: string, mode: 'off' | 'watch') {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const now = new Date().toISOString()

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) =>
                  session.id === sessionId
                    ? {
                        ...session,
                        supervisorMode: mode,
                        supervisorState: mode === 'watch'
                          ? session.status === 'running'
                            ? 'watching'
                            : 'idle'
                          : 'idle',
                        supervisorNote: mode === 'watch'
                          ? '已开启任务监督：会检测完成、异常退出和疑似停滞。'
                          : null,
                        lastHeartbeatAt: session.lastHeartbeatAt ?? now,
                        lastActivityAt: session.lastActivityAt ?? now,
                      }
                    : session,
                ),
              }
            }),
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
    }
  }), 'transient')

  showToast(mode === 'watch' ? '任务监督已开启' : '任务监督已关闭', `${located.workspace.name} / ${located.tab.name} / ${located.session.name}`)
}

function clearSessionAttention(sessionId: string, options: { silent?: boolean } = {}) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const now = new Date().toISOString()

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) =>
                  session.id === sessionId
                    ? {
                        ...session,
                        status: session.status === 'running' ? 'running' : 'idle',
                        lastExitCode: null,
                        supervisorState: session.status === 'running' && (session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume') ? 'watching' : 'idle',
                        supervisorNote: null,
                        lastHeartbeatAt: now,
                        lastActivityAt: now,
                      }
                    : session,
                ),
              }
            }),
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
    }
  }), 'transient')

  sessionAttentionNotificationState.delete(sessionId)
  lastAttentionBadgeCount = -1
  void applyWindowAttentionBadge(true)
  if (!options.silent) {
    showToast('提醒状态已清除', `${located.workspace.name} / ${located.tab.name} / ${located.session.name}`)
  }
}

function startSupervisorScan() {
  if (typeof window === 'undefined') return
  if (supervisorScanTimer) {
    window.clearInterval(supervisorScanTimer)
  }
  supervisorScanTimer = window.setInterval(() => {
    scanSupervisorSessions()
  }, SUPERVISOR_SCAN_INTERVAL_MS)
}

function scanSupervisorSessions() {
  const now = new Date()
  const nowMs = now.getTime()
  const nowIso = now.toISOString()

  commitWorkspaces((current) => {
    let didChange = false

    const nextWorkspaces = current.map((workspace) => {
      let workspaceChanged = false
      const nextTabs = workspace.tabs.map((tab) => ({
        ...tab,
        panes: visitPaneTree(tab.panes, (pane) => {
          let paneChanged = false
          const nextSessions = paneSessions(pane).map((session) => {
            const watched = session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume'
            if (!watched || session.status !== 'running' || !session.hasUserCommand) return session
            if (session.supervisorState === 'stalled' || session.supervisorState === 'needs-human') return session

            const activityAt = Date.parse(session.lastActivityAt || session.lastOutputAt || session.lastCommandAt || '')
            if (!Number.isFinite(activityAt) || nowMs - activityAt < SUPERVISOR_STALLED_THRESHOLD_MS) {
              return session
            }

            didChange = true
            workspaceChanged = true
            paneChanged = true
            return {
              ...session,
              supervisorState: 'stalled' as const,
              supervisorNote: '任务监督：超过 2 分钟未检测到终端输出，建议人工查看。',
              lastHeartbeatAt: nowIso,
            }
          })

          return paneChanged
            ? {
                ...pane,
                sessions: nextSessions,
              }
            : pane
        }),
      }))

      return workspaceChanged
        ? {
            ...workspace,
            tabs: nextTabs,
            terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
          }
        : workspace
    })

    return didChange ? nextWorkspaces : current
  }, 'transient')
}

function simulateSessionAttention(
  state: 'completed' | 'needs-input' | 'error' | 'stalled' | 'clear',
) {
  const sessionId = currentActiveRuntimeSessionId()
  const located = locateSessionAcrossWorkspaces(sessionId)

  if (!sessionId || !located) {
    showToast('提醒测试不可用', '请先选中一个终端会话。')
    return
  }

  const now = new Date().toISOString()

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) => {
                  if (session.id !== sessionId) return session

                  if (state === 'clear') {
                    return {
                      ...session,
                      status: 'idle',
                      hasUserCommand: false,
                      lastCommandAt: null,
                      lastOutputAt: null,
                      lastExitCode: null,
                      supervisorState: 'idle',
                      expectedDoneSignal: null,
                      lastHeartbeatAt: null,
                      lastActivityAt: null,
                      supervisorNote: null,
                    }
                  }

                  return {
                    ...session,
                    status: 'idle',
                    hasUserCommand: true,
                    lastCommandAt: now,
                    lastOutputAt: now,
                    lastExitCode: state === 'completed' ? 0 : state === 'error' ? 1 : null,
                    supervisorState: state === 'completed'
                      ? 'completed'
                      : state === 'error' || state === 'needs-input'
                        ? 'needs-human'
                        : 'stalled',
                    expectedDoneSignal: null,
                    lastHeartbeatAt: now,
                    lastActivityAt: now,
                    supervisorNote: state === 'completed'
                      ? '调试入口：模拟完成。'
                      : state === 'needs-input'
                        ? '调试入口：模拟等待输入。'
                        : state === 'error'
                          ? '调试入口：模拟异常退出。'
                          : '调试入口：模拟疑似停滞。',
                  }
                }),
              }
            }),
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
    }
  }), 'transient')

  showToast(
    state === 'clear' ? '提醒测试已清除' : '提醒测试已触发',
    `${located.workspace.name} / ${located.tab.name} / ${located.session.name}`,
  )

  lastAttentionBadgeCount = -1
  void applyWindowAttentionBadge(true)

  if (state !== 'clear') {
    try {
      if (windowAttentionEnabled.value && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
        void getCurrentWindow().requestUserAttention(UserAttentionType.Informational)
      }
    } catch {
    }

    if (systemNotificationsEnabled.value) {
      void sendSessionAttentionNotification({
        workspaceName: located.workspace.name,
        tabName: located.tab.name,
        sessionName: located.session.name,
        state,
      })
    }
  }
}

function queueSessionAttentionNotifications() {
  const visibleSessionIds = new Set<string>()

  for (const workspace of workspaces.value) {
    for (const tab of workspace.tabs) {
      for (const pane of flattenLeafPanes(tab.panes)) {
        for (const session of paneSessions(pane)) {
          const state = sessionAttentionState(session)
          visibleSessionIds.add(session.id)

          if (!shouldNotifyAttentionState(state)) {
            sessionAttentionNotificationState.delete(session.id)
            continue
          }

          const previous = sessionAttentionNotificationState.get(session.id)
          if (previous === state) continue

          sessionAttentionNotificationState.set(session.id, state)

          if (typeof document !== 'undefined' && document.visibilityState === 'visible' && document.hasFocus()) {
            continue
          }

          if (systemNotificationsEnabled.value) {
            void sendSessionAttentionNotification({
              workspaceName: workspace.name,
              tabName: tab.name,
              sessionName: session.name,
              state,
            })
          }
        }
      }
    }
  }

  Array.from(sessionAttentionNotificationState.keys()).forEach((sessionId) => {
    if (!visibleSessionIds.has(sessionId)) {
      sessionAttentionNotificationState.delete(sessionId)
    }
  })
}

function workspacePaneRunning(workspace: WorkspaceCard | undefined, pane: PaneNode) {
  if (!workspace) return false
  return paneSessions(pane).some((session) => session.status === 'running')
}

function syncSessionState(sessionId: string, status: 'idle' | 'running') {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  if (located.session.status === status) return
  const now = new Date().toISOString()

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    let didChange = false

    const nextTabs = workspace.tabs.map((tab) => {
      if (tab.id !== located.tab.id) return tab

      return {
        ...tab,
        panes: visitPaneTree(tab.panes, (pane) => {
          if (pane.id !== located.pane.id) return pane
          const nextSessions = paneSessions(pane).map((session) => {
            if (session.id !== sessionId) return session
            if (session.status === status) return session
            didChange = true
            return {
              ...session,
              status,
              lastHeartbeatAt: status === 'running' ? now : session.lastHeartbeatAt ?? now,
              lastActivityAt: status === 'running' ? now : session.lastActivityAt ?? now,
            }
          })

          return {
            ...pane,
            sessions: nextSessions,
          }
        }),
      }
    })

    if (!didChange) {
      return workspace
    }

    const nextEntries = applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs)

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: nextEntries,
    }
  }), 'transient')
}

function recordSessionCommand(sessionId: string, command: string) {
  const trimmed = command.trim()
  if (!trimmed) return

  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const entryId = located.session.terminalEntryId ?? located.pane.terminalEntryId ?? null
  const now = new Date().toISOString()
  sessionOutputTailBySession.set(sessionId, '')

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    return {
      ...workspace,
      tabs: workspace.tabs.map((tab) =>
        tab.id === located.tab.id
          ? {
              ...tab,
              panes: visitPaneTree(tab.panes, (pane) => {
                if (pane.id !== located.pane.id) return pane
                return {
                  ...pane,
                  sessions: paneSessions(pane).map((session) =>
                    session.id === sessionId
                    ? {
                        ...session,
                        status: 'running',
                        hasUserCommand: true,
                        lastCommandAt: now,
                        lastExitCode: null,
                        lastActivityAt: now,
                        lastHeartbeatAt: now,
                        supervisorState: session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume'
                          ? 'watching'
                          : 'idle',
                        supervisorNote: null,
                        }
                      : session,
                  ),
                }
              }),
            }
          : tab,
      ),
      terminalEntries: workspace.terminalEntries.map((entry) =>
        entry.id === entryId
          ? {
              ...entry,
              lastCommand: trimmed,
              commandHistory: [trimmed, ...(entry.commandHistory ?? []).filter((item) => item.trim() !== trimmed)].slice(0, 20),
              status: 'running',
              updatedAt: now,
            }
          : entry,
      ),
    }
  }), entryId ? 'persist' : 'transient')

  syncSessionState(sessionId, 'running')
}

function recordSessionOutput(sessionId: string, chunk: string) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located || !chunk.trim()) return

  const now = new Date().toISOString()
  const expected = located.session.expectedDoneSignal?.trim()
  const completed = Boolean(expected && chunk.includes(expected))
  const previousTail = sessionOutputTailBySession.get(sessionId) ?? ''
  const mergedTail = `${previousTail}${chunk}`.slice(-2048)
  sessionOutputTailBySession.set(sessionId, mergedTail)
  const promptReturned = /(^|\r?\n)PS [^\r\n]*>\s*$/.test(mergedTail)

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) =>
                  session.id === sessionId
                    ? {
                        ...session,
                        status: promptReturned ? 'idle' : session.status,
                        lastOutputAt: now,
                        lastActivityAt: now,
                        lastHeartbeatAt: now,
                        lastExitCode: completed ? 0 : session.lastExitCode ?? null,
                        supervisorState: completed
                          ? 'completed'
                          : promptReturned && session.hasUserCommand && (session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume')
                            ? 'completed'
                            : promptReturned && session.hasUserCommand
                              ? 'idle'
                            : session.supervisorState,
                        supervisorNote: completed || (promptReturned && session.hasUserCommand && (session.supervisorMode === 'watch' || session.supervisorMode === 'auto-resume'))
                          ? '任务监督：检测到终端回到可输入状态。'
                          : session.supervisorNote,
                      }
                    : session,
                ),
              }
            }),
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
    }
  }), 'transient')
}

function recordSessionExit(sessionId: string, exitCode: number) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const now = new Date().toISOString()
  sessionOutputTailBySession.delete(sessionId)

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    const nextTabs = workspace.tabs.map((tab) =>
      tab.id === located.tab.id
        ? {
            ...tab,
            panes: visitPaneTree(tab.panes, (pane) => {
              if (pane.id !== located.pane.id) return pane
              return {
                ...pane,
                sessions: paneSessions(pane).map((session) =>
                  session.id === sessionId
                    ? {
                        ...session,
                        status: 'idle',
                        lastExitCode: exitCode,
                        lastActivityAt: now,
                        lastHeartbeatAt: now,
                        supervisorState: exitCode === 0 && session.hasUserCommand
                          ? 'completed'
                          : exitCode !== 0
                            ? 'needs-human'
                            : session.supervisorState,
                      }
                    : session,
                ),
              }
            }),
          }
        : tab,
    )

    return {
      ...workspace,
      tabs: nextTabs,
      terminalEntries: applyTerminalEntryStatuses(workspace.terminalEntries, nextTabs),
    }
  }))
}

function launchModeLabel(mode?: TerminalEntry['launchMode']) {
  if (mode === 'open-only') return '仅打开窗口'
  if (mode === 'prefill') return '预填命令'
  if (mode === 'execute') return '立即执行'
  if (mode === 'switch-or-create') return '切换或创建'
  return '未设置'
}

function isSplitPane(pane: PaneNode) {
  return Boolean(pane.children?.length)
}

function paneChildren(pane: PaneNode) {
  return [...(pane.children || [])].sort((left, right) => left.order - right.order)
}

function paneContainerClass(pane: PaneNode) {
  if (pane.splitDirection === 'horizontal') return 'pane-tree pane-tree--horizontal'
  if (pane.splitDirection === 'vertical') return 'pane-tree pane-tree--vertical'
  return 'pane-tree'
}

const PaneTreeNode = defineComponent({
  name: 'PaneTreeNode',
  props: {
    pane: {
      type: Object as PropType<PaneNode>,
      required: true,
    },
  },
  setup(props) {
    return () => renderPaneTree(props.pane)
  },
})

function renderCommandPanel(pane: PaneNode): VNode {
  const entry = commandEntryForPane(pane)
  const sections = commandPanelSections(pane)

  return h('div', {
    class: 'command-panel command-panel--floating',
    onClick: (event: MouseEvent) => event.stopPropagation(),
    onPointerdown: (event: PointerEvent) => event.stopPropagation(),
  }, [
    h('div', { class: 'command-panel__header' }, [
      h('div', [
        h('strong', '快捷命令'),
        h('small', entry?.name || '工作区最近命令'),
      ]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: '关闭',
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          activeCommandPanelPaneId.value = null
        },
      }, [h(AppIcon, { name: 'close', size: 12 })]),
    ]),
    sections.length
      ? h('div', { class: 'command-panel__sections' }, sections.map((section) =>
          h('section', { key: section.key, class: 'command-panel__section' }, [
            h('span', { class: 'command-panel__title' }, section.title),
            h('div', { class: 'command-list' }, section.commands.map((command) => renderCommandRow(pane, command, section.favoriteScope))),
          ]),
        ))
      : h('div', { class: 'command-panel__empty' }, '当前终端还没有可复用命令。'),
  ])
}

function renderCommandRow(pane: PaneNode, command: string, favoriteScope: boolean): VNode {
  const favorite = isFavoriteCommand(pane, command)

  return h('div', { key: command, class: 'command-row' }, [
    h('button', {
      type: 'button',
      class: 'command-row__main',
      title: command,
      onClick: (event: MouseEvent) => {
        event.stopPropagation()
        void insertPaneText(pane, command)
      },
    }, [h('code', command)]),
    h('div', { class: 'command-row__actions' }, [
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: '插入命令',
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          void insertPaneText(pane, command)
        },
      }, [h(AppIcon, { name: 'bolt', size: 12 })]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: '执行命令',
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          void executePaneText(pane, command)
        },
      }, [h(AppIcon, { name: 'play', size: 12 })]),
      h('button', {
        type: 'button',
        class: ['icon-btn icon-btn--mini', { 'icon-btn--active': favorite }],
        title: favorite ? '取消收藏' : '收藏命令',
        disabled: !favoriteScope,
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          toggleFavoriteCommand(pane, command)
        },
      }, [h(AppIcon, { name: 'star', size: 12 })]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: '复制命令',
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          void copyCommandText(command)
        },
      }, [h(AppIcon, { name: 'copy', size: 12 })]),
    ]),
  ])
}

function renderPaneTree(pane: PaneNode): VNode {
  if (!isSplitPane(pane)) {
    return h('section', {
        'data-pane-id': pane.id,
        class: [
          'pane',
          {
            'pane--running': paneHasRunningSession(pane),
            'pane--selected': activeRuntimePaneId.value === pane.id,
          },
        ],
        style: {
          '--pane-flex': `${pane.sizeRatio || 1} 1 0`,
          minWidth: 0,
          minHeight: 0,
        },
        onClick: () => { activeRuntimePaneId.value = pane.id },
      }, [
        h('div', {
          class: 'pane__header pane__header--classic',
          onContextmenu: (event: MouseEvent) => {
            event.preventDefault()
            event.stopPropagation()
            suppressFloatingMenuCloseUntil = Date.now() + 320
            closeFloatingMenus()
            activePaneHeaderMenu.value = pane.id
            activePaneHeaderMenuPosition.value = menuPositionFromEvent(event)
          },
        }, [
          h('div', {
            class: ['terminal-window-tabs', {
              'terminal-window-tabs--drop-target': dragDropTarget.value?.kind === 'tabbar' && dragDropTarget.value.paneId === pane.id,
            }],
            'data-tabbar-pane-id': pane.id,
          }, [
            ...paneSessions(pane).map((session) => h('div', {
              key: session.id,
              role: 'button',
              tabindex: 0,
              'data-session-id': session.id,
              class: ['terminal-window-tab', {
                'terminal-window-tab--active': activePaneSession(pane)?.id === session.id,
                'terminal-window-tab--dragging': draggingSession.value?.sourceSessionId === session.id,
                'terminal-window-tab--supervised': sessionIsSupervised(session),
              }],
              title: session.pathLabel,
              onClick: (event: MouseEvent) => {
                event.stopPropagation()
                if (Date.now() < suppressSessionClickUntil.value) return
                activeRuntimePaneId.value = pane.id
                activatePaneSession(pane.id, session.id)
              },
              onContextmenu: (event: MouseEvent) => {
                event.preventDefault()
                event.stopPropagation()
                suppressFloatingMenuCloseUntil = Date.now() + 320
                closeFloatingMenus()
                activePaneSessionMenu.value = { paneId: pane.id, sessionId: session.id }
                activePaneSessionMenuPosition.value = menuPositionFromEvent(event)
              },
              onPointerdown: (event: PointerEvent) => {
                if (event.button !== 0) return
                const target = event.target as HTMLElement | null
                if (target?.closest('[data-no-drag="true"]')) return
                beginSessionDrag(pane.id, session.id, event)
              },
            }, [
              h(AppIcon, { name: 'terminal', size: 14 }),
              h('span', session.name),
              h('small', sessionStatusLabel(session)),
              h('button', {
                    type: 'button',
                    class: 'terminal-window-tab__reload',
                    title: '重新加载当前终端',
                    role: 'button',
                    tabindex: 0,
                    'data-no-drag': 'true',
                    onClick: (event: MouseEvent) => { event.stopPropagation(); reloadPaneSession(pane.id, session.id) },
                    onPointerdown: (event: PointerEvent) => { event.stopPropagation() },
                  }, [h(AppIcon, { name: 'refresh', size: 12 })]),
              h('button', {
                    type: 'button',
                    class: 'terminal-window-tab__close',
                    title: '关闭当前终端',
                    role: 'button',
                    tabindex: 0,
                    'data-no-drag': 'true',
                    onClick: (event: MouseEvent) => { event.stopPropagation(); removePaneSession(pane.id, session.id) },
                    onPointerdown: (event: PointerEvent) => { event.stopPropagation() },
                  }, [h(AppIcon, { name: 'close', size: 12 })]),
            ])),
            h('button', {
              type: 'button',
              class: 'terminal-window-tab terminal-window-tab--add',
              title: '在当前分组新建终端',
              'data-no-drag': 'true',
              onClick: (event: MouseEvent) => { event.stopPropagation(); createPaneSession(pane.id) },
            }, [h(AppIcon, { name: 'plus', size: 13 })]),
          ]),
          h('div', { class: 'pane__actions' }, [
            h('button', {
              type: 'button',
              class: ['icon-btn icon-btn--bolt', { 'icon-btn--active': activeCommandPanelPaneId.value === pane.id }],
              title: '快捷命令',
              'data-no-drag': 'true',
              onClick: (event: MouseEvent) => {
                event.stopPropagation()
                closeFloatingMenus()
                activeCommandPanelPaneId.value = activeCommandPanelPaneId.value === pane.id ? null : pane.id
              },
            }, [h(AppIcon, { name: 'bolt', size: 14 })]),
            h('button', {
              type: 'button',
              class: 'icon-btn pane__group-btn',
              title: 'Pane 分组操作',
              'data-no-drag': 'true',
              onClick: (event: MouseEvent) => {
                event.stopPropagation()
                activePaneHeaderMenu.value = null
                activePaneHeaderMenuPosition.value = null
                togglePaneMenu(pane.id, event)
              },
            }, [h(AppIcon, { name: 'more', size: 14 })]),
            h('div', { class: 'pane__more-wrap' }, [
              h(PopoverMenu, { open: activePaneHeaderMenu.value === pane.id, items: paneHeaderMenuItems(pane), position: activePaneHeaderMenuPosition.value }),
              h(PopoverMenu, { open: activePaneMenu.value === pane.id, items: paneMenuItems(pane), position: activePaneMenuPosition.value }),
              h(PopoverMenu, { open: activePaneBindingMenu.value === pane.id, items: paneBindingItems(pane), position: activePaneBindingMenuPosition.value }),
              ...(activePaneSessionMenu.value?.paneId === pane.id
                ? [
                    h(PopoverMenu, {
                      open: true,
                      items: paneSessionMenuItems(
                        pane,
                        paneSessions(pane).find((item) => item.id === activePaneSessionMenu.value?.sessionId) || activePaneSession(pane),
                      ),
                      position: activePaneSessionMenuPosition.value,
                    }),
                  ]
                : []),
            ]),
          ]),
        ]),
        h('div', { class: 'pane__body pane__body--terminal' }, [
          activeCommandPanelPaneId.value === pane.id ? renderCommandPanel(pane) : null,
          h(TerminalPane, {
            key: `${activePaneSession(pane)?.id || pane.id}-${terminalReloadVersions.value[activePaneSession(pane)?.id || pane.id] ?? 0}`,
            sessionId: activePaneSession(pane)?.id || pane.id,
            sessionName: activePaneSession(pane)?.name || pane.name,
            workingDirectory: terminalSessionWorkingDirectory(pane),
            shellLabel: terminalSessionShellLabel(pane),
            fontFamily: terminalFontFamily.value,
            fontSize: terminalFontSize.value,
            onCommandCommitted: recordSessionCommand,
            onSessionStateChange: syncSessionState,
            onOutputChunk: recordSessionOutput,
            onSessionExit: recordSessionExit,
          }),
        ]),
      ])
  }

  return h('div', {
      class: paneContainerClass(pane),
      style: {
        '--pane-flex': `${pane.sizeRatio || 1} 1 0`,
        minWidth: 0,
        minHeight: 0,
      },
    }, paneChildren(pane).flatMap((child, index, children) => {
      const nodes: VNode[] = [h(PaneTreeNode, { pane: child })]
      if (index < children.length - 1) {
        const nextChild = children[index + 1]
        nodes.push(h('div', {
          class: ['pane-splitter', `pane-splitter--${pane.splitDirection}`, {
            'pane-splitter--active': splitResizeState.value?.splitPaneId === pane.id
              && splitResizeState.value?.leftChildId === child.id
              && splitResizeState.value?.rightChildId === nextChild.id,
          }],
          onPointerdown: (event: PointerEvent) => startSplitResize(pane.id, pane.splitDirection as SplitDirection, child.id, nextChild.id, event),
        }))
      }
      return nodes
    }))
}

function splitLeafPane(targetPaneId: string, direction: SplitDirection) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return
  const now = new Date().toISOString()
  const splitNodeId = createId('split')
  const currentTabId = activeRuntimeTab.value.id
  const workspaceRoot = selectedWorkspace.value.rootPath

  const createLeaf = (paneName: string, paneId: string, ratio = 1): PaneNode => {
    const sessionId = createId('session')
    return {
      id: paneId,
      tabId: currentTabId,
      name: paneName,
      pathLabel: workspaceRoot,
      terminalEntryId: null,
      splitDirection: 'none',
      parentPaneId: null,
      order: 0,
      sizeRatio: ratio,
      activeSessionId: sessionId,
      sessions: [{
        id: sessionId,
        name: 'PowerShell 7',
        pathLabel: workspaceRoot,
        terminalEntryId: null,
        status: 'idle',
      }],
      children: [],
    }
  }

  const nextPane = createLeaf(nextPaneName(activeRuntimeTab.value!), createId('pane'), 1)

  const splitNode = (pane: PaneNode): PaneNode => {
    if (pane.id === targetPaneId && !isSplitPane(pane)) {
      const parentRatio = pane.sizeRatio || 1
      const currentLeaf: PaneNode = {
        ...pane,
        splitDirection: 'none',
        children: [],
        parentPaneId: splitNodeId,
        order: 0,
        sizeRatio: 1,
      }
      return {
        id: splitNodeId,
        tabId: pane.tabId,
        name: pane.name,
        pathLabel: pane.pathLabel,
        terminalEntryId: null,
        splitDirection: direction,
        parentPaneId: pane.parentPaneId ?? null,
        order: pane.order,
        sizeRatio: parentRatio,
        activeSessionId: null,
        sessions: [],
        children: [
          { ...currentLeaf, order: 0, sizeRatio: 1 },
          { ...nextPane, order: 1, parentPaneId: splitNodeId, sizeRatio: 1 },
        ],
      }
    }

    return {
      ...pane,
      children: pane.children?.map((child) => splitNode(child)) || [],
    }
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    tabs: workspace.tabs.map((tab) =>
      tab.id === activeRuntimeTab.value?.id
        ? {
            ...tab,
            paneSequence: (tab.paneSequence || 0) + 1,
            panes: normalizePaneTree(tab.panes.map((pane) => splitNode(pane))),
            updatedAt: now,
          }
        : tab,
    ),
    updatedAt: now,
  }))

  activeRuntimePaneId.value = nextPane.id
  setRuntimeActiveSessionId(nextPane.id, nextPane.sessions?.[0]?.id ?? null)
  showToast(direction === 'horizontal' ? '已拆分到右侧' : '已拆分到下方', `新建分组：${nextPane.name}`)
}

function collapsePaneBranch(targetPaneId: string, panes: PaneNode[]): PaneNode[] {
  const normalizeList = (list: PaneNode[]) => {
    const total = list.reduce((sum, item) => sum + (item.sizeRatio || 1), 0) || 1
    return list.map((item, index) => ({
      ...item,
      order: index,
      sizeRatio: (item.sizeRatio || 1) / total,
    }))
  }

  const visit = (pane: PaneNode): PaneNode | null => {
    if (!pane.children?.length) {
      return pane.id === targetPaneId ? null : pane
    }

    if (pane.id === targetPaneId) return null

    const nextChildren = normalizeList(
      pane.children
        .map((child) => visit(child))
        .filter((child): child is PaneNode => Boolean(child)),
    )

    if (nextChildren.length === 0) return null

    if (nextChildren.length === 1) {
      const only = nextChildren[0]
      return {
        ...only,
        parentPaneId: pane.parentPaneId ?? null,
        order: pane.order,
        sizeRatio: pane.sizeRatio || only.sizeRatio || 1,
      }
    }

    return {
      ...pane,
      children: nextChildren,
    }
  }

  return normalizeList(
    panes
      .map((pane) => visit(pane))
      .filter((pane): pane is PaneNode => Boolean(pane)),
  )
}

function visitPaneTree(panes: PaneNode[], visitor: (pane: PaneNode) => PaneNode): PaneNode[] {
  return panes.map((pane) => {
    const nextChildren = pane.children?.length ? visitPaneTree(pane.children, visitor) : pane.children
    return visitor({
      ...pane,
      children: nextChildren,
    })
  })
}

function findPaneById(panes: PaneNode[], paneId: string): PaneNode | undefined {
  for (const pane of panes) {
    if (pane.id === paneId) return pane
    if (pane.children?.length) {
      const nested = findPaneById(pane.children, paneId)
      if (nested) return nested
    }
  }
  return undefined
}

function normalizePaneOrders(panes: PaneNode[]): PaneNode[] {
  return panes.map((pane, index) => ({
    ...pane,
    order: index,
    children: pane.children?.length ? normalizePaneOrders(pane.children) : pane.children,
  }))
}

function normalizePaneRatios(panes: PaneNode[], parentPaneId: string | null = null): PaneNode[] {
  const total = panes.reduce((sum, pane) => sum + (pane.sizeRatio || 1), 0) || 1
  return panes.map((pane, index) => {
    const children = pane.children?.length ? normalizePaneRatios(pane.children, pane.id) : pane.children
    return {
      ...pane,
      parentPaneId,
      order: index,
      sizeRatio: (pane.sizeRatio || 1) / total,
      children,
    }
  })
}

function normalizePaneTree(panes: PaneNode[], parentPaneId: string | null = null): PaneNode[] {
  return normalizePaneRatios(normalizePaneOrders(panes), parentPaneId)
}

function findFirstLeafPaneId(panes: PaneNode[]): string {
  for (const pane of panes) {
    if (pane.children?.length) {
      const nested = findFirstLeafPaneId(pane.children)
      if (nested) return nested
      continue
    }
    return pane.id
  }
  return ''
}

function splitLabel(direction?: string) {
  if (direction === 'horizontal') return '水平分屏'
  if (direction === 'vertical') return '垂直分屏'
  return '单 Pane'
}

function paneSessions(pane: PaneNode): PaneTerminalSession[] {
  if (pane.sessions) return pane.sessions

  return [
    {
      id: `${pane.id}-session-main`,
      name: pane.name,
      pathLabel: pane.pathLabel,
      terminalEntryId: pane.terminalEntryId,
      status: 'idle',
    },
  ]
}

function activePaneSession(pane: PaneNode) {
  const sessions = paneSessions(pane)
  const runtimeSessionId = runtimeActiveSessionIds.value[pane.id]
  return sessions.find((session) => session.id === runtimeSessionId)
    ?? sessions.find((session) => session.id === pane.activeSessionId)
    ?? sessions[0]
}

function sessionStatusLabel(session?: PaneTerminalSession) {
  if (!session) return '空闲'
  if (session.status === 'running') return '运行中'
  if (entryById(session.terminalEntryId)?.lastCommand || entryById(session.terminalEntryId)?.defaultCommand) return '等待中'
  return '空闲'
}

function terminalSessionEntry(pane: PaneNode) {
  return entryById(activePaneSession(pane)?.terminalEntryId)
}

function terminalSessionWorkingDirectory(pane: PaneNode) {
  const session = activePaneSession(pane)
  const entryDirectory = terminalSessionEntry(pane)?.workingDirectory?.trim()
  if (entryDirectory) return entryDirectory

  const workspaceRoot = selectedWorkspace.value?.rootPath?.trim()
  if (workspaceRoot) return workspaceRoot

  const sessionPath = session?.pathLabel?.trim()
  if (sessionPath && !sessionPath.includes('...')) return sessionPath

  const panePath = pane.pathLabel?.trim()
  if (panePath && !panePath.includes('...')) return panePath

  return '~'
}

function terminalSessionShellLabel(pane: PaneNode) {
  const shellType = terminalSessionEntry(pane)?.shellType
  if (shellType === 'custom') return 'Custom Shell'
  return 'PowerShell 7'
}

function runtimeTabLabel(tab: WorkspaceTab) {
  const label = tab.name.trim() || '未命名项目'
  return label.length > 18 ? `${label.slice(0, 18)}…` : label
}

function currentActiveRuntimeSessionId() {
  if (!activeRuntimeTab.value || !activeRuntimePaneId.value) return ''
  const pane = findPaneById(activeRuntimeTab.value.panes, activeRuntimePaneId.value)
  if (!pane) return ''
  return runtimeActiveSessionIds.value[pane.id] ?? pane.activeSessionId ?? paneSessions(pane)[0]?.id ?? ''
}

function setRuntimeActiveSessionId(paneId: string, sessionId: string | null) {
  const next = { ...runtimeActiveSessionIds.value }
  if (sessionId) next[paneId] = sessionId
  else delete next[paneId]
  runtimeActiveSessionIds.value = next
}

function activatePaneSession(paneId: string, sessionId: string) {
  if (!activeRuntimeTab.value) return
  const targetPane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!targetPane) return
  const currentSessionId = runtimeActiveSessionIds.value[paneId] ?? targetPane.activeSessionId ?? paneSessions(targetPane)[0]?.id ?? ''
  const targetSession = paneSessions(targetPane).find((session) => session.id === sessionId)

  if (currentSessionId !== sessionId) {
    setRuntimeActiveSessionId(paneId, sessionId)
    queueSaveWorkbenchRestoreState()
  }

  if (targetSession && shouldCountAttentionState(sessionAttentionState(targetSession))) {
    clearSessionAttention(sessionId, { silent: true })
  }
}

function paneRuntimeTone(pane: PaneNode, workspace: WorkspaceCard | undefined = selectedWorkspace.value) {
  if (workspacePaneRunning(workspace, pane)) return 'running'
  const hasHistory = paneSessions(pane).some((session) => {
    const entry = workspace?.terminalEntries.find((item) => item.id === session.terminalEntryId)
    return Boolean(entry?.lastCommand || entry?.defaultCommand)
  })
  if (hasHistory) return 'waiting'
  return 'idle'
}

function paneRuntimeLabel(pane: PaneNode, workspace: WorkspaceCard | undefined = selectedWorkspace.value) {
  const tone = paneRuntimeTone(pane, workspace)

  if (tone === 'running') return '运行中'
  if (tone === 'waiting') return '等待中'
  return '空闲'
}

function openWorkspace(workspaceId: string) {
  rememberWorkspaceFocus()
  if (!openedWorkspaceIds.value.includes(workspaceId)) {
    openedWorkspaceIds.value = [...openedWorkspaceIds.value, workspaceId]
  }

  selectedWorkspaceId.value = workspaceId
  appSection.value = 'workspace'
  workspaceView.value = 'runtime'

  void refreshWorkspaceGitInfo(workspaceId)
  applyWorkspaceFocus(workspaceId)

  const now = new Date().toISOString()
  commitWorkspaces((current) => current.map((item) =>
    item.id === workspaceId
      ? { ...item, lastOpenedAt: now, updatedAt: now }
      : item,
  ), 'transient')
}

function openThemePanel(tab: ThemePanelTab) {
  themePanelTab.value = tab
  openThemeModal.value = true
  if (tab === 'system') {
    if (systemStatus.cpu === '检测中' || systemStatus.memory === '检测中' || systemStatus.gpu === '检测中') {
      void refreshSystemStatus()
    }
    if (!hasEnvironmentCheckCache.value && !environmentChecksRefreshing.value) {
      void refreshEnvironmentChecks()
    }
  }
}

function selectTheme(themeId: string) {
  activeThemeId.value = themeId
  const theme = themePresets.find((item) => item.id === themeId)
  if (theme) {
    showToast('主题已切换', `当前使用：${theme.name}`)
  }
}








function paneMenuPositionFromEvent(event: MouseEvent) {
  const menuWidth = 280
  const menuHeight = 320
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const x = Math.min(event.clientX, Math.max(16, viewportWidth - menuWidth - 16))
  const y = Math.min(event.clientY, Math.max(16, viewportHeight - menuHeight - 16))
  return { x, y }
}

function togglePaneMenu(paneId: string, event?: MouseEvent) {
  const shouldClose = activePaneMenu.value === paneId && (!event || event.type === 'click')
  const nextPosition = event ? paneMenuPositionFromEvent(event) : activePaneMenuPosition.value
  closeFloatingMenus()
  suppressFloatingMenuCloseUntil = Date.now() + 320

  if (shouldClose) return
  activePaneMenuPosition.value = nextPosition
  activePaneMenu.value = paneId
}

function togglePaneBindingMenu(paneId: string, event?: MouseEvent) {
  const shouldOpen = activePaneBindingMenu.value !== paneId
  const nextPosition = event ? paneMenuPositionFromEvent(event) : activePaneBindingMenuPosition.value
  closeFloatingMenus()

  if (shouldOpen) {
    activePaneBindingMenuPosition.value = nextPosition
    activePaneBindingMenu.value = paneId
  }
}

function setTerminalFontSize(size: number) {
  terminalFontSize.value = clamp(size, 8, 24)
}

function setSystemRefreshMode(mode: 'manual' | '5s' | '10s' | '30s') {
  systemRefreshInterval.value = mode
  scheduleSystemRefresh()
}

function toggleEnvironmentVisibility(name: string) {
  hiddenEnvironmentItems.value = hiddenEnvironmentItems.value.includes(name)
    ? hiddenEnvironmentItems.value.filter((item) => item !== name)
    : [...hiddenEnvironmentItems.value, name]
}

function applyThemeFromSettings(themeId: string) {
  selectTheme(themeId)
}

function applyTerminalFontSizeFromSettings(size: number) {
  setTerminalFontSize(size)
  showToast('终端字号已更新', `${size}px`)
}

function applyTerminalFontFamilyFromSettings(font: string) {
  terminalFontFamily.value = font
  showToast('终端字体已更新', font)
}

function applyRestoreStrategyFromSettings(strategy: RestoreCommandStrategy) {
  restoreCommandStrategy.value = strategy
  showToast('恢复策略已更新', restoreCommandStrategyLabel(strategy))
}

function applySystemRefreshModeFromSettings(mode: 'manual' | '5s' | '10s' | '30s') {
  setSystemRefreshMode(mode)
  showToast('系统刷新频率已更新', mode === 'manual' ? '手动刷新' : `${mode} 自动刷新`)
}

function toggleEnvironmentVisibilityFromSettings(name: string) {
  toggleEnvironmentVisibility(name)
  showToast('环境项显示已更新', name)
}

function clearDiagnosticsCaches() {
  if (typeof window === 'undefined') return
  window.localStorage.removeItem(SYSTEM_STATUS_CACHE_KEY)
  window.localStorage.removeItem(ENVIRONMENT_CHECK_CACHE_KEY)
  hasSystemStatusCache.value = false
  hasEnvironmentCheckCache.value = false
  systemStatusCacheUpdatedAt.value = 0
  environmentCheckCacheUpdatedAt.value = 0
  systemStatus.cpu = '检测中'
  systemStatus.memory = '检测中'
  systemStatus.gpu = '检测中'
  homeEnvironmentChecks.forEach((item) => {
    if (TAURI_ENVIRONMENT_CHECK_NAMES.includes(item.name as typeof TAURI_ENVIRONMENT_CHECK_NAMES[number])) {
      item.value = '待检测'
      item.status = 'pending'
    }
  })
  showToast('系统数据已重置', '已清空缓存并重新开始加载。')
  void refreshSystemStatus()
  void refreshEnvironmentChecks(true)
}

let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(title: string, message: string) {
  if (toastTimer) {
    clearTimeout(toastTimer)
    toastTimer = null
  }

  uiToast.value = {
    id: Date.now(),
    title,
    message,
    progress: 100,
    duration: 3000,
  }

  toastTimer = setTimeout(() => {
    dismissToast()
  }, uiToast.value.duration)
}

function dismissToast() {
  uiToast.value = null
  if (toastTimer) {
    clearTimeout(toastTimer)
    toastTimer = null
  }
}

function onAccentColorInput(value: string) {
  customAccentHex.value = normalizeHex(value, customAccentHex.value)
}

function onAccentHexChange(value: string) {
  customAccentHex.value = normalizeHex(value, customAccentHex.value)
}

function onAccentRgbChange(channel: 'r' | 'g' | 'b', value: string) {
  const current = hexToRgb(customAccentHex.value)
  const next = { ...current, [channel]: clamp(Number(value), 0, 255) }
  customAccentHex.value = rgbToHex(next.r, next.g, next.b)
}

function clamp(value: number, min: number, max: number) {
  if (Number.isNaN(value)) return min
  return Math.min(Math.max(Math.round(value), min), max)
}

function normalizeHex(value: string, fallback: string) {
  const raw = value.trim().replace('#', '')
  if (!/^[0-9a-fA-F]{6}$/.test(raw)) return fallback
  return `#${raw.toLowerCase()}`
}

function hexToRgb(hex: string) {
  const raw = normalizeHex(hex, '#3dd6c6').slice(1)
  return {
    r: Number.parseInt(raw.slice(0, 2), 16),
    g: Number.parseInt(raw.slice(2, 4), 16),
    b: Number.parseInt(raw.slice(4, 6), 16),
  }
}

function rgbToHex(r: number, g: number, b: number) {
  return `#${[r, g, b].map((channel) => clamp(channel, 0, 255).toString(16).padStart(2, '0')).join('')}`
}

function lightenColor(hex: string, amount: number) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex(r + amount, g + amount, b + amount)
}

function colorLuminance(hex: string) {
  const { r, g, b } = hexToRgb(hex)
  const channels = [r, g, b].map((channel) => {
    const value = channel / 255
    return value <= 0.03928
      ? value / 12.92
      : ((value + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
}

function rgba(hex: string, alpha: number) {
  const { r, g, b } = hexToRgb(hex)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}
</script>




