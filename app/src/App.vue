<template>
  <div class="app-shell" :class="appShellClasses" :style="appThemeVars">
    <header class="titlebar">
      <div class="titlebar__brand">
        <div class="titlebar__logo">CT</div>
        <div>
          <strong>{{ t('common.appName') }}</strong>
          <p>{{ t('common.appSubtitle') }}</p>
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
        <span class="titlebar__status-pill"><AppIcon name="runtime" :size="13" /> <strong>{{ t('titlebar.cpu') }}</strong> {{ systemStatusDisplayValue(systemStatus.cpu) }}</span>
        <span class="titlebar__status-pill"><AppIcon name="workspace" :size="13" /> <strong>{{ t('titlebar.memory') }}</strong> {{ systemStatusDisplayValue(systemStatus.memory) }}</span>
        <span class="titlebar__status-pill"><AppIcon name="theme" :size="13" /> <strong>{{ t('titlebar.gpu') }}</strong> {{ systemStatusDisplayValue(systemStatus.gpu) }}</span>
        <span v-if="attentionSessionCount" class="titlebar__status-pill titlebar__status-pill--attention">
          <AppIcon name="recent" :size="13" />
          <strong>{{ t('titlebar.pending') }}</strong> {{ attentionSessionCount }}
        </span>
        <button type="button" class="titlebar__status-action" @click="openThemePanel('system')">
          <AppIcon name="settings" :size="13" />
          <span>{{ t('titlebar.system') }}</span>
        </button>
      </div>
    </header>

    <div class="layout" :class="{ 'layout--rail-collapsed': effectiveRailCollapsed, 'layout--immersive': immersiveWorkbenchActive }">
      <aside class="rail" :class="{ 'rail--collapsed': effectiveRailCollapsed, 'rail--immersive': immersiveWorkbenchActive }">
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
              <span v-if="!effectiveRailCollapsed">{{ item.label }}</span>
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
              <span v-if="!effectiveRailCollapsed">{{ item.label }}</span>
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

          <button v-if="!immersiveWorkbenchActive" class="rail__collapse" :title="effectiveRailCollapsed ? t('rail.expand') : t('rail.collapse')" @click="toggleRailCollapsed()">
            <AppIcon name="chevron-right" :size="15" />
            <span v-if="!effectiveRailCollapsed">{{ effectiveRailCollapsed ? t('rail.expand') : t('rail.collapse') }}</span>
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
                  <span>{{ t('common.actions.runConfigs') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="openHelpTopic('layout')">
                  <AppIcon name="info" :size="14" />
                  <span>{{ t('common.actions.structureGuide') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="goWorkspaceOverview">
                  <AppIcon name="workspace" :size="14" />
                  <span>{{ t('common.actions.backToCards') }}</span>
                </button>
              </div>
            </template>
          </div>
        </section>

        <section class="home-dashboard" v-if="appSection === 'home'">
          <section class="home-hero panel">
            <div class="home-hero__copy">
              <h2>{{ t('home.welcome') }}</h2>
              <p>{{ t('home.intro') }}</p>
            </div>
            <div class="home-hero__meta">
              <span>{{ t('home.recentUsed', { name: recentHomeWorkspaces[0]?.name || t('common.none') }) }}</span>
              <span>{{ t('home.defaultShell') }}</span>
              <span>{{ t('home.desktopShell') }}</span>
            </div>
            <div class="home-hero__stats">
              <div class="home-stat">
                <strong>{{ totalWorkspaceCount }}</strong>
                <span>{{ t('home.workspaceCount') }}</span>
              </div>
              <div class="home-stat">
                <strong>{{ totalPaneCount }}</strong>
                <span>{{ t('home.paneCount') }}</span>
              </div>
              <div class="home-stat home-stat--running">
                <strong>{{ totalRunningCount }}</strong>
                <span>{{ t('home.runningCount') }}</span>
              </div>
            </div>
            <div class="home-hero__quick">
              <button class="ghost-btn ghost-btn--primary" @click="openWorkspaceCreateModal()">
                <AppIcon name="workspace" :size="15" />
                <span>{{ t('home.newWorkspace') }}</span>
              </button>
              <button class="ghost-btn" @click="openQuickSearch()">
                <AppIcon name="search" :size="15" />
                <span>{{ t('home.search') }}</span>
              </button>
              <button class="ghost-btn" @click="openThemePanel('theme')">
                <AppIcon name="theme" :size="15" />
                <span>{{ t('home.theme') }}</span>
              </button>
            </div>
          </section>

          <section class="home-grid">
            <article class="home-card home-card--recent">
              <div class="home-card__header">
                <div>
                  <h3>{{ t('home.recentTitle') }}</h3>
                  <p>{{ t('home.recentDesc') }}</p>
                </div>
                <button class="ghost-btn ghost-btn--small" @click="goWorkspaceOverview">
                  <AppIcon name="workspace" :size="14" />
                  <span>{{ t('home.openAll') }}</span>
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
                  <small>{{ t('workspace.projects', { count: workspace.tabs.length }) }} · {{ t('snapshot.panes', { count: totalPanes(workspace) }) }} · {{ t('snapshot.terminals', { count: totalWorkspaceSessions(workspace) }) }}</small>
                </button>
              </div>
            </article>

            <article class="home-card home-card--env">
              <div class="home-card__header">
                <div>
                  <h3>{{ t('home.envTitle') }}</h3>
                  <p>{{ t('home.envDesc') }}</p>
                </div>
                <button type="button" class="ghost-btn ghost-btn--small" @click="openThemePanel('system')">
                  <AppIcon name="settings" :size="14" />
                  <span>{{ t('home.system') }}</span>
                </button>
              </div>
              <div class="home-check-list">
                <div v-for="item in visibleEnvironmentChecks" :key="item.name" class="home-check-item" :class="`home-check-item--${item.status}`">
                  <span class="home-env-icon" :class="{ 'home-env-icon--mono': item.monochrome }">
                    <img v-if="item.iconSrc" :src="item.iconSrc" :alt="`${item.name} icon`" />
                    <span v-else>{{ item.icon }}</span>
                  </span>
                  <div>
                    <strong>{{ item.name }}</strong>
                    <small>{{ environmentItemNoteLabel(item.note) }}</small>
                  </div>
                  <span class="home-check-badge">{{ environmentItemValueLabel(item.value) }}</span>
                </div>
              </div>
            </article>
          </section>
        </section>

        <section class="workspace-overview" v-else-if="appSection === 'workspace' && workspaceView === 'overview'">
          <aside class="workspace-overview__list panel">
            <div class="workspace-overview__list-head">
              <div>
                <h2>{{ t('workspace.overviewTitle') }}</h2>
                <span>{{ t('workspace.overviewSummary', { total: workspaces.length, opened: openedWorkspaces.length }) }}</span>
              </div>
              <button class="icon-btn" :title="t('workspace.createWorkspace')" @click="openWorkspaceCreateModal()">
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
                <small>{{ t('workspace.running', { count: runningCount(workspace) }) }} · {{ t('workspace.projects', { count: workspace.tabs.length }) }} · {{ t('workspace.terminals', { count: totalPanes(workspace) }) }} · {{ workspaceGitBranchLabel(workspace) }}</small>
              </button>
              <div class="workspace-overview__item-actions">
                <button
                  type="button"
                  class="icon-btn icon-btn--mini workspace-overview__item-action workspace-overview__item-action--edit"
                  :title="t('workspace.editWorkspace')"
                  @click.stop="openWorkspaceEditModal(workspace.id)"
                >
                  <AppIcon name="edit" :size="13" />
                </button>
                <button
                  type="button"
                  class="icon-btn icon-btn--mini workspace-overview__item-action workspace-overview__item-action--delete"
                  :title="t('workspace.removeWorkspace')"
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
                <h2>{{ t('workspace.carouselTitle') }}</h2>
                <span>{{ t('workspace.carouselHint') }}</span>
              </div>
              <div class="workspace-carousel__actions">
                <span class="meta-badge meta-badge--soft">{{ overviewWorkspaceIndex + 1 }} / {{ workspaces.length || 1 }}</span>
                <button class="ghost-btn ghost-btn--small" @click="openQuickSearch()">
                  <AppIcon name="search" :size="14" />
                  <span>{{ t('home.search') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small ghost-btn--primary" @click="openWorkspaceCreateModal()">
                  <AppIcon name="workspace" :size="14" />
                  <span>{{ t('workspace.create') }}</span>
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
                    <p>{{ activeOverviewWorkspace.description || t('workspace.noDescription') }}</p>
                  </div>
                  <span class="meta-badge">{{ relativeTimeLabel(activeOverviewWorkspace.lastOpenedAt) }}</span>
                </div>
                <div class="path-row workspace-carousel__path">{{ activeOverviewWorkspace.rootPath }}</div>
                <div class="workspace-carousel__metrics">
                  <span>{{ t('workspace.projects', { count: activeOverviewWorkspace.tabs.length }) }}</span>
                  <span>{{ t('workspace.terminals', { count: totalPanes(activeOverviewWorkspace) }) }}</span>
                  <span class="meta-running">{{ t('workspace.running', { count: runningCount(activeOverviewWorkspace) }) }}</span>
                  <span>{{ workspaceGitBranchLabel(activeOverviewWorkspace) }}</span>
                </div>
                <div class="workspace-card__tags">
                  <span v-for="tag in activeOverviewWorkspace.tags" :key="tag" class="tag-chip">{{ tag }}</span>
                  <span v-if="!activeOverviewWorkspace.tags.length" class="tag-chip tag-chip--soft">{{ t('workspace.tagsEmpty') }}</span>
                </div>
                <div class="workspace-carousel__preview">
                  <div v-for="tab in activeOverviewWorkspace.tabs.slice(0, 3)" :key="tab.id" class="workspace-carousel__tab-row">
                    <strong>{{ tab.name }}</strong>
                    <span>{{ t('snapshot.panes', { count: countLeafPanes(tab.panes) }) }} · {{ t('snapshot.terminals', { count: countPaneSessions(tab.panes) }) }}</span>
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
                  <button type="button" class="card-action-btn card-action-btn--primary" @click="openWorkspace(activeOverviewWorkspace.id)">{{ t('workspace.openWorkspace') }}</button>
                  <button type="button" class="card-action-btn" @click="openWorkspaceEditModal(activeOverviewWorkspace.id)">{{ t('workspace.editWorkspace') }}</button>
                  <button type="button" class="card-action-btn card-action-btn--danger" @click="removeWorkspace(activeOverviewWorkspace.id)">{{ t('workspace.removeWorkspace') }}</button>
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
        <section v-else-if="isWorkspaceWorkbench && selectedWorkspace" class="workbench-shell" :class="workbenchShellClasses" :style="workbenchShellStyle">
          <aside class="workbench-sidebar" :class="{ 'workbench-sidebar--collapsed': workbenchExplorerCollapsed }">
            <section class="explorer-shell">
              <div class="explorer-shell__bar">
                <span class="explorer-shell__bar-title">{{ tr('Explorer', 'Explorer') }}</span>
                <button
                  type="button"
                  class="explorer-shell__collapse-btn"
                  :title="tr('收起 Explorer', 'Hide Explorer')"
                  @click="toggleWorkbenchExplorerCollapsed()"
                >
                  <AppIcon name="chevron-right" :size="13" class="explorer-shell__collapse-icon" />
                </button>
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
                          <span><AppIcon name="tab" :size="12" />{{ t('workspace.projects', { count: workspace.tabs.length }) }}</span>
                          <span><AppIcon name="pane" :size="12" />{{ t('workspace.terminals', { count: totalPanes(workspace) }) }}</span>
                        </div>
                        <div class="explorer-workspace__branch">
                          <AppIcon name="copy" :size="12" />
                          <span>{{ workspaceGitBranchLabel(workspace) }}</span>
                        </div>
                      </div>
                    </button>

                    <div class="explorer-workspace__menu-wrap">
                      <button class="icon-btn icon-btn--mini" :title="t('common.actions.moreActions')" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleWorkspaceMenu(workspace.id, $event)">
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
                          :title="isTreeTabCollapsed(tab.id) ? t('workspace.expandAll') : t('workspace.collapseAll')"
                          @click.stop="toggleTreeTab(tab.id)"
                        >
                          <AppIcon
                            class="explorer-project__chevron"
                            :class="{ 'explorer-project__chevron--open': !isTreeTabCollapsed(tab.id) }"
                            name="chevron-right"
                            :size="12"
                          />
                        </button>
                        <button
                          type="button"
                          class="explorer-project__button"
                          @click="openProjectWorkspace(workspace.id, tab.id)"
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
                              <span>{{ t('snapshot.terminals', { count: countPaneSessions(tab.panes) }) }}</span>
                              <span>{{ t('workspace.running', { count: tabRunningCount(workspace, tab) }) }}</span>
                            </div>
                          </div>
                        </button>
                        <div class="explorer-project__actions">
                          <button
                            type="button"
                            class="icon-btn icon-btn--mini explorer-project__action"
                            :title="t('common.actions.edit')"
                            @pointerdown="handleMenuTriggerPointerDown"
                            @click.stop="openExplorerTabRename(workspace.id, tab.id)"
                          >
                            <AppIcon name="edit" :size="13" />
                          </button>
                          <button
                            type="button"
                            class="icon-btn icon-btn--mini explorer-project__action"
                            :title="t('common.actions.moreActions')"
                            @pointerdown="handleMenuTriggerPointerDown"
                            @click.stop="toggleExplorerProjectMenu(workspace.id, tab.id, $event)"
                          >
                            <AppIcon name="more" :size="13" />
                          </button>
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
                              { 'explorer-pane--ai-cli': isConfirmedAiCliInfo(item.info) },
                              explorerSessionAttentionClass(item.session),
                            ]"
                            :style="isConfirmedAiCliInfo(item.info) ? explorerAiPaneStyle(workspace, item.pane, item.session) : undefined"
                            @click="openWorkspaceTerminalSession(workspace.id, tab.id, item.pane.id, item.session.id)"
                          >
                            <span class="status-dot" :class="`status-dot--${explorerSessionTone(item.session)}`"></span>
                            <div class="explorer-pane__body">
                              <div class="explorer-pane__title-row">
                                <strong>{{ item.displayName }}</strong>
                                <span v-if="isConfirmedAiCliInfo(item.info)" class="explorer-ai-badge__text">{{ item.info.label }}</span>
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
                        <li v-if="!explorerSessionItems(workspace, tab).length" class="tree-empty tree-empty--compact">{{ t('workspace.readyDesc') }}</li>
                      </ul>
                    </article>

                    <button type="button" class="explorer-list-action explorer-list-action--project" @click="createTab()">
                      <AppIcon name="tab" :size="14" />
                      <span>{{ t('common.actions.newProject') }}</span>
                    </button>
                  </div>
                </article>

                <button type="button" class="explorer-list-action explorer-list-action--workspace" @click="openWorkspaceCreateModal()">
                  <AppIcon name="workspace" :size="14" />
                  <span>{{ t('common.actions.newWorkspace') }}</span>
                </button>
              </div>
            </section>
          </aside>

          <div
            v-if="!workbenchExplorerCollapsed && !immersiveWorkbenchActive"
            class="workbench-resizer"
            :title="tr('拖拽调整 Explorer 宽度', 'Drag to resize Explorer width')"
            @pointerdown="startWorkbenchResize"
          ></div>

          <section class="workbench-main">
            <button
              v-if="workbenchExplorerCollapsed && !immersiveWorkbenchActive"
              type="button"
              class="explorer-reveal"
              :title="tr('显示 Explorer', 'Show Explorer')"
              @click="toggleWorkbenchExplorerCollapsed()"
            >
              <AppIcon name="chevron-right" :size="13" />
            </button>

            <div
              v-if="immersiveWorkbenchActive"
              class="immersive-bar"
            >
              <button
                type="button"
                class="immersive-bar__btn immersive-bar__btn--explorer"
                :class="{ 'is-active': !workbenchExplorerCollapsed }"
                :title="workbenchExplorerCollapsed ? tr('显示 Explorer (Ctrl+B)', 'Show Explorer (Ctrl+B)') : tr('收起 Explorer (Ctrl+B)', 'Hide Explorer (Ctrl+B)')"
                :aria-pressed="!workbenchExplorerCollapsed"
                @click="toggleWorkbenchExplorerCollapsed()"
              >
                <AppIcon name="workspace" :size="14" />
              </button>

              <div class="immersive-bar__tabs runtime-tabs runtime-tabs--embedded">
                <button
                  v-for="tab in selectedWorkspace.tabs"
                  :key="tab.id"
                  class="tab immersive-tab"
                  :class="{ 'tab--active': activeRuntimeTabId === tab.id }"
                  :title="runtimeTabLabel(tab)"
                  @click="openProjectWorkspace(selectedWorkspace.id, tab.id)"
                  @contextmenu.prevent.stop="toggleRuntimeTabMenu(tab.id, $event)"
                >
                  <span>{{ runtimeTabLabel(tab) }}</span>
                  <small v-if="countPaneSessions(tab.panes)" class="tab-badge">{{ countPaneSessions(tab.panes) }}</small>
                </button>
                <PopoverMenu :open="Boolean(activeRuntimeTabMenuId)" :items="runtimeTabMenuItems" :position="activeRuntimeTabMenuPosition" />
              </div>

              <button
                type="button"
                class="immersive-bar__btn immersive-bar__btn--exit"
                :title="tr('退出沉浸模式', 'Exit immersive mode')"
                :aria-pressed="immersiveWorkbenchActive"
                @click="toggleWorkbenchImmersive()"
              >
                <AppIcon name="detail" :size="14" />
              </button>
            </div>

            <div v-else class="runtime-header runtime-header--workbench">
              <div class="runtime-header__workspace runtime-header__workspace--compact">
                <button
                  type="button"
                  class="explorer-toggle-inline"
                  :class="{ 'explorer-toggle-inline--open': !workbenchExplorerCollapsed }"
                  :title="workbenchExplorerCollapsed ? tr('显示 Explorer (Ctrl+B)', 'Show Explorer (Ctrl+B)') : tr('收起 Explorer (Ctrl+B)', 'Hide Explorer (Ctrl+B)')"
                  :aria-pressed="!workbenchExplorerCollapsed"
                  @click="toggleWorkbenchExplorerCollapsed()"
                >
                  <AppIcon name="workspace" :size="14" />
                </button>
                <strong>{{ selectedWorkspace.name }}</strong>
                <span>{{ activeRuntimeTab?.name || tr('未选择项目', 'No project selected') }}</span>
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
                <button type="button" class="tab tab--create" :title="t('common.actions.newProject')" @click="createTab()">
                  <AppIcon name="plus" :size="14" />
                </button>
                <PopoverMenu :open="Boolean(activeRuntimeTabMenuId)" :items="runtimeTabMenuItems" :position="activeRuntimeTabMenuPosition" />
                
              </div>

              <div class="runtime-header__utility-strip">
                <button
                  type="button"
                  class="ghost-btn ghost-btn--small runtime-header__control"
                  :class="{ 'ghost-btn--active': immersiveWorkbenchActive }"
                  :title="immersiveWorkbenchActive ? tr('退出沉浸模式', 'Exit immersive mode') : tr('进入沉浸模式', 'Enter immersive mode')"
                  :aria-pressed="immersiveWorkbenchActive"
                  @click="toggleWorkbenchImmersive()"
                >
                  <AppIcon name="detail" :size="14" />
                  <span>{{ tr('沉浸模式', 'Immersive') }}</span>
                </button>
              </div>

              <div class="runtime-header__ai-actions">
                <button
                  type="button"
                  class="ghost-btn ghost-btn--small runtime-ai-action"
                  :disabled="!selectedWorkspaceAiCliSessions.length && !selectedWorkspaceAiCliCommands.length && !selectedWorkspaceAiResumeSnapshots.length"
                  @click="openAiHistoryDrawer = true"
                >
                  <AppIcon name="recent" :size="14" />
                  <span>{{ t('ai.history') }}</span>
                  <small>{{ currentWorkspaceAiCliSessions.length }}</small>
                </button>
                <button
                  type="button"
                  class="ghost-btn ghost-btn--small runtime-ai-action"
                  :class="{ 'ghost-btn--active': aiAssistantPinned && !aiAssistantMinimized }"
                  :disabled="!activeAiAssistItem"
                  @click="toggleAiAssistantVisibility()"
                >
                  <AppIcon name="bolt" :size="14" />
                  <span>{{ t('ai.assist') }}</span>
                </button>
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
                    <strong>{{ t('workspace.readyTitle') }}</strong>
                    <span>{{ t('workspace.readyDesc') }}</span>
                    <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" @click="createPane()">
                      <AppIcon name="pane" :size="14" />
                      <span>{{ t('common.actions.newTerminal') }}</span>
                    </button>
                  </div>
                </div>
              </section>
            </div>

            <div class="runtime-footer">
              <div class="runtime-footer__meta">
                <div>{{ t('workspace.currentWorkspace', { name: selectedWorkspace.name }) }}</div>
                <div>{{ t('workspace.currentProject', { name: activeRuntimeTab?.name || t('common.none') }) }}</div>
                <div>{{ t('workspace.terminalCount', { count: countPaneSessions(activeRuntimeTab?.panes || []) }) }}</div>
                <div v-if="defaultWorkspaceSnapshot">{{ t('workspace.recentSnapshot', { name: defaultWorkspaceSnapshot.name }) }}</div>
              </div>
              <div class="runtime-footer__actions">
                <div class="restore-strategy" role="group" :aria-label="t('workspace.splitStrategy')">
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
                  <span>{{ t('common.actions.saveSnapshot') }}</span>
                </button>
                <button
                  type="button"
                  class="ghost-btn ghost-btn--small"
                  :disabled="!activeRuntimeTab || !countLeafPanes(activeRuntimeTab.panes)"
                  @click="saveActiveTabAsWorkflowTemplate()"
                >
                  <AppIcon name="template" :size="14" />
                  <span>{{ t('common.actions.saveAsTemplate') }}</span>
                </button>
                <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" :disabled="!defaultWorkspaceSnapshot" @click="restoreDefaultWorkspaceSnapshot()">
                  <AppIcon name="refresh" :size="14" />
                  <span>{{ t('common.actions.restoreSnapshot') }}</span>
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
                  <span class="meta-badge meta-badge--soft summary-badge">{{ t('workspace.detailTitle') }}</span>
                  <h3>{{ selectedWorkspace.name }}</h3>
                  <div class="summary-path">
                    <AppIcon name="folder" :size="15" />
                    <span>{{ selectedWorkspace.rootPath }}</span>
                  </div>
                </div>
                <div class="summary-metrics">
                  <div class="summary-metric">
                    <span class="summary-metric__label"><AppIcon name="recent" :size="14" />{{ t('workspace.recentUsed') }}</span>
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
                  <span>{{ t('topbar.runConfigs') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small" @click="openWorkspaceEditModal(selectedWorkspace.id)">{{ t('workspace.editWorkspace') }}</button>
                <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeWorkspace(selectedWorkspace.id)">{{ t('workspace.removeWorkspace') }}</button>
              </div>
            </div>

            <div class="side-card">
              <div class="side-card__header side-card__header--tree">
                <h3>{{ t('workspace.hierarchy') }}</h3>
                <div class="tree-actions">
                  <button class="ghost-btn ghost-btn--small" @click="expandAllTreeTabs()">{{ t('workspace.expandAll') }}</button>
                  <button class="ghost-btn ghost-btn--small" @click="collapseAllTreeTabs()">{{ t('workspace.collapseAll') }}</button>
                </div>
              </div>
              <ul class="tree-list">
                <li v-for="tab in selectedWorkspace.tabs" :key="tab.id" class="tree-node tree-node--tab">
                  <button type="button" class="tree-node__row tree-node__row--tab" @click="toggleTreeTab(tab.id)">
                    <AppIcon class="tree-node__chevron" :class="{ 'tree-node__chevron--open': !isTreeTabCollapsed(tab.id) }" name="chevron-right" :size="12" />
                    <AppIcon name="tab" :size="15" />
                    <strong>{{ tab.name }}</strong>
                    <span class="tree-meta">{{ t('workspace.panesCount', { count: countLeafPanes(tab.panes) }) }}</span>
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
                        <span class="tree-status">{{ paneHasRunningSession(pane) ? t('ai.states.running') : t('workspace.standby') }}</span>
                      </button>
                    </li>
                  </ul>
                  <div v-else-if="!isTreeTabCollapsed(tab.id)" class="empty-inline">{{ t('workspace.noPaneInTab') }}</div>
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
                  <span class="tab-label">{{ t('workspace.panesCount', { count: countLeafPanes(tab.panes) }) }} · {{ t('snapshot.terminals', { count: countPaneSessions(tab.panes) }) }}</span>
                </div>
                <div class="tab-block__actions">
                  <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeTab(tab.id)">{{ t('workspace.deleteTab') }}</button>
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
                        {{ paneHasRunningSession(pane) ? t('ai.states.running') : t('workspace.standby') }}
                      </span>
                    </div>
                    <span class="pane-line__path">{{ entryById(pane.terminalEntryId)?.workingDirectory || pane.pathLabel }}</span>
                  </div>
                  <div class="pane-line__meta">
                    <span class="meta-inline">
                      <span class="meta-inline__label">{{ t('workspace.defaultCommand') }}</span>
                      <code class="meta-inline__value">{{ entryById(pane.terminalEntryId)?.defaultCommand || t('workspace.notSet') }}</code>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">{{ t('workspace.lastCommand') }}</span>
                      <code class="meta-inline__value meta-inline__value--muted">{{ entryById(pane.terminalEntryId)?.lastCommand || t('workspace.noneValue') }}</code>
                    </span>
                    <span class="meta-inline">
                      <span class="meta-inline__label">{{ t('workspace.launchMode') }}</span>
                      <span class="meta-inline__value meta-inline__value--accent">{{ launchModeLabel(entryById(pane.terminalEntryId)?.launchMode) }}</span>
                    </span>
                    <span v-if="entryById(pane.terminalEntryId)?.tags?.length" class="meta-inline meta-inline--tags">
                      <span class="meta-inline__label">{{ t('workspace.tags') }}</span>
                      <span class="meta-inline__value meta-inline__value--muted">{{ entryById(pane.terminalEntryId)?.tags.join(' · ') }}</span>
                    </span>
                  </div>
                </div>
              </div>

              <div v-else class="empty-inline empty-inline--block">{{ t('workspace.noPaneInTab') }}</div>
            </article>

          </div>
        </section>

        <section class="runtime-layout" v-else-if="appSection === 'workspace' && workspaceView === 'runtime' && selectedWorkspace">
          <div class="runtime-header">
            <div>
              <h2>{{ t('workspace.runtimeTitle') }}</h2>
              <p>{{ t('workspace.runtimeIntro') }}</p>
            </div>
            <div class="runtime-header__actions">
              <div class="layout-toggle-group">
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'grid' }" @click="setActiveTabLayout('grid')">
                  <AppIcon name="workspace" :size="14" />
                  <span>{{ t('workspace.layoutGrid') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'horizontal' }" @click="setActiveTabLayout('horizontal')">
                  <AppIcon name="copy" :size="14" />
                  <span>{{ t('workspace.layoutHorizontal') }}</span>
                </button>
                <button class="ghost-btn ghost-btn--small" :class="{ 'ghost-btn--active': activeRuntimeTab?.layoutMode === 'vertical' }" @click="setActiveTabLayout('vertical')">
                  <AppIcon name="pane" :size="14" />
                  <span>{{ t('workspace.layoutVertical') }}</span>
                </button>
              </div>
              <button class="ghost-btn" @click="createTab()">
                <AppIcon name="tab" :size="15" />
                <span>{{ t('workspace.newTab') }}</span>
              </button>
              <button class="ghost-btn" @click="createPane()">
                <AppIcon name="pane" :size="15" />
                <span>{{ t('workspace.newPane') }}</span>
              </button>
              <button class="ghost-btn" :disabled="!activeRuntimeTab || !countLeafPanes(activeRuntimeTab?.panes || [])" @click="saveActiveTabAsWorkflowTemplate()">
                <AppIcon name="copy" :size="15" />
                <span>{{ t('workspace.saveAsTemplate') }}</span>
              </button>
              <button class="ghost-btn" @click="openHelpDrawer = true">
                <AppIcon name="info" :size="15" />
                <span>{{ t('workspace.layoutGuide') }}</span>
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
                  <button class="icon-btn" :title="t('common.actions.openDirectory')" @click="openPaneDirectory(pane)">
                    <AppIcon name="folder" :size="15" />
                  </button>
                  <button class="icon-btn" :title="t('common.actions.copyPath')" @click="copyPanePath(pane)">
                    <AppIcon name="copy" :size="15" />
                  </button>
                  <button class="icon-btn" :title="t('common.actions.splitRight')" @click="splitLeafPane(pane.id, 'horizontal')">
                    <AppIcon name="pane" :size="15" />
                  </button>
                  <div class="pane__more-wrap">
                    <button class="icon-btn" :title="t('common.actions.moreActions')" @click.stop="togglePaneMenu(pane.id)">
                      <AppIcon name="more" :size="15" />
                    </button>
                    <PopoverMenu :open="activePaneMenu === pane.id" :items="paneMenuItems(pane)" />
                  </div>
                </div>
              </div>

                <div class="pane__statusbar">
                  <div class="pane__status-meta">
                    <span class="status-badge" :class="{ 'status-badge--running': paneHasRunningSession(pane) }">
                      {{ paneHasRunningSession(pane) ? t('ai.states.running') : t('workspace.standby') }}
                    </span>
                    <span>{{ splitLabel(pane.splitDirection) }}</span>
                  </div>
                  <div v-if="entryById(pane.terminalEntryId)?.tags?.length" class="pane__tags">
                    <span v-for="tag in entryById(pane.terminalEntryId)?.tags || []" :key="`${pane.id}-${tag}`" class="tag-chip tag-chip--soft">{{ tag }}</span>
                  </div>
                  <div class="pane__binding">
                    <span>{{ t('common.actions.runConfigs') }}</span>
                    <div class="pane__binding-wrap">
                    <button
                      type="button"
                      class="binding-trigger"
                      :class="{ 'binding-trigger--active': !!pane.terminalEntryId }"
                      @click.stop="togglePaneBindingMenu(pane.id)"
                    >
                      <span>{{ entryById(pane.terminalEntryId)?.name || t('workspace.unboundConfig') }}</span>
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
                      <strong>{{ t('common.actions.runConfigs') }}</strong>
                      <span>{{ entryById(pane.terminalEntryId)?.name || t('workspace.unboundConfigLong') }}</span>
                    </div>
                    <button type="button" class="icon-btn icon-btn--bolt" :title="t('common.actions.quickCommands')" @click.stop="activeCommandPanelPaneId = activeCommandPanelPaneId === pane.id ? null : pane.id">
                      <AppIcon name="bolt" :size="15" />
                    </button>
                  </div>
                  <div v-if="activeCommandPanelPaneId === pane.id" class="command-panel">
                    <div class="command-panel__section">
                      <span class="command-panel__title">{{ t('workspace.defaultCommand') }}</span>
                      <code>{{ entryById(pane.terminalEntryId)?.defaultCommand || t('workspace.notConfigured') }}</code>
                    </div>
                    <div class="command-panel__section">
                      <span class="command-panel__title">{{ t('workspace.lastCommand') }}</span>
                      <code>{{ entryById(pane.terminalEntryId)?.lastCommand || t('workspace.notRecorded') }}</code>
                    </div>
                    <div class="command-panel__section" v-if="recentCommandsByPane[pane.id]?.length">
                      <span class="command-panel__title">{{ tr('最近命令', 'Recent commands') }}</span>
                      <button v-for="command in recentCommandsByPane[pane.id]" :key="`${pane.id}-${command}`" type="button" class="command-chip" @click.stop="insertPaneText(pane, command)">
                        <span>{{ command }}</span>
                      </button>
                    </div>
                  </div>
                </div>
                <div class="terminal-lines" :style="terminalPreviewStyle">
                  <div>
                    <span class="terminal-key"># {{ t('common.actions.runConfigs') }}</span>
                    <span class="terminal-value terminal-value--accent">{{ entryById(pane.terminalEntryId)?.name || t('workspace.unboundConfig') }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># {{ t('workspace.defaultCommand') }}</span>
                    <span class="terminal-value">{{ entryById(pane.terminalEntryId)?.defaultCommand || t('workspace.notSet') }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># {{ t('workspace.lastCommand') }}</span>
                    <span class="terminal-value terminal-value--muted">{{ entryById(pane.terminalEntryId)?.lastCommand || t('workspace.noneValue') }}</span>
                  </div>
                  <div>
                    <span class="terminal-key"># {{ t('workspace.launchMode') }}</span>
                    <span class="terminal-value terminal-value--accent">{{ launchModeLabel(entryById(pane.terminalEntryId)?.launchMode) }}</span>
                  </div>
                  <div class="terminal-space"></div>
                  <div v-if="pane.terminalEntryId" class="terminal-hint">
                    <AppIcon name="info" :size="14" />
                    <span>{{ t('workspace.boundConfigHint') }}</span>
                  </div>
                  <div v-else class="terminal-hint">
                    <AppIcon name="info" :size="14" />
                    <span>{{ t('workspace.unboundConfigHint') }}</span>
                  </div>
                </div>
              </div>
            </section>

            <section v-if="!countLeafPanes(activeRuntimeTab?.panes || [])" class="pane pane--empty">
              <div class="pane__body">{{ t('workspace.noPaneRuntime') }}</div>
            </section>
          </div>

          <div class="runtime-footer">
            <div>{{ t('workspace.currentWorkspace', { name: selectedWorkspace.name }) }}</div>
            <div>{{ t('workspace.currentTab', { name: activeRuntimeTab?.name || t('common.none') }) }}</div>
            <div>{{ t('workspace.paneCount', { count: countLeafPanes(activeRuntimeTab?.panes || []) }) }}</div>
          </div>
        </section>

        <section class="p45-page p45-page--recent" v-else-if="appSection === 'recent'">
          <header class="p45-hero p45-hero--recent panel">
            <div class="p45-hero__copy">
              <h2>{{ t('recent.title') }}</h2>
              <p>{{ t('recent.intro') }}</p>
            </div>
            <div class="p45-hero__stats">
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="recent" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters[0]?.count ?? 0 }}</strong>
                  <small>{{ t('recent.records') }}</small>
                </div>
              </div>
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="bolt" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters.find((item) => item.id === 'command')?.count ?? 0 }}</strong>
                  <small>{{ t('recent.commandHistory') }}</small>
                </div>
              </div>
              <div class="p45-stat-card">
                <span class="p45-stat-card__icon"><AppIcon name="copy" :size="14" /></span>
                <div class="p45-stat-card__body">
                  <strong>{{ recentFilters.find((item) => item.id === 'snapshot')?.count ?? 0 }}</strong>
                  <small>{{ t('recent.snapshots') }}</small>
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
                  <span>{{ t('recent.workspaceFilter') }}</span>
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
              <span>{{ t('recent.recycleBin', { count: hiddenRecentItemCount }) }}</span>
            </button>
          </div>

          <section class="p45-list panel" :class="recentFilter === 'snapshot' ? 'p45-list--snapshots' : 'p45-list--records'">
            <div v-if="recentFilter === 'command'" class="p45-hint-card">
              <div class="p45-hint-card__icon">
                <AppIcon name="bolt" :size="16" />
              </div>
              <div class="p45-hint-card__body">
                <strong>{{ t('recent.commandHintTitle') }}</strong>
                <p>{{ t('recent.commandHintDesc') }}</p>
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
                    <span v-if="recentItemIsPinned(item.id)" class="p45-row__pin-label"><AppIcon name="star" :size="11" />{{ t('recent.pinned') }}</span>
                    <strong>{{ item.title }}</strong>
                    <small>{{ item.description }}</small>
                    <em>{{ item.meta }}</em>
                    <span class="p45-row__snapshot-metrics">
                      <span class="meta-badge meta-badge--soft">{{ t('recent.bestWorkspace', { count: item.previewTabs.length }) }}</span>
                      <span class="meta-badge meta-badge--soft">{{ t('recent.bestSessions', { count: item.previewTabs.reduce((count, tab) => count + countPaneSessions(tab.panes), 0) }) }}</span>
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
                    <span>{{ recentItemIsPinned(item.id) ? t('recent.unpin') : t('recent.pin') }}</span>
                  </button>
                  <span class="meta-badge meta-badge--soft">{{ item.badge }}</span>
                  <button type="button" class="ghost-btn ghost-btn--danger ghost-btn--small" @click.stop="removeWorkspaceSnapshot(item.workspaceId || '', item.snapshotId || '')">
                    <AppIcon name="trash" :size="13" />
                    <span>{{ t('recent.deleteSnapshot') }}</span>
                  </button>
                </span>
              </template>
              <template v-else>
                <span class="p45-row__icon"><AppIcon :name="item.icon" :size="15" /></span>
                <span class="p45-row__body">
                  <span v-if="recentItemIsPinned(item.id)" class="p45-row__pin-label"><AppIcon name="star" :size="11" />{{ t('recent.pinned') }}</span>
                  <strong>{{ item.title }}</strong>
                  <small>{{ item.description }}</small>
                  <em>{{ item.meta }}</em>
                  <span v-if="item.command && item.sourceSessionLabel" class="p45-row__meta-badges">
                    <span class="meta-badge meta-badge--soft">{{ t('recent.source', { label: item.sourceSessionLabel }) }}</span>
                  </span>
                </span>
                <span class="p45-row__actions">
                  <button type="button" class="ghost-btn ghost-btn--small" @click.stop="togglePinRecentItem(item.id)">
                    <AppIcon :name="recentItemIsPinned(item.id) ? 'star' : 'recent'" :size="13" />
                    <span>{{ recentItemIsPinned(item.id) ? t('recent.unpin') : t('recent.pin') }}</span>
                  </button>
                  <span class="meta-badge meta-badge--soft">{{ item.badge }}</span>
                  <button v-if="item.type !== 'workspace'" type="button" class="ghost-btn ghost-btn--small" @click.stop="hideRecentItem(item.id)">
                    <AppIcon name="close" :size="13" />
                    <span>{{ t('recent.remove') }}</span>
                  </button>
                    <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="copyCommandText(item.command || '')">
                      <AppIcon name="copy" :size="13" />
                      <span>{{ t('search.copyCommand') }}</span>
                    </button>
                  <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="openRecentCommandTarget(item.workspaceId, item.entryId, item.command)">
                    <AppIcon name="recent" :size="13" />
                    <span>{{ t('recent.backToSource') }}</span>
                  </button>
                    <button v-if="item.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="insertRecentCommand(item)">
                      <AppIcon name="terminal" :size="13" />
                      <span>{{ t('search.insertCurrentInput') }}</span>
                    </button>
                </span>
              </template>
            </article>

            <div v-if="!filteredRecentItems.length" class="empty-state">
              <div class="empty-state__icon"><AppIcon name="recent" :size="18" /></div>
              <div class="empty-state__body">
                <strong>{{ t('recent.noItemsTitle') }}</strong>
                <p>{{ recentFilter === 'command' ? t('recent.noCommandsDesc') : t('recent.noItemsDesc') }}</p>
              </div>
            </div>
          </section>
        </section>

        <section class="p45-page p45-page--templates" v-else-if="appSection === 'templates'">
          <header class="p45-hero p45-hero--templates panel">
            <div class="p45-hero__copy">
              <h2>{{ t('templates.title') }}</h2>
              <p>{{ t('templates.intro') }}</p>
              <small v-if="templateApplyTargetWorkspace" class="p45-hero__hint">{{ t('templates.defaultTarget', { name: templateApplyTargetWorkspace.name, path: templateApplyTargetWorkspace.rootPath }) }}</small>
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
                <span>{{ tag === 'all' ? t('templates.allTags') : tag }}</span>
              </button>
            </div>
          </div>

          <section class="template-grid">
            <article v-for="template in filteredWorkflowTemplates" :key="template.id" class="workflow-template-card panel">
              <div class="workflow-template-card__head" :class="{ 'workflow-template-card__head--system': template.kind === 'system', 'workflow-template-card__head--user': template.kind === 'user' }">
                <div>
                  <span class="meta-badge" :class="{ 'meta-badge--soft': template.kind === 'user' }">{{ template.kind === 'system' ? t('templates.systemTemplate') : t('templates.userTemplate') }}</span>
                  <h3>{{ template.name }}</h3>
                  <p>{{ template.description }}</p>
                </div>
                <span class="workflow-template-card__count">{{ t('templates.terminals', { count: template.panes.length }) }}</span>
              </div>
              <div class="workflow-template-card__panes">
                <div v-for="pane in template.panes" :key="`${template.id}-${pane.name}`" class="template-pane-line">
                  <AppIcon name="terminal" :size="14" />
                  <span>{{ pane.name }}</span>
                  <code>{{ pane.defaultCommand || t('templates.noAutoCommand') }}</code>
                </div>
              </div>
              <div class="workflow-template-card__tags">
                <span v-for="tag in template.tags" :key="`${template.id}-${tag}`" class="tag-chip tag-chip--soft">{{ tag }}</span>
              </div>
              <div class="workflow-template-card__actions">
                <button type="button" class="ghost-btn ghost-btn--primary ghost-btn--small" @click="applyWorkflowTemplate(template)">
                  <AppIcon name="plus" :size="13" />
                  <span>{{ t('templates.apply') }}</span>
                </button>
                <button type="button" class="ghost-btn ghost-btn--small" @click="duplicateWorkflowTemplate(template.id)">
                  <AppIcon name="copy" :size="13" />
                  <span>{{ t('templates.duplicate') }}</span>
                </button>
                <button v-if="template.kind === 'user'" type="button" class="ghost-btn ghost-btn--small" @click="openWorkflowTemplateEditModal(template.id)">
                  <AppIcon name="edit" :size="13" />
                  <span>{{ t('templates.edit') }}</span>
                </button>
                <button v-if="template.kind === 'user'" type="button" class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeWorkflowTemplate(template.id)">
                  <AppIcon name="trash" :size="13" />
                  <span>{{ t('templates.delete') }}</span>
                </button>
              </div>
            </article>
          </section>
        </section>

        <section class="p45-page p45-page--providers" v-else-if="appSection === 'providers'">
          <header class="p45-hero p45-hero--providers">
            <div class="p45-hero__copy">
              <h2>{{ t('provider.title') }}</h2>
            </div>
            <div class="p45-hero__actions">
              <button type="button" class="ghost-btn" :disabled="providerDetectionRunning" @click="syncNativeProviderProfiles({ explicit: true })">
                <AppIcon :name="providerDetectionRunning && providerSyncMode === 'native' ? 'refresh' : 'search'" :size="14" :class="{ 'is-spinning': providerDetectionRunning && providerSyncMode === 'native' }" />
                <span>{{ providerDetectionRunning && providerSyncMode === 'native' ? tr('同步中', 'Syncing') : tr('同步本机配置', 'Sync Local') }}</span>
              </button>
              <button type="button" class="ghost-btn" :disabled="providerDetectionRunning" @click="importCcSwitchProviderProfiles()">
                <AppIcon :name="providerDetectionRunning && providerSyncMode === 'cc-switch' ? 'refresh' : 'download'" :size="14" :class="{ 'is-spinning': providerDetectionRunning && providerSyncMode === 'cc-switch' }" />
                <span>{{ providerDetectionRunning && providerSyncMode === 'cc-switch' ? tr('导入中', 'Importing') : tr('从 CC Switch 导入', 'Import CC Switch') }}</span>
              </button>
              <button type="button" class="ghost-btn" @click="openModelPricingModal()">
                <AppIcon name="dollar" :size="14" />
                <span>{{ tr('模型价格', 'Model Pricing') }}</span>
              </button>
              <button type="button" class="ghost-btn ghost-btn--primary ghost-btn--small" @click="openProviderCreateModal()">
                <AppIcon name="plus" :size="13" />
                <span>{{ t('provider.newProfile') }}</span>
              </button>
            </div>
          </header>

          <!-- 固定高度状态条：未扫描/扫描中/完成后都不改变布局高度 -->
          <div class="provider-status-rail" aria-live="polite">
            <div class="provider-status-rail__row">
              <AppIcon :name="providerDetectionRunning ? 'refresh' : 'info'" :size="12" :class="{ 'is-spinning': providerDetectionRunning }" />
              <span class="provider-status-rail__text">{{ providerStatusRailText }}</span>
            </div>
            <p class="provider-metrics-scope-note" :title="providerMetricsScopeHint">
              <span v-if="managedUsageLive">
                {{ tr('卡片副指标 = 当前周期', 'Card secondary metrics = current period') }}
                <strong>{{ providerMetricsScopeShort }}</strong>
                · {{ tr('今日按自然日；副指标非全历史', 'Today is calendar day; secondary is not all-time') }}
              </span>
              <span v-else>
                {{ tr('副指标暂用本地缓存；同步或进入 Usage 后改为当前周期', 'Secondary metrics use cache until sync / Usage load') }}
              </span>
            </p>
          </div>

          <!-- Filters bar: kind tabs + search -->
          <div class="provider-filters-bar">
            <div class="provider-kind-tabs">
              <button
                v-for="tool in providerToolFilters"
                :key="tool.id"
                type="button"
                class="provider-kind-tab"
                :class="{ 'provider-kind-tab--active': activeProviderToolFilter === tool.id }"
                @click="activeProviderToolFilter = tool.id"
              >
                {{ tool.label }}<span v-if="tool.count > 0">{{ tool.count }}</span>
              </button>
            </div>
            <div class="provider-search-field">
              <AppIcon name="search" :size="13" />
              <input v-model.trim="providerSearchQuery" type="text" :placeholder="t('provider.searchPlaceholder')" />
            </div>
          </div>

          <!-- Full-width card list -->
          <div class="provider-card-list">
            <template v-if="filteredProviderCards.length">
              <div
                v-for="card in filteredProviderCards"
                :key="card.provider.id"
                class="provider-row-card"
                :class="{ 'provider-row-card--active': Boolean(card.provider.isActive) }"
              >
                <div class="provider-row-card__main">
                  <span
                    class="provider-row-card__icon"
                    v-html="providerKindSvgIcon(card.provider.providerKind)"
                  />

                  <!-- 左栏：图标+主信息固定宽度，不把指标顶到屏最右 -->
                  <div class="provider-row-card__info">
                    <div class="provider-row-card__line1">
                      <strong class="provider-row-card__name" :title="card.provider.name">{{ card.provider.name }}</strong>
                      <span class="provider-row-card__kind-tag">{{ providerKindLabel(card.provider.providerKind) }}</span>
                    </div>
                    <div class="provider-row-card__line2">
                      <span
                        v-if="card.provider.isActive || card.provider.status === 'missing' || card.provider.status === 'disabled'"
                        class="provider-status-badge"
                        :class="card.provider.isActive ? 'provider-status-badge--active' : card.provider.status === 'missing' ? 'provider-status-badge--warn' : 'provider-status-badge--disabled'"
                      ><span class="provider-status-dot"></span>{{ card.provider.isActive ? t('provider.statusActive') : providerStatusLabel(card.provider.status) }}</span>
                      <span v-if="card.url" class="provider-row-card__url-inline" :title="card.provider.homepageUrl ?? card.provider.requestBaseUrl ?? undefined">{{ card.url }}</span>
                      <span v-if="card.provider.defaultModel" class="provider-row-card__model-tag">{{ card.provider.defaultModel }}</span>
                    </div>
                  </div>

                  <!-- 紧凑指标组：紧跟主信息 -->
                  <div class="provider-row-card__metrics" :title="providerMetricsScopeHint">
                    <div class="provider-metric">
                      <span class="provider-metric__label">
                        <AppIcon name="activity" :size="11" />{{ tr('今日请求', 'Req Today') }}
                      </span>
                      <span class="provider-metric__value" :class="card.metrics.todayRequests > 0 ? 'provider-metric__value--accent' : ''">{{ card.metrics.todayRequests.toLocaleString(currentLocale) }}</span>
                      <span class="provider-metric__sub">
                        <span class="provider-metric__scope">{{ providerMetricsScopeShort }}</span>
                        {{ card.metrics.totalRequests.toLocaleString(currentLocale) }}
                      </span>
                    </div>
                    <div class="provider-metric">
                      <span class="provider-metric__label">
                        <AppIcon name="dollar" :size="11" />{{ tr('今日成本', 'Cost Today') }}
                      </span>
                      <span class="provider-metric__value" :class="card.metrics.todayCostUsd > 0 ? 'provider-metric__value--cost' : ''">${{ card.metrics.todayCostUsd.toFixed(2) }}</span>
                      <span class="provider-metric__sub">
                        <span class="provider-metric__scope">{{ providerMetricsScopeShort }}</span>
                        ${{ card.metrics.totalCostUsd.toFixed(2) }}
                      </span>
                    </div>
                    <div class="provider-metric">
                      <span class="provider-metric__label">
                        <AppIcon name="bolt" :size="11" />{{ tr('缓存率', 'Cache Hit') }}
                      </span>
                      <span
                        class="provider-metric__value"
                        :class="card.metrics.cacheHitRate >= 0.5 ? 'provider-metric__value--positive' : card.metrics.cacheHitRate > 0 ? 'provider-metric__value--amber' : ''"
                      >{{ Math.round(card.metrics.cacheHitRate * 1000) / 10 }}%</span>
                      <span class="provider-metric__sub">{{ providerMetricsScopeShort }}</span>
                    </div>
                    <div class="provider-metric">
                      <span class="provider-metric__label">
                        <AppIcon name="coin" :size="11" />{{ tr('余额', 'Balance') }}
                      </span>
                      <span
                        class="provider-metric__value"
                        :class="card.metrics.balance === null ? '' : card.metrics.balance > 0.01 ? 'provider-metric__value--positive' : card.metrics.balance < -0.001 ? 'provider-metric__value--negative' : 'provider-metric__value--zero'"
                      >{{ formatProviderRemainingBalance(card.metrics.balance) }}</span>
                    </div>
                  </div>

                  <div class="provider-row-card__spacer" aria-hidden="true" />

                  <div class="provider-row-card__actions">
                    <button
                      type="button"
                      class="prov-act-btn prov-act-btn--primary"
                      :disabled="!providerCanBeActivated(card.provider) || activatingProviderId === card.provider.id"
                      :title="providerActivationLabel(card.provider)"
                      @click.stop="activateProviderProfile(card.provider.id)"
                    >
                      <AppIcon
                        :name="activatingProviderId === card.provider.id ? 'refresh' : 'terminal'"
                        :size="16"
                        :class="{ 'is-spinning': activatingProviderId === card.provider.id }"
                      />
                    </button>
                    <button type="button" class="prov-act-btn" :title="t('common.actions.edit')" @click.stop="openProviderEditModal(card.provider.id)">
                      <AppIcon name="edit" :size="16" />
                    </button>
                    <button type="button" class="prov-act-btn" :title="t('provider.viewUsage')" @click.stop="openProviderUsageFilter(card.provider.id)">
                      <AppIcon name="activity" :size="16" />
                    </button>
                    <button type="button" class="prov-act-btn" :title="t('common.actions.duplicate')" @click.stop="duplicateProviderProfile(card.provider.id)">
                      <AppIcon name="copy" :size="16" />
                    </button>
                    <button type="button" class="prov-act-btn prov-act-btn--danger" :title="t('common.actions.delete')" @click.stop="removeProviderProfile(card.provider.id)">
                      <AppIcon name="trash" :size="16" />
                    </button>
                  </div>
                </div>
              </div>
            </template>
            <div v-else class="provider-card-list__empty">
              <AppIcon name="settings" :size="22" />
              <strong>{{ t('provider.noProfilesTitle') }}</strong>
              <p>{{ tr('扫描本地工具，或手动新建档案。', 'Scan local tools, or create a profile manually.') }}</p>
            </div>
          </div>
        </section>

        <section class="p45-page p45-page--usage" v-else-if="appSection === 'usage'">
          <header class="p45-hero p45-hero--usage">
            <div class="usage-page-header">
              <div class="usage-page-header__title">
                <h2>{{ t('usage.title') }}</h2>
                <p class="usage-hero-context">
                  <template v-if="activeProviderProfile">
                    {{ providerKindLabel(activeProviderProfile.providerKind) }} · {{ activeProviderProfile.name }}
                  </template>
                  <template v-else>{{ tr('当前工作区 · 所有 Provider', 'Current workspace · All providers') }}</template>
                  <template v-if="managedUsageUpdatedAt">
                    · {{ tr('更新于', 'Updated') }} {{ formatUsageDate(managedUsageUpdatedAt.includes('T') ? managedUsageUpdatedAt : new Date(Number(managedUsageUpdatedAt) * 1000 || Date.now()).toISOString()) }}
                  </template>
                  <template v-else-if="managedUsageLoadError">
                    · {{ tr('实时数据读取失败，显示本地缓存', 'Live fetch failed, showing local cache') }}
                  </template>
                </p>
              </div>
              <div class="usage-page-header__controls">
                <div class="usage-period-tabs">
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === '1h' }" @click="usagePeriodFilter = '1h'">1h</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === 'today' }" @click="usagePeriodFilter = 'today'">{{ t('usage.today') }}</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === '7d' }" @click="usagePeriodFilter = '7d'">7d</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === '30d' }" @click="usagePeriodFilter = '30d'">30d</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === 'month' }" @click="usagePeriodFilter = 'month'">{{ tr('本月', 'Month') }}</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === '90d' }" @click="usagePeriodFilter = '90d'">90d</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === 'all' }" @click="usagePeriodFilter = 'all'">{{ t('common.all') }}</button>
                  <button type="button" class="usage-period-tab" :class="{ 'usage-period-tab--active': usagePeriodFilter === 'custom' }" @click="usagePeriodFilter = 'custom'">{{ tr('自定义', 'Custom') }}</button>
                </div>
                <button type="button" class="ghost-btn" @click="openModelPricingModal()">
                  <AppIcon name="dollar" :size="14" />
                  <span>{{ tr('模型价格', 'Model Pricing') }}</span>
                </button>
                <button type="button" class="ghost-btn" :disabled="providerUsageRefreshRunning || usageManualRefreshRunning" @click="refreshProviderUsageStats()">
                  <AppIcon name="refresh" :size="14" :class="{ 'is-spinning': providerUsageRefreshRunning || usageManualRefreshRunning }" />
                  <span>{{ (providerUsageRefreshRunning || usageManualRefreshRunning) ? tr('刷新中', 'Refreshing') : tr('刷新', 'Refresh') }}</span>
                </button>
              </div>
              <div v-if="usagePeriodFilter === 'custom'" class="usage-custom-range">
                <label>
                  <span>{{ tr('开始', 'Start') }}</span>
                  <input v-model="usageCustomStartAt" type="datetime-local" />
                </label>
                <label>
                  <span>{{ tr('结束', 'End') }}</span>
                  <input v-model="usageCustomEndAt" type="datetime-local" />
                </label>
              </div>
              <p class="usage-bucket-hint">
                {{ tr(
                  `趋势按时间范围自动聚合（当前：${usageEffectiveBucket}）`,
                  `Trend auto-buckets by range (now: ${usageEffectiveBucket})`,
                ) }}
              </p>
            </div>
          </header>

          <section class="usage-dashboard">
            <!-- KPI tiles row: 4 key metrics -->
            <div class="usage-kpi-row">
              <div class="usage-kpi-tile usage-kpi-tile--tokens">
                <span class="usage-kpi-tile__label">
                  <AppIcon name="activity" :size="11" />{{ tr('实际 Tokens', 'Real Tokens') }}
                </span>
                <span class="usage-kpi-tile__value">{{ formatCompactWan(activeUsageSummary.totalInputTokens + activeUsageSummary.totalOutputTokens + activeUsageSummary.totalCacheReadTokens + activeUsageSummary.totalCacheCreationTokens) }}</span>
                <span class="usage-kpi-tile__sub">
                  {{ tr('输入', 'In') }} {{ formatCompactWan(activeUsageSummary.totalInputTokens) }}
                  · {{ tr('输出', 'Out') }} {{ formatCompactWan(activeUsageSummary.totalOutputTokens) }}
                </span>
              </div>
              <div class="usage-kpi-tile usage-kpi-tile--requests">
                <span class="usage-kpi-tile__label">
                  <AppIcon name="tab" :size="11" />{{ t('usage.totalRequests') }}
                </span>
                <span class="usage-kpi-tile__value">{{ activeUsageSummary.totalRequests.toLocaleString(currentLocale) }}</span>
                <span class="usage-kpi-tile__sub">{{ tr('次请求', 'requests') }}</span>
              </div>
              <div class="usage-kpi-tile usage-kpi-tile--cost">
                <span class="usage-kpi-tile__label">
                  <AppIcon name="dollar" :size="11" />{{ t('usage.totalCost') }}
                </span>
                <span class="usage-kpi-tile__value">${{ activeUsageSummary.totalCostUsd.toFixed(2) }}</span>
                <span class="usage-kpi-tile__sub">${{ activeUsageSummary.totalCostUsd.toFixed(4) }} {{ tr('精确', 'precise') }}</span>
              </div>
              <div class="usage-kpi-tile usage-kpi-tile--cache">
                <span class="usage-kpi-tile__label">
                  <AppIcon name="bolt" :size="11" />{{ t('usage.hitRate') }}
                </span>
                <span class="usage-kpi-tile__value">{{ Math.round(activeUsageSummary.cacheHitRate * 1000) / 10 }}%</span>
                <div class="usage-kpi-hit-bar">
                  <div class="usage-kpi-hit-bar__fill" :style="{ width: `${Math.min(100, activeUsageSummary.cacheHitRate * 100).toFixed(1)}%` }" />
                </div>
              </div>
            </div>

            <article class="usage-chart-card panel">
              <div class="usage-chart-card__head">
                <div class="usage-chart-card__title-row">
                  <strong>{{ t('usage.chartTitle') }}</strong>
                  <div class="usage-provider-filter">
                    <button
                      type="button"
                      class="usage-scope-tab"
                      :class="{ 'usage-scope-tab--active': usageSelectedProviderIds.length === 0 }"
                      @click="clearUsageProviderSelection()"
                    >{{ tr('全部 Provider', 'All Providers') }}</button>
                    <div class="usage-provider-picker">
                      <button
                        ref="usageProviderPickerTriggerRef"
                        type="button"
                        class="usage-provider-picker__trigger"
                        :class="{ 'usage-provider-picker__trigger--open': usageProviderPickerOpen }"
                        @click.stop="toggleUsageProviderPicker()"
                      >
                        <AppIcon name="search" :size="13" />
                        <span>{{ usageSelectedProviderIds.length ? tr(`已选 ${usageSelectedProviderIds.length}`, `${usageSelectedProviderIds.length} selected`) : tr('筛选 Provider', 'Filter Providers') }}</span>
                        <AppIcon name="chevron-down" :size="12" />
                      </button>
                      <Teleport to="body">
                        <div
                          v-if="usageProviderPickerOpen"
                          class="usage-provider-picker__panel usage-provider-picker__panel--fixed"
                          :style="usageProviderPickerStyle"
                          @click.stop
                          @pointerdown.stop
                          @mousedown.stop
                        >
                          <input
                            v-model.trim="usageProviderPickerQuery"
                            class="usage-provider-picker__search"
                            type="search"
                            :placeholder="tr('搜索名称 / 模型', 'Search name / model')"
                            @click.stop
                            @pointerdown.stop
                            @mousedown.stop
                          />
                          <div class="usage-provider-picker__actions">
                            <button type="button" class="ghost-btn ghost-btn--small" @click="clearUsageProviderSelection()">{{ tr('清空', 'Clear') }}</button>
                            <button type="button" class="ghost-btn ghost-btn--small" @click="closeUsageProviderPicker()">{{ tr('完成', 'Done') }}</button>
                          </div>
                          <div class="usage-provider-picker__scroll">
                            <div
                              v-for="group in usageProviderPickerGroups"
                              :key="group.id"
                              class="usage-provider-picker__group"
                            >
                              <div class="usage-provider-picker__group-title">{{ group.label }}</div>
                              <label
                                v-for="item in group.items"
                                :key="item.id"
                                class="usage-provider-picker__item"
                              >
                                <input
                                  type="checkbox"
                                  :checked="usageSelectedProviderIds.includes(item.id)"
                                  @change="toggleUsageProviderSelection(item.id)"
                                />
                                <span class="usage-provider-picker__icon" v-html="providerKindSvgIcon(item.providerKind)" />
                                <span class="usage-provider-picker__meta">
                                  <strong>{{ item.name }}</strong>
                                  <small>{{ item.model || '—' }} · {{ item.source }}</small>
                                </span>
                              </label>
                              <div v-if="!group.items.length" class="usage-provider-picker__empty">{{ tr('无匹配', 'No match') }}</div>
                            </div>
                          </div>
                        </div>
                      </Teleport>
                    </div>
                  </div>
                </div>
                <div v-if="usageSelectedProviderChips.length" class="usage-provider-chips">
                  <button
                    v-for="chip in usageSelectedProviderChips"
                    :key="chip.id"
                    type="button"
                    class="usage-provider-chip"
                    :title="tr('移除筛选', 'Remove filter')"
                    @click="toggleUsageProviderSelection(chip.id)"
                  >
                    <span class="usage-provider-chip__icon" v-html="providerKindSvgIcon(chip.providerKind)" />
                    <span>{{ chip.name }}</span>
                    <span class="usage-provider-chip__x">×</span>
                  </button>
                </div>
              </div>
              <div ref="usageChartShellRef" class="usage-chart-shell">
                <template v-if="usageTrendChartData.length">
                  <div
                    v-if="chartHoverIndex !== null && usageTrendChartData[chartHoverIndex]"
                    ref="usageChartTooltipRef"
                    class="usage-chart-tooltip"
                    :style="usageChartTooltipStyle"
                  >
                    <strong>{{ formatUsageTrendTooltipTime(usageTrendChartData[chartHoverIndex].timestamp) }}</strong>
                    <span class="usage-chart-tooltip__row usage-chart-tooltip__row--input">
                      {{ tr('输入', 'In') }} {{ formatCompactWan(usageTrendChartData[chartHoverIndex].inputTokens) }}
                    </span>
                    <span class="usage-chart-tooltip__row usage-chart-tooltip__row--output">
                      {{ tr('输出', 'Out') }} {{ formatCompactWan(usageTrendChartData[chartHoverIndex].outputTokens) }}
                    </span>
                    <span class="usage-chart-tooltip__row usage-chart-tooltip__row--cache-create">
                      {{ tr('缓存创建', 'Cache Create') }} {{ formatCompactWan(usageTrendChartData[chartHoverIndex].cacheCreationTokens ?? 0) }}
                    </span>
                    <span class="usage-chart-tooltip__row usage-chart-tooltip__row--cache">
                      {{ tr('缓存命中', 'Cache Hit') }} {{ formatCompactWan(usageTrendChartData[chartHoverIndex].cacheReadTokens) }}
                    </span>
                    <span class="usage-chart-tooltip__row usage-chart-tooltip__row--cost">
                      {{ tr('成本', 'Cost') }} ${{ usageTrendChartData[chartHoverIndex].costUsd.toFixed(4) }}
                    </span>
                  </div>
                  <svg
                    class="usage-trend-svg"
                    viewBox="0 0 1000 330"
                    preserveAspectRatio="xMidYMid meet"
                    :aria-label="tr('Provider 使用趋势图', 'Provider usage trend chart')"
                    @mousemove="onUsageChartMouseMove"
                    @mouseleave="chartHoverIndex = null"
                  >
                    <defs>
                      <linearGradient id="gradInput" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stop-color="#3b82f6" stop-opacity="0.16"/>
                        <stop offset="95%" stop-color="#3b82f6" stop-opacity="0"/>
                      </linearGradient>
                      <linearGradient id="gradOutput" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stop-color="#22c55e" stop-opacity="0.14"/>
                        <stop offset="95%" stop-color="#22c55e" stop-opacity="0"/>
                      </linearGradient>
                      <linearGradient id="gradCache" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stop-color="#a855f7" stop-opacity="0.14"/>
                        <stop offset="95%" stop-color="#a855f7" stop-opacity="0"/>
                      </linearGradient>
                    </defs>
                    <g class="usage-trend-y-labels">
                      <text v-for="lbl in usageTrendYLabels" :key="`yl-${lbl.y}`" x="48" :y="lbl.y + 4" text-anchor="end">{{ lbl.label }}</text>
                    </g>
                    <g class="usage-trend-y-labels usage-trend-y-labels--right">
                      <text v-for="lbl in usageTrendCostYLabels" :key="`yr-${lbl.y}`" x="958" :y="lbl.y + 4" text-anchor="start">{{ lbl.label }}</text>
                    </g>
                    <g class="usage-trend-grid">
                      <line x1="56" y1="52" x2="940" y2="52" />
                      <line x1="56" y1="110" x2="940" y2="110" />
                      <line x1="56" y1="168" x2="940" y2="168" />
                      <line x1="56" y1="226" x2="940" y2="226" />
                      <line x1="56" y1="284" x2="940" y2="284" />
                    </g>
                    <path :d="usageTrendCacheArea" fill="url(#gradCache)" />
                    <path :d="usageTrendInputArea" fill="url(#gradInput)" />
                    <path :d="usageTrendOutputArea" fill="url(#gradOutput)" />
                    <path class="usage-trend-line usage-trend-line--cache" :class="{ 'usage-trend-line--flat': !usageTrendHasSignal }" :d="usageTrendCachePath" fill="none" />
                    <path class="usage-trend-line usage-trend-line--input" :class="{ 'usage-trend-line--flat': !usageTrendHasSignal }" :d="usageTrendInputPath" fill="none" />
                    <path class="usage-trend-line usage-trend-line--output" :class="{ 'usage-trend-line--flat': !usageTrendHasSignal }" :d="usageTrendOutputPath" fill="none" />
                    <path class="usage-trend-line usage-trend-line--cost" :class="{ 'usage-trend-line--flat': !usageTrendHasSignal }" :d="usageTrendCostPath" fill="none" />
                    <line
                      v-if="chartHoverIndex !== null && usageTrendLabelPoints[chartHoverIndex]"
                      class="usage-trend-hover-line"
                      :x1="usageTrendLabelPoints[chartHoverIndex].x"
                      y1="48"
                      :x2="usageTrendLabelPoints[chartHoverIndex].x"
                      y2="286"
                    />
                    <template v-if="chartHoverIndex !== null && usageTrendChartData[chartHoverIndex]">
                      <circle :cx="usageTrendLabelPoints[chartHoverIndex]?.x" :cy="chartYForToken(usageTrendChartData[chartHoverIndex].cacheReadTokens)" r="3.5" class="usage-trend-dot usage-trend-dot--cache" />
                      <circle :cx="usageTrendLabelPoints[chartHoverIndex]?.x" :cy="chartYForToken(usageTrendChartData[chartHoverIndex].inputTokens)" r="3.5" class="usage-trend-dot usage-trend-dot--input" />
                      <circle :cx="usageTrendLabelPoints[chartHoverIndex]?.x" :cy="chartYForToken(usageTrendChartData[chartHoverIndex].outputTokens)" r="3.5" class="usage-trend-dot usage-trend-dot--output" />
                      <circle :cx="usageTrendLabelPoints[chartHoverIndex]?.x" :cy="chartYForCost(usageTrendChartData[chartHoverIndex].costUsd)" r="3.5" class="usage-trend-dot usage-trend-dot--cost" />
                    </template>
                    <g class="usage-trend-labels">
                      <text v-for="point in usageTrendLabelPoints.filter(p => p.show)" :key="`usage-label-${point.label}-${point.x}`" :x="point.x" y="308">{{ point.label }}</text>
                    </g>
                  </svg>
                  <!-- 唯一图例：底部居中 -->
                  <div class="usage-chart-legend-row">
                    <span class="usage-chart-legend-item usage-chart-legend-item--input">{{ t('usage.input') }}</span>
                    <span class="usage-chart-legend-item usage-chart-legend-item--output">{{ t('usage.output') }}</span>
                    <span class="usage-chart-legend-item usage-chart-legend-item--cache">{{ tr('缓存命中', 'Cache Hit') }}</span>
                    <span class="usage-chart-legend-item usage-chart-legend-item--cost">{{ t('usage.totalCost') }}</span>
                  </div>
                  <p v-if="!usageTrendHasSignal" class="usage-chart-zero-hint">
                    {{ tr(
                      '当前范围有时间点，但 Token / 成本均为 0：可能是筛选过窄、未匹配到档案，或后端未读到对应用量源。成本为 0 不一定是缺模型价格——Token 不依赖价格也能画线。',
                      'Buckets exist but all Token / cost values are 0: filter may be too narrow, profiles unmatched, or backend sources empty. Cost=0 is not only a pricing issue — token lines do not need model prices.',
                    ) }}
                  </p>
                </template>
                <div v-else class="usage-empty-state">
                  <AppIcon name="activity" :size="20" />
                  <strong>{{ t('usage.noTrendTitle') }}</strong>
                  <span>{{ t('usage.noTrendDesc') }}</span>
                </div>
              </div>
            </article>

            <article class="usage-log-card panel">
              <div class="usage-log-card__head">
                <div class="segmented-control segmented-control--secondary">
                  <button type="button" class="segmented-control__item" :class="{ 'segmented-control__item--active': usageView === 'requestLogs' }" @click="usageView = 'requestLogs'">
                    <span>{{ t('usage.requestLogs') }}</span>
                  </button>
                  <button type="button" class="segmented-control__item" :class="{ 'segmented-control__item--active': usageView === 'providerStats' }" @click="usageView = 'providerStats'">
                    <span>{{ t('usage.providerStats') }}</span>
                  </button>
                  <button type="button" class="segmented-control__item" :class="{ 'segmented-control__item--active': usageView === 'modelStats' }" @click="usageView = 'modelStats'">
                    <span>{{ t('usage.modelStats') }}</span>
                  </button>
                </div>
                <div class="usage-log-toolbar">
                  <button type="button" class="usage-log-filter" :class="{ 'usage-log-filter--active': usageAppFilter === 'all' }" @click="usageAppFilter = 'all'">{{ t('usage.allApps') }}</button>
                  <button type="button" class="usage-log-filter" :class="{ 'usage-log-filter--active': usageAppFilter === 'codex' }" @click="usageAppFilter = 'codex'">Codex</button>
                  <button type="button" class="usage-log-filter" :class="{ 'usage-log-filter--active': usageAppFilter === 'claude' }" @click="usageAppFilter = 'claude'">Claude</button>
                  <button type="button" class="usage-log-filter" :class="{ 'usage-log-filter--active': usageAppFilter === 'gemini' }" @click="usageAppFilter = 'gemini'">Gemini</button>
                  <button type="button" class="usage-log-filter" :class="{ 'usage-log-filter--active': usageAppFilter === 'hermes' }" @click="usageAppFilter = 'hermes'">Hermes</button>
                  <!-- OpenCode Usage 后端暂缓：禁用并标注，避免像可用筛选项 -->
                  <button
                    type="button"
                    class="usage-log-filter usage-log-filter--unavailable"
                    disabled
                    :title="tr('OpenCode Usage 暂未接入，后端独立阶段处理', 'OpenCode Usage is not available yet; backend support is deferred')"
                  >
                    OpenCode
                    <span class="usage-log-filter__badge">{{ tr('未接入', 'N/A') }}</span>
                  </button>
                  <div class="usage-toolbar-divider" />
                  <input
                    v-model.trim="usageSearchQuery"
                    class="usage-log-search"
                    type="search"
                    :placeholder="usageView === 'modelStats' ? t('usage.searchModel') : t('usage.searchProvider')"
                  />
                </div>
              </div>
              <div class="usage-log-table-wrap">
                <table class="usage-log-table">
                  <thead v-if="usageView === 'requestLogs'">
                    <tr>
                      <th>{{ t('usage.time') }}</th>
                      <th>{{ t('usage.provider') }}</th>
                      <th>{{ t('usage.billingModel') }}</th>
                      <th>{{ t('usage.input') }}</th>
                      <th>{{ t('usage.output') }}</th>
                      <th>{{ tr('缓存', 'Cache') }}</th>
                      <th>{{ t('usage.totalCostColumn') }}</th>
                      <th>{{ tr('首 token', 'TTFT') }}</th>
                      <th>{{ t('usage.duration') }}</th>
                      <th>{{ t('usage.status') }}</th>
                      <th>{{ t('usage.source') }}</th>
                    </tr>
                  </thead>
                  <thead v-else-if="usageView === 'providerStats'">
                    <tr>
                      <th>{{ t('usage.provider') }}</th>
                      <th>{{ t('usage.totalRequests') }}</th>
                      <th>{{ t('usage.input') }}</th>
                      <th>{{ t('usage.output') }}</th>
                      <th>{{ t('usage.totalCostColumn') }}</th>
                      <th>{{ t('usage.hitRate') }}</th>
                      <th>{{ tr('模型数', 'Models') }}</th>
                      <th>{{ t('usage.source') }}</th>
                      <th>{{ t('usage.time') }}</th>
                    </tr>
                  </thead>
                  <thead v-else>
                    <tr>
                      <th>{{ t('usage.billingModel') }}</th>
                      <th>{{ t('usage.totalRequests') }}</th>
                      <th>{{ t('usage.provider') }}</th>
                      <th>{{ t('usage.input') }}</th>
                      <th>{{ t('usage.output') }}</th>
                      <th>{{ t('usage.totalCostColumn') }}</th>
                      <th>{{ t('usage.source') }}</th>
                      <th>{{ t('usage.time') }}</th>
                    </tr>
                  </thead>
                  <tbody v-if="usageView === 'requestLogs' && activeUsageLogs.length">
                    <tr v-for="log in activeUsageLogs" :key="log.id">
                      <td>{{ formatUsageDate(log.createdAt) }}</td>
                      <td class="usage-provider-cell">
                        <span class="usage-provider-icon" v-html="providerKindSvgIcon(usageLogProviderKind(log))" />
                        <span :title="log.managedProviderId || undefined">{{ usageLogProviderName(log) }}</span>
                      </td>
                      <td>{{ log.model || log.pricingModel || '—' }}</td>
                      <td>{{ log.inputTokens.toLocaleString(currentLocale) }}</td>
                      <td>{{ log.outputTokens.toLocaleString(currentLocale) }}</td>
                      <td :title="tr('读缓存 / 写缓存', 'Cache read / write')">
                        {{ (log.cacheReadTokens ?? 0).toLocaleString(currentLocale) }}
                        <span class="usage-token-sep">/</span>
                        {{ (log.cacheCreationTokens ?? 0).toLocaleString(currentLocale) }}
                      </td>
                      <td :title="usageLogCostBreakdownTitle(log)">${{ log.costUsd.toFixed(4) }}</td>
                      <td>{{ formatFirstTokenMs(log.firstTokenMs) }}</td>
                      <td>{{ formatDurationMs(log.durationMs) }}</td>
                      <td>{{ log.statusCode || '—' }}</td>
                      <td>{{ log.dataSource }}</td>
                    </tr>
                  </tbody>
                  <tbody v-else-if="usageView === 'providerStats' && providerUsageRows.length">
                    <tr v-for="row in providerUsageRows" :key="row.providerProfileId">
                      <td>{{ row.providerName }}</td>
                      <td>{{ row.totalRequests }}</td>
                      <td>{{ row.totalInputTokens.toLocaleString(currentLocale) }}</td>
                      <td>{{ row.totalOutputTokens.toLocaleString(currentLocale) }}</td>
                      <td>${{ row.totalCostUsd.toFixed(4) }}</td>
                      <td>{{ Math.round(row.cacheHitRate * 1000) / 10 }}%</td>
                      <td>{{ row.models.size }}</td>
                      <td>{{ Array.from(row.appTypes).join(' · ') }}</td>
                      <td>{{ formatUsageDate(row.lastActivityAt) }}</td>
                    </tr>
                  </tbody>
                  <tbody v-else-if="usageView === 'modelStats' && modelUsageRows.length">
                    <tr v-for="row in modelUsageRows" :key="row.model">
                      <td>{{ row.model }}</td>
                      <td>{{ row.totalRequests }}</td>
                      <td>{{ Array.from(row.providerNames).join(' · ') }}</td>
                      <td>{{ row.totalInputTokens.toLocaleString(currentLocale) }}</td>
                      <td>{{ row.totalOutputTokens.toLocaleString(currentLocale) }}</td>
                      <td>${{ row.totalCostUsd.toFixed(4) }}</td>
                      <td>{{ Array.from(row.appTypes).join(' · ') }}</td>
                      <td>{{ formatUsageDate(row.lastActivityAt) }}</td>
                    </tr>
                  </tbody>
                  <tbody v-else>
                    <tr>
                      <td :colspan="usageView === 'requestLogs' ? 11 : usageView === 'modelStats' ? 8 : 9">
                        <div class="usage-empty-state usage-empty-state--table">
                          <AppIcon name="terminal" :size="18" />
                          <strong>{{ usageView === 'requestLogs' ? t('usage.noLogsTitle') : tr('暂无聚合数据', 'No aggregate data') }}</strong>
                          <span>{{ usageView === 'requestLogs' ? t('usage.noLogsDesc') : tr('当前筛选条件下没有可聚合的 Provider / 模型使用数据。', 'No Provider or model usage data matched the current filters.') }}</span>
                        </div>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-if="usageView === 'requestLogs' && activeUsageLogs.length" class="usage-log-footer">
                <span class="usage-log-footer__meta">
                  {{ tr(
                    `当前展示 ${activeUsageLogs.length} 条明细${managedUsageTotal ? ` / 共 ${managedUsageTotal}` : ''}`,
                    `Showing ${activeUsageLogs.length} detail rows${managedUsageTotal ? ` / total ${managedUsageTotal}` : ''}`,
                  ) }}
                  · {{ tr(
                    '「加载更多」只追加请求明细，不会重算 KPI / 趋势 / 排行',
                    '“Load more” only appends detail logs; KPI / trends / ranking stay from full aggregates',
                  ) }}
                </span>
                <button
                  v-if="usageHasMoreRequestLogs"
                  type="button"
                  class="ghost-btn ghost-btn--small"
                  :disabled="managedUsageInFlight || providerUsageRefreshRunning"
                  @click="loadMoreUsageRequestLogs()"
                >
                  <AppIcon name="refresh" :size="13" :class="{ 'is-spinning': managedUsageInFlight }" />
                  <span>{{ tr(`加载更多明细 +${usageRequestLogPageSize}`, `Load more details +${usageRequestLogPageSize}`) }}</span>
                </button>
              </div>
            </article>
          </section>
        </section>

        <section class="p45-page p45-page--search" v-else-if="appSection === 'search'">
          <section class="app-search-panel panel">
            <label class="app-search-input">
              <AppIcon name="search" :size="16" />
              <input data-app-search-input v-model.trim="appSearchQuery" type="search" :placeholder="t('search.placeholder')" @keydown="handleSearchInputKeydown" />
            </label>
            <div v-if="pageSearchResults.length" class="app-search-meta">
              <span>{{ t('search.selectedResult', { current: Math.max(activeSearchResultIndex() + 1, 1), total: pageSearchResults.length }) }}</span>
              <span :class="{ 'app-search-meta__hint--active': searchLoopHint }">{{ searchLoopHint || t('search.keyboardHint') }}</span>
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
                      <span>{{ t('search.copyCommand') }}</span>
                    </button>
                    <button v-if="result.command" type="button" class="ghost-btn ghost-btn--small" @click.stop="insertCommandToActivePane(result.command || '')">
                      <AppIcon name="terminal" :size="13" />
                      <span>{{ t('search.insertCurrentInput') }}</span>
                    </button>
                  </span>
                </button>
              </section>
            </div>

            <div v-else class="empty-state">
              <div class="empty-state__icon"><AppIcon name="search" :size="18" /></div>
              <div class="empty-state__body">
                <strong>{{ t('search.noResultsTitle') }}</strong>
                <p>{{ t('search.noResultsDesc') }}</p>
              </div>
            </div>
          </section>
        </section>

        <section class="p45-page p45-page--settings" v-else-if="appSection === 'settings'">
          <header class="p45-hero panel">
            <div class="p45-hero__copy">
              <h2>{{ t('settings.title') }}</h2>
              <p>{{ t('settings.intro') }}</p>
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
                    <span>{{ t('settings.actions.openThemePanel') }}</span>
                  </button>
                  <button type="button" class="ghost-btn" @click="toggleRailCollapsed()">
                    <AppIcon name="workspace" :size="14" />
                    <span>{{ railCollapsed ? t('settings.actions.railExpand') : t('settings.actions.railCollapse') }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-preview-card">
                  <div class="settings-preview-card__head">
                    <strong>{{ themeDisplayName(selectedThemePreset) }}</strong>
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
                    <span>{{ themeDisplayName(theme) }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-chip-group">
                  <button
                    v-for="option in localeOptions"
                    :key="`locale-${option.value}`"
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': currentLocale === option.value }"
                    @click="applyLocaleFromSettings(option.value)"
                  >
                    <span>{{ option.label }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance'" class="settings-note">
                  <strong>{{ t('common.language') }}</strong>
                  <p>{{ t('common.languageHint') }}</p>
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
                    <span>{{ t('settings.groups.appearance.fontSize', { size }) }}</span>
                  </button>
                </div>
                <div v-if="group.id === 'appearance' && activeThemeId === 'default'" class="settings-inline-field">
                  <label class="settings-inline-field__label" for="settings-accent-color">{{ t('settings.actions.themeAccent') }}</label>
                  <input id="settings-accent-color" type="color" :value="customAccentHex" @input="onAccentColorInput(($event.target as HTMLInputElement).value); showToast(t('toast.accentUpdated'), ($event.target as HTMLInputElement).value)" />
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
                    <span>{{ t('settings.actions.currentRestoreStrategy', { label: restoreCommandStrategyLabel(restoreCommandStrategy) }) }}</span>
                  </div>
                  <code class="settings-preview-card__terminal" :style="terminalPreviewStyle">PS&gt; echo "Hello Chuchen-Terminal"</code>
                  <div class="settings-control-row">
                    <label class="settings-control-row__label" for="settings-font-size">{{ t('settings.actions.terminalFontSize') }}</label>
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
                    <span>{{ t('settings.actions.openSystemPanel') }}</span>
                  </button>
                  <button type="button" class="ghost-btn" :disabled="environmentChecksRefreshing" @click="refreshEnvironmentChecks(true)">
                    <AppIcon name="refresh" :size="14" :class="{ 'is-spinning': environmentChecksRefreshing }" />
                    <span>{{ environmentChecksRefreshing ? t('settings.actions.checking') : t('settings.actions.recheckEnvironment') }}</span>
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
                    <span>{{ option === 'manual' ? t('settings.actions.refreshManual') : t('settings.actions.refreshQuick', { seconds: option.replace('s', '') }) }}</span>
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
                    <span>{{ systemNotificationsEnabled ? t('themePanel.notificationsOn') : t('themePanel.notificationsOff') }}</span>
                  </button>
                  <button
                    type="button"
                    class="settings-chip"
                    :class="{ 'settings-chip--active': windowAttentionEnabled }"
                    @click="windowAttentionEnabled = !windowAttentionEnabled"
                  >
                    <span>{{ windowAttentionEnabled ? t('themePanel.taskbarOn') : t('themePanel.taskbarOff') }}</span>
                  </button>
                </div>

                <div v-if="group.id === 'supervisor' && isDevBuild" class="settings-chip-group settings-chip-group--wrap">
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('completed')"><span>{{ t('themePanel.simulateCompleted') }}</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('needs-input')"><span>{{ t('themePanel.simulateNeedsInput') }}</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('error')"><span>{{ t('themePanel.simulateError') }}</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('stalled')"><span>{{ t('themePanel.simulateStalled') }}</span></button>
                  <button type="button" class="settings-chip" @click="simulateSessionAttention('clear')"><span>{{ t('themePanel.clearReminder') }}</span></button>
                </div>

                <div v-if="group.id === 'data'" class="settings-note">
                  <strong>{{ t('themePanel.providerImportTitle') }}</strong>
                  <p>{{ t('themePanel.providerImportDesc') }}</p>
                </div>
                <div v-if="group.id === 'data'" class="settings-actions">
                  <button type="button" class="ghost-btn" @click="clearDiagnosticsCaches()">
                    <AppIcon name="trash" :size="14" />
                    <span>{{ t('themePanel.reloadSystemData') }}</span>
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
              <span>{{ tr('该模块暂未纳入当前阶段的主开发闭环，先保留结构入口。', 'This module is not yet part of the current primary delivery loop, so the entry remains in place for now.') }}</span>
            </div>
          </div>
          <div class="placeholder-body">{{ placeholderDescription }}</div>
        </section>
      </main>
    </div>

    <ModelPricingModal
      :open="openModelPricingModalState"
      :locale="currentLocale"
      @close="closeModelPricingModal()"
      @saved="onModelPricingSaved()"
    />

    <ModalShell
      :open="openWorkspaceEditorModal"
      :title="workspaceEditorMode === 'create' ? t('forms.workspaceCreateTitle') : t('forms.workspaceEditTitle')"
      :description="t('forms.workspaceDesc')"
      icon="workspace"
      size="md"
      @close="closeWorkspaceEditorModal()"
    >
      <form class="editor-form editor-form--refined" @submit.prevent="submitWorkspaceForm()">
        <label class="form-field">
          <span>{{ t('forms.workspaceName') }}</span>
          <input v-model.trim="workspaceForm.name" type="text" :placeholder="t('forms.workspaceNamePlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.workspaceRoot') }}</span>
          <div class="path-field">
            <input v-model.trim="workspaceForm.rootPath" type="text" :placeholder="t('forms.workspaceRootPlaceholder')" />
            <button type="button" class="ghost-btn ghost-btn--small path-field__action" @click.stop="pickWorkspaceRootPath()">{{ t('common.actions.chooseDirectory') }}</button>
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('forms.workspaceDescription') }}</span>
          <textarea v-model.trim="workspaceForm.description" rows="3" :placeholder="t('forms.workspaceDescriptionPlaceholder')"></textarea>
        </label>
        <label class="form-field">
          <span>{{ t('forms.tags') }}</span>
          <input v-model.trim="workspaceForm.tagsText" type="text" :placeholder="t('forms.workspaceTagsPlaceholder')" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeWorkspaceEditorModal()">{{ t('common.actions.cancel') }}</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">{{ t('forms.saveWorkspace') }}</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTerminalEntryEditorModal"
      :title="terminalEntryEditorMode === 'create' ? t('forms.entryCreateTitle') : t('forms.entryEditTitle')"
      :description="t('forms.entryDesc')"
      icon="terminal"
      size="md"
      @close="closeTerminalEntryEditorModal()"
    >
      <form class="editor-form editor-form--refined" @submit.prevent="submitTerminalEntryForm()">
        <label class="form-field">
          <span>{{ t('forms.entryName') }}</span>
          <input v-model.trim="terminalEntryForm.name" type="text" :placeholder="t('forms.entryNamePlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.workingDirectory') }}</span>
          <div class="path-field">
            <input v-model.trim="terminalEntryForm.workingDirectory" type="text" :placeholder="t('forms.workingDirectoryPlaceholder')" />
            <button type="button" class="ghost-btn ghost-btn--small path-field__action" @click.stop="pickTerminalEntryWorkingDirectory()">{{ t('common.actions.chooseDirectory') }}</button>
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('forms.defaultCommand') }}</span>
          <input v-model.trim="terminalEntryForm.defaultCommand" type="text" :placeholder="t('forms.defaultCommandPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.launchMode') }}</span>
          <div class="pane__binding-wrap form-select-wrap">
            <button type="button" class="binding-trigger form-select-trigger" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleLaunchModeMenu()">
              <span>{{ launchModeLabel(terminalEntryForm.launchMode) }}</span>
              <AppIcon name="chevron-right" :size="14" />
            </button>
            <PopoverMenu :open="openLaunchModeMenu" :items="launchModeItems" />
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('forms.environmentVariables') }}</span>
          <textarea v-model.trim="terminalEntryForm.environmentVariablesText" rows="4" :placeholder="t('forms.envPlaceholder')"></textarea>
        </label>
        <label class="form-field">
          <span>{{ t('forms.tags') }}</span>
          <input v-model.trim="terminalEntryForm.tagsText" type="text" :placeholder="t('forms.entryTagsPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.note') }}</span>
          <textarea v-model.trim="terminalEntryForm.note" rows="3" :placeholder="t('forms.notePlaceholder')"></textarea>
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeTerminalEntryEditorModal()">{{ t('common.actions.cancel') }}</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">{{ t('forms.saveEntry') }}</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTemplateEditorModal"
      :title="templateEditorMode === 'create' ? t('forms.templateCreateTitle') : t('forms.templateEditTitle')"
      :description="t('forms.templateDesc')"
      icon="template"
      size="md"
      @close="closeWorkflowTemplateEditorModal()"
    >
      <form class="editor-form editor-form--refined" @submit.prevent="submitWorkflowTemplateForm()">
        <label class="form-field">
          <span>{{ t('forms.templateName') }}</span>
          <input v-model.trim="workflowTemplateForm.name" type="text" :placeholder="t('forms.templateNamePlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.templateDescription') }}</span>
          <textarea v-model.trim="workflowTemplateForm.description" rows="3" :placeholder="t('forms.templateDescriptionPlaceholder')"></textarea>
        </label>
        <label class="form-field">
          <span>{{ t('forms.tags') }}</span>
          <input v-model.trim="workflowTemplateForm.tagsText" type="text" :placeholder="t('forms.workspaceTagsPlaceholder')" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeWorkflowTemplateEditorModal()">{{ t('common.actions.cancel') }}</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">{{ t('forms.saveTemplate') }}</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openProviderEditorModal"
      :title="providerEditorMode === 'create' ? t('forms.profileCreateTitle') : t('forms.profileEditTitle')"
      :description="t('forms.profileDesc')"
      icon="settings"
      size="lg"
      @close="closeProviderEditorModal()"
    >
      <form class="editor-form editor-form--refined" @submit.prevent="submitProviderForm()">
        <label class="form-field">
          <span>{{ t('forms.profileName') }}</span>
          <input v-model.trim="providerForm.name" type="text" :placeholder="t('forms.profileNamePlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.cliApp') }}</span>
          <div class="pane__binding-wrap form-select-wrap">
            <button type="button" class="binding-trigger form-select-trigger" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleProviderKindMenu()">
              <span>{{ providerKindLabel(providerForm.providerKind) }}</span>
              <AppIcon name="chevron-right" :size="14" />
            </button>
            <PopoverMenu :open="openProviderKindMenu" :items="providerKindItems" />
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('forms.profileLabel') }}</span>
          <input v-model.trim="providerForm.profileName" type="text" :placeholder="t('forms.profileIdPlaceholder')" />
        </label>
        <label class="form-field form-field--readonly">
          <span>{{ t('forms.configFile') }}</span>
          <input v-model.trim="providerForm.configPath" type="text" :placeholder="t('forms.configFilePlaceholder')" readonly tabindex="-1" />
        </label>
        <label class="form-field">
          <span>{{ t('provider.configSource') }}</span>
          <div class="pane__binding-wrap form-select-wrap">
            <button type="button" class="binding-trigger form-select-trigger" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleProviderSourceMenu()">
              <span>{{ providerSourceLabel(providerForm.managedBy) }}</span>
              <AppIcon name="chevron-right" :size="14" />
            </button>
            <PopoverMenu :open="openProviderSourceMenu" :items="providerSourceItems" />
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('provider.scopeLabel') }}</span>
          <div class="pane__binding-wrap form-select-wrap">
            <button type="button" class="binding-trigger form-select-trigger" @pointerdown="handleMenuTriggerPointerDown" @click.stop="toggleProviderScopeMenu()">
              <span>{{ providerScopeLabel(providerForm.configScope) }}</span>
              <AppIcon name="chevron-right" :size="14" />
            </button>
            <PopoverMenu :open="openProviderScopeMenu" :items="providerScopeItems" />
          </div>
        </label>
        <label class="form-field">
          <span>{{ t('provider.targetCli') }}</span>
          <input v-model.trim="providerForm.toolTargetsText" type="text" :placeholder="t('forms.targetCliPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ tr('默认模型', 'Default model') }}</span>
          <input v-model.trim="providerForm.defaultModel" type="text" :placeholder="tr('留空则使用 CLI 内置默认', 'Leave empty to use CLI default')" />
        </label>
        <label class="form-field">
          <span>{{ t('provider.authSource') }}</span>
          <input v-model.trim="providerForm.authSource" type="text" :placeholder="t('forms.authSourcePlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.homepageUrl') }}</span>
          <input v-model.trim="providerForm.homepageUrl" type="text" :placeholder="t('forms.homepageUrlPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.requestBaseUrl') }}</span>
          <input v-model.trim="providerForm.requestBaseUrl" type="text" :placeholder="t('forms.requestBaseUrlPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('provider.switchCommand') }}</span>
          <input
            v-model.trim="providerForm.switchCommand"
            type="text"
            :placeholder="switchCommandPlaceholderForKind(providerForm.providerKind, providerForm.profileName)"
          />
        </label>
        <label class="form-field">
          <span>{{ t('provider.colorTag') }}</span>
          <input v-model.trim="providerForm.color" type="text" :placeholder="t('forms.colorPlaceholder')" />
        </label>
        <label class="form-field">
          <span>{{ t('forms.note') }}</span>
          <textarea v-model.trim="providerForm.note" rows="3" :placeholder="t('forms.profileNotePlaceholder')"></textarea>
        </label>
        <label class="form-field">
          <span>{{ t('forms.authPayload') }}</span>
          <textarea v-model.trim="providerForm.authPayload" rows="5" :placeholder="t('forms.authPayloadPlaceholder')"></textarea>
        </label>
        <label class="form-field">
          <span>{{ t('forms.configPayload') }}</span>
          <textarea v-model.trim="providerForm.configPayload" rows="10" :placeholder="t('forms.configPayloadPlaceholder')"></textarea>
        </label>
        <label class="form-field form-field--inline">
          <input v-model="providerForm.isDefault" type="checkbox" />
          <span>{{ t('provider.setDefaultProfile') }}</span>
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeProviderEditorModal()">{{ t('common.actions.cancel') }}</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">{{ t('forms.saveProfile') }}</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openTerminalEntriesModal"
      :title="t('provider.runConfigsTitle')"
      :description="t('provider.runConfigsDesc')"
      icon="terminal"
      size="lg"
      @close="openTerminalEntriesModal = false"
    >
      <div class="entry-modal">
        <div class="entry-modal__toolbar">
          <div class="entry-modal__intro">
            <strong>{{ selectedWorkspace?.name }}</strong>
            <span>{{ t('provider.currentWorkspaceEntries') }}</span>
          </div>
          <div class="entry-modal__toolbar-actions">
            <button class="ghost-btn ghost-btn--primary" @click="openTerminalEntryCreateModal()">
              <AppIcon name="terminal" :size="15" />
              <span>{{ t('provider.newRunConfig') }}</span>
            </button>
          </div>
        </div>

        <div v-if="selectedWorkspaceEntries.length" class="entry-list">
          <article v-for="entry in selectedWorkspaceEntries" :key="entry.id" class="entry-card">
            <div class="entry-card__head">
              <div>
                <strong>{{ entry.name }}</strong>
                <span>{{ entry.workingDirectory }}</span>
              </div>
            <div class="entry-card__head-meta">
                <span class="meta-badge">{{ launchModeLabel(entry.launchMode) }}</span>
                <span class="meta-badge meta-badge--soft">{{ t('provider.boundPanes', { count: entryUsageCount(entry.id) }) }}</span>
              </div>
            </div>
            <div class="entry-card__body">
              <span>{{ t('workspace.defaultCommand') }}：{{ entry.defaultCommand || t('workspace.notSet') }}</span>
              <span>{{ t('forms.environmentVariables') }}：{{ entry.environmentVariablesText ? t('provider.envConfigured') : t('provider.envNotConfigured') }}</span>
              <span>{{ t('workspace.lastCommand') }}：{{ entry.lastCommand || t('workspace.notRecorded') }}</span>
            </div>
            <div class="entry-card__actions">
              <button class="ghost-btn ghost-btn--small" @click="openTerminalEntryEditModal(entry.id)">{{ t('common.actions.edit') }}</button>
              <button class="ghost-btn ghost-btn--danger ghost-btn--small" @click="removeTerminalEntry(entry.id)">{{ t('common.actions.delete') }}</button>
            </div>
          </article>
        </div>

        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="terminal" :size="15" />
          </div>
          <div class="empty-state__body">
            <strong>{{ t('provider.noRunConfigsTitle') }}</strong>
            <p>{{ t('provider.noRunConfigsDesc') }}</p>
          </div>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openSplitActionModal"
      :title="splitActionState.mode === 'create' ? t('modal.splitPaneTitleCreate') : t('modal.splitPaneTitleDuplicate')"
      :description="t('modal.splitPaneDesc')"
      icon="pane"
      size="md"
      @close="openSplitActionModal = false"
    >
      <div class="split-action-dialog">
        <div class="split-action-grid">
          <button type="button" class="split-choice-card" @click="submitSplitAction('grid')">
            <AppIcon name="workspace" :size="18" />
            <strong>{{ t('workspace.layoutGrid') }}</strong>
            <span>{{ tr('更适合多个 Pane 同时总览。', 'Best when several panes must stay visible together.') }}</span>
          </button>
          <button type="button" class="split-choice-card" @click="submitSplitAction('horizontal')">
            <AppIcon name="copy" :size="18" />
            <strong>{{ t('workspace.layoutHorizontal') }}</strong>
            <span>{{ tr('让 Pane 以左右并排的方式展示。', 'Arrange panes side by side.') }}</span>
          </button>
          <button type="button" class="split-choice-card" @click="submitSplitAction('vertical')">
            <AppIcon name="pane" :size="18" />
            <strong>{{ t('workspace.layoutVertical') }}</strong>
            <span>{{ tr('让 Pane 以上下堆叠的方式展示。', 'Stack panes vertically.') }}</span>
          </button>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openThemeModal"
      :title="t('modal.themeTitle')"
      :description="t('modal.themeDesc')"
      size="lg"
      @close="openThemeModal = false"
    >
      <div class="modal-tabs">
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'theme' }" @click.stop="themePanelTab = 'theme'">
          <AppIcon name="theme" :size="15" />
          <span>{{ t('themePanel.themeTab') }}</span>
        </button>
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'font' }" @click.stop="themePanelTab = 'font'">
          <AppIcon name="terminal" :size="15" />
          <span>{{ t('themePanel.fontTab') }}</span>
        </button>
        <button type="button" class="modal-tab" :class="{ 'modal-tab--active': themePanelTab === 'system' }" @click.stop="themePanelTab = 'system'">
          <AppIcon name="runtime" :size="15" />
          <span>{{ t('themePanel.systemTab') }}</span>
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
            <h4>{{ t('themePanel.systemThemes') }}</h4>
            <span>{{ t('themePanel.systemThemesDesc') }}</span>
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
                <strong>{{ themeDisplayName(theme) }}</strong>
                <span>{{ themeDisplayDescription(theme) }}</span>
              </span>
              <small>{{ themeDisplayKind(theme) }}</small>
            </button>
          </div>
        </section>

        <section class="theme-section">
          <div class="theme-section__head">
            <h4>{{ t('themePanel.importedThemes') }}</h4>
            <span>{{ t('themePanel.importedThemesDesc') }}</span>
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
                <strong>{{ themeDisplayName(theme) }}</strong>
                <span>{{ themeDisplayDescription(theme) }}</span>
              </span>
              <small>{{ themeDisplayKind(theme) }}</small>
            </button>
          </div>
        </section>

        <section class="theme-customizer" v-if="activeThemeId === 'default'">
          <div class="theme-section__head">
            <h4>{{ t('themePanel.customizeTheme') }}</h4>
            <span>{{ t('themePanel.customizeThemeDesc') }}</span>
          </div>
          <div class="color-editor">
            <label class="color-input-card">
              <span>{{ t('themePanel.accent') }}</span>
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
          <div class="font-preview__chars">{{ t('themePanel.fontSpecialChars') }}</div>
        </div>

        <div class="font-presets">
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 11 }" @click.stop="setTerminalFontSize(11)">
            <strong>{{ t('themePanel.small') }}</strong>
            <span>11px</span>
          </button>
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 13 }" @click.stop="setTerminalFontSize(13)">
            <strong>{{ t('themePanel.medium') }}</strong>
            <span>13px</span>
          </button>
          <button type="button" class="font-preset" :class="{ 'font-preset--active': terminalFontSize === 16 }" @click.stop="setTerminalFontSize(16)">
            <strong>{{ t('themePanel.large') }}</strong>
            <span>16px</span>
          </button>
        </div>

        <div class="font-controls">
          <div class="font-control">
            <div class="font-control__head">
              <strong>{{ t('themePanel.terminalFontSize') }}</strong>
              <span>{{ terminalFontSize }}px</span>
            </div>
            <div class="font-control__body">
              <input type="range" min="8" max="24" :value="terminalFontSize" @input="setTerminalFontSize(Number(($event.target as HTMLInputElement).value))" />
              <input class="font-number" type="number" min="8" max="24" :value="terminalFontSize" @change="setTerminalFontSize(Number(($event.target as HTMLInputElement).value))" />
            </div>
          </div>

          <div class="font-control">
            <div class="font-control__head">
              <strong>{{ t('themePanel.terminalFont') }}</strong>
              <span>{{ t('themePanel.candidateCount', { count: fontOptions.length }) }}</span>
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
                <small>{{ t('themePanel.terminalSample') }}</small>
              </button>
            </div>
          </div>
        </div>

        <div class="font-note">{{ t('themePanel.fontNote') }}</div>
      </div>

      <div v-else class="font-panel">
        <section class="theme-section">
          <div class="theme-section__head">
            <h4>{{ tr('系统资源刷新', 'System resource refresh') }}</h4>
            <span>{{ tr('自动刷新默认开启，也可以切为手动刷新，降低系统负担。', 'Auto refresh is enabled by default, but you can switch to manual refresh to reduce system load.') }}</span>
          </div>
          <div class="theme-section__actions">
            <button type="button" class="ghost-btn ghost-btn--small" @click.stop="refreshSystemStatus()">
              <AppIcon name="runtime" :size="14" />
              <span>{{ t('common.actions.refreshNow') }}</span>
            </button>
          </div>
          <div class="theme-grid theme-grid--compact">
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === 'manual' }" @click.stop="systemRefreshInterval = 'manual'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>{{ t('settings.actions.refreshManual') }}</strong><span>{{ tr('仅在打开应用时读取一次', 'Only read once when the app opens.') }}</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '5s' }" @click.stop="systemRefreshInterval = '5s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>{{ t('settings.actions.refreshQuick', { seconds: 5 }) }}</strong><span>{{ tr('更实时，但开销更大', 'More real-time, but heavier.') }}</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '10s' }" @click.stop="systemRefreshInterval = '10s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>{{ t('settings.actions.refreshQuick', { seconds: 10 }) }}</strong><span>{{ tr('默认建议值', 'Recommended default.') }}</span></span></button>
            <button type="button" class="theme-card" :class="{ 'theme-card--active': systemRefreshInterval === '30s' }" @click.stop="systemRefreshInterval = '30s'; scheduleSystemRefresh()"><span class="theme-card__body"><strong>{{ t('settings.actions.refreshQuick', { seconds: 30 }) }}</strong><span>{{ tr('更省资源', 'Lighter on resources.') }}</span></span></button>
          </div>
        </section>

        <section class="theme-section">
          <div class="theme-section__head">
            <h4>{{ tr('环境项显示', 'Environment visibility') }}</h4>
            <span>{{ tr('常见技术栈默认展示，可以按你的习惯隐藏不关心的项。', 'Common stacks are shown by default; hide the ones you do not care about.') }}</span>
          </div>
          <div class="theme-section__actions">
            <button type="button" class="ghost-btn ghost-btn--small" :disabled="environmentChecksRefreshing" @click.stop="refreshEnvironmentChecks(true)">
              <AppIcon name="refresh" :size="14" :class="{ 'is-spinning': environmentChecksRefreshing }" />
              <span>{{ environmentChecksRefreshing ? tr('检测中', 'Checking') : tr('重新检测环境', 'Recheck environment') }}</span>
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
                <img v-if="item.iconSrc" :src="item.iconSrc" :alt="`${item.name} icon`" />
                <span v-else>{{ item.icon }}</span>
              </span>
              <span class="theme-card__body">
                <strong>{{ item.name }}</strong>
                <span>{{ hiddenEnvironmentItems.includes(item.name) ? tr('已隐藏', 'Hidden') : tr('已显示', 'Visible') }}</span>
              </span>
            </button>
          </div>
        </section>

        <section v-if="isDevBuild" class="theme-section">
          <div class="theme-section__head">
            <h4>{{ tr('提醒测试（开发环境）', 'Alert testing (dev only)') }}</h4>
            <span>{{ tr('只在开发环境显示，用于直接模拟终端提醒状态，验证高亮、通知、待处理数与任务栏提醒。', 'Shown only in development mode. Simulate terminal alert states to verify highlights, notifications, pending counters, and taskbar attention.') }}</span>
          </div>
          <div class="theme-panel__summary theme-panel__summary--devtest">
            <div>
              <strong>{{ currentActiveRuntimeSessionMeta?.sessionName || tr('未选中终端', 'No terminal selected') }}</strong>
              <span>{{ currentActiveRuntimeSessionMeta ? `${currentActiveRuntimeSessionMeta.workspaceName} / ${currentActiveRuntimeSessionMeta.tabName}` : tr('请先在右侧选中一个终端标签', 'Select a terminal tab on the right first.') }}</span>
            </div>
            <span class="meta-badge meta-badge--soft">DEV ONLY</span>
          </div>
          <div class="theme-grid theme-grid--compact">
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('completed')">
              <span class="theme-card__body">
                <strong>{{ tr('模拟完成', 'Simulate completed') }}</strong>
                <span>{{ tr('触发已完成高亮、通知与待处理数。', 'Trigger completed highlights, notifications, and pending counters.') }}</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('needs-input')">
              <span class="theme-card__body">
                <strong>{{ tr('模拟等待输入', 'Simulate needs input') }}</strong>
                <span>{{ tr('触发待处理高亮与系统提醒。', 'Trigger pending highlights and system reminders.') }}</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('error')">
              <span class="theme-card__body">
                <strong>{{ tr('模拟异常退出', 'Simulate error exit') }}</strong>
                <span>{{ tr('触发异常高亮、通知与待处理数。', 'Trigger error highlights, notifications, and pending counters.') }}</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('stalled')">
              <span class="theme-card__body">
                <strong>{{ tr('模拟疑似停滞', 'Simulate stalled') }}</strong>
                <span>{{ tr('触发黄色提醒，用于测试人工介入场景。', 'Trigger an amber alert state for manual intervention testing.') }}</span>
              </span>
            </button>
            <button type="button" class="theme-card" @click.stop="simulateSessionAttention('clear')">
              <span class="theme-card__body">
                <strong>{{ tr('清除提醒', 'Clear reminder') }}</strong>
                <span>{{ tr('把当前终端恢复为无提醒状态。', 'Return the current terminal to a neutral state.') }}</span>
              </span>
            </button>
          </div>
        </section>
      </div>
    </ModalShell>

    <ModalShell
      :open="openRenameModal"
      :title="renameTarget.title"
      :description="tr('支持手动调整名称，不影响内部编号与布局状态。', 'Manual renaming only changes the display name and does not affect internal IDs or layout state.')"
      icon="edit"
      size="sm"
      @close="closeRenameModal()"
    >
      <form class="editor-form editor-form--refined" @submit.prevent="submitRenameModal()">
        <label class="form-field">
          <span>{{ renameTarget.kind === 'tab' ? tr('项目名称', 'Project name') : renameTarget.kind === 'session' ? tr('终端名称', 'Terminal name') : tr('Pane 名称', 'Pane name') }}</span>
          <input ref="renameInputRef" v-model.trim="renameTarget.value" type="text" :placeholder="renameTarget.placeholder" />
        </label>
        <div class="form-actions">
          <button type="button" class="ghost-btn" @click="closeRenameModal()">{{ t('common.actions.cancel') }}</button>
          <button type="submit" class="ghost-btn ghost-btn--primary">{{ tr('保存名称', 'Save name') }}</button>
        </div>
      </form>
    </ModalShell>

    <ModalShell
      :open="openSearchModal"
      :title="tr('快速搜索', 'Quick search')"
      :description="tr('双击 Shift 或 Ctrl+K 随时唤起，快速打开工作区、项目、终端、配置和命令。', 'Open it anytime with double Shift or Ctrl+K to jump to workspaces, projects, terminals, configs, and commands.')"
      size="lg"
      @close="openSearchModal = false"
    >
      <div class="quick-search-panel">
        <label class="app-search-input app-search-input--modal">
          <AppIcon name="search" :size="16" />
          <input data-quick-search-input v-model.trim="appSearchQuery" type="search" :placeholder="t('search.quickPlaceholder')" @keydown="handleSearchInputKeydown" />
        </label>
        <div class="quick-search-panel__meta">
          <span class="meta-badge meta-badge--soft">Shift Shift</span>
          <span class="meta-badge meta-badge--soft">Ctrl+K</span>
          <span v-if="searchLoopHint" class="meta-badge meta-badge--notice">{{ searchLoopHint }}</span>
          <button type="button" class="ghost-btn ghost-btn--small" @click="openSearchModal = false; openSearchPage()">
            <AppIcon name="search" :size="13" />
            <span>{{ t('search.openSearchPage') }}</span>
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
            <strong>{{ t('search.noResultsTitle') }}</strong>
            <p>{{ tr('这里只检索 Chuchen-Terminal 内部数据，不会扫描整个电脑磁盘。', 'Only data inside Chuchen-Terminal is searched here; your whole disk is not scanned.') }}</p>
          </div>
        </div>
      </div>
    </ModalShell>

    <ModalShell
      :open="openRecentRecycleBinModal"
      :title="tr('最近记录回收站', 'Recent history recycle bin')"
      :description="tr('这里保存你从最近页移除的记录。可以单条恢复，也可以全部恢复。', 'Records removed from the Recent page are stored here. You can restore one or restore all of them.')"
      icon="trash"
      size="md"
      @close="openRecentRecycleBinModal = false"
    >
      <div class="entry-modal">
        <div class="entry-modal__toolbar">
          <div class="entry-modal__intro">
            <strong>{{ tr('{count} 条记录', '{count} records').replace('{count}', String(removedRecentItems.length)) }}</strong>
            <span>{{ tr('恢复后会重新出现在最近页中。', 'Restored records will appear on the Recent page again.') }}</span>
          </div>
          <div class="entry-modal__toolbar-actions">
            <button class="ghost-btn" @click="clearHiddenRecentItems()">
              <AppIcon name="trash" :size="14" />
              <span>{{ tr('清空回收站', 'Clear recycle bin') }}</span>
            </button>
            <button class="ghost-btn ghost-btn--primary" @click="restoreHiddenRecentItems()">
              <AppIcon name="refresh" :size="14" />
              <span>{{ tr('全部恢复', 'Restore all') }}</span>
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
              <button class="ghost-btn ghost-btn--small" @click="restoreHiddenRecentItem(item.id)">{{ t('common.actions.restore') }}</button>
            </div>
          </article>
        </div>

        <div v-else class="empty-state empty-state--modal">
          <div class="empty-state__icon">
            <AppIcon name="trash" :size="18" />
          </div>
          <div class="empty-state__body">
            <strong>{{ tr('回收站为空', 'Recycle bin is empty') }}</strong>
            <p>{{ tr('当前没有被移除的最近记录。', 'There are no removed recent records right now.') }}</p>
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
          <button type="button" class="ghost-btn" @click="closeConfirmModal()">{{ t('common.actions.cancel') }}</button>
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

    <DrawerPanel
      :open="openAiHistoryDrawer"
      :title="tr('AI CLI 历史 / Resume', 'AI CLI History / Resume')"
      :description="tr('聚合当前工作区内 Codex、Claude Code、Gemini CLI 等 AI 终端上下文，用于快速回到最近会话或复用命令。', 'Aggregate Codex, Claude Code, Gemini CLI, and similar AI terminal context in the current workspace so you can jump back to sessions or reuse commands quickly.')"
      side="right"
      @close="openAiHistoryDrawer = false"
    >
      <div class="ai-history-drawer">
        <section class="ai-history-section">
          <div class="ai-history-section__head">
            <h4>{{ tr('AI CLI 会话', 'AI CLI sessions') }}</h4>
            <span>{{ tr('{count} 个', '{count} items').replace('{count}', String(selectedWorkspaceAiCliSessions.length)) }}</span>
          </div>
          <div v-if="selectedWorkspaceAiCliSessions.length" class="ai-history-list">
            <button
              v-for="item in selectedWorkspaceAiCliSessions.slice(0, 12)"
              :key="item.id"
              type="button"
              class="ai-history-card"
              :class="`ai-history-card--${item.statusState}`"
              @click="openAiCliSessionItem(item)"
            >
              <span class="ai-history-card__rail">{{ item.lastInfo.shortLabel }}</span>
              <span class="ai-history-card__body">
                <strong>{{ item.sessionName }}</strong>
                <small>
                  {{ item.tabName }} / {{ item.path }}
                  <template v-if="item.info.kind !== item.lastInfo.kind">
                    · {{ tr('当前：普通终端', 'Current: normal terminal') }}
                  </template>
                </small>
                <code v-if="item.command">{{ item.command }}</code>
              </span>
              <span class="ai-history-card__state">
                <span class="session-status-badge" :class="`session-status-badge--${aiSessionStateBadgeClass(item.statusState)}`">
                  {{ item.statusLabel }}
                </span>
              </span>
            </button>
          </div>
          <div v-else class="ai-history-empty">{{ tr('当前工作区还没有识别到 AI CLI 终端。', 'No AI CLI terminal has been recognized in the current workspace yet.') }}</div>
        </section>

        <section class="ai-history-section">
          <div class="ai-history-section__head">
            <h4>{{ tr('Resume 命令', 'Resume commands') }}</h4>
            <span>{{ tr('{count} 条', '{count} items').replace('{count}', String(selectedWorkspaceAiCliCommands.length)) }}</span>
          </div>
          <div v-if="selectedWorkspaceAiCliCommands.length" class="ai-command-list">
            <article v-for="item in selectedWorkspaceAiCliCommands.slice(0, 10)" :key="item.id" class="ai-command-card">
              <div class="ai-command-card__body">
                <strong>{{ item.entryName }}</strong>
                <span>{{ item.workingDirectory }}</span>
                <code>{{ item.command }}</code>
              </div>
              <div class="ai-command-card__actions">
                <button type="button" class="ghost-btn ghost-btn--small" @click="copyCommandText(item.command)">{{ t('search.copyCommand') }}</button>
                <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" @click="insertAiCommand(item)">{{ t('search.insertCurrentInput') }}</button>
              </div>
            </article>
          </div>
          <div v-else class="ai-history-empty">{{ tr('还没有 AI CLI 命令历史。', 'No AI CLI command history yet.') }}</div>
        </section>

        <section class="ai-history-section">
          <div class="ai-history-section__head">
            <h4>{{ tr('布局快照', 'Layout snapshots') }}</h4>
            <span>{{ tr('{count} 个', '{count} items').replace('{count}', String(selectedWorkspaceAiResumeSnapshots.length)) }}</span>
          </div>
          <div v-if="selectedWorkspaceAiResumeSnapshots.length" class="ai-snapshot-list">
            <button
              v-for="snapshot in selectedWorkspaceAiResumeSnapshots"
              :key="snapshot.id"
              type="button"
              class="ai-snapshot-card"
              @click="restoreAiSnapshotFromHistory(snapshot.workspaceId, snapshot.id)"
            >
              <strong>{{ snapshot.name }}</strong>
              <span>{{ relativeTimeLabel(snapshot.updatedAt) }} · {{ t('workspace.projects', { count: snapshot.tabsState.length }) }}</span>
            </button>
          </div>
          <div v-else class="ai-history-empty">{{ tr('还没有包含 AI CLI 的布局快照。', 'There are no layout snapshots containing AI CLI sessions yet.') }}</div>
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

    <aside
      v-if="visibleAiAssistItems.length && aiAssistantPinned && isWorkspaceWorkbench"
      ref="aiAssistCardRef"
      class="ai-assist-card"
      :class="[{ 'ai-assist-card--minimized': aiAssistantMinimized }]"
      :style="aiAssistCardStyle"
    >
      <header class="ai-assist-card__head" @pointerdown="beginAiAssistDrag">
        <div>
          <span class="ai-assist-card__eyebrow">
            <span>{{ t('ai.monitorTitle') }}</span>
          </span>
          <strong v-if="!aiAssistantMinimized">{{ t('ai.monitorSummary', { workspace: selectedWorkspace?.name || t('common.none'), count: visibleAiAssistItems.length }) }}</strong>
          <div v-else class="ai-assist-card__compact-strip">
            <button
              v-for="item in visibleAiAssistItems.slice(0, 4)"
              :key="item.id"
              type="button"
              class="ai-assist-card__compact-pill"
              :class="`ai-assist-card__compact-pill--${aiSessionStateBadgeClass(item.state)}`"
              :title="`${item.meta} / ${item.title}`"
              data-no-drag="true"
              @click="openAiAssistTarget(item)"
            >
              <img
                v-if="aiCliBrandIconSrc(item.info.kind)"
                :src="aiCliBrandIconSrc(item.info.kind) || undefined"
                class="ai-brand-icon ai-brand-icon--compact"
                :class="aiCliBrandIconClass(item.info.kind)"
                alt=""
              />
              <AppIcon v-else :name="item.info.iconName" :size="11" />
              <strong>{{ item.info.shortLabel }}</strong>
              <span>{{ item.statusLabel }}</span>
            </button>
          </div>
        </div>
        <div class="ai-assist-card__actions">
          <button type="button" class="icon-btn icon-btn--mini" data-no-drag="true" :title="t('ai.openHistory')" @click="openAiHistoryDrawer = true">
            <AppIcon name="recent" :size="12" />
          </button>
          <button type="button" class="icon-btn icon-btn--mini" data-no-drag="true" :title="aiAssistantMinimized ? tr('展开辅助层', 'Expand assist layer') : tr('最小化辅助层', 'Minimize assist layer')" @click="aiAssistantMinimized = !aiAssistantMinimized">
            <AppIcon :name="aiAssistantMinimized ? 'chevron-right' : 'chevron-down'" :size="12" />
          </button>
          <button type="button" class="icon-btn icon-btn--mini" data-no-drag="true" :title="tr('关闭辅助层', 'Close assist layer')" @click="aiAssistantPinned = false">
            <AppIcon name="close" :size="12" />
          </button>
        </div>
      </header>

      <div v-if="!aiAssistantMinimized" class="ai-assist-card__body">
        <div class="ai-assist-monitor-list">
          <article
            v-for="item in visibleAiAssistItems"
            :key="item.id"
            class="ai-assist-monitor-item"
            :class="`ai-assist-monitor-item--${aiSessionStateBadgeClass(item.state)}`"
          >
            <div class="ai-assist-monitor-item__pin">
              <button
                type="button"
                class="icon-btn icon-btn--mini"
                :title="pinnedAiAssistItemId === item.id ? tr('取消置顶', 'Unpin') : tr('置顶', 'Pin')"
                @click="pinAiAssistItem(item.id)"
              >
                <AppIcon name="star" :size="12" />
              </button>
            </div>
            <div class="ai-assist-monitor-item__head">
              <span class="ai-assist-monitor-item__brand">
                <img
                  v-if="resolvedAiBrandKind(selectedWorkspace, item.info) && aiCliBrandIconSrc(resolvedAiBrandKind(selectedWorkspace, item.info)!)"
                  :src="aiCliBrandIconSrc(resolvedAiBrandKind(selectedWorkspace, item.info)!) || undefined"
                  class="ai-brand-icon ai-brand-icon--eyebrow"
                  :class="aiCliBrandIconClass(resolvedAiBrandKind(selectedWorkspace, item.info)!)"
                  alt=""
                />
                <AppIcon v-else :name="item.info.iconName" :size="12" />
                <strong>{{ item.info.label }}</strong>
              </span>
              <span class="session-status-badge" :class="`session-status-badge--${aiSessionStateBadgeClass(item.state)}`">
                {{ item.statusLabel }}
              </span>
            </div>
            <div class="ai-assist-monitor-item__body">
              <div class="ai-assist-monitor-item__identity">
                <strong>{{ item.meta }}</strong>
                <span>{{ item.title }}</span>
              </div>
              <div class="ai-assist-monitor-item__path-row">
                <code :title="item.path">{{ item.path }}</code>
                <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary ai-assist-monitor-item__jump" @click="openAiAssistTarget(item)">
                  <AppIcon name="terminal" :size="12" />
                  <span>{{ t('ai.locate') }}</span>
                </button>
                <button
                  v-if="shouldCountAttentionState(item.state)"
                  type="button"
                  class="icon-btn icon-btn--mini ai-assist-monitor-item__ack"
                  :title="t('ai.handled')"
                  @click="clearSessionAttention(item.sessionId)"
                >
                  <AppIcon name="check" :size="12" />
                </button>
              </div>
            </div>
            <div class="ai-assist-monitor-item__actions sr-only">
              <button type="button" class="ghost-btn ghost-btn--small ghost-btn--primary" @click="openAiAssistTarget(item)">
                <AppIcon name="terminal" :size="13" />
                <span>{{ t('ai.locate') }}</span>
              </button>
            </div>
          </article>
        </div>
      </div>
    </aside>
  </teleport>

</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, defineComponent, h, nextTick, onBeforeUnmount, onMounted, reactive, ref, shallowRef, watch, type PropType, type VNode } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window'
import AppIcon from './components/AppIcon.vue'
import DrawerPanel from './components/DrawerPanel.vue'
import ModalShell from './components/ModalShell.vue'
import ModelPricingModal from './components/ModelPricingModal.vue'
import PopoverMenu, { type PopoverItem } from './components/PopoverMenu.vue'
import SnapshotMiniPreview from './components/SnapshotMiniPreview.vue'
const TerminalPane = defineAsyncComponent(() => import('./components/TerminalPane.vue'))
import githubIcon from './assets/env-icons/github.svg'
import goIcon from './assets/env-icons/go.svg'
import javaIcon from './assets/env-icons/java.svg'
import nodejsIcon from './assets/env-icons/nodejs.svg'
import openAiBrandIcon from './assets/ai-brand-icons/openai.svg'
import claudeBrandIcon from './assets/ai-brand-icons/claude.svg'
import geminiBrandIcon from './assets/ai-brand-icons/gemini.svg'
import hermesBrandIcon from './assets/ai-brand-icons/hermes.svg'
import deepseekBrandIcon from './assets/ai-brand-icons/deepseek.svg'
import powershellIcon from './assets/env-icons/powershell.svg'
import pythonIcon from './assets/env-icons/python.svg'
import rustIcon from './assets/env-icons/Rust-icon.svg'
import tauriIcon from './assets/env-icons/tauri.svg'
import tsIcon from './assets/env-icons/ts.svg'
import vueIcon from './assets/env-icons/vue3.svg'
import { SUPPORTED_LOCALES, setAppLocale, type AppLocale } from './i18n'
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
  applyProviderProfile,
  detectLocalProviderProfiles,
  queryManagedUsage,
  readCurrentClaudeProfile,
  readCurrentCodexProfile,
  readCurrentGeminiProfile,
  readCurrentHermesProfile,
  type DetectedProviderProfile,
  type ManagedUsageQueryResult,
} from './services/provider-detector'
import {
  createWorkflowTemplateFromInput,
  getSystemWorkflowTemplates,
  loadUserWorkflowTemplates,
  saveUserWorkflowTemplates,
} from './services/workflow-templates'
import { destroyTerminalRuntime, ensureTerminalReady, getTerminalRuntimeState, writeTerminalText } from './services/terminal-runtime'
import { sendSessionAttentionNotification } from './services/session-attention-notifier'
import type {
  AiCliKind,
  AppSection,
  PaneNode,
  PaneTerminalSession,
  ProviderConfigScope,
  ProviderKind,
  ProviderProfile,
  ProviderProfileSource,
  ProviderRequestLog,
  ProviderToolTarget,
  ProviderUsageStats,
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

type AiCliInfo = {
  isAi: boolean
  kind: AiCliKind
  label: string
  shortLabel: string
  tone: string
  iconName: string
}

type AiCliSessionItem = {
  id: string
  workspaceId: string
  tabId: string
  paneId: string
  sessionId: string
  workspaceName: string
  tabName: string
  paneName: string
  sessionName: string
  path: string
  statusLabel: string
  statusState: SessionAttentionState
  command: string
  updatedAt: string
  info: AiCliInfo
  lastInfo: AiCliInfo
}

type AiCliCommandItem = {
  id: string
  workspaceId: string
  entryId: string
  entryName: string
  command: string
  workingDirectory: string
  meta: string
  info: AiCliInfo
}

type AiAssistItem = {
  id: string
  title: string
  meta: string
  path: string
  statusLabel: string
  state: SessionAttentionState
  workspaceId: string
  tabId: string
  paneId: string
  sessionId: string
  info: AiCliInfo
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

const { t, locale } = useI18n()

const localeOptions = computed<Array<{ value: AppLocale; label: string }>>(() => [
  { value: 'zh-CN', label: t('common.localeZhCn') },
  { value: 'en-US', label: t('common.localeEnUs') },
])

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
    description: '\u4e2d\u6027\u5e95\u914d\u6a59\u8272\u5f3a\u8c03\uff0c\u80cc\u666f\u4e0d\u67d3\u8272\uff0c\u4ec5\u6309\u94ae\u4e0e\u9009\u4e2d\u6001\u7528\u6a59\u3002',
    accent: '#eb8a2f',
    background: '#f6f8fb',
    background2: '#eef2f7',
    panel: '#ffffff',
    panelElevated: '#f3f6fb',
    accentBlue: '#ef9f49',
    textPrimary: '#18212b',
    textSecondary: '#5f7286',
    swatches: ['#f6f8fb', '#ffffff', '#eb8a2f', '#ef9f49'],
    scheme: 'light',
  },
  {
    id: 'blue',
    name: '\u660e\u4eae\u84dd',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u4e2d\u6027\u5e95\u914d\u84dd\u8272\u5f3a\u8c03\uff0c\u80cc\u666f\u4e0d\u67d3\u8272\uff0c\u4ec5\u6309\u94ae\u4e0e\u9009\u4e2d\u6001\u7528\u84dd\u3002',
    accent: '#4b83ff',
    background: '#f6f8fb',
    background2: '#eef2f7',
    panel: '#ffffff',
    panelElevated: '#f3f6fb',
    accentBlue: '#6a9cff',
    textPrimary: '#18212b',
    textSecondary: '#5f7286',
    swatches: ['#f6f8fb', '#ffffff', '#4b83ff', '#6a9cff'],
    scheme: 'light',
  },
  {
    id: 'purple',
    name: '\u660e\u4eae\u7d2b',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u4e2d\u6027\u5e95\u914d\u7d2b\u8272\u5f3a\u8c03\uff0c\u80cc\u666f\u4e0d\u67d3\u8272\uff0c\u4ec5\u6309\u94ae\u4e0e\u9009\u4e2d\u6001\u7528\u7d2b\u3002',
    accent: '#8b6cff',
    background: '#f6f8fb',
    background2: '#eef2f7',
    panel: '#ffffff',
    panelElevated: '#f3f6fb',
    accentBlue: '#a084ff',
    textPrimary: '#18212b',
    textSecondary: '#5f7286',
    swatches: ['#f6f8fb', '#ffffff', '#8b6cff', '#a084ff'],
    scheme: 'light',
  },
  {
    id: 'pink',
    name: '\u660e\u4eae\u7c89',
    kind: '\u7cfb\u7edf\u4e3b\u9898',
    description: '\u4e2d\u6027\u5e95\u914d\u7c89\u8272\u5f3a\u8c03\uff0c\u80cc\u666f\u4e0d\u67d3\u8272\uff0c\u4ec5\u6309\u94ae\u4e0e\u9009\u4e2d\u6001\u7528\u7c89\u3002',
    accent: '#d974a5',
    background: '#f6f8fb',
    background2: '#eef2f7',
    panel: '#ffffff',
    panelElevated: '#f3f6fb',
    accentBlue: '#e18bb5',
    textPrimary: '#18212b',
    textSecondary: '#5f7286',
    swatches: ['#f6f8fb', '#ffffff', '#d974a5', '#e18bb5'],
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
const openAiHistoryDrawer = ref(false)
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
const openModelPricingModalState = ref(false)
const openTerminalEntryEditorModal = ref(false)
const openProviderEditorModal = ref(false)
const openTemplateEditorModal = ref(false)
const openTerminalEntriesModal = ref(false)
const activePaneMenu = ref<string | null>(null)
const activePaneHeaderMenu = ref<string | null>(null)
const activePaneBindingMenu = ref<string | null>(null)
const activePaneSessionMenu = ref<{ paneId: string; sessionId: string } | null>(null)
const activeCommandPanelPaneId = ref<string | null>(null)
const aiAssistantPinned = ref(true)
const aiAssistantMinimized = ref(false)
const immersiveAiAutoMinimized = ref(false)
const pinnedAiAssistItemId = ref<string | null>(null)
const aiAssistCardRef = ref<HTMLElement | null>(null)
const aiAssistCardPosition = ref<{ left: number; top: number } | null>(null)
const aiAssistDragging = ref(false)
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
const openProviderKindMenu = ref(false)
const openProviderSourceMenu = ref(false)
const openProviderScopeMenu = ref(false)
const openSplitActionModal = ref(false)
const openConfirmModal = ref(false)
const railCollapsed = ref(false)
const workbenchImmersive = ref(false)
const workbenchExplorerCollapsed = ref(false)
const workbenchExplorerAutoCollapsed = ref(false)
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
  confirmLabel: '',
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
const runtimeSessionOverlays = shallowRef<Record<string, Partial<PaneTerminalSession>>>({})
const runtimeEntryStatusOverlays = shallowRef<Record<string, TerminalEntry['status']>>({})
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
const activeProviderStatsId = ref('')
const activatingProviderId = ref('')
const expandedProviderId = ref('')
const activeProviderToolFilter = ref<'all' | ProviderProfile['providerKind']>('all')
const providerSearchQuery = ref('')
const providerDetectionRunning = ref(false)
const providerDetectionSummary = ref('')
/** silent | native | cc-switch：区分静默同步 / 本机 / 显式 CC Switch 导入 */
const providerSyncMode = ref<'idle' | 'silent' | 'native' | 'cc-switch'>('idle')
/** 本页生命周期内是否已做过静默 native 同步 */
const providerSilentSyncedOnce = ref(false)
const providerUsageRefreshRunning = ref(false)
const usageView = ref<'requestLogs' | 'providerStats' | 'modelStats'>('requestLogs')
const usageAppFilter = ref<'all' | ProviderRequestLog['appType']>('all')
const usageProviderFilter = ref('')
type UsagePeriodFilter = 'all' | 'today' | '1h' | '7d' | '30d' | '90d' | 'month' | 'custom'
type UsageBucketFilter = 'auto' | 'minute' | 'hour' | 'day' | 'week' | 'month'
const usagePeriodFilter = ref<UsagePeriodFilter>('7d')
// 粒度不再暴露手动选项；始终按时间范围自动映射
const usageCustomStartAt = ref('')
const usageCustomEndAt = ref('')
// OpenCode Usage 后端暂缓：若历史状态误选 opencode，强制回退 all
watch(usageAppFilter, (value) => {
  if (value === 'opencode') usageAppFilter.value = 'all'
})
/** 多选 Provider 筛选（本地 profile id）；空 = 全部；请求时映射为 identityKey */
const usageSelectedProviderIds = ref<string[]>([])
const usageProviderPickerOpen = ref(false)
const usageProviderPickerQuery = ref('')
const usageProviderPickerTriggerRef = ref<HTMLElement | null>(null)
const usageProviderPickerStyle = ref<Record<string, string>>({})
const usageSearchQuery = ref('')
/** 请求明细：cursor 分页；与 KPI/趋势解耦 */
const usageRequestLogPageSize = 80
const managedUsageNextCursor = ref<string | null>(null)
const managedUsageHasMore = ref(false)
const managedUsageTotal = ref(0)
const managedUsageRequestLogs = ref<ManagedUsageQueryResult['requestLogs']>([])
const managedUsageInFlight = ref(false)
/** 仅手动点「刷新」时显示按钮 loading，后台轮询不抢占 UI */
const usageManualRefreshRunning = ref(false)
let managedUsageRequestSeq = 0
let managedUsageDebounceTimer: number | null = null
const MANAGED_USAGE_DEBOUNCE_MS = 280
/** 筛选变更后若上一请求仍在飞，完成后补拉一次最新条件 */
let managedUsageNeedsReload = false
const MANAGED_USAGE_TIMEOUT_MS = 45_000
/** 实时用量聚合：summary / trends / providerStats（全量过滤后、limit 前） */
const managedUsageLive = ref<ManagedUsageQueryResult | null>(null)
const managedUsageUpdatedAt = ref('')
const managedUsageLoadError = ref('')
const chartHoverIndex = ref<number | null>(null)
const usageChartShellRef = ref<HTMLElement | null>(null)
const usageChartTooltipRef = ref<HTMLElement | null>(null)
const usageChartTooltipStyle = ref<Record<string, string>>({
  left: '12px',
  top: '8px',
  transform: 'none',
})
let workbenchResizeCleanup: (() => void) | null = null
let saveWorkspacesTimer: number | null = null
let saveWorkspacesIdleHandle: number | null = null
let saveWorkbenchRestoreStateTimer: ReturnType<typeof setTimeout> | null = null
let nextWorkspacePersistenceMode: 'persist' | 'transient' = 'persist'
let systemRefreshTimer: number | null = null
let systemRefreshTickTimer: number | null = null
/** Usage 页周期轮询 query_managed_usage —— 全局唯一；事件优先，poll 兜底 */
let managedUsagePollTimer: number | null = null
const MANAGED_USAGE_POLL_MS = 60_000
let unlistenUsageLogRecorded: UnlistenFn | null = null
let unlistenModelPricingUpdated: UnlistenFn | null = null
let environmentRefreshRunId = 0
let supervisorScanTimer: number | null = null
const windowVisible = ref(typeof document === 'undefined' ? true : !document.hidden)
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
  environmentVariablesText: '',
  tagsText: '',
  note: '',
})

const providerForm = reactive({
  name: '',
  providerKind: 'codex' as ProviderProfile['providerKind'],
  profileName: '',
  configPath: '',
  configScope: 'global' as ProviderConfigScope,
  managedBy: 'cli-config' as ProviderProfileSource,
  authSource: '',
  homepageUrl: '',
  requestBaseUrl: '',
  switchCommand: '',
  defaultModel: '',
  toolTargetsText: 'codex',
  color: '#4b83ff',
  note: '',
  configPayload: null as string | null,
  authPayload: null as string | null,
  isDefault: true,
  isActive: false,
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

const launchModeOptions = computed<Array<{ value: TerminalEntry['launchMode']; label: string; description: string }>>(() => [
  { value: 'open-only', label: tr('仅打开窗口', 'Open only'), description: tr('只打开终端，不自动填充和发送命令。', 'Open the terminal only without filling or sending commands automatically.') },
  { value: 'prefill', label: tr('预填命令', 'Prefill command'), description: tr('将命令填入终端输入区，由用户确认后发送。', 'Prefill the command in the terminal input so the user can review it before sending.') },
  { value: 'execute', label: tr('立即执行', 'Execute immediately'), description: tr('打开终端后按配置发送默认命令，适合可信的本地开发命令。', 'Send the default command right after opening the terminal. Best for trusted local development commands.') },
  { value: 'switch-or-create', label: tr('切换或创建', 'Switch or create'), description: tr('优先切换已有终端，没有再创建新的终端。', 'Switch to an existing terminal first; create a new one only if needed.') },
])

const providerKindOptions = computed<Array<{ value: ProviderProfile['providerKind']; label: string; description: string }>>(() => [
  { value: 'codex', label: 'Codex CLI', description: tr('读取或切换本机 Codex CLI 的配置档案。', 'Read or switch local Codex CLI profiles on this machine.') },
  { value: 'claude-code', label: 'Claude Code', description: tr('读取或切换 Claude Code 的本地配置档案。', 'Read or switch Claude Code local profiles.') },
  { value: 'gemini-cli', label: 'Gemini CLI', description: tr('读取或切换 Gemini CLI 的账号与配置档案。', 'Read or switch Gemini CLI accounts and config profiles.') },
  { value: 'opencode', label: 'OpenCode', description: tr('读取或切换 OpenCode 的本地 provider 配置。', 'Read or switch OpenCode local provider configs.') },
  { value: 'hermes', label: 'Hermes', description: tr('读取或切换 Hermes 的本地配置档案。', 'Read or switch Hermes local profiles.') },
  { value: 'custom-cli', label: providerKindLabel('custom-cli'), description: tr('适合脚本切换、团队代理或其他未预置的本地 CLI 档案。', 'Use it for script-based switching, team proxies, or any local CLI profile not covered by the built-in presets.') },
])

const providerSourceOptions = computed<Array<{ value: ProviderProfileSource; label: string; description: string }>>(() => [
  { value: 'cli-config', label: providerSourceLabel('cli-config'), description: tr('由工具自己的配置文件决定，不在运行配置中注入。', 'Driven by the tool’s own config file instead of terminal run config injection.') },
  { value: 'cc-switch', label: 'CC Switch', description: tr('通过 cc-switch 风格的配置档案或切换层管理。', 'Managed through a cc-switch style profile or switching layer.') },
  { value: 'oauth', label: providerSourceLabel('oauth'), description: tr('使用 CLI 已登录账号和本地 token。', 'Use the CLI’s signed-in account and local token state.') },
  { value: 'env', label: providerSourceLabel('env'), description: tr('由用户 shell/profile 或系统环境变量决定。', 'Driven by shell profile values or system environment variables.') },
  { value: 'script', label: providerSourceLabel('script'), description: tr('通过自定义脚本完成配置切换。', 'Switch configuration through a custom script.') },
  { value: 'manual', label: providerSourceLabel('manual'), description: tr('只记录配置档案信息，不自动读取。', 'Record profile information only, without reading it automatically.') },
])

const providerScopeOptions = computed<Array<{ value: ProviderConfigScope; label: string; description: string }>>(() => [
  { value: 'global', label: providerScopeLabel('global'), description: tr('配置位于用户目录，影响该 CLI 的默认行为。', 'The config lives in the user directory and affects the CLI default behavior globally.') },
  { value: 'workspace', label: providerScopeLabel('workspace'), description: tr('配置跟随当前 Chuchen 工作区管理。', 'The config is managed together with the current Chuchen workspace.') },
  { value: 'project', label: providerScopeLabel('project'), description: tr('配置位于项目根目录或项目子目录。', 'The config lives in the project root or a project subdirectory.') },
])

const restoreCommandStrategyOptions = computed<Array<{ value: RestoreCommandStrategy; label: string; description: string }>>(() => [
  { value: 'layout-only', label: t('settings.restoreStrategies.layoutOnly'), description: 'layout-only' },
  { value: 'prefill', label: t('settings.restoreStrategies.prefill'), description: 'prefill' },
  { value: 'execute', label: t('settings.restoreStrategies.execute'), description: 'execute' },
])

let confirmAction: (() => void) | null = null

const fontOptions = ['Cascadia Code', 'FiraCode Nerd Font', 'JetBrains Mono', 'Consolas', 'Geist Mono']

const helpContentMap = computed<Record<string, HelpContent>>(() => ({
  layout: {
    title: tr('布局说明', 'Layout guide'),
    description: tr('抽屉用于持续浏览的辅助信息，不打断主终端画布。', 'The drawer is for persistent browsing of supporting information without interrupting the main terminal canvas.'),
    sections: [
      {
        title: tr('双层结构', 'Two-layer structure'),
        body: tr('保留工作区卡片首页作为总览入口；进入后切换到左控右画布的工作台壳层。', 'Keep the workspace card home as the overview entry, then switch into the workbench shell with left controls and the right-side terminal canvas.'),
      },
      {
        title: tr('核心层级', 'Core hierarchy'),
        items: [
          tr('工作区卡片是最高一级入口。', 'Workspace cards are the top-level entry point.'),
          tr('工作区内部以 Tab 组织页面单元。', 'Tabs organize page units inside a workspace.'),
          tr('Tab 内部用 Pane 承载真正的终端实例。', 'Panes carry real terminal sessions inside each tab.'),
        ],
      },
      {
        title: tr('终端能力', 'Terminal capability'),
        items: [
          tr('已接入本地终端运行时。', 'A local terminal runtime is already integrated.'),
          tr('支持工作现场保存、恢复与任务提醒。', 'Supports workspace snapshot save/restore and task reminders.'),
        ],
      },
    ],
  },
  workspace: {
    title: t('rail.primary.workspace.label'),
    description: tr('工作区菜单负责卡片首页、进入工作台，以及已打开工作区之间的切换。', 'The workspace menu covers the home cards, workbench entry, and switching between already opened workspaces.'),
    sections: [
      {
        title: tr('为什么保留卡片首页', 'Why keep the card-based home'),
        items: [
          tr('目录一多时，纯列表不利于总览。', 'When there are many directories, a plain list is poor for overview.'),
          tr('卡片更适合展示 Tab / Pane 数量、最近使用时间和标签。', 'Cards are better for showing tab/pane counts, recent activity, and tags.'),
        ],
      },
      {
        title: tr('进入后如何工作', 'How it works after entry'),
        items: [
          tr('左侧切换已打开工作区。', 'Use the left side to switch between opened workspaces.'),
          tr('左侧项目树负责 Tab / Pane 层级导航。', 'The left project tree handles tab/pane hierarchy navigation.'),
          tr('右侧区域专注于终端工作台本身。', 'The right side focuses on the terminal workbench itself.'),
        ],
      },
    ],
  },
  recent: {
    title: t('recent.title'),
    description: tr('后续这里会收纳最近恢复的工作区、最近打开的会话以及高频命令入口。', 'This area will gather recently restored workspaces, recently opened sessions, and high-frequency command entrances.'),
    sections: [
      {
        title: tr('规划用途', 'Planned usage'),
        items: [
          tr('最近打开的工作区。', 'Recently opened workspaces.'),
          tr('最近恢复的布局快照。', 'Recently restored layout snapshots.'),
          tr('最近使用的运行配置与命令。', 'Recently used run configs and commands.'),
        ],
      },
    ],
  },
  templates: {
    title: t('templates.title'),
    description: tr('模板用于沉淀常用工作区蓝图，而不是只保存某一次运行态。', 'Templates are for preserving reusable workspace blueprints, not just a single runtime snapshot.'),
    sections: [
      {
        title: tr('规划用途', 'Planned usage'),
        items: [
          tr('常用前后端双 Pane 模板。', 'Common frontend/backend dual-pane templates.'),
          tr('AI CLI 工作台模板。', 'AI CLI workbench templates.'),
          tr('项目初始化与测试脚本模板。', 'Project bootstrap and test script templates.'),
        ],
      },
    ],
  },
  search: {
    title: t('pages.search'),
    description: tr('搜索入口会跨工作区检索名称、路径、运行配置和高频命令。', 'Search runs across workspaces for names, paths, run configs, and frequent commands.'),
    sections: [
      {
        title: tr('建议范围', 'Suggested scope'),
        items: [
          tr('工作区名称与标签。', 'Workspace names and tags.'),
          tr('路径与配置名。', 'Paths and config names.'),
          tr('默认命令与最近命令。', 'Default commands and recent commands.'),
        ],
      },
    ],
  },
  theme: {
    title: tr('主题与字体', 'Theme & font'),
    description: tr('主题、字体和界面密度属于高频配置，适合居中 Modal。', 'Theme, font, and interface density are high-frequency settings and fit a centered modal.'),
    sections: [
      {
        title: tr('设计原则', 'Design principles'),
        items: [
          tr('默认提供系统主题预设。', 'Ship system theme presets by default.'),
          tr('默认主题允许自由调整主色。', 'Allow free accent tuning in the default theme.'),
          tr('字体与主题统一在一个面板中切换。', 'Switch font and theme in one unified panel.'),
        ],
      },
    ],
  },
  settings: {
    title: t('settings.title'),
    description: tr('设置用于放置较低频的全局偏好，例如默认 Shell、启动行为和桌面端选项。', 'Settings are for lower-frequency global preferences such as default shell, startup behavior, and desktop options.'),
    sections: [
      {
        title: tr('后续规划', 'Next planning'),
        items: [
          tr('默认 Shell 与启动方式。', 'Default shell and startup flow.'),
          tr('窗口关闭、托盘与最小化策略。', 'Window close, tray, and minimize behavior.'),
          tr('Tauri 级别的路径与权限偏好。', 'Tauri-level path and permission preferences.'),
        ],
      },
    ],
  },
}))

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
const isRuntimeWorkbench = computed(() => appSection.value === 'workspace' && workspaceView.value === 'runtime' && Boolean(selectedWorkspace.value))
const immersiveWorkbenchActive = computed(() => isRuntimeWorkbench.value && workbenchImmersive.value)
const effectiveRailCollapsed = computed(() => railCollapsed.value || immersiveWorkbenchActive.value)
const activeHelpContent = computed(() => helpContentMap.value[activeHelpTopicId.value] ?? helpContentMap.value.layout)
const selectedWorkspaceEntries = computed(() =>
  (selectedWorkspace.value?.terminalEntries ?? []).map((entry) => {
    const runtimeStatus = runtimeEntryStatusOverlays.value[entry.id]
    return runtimeStatus ? { ...entry, status: runtimeStatus } : entry
  }),
)
const selectedWorkspaceEntryMap = computed(() => new Map(selectedWorkspaceEntries.value.map((entry) => [entry.id, entry])))
const selectedWorkspaceProviders = computed(() => selectedWorkspace.value?.providerProfiles ?? [])
const selectedWorkspaceProviderQuotas = computed(() => selectedWorkspace.value?.providerQuotas ?? [])
const selectedWorkspaceProviderUsageStats = computed(() => selectedWorkspace.value?.providerUsageStats ?? [])
const selectedWorkspaceAiCliSessions = computed<AiCliSessionItem[]>(() => {
  const workspace = selectedWorkspace.value
  if (!workspace) return []

  const items: AiCliSessionItem[] = []
  workspace.tabs.forEach((tab) => {
    flattenLeafPanes(tab.panes).forEach((pane) => {
      paneSessions(pane).forEach((session) => {
        const entry = workspaceEntryById(workspace, session.terminalEntryId ?? pane.terminalEntryId)
        const info = aiCliInfoForSession(workspace, pane, session)
        const lastInfo = lastAiCliInfoForSession(workspace, pane, session)
        if (!lastInfo.isAi) return
        const displayName = sessionHistoryDisplayName(workspace, pane, session, info, lastInfo)

        items.push({
          id: `${workspace.id}-${tab.id}-${pane.id}-${session.id}`,
          workspaceId: workspace.id,
          tabId: tab.id,
          paneId: pane.id,
          sessionId: session.id,
          workspaceName: workspace.name,
          tabName: tab.name,
          paneName: pane.name,
          sessionName: displayName,
          path: entry?.workingDirectory || session.pathLabel || pane.pathLabel || workspace.rootPath,
          statusLabel: explorerSessionLabel(session),
          statusState: sessionAttentionState(session),
          command: entry?.lastCommand || entry?.defaultCommand || '',
          updatedAt: session.lastActivityAt || session.lastOutputAt || session.lastCommandAt || entry?.updatedAt || tab.updatedAt || workspace.updatedAt,
          info,
          lastInfo,
        })
      })
    })
  })

  return items.sort((left, right) => new Date(right.updatedAt).getTime() - new Date(left.updatedAt).getTime())
})
const currentWorkspaceAiCliSessions = computed(() =>
  selectedWorkspaceAiCliSessions.value.filter((item) => item.info.isAi),
)
const selectedWorkspaceAiCliCommands = computed<AiCliCommandItem[]>(() => {
  const workspace = selectedWorkspace.value
  if (!workspace) return []

  return workspace.terminalEntries.flatMap((entry) => {
    const info = aiCliInfoForEntry(entry)
    if (!info.isAi) return []

    return uniqueCommandList([entry.lastCommand, ...(entry.commandHistory ?? []), entry.defaultCommand], 5).map((command, index) => ({
      id: `${workspace.id}-${entry.id}-${index}`,
      workspaceId: workspace.id,
      entryId: entry.id,
      entryName: entry.name,
      command,
      workingDirectory: entry.workingDirectory,
      meta: entry.tags.join(' / ') || launchModeLabel(entry.launchMode),
      info,
    }))
  })
})
const selectedWorkspaceAiResumeSnapshots = computed(() =>
  selectedWorkspaceSnapshots.value
    .filter((snapshot) => snapshot.tabsState.some((tab) =>
      flattenLeafPanes(tab.panes).some((pane) =>
        paneSessions(pane).some((session) => {
          const entry = selectedWorkspace.value?.terminalEntries.find((item) => item.id === (session.terminalEntryId ?? pane.terminalEntryId))
          return aiCliInfoForEntry(entry).isAi || Boolean(session.lastAiCliKind) || Boolean(session.aiCliKind)
        }),
      ),
    ))
    .slice(0, 6),
)
const aiAssistItems = computed<AiAssistItem[]>(() =>
  currentWorkspaceAiCliSessions.value
    .filter((item) => shouldShowAiAssistState(item.statusState) || item.statusState === 'fresh' || item.statusState === 'idle')
    .map((item): AiAssistItem => ({
      id: item.id,
      title: item.sessionName,
      meta: item.tabName,
      path: item.path,
      statusLabel: item.statusLabel,
      state: item.statusState,
      workspaceId: item.workspaceId,
      tabId: item.tabId,
      paneId: item.paneId,
      sessionId: item.sessionId,
      info: item.info,
    })),
)
const activeAiAssistItem = computed<AiAssistItem | null>(() => {
  const items = aiAssistItems.value
  if (!items.length) return null
  return items.find((item) => item.id === pinnedAiAssistItemId.value)
    ?? items.find((item) => shouldCountAttentionState(item.state))
    ?? items[0]
})
const visibleAiAssistItems = computed(() => {
  const items = aiAssistItems.value
  if (!items.length) return []
  const primary = activeAiAssistItem.value
  const ordered = primary
    ? [primary, ...items.filter((item) => item.id !== primary.id)]
    : items
  return ordered.slice(0, 6)
})
const providerToolFilters = computed<Array<{ id: 'all' | ProviderProfile['providerKind']; label: string; count: number }>>(() => {
  const providers = selectedWorkspaceProviders.value
  const countByKind = providers.reduce<Record<string, number>>((result, provider) => {
    result[provider.providerKind] = (result[provider.providerKind] ?? 0) + 1
    return result
  }, {})

  return [
    { id: 'all', label: t('common.all'), count: providers.length },
    ...providerKindOptions.value.map((option) => ({
      id: option.value,
      label: option.label,
      count: countByKind[option.value] ?? 0,
    })),
  ]
})
const filteredWorkspaceProviders = computed(() => {
  const query = normalizeSearchText(providerSearchQuery.value)
  return selectedWorkspaceProviders.value.filter((provider) => {
    if (activeProviderToolFilter.value !== 'all' && provider.providerKind !== activeProviderToolFilter.value) {
      return false
    }
    if (!query) return true

    return [
      provider.name,
      provider.profileName,
      provider.configPath,
      provider.authSource,
      provider.defaultModel,
      provider.note ?? '',
    ].some((value) => normalizeSearchText(value).includes(query))
  })
})
const activeProviderProfile = computed(() => {
  const providerId = activeProviderStatsId.value || filteredWorkspaceProviders.value[0]?.id || selectedWorkspaceProviders.value[0]?.id || ''
  return selectedWorkspaceProviders.value.find((provider) => provider.id === providerId) ?? null
})
const activeProviderQuota = computed(() => {
  const provider = activeProviderProfile.value
  if (!provider) return null
  return selectedWorkspaceProviderQuotas.value.find((quota) => quota.providerProfileId === provider.id) ?? null
})
const activeProviderStats = computed(() => {
  const provider = activeProviderProfile.value
  if (!provider) return null
  const stats = selectedWorkspaceProviderUsageStats.value.find((item) => item.providerProfileId === provider.id)
  return normalizeProviderUsageStatsForProfile(provider, stats ?? createEmptyProviderUsageStatsForProfile(provider.id))
})

/** 时间段是否落在 usagePeriodFilter 内（明细二次过滤兜底；主过滤在后端） */
function usageLogInPeriod(createdAt: string, period: UsagePeriodFilter, nowMs = Date.now()) {
  if (period === 'all') return true
  if (period === 'custom') {
    const ts = new Date(createdAt).getTime()
    if (Number.isNaN(ts)) return false
    const startIso = datetimeLocalToIso(usageCustomStartAt.value)
    const endIso = datetimeLocalToIso(usageCustomEndAt.value)
    if (startIso) {
      const start = new Date(startIso).getTime()
      if (!Number.isNaN(start) && ts < start) return false
    }
    if (endIso) {
      const end = new Date(endIso).getTime()
      if (!Number.isNaN(end) && ts > end) return false
    }
    return true
  }
  const ts = new Date(createdAt).getTime()
  if (Number.isNaN(ts)) return false
  if (period === '1h') return nowMs - ts <= 3_600_000
  if (period === 'today') return isSameLocalDay(createdAt)
  if (period === 'month') {
    const d = new Date(createdAt)
    const now = new Date(nowMs)
    return d.getFullYear() === now.getFullYear() && d.getMonth() === now.getMonth()
  }
  const days = period === '7d' ? 7 : period === '30d' ? 30 : period === '90d' ? 90 : 0
  if (!days) return true
  return nowMs - ts <= days * 86400_000
}

function datetimeLocalToIso(value: string): string | null {
  const raw = value.trim()
  if (!raw) return null
  const date = new Date(raw)
  if (Number.isNaN(date.getTime())) return null
  return date.toISOString()
}

/** Usage 周期短标签（Provider 卡片副行 / 提示共用） */
function usagePeriodShortLabel(period: UsagePeriodFilter) {
  if (period === '1h') return '1h'
  if (period === 'today') return tr('今日', 'Today')
  if (period === '7d') return '7d'
  if (period === '30d') return '30d'
  if (period === 'month') return tr('本月', 'Month')
  if (period === '90d') return '90d'
  if (period === 'custom') return tr('自定义', 'Custom')
  return tr('全部', 'All')
}

function resolveAutoUsageBucket(period: UsagePeriodFilter, startAt?: string | null, endAt?: string | null): Exclude<UsageBucketFilter, 'auto'> {
  if (period === '1h') return 'minute'
  if (period === 'today') return 'hour'
  if (period === '7d' || period === '30d' || period === 'month') return 'day'
  if (period === '90d') return 'week'
  if (period === 'all') return 'month'
  if (period === 'custom') {
    const startMs = startAt ? new Date(startAt).getTime() : Number.NaN
    const endMs = endAt ? new Date(endAt).getTime() : Date.now()
    if (!Number.isNaN(startMs) && !Number.isNaN(endMs)) {
      const span = Math.max(0, endMs - startMs)
      if (span <= 3_600_000) return 'minute'
      if (span <= 86_400_000) return 'hour'
      if (span <= 30 * 86_400_000) return 'day'
      if (span <= 90 * 86_400_000) return 'week'
      return 'month'
    }
    return 'day'
  }
  return 'day'
}

/**
 * Provider 卡片指标范围：
 * - 有 managedUsageLive：副指标是「当前 Usage 周期」内聚合，不是全历史
 * - 无 live：回退本地缓存快照
 */
const providerMetricsScopeShort = computed(() => {
  if (managedUsageLive.value) return usagePeriodShortLabel(usagePeriodFilter.value)
  return tr('缓存', 'Cache')
})

const providerMetricsScopeHint = computed(() => {
  if (managedUsageLive.value) {
    return tr(
      `副指标为当前周期（${usagePeriodShortLabel(usagePeriodFilter.value)}）内聚合；今日请求/成本仍按自然日。卡片总量来自后端 providerStats，不受明细分页影响。`,
      `Secondary metrics are for the current period (${usagePeriodShortLabel(usagePeriodFilter.value)}). Today uses calendar day. Card totals come from backend providerStats and are unaffected by detail pagination.`,
    )
  }
  return tr(
    '尚未拉取实时 Usage，副指标来自本地缓存快照，可能偏旧。进入 Usage 页或点击刷新后会改为当前周期。',
    'Live Usage has not been loaded; secondary metrics come from a local cache snapshot and may be stale. Open Usage or refresh to switch to the current period.',
  )
})

const providerStatusRailText = computed(() => {
  if (providerDetectionRunning.value) {
    if (providerSyncMode.value === 'cc-switch') return tr('正在从 CC Switch 导入档案…', 'Importing profiles from CC Switch…')
    if (providerSyncMode.value === 'silent') return tr('正在静默同步本机配置…', 'Silently syncing local configs…')
    return tr('正在同步本机配置…', 'Syncing local configs…')
  }
  if (providerDetectionSummary.value) return providerDetectionSummary.value
  if (providerSilentSyncedOnce.value) {
    return tr(
      `已同步本机配置 · ${selectedWorkspaceProviders.value.length} 个档案（identityKey 去重）`,
      `Local configs synced · ${selectedWorkspaceProviders.value.length} profiles (identityKey-deduped)`,
    )
  }
  return tr(
    '进入页面将自动静默同步本机配置；CC Switch 需点「从 CC Switch 导入」。当前 detect API 仍返回合并目录，前后端拆分前不会假装只含 native。',
    'Page entry silently syncs local configs; CC Switch needs explicit import. Detect still returns a merged catalog — frontend will not fake a native-only split.',
  )
})

/** 将本地 profile id 映射为后端 providerIds（identityKey） */
/** 将本地 profile id 映射为后端 providerIds（identityKey）
 * - null：未选筛选，查全部
 * - 空数组：选了档案但没有可映射 identityKey（前端应短路为空结果，勿当全量）
 */
function resolveUsageProviderIdentityKeys(): string[] | null {
  if (!usageSelectedProviderIds.value.length) return null
  const keys: string[] = []
  for (const id of usageSelectedProviderIds.value) {
    const profile = selectedWorkspaceProviders.value.find((provider) => provider.id === id)
    if (profile?.identityKey) {
      keys.push(profile.identityKey)
      continue
    }
    if (id.startsWith('identity:')) {
      keys.push(id.slice('identity:'.length))
      continue
    }
  }
  // 允许空数组：表示“有选择但不可映射”
  return keys
}

/**
 * KPI：优先后端 summary（过滤后、limit 前全量聚合）
 * 后端已支持 providerIds，多选也不再从分页明细重算
 */
const activeUsageSummary = computed(() => {
  if (managedUsageLive.value?.summary) {
    return normalizeProviderUsageSummary(managedUsageLive.value.summary)
  }
  return recalculateProviderUsageSummary(activeUsageLogs.value)
})

const usageHasMoreRequestLogs = computed(() => Boolean(managedUsageHasMore.value))

const usageEffectiveBucket = computed(() => {
  // 粒度完全由时间范围自动决定，不再暴露手动 bucket，避免 30d+minute 这类无意义组合
  const range = usagePeriodRange(usagePeriodFilter.value)
  return resolveAutoUsageBucket(usagePeriodFilter.value, range.startAt, range.endAt)
})
const usageProviderPickerGroups = computed(() => {
  type PickerItem = {
    id: string
    name: string
    model: string
    providerKind: ProviderProfile['providerKind']
    source: string
  }
  type PickerGroup = {
    id: string
    label: string
    items: PickerItem[]
  }

  const query = normalizeSearchText(usageProviderPickerQuery.value)
  const groups: PickerGroup[] = providerKindOptions.value.map((option) => ({
    id: option.value,
    label: option.label,
    items: selectedWorkspaceProviders.value
      .filter((provider) => provider.providerKind === option.value)
      .filter((provider) => {
        if (!query) return true
        return [
          provider.name,
          provider.profileName,
          provider.defaultModel,
          provider.managedBy,
          provider.identityKey ?? '',
        ].some((value) => normalizeSearchText(value).includes(query))
      })
      .map((provider) => ({
        id: provider.id,
        name: provider.name,
        model: provider.defaultModel,
        providerKind: provider.providerKind,
        source: providerSourceLabel(provider.managedBy),
      })),
  })).filter((group) => group.items.length > 0 || !query)

  const orphanLogs = allWorkspaceUsageLogs.value.filter((log) => String(log.providerProfileId).startsWith('identity:') || String(log.providerProfileId).startsWith('unknown:'))
  const orphanMap = new Map<string, PickerItem>()
  for (const log of orphanLogs) {
    if (orphanMap.has(log.providerProfileId)) continue
    orphanMap.set(log.providerProfileId, {
      id: log.providerProfileId,
      name: log.managedProviderName || tr('未关联档案', 'Unlinked profile'),
      model: log.model || '',
      providerKind: usageLogProviderKind(log) ?? 'custom-cli',
      source: tr('未关联档案', 'Unlinked profile'),
    })
  }
  const orphanItems = [...orphanMap.values()].filter((item) => {
    if (!query) return true
    return normalizeSearchText(`${item.name} ${item.model} ${item.id}`).includes(query)
  })
  if (orphanItems.length) {
    groups.push({
      id: 'unlinked',
      label: tr('未关联档案', 'Unlinked profiles'),
      items: orphanItems,
    })
  }
  return groups
})

const usageSelectedProviderChips = computed(() => {
  const byId = new Map(selectedWorkspaceProviders.value.map((provider) => [provider.id, provider]))
  return usageSelectedProviderIds.value.map((id) => {
    const profile = byId.get(id)
    if (profile) {
      return {
        id: profile.id,
        name: profile.name,
        providerKind: profile.providerKind,
      }
    }
    const log = allWorkspaceUsageLogs.value.find((item) => item.providerProfileId === id)
    return {
      id,
      name: log?.managedProviderName || tr('未关联档案', 'Unlinked profile'),
      providerKind: (log ? (usageLogProviderKind(log) ?? 'custom-cli') : 'custom-cli') as ProviderProfile['providerKind'],
    }
  })
})

// Per-provider inline metrics for the card list (requests / cost / balance / last-used).
const providerMetricsById = computed(() => {
  const map = new Map<string, {
    requests: number
    todayRequests: number
    totalRequests: number
    costUsd: number
    todayCostUsd: number
    totalCostUsd: number
    cacheHitRate: number
    balance: number | null
    lastUsedAt: string
  }>()

  // 后端 providerStats：过滤后全量聚合，不受 requestLogs 分页影响
  const statsByIdentity = new Map<string, NonNullable<ManagedUsageQueryResult['providerStats']>[number]>()
  if (managedUsageLive.value?.providerStats?.length) {
    for (const stat of managedUsageLive.value.providerStats) {
      const key = (stat.providerId || '').trim()
      if (key) statsByIdentity.set(key, stat)
    }
  }

  // 今日请求/成本仍按自然日：仅用已加载明细估算（非全量 KPI）
  const liveLogsByProfile = new Map<string, Array<{ createdAt: string; costUsd: number }>>()
  if (managedUsageRequestLogs.value.length) {
    const byIdentity = new Map<string, string>()
    for (const provider of selectedWorkspaceProviders.value) {
      const key = (provider.identityKey || '').trim()
      if (key && !byIdentity.has(key)) byIdentity.set(key, provider.id)
    }
    for (const log of managedUsageRequestLogs.value) {
      const identity = (log.providerId || '').trim()
      const profileId = identity ? byIdentity.get(identity) : undefined
      if (!profileId) continue
      const list = liveLogsByProfile.get(profileId) ?? []
      list.push({
        createdAt: log.createdAt,
        costUsd: log.totalCostUsd ?? 0,
      })
      liveLogsByProfile.set(profileId, list)
    }
  }

  for (const provider of selectedWorkspaceProviders.value) {
    const rawStats = selectedWorkspaceProviderUsageStats.value.find((item) => item.providerProfileId === provider.id)
    const stats = normalizeProviderUsageStatsForProfile(provider, rawStats ?? createEmptyProviderUsageStatsForProfile(provider.id))
    const quota = selectedWorkspaceProviderQuotas.value.find((item) => item.providerProfileId === provider.id) ?? null
    const identity = (provider.identityKey || '').trim()
    const backendStat = identity ? statsByIdentity.get(identity) : undefined
    const liveLogs = liveLogsByProfile.get(provider.id) ?? []
    const todayLogs = liveLogs.filter((log) => isSameLocalDay(log.createdAt))
    const todayRequests = todayLogs.length || (quota?.requestsToday ?? 0)
    const todayCostUsd = todayLogs.reduce((acc, log) => acc + (log.costUsd ?? 0), 0)
    const lastUsedAt = backendStat?.lastActivityAt
      || liveLogs.reduce(
        (acc, log) => (new Date(log.createdAt).getTime() > new Date(acc || 0).getTime() ? log.createdAt : acc),
        '',
      )
    const totalCostUsd = backendStat?.totalCostUsd ?? stats.summary.totalCostUsd
    const totalRequests = backendStat?.totalRequests ?? stats.summary.totalRequests
    const cacheHitRate = backendStat?.cacheHitRate ?? stats.summary.cacheHitRate
    map.set(provider.id, {
      requests: todayRequests,
      todayRequests,
      totalRequests,
      costUsd: todayCostUsd,
      todayCostUsd,
      totalCostUsd,
      cacheHitRate,
      balance: quota?.usdRemaining ?? null,
      lastUsedAt,
    })
  }
  return map
})
const filteredProviderCards = computed(() => {
  const cards = filteredWorkspaceProviders.value.map((provider) => ({
    provider,
    metrics: providerMetricsById.value.get(provider.id) ?? { requests: 0, todayRequests: 0, totalRequests: 0, costUsd: 0, todayCostUsd: 0, totalCostUsd: 0, cacheHitRate: 0, balance: null, lastUsedAt: '' },
    url: (provider.homepageUrl || provider.requestBaseUrl || '').replace(/^https?:\/\//, '').replace(/\/$/, ''),
  }))
  // Active provider always first
  return [...cards].sort((a, b) => {
    if (a.provider.isActive && !b.provider.isActive) return -1
    if (!a.provider.isActive && b.provider.isActive) return 1
    return 0
  })
})
/** 明细表：使用累计 requestLogs（cursor 分页追加），不是只取最后一页 */
const allWorkspaceUsageLogs = computed(() => {
  if (managedUsageLive.value || managedUsageRequestLogs.value.length) {
    // 契约：Usage.providerId ↔ Profile.identityKey 唯一匹配；禁止拆解 providerId / 名称回退当主键
    const byIdentity = new Map<string, ProviderProfile>()
    for (const provider of selectedWorkspaceProviders.value) {
      const key = (provider.identityKey || '').trim()
      if (key && !byIdentity.has(key)) byIdentity.set(key, provider)
    }

    return managedUsageRequestLogs.value.map((log, index) => {
      const identity = (log.providerId || '').trim()
      const matched = identity ? byIdentity.get(identity) : undefined
      const appType = (['claude', 'codex', 'gemini', 'opencode', 'hermes'].includes(log.appType)
        ? log.appType
        : matched
          ? providerKindToAppType(matched.providerKind)
          : 'codex') as ProviderRequestLog['appType']
      return {
        id: log.id || `managed-usage-${index}`,
        // 未匹配到本地 Profile 时用 managed providerId 作占位 id（不拆解、不发明 active）
        providerProfileId: matched?.id || (identity ? `identity:${identity}` : `unknown:${log.providerName || index}`),
        appType,
        model: log.model || log.pricingModel || '',
        pricingModel: log.pricingModel || '',
        inputTokens: log.inputTokens ?? 0,
        outputTokens: log.outputTokens ?? 0,
        cacheReadTokens: log.cacheReadTokens ?? 0,
        cacheCreationTokens: log.cacheCreationTokens ?? 0,
        costUsd: log.totalCostUsd ?? 0,
        inputCostUsd: log.inputCostUsd ?? 0,
        outputCostUsd: log.outputCostUsd ?? 0,
        cacheReadCostUsd: log.cacheReadCostUsd ?? 0,
        cacheCreationCostUsd: log.cacheCreationCostUsd ?? 0,
        firstTokenMs: log.firstTokenMs ?? null,
        statusCode: log.statusCode ?? 0,
        durationMs: log.durationMs ?? 0,
        dataSource: log.dataSource || 'managed',
        createdAt: log.createdAt,
        managedProviderId: identity || undefined,
        managedProviderName: log.providerName || undefined,
      } satisfies ProviderRequestLog
    }).sort((left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime())
  }

  return selectedWorkspaceProviderUsageStats.value
    .flatMap((stats) => stats.requestLogs || [])
    .sort((left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime())
})

// Unique provider names for filter dropdown
const usageProviderOptions = computed(() => {
  const names = new Set<string>()
  for (const log of allWorkspaceUsageLogs.value) {
    const name = usageLogProviderName(log)
    if (name) names.add(name)
  }
  return [...names].sort()
})

const activeUsageLogs = computed(() => {
  // 请求日志表：后端已按筛选返回；客户端只做搜索与兜底 period 过滤
  const baseLogs = allWorkspaceUsageLogs.value
  const query = normalizeSearchText(usageSearchQuery.value)
  const now = Date.now()
  const period = usagePeriodFilter.value

  return baseLogs.filter((log) => {
    if (!usageLogInPeriod(log.createdAt, period, now)) return false
    if (usageAppFilter.value !== 'all' && log.appType !== usageAppFilter.value) return false
    if (!query) return true
    return [
      usageLogProviderName(log),
      log.model,
      log.dataSource,
      log.appType,
      log.managedProviderId,
    ].some((value) => value && normalizeSearchText(value).includes(query))
  })
})
const providerUsageRows = computed(() => {
  // 排行榜：优先后端 providerStats（全量聚合）
  if (managedUsageLive.value?.providerStats?.length) {
    const byIdentity = new Map<string, ProviderProfile>()
    for (const provider of selectedWorkspaceProviders.value) {
      const key = (provider.identityKey || '').trim()
      if (key && !byIdentity.has(key)) byIdentity.set(key, provider)
    }
    return managedUsageLive.value.providerStats
      .map((stat) => {
        const identity = (stat.providerId || '').trim()
        const matched = identity ? byIdentity.get(identity) : undefined
        const models = new Set(stat.models || [])
        const appTypes = new Set<string>(stat.appType ? [stat.appType] : [])
        return {
          providerProfileId: matched?.id || (identity ? `identity:${identity}` : `unknown:${stat.providerName}`),
          providerName: matched?.name
            || (identity
              ? (stat.providerName
                ? tr(`未关联档案 · ${stat.providerName}`, `Unlinked · ${stat.providerName}`)
                : tr('未关联档案', 'Unlinked profile'))
              : (stat.providerName || tr('未关联档案', 'Unlinked profile'))),
          appTypes,
          models,
          totalRequests: stat.totalRequests ?? 0,
          totalInputTokens: stat.totalInputTokens ?? 0,
          totalOutputTokens: stat.totalOutputTokens ?? 0,
          totalCacheReadTokens: stat.totalCacheReadTokens ?? 0,
          totalCacheCreationTokens: stat.totalCacheCreationTokens ?? 0,
          totalCostUsd: Number((stat.totalCostUsd ?? 0).toFixed(4)),
          lastActivityAt: stat.lastActivityAt || '',
          cacheHitRate: stat.cacheHitRate ?? 0,
        }
      })
      .sort((left, right) => right.totalCostUsd - left.totalCostUsd)
  }

  // 无 live stats 时回退本地缓存明细（可能截断）
  const rows = new Map<string, {
    providerProfileId: string
    providerName: string
    appTypes: Set<string>
    models: Set<string>
    totalRequests: number
    totalInputTokens: number
    totalOutputTokens: number
    totalCacheReadTokens: number
    totalCacheCreationTokens: number
    totalCostUsd: number
    lastActivityAt: string
  }>()

  activeUsageLogs.value.forEach((log) => {
    const existing = rows.get(log.providerProfileId) ?? {
      providerProfileId: log.providerProfileId,
      providerName: usageLogProviderName(log),
      appTypes: new Set<string>(),
      models: new Set<string>(),
      totalRequests: 0,
      totalInputTokens: 0,
      totalOutputTokens: 0,
      totalCacheReadTokens: 0,
      totalCacheCreationTokens: 0,
      totalCostUsd: 0,
      lastActivityAt: log.createdAt,
    }
    existing.appTypes.add(log.appType)
    existing.models.add(log.model)
    existing.totalRequests += 1
    existing.totalInputTokens += log.inputTokens
    existing.totalOutputTokens += log.outputTokens
    existing.totalCacheReadTokens += log.cacheReadTokens
    existing.totalCacheCreationTokens += log.cacheCreationTokens
    existing.totalCostUsd += log.costUsd
    if (new Date(log.createdAt).getTime() > new Date(existing.lastActivityAt).getTime()) {
      existing.lastActivityAt = log.createdAt
    }
    rows.set(log.providerProfileId, existing)
  })

  return Array.from(rows.values())
    .map((row) => ({
      ...row,
      cacheHitRate: row.totalCacheReadTokens + row.totalInputTokens + row.totalCacheCreationTokens > 0
        ? row.totalCacheReadTokens / (row.totalCacheReadTokens + row.totalInputTokens + row.totalCacheCreationTokens)
        : 0,
      totalCostUsd: Number(row.totalCostUsd.toFixed(4)),
    }))
    .sort((left, right) => right.totalCostUsd - left.totalCostUsd)
})
const modelUsageRows = computed(() => {
  const query = normalizeSearchText(usageSearchQuery.value)

  // 模型排行只信后端 modelStats（含 rollup）；禁止用分页 logs 重算
  if (managedUsageLive.value?.modelStats) {
    return managedUsageLive.value.modelStats
      .filter((stat) => {
        if (!query) return true
        const haystack = [
          stat.model,
          ...(stat.providerNames || []),
          ...(stat.appTypes || []),
        ].join(' ')
        return normalizeSearchText(haystack).includes(query)
      })
      .map((stat) => ({
        model: stat.model,
        providerNames: new Set(stat.providerNames || []),
        appTypes: new Set(stat.appTypes || []),
        totalRequests: stat.totalRequests ?? 0,
        totalInputTokens: stat.totalInputTokens ?? 0,
        totalOutputTokens: stat.totalOutputTokens ?? 0,
        totalCacheReadTokens: stat.totalCacheReadTokens ?? 0,
        totalCacheCreationTokens: stat.totalCacheCreationTokens ?? 0,
        totalCostUsd: Number((stat.totalCostUsd ?? 0).toFixed(4)),
        lastActivityAt: stat.lastActivityAt || '',
      }))
      .sort((left, right) => right.totalCostUsd - left.totalCostUsd)
  }

  // 无 live 时兜底：仅本地已加载明细（可能截断）
  const rows = new Map<string, {
    model: string
    providerNames: Set<string>
    appTypes: Set<string>
    totalRequests: number
    totalInputTokens: number
    totalOutputTokens: number
    totalCacheReadTokens: number
    totalCacheCreationTokens: number
    totalCostUsd: number
    lastActivityAt: string
  }>()

  activeUsageLogs.value.forEach((log) => {
    const existing = rows.get(log.model) ?? {
      model: log.model,
      providerNames: new Set<string>(),
      appTypes: new Set<string>(),
      totalRequests: 0,
      totalInputTokens: 0,
      totalOutputTokens: 0,
      totalCacheReadTokens: 0,
      totalCacheCreationTokens: 0,
      totalCostUsd: 0,
      lastActivityAt: log.createdAt,
    }
    existing.providerNames.add(usageLogProviderName(log))
    existing.appTypes.add(log.appType)
    existing.totalRequests += 1
    existing.totalInputTokens += log.inputTokens
    existing.totalOutputTokens += log.outputTokens
    existing.totalCacheReadTokens += log.cacheReadTokens
    existing.totalCacheCreationTokens += log.cacheCreationTokens
    existing.totalCostUsd += log.costUsd
    if (new Date(log.createdAt).getTime() > new Date(existing.lastActivityAt).getTime()) {
      existing.lastActivityAt = log.createdAt
    }
    rows.set(log.model, existing)
  })

  return Array.from(rows.values())
    .map((row) => ({
      ...row,
      totalCostUsd: Number(row.totalCostUsd.toFixed(4)),
    }))
    .sort((left, right) => right.totalCostUsd - left.totalCostUsd)
})
/** 从 logs 按时间桶聚合趋势（无后端 trends 时回退；明细可能截断） */
function buildUsageTrendFromLogs(logs: ProviderRequestLog[], period: typeof usagePeriodFilter.value) {
  const bucketMode = resolveAutoUsageBucket(period, usagePeriodRange(period).startAt, usagePeriodRange(period).endAt)
  const byBucket = new Map<string, {
    inputTokens: number
    outputTokens: number
    cacheReadTokens: number
    cacheCreationTokens: number
    costUsd: number
    timestamp: string
  }>()

  for (const log of logs) {
    const d = new Date(log.createdAt)
    if (Number.isNaN(d.getTime())) continue
    let key: string
    let bucketTs: Date
    if (bucketMode === 'minute') {
      key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}-${d.getHours()}-${d.getMinutes()}`
      bucketTs = new Date(d.getFullYear(), d.getMonth(), d.getDate(), d.getHours(), d.getMinutes())
    } else if (bucketMode === 'hour') {
      key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}-${d.getHours()}`
      bucketTs = new Date(d.getFullYear(), d.getMonth(), d.getDate(), d.getHours())
    } else if (bucketMode === 'day') {
      key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`
      bucketTs = new Date(d.getFullYear(), d.getMonth(), d.getDate())
    } else if (bucketMode === 'week') {
      const dayOfWeek = d.getDay() === 0 ? 6 : d.getDay() - 1
      const weekStart = new Date(d.getFullYear(), d.getMonth(), d.getDate() - dayOfWeek)
      key = `${weekStart.getFullYear()}-W${weekStart.getMonth()}-${weekStart.getDate()}`
      bucketTs = weekStart
    } else {
      key = `${d.getFullYear()}-${d.getMonth()}`
      bucketTs = new Date(d.getFullYear(), d.getMonth(), 1)
    }
    if (!byBucket.has(key)) {
      byBucket.set(key, {
        inputTokens: 0,
        outputTokens: 0,
        cacheReadTokens: 0,
        cacheCreationTokens: 0,
        costUsd: 0,
        timestamp: bucketTs.toISOString(),
      })
    }
    const b = byBucket.get(key)!
    b.inputTokens += log.inputTokens ?? 0
    b.outputTokens += log.outputTokens ?? 0
    b.cacheReadTokens += log.cacheReadTokens ?? 0
    b.cacheCreationTokens += log.cacheCreationTokens ?? 0
    b.costUsd += log.costUsd ?? 0
  }

  return [...byBucket.values()]
    .sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())
    .map((pt) => ({
      ...pt,
      costUsd: Number(pt.costUsd.toFixed(6)),
      label: formatTrendBucketLabel(pt.timestamp, period, bucketMode),
    }))
}

/** 优先后端 trends（全量聚合）；无 trends 时回退已加载明细（可能截断） */
const usageTrendChartData = computed(() => {
  if (managedUsageLive.value?.trends?.length) {
    const period = usagePeriodFilter.value
    const bucket = usageEffectiveBucket.value
    return managedUsageLive.value.trends.map((point) => ({
      timestamp: point.timestamp,
      inputTokens: point.inputTokens ?? 0,
      outputTokens: point.outputTokens ?? 0,
      cacheReadTokens: point.cacheReadTokens ?? 0,
      cacheCreationTokens: point.cacheCreationTokens ?? 0,
      costUsd: Number((point.costUsd ?? 0).toFixed(6)),
      label: formatTrendBucketLabel(point.timestamp, period, bucket),
    }))
  }
  return buildUsageTrendFromLogs(activeUsageLogs.value, usagePeriodFilter.value)
})
const usageTrendHasSignal = computed(() => usageTrendChartData.value.some((point) => (
  point.inputTokens > 0
  || point.outputTokens > 0
  || point.cacheReadTokens > 0
  || (point.cacheCreationTokens ?? 0) > 0
  || point.costUsd > 0
)))
const usageTrendMaxTokens = computed(() => Math.max(
  1,
  ...usageTrendChartData.value.map((point) => Math.max(
    point.inputTokens,
    point.outputTokens,
    point.cacheReadTokens,
    point.cacheCreationTokens ?? 0,
  )),
))
const usageTrendMaxCost = computed(() => Math.max(1, ...usageTrendChartData.value.map((point) => point.costUsd)))
const usageTrendLabelPoints = computed(() => {
  const arr = usageTrendChartData.value
  // 超过12个数据点时，按间隔只显示约8个label，避免x轴拥挤
  const step = arr.length > 12 ? Math.ceil(arr.length / 8) : 1
  return arr.map((point, index) => ({
    label: point.label,
    x: arr.length <= 1 ? 500 : 56 + (884 / Math.max(arr.length - 1, 1)) * index,
    pct: arr.length <= 1 ? 50 : (index / Math.max(arr.length - 1, 1)) * 100,
    show: index === 0 || index === arr.length - 1 || index % step === 0,
  }))
})
const usageTrendInputPath = computed(() => buildUsageLinePath(usageTrendChartData.value, (point) => point.inputTokens, usageTrendMaxTokens.value))
const usageTrendOutputPath = computed(() => buildUsageLinePath(usageTrendChartData.value, (point) => point.outputTokens, usageTrendMaxTokens.value))
const usageTrendCachePath = computed(() => buildUsageLinePath(usageTrendChartData.value, (point) => point.cacheReadTokens, usageTrendMaxTokens.value))
const usageTrendCostPath = computed(() => buildUsageLinePath(usageTrendChartData.value, (point) => point.costUsd, usageTrendMaxCost.value))
// Area paths (closed for gradient fill)
const usageTrendInputArea = computed(() => buildUsageAreaPath(usageTrendChartData.value, (point) => point.inputTokens, usageTrendMaxTokens.value))
const usageTrendOutputArea = computed(() => buildUsageAreaPath(usageTrendChartData.value, (point) => point.outputTokens, usageTrendMaxTokens.value))
const usageTrendCacheArea = computed(() => buildUsageAreaPath(usageTrendChartData.value, (point) => point.cacheReadTokens, usageTrendMaxTokens.value))
// Y-axis actual value labels (token scale, 5 levels) — chart plot y: 52..284（与网格线对齐）
const USAGE_CHART_TOP = 52
const USAGE_CHART_BOTTOM = 284
const USAGE_CHART_HEIGHT = USAGE_CHART_BOTTOM - USAGE_CHART_TOP
const usageTrendYLabels = computed(() => {
  const max = usageTrendMaxTokens.value
  return [100, 75, 50, 25, 0].map((pct) => ({
    y: USAGE_CHART_TOP + ((100 - pct) / 100) * USAGE_CHART_HEIGHT,
    label: pct === 0 ? '0' : formatCompactWan(Math.round(max * pct / 100)),
  }))
})
// Y-axis cost labels (right side)
const usageTrendCostYLabels = computed(() => {
  const max = usageTrendMaxCost.value
  return [100, 75, 50, 25, 0].map((pct) => ({
    y: USAGE_CHART_TOP + ((100 - pct) / 100) * USAGE_CHART_HEIGHT,
    label: max === 0 ? '$0' : `$${(max * pct / 100).toFixed(max * pct / 100 >= 1 ? 0 : 2)}`,
  }))
})

function chartYForToken(value: number) {
  const max = Math.max(1, usageTrendMaxTokens.value)
  return USAGE_CHART_BOTTOM - (Math.max(0, value) / max) * USAGE_CHART_HEIGHT
}

function chartYForCost(value: number) {
  const max = Math.max(1, usageTrendMaxCost.value)
  return USAGE_CHART_BOTTOM - (Math.max(0, value) / max) * USAGE_CHART_HEIGHT
}

function onUsageChartMouseMove(event: MouseEvent) {
  const svg = event.currentTarget as SVGSVGElement | null
  if (!svg || !usageTrendChartData.value.length) {
    chartHoverIndex.value = null
    return
  }
  const rect = svg.getBoundingClientRect()
  if (!rect.width) return
  const x = ((event.clientX - rect.left) / rect.width) * 1000
  let best = 0
  let bestDist = Number.POSITIVE_INFINITY
  usageTrendLabelPoints.value.forEach((point, index) => {
    const dist = Math.abs(point.x - x)
    if (dist < bestDist) {
      bestDist = dist
      best = index
    }
  })
  chartHoverIndex.value = best
  updateUsageChartTooltipPosition(best)
}

function updateUsageChartTooltipPosition(index: number) {
  const shell = usageChartShellRef.value
  if (!shell || !usageTrendLabelPoints.value[index]) {
    return
  }
  const shellRect = shell.getBoundingClientRect()
  if (!shellRect.width) return
  // SVG plot x mapped to shell width (viewBox 0..1000)
  const pointX = usageTrendLabelPoints.value[index].x
  const rawLeft = (pointX / 1000) * shellRect.width
  const tooltipEl = usageChartTooltipRef.value
  const tooltipWidth = tooltipEl?.offsetWidth || 160
  const pad = 12
  const minLeft = pad + tooltipWidth / 2
  const maxLeft = shellRect.width - pad - tooltipWidth / 2
  const clamped = Math.min(Math.max(rawLeft, minLeft), Math.max(minLeft, maxLeft))
  usageChartTooltipStyle.value = {
    left: `${clamped}px`,
    top: '8px',
    transform: 'translateX(-50%)',
  }
}

function positionUsageProviderPicker() {
  const trigger = usageProviderPickerTriggerRef.value
  if (!trigger) return
  const rect = trigger.getBoundingClientRect()
  const viewportW = window.innerWidth
  const viewportH = window.innerHeight
  const width = Math.min(320, Math.max(260, Math.min(rect.width + 120, viewportW - 24)))
  const gap = 6
  const pad = 12
  // 优先与 trigger 左对齐；右侧溢出再左移，避免整块飞到视口最左
  let left = rect.left
  if (left + width > viewportW - pad) left = viewportW - width - pad
  if (left < pad) left = pad
  const spaceBelow = viewportH - rect.bottom - pad
  const spaceAbove = rect.top - pad
  const preferBelow = spaceBelow >= 220 || spaceBelow >= spaceAbove
  const maxHeight = Math.max(180, Math.min(380, preferBelow ? spaceBelow - gap : spaceAbove - gap))
  if (preferBelow) {
    usageProviderPickerStyle.value = {
      position: 'fixed',
      top: `${Math.round(rect.bottom + gap)}px`,
      left: `${Math.round(left)}px`,
      width: `${Math.round(width)}px`,
      maxHeight: `${Math.round(maxHeight)}px`,
      zIndex: '10050',
    }
  } else {
    usageProviderPickerStyle.value = {
      position: 'fixed',
      top: 'auto',
      bottom: `${Math.round(viewportH - rect.top + gap)}px`,
      left: `${Math.round(left)}px`,
      width: `${Math.round(width)}px`,
      maxHeight: `${Math.round(maxHeight)}px`,
      zIndex: '10050',
    }
  }
}

function toggleUsageProviderPicker() {
  const next = !usageProviderPickerOpen.value
  usageProviderPickerOpen.value = next
  if (next) {
    // 打开时吞掉随后的全局 pointerdown/click，避免立刻被 closeFloatingMenus 关掉
    suppressFloatingMenuCloseUntil = Date.now() + 280
    nextTick(() => positionUsageProviderPicker())
  }
}

function repositionUsageProviderPickerIfOpen() {
  if (!usageProviderPickerOpen.value) return
  positionUsageProviderPicker()
}

function formatUsageTrendTooltipTime(timestamp: string) {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  const yyyy = date.getFullYear()
  const mm = String(date.getMonth() + 1).padStart(2, '0')
  const dd = String(date.getDate()).padStart(2, '0')
  const hh = String(date.getHours()).padStart(2, '0')
  const mi = String(date.getMinutes()).padStart(2, '0')
  const ss = String(date.getSeconds()).padStart(2, '0')
  const bucket = usageEffectiveBucket.value
  if (bucket === 'minute' || usagePeriodFilter.value === '1h') {
    return `${yyyy}-${mm}-${dd} ${hh}:${mi}:${ss}`
  }
  if (bucket === 'hour' || usagePeriodFilter.value === 'today') {
    return `${yyyy}-${mm}-${dd} ${hh}:${mi}`
  }
  if (bucket === 'month') {
    return `${yyyy}-${mm}`
  }
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}`
}

function clearUsageProviderSelection() {
  usageSelectedProviderIds.value = []
  usageProviderFilter.value = ''
  usageProviderPickerOpen.value = false
  // 清空后立即按「全部」重拉，避免残留空筛选结果
  managedUsageNextCursor.value = null
  managedUsageHasMore.value = false
  scheduleManagedUsageReload()
}

function toggleUsageProviderSelection(id: string) {
  const set = new Set(usageSelectedProviderIds.value)
  if (set.has(id)) set.delete(id)
  else set.add(id)
  usageSelectedProviderIds.value = [...set]
  // 选择变更由 watch debounce 重拉；此处不关浮层，方便连续多选
}

function closeUsageProviderPicker() {
  usageProviderPickerOpen.value = false
}

/** 从 Provider 卡片「查看用量」：进入 Usage 并选中该档案（多选筛选，不制造假 tab） */
function openProviderUsageFilter(providerId: string) {
  usageSelectedProviderIds.value = [providerId]
  usageProviderFilter.value = ''
  usageProviderPickerOpen.value = false
  appSection.value = 'usage'
}

function loadMoreUsageRequestLogs() {
  if (!managedUsageHasMore.value || !managedUsageNextCursor.value) return
  if (managedUsageInFlight.value) return
  void loadManagedUsageLive({ append: true })
}

function scheduleManagedUsageReload() {
  if (managedUsageDebounceTimer != null) {
    window.clearTimeout(managedUsageDebounceTimer)
    managedUsageDebounceTimer = null
  }
  managedUsageDebounceTimer = window.setTimeout(() => {
    managedUsageDebounceTimer = null
    void loadManagedUsageLive({ reset: true })
  }, MANAGED_USAGE_DEBOUNCE_MS)
}
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
const allWorkflowTemplates = computed(() => [...getSystemWorkflowTemplates(currentLocale.value), ...userWorkflowTemplates.value])
const workflowTemplateFilters = computed<Array<{ id: 'all' | 'system' | 'user'; label: string; count: number; icon: string }>>(() => [
  { id: 'all', label: t('common.all'), count: allWorkflowTemplates.value.length, icon: 'template' },
  { id: 'system', label: t('templates.systemTemplate'), count: allWorkflowTemplates.value.filter((template) => template.kind === 'system').length, icon: 'workspace' },
  { id: 'user', label: t('templates.userTemplate'), count: userWorkflowTemplates.value.length, icon: 'edit' },
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
  { id: 'all', label: tr('全部工作区', 'All workspaces') },
  ...workspaces.value.map((workspace) => ({ id: workspace.id, label: workspace.name })),
])
const hiddenRecentItemCount = computed(() => hiddenRecentItemIds.value.length)
const removedRecentItems = computed(() =>
  allRecentItems.value.filter((item) => recentItemIsHidden(item.id)),
)
const recentWorkspaceFilterLabel = computed(() => recentWorkspaceOptions.value.find((workspace) => workspace.id === recentWorkspaceFilter.value)?.label ?? tr('全部工作区', 'All workspaces'))
const recentFilters = computed<Array<{ id: RecentFilter; label: string; count: number; icon: string }>>(() => {
  const items = allRecentItems.value
  return [
    { id: 'all', label: t('common.all'), count: items.length, icon: 'recent' },
    { id: 'workspace', label: t('search.groups.workspace'), count: items.filter((item) => item.type === 'workspace').length, icon: 'workspace' },
    { id: 'project', label: t('search.groups.project'), count: items.filter((item) => item.type === 'project').length, icon: 'tab' },
    { id: 'session', label: t('search.groups.session'), count: items.filter((item) => item.type === 'session').length, icon: 'terminal' },
    { id: 'command', label: t('search.groups.command'), count: items.filter((item) => item.type === 'command').length, icon: 'bolt' },
    { id: 'snapshot', label: t('search.groups.snapshot'), count: items.filter((item) => item.type === 'snapshot').length, icon: 'copy' },
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
        meta: `${t('workspace.projects', { count: workspace.tabs.length })} · ${t('snapshot.terminals', { count: totalWorkspaceSessions(workspace) })}`,
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
          meta: `${t('snapshot.panes', { count: countLeafPanes(tab.panes) })} · ${t('snapshot.terminals', { count: countPaneSessions(tab.panes) })}`,
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
            title: sessionDisplayName(workspace, pane, session),
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
          badge: entry.tags[0] || t('search.groups.command'),
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
          meta: `${workspace.name} · ${t('workspace.projects', { count: snapshot.tabsState.length })}`,
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
        meta: `${t('workspace.projects', { count: workspace.tabs.length })} · ${workspace.tags.join(' · ') || t('workspace.tagsEmpty')}`,
        actionLabel: t('workspace.openWorkspace'),
      onOpen: () => openWorkspace(workspace.id),
    }, [workspace.name, workspace.rootPath, workspace.description, workspace.tags.join(' ')])

    workspace.tabs.forEach((tab) => {
      pushSearchResult(results, query, {
        id: `project-${workspace.id}-${tab.id}`,
        type: 'project',
        icon: 'tab',
        title: tab.name,
        description: workspace.rootPath,
        meta: `${workspace.name} · ${t('snapshot.terminals', { count: countPaneSessions(tab.panes) })}`,
        actionLabel: tr('定位项目', 'Locate project'),
        onOpen: () => openProjectWorkspace(workspace.id, tab.id),
      }, [workspace.name, workspace.rootPath, tab.name])

      flattenLeafPanes(tab.panes).forEach((pane) => {
        paneSessions(pane).forEach((session) => {
          const entry = workspaceEntryById(workspace, session.terminalEntryId)
          pushSearchResult(results, query, {
            id: `session-${workspace.id}-${tab.id}-${pane.id}-${session.id}`,
            type: 'session',
            icon: 'terminal',
            title: sessionDisplayName(workspace, pane, session),
            description: entry?.workingDirectory || session.pathLabel || pane.pathLabel,
            meta: `${workspace.name} / ${tab.name}`,
            actionLabel: tr('打开终端', 'Open terminal'),
            onOpen: () => openWorkspaceTerminalSession(workspace.id, tab.id, pane.id, session.id),
          }, sessionDisplayTitle(workspace, tab, pane, session))
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
        actionLabel: tr('查看配置', 'View config'),
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
          actionLabel: t('recent.backToSource'),
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
        meta: `${t('workspace.projects', { count: snapshot.tabsState.length })} · ${relativeTimeLabel(snapshot.updatedAt)}`,
        actionLabel: t('common.actions.restoreSnapshot'),
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
      meta: tr('设置入口', 'Settings entry'),
      actionLabel: tr('打开设置', 'Open settings'),
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
    { key: 'best', type: 'workspace', title: t('search.groups.best'), items: [] },
    { key: 'workspace', type: 'workspace', title: t('search.groups.workspace'), items: [] },
    { key: 'project', type: 'project', title: t('search.groups.project'), items: [] },
    { key: 'session', type: 'session', title: t('search.groups.session'), items: [] },
    { key: 'command', type: 'command', title: t('search.groups.command'), items: [] },
    { key: 'config', type: 'config', title: t('search.groups.config'), items: [] },
    { key: 'snapshot', type: 'snapshot', title: t('search.groups.snapshot'), items: [] },
    { key: 'setting', type: 'setting', title: t('search.groups.setting'), items: [] },
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
    title: t('settings.groups.appearance.title'),
    icon: 'theme',
    badge: t('settings.groups.appearance.badge'),
    description: t('settings.groups.appearance.description'),
    items: [
      t('settings.groups.appearance.currentTheme', { name: themeDisplayName(selectedThemePreset.value) }),
      t('settings.groups.appearance.fontSize', { size: terminalFontSize.value }),
      railCollapsed.value ? t('settings.groups.appearance.railCollapsed') : t('settings.groups.appearance.railExpanded'),
    ],
  },
  {
    id: 'terminal',
    title: t('settings.groups.terminal.title'),
    icon: 'terminal',
    badge: t('settings.groups.terminal.badge'),
    description: t('settings.groups.terminal.description'),
    items: [
      t('settings.groups.terminal.defaultShell'),
      t('settings.groups.terminal.restoreStrategy', { label: restoreCommandStrategyLabel(restoreCommandStrategy.value) }),
      t('settings.groups.terminal.autoExecute'),
    ],
  },
  {
    id: 'system',
    title: t('settings.groups.system.title'),
    icon: 'runtime',
    badge: t('settings.groups.system.badge'),
    description: t('settings.groups.system.description'),
    items: [
      t('settings.groups.system.refresh', { label: systemRefreshLabel.value }),
      t('settings.groups.system.envVisible', { count: visibleEnvironmentChecks.value.length }),
      hasEnvironmentCheckCache.value ? t('settings.groups.system.envCacheLoaded') : t('settings.groups.system.envCachePending'),
    ],
  },
  {
    id: 'notifications',
    title: t('settings.groups.notifications.title'),
    icon: 'recent',
    badge: t('settings.groups.notifications.badge'),
    description: t('settings.groups.notifications.description'),
    items: [
      t('settings.groups.notifications.attentionSessions', { count: countAttentionSessions() }),
      systemNotificationsEnabled.value ? t('settings.groups.notifications.notificationsEnabled') : t('settings.groups.notifications.notificationsDisabled'),
      windowAttentionEnabled.value ? t('settings.groups.notifications.taskbarEnabled') : t('settings.groups.notifications.taskbarDisabled'),
    ],
  },
  {
    id: 'supervisor',
    title: t('settings.groups.supervisor.title'),
    icon: 'bolt',
    badge: t('settings.groups.supervisor.badge'),
    description: t('settings.groups.supervisor.description'),
    items: [
      t('settings.groups.supervisor.stalledThreshold', { minutes: Math.round(SUPERVISOR_STALLED_THRESHOLD_MS / 60000) }),
      t('settings.groups.supervisor.sharedState'),
      isDevBuild ? t('settings.groups.supervisor.debugAvailable') : t('settings.groups.supervisor.debugHidden'),
    ],
  },
  {
    id: 'data',
    title: t('settings.groups.data.title'),
    icon: 'copy',
    badge: t('settings.groups.data.badge'),
    description: t('settings.groups.data.description'),
    items: [
      t('settings.groups.data.workspacePersistence'),
      t('settings.groups.data.importPreview'),
      t('settings.groups.data.exportSafe'),
    ],
  },
])
const currentLocale = computed(() => locale.value as AppLocale)
function tr(zh: string, en: string) {
  return currentLocale.value === 'zh-CN' ? zh : en
}
function themeDisplayName(theme: ThemePreset) {
  if (theme.id === 'default') return tr('明亮默认', 'Bright Default')
  if (theme.id === 'orange') return tr('明亮橙', 'Bright Orange')
  if (theme.id === 'blue') return tr('明亮蓝', 'Bright Blue')
  if (theme.id === 'purple') return tr('明亮紫', 'Bright Purple')
  if (theme.id === 'pink') return tr('明亮粉', 'Bright Pink')
  if (theme.id === 'dark-default') return tr('暗色经典', 'Dark Classic')
  return theme.name
}
function themeDisplayKind(theme: ThemePreset) {
  if (theme.kind === '系统主题') return tr('系统主题', 'System Theme')
  if (theme.kind === '常见主题') return tr('常见主题', 'Common Theme')
  return theme.kind
}
function themeDisplayDescription(theme: ThemePreset) {
  if (theme.id === 'default') return tr('温和中性色基底，适合作为默认工作主题。', 'A calm neutral baseline that works well as the default work theme.')
  if (theme.id === 'orange') return tr('暖白底配橙色强调，信息清晰而不刺眼。', 'Warm white surfaces with orange emphasis for clear but comfortable scanning.')
  if (theme.id === 'blue') return tr('冷白底配蓝色强调，更接近专业桌面工具。', 'Cool white surfaces with blue emphasis, closer to a professional desktop tool feel.')
  if (theme.id === 'purple') return tr('柔和紫色强调，保留一点个性但整体仍偏清爽。', 'A soft purple accent that keeps some personality while staying clean overall.')
  if (theme.id === 'pink') return tr('雾粉强调，整体更轻但保持足够对比。', 'A misty pink accent that feels lighter while still preserving enough contrast.')
  if (theme.id === 'dark-default') return tr('面向长时间终端工作的低亮度主题，和浅色主题共用同一套组件语义。', 'A low-luminance theme for long terminal sessions, sharing the same component semantics as the light themes.')
  if (theme.id === 'ayu-mirage') return tr('保留一个成熟暗色主题作为可选项。', 'A mature dark theme preserved as an optional preset.')
  if (theme.id === 'arc-dark') return tr('更偏桌面工具的中性暗色。', 'A neutral dark palette that leans more toward desktop tooling.')
  if (theme.id === 'catppuccin-latte') return tr('较柔和的浅色可选风格，适合白天工作。', 'A softer light optional palette that fits daytime work well.')
  return theme.description
}
function localizeBuiltInWorkspaces(current: WorkspaceCard[]) {
  return current.map((workspace) => {
    if (workspace.id === 'demo-workspace' || workspace.id === 'memos') {
      return {
        ...workspace,
        description: tr('本地项目工作区，包含前端、后端、AI 协作与临时终端。', 'Local project workspace with frontend, backend, AI collaboration, and temporary terminal sessions.'),
        tags: [tr('前端', 'Frontend'), tr('后端', 'Backend'), 'AI'],
        providerProfiles: (workspace.providerProfiles ?? []).map((profile) => {
          if (profile.id === 'provider-openai-main' || profile.id === 'memos-provider-openai-main') {
            return {
              ...profile,
              authSource: tr('Codex CLI OAuth 登录态', 'Codex CLI OAuth session'),
              note: tr('从本机 Codex CLI 配置读取，当前作为默认 Codex 档案。', 'Read from the local Codex CLI config and currently used as the default Codex profile.'),
            }
          }
          if (profile.id === 'provider-claude-team' || profile.id === 'memos-provider-claude-team') {
            return {
              ...profile,
              authSource: tr('Claude Code 本地登录态', 'Claude Code local sign-in state'),
              note: tr('读取 Claude Code 本地配置，切换时不写入终端运行配置。', 'Reads the local Claude Code config and does not inject it into terminal run configs when switching.'),
            }
          }
          if (profile.id === 'provider-gemini-personal' || profile.id === 'memos-provider-gemini-personal') {
            return {
              ...profile,
              authSource: tr('Google OAuth 登录态', 'Google OAuth session'),
              note: tr('Gemini CLI 本地登录档案，用于快速切换账号与默认模型。', 'Gemini CLI local profile for switching accounts and default models quickly.'),
            }
          }
          return profile
        }),
        terminalEntries: (workspace.terminalEntries ?? []).map((entry) => {
          if (entry.id === 'demo-frontend' || entry.id === 'memos-frontend') {
            return {
              ...entry,
              name: tr('前端', 'Frontend'),
              tags: [tr('前端', 'Frontend'), tr('常用', 'Common')],
              note: tr('前端本地开发服务', 'Frontend local development service'),
            }
          }
          if (entry.id === 'demo-backend' || entry.id === 'memos-backend') {
            return {
              ...entry,
              name: tr('后端', 'Backend'),
              tags: [tr('后端', 'Backend'), tr('常用', 'Common')],
              note: tr('后端调试入口', 'Backend debugging entry'),
            }
          }
          if (entry.id === 'demo-codex' || entry.id === 'memos-codex') {
            return {
              ...entry,
              tags: ['AI'],
              note: tr('AI 协作终端条目', 'AI collaboration terminal entry'),
            }
          }
          if (entry.id === 'demo-shell-a' || entry.id === 'demo-shell-b' || entry.id === 'memos-shell-a' || entry.id === 'memos-shell-b') {
            return {
              ...entry,
              tags: [tr('临时', 'Temporary')],
            }
          }
          return entry
        }),
        tabs: (workspace.tabs ?? []).map((tab) => {
          if (tab.id === 'demo-dev' || tab.id === 'memos-dev') {
            return {
              ...tab,
              name: tr('开发', 'Development'),
              panes: tab.panes.map((pane) => {
                if (pane.id === 'pane-frontend' || pane.id === 'memos-pane-frontend') return { ...pane, name: tr('前端', 'Frontend') }
                if (pane.id === 'pane-backend' || pane.id === 'memos-pane-backend') return { ...pane, name: tr('后端', 'Backend') }
                return pane
              }),
            }
          }
          if (tab.id === 'demo-ai' || tab.id === 'memos-ai') {
            return {
              ...tab,
              name: tr('AI 协作', 'AI collaboration'),
            }
          }
          if (tab.id === 'demo-temp' || tab.id === 'memos-temp') {
            return {
              ...tab,
              name: tr('临时操作', 'Scratchpad'),
            }
          }
          return tab
        }),
      }
    }

    if (workspace.id === 'demo-api-suite' || workspace.id === 'live') {
      return {
        ...workspace,
        description: tr('较轻量的前后端联调工作区。', 'A lighter workspace for frontend/backend integration work.'),
        tags: [tr('联调', 'Integration')],
        terminalEntries: (workspace.terminalEntries ?? []).map((entry) => {
          if (entry.id === 'suite-web' || entry.id === 'live-web') {
            return {
              ...entry,
              tags: [tr('前端', 'Frontend')],
            }
          }
          if (entry.id === 'suite-api' || entry.id === 'live-api') {
            return {
              ...entry,
              tags: [tr('后端', 'Backend')],
            }
          }
          return entry
        }),
        tabs: (workspace.tabs ?? []).map((tab) => {
          if (tab.id === 'suite-dev' || tab.id === 'live-dev' || tab.id === 'suite-main') {
            return {
              ...tab,
              name: tr('开发', 'Development'),
            }
          }
          if (tab.id === 'suite-note' || tab.id === 'live-note') {
            return {
              ...tab,
              name: tr('临时', 'Temporary'),
            }
          }
          return tab
        }),
      }
    }

    return workspace
  })
}
function environmentItemValueLabel(value: string) {
  if (value === '待检测') return tr('待检测', 'Pending')
  if (value === '检测中') return tr('检测中', 'Checking')
  if (value === '检测失败') return tr('检测失败', 'Check failed')
  if (value === '浏览器模式') return tr('浏览器模式', 'Browser mode')
  if (value === '读取失败') return tr('读取失败', 'Read failed')
  if (value === '项目依赖') return tr('项目依赖', 'Project dependency')
  if (value === '已接入') return tr('已接入', 'Integrated')
  return value
}
function environmentItemNoteLabel(note: string) {
  if (note === '前端 UI 框架') return tr('前端 UI 框架', 'Frontend UI framework')
  if (note === '类型系统') return tr('类型系统', 'Type system')
  if (note === '桌面壳运行时') return tr('桌面壳运行时', 'Desktop shell runtime')
  if (note === '真实终端渲染') return tr('真实终端渲染', 'Real terminal rendering')
  return note
}
function systemStatusDisplayValue(value: string) {
  if (value === '检测中') return tr('检测中', 'Checking')
  if (value === '浏览器模式') return tr('浏览器模式', 'Browser mode')
  if (value === '读取失败') return tr('读取失败', 'Read failed')
  return value
}
const systemRefreshLabel = computed(() => {
  if (systemStatusRefreshing.value) return t('refresh.refreshing')
  if (systemRefreshInterval.value === 'manual') return t('refresh.manual')
  if (systemRefreshCountdown.value > 0) return t('refresh.autoAfter', { seconds: systemRefreshCountdown.value })
  return t('refresh.autoInterval', { interval: systemRefreshInterval.value })
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
    label: t('rail.primary.home.label'),
    icon: 'home',
    summary: t('rail.primary.home.summary'),
    active: appSection.value === 'home',
    action: () => {
      appSection.value = 'home'
      workspaceView.value = 'overview'
    },
  },
  {
    id: 'workspace',
    label: t('rail.primary.workspace.label'),
    icon: 'workspace',
    summary: t('rail.primary.workspace.summary'),
    active: appSection.value === 'workspace',
    action: () => goWorkspaceOverview(),
  },
  {
    id: 'recent',
    label: t('rail.primary.recent.label'),
    icon: 'recent',
    summary: t('rail.primary.recent.summary'),
    active: appSection.value === 'recent',
    action: () => { appSection.value = 'recent' },
  },
  {
    id: 'templates',
    label: t('rail.primary.templates.label'),
    icon: 'template',
    summary: t('rail.primary.templates.summary'),
    active: appSection.value === 'templates',
    action: () => { appSection.value = 'templates' },
  },
  {
    id: 'providers',
    label: t('rail.primary.providers.label'),
    icon: 'package',
    summary: t('rail.primary.providers.summary'),
    active: appSection.value === 'providers',
    action: () => { appSection.value = 'providers' },
  },
  {
    id: 'usage',
    label: t('rail.primary.usage.label'),
    icon: 'bar-chart',
    summary: t('rail.primary.usage.summary'),
    active: appSection.value === 'usage',
    action: () => { appSection.value = 'usage' },
  },
])
const secondaryRailItems = computed<RailItem[]>(() => [
  {
    id: 'search',
    label: t('rail.secondary.search.label'),
    icon: 'search',
    summary: t('rail.secondary.search.summary'),
    active: appSection.value === 'search',
    action: () => openSearchPage(),
  },
  {
    id: 'theme',
    label: t('rail.secondary.theme.label'),
    icon: 'theme',
    summary: t('rail.secondary.theme.summary'),
    active: openThemeModal.value,
    action: () => openThemePanel('theme'),
  },
  {
    id: 'settings',
    label: t('rail.secondary.settings.label'),
    icon: 'settings',
    summary: t('rail.secondary.settings.summary'),
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

watch(currentLocale, () => {
  commitWorkspaces((current) => localizeBuiltInWorkspaces(current), 'transient')
}, { immediate: true })

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

watch(() => providerForm.providerKind, (kind) => {
  if (!openProviderEditorModal.value) return
  if (!providerForm.configPath.trim()) {
    providerForm.configPath = defaultConfigPathForProvider(kind)
  }
  if (!providerForm.toolTargetsText.trim()) {
    providerForm.toolTargetsText = defaultProviderTargetsText(kind)
  }
  if (!providerForm.color.trim()) {
    providerForm.color = providerKindColor(kind)
  }
  if (!providerForm.authSource.trim()) {
    providerForm.authSource = defaultAuthSourceForProvider(kind, providerForm.managedBy)
  }
})

watch(() => providerForm.managedBy, (source) => {
  if (!openProviderEditorModal.value || providerForm.authSource.trim()) return
  providerForm.authSource = defaultAuthSourceForProvider(providerForm.providerKind, source)
})

watch(filteredWorkspaceProviders, (providers) => {
  if (!providers.length) {
    activeProviderStatsId.value = ''
    return
  }
  if (!providers.some((provider) => provider.id === activeProviderStatsId.value)) {
    activeProviderStatsId.value = providers[0].id
  }
})

/**
 * - Usage 页：拉实时数据 + 60s 轮询兜底（唯一 timer）；事件 usage-log-recorded 优先 invalidate
 * - Providers 页：静默同步一次 + 按当前周期拉 Usage（卡片副标），不轮询
 * - 筛选变化：debounce 后 reset 明细 cursor 并刷新聚合
 * - 其它页 / 隐藏：停轮询
 */
watch(
  [appSection, usagePeriodFilter, usageAppFilter, usageProviderFilter, usageSelectedProviderIds, usageCustomStartAt, usageCustomEndAt],
  ([section], previous) => {
    if (!("__TAURI_INTERNALS__" in window)) {
      stopManagedUsagePolling()
      return
    }

    const prevSection = previous?.[0]
    const filtersChanged = Boolean(previous) && (
      previous![1] !== usagePeriodFilter.value
      || previous![2] !== usageAppFilter.value
      || previous![3] !== usageProviderFilter.value
      || previous![4] !== usageSelectedProviderIds.value
      || previous![5] !== usageCustomStartAt.value
      || previous![6] !== usageCustomEndAt.value
    )

    if (section === 'usage') {
      if (filtersChanged || prevSection !== 'usage') {
        managedUsageNextCursor.value = null
        managedUsageHasMore.value = false
        scheduleManagedUsageReload()
      }
      startManagedUsagePolling()
      return
    }

    stopManagedUsagePolling()
    if (section === 'providers') {
      if (!providerSilentSyncedOnce.value && !providerDetectionRunning.value) {
        void syncNativeProviderProfiles({ silent: true })
      }
      void loadManagedUsageLive({ reset: true })
    }
  },
  { immediate: false, deep: true },
)

watch(usageProviderPickerOpen, (open) => {
  if (!open) return
  nextTick(() => positionUsageProviderPicker())
})

watch(chartHoverIndex, (index) => {
  if (index == null) return
  nextTick(() => updateUsageChartTooltipPosition(index))
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

function closeFloatingMenus(event?: Event) {
  // Teleport 到 body 的 Usage Provider 浮层：点击其内部时不要关
  if (usageProviderPickerOpen.value && event?.target instanceof Node) {
    const target = event.target as Node
    const panel = document.querySelector('.usage-provider-picker__panel--fixed')
    const trigger = usageProviderPickerTriggerRef.value
    if ((panel && panel.contains(target)) || (trigger && trigger.contains(target))) {
      return
    }
  }
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
  openProviderKindMenu.value = false
  openProviderSourceMenu.value = false
  openProviderScopeMenu.value = false
  activeRailTooltipId.value = null
  usageProviderPickerOpen.value = false
}

function handleGlobalClick(event: MouseEvent) {
  if (Date.now() < suppressFloatingMenuCloseUntil) return
  closeFloatingMenus(event)
}

function handleGlobalPointerDown(event: PointerEvent) {
  if (Date.now() < suppressFloatingMenuCloseUntil) return
  closeFloatingMenus(event)
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

  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'b' && !event.altKey && immersiveWorkbenchActive.value) {
    event.preventDefault()
    toggleWorkbenchExplorerCollapsed()
    return
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
  window.addEventListener('pointermove', handleAiAssistPointerMove)
  window.addEventListener('pointerup', endAiAssistDrag)
  window.addEventListener('pointercancel', endAiAssistDrag)
  window.addEventListener('keydown', handleGlobalKeydown)
  window.addEventListener('resize', refreshCachedDropZones)
  window.addEventListener('resize', clampAiAssistCardIntoViewport)
  window.addEventListener('resize', repositionUsageProviderPickerIfOpen)
  window.addEventListener('scroll', repositionUsageProviderPickerIfOpen, true)
  document.addEventListener('visibilitychange', handleVisibilityChange)
  if ('__TAURI_INTERNALS__' in window) {
    void setupManagedUsageEventListeners()
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
  window.removeEventListener('pointermove', handleAiAssistPointerMove)
  window.removeEventListener('pointerup', endAiAssistDrag)
  window.removeEventListener('pointercancel', endAiAssistDrag)
  window.removeEventListener('keydown', handleGlobalKeydown)
  window.removeEventListener('resize', refreshCachedDropZones)
  window.removeEventListener('resize', clampAiAssistCardIntoViewport)
  window.removeEventListener('resize', repositionUsageProviderPickerIfOpen)
  window.removeEventListener('scroll', repositionUsageProviderPickerIfOpen, true)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  document.body.classList.remove('is-dragging-ai-assist')
  unlistenWindowResize?.()
  unlistenWindowResize = null
  teardownManagedUsageEventListeners()
  stopManagedUsagePolling()
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
  stopManagedUsagePolling()
  if (managedUsageDebounceTimer != null) {
    window.clearTimeout(managedUsageDebounceTimer)
    managedUsageDebounceTimer = null
  }
  dragPointerCleanup?.()
  cleanupSplitResizeListeners()
  workbenchResizeCleanup?.()
})

function handleVisibilityChange() {
  if (typeof document === 'undefined') return
  windowVisible.value = !document.hidden
  scheduleSystemRefresh()
  startSupervisorScan()
  if (windowVisible.value) {
    void refreshSystemStatus()
    if (appSection.value === 'usage') {
      void loadManagedUsageLive({ reset: true })
      startManagedUsagePolling()
    }
  } else {
    stopManagedUsagePolling()
  }
}

const activeRuntimeTab = computed(() => {
  if (!selectedWorkspace.value) return undefined
  return selectedWorkspace.value.tabs.find((tab) => tab.id === activeRuntimeTabId.value) ?? selectedWorkspace.value.tabs[0]
})

const workbenchShellClasses = computed(() => ({
  'workbench-shell--immersive': immersiveWorkbenchActive.value,
  'workbench-shell--explorer-collapsed': workbenchExplorerCollapsed.value,
}))

const workbenchShellStyle = computed(() => ({
  '--workbench-sidebar-width': `${immersiveWorkbenchActive.value ? Math.min(workbenchSidebarWidth.value, 212) : workbenchSidebarWidth.value}px`,
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
  syncRuntimeEntryStatusesForCurrentWorkspaces()
  queueSessionAttentionNotifications()
  void applyWindowAttentionBadge()
}, { deep: true, flush: 'post', immediate: true })

watch(runtimeSessionOverlays, () => {
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

watch([restoreCommandStrategy, railCollapsed, workbenchImmersive, workbenchExplorerCollapsed, systemNotificationsEnabled, windowAttentionEnabled, pinnedRecentItemIds, hiddenRecentItemIds], () => {
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

const activeThemeLabel = computed(() => `${themeDisplayName(appliedTheme.value)} · ${themeDisplayKind(appliedTheme.value)}`)
const activeThemeDescription = computed(() => themeDisplayDescription(appliedTheme.value))
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
    { key: 'home-root', label: t('pages.home'), icon: 'home', onClick: goHome },
  ]

  if (appSection.value === 'home') {
    return items
  }

  if (appSection.value !== 'workspace') {
    items.push({
      key: appSection.value,
      label: placeholderTitle.value,
      icon: appSection.value === 'recent'
        ? 'recent'
        : appSection.value === 'templates'
          ? 'template'
          : appSection.value === 'search'
            ? 'search'
            : appSection.value === 'providers'
              ? 'settings'
              : appSection.value === 'usage'
                ? 'recent'
                : 'settings',
    })
    return items
  }

  items.push({ key: 'workspace-root', label: t('rail.primary.workspace.label'), icon: 'workspace', onClick: goWorkspaceOverview })

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
  if (appSection.value === 'home') return t('pages.home')
  if (appSection.value === 'recent') return t('pages.recent')
  if (appSection.value === 'templates') return t('pages.templates')
  if (appSection.value === 'providers') return t('pages.providers')
  if (appSection.value === 'usage') return t('pages.usage')
  if (appSection.value === 'search') return t('pages.search')
  if (appSection.value === 'settings') return t('pages.settings')
  return t('pages.placeholder')
})

const placeholderDescription = computed(() => {
  if (appSection.value === 'recent') return t('pages.recentDesc')
  if (appSection.value === 'templates') return t('pages.templatesDesc')
  if (appSection.value === 'providers') return t('pages.providersDesc')
  if (appSection.value === 'usage') return t('pages.usageDesc')
  if (appSection.value === 'search') return t('pages.searchDesc')
  if (appSection.value === 'settings') return t('pages.settingsDesc')
  return t('pages.defaultDesc')
})

function paneCommandPreview(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) return tr('未绑定运行配置', 'Run config not bound')
  const command = mode === 'default' ? entry.defaultCommand?.trim() : entry.lastCommand?.trim()
  if (!command) return mode === 'default' ? tr('未设置默认命令', 'Default command not set') : tr('还没有最后命令', 'No last command yet')
  return command.length > 72 ? `${command.slice(0, 72)}…` : command
}

function paneMenuItems(pane: PaneNode): PopoverItem[] {
  const items: PopoverItem[] = [
    {
      label: tr('重命名 Pane', 'Rename pane'),
      icon: 'edit',
      description: tr('修改当前 Pane 分组名称与其组内标签前缀。', 'Rename the current pane group and the tab prefix inside it.'),
      shortcut: 'F2',
      onClick: () => {
        activePaneMenu.value = null
        openPaneRenameModal(pane.id)
      },
    },
    {
      label: tr('选择运行配置', 'Choose run config'),
      icon: 'folder',
      description: tr('将当前 Pane 绑定到一个已有运行配置。', 'Bind the current pane to an existing run config.'),
      shortcut: 'Alt+T',
      onClick: () => {
        activePaneBindingMenuPosition.value = activePaneMenuPosition.value
        activePaneMenu.value = null
        activePaneBindingMenu.value = pane.id
      },
    },
    {
      label: t('common.actions.openDirectory'),
      icon: 'folder',
      description: tr('打开当前终端对应的工作目录。', 'Open the working directory used by the current terminal.'),
      shortcut: 'Alt+O',
      onClick: () => {
        activePaneMenu.value = null
        openPaneDirectory(pane)
      },
    },
    {
      label: t('common.actions.copyPath'),
      icon: 'copy',
      description: tr('复制当前 Pane 使用的工作目录路径。', 'Copy the working directory path used by the current pane.'),
      shortcut: 'Alt+C',
      onClick: () => {
        activePaneMenu.value = null
        copyPanePath(pane)
      },
    },
    {
      label: tr('复制默认命令', 'Copy default command'),
      icon: 'copy',
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneMenu.value = null
        copyPaneCommand(pane, 'default')
      },
    },
    {
      label: tr('插入默认命令', 'Insert default command'),
      icon: 'terminal',
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneMenu.value = null
        insertPaneCommand(pane, 'default')
      },
    },
    {
      label: tr('复制最后命令', 'Copy last command'),
      icon: 'copy',
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneMenu.value = null
        copyPaneCommand(pane, 'last')
      },
    },
    {
      label: tr('插入最后命令', 'Insert last command'),
      icon: 'terminal',
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneMenu.value = null
        insertPaneCommand(pane, 'last')
      },
    },
    {
      label: t('common.actions.splitRight'),
      icon: 'pane',
      description: tr('在当前 Pane 右侧拆出一个新的分组。', 'Create a new group to the right of the current pane.'),
      shortcut: 'Alt+→',
      onClick: () => {
        activePaneMenu.value = null
        splitLeafPane(pane.id, 'horizontal')
      },
    },
    {
      label: tr('拆分到下方', 'Split below'),
      icon: 'pane',
      description: tr('在当前 Pane 下方拆出一个新的分组。', 'Create a new group below the current pane.'),
      shortcut: 'Alt+↓',
      onClick: () => {
        activePaneMenu.value = null
        splitLeafPane(pane.id, 'vertical')
      },
    },
  ]

  if (pane.terminalEntryId) {
    items.push({
      label: tr('解除配置绑定', 'Clear config binding'),
      icon: 'close',
      description: tr('将当前 Pane 恢复为独立空白终端。', 'Return the current pane to an independent blank terminal.'),
      shortcut: 'Alt+U',
      onClick: () => {
        activePaneMenu.value = null
        assignEntryToPane(pane.id, null)
      },
    })
  }

  items.push({
    label: tr('删除 Pane', 'Delete pane'),
    icon: 'trash',
    description: tr('删除当前 Pane 分组及其中的所有终端。', 'Delete the current pane group and every terminal inside it.'),
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
      label: tr('重命名终端', 'Rename terminal'),
      icon: 'edit',
      description: tr('修改当前终端标签名称。', 'Rename the current terminal tab.'),
      shortcut: 'F2',
      onClick: () => {
        activePaneSessionMenu.value = null
        renameTarget.kind = 'session'
        renameTarget.id = session.id
        renameTarget.title = tr('重命名终端', 'Rename terminal')
        renameTarget.placeholder = tr('例如：PowerShell 7 / 测试终端 / 构建命令', 'Examples: PowerShell 7 / Test terminal / Build command')
        renameTarget.value = session.name
        openRenameModal.value = true
      },
    },
    {
      label: tr('选择运行配置', 'Choose run config'),
      icon: 'folder',
      description: tr('将当前 Pane 绑定到一个已有运行配置。', 'Bind the current pane to an existing run config.'),
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
      label: supervisorEnabled ? tr('关闭任务监督', 'Disable task supervision') : tr('开启任务监督', 'Enable task supervision'),
      icon: supervisorEnabled ? 'close' : 'recent',
      description: supervisorEnabled
        ? tr('停止对当前终端任务的完成、停滞和待输入状态进行提醒。', 'Stop reminders for completion, stalls, and input-needed states in the current terminal task.')
        : tr('监控当前终端任务，长时间无输出或命令完成时给出提醒。', 'Monitor the current terminal task and alert when output stalls or a command finishes.'),
      onClick: () => {
        activePaneSessionMenu.value = null
        setSessionSupervisorMode(session.id, supervisorEnabled ? 'off' : 'watch')
      },
    },
    {
      label: t('common.actions.openDirectory'),
      icon: 'folder',
      description: tr('打开当前终端对应的工作目录。', 'Open the working directory used by the current terminal.'),
      shortcut: 'Alt+O',
      onClick: () => {
        activePaneSessionMenu.value = null
        openPaneDirectory(pane)
      },
    },
    {
      label: t('common.actions.copyPath'),
      icon: 'copy',
      description: tr('复制当前 Pane 使用的工作目录路径。', 'Copy the working directory path used by the current pane.'),
      shortcut: 'Alt+C',
      onClick: () => {
        activePaneSessionMenu.value = null
        copyPanePath(pane)
      },
    },
  )

  if (pane.terminalEntryId) {
    items.push({
      label: tr('解除配置绑定', 'Clear config binding'),
      icon: 'close',
      description: tr('将当前 Pane 恢复为独立空白终端。', 'Return the current pane to an independent blank terminal.'),
      shortcut: 'Alt+U',
      onClick: () => {
        activePaneSessionMenu.value = null
        assignEntryToPane(pane.id, null)
      },
    })
  }

  items.push({
    label: tr('关闭终端', 'Close terminal'),
    icon: 'trash',
    description: tr('仅关闭当前终端标签，不删除整个 Pane 分组。', 'Close only the current terminal tab without deleting the whole pane group.'),
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
      label: t('common.actions.newTerminal'),
      description: tr('在当前 Pane 分组中新建一个终端标签。', 'Create a new terminal tab in the current pane group.'),
      onClick: () => {
        activePaneHeaderMenu.value = null
        createPaneSession(pane.id)
      },
    },
    {
      label: tr('插入默认命令', 'Insert default command'),
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneHeaderMenu.value = null
        insertPaneCommand(pane, 'default')
      },
    },
    {
      label: tr('复制默认命令', 'Copy default command'),
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'default')}`,
      onClick: () => {
        activePaneHeaderMenu.value = null
        copyPaneCommand(pane, 'default')
      },
    },
    {
      label: tr('复制最后命令', 'Copy last command'),
      description: `${tr('预览', 'Preview')}: ${paneCommandPreview(pane, 'last')}`,
      onClick: () => {
        activePaneHeaderMenu.value = null
        copyPaneCommand(pane, 'last')
      },
    },
    {
      label: t('common.actions.splitRight'),
      description: tr('在当前 Pane 右侧拆出一个新的分组。', 'Create a new group to the right of the current pane.'),
      onClick: () => {
        activePaneHeaderMenu.value = null
        splitLeafPane(pane.id, 'horizontal')
      },
    },
    {
      label: tr('拆分到下方', 'Split below'),
      description: tr('在当前 Pane 下方拆出一个新的分组。', 'Create a new group below the current pane.'),
      onClick: () => {
        activePaneHeaderMenu.value = null
        splitLeafPane(pane.id, 'vertical')
      },
    },
  ]
}

function workspaceMenuItems(workspace: WorkspaceCard): PopoverItem[] {
  const items: PopoverItem[] = [
    {
      label: tr('切换到此工作区', 'Switch to this workspace'),
      description: tr('在右侧终端画布中打开这个工作区', 'Open this workspace in the terminal canvas on the right.'),
      onClick: () => {
        activeWorkspaceMenu.value = null
        switchOpenedWorkspace(workspace.id)
      },
    },
    {
      label: t('common.actions.runConfigs'),
      description: tr('管理该工作区下可复用的终端模板', 'Manage reusable terminal templates in this workspace.'),
      onClick: () => {
        activeWorkspaceMenu.value = null
        openWorkspaceTerminalEntries(workspace.id)
      },
    },
    {
      label: t('workspace.editWorkspace'),
      description: tr('修改名称、路径、描述与标签', 'Edit the name, path, description, and tags.'),
      onClick: () => {
        activeWorkspaceMenu.value = null
        openWorkspaceEditModal(workspace.id)
      },
    },
  ]

  if (openedWorkspaces.value.length > 1) {
    items.push({
      label: tr('从常用区移除', 'Remove from favorites'),
      description: tr('仅移出左侧常用工作区列表，不删除数据', 'Remove it from the left favorite workspace list only without deleting any data.'),
      onClick: () => {
        activeWorkspaceMenu.value = null
        closeOpenedWorkspace(workspace.id)
      },
    })
  }

  items.push({
    label: t('workspace.removeWorkspace'),
    description: tr('删除该工作区及其项目、终端与配置模板', 'Delete this workspace together with its projects, terminals, and config templates.'),
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
      label: tr('未绑定配置', 'No config bound'),
      description: tr('保持当前 Pane 为空白独立终端', 'Keep the current pane as a blank standalone terminal.'),
      active: currentEntryId === '',
      onClick: () => assignEntryToPane(pane.id, null),
    },
  ]

  selectedWorkspaceEntries.value.forEach((entry) => {
    const commandText = entry.defaultCommand ? `${tr('命令', 'Command')}: ${entry.defaultCommand}` : tr('未设置默认命令', 'Default command not set')
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
  renameTarget.title = tr('重命名项目', 'Rename project')
  renameTarget.placeholder = tr('例如：默认标签页 / 直播测试', 'Examples: Default tab / Live test')
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
  renameTarget.title = tr('重命名 Pane', 'Rename pane')
  renameTarget.placeholder = tr('例如：直播 / 终端测试 / PowerShell 7', 'Examples: Live / Terminal test / PowerShell 7')
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
    sessionName: sessionDisplayName(selectedWorkspace.value, sourcePane, sourceSession),
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
    showToast(tr('项目已重命名', 'Project renamed'), tr(`已更新为：${value}`, `Updated to: ${value}`))
  } else if (renameTarget.kind === 'session') {
    renameSession(renameTarget.id, value)
    showToast(tr('终端已重命名', 'Terminal renamed'), tr(`已更新为：${value}`, `Updated to: ${value}`))
  } else {
    renamePane(renameTarget.id, value)
    showToast(tr('Pane 已重命名', 'Pane renamed'), tr(`已更新为：${value}`, `Updated to: ${value}`))
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
  openAiHistoryDrawer.value = false
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

  showToast(
    tr('布局已切换', 'Layout changed'),
    layoutMode === 'grid'
      ? tr('当前标签页已切换为网格布局', 'The current tab was switched to grid layout.')
      : layoutMode === 'horizontal'
        ? tr('当前标签页已切换为横向布局', 'The current tab was switched to horizontal layout.')
        : tr('当前标签页已切换为纵向布局', 'The current tab was switched to vertical layout.'),
  )
}

const launchModeItems = computed<PopoverItem[]>(() => {
  return launchModeOptions.value.map((option) => ({
    label: option.label,
    description: option.description,
    active: terminalEntryForm.launchMode === option.value,
    onClick: () => {
      terminalEntryForm.launchMode = option.value
      openLaunchModeMenu.value = false
    },
  }))
})

const providerKindItems = computed<PopoverItem[]>(() => (
  providerKindOptions.value.map((option) => ({
    label: option.label,
    description: option.description,
    active: providerForm.providerKind === option.value,
    onClick: () => {
      providerForm.providerKind = option.value
      openProviderKindMenu.value = false
    },
  }))
))

const providerSourceItems = computed<PopoverItem[]>(() => (
  providerSourceOptions.value.map((option) => ({
    label: option.label,
    description: option.description,
    active: providerForm.managedBy === option.value,
    onClick: () => {
      providerForm.managedBy = option.value
      openProviderSourceMenu.value = false
    },
  }))
))

const providerScopeItems = computed<PopoverItem[]>(() => (
  providerScopeOptions.value.map((option) => ({
    label: option.label,
    description: option.description,
    active: providerForm.configScope === option.value,
    onClick: () => {
      providerForm.configScope = option.value
      openProviderScopeMenu.value = false
    },
  }))
))

function toggleLaunchModeMenu() {
  const shouldOpen = !openLaunchModeMenu.value
  closeFloatingMenus()
  openLaunchModeMenu.value = shouldOpen
}

function toggleProviderKindMenu() {
  const shouldOpen = !openProviderKindMenu.value
  closeFloatingMenus()
  openProviderKindMenu.value = shouldOpen
}

function toggleProviderSourceMenu() {
  const shouldOpen = !openProviderSourceMenu.value
  closeFloatingMenus()
  openProviderSourceMenu.value = shouldOpen
}

function toggleProviderScopeMenu() {
  const shouldOpen = !openProviderScopeMenu.value
  closeFloatingMenus()
  openProviderScopeMenu.value = shouldOpen
}

function toggleRailCollapsed() {
  railCollapsed.value = !railCollapsed.value
}

function toggleWorkbenchExplorerCollapsed() {
  workbenchExplorerCollapsed.value = !workbenchExplorerCollapsed.value
  workbenchExplorerAutoCollapsed.value = false
}

function toggleWorkbenchImmersive() {
  const next = !workbenchImmersive.value
  workbenchImmersive.value = next

  if (next) {
    if (!workbenchExplorerCollapsed.value) {
      workbenchExplorerCollapsed.value = true
      workbenchExplorerAutoCollapsed.value = true
    }
    // 沉浸模式下 AI 辅助层默认收起，不抢终端主焦点（保留可调用，仅最小化）
    if (aiAssistantPinned.value && !aiAssistantMinimized.value) {
      aiAssistantMinimized.value = true
      immersiveAiAutoMinimized.value = true
    }
    return
  }

  if (workbenchExplorerAutoCollapsed.value) {
    workbenchExplorerCollapsed.value = false
  }
  workbenchExplorerAutoCollapsed.value = false
  // 退出沉浸时还原被自动收起的 AI 辅助层
  if (immersiveAiAutoMinimized.value) {
    aiAssistantMinimized.value = false
    immersiveAiAutoMinimized.value = false
  }
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

const aiAssistCardStyle = computed(() => {
  if (!aiAssistCardPosition.value) return undefined
  return {
    left: `${aiAssistCardPosition.value.left}px`,
    top: `${aiAssistCardPosition.value.top}px`,
    right: 'auto',
    bottom: 'auto',
  }
})

const aiAssistDragState = reactive({
  active: false,
  pointerId: -1,
  pointerOffsetX: 0,
  pointerOffsetY: 0,
})

function aiCliBadgeText(info: AiCliInfo) {
  return aiCliBrandIconSrc(info.kind) ? '' : info.kind === 'generic-ai' ? '' : info.shortLabel
}

function isConfirmedAiCliInfo(info: AiCliInfo) {
  return info.isAi && info.kind !== 'generic-ai'
}

function shouldShowAiSessionStyling(info: AiCliInfo) {
  return info.isAi
}

function hasAiBrandIcon(info: AiCliInfo) {
  return Boolean(aiCliBrandIconSrc(info.kind))
}

function aiCliDisplayName(kind: AiCliKind) {
  if (kind === 'claude-code') return 'Claude'
  if (kind === 'gemini-cli') return 'Gemini'
  if (kind === 'deepseek-cli') return 'DeepSeek'
  if (kind === 'opencode') return 'OpenCode'
  if (kind === 'generic-ai') return 'AI CLI'
  if (kind === 'custom-cli') return 'Terminal'
  return 'Codex'
}

function aiCliBrandIconSrc(kind: AiCliKind) {
  if (kind === 'codex') return openAiBrandIcon
  if (kind === 'claude-code') return claudeBrandIcon
  if (kind === 'gemini-cli') return geminiBrandIcon
  if (kind === 'deepseek-cli') return deepseekBrandIcon
  return null
}

function aiCliBrandIconClass(kind: AiCliKind) {
  if (kind === 'codex') return 'ai-brand-icon--codex'
  if (kind === 'claude-code') return 'ai-brand-icon--claude-code'
  if (kind === 'gemini-cli') return 'ai-brand-icon--gemini-cli'
  if (kind === 'deepseek-cli') return 'ai-brand-icon--deepseek-cli'
  if (kind === 'opencode') return 'ai-brand-icon--opencode'
  return 'ai-brand-icon--generic'
}

function tokenizeShellLikeCommand(command: string) {
  return command.match(/"[^"]*"|'[^']*'|\S+/g) ?? []
}

function matchesAiCliLaunchAlias(token: string, kind: AiCliKind) {
  const normalized = token.trim().toLocaleLowerCase()
  if (kind === 'codex') return normalized === 'codex' || normalized === 'cx'
  if (kind === 'claude-code') return normalized === 'claude' || normalized === 'cc'
  if (kind === 'gemini-cli') return normalized === 'gemini'
  if (kind === 'deepseek-cli') return normalized === 'deepseek'
  if (kind === 'opencode') return normalized === 'opencode'
  return false
}

function isCliOptionToken(token: string) {
  return /^-{1,2}[a-z0-9][\w-]*$/i.test(token) || /^\/[a-z][\w-]*$/i.test(token)
}

function isAiCliLaunchOnlyCommand(command: string, kind: AiCliKind | null | undefined) {
  if (!kind || kind === 'generic-ai' || kind === 'custom-cli') return false
  const tokens = tokenizeShellLikeCommand(command.trim())
  const firstToken = tokens[0]
  if (!firstToken || !matchesAiCliLaunchAlias(firstToken, kind)) return false
  if (tokens.length === 1) return true

  for (let index = 1; index < tokens.length; index += 1) {
    const token = tokens[index]
    const previous = tokens[index - 1]
    if (!token) continue
    if (isCliOptionToken(token)) continue
    if (previous && isCliOptionToken(previous) && !/[<>&|]/.test(token)) continue
    return false
  }

  return true
}

function activeProviderKindForWorkspace(workspace: WorkspaceCard | undefined | null): ProviderKind | null {
  if (!workspace?.providerProfiles?.length) return null
  const active = workspace.providerProfiles.find((profile) => profile.isActive)
    ?? workspace.providerProfiles.find((profile) => profile.isDefault)
  return active?.providerKind ?? null
}

function aiKindFromProviderKind(kind: ProviderKind | null | undefined): AiCliKind | null {
  if (kind === 'codex' || kind === 'claude-code' || kind === 'gemini-cli' || kind === 'deepseek-cli' || kind === 'opencode') {
    return kind
  }
  return null
}

function resolvedAiBrandKind(
  workspace: WorkspaceCard | undefined | null,
  info: AiCliInfo,
): AiCliKind | null {
  const direct = hasAiBrandIcon(info) ? info.kind : null
  if (direct) return direct
  return null
}

function renderAiBrandIcon(kind: AiCliKind, variant: string, fallbackIconName: string, size = 10): VNode {
  const src = aiCliBrandIconSrc(kind)
  if (src) {
    return h('img', {
      src,
      alt: '',
      class: ['ai-brand-icon', `ai-brand-icon--${variant}`, aiCliBrandIconClass(kind)],
    })
  }
  return h(AppIcon, { name: fallbackIconName, size })
}

function explorerAiPaneStyle(workspace: WorkspaceCard, pane: PaneNode, session: PaneTerminalSession) {
  const info = aiCliInfoForSession(workspace, pane, session)
  const brandKind = resolvedAiBrandKind(workspace, info)
  const iconSrc = brandKind ? aiCliBrandIconSrc(brandKind) : null
  if (!iconSrc) {
    return info.kind === 'generic-ai'
      ? ({
          '--explorer-ai-watermark': 'linear-gradient(135deg, rgba(var(--accent-rgb), 0.22), rgba(var(--accent-rgb), 0.02))',
        } as Record<string, string>)
      : undefined
  }
  return {
    '--explorer-ai-watermark': `url("${iconSrc}")`,
  } as Record<string, string>
}

function aiCliInfoFromText(...parts: Array<string | null | undefined>): AiCliInfo {
  const text = normalizeSearchText(parts.filter(Boolean).join(' '))
  if (/(^|[\s>])(codex|cx)(?=$|[\s/:-])|openai codex|gpt-/.test(text)) {
    return { isAi: true, kind: 'codex', label: 'Codex CLI', shortLabel: 'CX', tone: 'blue', iconName: 'codex' }
  }
  if (/(^|[\s>])(claude|cc)(?=$|[\s/:-])|claude code|anthropic|sonnet|opus/.test(text)) {
    return { isAi: true, kind: 'claude-code', label: 'Claude Code', shortLabel: 'CC', tone: 'amber', iconName: 'claude' }
  }
  if (/(^|[\s>])gemini(?=$|[\s/:-])|google ai|google-ai/.test(text)) {
    return { isAi: true, kind: 'gemini-cli', label: 'Gemini CLI', shortLabel: 'GM', tone: 'green', iconName: 'gemini' }
  }
  if (/(^|[\s>])deepseek(?=$|[\s/:-])|deep seek/.test(text)) {
    return { isAi: true, kind: 'deepseek-cli', label: 'DeepSeek CLI', shortLabel: 'DS', tone: 'cyan', iconName: 'sparkle' }
  }
  if (/opencode|open code/.test(text)) {
    return { isAi: true, kind: 'opencode', label: 'OpenCode', shortLabel: 'OC', tone: 'indigo', iconName: 'opencode' }
  }
  if (/\b(ai\s*cli|agent|assistant|llm)\b/.test(text)) {
    return { isAi: true, kind: 'generic-ai', label: 'AI CLI', shortLabel: 'AI', tone: 'cyan', iconName: 'sparkle' }
  }
  return { isAi: false, kind: 'custom-cli', label: 'Terminal', shortLabel: 'SH', tone: 'neutral', iconName: 'terminal' }
}

function aiCliInfoForEntry(entry?: TerminalEntry | null): AiCliInfo {
  if (!entry) return aiCliInfoFromText('')
  return aiCliInfoFromText(
    entry.defaultCommand,
    entry.lastCommand,
  )
}

function lastAiCliInfoForSession(workspace: WorkspaceCard | undefined | null, pane: PaneNode, session?: PaneTerminalSession): AiCliInfo {
  if (session?.lastAiCliKind && session.lastAiCliKind !== 'generic-ai') {
    return aiCliInfoFromText(session.lastAiCliKind)
  }
  if (session?.aiCliKind && session.aiCliKind !== 'generic-ai') {
    return aiCliInfoFromText(session.aiCliKind)
  }
  const entry = workspace ? workspaceEntryById(workspace, session?.terminalEntryId ?? pane.terminalEntryId) : undefined
  const outputInfo = aiCliInfoFromText(session ? sessionOutputTailBySession.get(session.id) ?? '' : '')
  const entryInfo = aiCliInfoForEntry(entry)
  if (outputInfo.isAi && outputInfo.kind !== 'generic-ai') return outputInfo
  if (entryInfo.isAi && entryInfo.kind !== 'generic-ai') return entryInfo
  return aiCliInfoFromText('')
}

function aiCliInfoForSession(workspace: WorkspaceCard | undefined | null, pane: PaneNode, session?: PaneTerminalSession): AiCliInfo {
  if (session?.aiCliKind && session.aiCliKind !== 'generic-ai') {
    return aiCliInfoFromText(session.aiCliKind)
  }
  const outputInfo = aiCliInfoFromText(session ? sessionOutputTailBySession.get(session.id) ?? '' : '')
  if (outputInfo.isAi && outputInfo.kind !== 'generic-ai') return outputInfo
  return aiCliInfoFromText('')
}

function stripAnsiSequences(value: string) {
  return value
    .replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, '')
    .replace(/\u001b\][^\u0007]*(\u0007|\u001b\\)/g, '')
}

function hasReturnedToShellPrompt(value: string) {
  const cleaned = stripAnsiSequences(value).replace(/\r/g, '')
  return /(^|\n)(?:PS [^\n>]*>\s*|[A-Z]:\\[^\n>]*>\s*|\[[^\n\]]+\]\s*[#$>]\s*|[^\n]*[#$>]\s*)$/i.test(cleaned)
}

function hasReturnedToAiReadyState(value: string, kind: AiCliKind | null | undefined) {
  if (!kind || kind === 'generic-ai' || kind === 'custom-cli') return false
  const cleaned = stripAnsiSequences(value).replace(/\r/g, '\n')

  if (kind === 'claude-code') {
    return /\?\s+for shortcuts/i.test(cleaned) || /esc\s+for agents/i.test(cleaned) || /Try\s+"[^"]+"/i.test(cleaned)
  }

  if (kind === 'codex') {
    return /Use \/skills to list available skills/i.test(cleaned) || /\/model to change/i.test(cleaned)
  }

  if (kind === 'gemini-cli') {
    return /Type (?:a message|something)/i.test(cleaned) || /\/help/i.test(cleaned)
  }

  return false
}

function isGenericTerminalSessionName(value: string | null | undefined) {
  const text = normalizeSearchText(value ?? '')
  if (!text) return true
  return text === 'powershell'
    || text === 'powershell 7'
    || text === 'pwsh'
    || text === 'terminal'
    || text === 'shell'
    || text === 'cmd'
    || text === 'command prompt'
    || text === 'bash'
    || text === 'zsh'
    || /^pane\d+$/.test(text)
}

function sessionDisplayName(
  workspace: WorkspaceCard | undefined | null,
  pane: PaneNode,
  session: PaneTerminalSession,
  info = aiCliInfoForSession(workspace, pane, session),
) {
  const raw = session.name?.trim() || pane.name?.trim() || tr('终端', 'Terminal')
  if (!info.isAi) return raw
  if (isGenericTerminalSessionName(raw)) return aiCliDisplayName(info.kind)
  return raw
}

function sessionHistoryDisplayName(
  workspace: WorkspaceCard | undefined | null,
  pane: PaneNode,
  session: PaneTerminalSession,
  currentInfo = aiCliInfoForSession(workspace, pane, session),
  historyInfo = lastAiCliInfoForSession(workspace, pane, session),
) {
  if (currentInfo.isAi) return sessionDisplayName(workspace, pane, session, currentInfo)
  const raw = session.name?.trim() || pane.name?.trim() || tr('终端', 'Terminal')
  if (historyInfo.isAi && isGenericTerminalSessionName(raw)) return aiCliDisplayName(historyInfo.kind)
  return raw
}

function sessionDisplayTitle(
  workspace: WorkspaceCard | undefined | null,
  tab: WorkspaceTab | undefined | null,
  pane: PaneNode,
  session: PaneTerminalSession,
) {
  const info = aiCliInfoForSession(workspace, pane, session)
  const displayName = sessionDisplayName(workspace, pane, session, info)
  const entry = workspace ? workspaceEntryById(workspace, session.terminalEntryId ?? pane.terminalEntryId) : undefined
  return [workspace?.name ?? '', tab?.name ?? '', pane.name, displayName, entry?.workingDirectory ?? '', entry?.defaultCommand ?? '', entry?.lastCommand ?? '']
}

function shouldShowAiAssistState(state: SessionAttentionState) {
  return state === 'running'
    || state === 'waiting'
    || state === 'needs-input'
    || state === 'stalled'
    || state === 'completed'
    || state === 'error'
}

function aiSessionStateBadgeClass(state: SessionAttentionState) {
  if (state === 'waiting' || state === 'needs-input' || state === 'stalled') return 'waiting'
  if (state === 'completed') return 'completed'
  if (state === 'error') return 'error'
  if (state === 'running') return 'running'
  if (state === 'fresh') return 'fresh'
  return 'idle'
}

function sessionAttentionStateLabel(state: SessionAttentionState) {
  if (state === 'fresh') return t('ai.states.fresh')
  if (state === 'running') return t('ai.states.running')
  if (state === 'waiting') return t('ai.states.waiting')
  if (state === 'needs-input') return t('ai.states.needsInput')
  if (state === 'completed') return t('ai.states.completed')
  if (state === 'error') return t('ai.states.error')
  if (state === 'stalled') return t('ai.states.stalled')
  return t('ai.states.idle')
}

function toggleAiAssistantVisibility() {
  if (!activeAiAssistItem.value) return
  if (!aiAssistantPinned.value) {
    aiAssistantPinned.value = true
    aiAssistantMinimized.value = false
    return
  }
  aiAssistantMinimized.value = !aiAssistantMinimized.value
}

function pinAiAssistItem(itemId: string) {
  pinnedAiAssistItemId.value = pinnedAiAssistItemId.value === itemId ? null : itemId
}

function clampAiAssistCardPosition(left: number, top: number) {
  const card = aiAssistCardRef.value
  if (!card) return { left, top }

  const padding = 12
  const rect = card.getBoundingClientRect()
  return {
    left: clamp(left, padding, Math.max(padding, window.innerWidth - rect.width - padding)),
    top: clamp(top, padding, Math.max(padding, window.innerHeight - rect.height - padding)),
  }
}

function beginAiAssistDrag(event: PointerEvent) {
  if (event.button !== 0) return
  const target = event.target as HTMLElement | null
  if (target?.closest('[data-no-drag="true"]')) return

  const card = aiAssistCardRef.value
  const header = event.currentTarget as HTMLElement | null
  if (!card || !header) return

  const rect = card.getBoundingClientRect()
  aiAssistCardPosition.value = { left: rect.left, top: rect.top }
  aiAssistDragging.value = true
  aiAssistDragState.active = true
  aiAssistDragState.pointerId = event.pointerId
  aiAssistDragState.pointerOffsetX = event.clientX - rect.left
  aiAssistDragState.pointerOffsetY = event.clientY - rect.top
  try {
    header.setPointerCapture?.(event.pointerId)
  } catch {
    // Synthetic pointer events used in verification do not always create capturable pointers.
  }
  document.body.classList.add('is-dragging-ai-assist')
}

function handleAiAssistPointerMove(event: PointerEvent) {
  if (!aiAssistDragState.active || event.pointerId !== aiAssistDragState.pointerId) return
  const nextPosition = clampAiAssistCardPosition(
    event.clientX - aiAssistDragState.pointerOffsetX,
    event.clientY - aiAssistDragState.pointerOffsetY,
  )
  aiAssistCardPosition.value = nextPosition
}

function endAiAssistDrag(event?: PointerEvent) {
  if (!aiAssistDragState.active) return
  if (event && event.pointerId !== aiAssistDragState.pointerId) return
  aiAssistDragState.active = false
  aiAssistDragState.pointerId = -1
  aiAssistDragging.value = false
  document.body.classList.remove('is-dragging-ai-assist')
}

function clampAiAssistCardIntoViewport() {
  if (!aiAssistCardPosition.value) return
  aiAssistCardPosition.value = clampAiAssistCardPosition(aiAssistCardPosition.value.left, aiAssistCardPosition.value.top)
}

function closeAiHistoryDrawerBeforeAction(action: () => void) {
  openAiHistoryDrawer.value = false
  nextTick(() => {
    window.requestAnimationFrame(action)
  })
}

function openAiCliSessionItem(item: AiCliSessionItem) {
  closeAiHistoryDrawerBeforeAction(() => {
    openWorkspaceTerminalSession(item.workspaceId, item.tabId, item.paneId, item.sessionId)
  })
}

function openAiAssistTarget(item: AiAssistItem) {
  openWorkspaceTerminalSession(item.workspaceId, item.tabId, item.paneId, item.sessionId)
}

function insertAiCommand(item: AiCliCommandItem) {
  closeAiHistoryDrawerBeforeAction(() => {
    openRecentCommandTarget(item.workspaceId, item.entryId, item.command)
  })
}

function restoreAiSnapshotFromHistory(workspaceId: string, snapshotId: string) {
  closeAiHistoryDrawerBeforeAction(() => {
    restoreWorkspaceSnapshotFromRecent(workspaceId, snapshotId)
  })
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
  showToast(t('toast.movedToRecycleBin'), t('toast.movedToRecycleBinMsg'))
}

function restoreHiddenRecentItems() {
  const count = hiddenRecentItemIds.value.length
  hiddenRecentItemIds.value = []
  showToast(t('toast.recentRestored'), t('toast.recentRestoredCount', { count }))
}

function restoreHiddenRecentItem(itemId: string) {
  hiddenRecentItemIds.value = hiddenRecentItemIds.value.filter((id) => id !== itemId)
  showToast(t('toast.recentRestored'), t('toast.recentRestoredSingle'))
}

function clearHiddenRecentItems() {
  hiddenRecentItemIds.value = []
  openRecentRecycleBinModal.value = false
  showToast(t('toast.recycleBinCleared'), t('toast.recycleBinClearedMsg'))
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
    showSearchLoopHint(t('search.loopFirst'))
  } else if (didLoopToEnd) {
    showSearchLoopHint(t('search.loopLast'))
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
  return restoreCommandStrategyOptions.value.find((option) => option.value === strategy)?.label ?? t('settings.restoreStrategies.layoutOnly')
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
  const fullWidthIndex = item.indexOf('：')
  const asciiIndex = item.indexOf(':')
  const index = fullWidthIndex >= 0 ? fullWidthIndex : asciiIndex
  if (index < 0) {
    return {
      label: item,
      value: tr('已配置', 'Configured'),
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
    showToast(tr('未找到来源终端', 'Source terminal not found'), tr('已先复制命令。你也可以在运行态里回填到当前输入框。', 'The command was copied first. You can still insert it manually from the runtime page.'))
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
    title: tr(`恢复现场「${snapshot.name}」`, `Restore snapshot "${snapshot.name}"`),
    description: tr('将恢复该现场保存时的项目结构、Pane 分栏和终端焦点，不会自动执行命令。', 'Restore the project structure, pane layout, and terminal focus captured in this snapshot without executing commands automatically.'),
    confirmLabel: tr('恢复现场', 'Restore snapshot'),
    variant: 'primary',
    details: [
      tr(`工作区：${workspace.name}`, `Workspace: ${workspace.name}`),
      tr(`项目数：${snapshot.tabsState.length}`, `Projects: ${snapshot.tabsState.length}`),
      tr(`终端数：${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`, `Terminals: ${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`),
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
    showToast(tr('暂无工作区', 'No workspace available'), tr('请先创建一个工作区，再使用工作流模板。', 'Create a workspace first before using workflow templates.'))
    return
  }

  requestConfirm({
    title: tr(`使用模板「${template.name}」`, `Apply template "${template.name}"`),
    description: tr(`将向工作区「${workspace.name}」新增一个项目，包含 ${template.panes.length} 个终端分组。不会自动执行命令；如果目标不是你想要的工作区，请先切换工作区后再使用。`, `A new project will be added into workspace "${workspace.name}" with ${template.panes.length} terminal groups. Commands will not execute automatically. If this is not the target workspace you want, switch workspaces first.`),
    confirmLabel: tr('创建项目', 'Create project'),
    variant: 'primary',
    details: [
      tr(`目标工作区：${workspace.name}`, `Target workspace: ${workspace.name}`),
      tr(`根目录：${workspace.rootPath}`, `Root path: ${workspace.rootPath}`),
      tr(`将创建终端：${template.panes.length} 个`, `Terminals to create: ${template.panes.length}`),
      tr(`预置命令：${template.panes.filter((pane) => pane.defaultCommand.trim()).length} 条`, `Preset commands: ${template.panes.filter((pane) => pane.defaultCommand.trim()).length}`),
      tr(`模板标签：${template.tags.length ? template.tags.join(' / ') : '无'}`, `Template tags: ${template.tags.length ? template.tags.join(' / ') : 'None'}`),
    ],
    action: () => applyWorkflowTemplateToWorkspace(template, workspace.id),
  })
}

function applyWorkflowTemplateToWorkspace(template: WorkflowTemplate, workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId) ?? templateApplyTargetWorkspace.value
  if (!workspace) {
    showToast(tr('工作区不存在', 'Workspace missing'), tr('当前模板应用目标已失效，请重新选择工作区。', 'The current template target is no longer valid. Please choose a workspace again.'))
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
      note: tr(`${template.name} 工作流模板生成`, `Generated from workflow template: ${template.name}`),
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
        aiCliKind: null,
        lastAiCliKind: null,
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
  showToast(tr('工作流模板已应用', 'Workflow template applied'), tr(`${template.name} · 已加入 ${workspace.name}，创建了 ${panes.length} 个终端，不会自动执行命令。`, `${template.name} was added into ${workspace.name}, creating ${panes.length} terminals without executing commands automatically.`))
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
    showToast(tr('无法保存模板', 'Cannot save template'), tr('请先打开一个工作区项目。', 'Open a workspace project first.'))
    return
  }

  const panes = flattenLeafPanes(tab.panes)
  if (!panes.length) {
    showToast(tr('无法保存模板', 'Cannot save template'), tr('当前项目还没有终端 Pane。', 'The current project has no terminal panes yet.'))
    return
  }

  requestConfirm({
    title: tr(`保存项目「${tab.name}」为模板`, `Save project "${tab.name}" as template`),
    description: tr(`将从工作区「${workspace.name}」的当前项目结构生成一条“我的模板”，包含 ${panes.length} 个 Pane，不会立即创建新终端。`, `A new personal template will be generated from the current project structure in workspace "${workspace.name}". It contains ${panes.length} panes and does not create terminals immediately.`),
    confirmLabel: tr('保存模板', 'Save template'),
    variant: 'primary',
    details: [
      tr(`工作区：${workspace.name}`, `Workspace: ${workspace.name}`),
      tr(`项目：${tab.name}`, `Project: ${tab.name}`),
      tr(`Pane 数：${panes.length}`, `Pane count: ${panes.length}`),
      tr(`终端数：${countPaneSessions(tab.panes)}`, `Terminal count: ${countPaneSessions(tab.panes)}`),
    ],
    action: () => {
      const template = createWorkflowTemplateFromInput({
        name: tr(`${tab.name} 模板`, `${tab.name} Template`),
        description: tr(`从 ${workspace.name} / ${tab.name} 保存的工作流模板。`, `Workflow template saved from ${workspace.name} / ${tab.name}.`),
        tags: [tr('我的模板', 'My template')],
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
      showToast(tr('模板已保存', 'Template saved'), tr(`${template.name} · 来自 ${workspace.name} / ${tab.name}`, `${template.name} · from ${workspace.name} / ${tab.name}`))
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
    showToast(tr('信息未完整', 'Incomplete information'), tr('模板名称不能为空。', 'Template name cannot be empty.'))
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
    showToast(tr('模板已更新', 'Template updated'), workflowTemplateForm.name.trim())
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
    title: tr(`删除模板「${template.name}」`, `Delete template "${template.name}"`),
    description: tr('只会删除这条用户模板本身，不会影响已经基于它创建出来的工作区、项目、Pane 和终端。', 'This only removes the user template itself and does not affect workspaces, projects, panes, or terminals already created from it.'),
    confirmLabel: tr('删除模板', 'Delete template'),
    variant: 'danger',
    action: () => {
      userWorkflowTemplates.value = userWorkflowTemplates.value.filter((item) => item.id !== templateId)
      showToast(tr('模板已删除', 'Template deleted'), template.name)
    },
  })
}

function duplicateWorkflowTemplate(templateId: string) {
  const template = allWorkflowTemplates.value.find((item) => item.id === templateId)
  if (!template) return

  const copy = createWorkflowTemplateFromInput({
    name: tr(`${template.name} 副本`, `${template.name} Copy`),
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
  showToast(tr('模板已复制', 'Template duplicated'), copy.name)
}

function removeWorkspaceSnapshot(workspaceId: string, snapshotId: string) {
  if (!workspaceId || !snapshotId) return
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  const snapshot = workspace?.snapshots?.find((item) => item.id === snapshotId)
  if (!workspace || !snapshot) return

  requestConfirm({
    title: tr(`删除现场「${snapshot.name}」`, `Delete snapshot "${snapshot.name}"`),
    description: tr('删除后，这条现场记录将从最近页和工作区快照列表中移除。', 'After deletion, this snapshot will be removed from the Recent page and the workspace snapshot list.'),
    confirmLabel: tr('删除现场', 'Delete snapshot'),
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

      showToast(tr('现场已删除', 'Snapshot deleted'), snapshot.name)
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
    showToast(tr('信息未完整', 'Incomplete information'), tr('工作区名称和根目录不能为空。', 'Workspace name and root path cannot be empty.'))
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
    showToast(tr('工作区已创建', 'Workspace created'), tr(`已新增工作区：${nextWorkspace.name}`, `Created workspace: ${nextWorkspace.name}`))
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

    showToast(tr('工作区已更新', 'Workspace updated'), tr(`已保存：${workspaceForm.name.trim()}`, `Saved: ${workspaceForm.name.trim()}`))
  }

  closeWorkspaceEditorModal()
}

function removeWorkspace(workspaceId: string) {
  const workspace = workspaces.value.find((item) => item.id === workspaceId)
  if (!workspace) return

  requestConfirm({
    title: tr(`删除工作区「${workspace.name}」`, `Delete workspace "${workspace.name}"`),
    description: tr('该操作会同时移除工作区内的标签页、Pane 布局、运行配置和 Provider。', 'This removes tabs, pane layouts, run configs, and providers inside the workspace.'),
    confirmLabel: tr('确认删除', 'Delete workspace'),
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
      showToast(tr('工作区已删除', 'Workspace deleted'), tr(`已移除：${workspace.name}`, `Removed: ${workspace.name}`))
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
  terminalEntryForm.environmentVariablesText = ''
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
  terminalEntryForm.environmentVariablesText = entry.environmentVariablesText ?? ''
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
    showToast(tr('信息未完整', 'Incomplete information'), tr('配置名称和工作目录不能为空。', 'Config name and working directory cannot be empty.'))
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
      environmentVariablesText: terminalEntryForm.environmentVariablesText.trim() || null,
      tags,
      note: terminalEntryForm.note.trim() || null,
    })

    patchSelectedWorkspace((workspace) => ({
      ...workspace,
      terminalEntries: [...workspace.terminalEntries, nextEntry],
      updatedAt: new Date().toISOString(),
    }))

    showToast(tr('运行配置已创建', 'Run config created'), tr(`已新增配置：${nextEntry.name}`, `Created config: ${nextEntry.name}`))
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
          environmentVariablesText: terminalEntryForm.environmentVariablesText.trim() || null,
          tags,
          note: terminalEntryForm.note.trim() || null,
          updatedAt: now,
        }
      }),
      updatedAt: new Date().toISOString(),
    }))

    showToast(tr('运行配置已更新', 'Run config updated'), tr(`已保存：${terminalEntryForm.name.trim()}`, `Saved: ${terminalEntryForm.name.trim()}`))
  }

  closeTerminalEntryEditorModal()
}

function removeTerminalEntry(entryId: string) {
  if (!selectedWorkspace.value) return
  const entry = selectedWorkspace.value.terminalEntries.find((item) => item.id === entryId)
  if (!entry) return

  if (referencedTerminalEntryIds.value.has(entryId)) {
    showToast(tr('无法删除配置', 'Cannot delete config'), tr('该配置仍被某个 Pane 引用，请先解除绑定或修改布局。', 'This config is still referenced by a pane. Unbind it or change the layout first.'))
    return
  }

  requestConfirm({
    title: tr(`删除运行配置「${entry.name}」`, `Delete run config "${entry.name}"`),
    description: tr('删除后，该运行配置的目录、命令、环境变量和启动模式会从当前工作区移除。', 'After deletion, the directory, command, environment variables, and launch mode stored in this config will be removed from the current workspace.'),
    confirmLabel: tr('确认删除', 'Delete config'),
    action: () => {
      patchSelectedWorkspace((workspace) => ({
        ...workspace,
        terminalEntries: workspace.terminalEntries.filter((item) => item.id !== entryId),
        updatedAt: new Date().toISOString(),
      }))

      showToast(tr('运行配置已删除', 'Run config deleted'), tr(`已移除：${entry.name}`, `Removed: ${entry.name}`))
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
    name: tr(`标签页 ${nextIndex}`, `Tab ${nextIndex}`),
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
  showToast(tr('已新建标签页', 'Tab created'), tr(`当前标签页：${nextTab.name}`, `Current tab: ${nextTab.name}`))
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
      aiCliKind: null,
      lastAiCliKind: null,
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
  showToast(tr('终端已创建', 'Terminal created'), tr(`当前标签页新增：${paneName}`, `Added to current tab: ${paneName}`))
}

function openProviderCreateModal() {
  if (!selectedWorkspace.value) return
  closeFloatingMenus()
  openTerminalEntriesModal.value = false
  providerEditorMode.value = 'create'
  editingProviderId.value = null
  providerForm.name = ''
  providerForm.providerKind = activeProviderToolFilter.value === 'all' ? 'codex' : activeProviderToolFilter.value
  providerForm.profileName = ''
  providerForm.configPath = defaultConfigPathForProvider(providerForm.providerKind)
  providerForm.configScope = 'global'
  providerForm.managedBy = 'cli-config'
  providerForm.authSource = defaultAuthSourceForProvider(providerForm.providerKind, providerForm.managedBy)
  providerForm.homepageUrl = ''
  providerForm.requestBaseUrl = ''
  providerForm.switchCommand = ''
  providerForm.defaultModel = ''
  providerForm.toolTargetsText = defaultProviderTargetsText(providerForm.providerKind)
  providerForm.color = providerKindColor(providerForm.providerKind)
  providerForm.note = ''
  providerForm.configPayload = null
  providerForm.authPayload = null
  providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
  providerForm.isActive = selectedWorkspaceProviders.value.length === 0
  openProviderEditorModal.value = true
}

function openModelPricingModal() {
  closeFloatingMenus()
  openModelPricingModalState.value = true
}

function closeModelPricingModal() {
  openModelPricingModalState.value = false
}

function onModelPricingSaved() {
  showToast(
    tr('模型价格已保存', 'Model pricing saved'),
    tr('已写入后端计费表；新查询的 Usage 成本将使用最新价格。', 'Saved to backend pricing table; new usage queries will use the latest prices.'),
  )
  if (appSection.value === 'usage' || appSection.value === 'providers') {
    void loadManagedUsageLive({ reset: true })
  }
}

async function importCurrentCodexProfile() {
  if (!selectedWorkspace.value) return
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast(
      tr('浏览器模式无法导入', 'Browser mode cannot import'),
      tr('导入当前 Codex 配置需要 Tauri 桌面模式读取用户目录。', 'Importing the current Codex config needs Tauri desktop mode to read your user directory.'),
    )
    return
  }

  providerDetectionRunning.value = true
  providerDetectionSummary.value = tr('正在读取当前 ~/.codex/config.toml 与 auth.json。', 'Reading the current ~/.codex/config.toml and auth.json.')
  try {
    const detected = await readCurrentCodexProfile()
    closeFloatingMenus()
    openTerminalEntriesModal.value = false
    providerEditorMode.value = 'create'
    editingProviderId.value = null
    providerForm.name = `${detected.name} Snapshot`
    providerForm.providerKind = 'codex'
    providerForm.profileName = detected.profileName || 'default'
    providerForm.configPath = detected.configPath || defaultConfigPathForProvider('codex')
    providerForm.configScope = detected.configScope
    providerForm.managedBy = detected.managedBy
    providerForm.authSource = detected.authSource || defaultAuthSourceForProvider('codex', detected.managedBy)
    providerForm.homepageUrl = baseUrlOrigin(extractCodexBaseUrlFromPayload(detected.configPayload))
    providerForm.requestBaseUrl = extractCodexBaseUrlFromPayload(detected.configPayload)
    providerForm.switchCommand = ''
    providerForm.defaultModel = detected.defaultModel
    providerForm.toolTargetsText = detected.toolTargets.join(', ') || defaultProviderTargetsText('codex')
    providerForm.color = detected.color || providerKindColor('codex')
    providerForm.note = detected.note || tr('从当前 Codex live config 导入，可作为独立档案再次激活写回。', 'Imported from the current Codex live config and can be activated later as an independent profile.')
    providerForm.configPayload = detected.configPayload ?? null
    providerForm.authPayload = detected.authPayload ?? null
    providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
    providerForm.isActive = false
    openProviderEditorModal.value = true
    providerDetectionSummary.value = tr('已读取当前 Codex 配置，确认名称后保存为 Chuchen 档案。', 'Current Codex config loaded. Confirm the name to save it as a Chuchen profile.')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`导入失败：${message}`, `Import failed: ${message}`)
    showToast(tr('Codex 导入失败', 'Codex import failed'), message)
  } finally {
    providerDetectionRunning.value = false
  }
}

async function importCurrentGeminiProfileAction() {
  if (!selectedWorkspace.value) return
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast(
      tr('浏览器模式无法导入', 'Browser mode cannot import'),
      tr('导入当前 Gemini 配置需要 Tauri 桌面模式读取用户目录。', 'Importing the current Gemini config needs Tauri desktop mode to read your user directory.'),
    )
    return
  }

  providerDetectionRunning.value = true
  providerDetectionSummary.value = tr('正在读取当前 ~/.gemini/.env 与 settings.json。', 'Reading the current ~/.gemini/.env and settings.json.')
  try {
    const detected = await readCurrentGeminiProfile()
    closeFloatingMenus()
    openTerminalEntriesModal.value = false
    providerEditorMode.value = 'create'
    editingProviderId.value = null
    providerForm.name = `${detected.name} Snapshot`
    providerForm.providerKind = 'gemini-cli'
    providerForm.profileName = detected.profileName || 'default'
    providerForm.configPath = detected.configPath || defaultConfigPathForProvider('gemini-cli')
    providerForm.configScope = detected.configScope
    providerForm.managedBy = detected.managedBy
    providerForm.authSource = detected.authSource || defaultAuthSourceForProvider('gemini-cli', detected.managedBy)
    providerForm.homepageUrl = detected.homepageUrl ?? ''
    providerForm.requestBaseUrl = detected.requestBaseUrl ?? ''
    providerForm.switchCommand = ''
    providerForm.defaultModel = detected.defaultModel
    providerForm.toolTargetsText = detected.toolTargets.join(', ') || defaultProviderTargetsText('gemini-cli')
    providerForm.color = detected.color || providerKindColor('gemini-cli')
    providerForm.note = detected.note || tr('从当前 Gemini live config 导入，可作为独立档案再次激活写回。', 'Imported from the current Gemini live config and can be activated later as an independent profile.')
    providerForm.configPayload = detected.configPayload ?? null
    providerForm.authPayload = detected.authPayload ?? null
    providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
    providerForm.isActive = false
    openProviderEditorModal.value = true
    providerDetectionSummary.value = tr('已读取当前 Gemini 配置，确认名称后保存为 Chuchen 档案。', 'Current Gemini config loaded. Confirm the name to save it as a Chuchen profile.')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`导入失败：${message}`, `Import failed: ${message}`)
    showToast(tr('Gemini 导入失败', 'Gemini import failed'), message)
  } finally {
    providerDetectionRunning.value = false
  }
}

async function importCurrentClaudeProfileAction() {
  if (!selectedWorkspace.value) return
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast(
      tr('浏览器模式无法导入', 'Browser mode cannot import'),
      tr('导入当前 Claude 配置需要 Tauri 桌面模式读取用户目录。', 'Importing the current Claude config needs Tauri desktop mode to read your user directory.'),
    )
    return
  }

  providerDetectionRunning.value = true
  providerDetectionSummary.value = tr('正在读取当前 ~/.claude/settings.json 与 ~/.claude.json。', 'Reading the current ~/.claude/settings.json and ~/.claude.json.')
  try {
    const detected = await readCurrentClaudeProfile()
    closeFloatingMenus()
    openTerminalEntriesModal.value = false
    providerEditorMode.value = 'create'
    editingProviderId.value = null
    providerForm.name = `${detected.name} Snapshot`
    providerForm.providerKind = 'claude-code'
    providerForm.profileName = detected.profileName || 'default'
    providerForm.configPath = detected.configPath || defaultConfigPathForProvider('claude-code')
    providerForm.configScope = detected.configScope
    providerForm.managedBy = detected.managedBy
    providerForm.authSource = detected.authSource || defaultAuthSourceForProvider('claude-code', detected.managedBy)
    providerForm.homepageUrl = detected.homepageUrl ?? ''
    providerForm.requestBaseUrl = detected.requestBaseUrl ?? ''
    providerForm.switchCommand = ''
    providerForm.defaultModel = detected.defaultModel
    providerForm.toolTargetsText = detected.toolTargets.join(', ') || defaultProviderTargetsText('claude-code')
    providerForm.color = detected.color || providerKindColor('claude-code')
    providerForm.note = detected.note || tr('从当前 Claude live config 导入，可作为独立档案再次激活写回。', 'Imported from the current Claude live config and can be activated later as an independent profile.')
    providerForm.configPayload = detected.configPayload ?? null
    providerForm.authPayload = detected.authPayload ?? null
    providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
    providerForm.isActive = false
    openProviderEditorModal.value = true
    providerDetectionSummary.value = tr('已读取当前 Claude 配置，确认名称后保存为 Chuchen 档案。', 'Current Claude config loaded. Confirm the name to save it as a Chuchen profile.')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`导入失败：${message}`, `Import failed: ${message}`)
    showToast(tr('Claude 导入失败', 'Claude import failed'), message)
  } finally {
    providerDetectionRunning.value = false
  }
}

async function importCurrentHermesProfileAction() {
  if (!selectedWorkspace.value) return
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast(
      tr('浏览器模式无法导入', 'Browser mode cannot import'),
      tr('导入当前 Hermes 配置需要 Tauri 桌面模式读取用户目录。', 'Importing the current Hermes config needs Tauri desktop mode to read your user directory.'),
    )
    return
  }

  providerDetectionRunning.value = true
  providerDetectionSummary.value = tr('正在读取当前 ~/.hermes/config.yaml。', 'Reading the current ~/.hermes/config.yaml.')
  try {
    const detected = await readCurrentHermesProfile()
    closeFloatingMenus()
    openTerminalEntriesModal.value = false
    providerEditorMode.value = 'create'
    editingProviderId.value = null
    providerForm.name = `${detected.name} Snapshot`
    providerForm.providerKind = 'hermes'
    providerForm.profileName = detected.profileName || 'default'
    providerForm.configPath = detected.configPath || defaultConfigPathForProvider('hermes')
    providerForm.configScope = detected.configScope
    providerForm.managedBy = detected.managedBy
    providerForm.authSource = detected.authSource || defaultAuthSourceForProvider('hermes', detected.managedBy)
    providerForm.homepageUrl = detected.homepageUrl ?? ''
    providerForm.requestBaseUrl = detected.requestBaseUrl ?? ''
    providerForm.switchCommand = ''
    providerForm.defaultModel = detected.defaultModel
    providerForm.toolTargetsText = detected.toolTargets.join(', ') || defaultProviderTargetsText('hermes')
    providerForm.color = detected.color || providerKindColor('hermes')
    providerForm.note = detected.note || tr('从当前 Hermes live config 导入，可作为独立档案再次激活写回。', 'Imported from the current Hermes live config and can be activated later as an independent profile.')
    providerForm.configPayload = detected.configPayload ?? null
    providerForm.authPayload = detected.authPayload ?? null
    providerForm.isDefault = selectedWorkspaceProviders.value.length === 0
    providerForm.isActive = false
    openProviderEditorModal.value = true
    providerDetectionSummary.value = tr('已读取当前 Hermes 配置，确认名称后保存为 Chuchen 档案。', 'Current Hermes config loaded. Confirm the name to save it as a Chuchen profile.')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`导入失败：${message}`, `Import failed: ${message}`)
    showToast(tr('Hermes 导入失败', 'Hermes import failed'), message)
  } finally {
    providerDetectionRunning.value = false
  }
}

function toggleProviderExpand(providerId: string) {
  if (expandedProviderId.value === providerId) {
    expandedProviderId.value = ''
  } else {
    expandedProviderId.value = providerId
    activeProviderStatsId.value = providerId
  }
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
  providerForm.profileName = provider.profileName
  providerForm.configPath = provider.configPath
  providerForm.configScope = provider.configScope
  providerForm.managedBy = provider.managedBy
  providerForm.authSource = provider.authSource
  providerForm.homepageUrl = provider.homepageUrl ?? ''
  providerForm.requestBaseUrl = provider.requestBaseUrl ?? ''
  providerForm.switchCommand = provider.switchCommand
  providerForm.defaultModel = provider.defaultModel
  providerForm.toolTargetsText = provider.toolTargets.join(', ')
  providerForm.color = provider.color || providerKindColor(provider.providerKind)
  providerForm.note = provider.note || ''
  providerForm.configPayload = provider.configPayload ?? null
  providerForm.authPayload = provider.authPayload ?? null
  providerForm.isDefault = Boolean(provider.isDefault)
  providerForm.isActive = Boolean(provider.isActive)
  openProviderEditorModal.value = true
}

function closeProviderEditorModal() {
  openProviderEditorModal.value = false
  editingProviderId.value = null
  openProviderKindMenu.value = false
  openProviderSourceMenu.value = false
  openProviderScopeMenu.value = false
}

function submitProviderForm() {
  if (!selectedWorkspace.value) return
  if (!providerForm.name.trim()) {
    showToast(tr('信息未完整', 'Incomplete information'), tr('Provider 名称不能为空。', 'Provider name cannot be empty.'))
    return
  }

  const toolTargets = parseProviderToolTargets(providerForm.toolTargetsText)
  const now = new Date().toISOString()
  const profileName = providerForm.profileName.trim() || 'default'
  const switchCommand = providerForm.switchCommand.trim()
  const homepageUrl = providerForm.homepageUrl.trim() || baseUrlOrigin(providerForm.requestBaseUrl.trim()) || null
  const requestBaseUrl = providerForm.requestBaseUrl.trim() || null
  const rawAuthPayload = normalizeProviderPayload(providerForm.authPayload)
  const authPayload = providerForm.providerKind === 'gemini-cli'
    ? buildGeminiAuthPayloadFromForm() || rawAuthPayload || null
    : rawAuthPayload || null
  const configPayload = providerForm.providerKind === 'codex'
    ? buildCodexPayloadFromForm()
    : providerForm.providerKind === 'gemini-cli'
      ? buildGeminiSettingsPayloadFromForm(authPayload ?? '')
      : normalizeProviderPayload(providerForm.configPayload) || null

  if (providerForm.providerKind === 'codex' && !configPayload) {
    showToast(tr('信息未完整', 'Incomplete information'), tr('Codex 档案至少需要一份完整 config.toml。', 'A Codex profile needs a full config.toml payload.'))
    return
  }
  if (providerForm.providerKind === 'gemini-cli' && !configPayload) {
    showToast(tr('信息未完整', 'Incomplete information'), tr('Gemini 档案至少需要一份 settings.json 快照。', 'A Gemini profile needs a settings.json payload.'))
    return
  }

  if (providerEditorMode.value === 'create') {
    const canActivate = providerCanBeActivated({
      providerKind: providerForm.providerKind,
      status: 'available',
      switchCommand,
      configPayload,
    })
    if (providerForm.isActive && !canActivate) {
      showToast(tr('无法设为当前档案', 'Cannot activate profile'), tr('当前档案还没有接通真实写回路径，请先补全 payload 或外部切换命令。', 'This profile does not have a real writeback path yet. Complete the payload or provide an external switch command first.'))
      return
    }
    const provider = createProviderProfileRecord({
      workspaceId: selectedWorkspace.value.id,
      name: providerForm.name.trim(),
      providerKind: providerForm.providerKind,
      profileName,
      configPath: providerForm.configPath.trim() || defaultConfigPathForProvider(providerForm.providerKind),
      configScope: providerForm.configScope,
      managedBy: providerForm.managedBy,
      authSource: providerForm.authSource.trim() || defaultAuthSourceForProvider(providerForm.providerKind, providerForm.managedBy),
      switchCommand,
      defaultModel: providerForm.defaultModel.trim(),
      toolTargets,
      status: providerForm.isActive && canActivate ? 'active' : 'available',
      color: providerForm.color.trim() || null,
      note: providerForm.note.trim() || null,
      homepageUrl,
      requestBaseUrl,
      configPayload,
      authPayload,
      isDefault: providerForm.isDefault,
      isActive: providerForm.isActive && canActivate,
    })

    patchSelectedWorkspace((workspace) => {
      const nextProfiles = normalizeActiveProviderProfiles([
        ...(workspace.providerProfiles ?? []).map((item) => ({
          ...item,
          isDefault: provider.isDefault ? false : item.isDefault,
          isActive: provider.isActive && item.providerKind === provider.providerKind ? false : item.isActive,
          status: provider.isActive && item.providerKind === provider.providerKind ? 'available' : item.status,
        })),
        provider,
      ])

      return {
        ...workspace,
        providerProfiles: nextProfiles,
        providerUsageStats: upsertProviderUsageStatsForProfiles(workspace.providerUsageStats ?? [], nextProfiles),
        updatedAt: now,
      }
    })

    activeProviderStatsId.value = provider.id
    showToast(tr('配置档案已创建', 'Profile created'), provider.name)
  } else if (editingProviderId.value) {
    const existingProvider = selectedWorkspaceProviders.value.find((provider) => provider.id === editingProviderId.value)
    if (!existingProvider) return
    const canActivate = providerCanBeActivated({
      providerKind: providerForm.providerKind,
      status: existingProvider.status === 'missing' || existingProvider.status === 'disabled' ? existingProvider.status : 'available',
      switchCommand,
      configPayload,
    })

    const editedProvider: ProviderProfile = {
      ...existingProvider,
      name: providerForm.name.trim(),
      providerKind: providerForm.providerKind,
      profileName,
      configPath: providerForm.configPath.trim() || defaultConfigPathForProvider(providerForm.providerKind),
      configScope: providerForm.configScope,
      managedBy: providerForm.managedBy,
      authSource: providerForm.authSource.trim() || defaultAuthSourceForProvider(providerForm.providerKind, providerForm.managedBy),
      switchCommand,
      defaultModel: providerForm.defaultModel.trim(),
      toolTargets,
      status: providerForm.isActive && canActivate ? 'active' : existingProvider.status === 'active' ? 'available' : existingProvider.status,
      isActive: providerForm.isActive && canActivate,
      lastDetectedAt: existingProvider.lastDetectedAt ?? now,
      color: providerForm.color.trim() || null,
      note: providerForm.note.trim() || null,
      homepageUrl,
      requestBaseUrl,
      configPayload,
      authPayload,
      isDefault: providerForm.isDefault,
      updatedAt: now,
    }

    if (providerForm.isActive && !canActivate) {
      showToast(tr('无法设为当前档案', 'Cannot activate profile'), tr('当前档案还没有接通真实写回路径，请先补全 payload 或外部切换命令。', 'This profile does not have a real writeback path yet. Complete the payload or provide an external switch command first.'))
      return
    }

    patchSelectedWorkspace((workspace) => {
      const nextProfiles = normalizeActiveProviderProfiles((workspace.providerProfiles ?? []).map((provider) => {
        if (provider.id === editingProviderId.value) return editedProvider
        return {
          ...provider,
          isDefault: editedProvider.isDefault ? false : provider.isDefault,
          isActive: editedProvider.isActive && provider.providerKind === editedProvider.providerKind ? false : provider.isActive,
          status: editedProvider.isActive && provider.providerKind === editedProvider.providerKind ? 'available' : provider.status,
        }
      }))

      return {
        ...workspace,
        providerProfiles: nextProfiles,
        providerUsageStats: upsertProviderUsageStatsForProfiles(workspace.providerUsageStats ?? [], nextProfiles),
        updatedAt: now,
      }
    })

    showToast(tr('配置档案已更新', 'Profile updated'), providerForm.name.trim())
  }

  closeProviderEditorModal()
}

function removeProviderProfile(providerId: string) {
  if (!selectedWorkspace.value) return
  const provider = selectedWorkspaceProviders.value.find((item) => item.id === providerId)
  if (!provider) return

  requestConfirm({
    title: tr(`删除配置档案「${provider.name}」`, `Delete profile "${provider.name}"`),
    description: tr('删除后只会移除 Chuchen-Terminal 中登记的档案记录，不会改动本机真实 CLI 配置文件。', 'This only removes the registered profile inside Chuchen-Terminal and does not modify the real local CLI config files.'),
    confirmLabel: tr('删除档案', 'Delete profile'),
    action: () => {
      patchSelectedWorkspace((workspace) => ({
        ...workspace,
        providerProfiles: (workspace.providerProfiles ?? []).filter((item) => item.id !== providerId),
        providerUsageStats: (workspace.providerUsageStats ?? []).filter((stats) => stats.providerProfileId !== providerId),
        updatedAt: new Date().toISOString(),
      }))
      activeProviderStatsId.value = selectedWorkspaceProviders.value.find((item) => item.id !== providerId)?.id ?? ''
      showToast(tr('配置档案已删除', 'Profile deleted'), provider.name)
    },
  })
}

/**
 * 仅浏览器预览：桌面模式激活后必须 re-detect，isActive 只信后端。
 * 禁止在 Tauri 路径里本地发明 active。
 */
function commitProviderActivationStatePreview(provider: ProviderProfile, activatedAt: string) {
  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    providerProfiles: normalizeActiveProviderProfiles((workspace.providerProfiles ?? []).map((item) => ({
      ...item,
      isActive: item.providerKind === provider.providerKind ? item.id === provider.id : item.isActive,
      isDefault: item.id === provider.id,
      status: item.providerKind === provider.providerKind
        ? item.id === provider.id ? 'active' : item.status === 'disabled' || item.status === 'missing' ? item.status : 'available'
        : item.status,
      lastDetectedAt: item.id === provider.id ? activatedAt : item.lastDetectedAt,
      updatedAt: item.id === provider.id ? activatedAt : item.updatedAt,
    }))),
    updatedAt: activatedAt,
  }))
}

/**
 * 激活成功后：按 detect 结果同步 isActive / payload / identityKey。
 * 不自行把目标档案标 active；只写后端返回的 isActive。
 */
async function rescanProviderProfilesAfterApply(preferredProviderId?: string) {
  if (!selectedWorkspace.value) return
  if (!("__TAURI_INTERNALS__" in window)) return

  const detectedProfiles = await detectLocalProviderProfiles()
  const now = new Date().toISOString()

  patchSelectedWorkspace((workspace) => {
    const nextProfiles = normalizeActiveProviderProfiles((workspace.providerProfiles ?? []).map((profile) => {
      const detected = findDetectedProfileForStoredProfile(detectedProfiles, profile)
      if (!detected) {
        return {
          ...profile,
          isActive: false,
          status: profile.status === 'active' ? 'available' : profile.status,
          updatedAt: now,
        }
      }
      const canActivate = detected.status !== 'missing' && detected.status !== 'disabled'
      return {
        ...profile,
        status: detected.status,
        isActive: Boolean(detected.isActive) && canActivate,
        identityKey: detected.identityKey || profile.identityKey || null,
        lastDetectedAt: now,
        authSource: detected.authSource || profile.authSource,
        switchCommand: detected.switchCommand || profile.switchCommand,
        defaultModel: detected.defaultModel || profile.defaultModel,
        toolTargets: detected.toolTargets.length ? detected.toolTargets : profile.toolTargets,
        color: detected.color || profile.color,
        note: detected.note || profile.note,
        homepageUrl: detected.homepageUrl ?? profile.homepageUrl ?? null,
        requestBaseUrl: detected.requestBaseUrl ?? profile.requestBaseUrl ?? null,
        configPayload: detected.configPayload ?? profile.configPayload ?? null,
        authPayload: detected.authPayload ?? profile.authPayload ?? null,
        updatedAt: now,
      }
    }))

    return {
      ...workspace,
      providerProfiles: nextProfiles,
      providerUsageStats: upsertProviderUsageStatsForProfiles(workspace.providerUsageStats ?? [], nextProfiles),
      updatedAt: now,
    }
  })

  if (preferredProviderId) {
    activeProviderStatsId.value = preferredProviderId
  }
}

async function activateProviderProfile(providerId: string) {
  if (activatingProviderId.value) return
  const provider = selectedWorkspaceProviders.value.find((item) => item.id === providerId)
  if (!provider) return
  if (!providerCanBeActivated(provider)) {
    showToast(tr('无法设为当前档案', 'Cannot activate profile'), tr('未检测到或已停用的配置档案不能标记为当前。', 'Profiles that are missing or disabled cannot be marked as current.'))
    return
  }
  activatingProviderId.value = providerId

  if ("__TAURI_INTERNALS__" in window) {
    try {
      // 契约：切换时传完整 configPayload / authPayload
      await applyProviderProfile({
        providerKind: provider.providerKind,
        profileName: provider.profileName,
        switchCommand: provider.switchCommand,
        configPayload: provider.configPayload ?? null,
        authPayload: provider.authPayload ?? null,
      })
      // 写回成功后 re-scan；isActive 只信 detect
      await rescanProviderProfilesAfterApply(providerId)
      void loadManagedUsageLive({ reset: true })
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      showToast(tr('配置档案启用失败', 'Profile activation failed'), message)
      activatingProviderId.value = ''
      return
    }
  } else {
    // 浏览器预览：无真实写回，仅本地预览态
    commitProviderActivationStatePreview(provider, new Date().toISOString())
    showToast(
      tr('浏览器模式仅更新预览状态', 'Browser mode only updates preview state'),
      tr('真实档案切换需要在 Tauri 桌面模式下执行。', 'Real profile switching runs only in Tauri desktop mode.'),
    )
  }

  activeProviderStatsId.value = providerId
  activatingProviderId.value = ''
  showToast(tr('配置档案已启用', 'Profile activated'), `${providerKindLabel(provider.providerKind)} · ${provider.name}`)
}

function duplicateProviderProfile(providerId: string) {
  if (!selectedWorkspace.value) return
  const provider = selectedWorkspaceProviders.value.find((item) => item.id === providerId)
  if (!provider) return

  const now = new Date().toISOString()
  const profileName = `${provider.profileName || 'default'}-copy`
  const copy: ProviderProfile = {
    ...provider,
    id: createId('provider'),
    name: `${provider.name} Copy`,
    profileName,
    switchCommand: '',
    isDefault: false,
    isActive: false,
    status: provider.status === 'disabled' || provider.status === 'missing' ? provider.status : 'available',
    createdAt: now,
    updatedAt: now,
  }

  patchSelectedWorkspace((workspace) => ({
    ...workspace,
    providerProfiles: [...(workspace.providerProfiles ?? []), copy],
    providerUsageStats: [...(workspace.providerUsageStats ?? []), createEmptyProviderUsageStatsForProfile(copy.id)],
    updatedAt: now,
  }))

  activeProviderStatsId.value = copy.id
  showToast(tr('配置档案已复制', 'Profile duplicated'), copy.name)
}

async function copyProviderSwitchCommand(provider: ProviderProfile) {
  const command = provider.switchCommand.trim()
  if (!command) {
    showToast(
      tr('无法执行切换命令', 'Cannot run switch command'),
      tr('该档案没有登记外部切换命令；请使用内置激活逻辑或手动填写命令。', 'This profile has no external switch command. Use the built-in activation flow or enter a command manually.'),
    )
    return
  }
  await copyCommandText(command)
}

/**
 * 合并 detect 结果到本地档案。
 * - identityKey 优先匹配；payload/legacy key 次之
 * - 同步来源（cli-config / cc-switch）在本轮 detect 中消失 → 清理
 * - manual / oauth / env / script 手工档案一律保留
 * 注意：当前后端 detect_local_provider_profiles 仍返回 native+CC Switch 合并目录，
 * 前端入口拆分但不会伪造“只含 native”的结果。
 */
function mergeDetectedProviderProfiles(
  workspace: WorkspaceCard,
  detectedProfiles: DetectedProviderProfile[],
  options: { pruneSyncedStale: boolean; preferManagedBy?: ProviderProfileSource | null } = { pruneSyncedStale: true },
) {
  const now = new Date().toISOString()
  let addedCount = 0
  let updatedCount = 0
  let prunedCount = 0
  let firstImportedId = ''

  const nextProfiles = [...(workspace.providerProfiles ?? [])]
  const nextUsageStats = [...(workspace.providerUsageStats ?? [])]
  const seenIdentityKeys = new Set<string>()
  const matchedExistingIds = new Set<string>()

  // detect 结果自身按 identityKey 去重（保留第一条）
  const uniqueDetected: DetectedProviderProfile[] = []
  for (const detected of detectedProfiles) {
    const key = (detected.identityKey || '').trim()
    if (key) {
      if (seenIdentityKeys.has(key)) continue
      seenIdentityKeys.add(key)
    }
    uniqueDetected.push(detected)
  }

  uniqueDetected.forEach((detected) => {
    const existingIndex = findExistingProviderIndexByDetection(nextProfiles, detected)
    if (existingIndex >= 0) {
      const existing = nextProfiles[existingIndex]
      matchedExistingIds.add(existing.id)
      const detectedCanActivate = detected.status !== 'missing' && detected.status !== 'disabled'
      nextProfiles[existingIndex] = {
        ...existing,
        name: existing.name || detected.name,
        identityKey: detected.identityKey || existing.identityKey || null,
        configScope: detected.configScope,
        managedBy: detected.managedBy || existing.managedBy,
        authSource: detected.authSource || existing.authSource,
        switchCommand: detected.switchCommand || existing.switchCommand,
        defaultModel: detected.defaultModel || existing.defaultModel,
        toolTargets: detected.toolTargets.length ? detected.toolTargets : existing.toolTargets,
        status: detected.status,
        // 只信本机检测结果，禁止用旧 localStorage isActive 把整列表刷成“当前启用”
        isActive: detectedCanActivate && Boolean(detected.isActive),
        color: detected.color || existing.color,
        note: detected.note || existing.note,
        homepageUrl: detected.homepageUrl ?? existing.homepageUrl ?? null,
        requestBaseUrl: detected.requestBaseUrl ?? existing.requestBaseUrl ?? null,
        configPayload: detected.configPayload ?? existing.configPayload ?? null,
        authPayload: detected.authPayload ?? existing.authPayload ?? null,
        lastDetectedAt: now,
        updatedAt: now,
      }
      updatedCount += 1
      if (!firstImportedId) firstImportedId = existing.id
      const detectedStats = createProviderUsageStatsFromDetection(existing.id, detected)
      const usageStatsIndex = nextUsageStats.findIndex((stats) => stats.providerProfileId === existing.id)
      if (detectedStats && usageStatsIndex >= 0) {
        nextUsageStats[usageStatsIndex] = detectedStats
      } else if (detectedStats) {
        nextUsageStats.push(detectedStats)
      } else if (usageStatsIndex < 0) {
        nextUsageStats.push(createEmptyProviderUsageStatsForProfile(existing.id))
      }
      return
    }

    const provider = createProviderProfileFromDetection(workspace.id, detected)
    nextProfiles.push(provider)
    matchedExistingIds.add(provider.id)
    nextUsageStats.push(createProviderUsageStatsFromDetection(provider.id, detected) ?? createEmptyProviderUsageStatsForProfile(provider.id))
    addedCount += 1
    if (!firstImportedId) firstImportedId = provider.id
  })

  let keptProfiles = nextProfiles
  if (options.pruneSyncedStale) {
    keptProfiles = nextProfiles.filter((profile) => {
      const isSyncedSource = profile.managedBy === 'cli-config' || profile.managedBy === 'cc-switch'
      if (!isSyncedSource) return true
      if (matchedExistingIds.has(profile.id)) return true
      // 本轮 detect 未命中的同步档案 → 清理
      prunedCount += 1
      return false
    })
  }

  // identityKey 二次去重（保留更新更晚 / isActive 优先）
  const byIdentity = new Map<string, ProviderProfile>()
  const withoutIdentity: ProviderProfile[] = []
  for (const profile of keptProfiles) {
    const key = (profile.identityKey || '').trim()
    if (!key) {
      withoutIdentity.push(profile)
      continue
    }
    const existing = byIdentity.get(key)
    if (!existing) {
      byIdentity.set(key, profile)
      continue
    }
    const preferNext = (profile.isActive && !existing.isActive)
      || new Date(profile.updatedAt).getTime() >= new Date(existing.updatedAt).getTime()
    byIdentity.set(key, preferNext ? profile : existing)
  }
  const dedupedProfiles = normalizeActiveProviderProfiles([...byIdentity.values(), ...withoutIdentity])

  return {
    workspacePatch: {
      ...workspace,
      providerProfiles: dedupedProfiles,
      providerUsageStats: upsertProviderUsageStatsForProfiles(nextUsageStats, dedupedProfiles),
      updatedAt: now,
    } satisfies WorkspaceCard,
    addedCount,
    updatedCount,
    prunedCount,
    firstImportedId,
    totalDetected: uniqueDetected.length,
  }
}

/** 同步本机配置（入口：静默 / 显式）。后端仍可能混入 CC Switch 条目，文案诚实说明。 */
async function syncNativeProviderProfiles(options: { silent?: boolean; explicit?: boolean } = {}) {
  if (!selectedWorkspace.value) return
  if (providerDetectionRunning.value) return

  const silent = Boolean(options.silent)
  providerDetectionRunning.value = true
  providerSyncMode.value = silent ? 'silent' : 'native'
  if (!silent) {
    providerDetectionSummary.value = tr(
      '正在同步本机 CLI 配置（当前 detect 仍可能含 CC Switch 合并项，拆分 API 待后端）。',
      'Syncing local CLI configs (detect may still include merged CC Switch entries until backend split).',
    )
  }

  try {
    const detectedProfiles = await detectLocalProviderProfiles()
    let summaryText = ''
    let firstImportedId = ''

    patchSelectedWorkspace((workspace) => {
      const merged = mergeDetectedProviderProfiles(workspace, detectedProfiles, { pruneSyncedStale: true })
      firstImportedId = merged.firstImportedId
      summaryText = tr(
        `本机同步完成：新增 ${merged.addedCount} · 更新 ${merged.updatedCount} · 清理过期同步 ${merged.prunedCount}（detect ${merged.totalDetected} 条，identityKey 去重）。`,
        `Local sync done: +${merged.addedCount} · ~${merged.updatedCount} · pruned ${merged.prunedCount} stale synced (detect ${merged.totalDetected}, identityKey-deduped).`,
      )
      return merged.workspacePatch
    })

    if (firstImportedId) activeProviderStatsId.value = firstImportedId
    providerSilentSyncedOnce.value = true
    providerDetectionSummary.value = summaryText
    if (!silent) {
      showToast(tr('本机配置已同步', 'Local configs synced'), summaryText)
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`本机同步失败：${message}`, `Local sync failed: ${message}`)
    if (!silent) showToast(tr('本机同步失败', 'Local sync failed'), message)
  } finally {
    providerDetectionRunning.value = false
    providerSyncMode.value = 'idle'
  }
}

/**
 * 从 CC Switch 导入。
 * 后端尚未提供独立 import API：当前仍调用 detect_local_provider_profiles（合并目录），
 * 前端不做假过滤；入口与文案与「同步本机」分开，待后端拆分。
 */
async function importCcSwitchProviderProfiles() {
  if (!selectedWorkspace.value) return
  if (providerDetectionRunning.value) return

  providerDetectionRunning.value = true
  providerSyncMode.value = 'cc-switch'
  providerDetectionSummary.value = tr(
    '正在从 CC Switch 导入（当前与本机 detect 共用合并 API，后端拆分前无法只返回 CC Switch）。',
    'Importing from CC Switch (still shares the merged detect API; cannot return CC Switch-only until backend split).',
  )

  try {
    const detectedProfiles = await detectLocalProviderProfiles()
    let summaryText = ''
    let firstImportedId = ''

    patchSelectedWorkspace((workspace) => {
      const merged = mergeDetectedProviderProfiles(workspace, detectedProfiles, { pruneSyncedStale: true })
      firstImportedId = merged.firstImportedId
      const ccCount = detectedProfiles.filter((item) => item.managedBy === 'cc-switch').length
      summaryText = tr(
        `CC Switch 导入完成：新增 ${merged.addedCount} · 更新 ${merged.updatedCount} · 清理 ${merged.prunedCount}（合并目录共 ${merged.totalDetected}，其中 managedBy=cc-switch ${ccCount}）。`,
        `CC Switch import done: +${merged.addedCount} · ~${merged.updatedCount} · pruned ${merged.prunedCount} (merged catalog ${merged.totalDetected}, managedBy=cc-switch ${ccCount}).`,
      )
      return merged.workspacePatch
    })

    if (firstImportedId) activeProviderStatsId.value = firstImportedId
    providerSilentSyncedOnce.value = true
    providerDetectionSummary.value = summaryText
    showToast(tr('CC Switch 导入完成', 'CC Switch import finished'), summaryText)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    providerDetectionSummary.value = tr(`CC Switch 导入失败：${message}`, `CC Switch import failed: ${message}`)
    showToast(tr('CC Switch 导入失败', 'CC Switch import failed'), message)
  } finally {
    providerDetectionRunning.value = false
    providerSyncMode.value = 'idle'
  }
}

/** @deprecated 保留别名，避免外部残留调用；统一走 syncNative */
async function seedCliProviderProfiles() {
  await syncNativeProviderProfiles({ explicit: true })
}

async function refreshProviderUsageStats() {
  if (!selectedWorkspace.value) return
  if (providerUsageRefreshRunning.value) return
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast(
      tr('浏览器模式不支持实时刷新', 'Browser mode cannot refresh live usage'),
      tr('真实用量刷新需要在 Tauri 桌面模式下读取本机 CC Switch / CLI 数据。', 'Live usage refresh needs Tauri desktop mode to read local CC Switch / CLI data.'),
    )
    return
  }

  providerUsageRefreshRunning.value = true
  managedUsageLoadError.value = ''
  try {
    // 1) 刷新 Provider 档案状态（isActive 只信本机检测 + 归一化）
    const detectedProfiles = await detectLocalProviderProfiles()
    const now = new Date().toISOString()

    patchSelectedWorkspace((workspace) => {
      const nextProfiles = normalizeActiveProviderProfiles((workspace.providerProfiles ?? []).map((profile) => {
        const detected = findDetectedProfileForStoredProfile(detectedProfiles, profile)
        if (!detected) {
          // 检测不到的档案：不要继续假“当前启用”
          return {
            ...profile,
            isActive: false,
            status: profile.status === 'active' ? 'available' : profile.status,
          }
        }
        const canActivate = detected.status !== 'missing' && detected.status !== 'disabled'
        return {
          ...profile,
          status: detected.status,
          isActive: Boolean(detected.isActive) && canActivate,
          identityKey: detected.identityKey || profile.identityKey || null,
          lastDetectedAt: now,
          authSource: detected.authSource || profile.authSource,
          switchCommand: detected.switchCommand || profile.switchCommand,
          defaultModel: detected.defaultModel || profile.defaultModel,
          toolTargets: detected.toolTargets.length ? detected.toolTargets : profile.toolTargets,
          color: detected.color || profile.color,
          note: detected.note || profile.note,
          homepageUrl: detected.homepageUrl ?? profile.homepageUrl ?? null,
          requestBaseUrl: detected.requestBaseUrl ?? profile.requestBaseUrl ?? null,
          configPayload: detected.configPayload ?? profile.configPayload ?? null,
          authPayload: detected.authPayload ?? profile.authPayload ?? null,
          updatedAt: now,
        }
      }))

      return {
        ...workspace,
        providerProfiles: nextProfiles,
        // 保留旧 stats 作为兜底；真正展示优先用 managedUsageLive
        providerUsageStats: nextProfiles.map((profile) => {
          const detected = findDetectedProfileForStoredProfile(detectedProfiles, profile)
          return normalizeProviderUsageStatsForProfile(
            profile,
            (detected && createProviderUsageStatsFromDetection(profile.id, detected))
              ?? workspace.providerUsageStats?.find((stats) => stats.providerProfileId === profile.id)
              ?? createEmptyProviderUsageStatsForProfile(profile.id),
          )
        }),
        updatedAt: now,
      }
    })

    // 2) 实时拉取 managed usage（native + CC Switch proxy_request_logs）
    await loadManagedUsageLive({ reset: true, manual: true })

    const logCount = managedUsageRequestLogs.value.length
    const totalHint = managedUsageTotal.value ? ` / total ${managedUsageTotal.value}` : ''
    showToast(
      tr('统计已刷新', 'Usage refreshed'),
      tr(`已重新读取本机 Provider / Usage（${logCount} 条明细${totalHint}）。`, `Reloaded local Provider / Usage (${logCount} detail rows${totalHint}).`),
    )
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    managedUsageLoadError.value = message
    showToast(tr('统计刷新失败', 'Usage refresh failed'), message)
  } finally {
    providerUsageRefreshRunning.value = false
  }
}

/** 映射 period UI → 后端 RFC3339 startAt/endAt；bucket 由 usageEffectiveBucket 决定 */
function usagePeriodRange(period: typeof usagePeriodFilter.value): { startAt?: string; endAt?: string } {
  const end = new Date()
  const endAt = end.toISOString()
  if (period === 'all') return { endAt }
  if (period === 'custom') {
    const startIso = datetimeLocalToIso(usageCustomStartAt.value)
    const endIso = datetimeLocalToIso(usageCustomEndAt.value)
    return {
      startAt: startIso ?? undefined,
      endAt: endIso ?? endAt,
    }
  }
  if (period === '1h') {
    const start = new Date(end.getTime() - 3_600_000)
    return { startAt: start.toISOString(), endAt }
  }
  if (period === 'today') {
    const start = new Date(end.getFullYear(), end.getMonth(), end.getDate())
    return { startAt: start.toISOString(), endAt }
  }
  if (period === 'month') {
    const start = new Date(end.getFullYear(), end.getMonth(), 1)
    return { startAt: start.toISOString(), endAt }
  }
  const days = period === '7d' ? 7 : period === '30d' ? 30 : 90
  const start = new Date(end.getTime() - days * 86400_000)
  return {
    startAt: start.toISOString(),
    endAt,
  }
}

/**
 * 拉取 managed usage。
 * - reset/默认：刷新 summary/trends/providerStats + 明细第 1 页（替换 logs）
 * - append：仅用 nextCursor 追加 requestLogs，不覆盖聚合
 * - 超时：45s 强制结束 inFlight，避免「刷新中」永久卡住
 */
async function loadManagedUsageLive(options: { append?: boolean; reset?: boolean; manual?: boolean } = {}) {
  if (!("__TAURI_INTERNALS__" in window)) return
  if (options.append && managedUsageInFlight.value) return

  const append = Boolean(options.append)
  if (append && (!managedUsageHasMore.value || !managedUsageNextCursor.value)) return

  // 非 append 且已有请求在飞：标记需要在结束后按最新筛选再拉一次
  if (!append && managedUsageInFlight.value) {
    managedUsageNeedsReload = true
    return
  }

  const range = usagePeriodRange(usagePeriodFilter.value)
  if (usagePeriodFilter.value === 'custom' && !range.startAt && !range.endAt) {
    return
  }
  const appType = usageAppFilter.value === 'all' ? null : usageAppFilter.value
  const providerIds = resolveUsageProviderIdentityKeys()
  // 后端空 provider_ids = 不筛选；选中了但无 identityKey 时必须短路，避免误查全量
  if (Array.isArray(providerIds) && providerIds.length === 0) {
    managedUsageLive.value = {
      summary: {
        totalRequests: 0,
        totalCostUsd: 0,
        totalInputTokens: 0,
        totalOutputTokens: 0,
        totalCacheReadTokens: 0,
        totalCacheCreationTokens: 0,
        cacheHitRate: 0,
      },
      channels: [],
      trends: [],
      providerStats: [],
      modelStats: [],
      requestLogs: [],
      nextCursor: null,
      hasMore: false,
      total: 0,
      updatedAt: new Date().toISOString(),
    }
    managedUsageRequestLogs.value = []
    managedUsageNextCursor.value = null
    managedUsageHasMore.value = false
    managedUsageTotal.value = 0
    managedUsageUpdatedAt.value = managedUsageLive.value.updatedAt
    managedUsageLoadError.value = tr(
      '所选档案缺少 identityKey，无法按契约过滤用量。请先同步本机 Provider。',
      'Selected profiles lack identityKey; usage cannot be filtered by contract. Sync local providers first.',
    )
    return
  }
  const bucket = usageEffectiveBucket.value
  const seq = ++managedUsageRequestSeq
  managedUsageInFlight.value = true
  if (options.manual) usageManualRefreshRunning.value = true

  const timeoutId = window.setTimeout(() => {
    // 超时后作废本请求序号，避免迟到响应覆盖新筛选结果；并解除 inFlight 卡死
    if (seq !== managedUsageRequestSeq) return
    managedUsageRequestSeq += 1
    managedUsageInFlight.value = false
    if (options.manual) usageManualRefreshRunning.value = false
    managedUsageLoadError.value = tr(
      '用量查询超时（可能数据量过大）。请缩小时间范围后重试。',
      'Usage query timed out (dataset may be large). Narrow the range and retry.',
    )
    if (managedUsageNeedsReload && !append) {
      managedUsageNeedsReload = false
      void loadManagedUsageLive({ reset: true })
    }
  }, MANAGED_USAGE_TIMEOUT_MS)

  try {
    const result = await queryManagedUsage({
      appType,
      providerIds,
      // 不再叠 providerName：避免与 providerIds 冲突导致空结果
      providerName: null,
      startAt: range.startAt ?? null,
      endAt: range.endAt ?? null,
      bucket,
      cursor: append ? managedUsageNextCursor.value : null,
      limit: usageRequestLogPageSize,
    })
    if (seq !== managedUsageRequestSeq) return

    if (append) {
      const seen = new Set(managedUsageRequestLogs.value.map((log) => log.id))
      const merged = [...managedUsageRequestLogs.value]
      for (const log of result.requestLogs || []) {
        if (log.id && seen.has(log.id)) continue
        if (log.id) seen.add(log.id)
        merged.push(log)
      }
      managedUsageRequestLogs.value = merged
    } else {
      managedUsageLive.value = {
        ...result,
        modelStats: Array.isArray(result.modelStats) ? result.modelStats : [],
        providerStats: Array.isArray(result.providerStats) ? result.providerStats : [],
        trends: Array.isArray(result.trends) ? result.trends : [],
        channels: Array.isArray(result.channels) ? result.channels : [],
        requestLogs: Array.isArray(result.requestLogs) ? result.requestLogs : [],
      }
      managedUsageRequestLogs.value = managedUsageLive.value.requestLogs
      managedUsageUpdatedAt.value = result.updatedAt || new Date().toISOString()
    }

    managedUsageNextCursor.value = result.nextCursor ?? null
    managedUsageHasMore.value = Boolean(result.hasMore)
    managedUsageTotal.value = typeof result.total === 'number' ? result.total : managedUsageTotal.value
    managedUsageLoadError.value = ''
  } catch (error) {
    if (seq !== managedUsageRequestSeq) return
    managedUsageLoadError.value = error instanceof Error ? error.message : String(error)
  } finally {
    window.clearTimeout(timeoutId)
    if (seq === managedUsageRequestSeq) {
      managedUsageInFlight.value = false
      if (options.manual) usageManualRefreshRunning.value = false
      if (managedUsageNeedsReload && !append) {
        managedUsageNeedsReload = false
        void loadManagedUsageLive({ reset: true })
      }
    }
  }
}

function stopManagedUsagePolling() {
  if (managedUsagePollTimer) {
    window.clearInterval(managedUsagePollTimer)
    managedUsagePollTimer = null
  }
}

function startManagedUsagePolling() {
  stopManagedUsagePolling()
  if (!("__TAURI_INTERNALS__" in window)) return
  if (appSection.value !== 'usage') return
  if (typeof document !== 'undefined' && document.visibilityState === 'hidden') return
  // 60s 盲轮询仅作兜底；主路径靠 usage-log-recorded 事件 invalidate
  managedUsagePollTimer = window.setInterval(() => {
    if (appSection.value !== 'usage') {
      stopManagedUsagePolling()
      return
    }
    if (typeof document !== 'undefined' && document.visibilityState === 'hidden') return
    if (managedUsageInFlight.value) return
    void loadManagedUsageLive({ reset: true })
  }, MANAGED_USAGE_POLL_MS)
}

async function setupManagedUsageEventListeners() {
  if (!("__TAURI_INTERNALS__" in window)) return
  if (!unlistenUsageLogRecorded) {
    try {
      unlistenUsageLogRecorded = await listen('usage-log-recorded', () => {
        if (appSection.value !== 'usage' && appSection.value !== 'providers') return
        if (managedUsageInFlight.value) {
          managedUsageNeedsReload = true
          return
        }
        void loadManagedUsageLive({ reset: true })
      })
    } catch {
      // 非桌面壳忽略
    }
  }
  if (!unlistenModelPricingUpdated) {
    try {
      unlistenModelPricingUpdated = await listen('model-pricing-updated', () => {
        if (appSection.value !== 'usage' && appSection.value !== 'providers') return
        void loadManagedUsageLive({ reset: true })
      })
    } catch {
      // 非桌面壳忽略
    }
  }
}

function teardownManagedUsageEventListeners() {
  unlistenUsageLogRecorded?.()
  unlistenUsageLogRecorded = null
  unlistenModelPricingUpdated?.()
  unlistenModelPricingUpdated = null
}

function providerProfileKey(provider: Pick<ProviderProfile, 'providerKind' | 'profileName' | 'configPath' | 'identityKey'>) {
  const identity = (provider.identityKey || '').trim().toLowerCase()
  if (identity) return `idk::${identity}`
  return [
    provider.providerKind,
    provider.profileName.trim().toLowerCase() || 'default',
    provider.configPath.trim().toLowerCase().replace(/\\/g, '/'),
  ].join('::')
}

function detectedProviderProfileKey(provider: Pick<DetectedProviderProfile, 'providerKind' | 'profileName' | 'configPath' | 'identityKey'>) {
  const identity = (provider.identityKey || '').trim().toLowerCase()
  if (identity) return `idk::${identity}`
  return [
    provider.providerKind,
    provider.profileName.trim().toLowerCase() || 'default',
    provider.configPath.trim().toLowerCase().replace(/\\/g, '/'),
  ].join('::')
}

function normalizeProviderPayload(payload?: string | null) {
  return (payload ?? '').replace(/\r\n/g, '\n').trim()
}

function escapeTomlString(value: string) {
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
}

function extractTomlStringField(raw: string, field: string) {
  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const equalIndex = trimmed.indexOf('=')
    if (equalIndex < 0) continue
    const key = trimmed.slice(0, equalIndex).trim()
    if (key !== field) continue
    return trimmed.slice(equalIndex + 1).trim().replace(/^['"]|['"]$/g, '')
  }
  return ''
}

function extractTomlSection(raw: string, sectionName: string) {
  const header = `[${sectionName}]`
  const lines = raw.split(/\r?\n/)
  let active = false
  const section: string[] = []
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      if (active) break
      active = trimmed === header
      continue
    }
    if (active) section.push(line)
  }
  return section.join('\n')
}

function replaceTopLevelTomlField(raw: string, keyName: string, valueText: string) {
  const lines = raw.split(/\r?\n/)
  let replaced = false
  const nextLines = lines.map((line) => {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) return line
    const equalIndex = trimmed.indexOf('=')
    if (equalIndex < 0) return line
    const key = trimmed.slice(0, equalIndex).trim()
    if (key !== keyName) return line
    replaced = true
    return `${keyName} = "${escapeTomlString(valueText)}"`
  })
  if (!replaced) nextLines.unshift(`${keyName} = "${escapeTomlString(valueText)}"`)
  return nextLines.join('\n')
}

function upsertTomlSectionField(raw: string, sectionName: string, fieldName: string, valueText: string) {
  const lines = raw.split(/\r?\n/)
  const header = `[${sectionName}]`
  let sectionStart = -1
  let sectionEnd = lines.length
  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index].trim()
    if (!(trimmed.startsWith('[') && trimmed.endsWith(']'))) continue
    if (trimmed === header) {
      sectionStart = index
      continue
    }
    if (sectionStart >= 0) {
      sectionEnd = index
      break
    }
  }

  if (sectionStart < 0) {
    const suffix = raw.trim() ? '\n\n' : ''
    return `${raw.trimEnd()}${suffix}${header}\n${fieldName} = "${escapeTomlString(valueText)}"\n`
  }

  let replaced = false
  const sectionLines = lines.slice(sectionStart + 1, sectionEnd).map((line) => {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) return line
    const equalIndex = trimmed.indexOf('=')
    if (equalIndex < 0) return line
    const key = trimmed.slice(0, equalIndex).trim()
    if (key !== fieldName) return line
    replaced = true
    return `${fieldName} = "${escapeTomlString(valueText)}"`
  })
  if (!replaced) sectionLines.unshift(`${fieldName} = "${escapeTomlString(valueText)}"`)
  return [
    ...lines.slice(0, sectionStart + 1),
    ...sectionLines,
    ...lines.slice(sectionEnd),
  ].join('\n')
}

function baseUrlOrigin(baseUrl: string) {
  try {
    return new URL(baseUrl).origin
  } catch {
    return ''
  }
}

function extractCodexBaseUrlFromPayload(payload?: string | null) {
  const raw = normalizeProviderPayload(payload)
  if (!raw) return ''
  const activeProvider = extractTomlStringField(raw, 'model_provider')
  if (activeProvider) {
    const sectionBaseUrl = extractTomlStringField(extractTomlSection(raw, `model_providers.${activeProvider}`), 'base_url')
    if (sectionBaseUrl) return sectionBaseUrl
  }
  return extractTomlStringField(raw, 'base_url')
}

function buildCodexPayloadFromForm() {
  const profileName = providerForm.profileName.trim() || 'default'
  const providerName = providerForm.name.trim() || profileName
  const model = providerForm.defaultModel.trim() || 'gpt-5'
  const requestBaseUrl = providerForm.requestBaseUrl.trim()
  let payload = normalizeProviderPayload(providerForm.configPayload)

  if (!payload) {
    const lines = [
      `model_provider = "${escapeTomlString(profileName)}"`,
      `model = "${escapeTomlString(model)}"`,
      '',
      `[model_providers.${profileName}]`,
      `name = "${escapeTomlString(providerName)}"`,
    ]
    if (requestBaseUrl) {
      lines.push(`base_url = "${escapeTomlString(requestBaseUrl)}"`)
      lines.push('wire_api = "responses"')
    }
    return lines.join('\n')
  }

  payload = replaceTopLevelTomlField(payload, 'model_provider', profileName)
  payload = replaceTopLevelTomlField(payload, 'model', model)
  payload = upsertTomlSectionField(payload, `model_providers.${profileName}`, 'name', providerName)
  if (requestBaseUrl) {
    payload = upsertTomlSectionField(payload, `model_providers.${profileName}`, 'base_url', requestBaseUrl)
    payload = upsertTomlSectionField(payload, `model_providers.${profileName}`, 'wire_api', 'responses')
  }
  return payload
}

function parseEnvPayload(payload?: string | null) {
  const raw = normalizeProviderPayload(payload)
  const lines = raw ? raw.split('\n') : []
  const pairs = new Map<string, string>()
  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const equalIndex = trimmed.indexOf('=')
    if (equalIndex < 0) continue
    const key = trimmed.slice(0, equalIndex).trim()
    const value = trimmed.slice(equalIndex + 1).trim().replace(/^['"]|['"]$/g, '')
    if (key) pairs.set(key, value)
  }
  return pairs
}

function serializeEnvPayload(map: Map<string, string>) {
  return Array.from(map.entries())
    .map(([key, value]) => `${key}=${value}`)
    .join('\n')
}

function buildGeminiAuthPayloadFromForm() {
  const env = parseEnvPayload(providerForm.authPayload)
  const requestBaseUrl = providerForm.requestBaseUrl.trim()
  if (requestBaseUrl) {
    env.set('GOOGLE_GEMINI_BASE_URL', requestBaseUrl)
  }
  return serializeEnvPayload(env)
}

function buildGeminiSettingsPayloadFromForm(authPayload: string) {
  const raw = normalizeProviderPayload(providerForm.configPayload)
  let parsed: Record<string, any> = {}
  if (raw) {
    try {
      parsed = JSON.parse(raw)
    } catch {
      parsed = {}
    }
  }
  if (!parsed.security || typeof parsed.security !== 'object') parsed.security = {}
  if (!parsed.security.auth || typeof parsed.security.auth !== 'object') parsed.security.auth = {}
  parsed.security.auth.selectedType = authPayload.trim() ? 'gemini-api-key' : 'oauth-personal'
  return JSON.stringify(parsed, null, 2)
}

function providerPayloadMatchesDetection(
  provider: Pick<ProviderProfile, 'providerKind' | 'configPayload' | 'authPayload'>,
  detected: Pick<DetectedProviderProfile, 'providerKind' | 'configPayload' | 'authPayload'>,
) {
  if (provider.providerKind !== detected.providerKind) return false
  const providerConfig = normalizeProviderPayload(provider.configPayload)
  const detectedConfig = normalizeProviderPayload(detected.configPayload)
  if (!providerConfig || !detectedConfig || providerConfig !== detectedConfig) return false
  const providerAuth = normalizeProviderPayload(provider.authPayload)
  const detectedAuth = normalizeProviderPayload(detected.authPayload)
  return providerAuth === detectedAuth
}

function findExistingProviderIndexByDetection(providers: ProviderProfile[], detected: DetectedProviderProfile) {
  const detectedKey = (detected.identityKey || '').trim()
  if (detectedKey) {
    const byIdentity = providers.findIndex((provider) => (provider.identityKey || '').trim() === detectedKey)
    if (byIdentity >= 0) return byIdentity
  }
  const payloadMatchIndex = providers.findIndex((provider) => providerPayloadMatchesDetection(provider, detected))
  if (payloadMatchIndex >= 0) return payloadMatchIndex
  return providers.findIndex((provider) => providerProfileKey(provider) === detectedProviderProfileKey(detected))
}

function findDetectedProfileForStoredProfile(
  detectedProfiles: DetectedProviderProfile[],
  profile: ProviderProfile,
) {
  const profileKey = (profile.identityKey || '').trim()
  if (profileKey) {
    const byIdentity = detectedProfiles.find((detected) => (detected.identityKey || '').trim() === profileKey)
    if (byIdentity) return byIdentity
  }
  return detectedProfiles.find((detected) => providerPayloadMatchesDetection(profile, detected))
    ?? detectedProfiles.find((detected) => detectedProviderProfileKey(detected) === providerProfileKey(profile))
}

function createProviderProfileFromDetection(workspaceId: string, detected: DetectedProviderProfile) {
  // Frontend Contract: 绝不因“列表第一个/空列表”隐式 active；只信后端 isActive
  const canActivate = detected.status !== 'missing' && detected.status !== 'disabled'
  const isActive = canActivate && Boolean(detected.isActive)
  return createProviderProfileRecord({
    workspaceId,
    name: detected.name,
    providerKind: detected.providerKind,
    profileName: detected.profileName || 'default',
    configPath: detected.configPath || defaultConfigPathForProvider(detected.providerKind),
    configScope: detected.configScope,
    managedBy: detected.managedBy,
    authSource: detected.authSource,
    switchCommand: detected.switchCommand || '',
    defaultModel: detected.defaultModel,
    toolTargets: detected.toolTargets.length ? detected.toolTargets : parseProviderToolTargets(defaultProviderTargetsText(detected.providerKind)),
    status: isActive ? 'active' : (detected.status === 'active' ? 'available' : detected.status),
    identityKey: detected.identityKey || null,
    color: detected.color || providerKindColor(detected.providerKind),
    note: detected.note,
    homepageUrl: detected.homepageUrl ?? (detected.providerKind === 'codex' ? baseUrlOrigin(extractCodexBaseUrlFromPayload(detected.configPayload)) || null : null),
    requestBaseUrl: detected.requestBaseUrl ?? (detected.providerKind === 'codex' ? extractCodexBaseUrlFromPayload(detected.configPayload) || null : null),
    configPayload: detected.configPayload ?? null,
    authPayload: detected.authPayload ?? null,
    isDefault: false,
    isActive,
  })
}

function createProviderUsageStatsFromDetection(providerProfileId: string, detected: DetectedProviderProfile): ProviderUsageStats | null {
  if (!detected.usageStats) return null

  return {
    providerProfileId,
    summary: normalizeProviderUsageSummary(detected.usageStats.summary),
    trends: (detected.usageStats.trends || []).map((point) => ({ ...point })),
    requestLogs: (detected.usageStats.requestLogs || []).map((log, index) => ({
      ...log,
      id: log.id || `${providerProfileId}-cc-switch-${index}`,
      providerProfileId,
      appType: log.appType || providerKindToAppType(detected.providerKind),
      model: log.model || providerDefaultModel(detected),
      dataSource: log.dataSource || 'cc_switch_db',
    })),
  }
}

function normalizeActiveProviderProfiles(profiles: ProviderProfile[]): ProviderProfile[] {
  const activeByKind = new Set<string>()
  return profiles.map((profile) => {
    if (profile.isActive && !providerCanBeActivated(profile)) {
      return {
        ...profile,
        isActive: false,
        status: profile.status === 'active' ? 'available' : profile.status,
      }
    }

    if (profile.isActive && !activeByKind.has(profile.providerKind)) {
      activeByKind.add(profile.providerKind)
      return {
        ...profile,
        status: (profile.status === 'missing' || profile.status === 'disabled' ? profile.status : 'active') as ProviderProfile['status'],
      }
    }

    if (profile.isActive) {
      return {
        ...profile,
        isActive: false,
        status: (profile.status === 'missing' || profile.status === 'disabled' ? profile.status : 'available') as ProviderProfile['status'],
      }
    }

    return profile
  })
}

function providerSupportsBuiltinActivation(kind: ProviderProfile['providerKind']) {
  return kind === 'codex' || kind === 'gemini-cli' || kind === 'claude-code' || kind === 'hermes'
}

function providerCanBeActivated(provider: Pick<ProviderProfile, 'providerKind' | 'status' | 'switchCommand' | 'configPayload'>) {
  if (provider.status === 'missing' || provider.status === 'disabled') return false
  if (providerSupportsBuiltinActivation(provider.providerKind)) {
    return Boolean(normalizeProviderPayload(provider.configPayload))
  }
  return Boolean(provider.switchCommand.trim())
}

function providerActivationLabel(provider: Pick<ProviderProfile, 'providerKind' | 'status' | 'switchCommand' | 'configPayload'>) {
  if (provider.status === 'missing' || provider.status === 'disabled') return t('provider.configMissing')
  if (providerCanBeActivated(provider)) return t('provider.activateCurrent')
  return tr('写回未接通', 'Writeback unavailable')
}

function providerKindToAppType(kind: ProviderProfile['providerKind']): ProviderRequestLog['appType'] {
  if (kind === 'claude-code') return 'claude'
  if (kind === 'gemini-cli') return 'gemini'
  if (kind === 'opencode') return 'opencode'
  if (kind === 'hermes') return 'hermes'
  return 'codex'
}

function providerDefaultModel(provider: Pick<ProviderProfile, 'providerKind' | 'defaultModel'>) {
  if (provider.defaultModel.trim()) return provider.defaultModel.trim()
  if (provider.providerKind === 'claude-code') return 'claude-sonnet-4'
  if (provider.providerKind === 'gemini-cli') return 'gemini-2.5-pro'
  if (provider.providerKind === 'opencode') return 'provider/default'
  if (provider.providerKind === 'hermes') return 'anthropic/claude-opus-4-8'
  return 'gpt-5'
}

function emptyProviderUsageSummary() {
  return {
    totalRequests: 0,
    totalCostUsd: 0,
    totalInputTokens: 0,
    totalOutputTokens: 0,
    totalCacheReadTokens: 0,
    totalCacheCreationTokens: 0,
    cacheHitRate: 0,
  }
}

function normalizeProviderUsageSummary(summary?: Partial<ProviderUsageStats['summary']>) {
  return {
    totalRequests: summary?.totalRequests ?? 0,
    totalCostUsd: summary?.totalCostUsd ?? 0,
    totalInputTokens: summary?.totalInputTokens ?? 0,
    totalOutputTokens: summary?.totalOutputTokens ?? 0,
    totalCacheReadTokens: summary?.totalCacheReadTokens ?? 0,
    totalCacheCreationTokens: summary?.totalCacheCreationTokens ?? 0,
    cacheHitRate: summary?.cacheHitRate ?? 0,
  }
}

function recalculateProviderUsageSummary(requestLogs: ProviderRequestLog[]) {
  if (!requestLogs.length) return emptyProviderUsageSummary()

  const totals = requestLogs.reduce((result, log) => {
    result.totalRequests += 1
    result.totalCostUsd += log.costUsd
    result.totalInputTokens += log.inputTokens
    result.totalOutputTokens += log.outputTokens
    result.totalCacheReadTokens += log.cacheReadTokens
    result.totalCacheCreationTokens += log.cacheCreationTokens
    return result
  }, emptyProviderUsageSummary())
  const cacheableTokens = totals.totalInputTokens + totals.totalCacheReadTokens + totals.totalCacheCreationTokens

  return {
    ...totals,
    totalCostUsd: Number(totals.totalCostUsd.toFixed(4)),
    cacheHitRate: cacheableTokens > 0 ? totals.totalCacheReadTokens / cacheableTokens : 0,
  }
}

function normalizeProviderUsageStatsForProfile(provider: ProviderProfile, stats: ProviderUsageStats): ProviderUsageStats {
  if (provider.status === 'missing' || provider.status === 'disabled') {
    return createEmptyProviderUsageStatsForProfile(provider.id)
  }

  const appType = providerKindToAppType(provider.providerKind)
  const model = providerDefaultModel(provider)
  const dataSource = provider.managedBy === 'cc-switch' ? 'cc_switch_profile' : `${appType}_session`
  const requestLogs = (stats.requestLogs || []).map((log, index) => ({
    ...log,
    id: log.id || `${provider.id}-usage-${index}`,
    providerProfileId: provider.id,
    appType: log.appType || appType,
    model: log.model || model,
    dataSource: log.dataSource || dataSource,
  }))

  return {
    providerProfileId: provider.id,
    summary: stats.summary?.totalRequests ? normalizeProviderUsageSummary(stats.summary) : recalculateProviderUsageSummary(requestLogs),
    trends: (stats.trends || []).map((point) => ({ ...point })),
    requestLogs,
  }
}

function upsertProviderUsageStats(statsList: ProviderUsageStats[], provider: ProviderProfile) {
  const nextStats = normalizeProviderUsageStatsForProfile(
    provider,
    statsList.find((stats) => stats.providerProfileId === provider.id) ?? createEmptyProviderUsageStatsForProfile(provider.id),
  )
  const existingIndex = statsList.findIndex((stats) => stats.providerProfileId === provider.id)

  if (existingIndex < 0) {
    return [...statsList, nextStats]
  }

  return statsList.map((stats, index) => index === existingIndex ? nextStats : stats)
}

function upsertProviderUsageStatsForProfiles(statsList: ProviderUsageStats[], providers: ProviderProfile[]) {
  const providerIds = new Set(providers.map((provider) => provider.id))
  return providers.reduce<ProviderUsageStats[]>((result, provider) => {
    return upsertProviderUsageStats(result, provider)
  }, statsList.filter((stats) => providerIds.has(stats.providerProfileId)))
}

function createEmptyProviderUsageStatsForProfile(providerProfileId: string): ProviderUsageStats {
  return {
    providerProfileId,
    summary: emptyProviderUsageSummary(),
    trends: [],
    requestLogs: [],
  }
}

function snapshotName(workspaceName = tr('工作区', 'Workspace')) {
  const date = new Date()
  const pad = (value: number) => String(value).padStart(2, '0')
  return currentLocale.value === 'zh-CN'
    ? `${workspaceName} · 现场 ${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
    : `${workspaceName} · Snapshot ${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
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

  showToast(
    tr('现场已保存', 'Snapshot saved'),
    tr(
      `${snapshot.name} · ${workspace.tabs.length} 个项目 · ${totalWorkspaceSessions(workspace)} 个终端`,
      `${snapshot.name} · ${workspace.tabs.length} projects · ${totalWorkspaceSessions(workspace)} terminals`,
    ),
  )
}

function restoreDefaultWorkspaceSnapshot() {
  const snapshot = defaultWorkspaceSnapshot.value
  if (!snapshot) {
    showToast(tr('暂无现场快照', 'No snapshot available'), tr('请先保存一次当前工作现场。', 'Save the current workspace once before restoring a snapshot.'))
    return
  }

  requestConfirm({
    title: tr(`恢复现场「${snapshot.name}」`, `Restore snapshot "${snapshot.name}"`),
    description: tr('将恢复该快照中的项目、Pane 分栏、终端标签与焦点信息，不会自动执行任何命令。', 'Restore the projects, pane layout, terminal tabs, and focus captured in this snapshot without running any commands automatically.'),
    confirmLabel: tr('恢复现场', 'Restore snapshot'),
    variant: 'primary',
    details: [
      tr(`工作区：${selectedWorkspace.value?.name || snapshot.workspaceId}`, `Workspace: ${selectedWorkspace.value?.name || snapshot.workspaceId}`),
      tr(`项目数：${snapshot.tabsState.length}`, `Projects: ${snapshot.tabsState.length}`),
      tr(`终端数：${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`, `Terminals: ${snapshot.tabsState.reduce((count, tab) => count + countPaneSessions(tab.panes), 0)}`),
    ],
    action: () => restoreWorkspaceSnapshot(snapshot.id),
  })
}

function restoreActiveTabFromDefaultSnapshot() {
  if (!activeRuntimeTab.value) return
  const snapshot = defaultWorkspaceSnapshot.value
  if (!snapshot) {
    showToast(tr('暂无现场快照', 'No snapshot available'), tr('请先保存一次当前工作现场。', 'Save the current workspace once before restoring a snapshot.'))
    return
  }

  const snapshotTab = snapshot.tabsState.find((tab) => tab.id === activeRuntimeTab.value?.id)
  if (!snapshotTab) {
    showToast(tr('无法恢复当前项目', 'Cannot restore current project'), tr('最近现场中没有找到同名项目。', 'No project with the same name was found in the latest snapshot.'))
    return
  }

  requestConfirm({
    title: tr(`恢复项目「${snapshotTab.name}」`, `Restore project "${snapshotTab.name}"`),
    description: tr('只恢复当前项目的 Pane 分栏和终端标签，不影响同工作区内其他项目。', 'Restore only the pane layout and terminal tabs for this project without affecting other projects in the workspace.'),
    confirmLabel: tr('恢复项目', 'Restore project'),
    variant: 'primary',
    details: [
      tr(`工作区：${selectedWorkspace.value?.name || snapshot.workspaceId}`, `Workspace: ${selectedWorkspace.value?.name || snapshot.workspaceId}`),
      tr(`项目：${snapshotTab.name}`, `Project: ${snapshotTab.name}`),
      tr(`Pane 数：${countLeafPanes(snapshotTab.panes)}`, `Panes: ${countLeafPanes(snapshotTab.panes)}`),
      tr(`终端数：${countPaneSessions(snapshotTab.panes)}`, `Terminals: ${countPaneSessions(snapshotTab.panes)}`),
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
  showToast(tr('现场已恢复', 'Snapshot restored'), tr(`${snapshot.name} · 已恢复整个工作区布局`, `${snapshot.name} · Full workspace layout restored`))
  void applyCommandRestoreStrategy(tr('工作区现场', 'Workspace snapshot'), { silentNoop: true })
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
  showToast(tr('项目布局已恢复', 'Project layout restored'), tr(`已恢复：${snapshotTab.name}`, `Restored: ${snapshotTab.name}`))
  void applyCommandRestoreStrategy(tr('当前项目', 'Current project'), { silentNoop: true })
}

function cloneSnapshotTabs(tabs: WorkspaceTab[], workspaceId: string, updatedAt: string) {
  return tabs.map((tab, index) => ({ ...cloneSnapshotTab(tab, workspaceId, updatedAt), order: index }))
}

function cloneSnapshotTab(tab: WorkspaceTab, workspaceId: string, updatedAt: string): WorkspaceTab {
  return {
    ...tab,
    workspaceId,
    panes: clonePaneTreeForRestore(tab.panes, tab),
    updatedAt,
  }
}

function clonePaneSessionForRestore(
  pane: PaneNode,
  session: PaneTerminalSession | undefined,
  tab: WorkspaceTab | undefined,
  fallbackIndex = 0,
): PaneTerminalSession {
  const fallbackName = fallbackIndex === 0 ? pane.name : `${pane.name} · ${fallbackIndex + 1}`
  return {
    id: createId('session'),
    name: session?.name || fallbackName,
    pathLabel: session?.pathLabel || pane.pathLabel,
    terminalEntryId: session?.terminalEntryId ?? pane.terminalEntryId ?? null,
    status: 'idle',
    aiCliKind: null,
    lastAiCliKind: session?.lastAiCliKind ?? session?.aiCliKind ?? null,
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

function clonePaneTreeForRestore(panes: PaneNode[], tab?: WorkspaceTab): PaneNode[] {
  return panes.map((pane) => {
    const nextChildren = pane.children?.length ? clonePaneTreeForRestore(pane.children, tab) : []
    const nextSessions = nextChildren.length
      ? []
      : (pane.sessions?.length
          ? pane.sessions.map((session, index) => clonePaneSessionForRestore(pane, session, tab, index))
          : [clonePaneSessionForRestore(pane, undefined, tab, 0)])

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
      clearRuntimeSessionOverlay(sessionId)
      sessionOutputTailBySession.delete(sessionId)
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
      showToast(tr('恢复命令未触发', 'No restore command triggered'), tr('当前可见项目中没有符合启动策略的默认命令。', 'No default commands matching the restore strategy were found in the currently visible projects.'))
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
      strategy === 'execute' ? tr('恢复命令已处理', 'Restore commands processed') : tr('恢复命令已预填', 'Restore commands prefilled'),
      currentLocale.value === 'zh-CN'
        ? `${scopeLabel}：成功 ${successCount} 个${failedCount ? `，失败 ${failedCount} 个` : ''}`
        : `${scopeLabel}: ${successCount} succeeded${failedCount ? `, ${failedCount} failed` : ''}`,
    )
    return
  }

  if (failedCount) {
    showToast(tr('恢复命令失败', 'Restore commands failed'), tr('终端尚未准备完成，稍后可用闪电入口手动插入。', 'The terminal is not ready yet. Use the lightning action later to insert commands manually.'))
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
        aiCliKind: null,
        lastAiCliKind: null,
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
  showToast(tr('已新增分屏 Pane', 'Split pane added'), tr(`当前标签页新增：${nextPane.name}`, `Added to current tab: ${nextPane.name}`))
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
      showToast(tr('已切换到现有终端', 'Switched to existing terminal'), tr(`当前配置已定位：${boundEntry.name}`, `Current config located: ${boundEntry.name}`))
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
    aiCliKind: null,
    lastAiCliKind: null,
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
  showToast(tr('终端已新增', 'Terminal added'), tr(`已在当前分组中新建：${nextSession.name}`, `Created in current group: ${nextSession.name}`))
}

function removePaneSession(paneId: string, sessionId: string) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const targetPane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!targetPane) return

  const sessions = paneSessions(targetPane)
  const targetSession = sessions.find((session) => session.id === sessionId)
  if (!targetSession) return

  requestConfirm({
    title: tr(`关闭终端「${targetSession.name}」`, `Close terminal "${targetSession.name}"`),
    description: tr('关闭后将结束当前终端会话，Pane 分组会保留。', 'Closing it will end the current terminal session while keeping the pane group.'),
    confirmLabel: tr('确认关闭', 'Close terminal'),
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

      clearRuntimeSessionOverlay(sessionId)
      void destroyTerminalRuntime(sessionId).catch(() => undefined)
      showToast(tr('终端已关闭', 'Terminal closed'), tr(`已关闭：${targetSession.name}`, `Closed: ${targetSession.name}`))
    },
  })
}

function reloadPaneSession(paneId: string, sessionId: string) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return

  setRuntimeActiveSessionId(paneId, sessionId)
  activeRuntimePaneId.value = paneId
  clearRuntimeSessionOverlay(sessionId)
  sessionOutputTailBySession.delete(sessionId)
  sessionAttentionNotificationState.delete(sessionId)
  clearSessionAttention(sessionId, { silent: true })

  terminalReloadVersions.value = {
    ...terminalReloadVersions.value,
    [sessionId]: (terminalReloadVersions.value[sessionId] ?? 0) + 1,
  }

  void destroyTerminalRuntime(sessionId)
    .catch(() => undefined)
    .finally(() => showToast(tr('终端已重新加载', 'Terminal reloaded'), located.session.name))
}

const runtimeTabMenuItems = computed<PopoverItem[]>(() => {
  if (!activeRuntimeTabMenuId.value || !selectedWorkspace.value) return []

  const tab = selectedWorkspace.value.tabs.find((item) => item.id === activeRuntimeTabMenuId.value)
  if (!tab) return []

  return [
    {
      label: tr('保存工作现场', 'Save workspace snapshot'),
      icon: 'copy',
      description: tr('保存当前工作区的 Tab、Pane、终端标签与焦点。', 'Save the current workspace tabs, panes, terminal tabs, and focus state.'),
      onClick: () => {
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
        saveCurrentWorkspaceSnapshot()
      },
    },
    {
      label: tr('恢复当前项目布局', 'Restore current project layout'),
      icon: 'refresh',
      description: defaultWorkspaceSnapshot.value ? tr(`从最近现场恢复：${defaultWorkspaceSnapshot.value.name}`, `Restore from latest snapshot: ${defaultWorkspaceSnapshot.value.name}`) : tr('暂无可恢复的现场快照。', 'No snapshot is available to restore.'),
      onClick: () => {
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
        restoreActiveTabFromDefaultSnapshot()
      },
    },
    {
      label: tr('重命名项目', 'Rename project'),
      icon: 'edit',
      description: tr('修改当前项目名称。', 'Rename the current project.'),
      onClick: () => {
        openTabRenameModal(tab.id)
        activeRuntimeTabMenuId.value = null
        activeRuntimeTabMenuPosition.value = null
      },
    },
    {
      label: tr('删除项目', 'Delete project'),
      icon: 'trash',
      description: tr('删除当前项目及其下所有 Pane。', 'Delete the current project and every pane inside it.'),
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
      label: tr('保存工作现场', 'Save workspace snapshot'),
      icon: 'copy',
      description: tr('保存当前工作区的完整布局快照。', 'Save a full layout snapshot for the current workspace.'),
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
      label: tr('恢复该项目布局', 'Restore this project layout'),
      icon: 'refresh',
      description: (workspace.snapshots ?? []).length ? tr('从最近现场恢复这个项目。', 'Restore this project from the latest snapshot.') : tr('暂无可恢复的现场快照。', 'No snapshot is available to restore.'),
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
      label: tr('重命名项目', 'Rename project'),
      icon: 'edit',
      description: tr('修改当前项目名称。', 'Rename the current project.'),
      onClick: () => {
        openExplorerTabRename(workspace.id, tab.id)
        activeExplorerProjectMenuId.value = null
        activeExplorerProjectWorkspaceId.value = null
        activeExplorerProjectMenuPosition.value = null
      },
    },
    {
      label: tr('删除项目', 'Delete project'),
      icon: 'trash',
      description: tr('删除当前项目及其下所有 Pane。', 'Delete the current project and every pane inside it.'),
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
  showToast(tr('终端标签已复制', 'Terminal tab duplicated'), tr(`已在当前 Pane 内新增：${nextSession.name}`, `Added in current pane: ${nextSession.name}`))
}

function removePane(paneId: string) {
  if (!selectedWorkspace.value || !activeRuntimeTab.value) return

  const pane = findPaneById(activeRuntimeTab.value.panes, paneId)
  if (!pane) return

  requestConfirm({
    title: tr(`删除 Pane「${pane.name}」`, `Delete pane "${pane.name}"`),
    description: tr('删除后会移除当前 Pane 分组及其中的所有终端会话。', 'Deleting it will remove the current pane group and all terminal sessions inside it.'),
    confirmLabel: tr('确认删除', 'Delete pane'),
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
        clearRuntimeSessionOverlay(sessionId)
        sessionOutputTailBySession.delete(sessionId)
        void destroyTerminalRuntime(sessionId).catch(() => undefined)
      })
      setRuntimeActiveSessionId(paneId, null)

      showToast(tr('Pane 已删除', 'Pane deleted'), tr(`已移除：${pane.name}`, `Removed: ${pane.name}`))
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
      showToast(tr('已切换到现有终端', 'Switched to existing terminal'), tr(`当前配置已定位：${targetEntry.name}`, `Current config located: ${targetEntry.name}`))
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
    showToast(tr('Pane 已绑定配置', 'Pane bound to config'), tr(`当前 Pane 已绑定：${targetEntry.name}`, `Current pane bound to: ${targetEntry.name}`))
    return
  }

  showToast(tr('Pane 已清空绑定', 'Pane binding cleared'), tr('当前 Pane 已恢复为独立空白终端。', 'The current pane is now an independent blank terminal again.'))
}

function openPaneDirectory(pane: PaneNode) {
  const path = terminalSessionWorkingDirectory(pane)
  void invoke('open_directory', { path })
    .then(() => {
      showToast(tr('目录已打开', 'Directory opened'), path)
    })
    .catch((error) => {
      const message = error instanceof Error ? error.message : String(error)
      showToast(tr('打开失败', 'Open failed'), message)
    })
}

async function copyPanePath(pane: PaneNode) {
  const path = terminalSessionWorkingDirectory(pane)

  try {
    await navigator.clipboard.writeText(path)
    showToast(tr('路径已复制', 'Path copied'), path)
  } catch {
    showToast(tr('复制失败', 'Copy failed'), tr('当前浏览器环境未允许写入剪贴板。', 'Clipboard access is not allowed in the current browser environment.'))
  }
}

async function copyPaneCommand(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) {
    showToast(tr('未绑定运行配置', 'Run config not bound'), tr('当前 Pane 还没有绑定运行配置，请先绑定一个运行配置。', 'The current pane has no bound run config yet. Bind one first.'))
    return
  }
  const command = mode === 'default'
    ? entry.defaultCommand?.trim()
    : entry.lastCommand?.trim()

  if (!command) {
    showToast(tr('暂无命令', 'No command available'), mode === 'default' ? tr('当前 Pane 未设置默认命令。', 'No default command is set for the current pane.') : tr('当前 Pane 还没有历史命令。', 'The current pane has no command history yet.'))
    return
  }

  try {
    await navigator.clipboard.writeText(command)
    showToast(mode === 'default' ? tr('默认命令已复制', 'Default command copied') : tr('最后命令已复制', 'Last command copied'), command)
  } catch {
    showToast(tr('复制失败', 'Copy failed'), tr('当前浏览器环境未允许写入剪贴板。', 'Clipboard access is not allowed in the current browser environment.'))
  }
}

async function insertPaneCommand(pane: PaneNode, mode: 'default' | 'last') {
  const entry = entryById(pane.terminalEntryId)
  if (!entry) {
    showToast(tr('未绑定运行配置', 'Run config not bound'), tr('当前 Pane 还没有绑定运行配置，请先绑定一个运行配置。', 'The current pane has no bound run config yet. Bind one first.'))
    return
  }
  const command = mode === 'default'
    ? entry.defaultCommand?.trim()
    : entry.lastCommand?.trim()

  if (!command) {
    showToast(tr('暂无命令', 'No command available'), mode === 'default' ? tr('当前 Pane 未设置默认命令。', 'No default command is set for the current pane.') : tr('当前 Pane 还没有历史命令。', 'The current pane has no command history yet.'))
    return
  }

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast(tr('终端未就绪', 'Terminal not ready'), tr('当前 Pane 还没有可写入的终端会话。', 'The current pane has no writable terminal session yet.'))
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, command)
    showToast(tr('命令已插入', 'Command inserted'), command)
  } catch {
    showToast(tr('插入失败', 'Insert failed'), tr('当前终端尚未准备完成，稍后再试。', 'The current terminal is not ready yet. Try again later.'))
  }
}

async function insertPaneText(pane: PaneNode, text: string) {
  const command = text.trim()
  if (!command) return

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast(tr('终端未就绪', 'Terminal not ready'), tr('当前 Pane 还没有可写入的终端会话。', 'The current pane has no writable terminal session yet.'))
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, command)
    showToast(tr('命令已插入', 'Command inserted'), command)
  } catch {
    showToast(tr('插入失败', 'Insert failed'), tr('当前终端尚未准备完成，稍后再试。', 'The current terminal is not ready yet. Try again later.'))
  }
}

async function executePaneText(pane: PaneNode, text: string) {
  const command = text.trim()
  if (!command) return

  const sessionId = pane.activeSessionId ?? paneSessions(pane)[0]?.id
  if (!sessionId) {
    showToast(tr('终端未就绪', 'Terminal not ready'), tr('当前 Pane 还没有可执行的终端会话。', 'The current pane has no executable terminal session yet.'))
    return
  }

  try {
    activeRuntimePaneId.value = pane.id
    const mounted = await waitForTerminalRuntimeMount(sessionId)
    if (!mounted) throw new Error('terminal_not_mounted')
    await ensureTerminalReady(sessionId)
    await writeTerminalText(sessionId, `${command}\r`)
    recordSessionCommand(sessionId, command)
    showToast(tr('命令已执行', 'Command executed'), command)
  } catch {
    showToast(tr('执行失败', 'Execution failed'), tr('当前终端尚未准备完成，稍后再试。', 'The current terminal is not ready yet. Try again later.'))
  }
}

async function copyCommandText(command: string) {
  const value = command.trim()
  if (!value) return

  try {
    await navigator.clipboard.writeText(value)
    showToast(tr('命令已复制', 'Command copied'), value)
  } catch {
    showToast(tr('复制失败', 'Copy failed'), tr('当前环境未允许写入剪贴板。', 'Clipboard access is not allowed in the current environment.'))
  }
}

async function insertCommandToActivePane(command: string) {
  const value = command.trim()
  if (!value) return
  if (!(appSection.value === 'workspace' && workspaceView.value === 'runtime')) {
    await copyCommandText(value)
    showToast(tr('已先复制命令', 'Command copied first'), tr('请先进入运行态并选中一个终端，再回填到当前输入框。', 'Enter runtime mode and select a terminal first, then insert the command into the current input.'))
    return
  }
  const pane = activeRuntimeTab.value ? findPaneById(activeRuntimeTab.value.panes, activeRuntimePaneId.value) : null

  if (!pane) {
    await copyCommandText(value)
    showToast(tr('已先复制命令', 'Command copied first'), tr('当前没有选中终端。进入运行态后可手动粘贴。', 'No terminal is selected right now. You can paste it manually after entering runtime mode.'))
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
  showToast(tr('已先复制命令', 'Command copied first'), tr('当前没有选中终端。若要回填，请先进入运行态并选中一个终端。', 'No terminal is selected right now. Enter runtime mode and select a terminal first if you want to insert it.'))
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
    pushSection('default', tr('默认命令', 'Default command'), [entry.defaultCommand])
    pushSection('last', tr('最后命令', 'Last command'), [entry.lastCommand])
    pushSection('favorite', tr('收藏命令', 'Favorite commands'), entry.favoriteCommands ?? [])
    pushSection('history', tr('最近命令', 'Recent commands'), entry.commandHistory ?? [])
  }

  pushSection('workspace', entry ? tr('工作区最近', 'Workspace recent') : tr('最近命令', 'Recent commands'), workspaceRecentCommands(), Boolean(entry))
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
    showToast(tr('无法收藏命令', 'Cannot favorite command'), tr('请先给当前终端绑定一个运行配置。', 'Bind a run config to the current terminal first.'))
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

  showToast(exists ? tr('已取消收藏', 'Removed from favorites') : tr('命令已收藏', 'Command favorited'), value)
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

function providerKindLabel(kind?: ProviderProfile['providerKind']) {
  if (kind === 'claude-code') return 'Claude Code'
  if (kind === 'gemini-cli') return 'Gemini CLI'
  if (kind === 'opencode') return 'OpenCode'
  if (kind === 'hermes') return 'Hermes'
  if (kind === 'custom-cli') return t('provider.customCli')
  return 'Codex CLI'
}

function providerKindShortLabel(kind?: ProviderProfile['providerKind']) {
  if (kind === 'claude-code') return 'CC'
  if (kind === 'gemini-cli') return 'GM'
  if (kind === 'opencode') return 'OC'
  if (kind === 'hermes') return 'HM'
  if (kind === 'custom-cli') return 'SH'
  return 'CX'
}

function providerKindColor(kind?: ProviderProfile['providerKind']) {
  if (kind === 'claude-code') return '#d97706'
  if (kind === 'gemini-cli') return '#0f9f6e'
  if (kind === 'opencode') return '#2563eb'
  if (kind === 'hermes') return '#0891b2'
  if (kind === 'custom-cli') return '#475569'
  return '#4b83ff'
}

function providerToolTargetLabel(target: ProviderToolTarget) {
  if (target === 'claude') return 'Claude'
  if (target === 'gemini') return 'Gemini'
  if (target === 'opencode') return 'OpenCode'
  if (target === 'codex') return 'Codex'
  return t('provider.genericTarget')
}

function providerSourceLabel(source?: ProviderProfileSource) {
  if (source === 'cc-switch') return 'CC Switch'
  if (source === 'oauth') return t('provider.oauthSource')
  if (source === 'env') return t('provider.envSource')
  if (source === 'script') return t('provider.scriptSource')
  if (source === 'manual') return t('provider.manualSource')
  return t('provider.localCliConfig')
}

function providerKindSvgIcon(kind?: ProviderProfile['providerKind']): string {
  if (kind === 'claude-code') {
    return `<img src="${claudeBrandIcon}" style="width:100%;height:100%;object-fit:contain;display:block;" alt="Claude" />`
  }
  if (kind === 'gemini-cli') {
    return `<img src="${geminiBrandIcon}" style="width:100%;height:100%;object-fit:contain;display:block;" alt="Gemini" />`
  }
  if (kind === 'codex') {
    return `<img src="${openAiBrandIcon}" style="width:100%;height:100%;object-fit:contain;display:block;" alt="Codex" />`
  }
  if (kind === 'opencode') {
    return `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M8 7L3 12L8 17M16 7L21 12L16 17" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>`
  }
  if (kind === 'hermes') {
    return `<img src="${hermesBrandIcon}" style="width:100%;height:100%;object-fit:contain;display:block;" alt="Hermes" />`
  }
  if (kind === 'custom-cli') {
    return `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="3" stroke="currentColor" stroke-width="1.8"/><path d="M7 9L10 12L7 15M12 15H17" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>`
  }
  // fallback: generic terminal
  return `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="3" stroke="currentColor" stroke-width="1.8"/><path d="M7 9L10 12L7 15M12 15H17" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>`
}

function formatProviderRemainingBalance(value?: number | null) {
  if (typeof value !== 'number' || Number.isNaN(value)) return tr('未接通', 'Not connected')
  return `$${value.toFixed(2)}`
}

function providerPayloadStatusLabel(provider: Pick<ProviderProfile, 'configPayload' | 'authPayload'>) {
  const hasConfig = Boolean(normalizeProviderPayload(provider.configPayload))
  const hasAuth = Boolean(normalizeProviderPayload(provider.authPayload))
  if (hasConfig && hasAuth) return tr('auth + config 已保存', 'auth + config saved')
  if (hasConfig) return tr('仅保存 config', 'config only')
  if (hasAuth) return tr('仅保存 auth', 'auth only')
  return tr('未保存快照', 'no snapshot')
}

function providerScopeLabel(scope?: ProviderConfigScope) {
  if (scope === 'workspace') return t('provider.workspaceScope')
  if (scope === 'project') return t('provider.projectScope')
  return t('provider.globalScope')
}

function providerStatusLabel(status?: ProviderProfile['status']) {
  if (status === 'active') return t('provider.statusActive')
  if (status === 'missing') return t('provider.statusMissing')
  if (status === 'disabled') return t('provider.statusDisabled')
  return t('provider.statusAvailable')
}

function defaultProviderTargetsText(kind: ProviderProfile['providerKind']) {
  if (kind === 'claude-code') return 'claude'
  if (kind === 'gemini-cli') return 'gemini'
  if (kind === 'opencode') return 'opencode'
  if (kind === 'hermes') return 'generic'
  if (kind === 'custom-cli') return 'generic'
  return 'codex'
}

function defaultConfigPathForProvider(kind: ProviderProfile['providerKind']) {
  if (kind === 'claude-code') return '~/.claude.json'
  if (kind === 'gemini-cli') return '~/.gemini/settings.json'
  if (kind === 'opencode') return '~/.config/opencode/opencode.json'
  if (kind === 'hermes') return '~/.hermes/config.yaml'
  if (kind === 'custom-cli') return t('provider.localCliFile')
  return '~/.codex/config.toml'
}

function defaultAuthSourceForProvider(kind: ProviderProfile['providerKind'], source: ProviderProfileSource) {
  if (source === 'oauth') return t('provider.cliOauthState')
  if (source === 'env') return t('provider.shellEnv')
  if (source === 'script') return t('provider.scriptDriven')
  if (kind === 'claude-code') return t('provider.claudeLocalState')
  if (kind === 'gemini-cli') return t('provider.geminiLocalState')
  if (kind === 'codex') return t('provider.codexLocalState')
  if (kind === 'hermes') return t('provider.localConfigDriven')
  return t('provider.localConfigDriven')
}

/** Provider 编辑表单：切换命令 placeholder 按 CLI 平台，不写虚假的 cc-switch 示例 */
function switchCommandPlaceholderForKind(
  kind: ProviderProfile['providerKind'],
  profileName?: string,
) {
  void profileName
  if (kind === 'codex') return 'codex resume <session-id>'
  if (kind === 'claude-code') return 'claude --resume'
  if (kind === 'gemini-cli') return 'gemini'
  if (kind === 'opencode') return 'opencode'
  if (kind === 'hermes') return 'hermes'
  if (kind === 'deepseek-cli') return 'deepseek'
  return tr('可选；留空则仅通过配置写回切换', 'Optional; leave empty to switch via config write-back only')
}

function providerNameById(providerId: string) {
  return selectedWorkspaceProviders.value.find((provider) => provider.id === providerId)?.name ?? tr('未知配置档案', 'Unknown profile')
}

function providerKindByProfileId(profileId: string): ProviderProfile['providerKind'] | undefined {
  return selectedWorkspaceProviders.value.find((p) => p.id === profileId)?.providerKind
}

/** Usage 行：优先 identityKey 命中的 Profile 名称；未匹配显示「未关联档案」 */
function usageLogProviderName(log: Pick<ProviderRequestLog, 'providerProfileId' | 'managedProviderName' | 'managedProviderId'>) {
  const profile = selectedWorkspaceProviders.value.find((provider) => provider.id === log.providerProfileId)
  if (profile?.name) return profile.name
  if (String(log.providerProfileId || '').startsWith('identity:') || String(log.providerProfileId || '').startsWith('unknown:')) {
    const backendName = (log.managedProviderName || '').trim()
    return backendName
      ? tr(`未关联档案 · ${backendName}`, `Unlinked · ${backendName}`)
      : tr('未关联档案', 'Unlinked profile')
  }
  if (log.managedProviderName) return log.managedProviderName
  return tr('未知配置档案', 'Unknown profile')
}

/** Usage 行：优先 Profile.kind；未匹配时按 appType 映射图标（Hermes 独立，不伪装 Codex） */
function usageLogProviderKind(log: Pick<ProviderRequestLog, 'providerProfileId' | 'appType'>): ProviderProfile['providerKind'] | undefined {
  const fromProfile = providerKindByProfileId(log.providerProfileId)
  if (fromProfile) return fromProfile
  if (log.appType === 'claude') return 'claude-code'
  if (log.appType === 'gemini') return 'gemini-cli'
  if (log.appType === 'opencode') return 'opencode'
  if (log.appType === 'hermes') return 'hermes'
  if (log.appType === 'codex') return 'codex'
  return undefined
}

function formatFirstTokenMs(value?: number | null) {
  if (value == null || Number.isNaN(value) || value < 0) return '—'
  if (value < 1000) return `${Math.round(value)}ms`
  return `${(value / 1000).toFixed(2)}s`
}

function formatDurationMs(value?: number | null) {
  if (value == null || Number.isNaN(value) || value < 0) return '—'
  if (value < 1000) return `${Math.round(value)}ms`
  return `${(value / 1000).toFixed(1)}s`
}

function usageLogCostBreakdownTitle(log: ProviderRequestLog) {
  const parts = [
    `in $${(log.inputCostUsd ?? 0).toFixed(4)}`,
    `out $${(log.outputCostUsd ?? 0).toFixed(4)}`,
    `cacheR $${(log.cacheReadCostUsd ?? 0).toFixed(4)}`,
    `cacheW $${(log.cacheCreationCostUsd ?? 0).toFixed(4)}`,
  ]
  return parts.join(' · ')
}

function formatTrendBucketLabel(timestamp: string, period: string, bucket?: string) {
  const d = new Date(timestamp)
  if (Number.isNaN(d.getTime())) return timestamp
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  const hh = String(d.getHours()).padStart(2, '0')
  const mi = String(d.getMinutes()).padStart(2, '0')
  const mode = bucket || (
    period === '1h' ? 'minute'
      : period === 'today' ? 'hour'
        : (period === '7d' || period === '30d' || period === 'month') ? 'day'
          : period === '90d' ? 'week'
            : 'month'
  )
  if (mode === 'minute') return `${hh}:${mi}`
  if (mode === 'hour') return `${hh}:00`
  if (mode === 'week') return `${mm}/${dd}`
  if (mode === 'month') return `${d.getFullYear()}-${mm}`
  return `${mm}/${dd}`
}

function formatLargeNumber(value: number) {
  return Math.round(value).toLocaleString('en-US')
}

function formatCompactWan(value: number) {
  return currentLocale.value === 'zh-CN'
    ? `${(value / 10000).toFixed(1)} 万`
    : `${(value / 1000).toFixed(1)}k`
}

function formatUsageHour(value: string) {
  const date = new Date(value)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  return `${month}/${day} ${hour}:00`
}

function formatUsageDate(value: string) {
  const date = new Date(value)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')
  return `${month}/${day} ${hour}:${minute}`
}

function formatRelativeTime(value: string) {
  if (!value) return tr('从未使用', 'Never')
  const time = new Date(value).getTime()
  if (Number.isNaN(time)) return tr('从未使用', 'Never')
  const diffMs = Date.now() - time
  const minute = 60_000
  const hour = 60 * minute
  const day = 24 * hour
  if (diffMs < minute) return tr('刚刚', 'just now')
  if (diffMs < hour) return tr(`${Math.floor(diffMs / minute)} 分钟前`, `${Math.floor(diffMs / minute)}m ago`)
  if (diffMs < day) return tr(`${Math.floor(diffMs / hour)} 小时前`, `${Math.floor(diffMs / hour)}h ago`)
  if (diffMs < 30 * day) return tr(`${Math.floor(diffMs / day)} 天前`, `${Math.floor(diffMs / day)}d ago`)
  return formatUsageDate(value)
}

function isSameLocalDay(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return false
  const now = new Date()
  return date.getFullYear() === now.getFullYear()
    && date.getMonth() === now.getMonth()
    && date.getDate() === now.getDate()
}

function buildUsageLinePath(
  points: Array<{ inputTokens: number; outputTokens: number; cacheReadTokens: number; costUsd: number }>,
  getter: (point: { inputTokens: number; outputTokens: number; cacheReadTokens: number; costUsd: number }) => number,
  maxValue: number,
) {
  if (!points.length) return ''
  const top = 52
  const bottom = 284
  const height = bottom - top
  const coords = points.map((point, index) => ({
    x: points.length === 1 ? 500 : 56 + (884 / Math.max(points.length - 1, 1)) * index,
    y: bottom - (Math.max(0, getter(point)) / Math.max(1, maxValue)) * height,
  }))
  if (coords.length === 1) return `M${coords[0].x},${coords[0].y}`
  // Catmull-Rom → cubic bezier, tension 0.3
  let d = `M${coords[0].x},${coords[0].y}`
  const t = 0.3
  for (let i = 0; i < coords.length - 1; i++) {
    const p0 = coords[Math.max(0, i - 1)]
    const p1 = coords[i]
    const p2 = coords[i + 1]
    const p3 = coords[Math.min(coords.length - 1, i + 2)]
    const cp1x = p1.x + (p2.x - p0.x) * t
    const cp1y = p1.y + (p2.y - p0.y) * t
    const cp2x = p2.x - (p3.x - p1.x) * t
    const cp2y = p2.y - (p3.y - p1.y) * t
    d += ` C${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p2.x.toFixed(1)},${p2.y.toFixed(1)}`
  }
  return d
}

// Closed area path for gradient fill (same bezier, closes at bottom baseline)
function buildUsageAreaPath(
  points: Array<{ inputTokens: number; outputTokens: number; cacheReadTokens: number; costUsd: number }>,
  getter: (point: { inputTokens: number; outputTokens: number; cacheReadTokens: number; costUsd: number }) => number,
  maxValue: number,
) {
  const line = buildUsageLinePath(points, getter, maxValue)
  if (!line) return ''
  const firstX = points.length === 1 ? 500 : 56
  const lastX = points.length === 1 ? 500 : 56 + 884
  return `${line} L${lastX},284 L${firstX},284 Z`
}

function removeTab(tabId: string) {
  if (!selectedWorkspace.value) return

  if (selectedWorkspace.value.tabs.length <= 1) {
    showToast(t('toast.tabDeleteBlocked'), t('toast.tabDeleteBlockedMsg'))
    return
  }

  const tab = selectedWorkspace.value.tabs.find((item) => item.id === tabId)
  if (!tab) return

  requestConfirm({
    title: t('confirm.deleteTabTitle', { name: tab.name }),
    description: t('confirm.deleteTabDesc'),
    confirmLabel: t('confirm.deleteTabConfirm'),
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

      showToast(t('toast.tabDeleted'), t('toast.removedName', { name: tab.name }))
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
  confirmDialog.confirmLabel = options.confirmLabel ?? t('confirm.confirm')
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
  if (typeof window === 'undefined') return 248
  const raw = window.localStorage.getItem('chuchen-terminal.workbench-sidebar-width')
  const parsed = raw ? Number(raw) : NaN
  return Number.isFinite(parsed) ? Math.min(420, Math.max(192, parsed)) : 248
}

function saveWorkbenchSidebarWidth(width: number) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem('chuchen-terminal.workbench-sidebar-width', String(width))
}

async function pickDirectory(defaultPath?: string) {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    showToast(t('toast.desktopOnly'), t('toast.desktopOnlyMsg'))
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
    showToast(t('toast.directoryPickFailed'), t('toast.directoryPickFailedMsg'))
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
  const workbenchImmersiveValue = window.localStorage.getItem('chuchen-terminal.workbench-immersive')
  const workbenchExplorerCollapsedValue = window.localStorage.getItem('chuchen-terminal.workbench-explorer-collapsed')
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

  if (workbenchImmersiveValue === '1' || workbenchImmersiveValue === '0') {
    workbenchImmersive.value = workbenchImmersiveValue === '1'
  }

  if (workbenchExplorerCollapsedValue === '1' || workbenchExplorerCollapsedValue === '0') {
    workbenchExplorerCollapsed.value = workbenchExplorerCollapsedValue === '1'
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
  window.localStorage.setItem('chuchen-terminal.workbench-immersive', workbenchImmersive.value ? '1' : '0')
  window.localStorage.setItem('chuchen-terminal.workbench-explorer-collapsed', workbenchExplorerCollapsed.value ? '1' : '0')
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

  if (!windowVisible.value) {
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
  if (workbenchExplorerCollapsed.value || immersiveWorkbenchActive.value) return
  event.preventDefault()
  event.stopPropagation()
  ;(event.currentTarget as HTMLElement | null)?.setPointerCapture?.(event.pointerId)
  const startX = event.clientX
  const startWidth = workbenchSidebarWidth.value

  const handleMove = (moveEvent: PointerEvent) => {
    const next = startWidth + (moveEvent.clientX - startX)
    workbenchSidebarWidth.value = Math.min(420, Math.max(192, next))
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
    paneSessions(pane).map((session) => {
      const info = aiCliInfoForSession(workspace, pane, session)
      return {
        pane,
        session,
        info,
        displayName: sessionDisplayName(workspace, pane, session, info),
      }
    }),
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
  if (workspace.id === selectedWorkspace.value?.id) {
    return entryId ? selectedWorkspaceEntryMap.value.get(entryId) : undefined
  }
  const entry = workspace.terminalEntries.find((item) => item.id === entryId)
  if (!entry) return entry
  const runtimeStatus = runtimeEntryStatusOverlays.value[entry.id]
  return runtimeStatus ? { ...entry, status: runtimeStatus } : entry
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
    const nextBranch = info.isRepo ? (info.branch?.trim() || 'HEAD') : tr('非 Git 仓库', 'Not a Git repository')
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
    const nextBranch = workspace.gitBranch?.trim() || tr('未检测分支', 'Branch not detected')
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
  return workspace.gitBranch?.trim() || tr('未检测分支', 'Branch not detected')
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
  return entryId ? selectedWorkspaceEntryMap.value.get(entryId) : undefined
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
  if (state === 'fresh' && !(session.lastAiCliKind || session.aiCliKind)) return t('ai.states.notStarted')
  return sessionAttentionStateLabel(state)
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
  if (state === 'error') return tr('异常', 'Error')
  if (state === 'needs-input') return tr('待处理', 'Needs input')
  if (state === 'stalled') return tr('停滞', 'Stalled')
  if (state === 'completed') return tr('完成', 'Done')
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
  const info = aiCliInfoForSession(located.workspace, located.pane, located.session)
  return {
    workspaceName: located.workspace.name,
    tabName: located.tab.name,
    sessionName: sessionDisplayName(located.workspace, located.pane, located.session, info),
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
                          ? tr('已开启任务监督：会检测完成、异常退出和疑似停滞。', 'Task supervision enabled: completion, abnormal exits, and suspected stalls will be monitored.')
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
    }
  }), 'transient')

  showToast(mode === 'watch' ? tr('任务监督已开启', 'Task supervision enabled') : tr('任务监督已关闭', 'Task supervision disabled'), `${located.workspace.name} / ${located.tab.name} / ${located.session.name}`)
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
    }
  }), 'transient')

  sessionAttentionNotificationState.delete(sessionId)
  lastAttentionBadgeCount = -1
  void applyWindowAttentionBadge(true)
  if (!options.silent) {
    showToast(tr('提醒状态已清除', 'Alert state cleared'), `${located.workspace.name} / ${located.tab.name} / ${located.session.name}`)
  }
}

function startSupervisorScan() {
  if (typeof window === 'undefined') return
  if (supervisorScanTimer) {
    window.clearInterval(supervisorScanTimer)
  }
  const intervalMs = windowVisible.value ? SUPERVISOR_SCAN_INTERVAL_MS : Math.max(SUPERVISOR_SCAN_INTERVAL_MS * 3, 15000)
  supervisorScanTimer = window.setInterval(() => {
    scanSupervisorSessions()
  }, intervalMs)
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
              supervisorNote: tr('任务监督：超过 2 分钟未检测到终端输出，建议人工查看。', 'Task supervision: no terminal output was detected for over 2 minutes. Manual review is recommended.'),
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
    showToast(tr('提醒测试不可用', 'Alert test unavailable'), tr('请先选中一个终端会话。', 'Select a terminal session first.'))
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
                      aiCliKind: null,
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
                    aiCliKind: null,
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
                      ? tr('调试入口：模拟完成。', 'Debug action: simulated completion.')
                      : state === 'needs-input'
                        ? tr('调试入口：模拟等待输入。', 'Debug action: simulated needs input.')
                        : state === 'error'
                          ? tr('调试入口：模拟异常退出。', 'Debug action: simulated abnormal exit.')
                          : tr('调试入口：模拟疑似停滞。', 'Debug action: simulated suspected stall.'),
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
    }
  }), 'transient')

  showToast(
    state === 'clear' ? tr('提醒测试已清除', 'Alert test cleared') : tr('提醒测试已触发', 'Alert test triggered'),
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
        sessionName: sessionDisplayName(located.workspace, located.pane, located.session),
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
              sessionName: sessionDisplayName(workspace, pane, session),
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
  setRuntimeSessionOverlay(sessionId, {
    status,
    lastHeartbeatAt: status === 'running' ? now : located.session.lastHeartbeatAt ?? now,
    lastActivityAt: status === 'running' ? now : located.session.lastActivityAt ?? now,
  })
  syncRuntimeEntryStatusesForWorkspace(located.workspace)
}

function recordSessionCommand(sessionId: string, command: string) {
  const trimmed = command.trim()
  if (!trimmed) return

  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const entryId = located.session.terminalEntryId ?? located.pane.terminalEntryId ?? null
  const now = new Date().toISOString()
  sessionOutputTailBySession.set(sessionId, '')
  const commandAiInfo = aiCliInfoFromText(trimmed)
  const nextDetectedKind = isConfirmedAiCliInfo(commandAiInfo)
    ? commandAiInfo.kind
    : (located.session.aiCliKind && located.session.aiCliKind !== 'generic-ai' ? located.session.aiCliKind : null)
  const nextLastDetectedKind = isConfirmedAiCliInfo(commandAiInfo)
    ? commandAiInfo.kind
    : (located.session.lastAiCliKind && located.session.lastAiCliKind !== 'generic-ai'
        ? located.session.lastAiCliKind
        : nextDetectedKind)
  const launchOnlyAiCommand = isAiCliLaunchOnlyCommand(trimmed, nextLastDetectedKind)
  setRuntimeSessionOverlay(sessionId, {
    status: 'running',
    aiCliKind: nextDetectedKind,
    lastAiCliKind: nextLastDetectedKind,
    hasUserCommand: launchOnlyAiCommand ? false : true,
    lastCommandAt: now,
    lastExitCode: null,
    lastActivityAt: now,
    lastHeartbeatAt: now,
    supervisorState: launchOnlyAiCommand
      ? 'idle'
      : located.session.supervisorMode === 'watch' || located.session.supervisorMode === 'auto-resume'
        ? 'watching'
        : 'idle',
    supervisorNote: null,
  })

  commitWorkspaces((current) => current.map((workspace) => {
    if (workspace.id !== located.workspace.id) return workspace

    return {
      ...workspace,
      terminalEntries: workspace.terminalEntries.map((entry) =>
        entry.id === entryId
          ? {
              ...entry,
              lastCommand: trimmed,
              commandHistory: [trimmed, ...(entry.commandHistory ?? []).filter((item) => item.trim() !== trimmed)].slice(0, 20),
              updatedAt: now,
            }
          : entry,
      ),
    }
  }), 'persist')

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
  const promptReturned = hasReturnedToShellPrompt(mergedTail)
  const detectedAiInfo = aiCliInfoFromText(mergedTail)
  const nextDetectedKind = isConfirmedAiCliInfo(detectedAiInfo)
    ? detectedAiInfo.kind
    : (located.session.aiCliKind && located.session.aiCliKind !== 'generic-ai' ? located.session.aiCliKind : null)
  const aiReadyReturned = hasReturnedToAiReadyState(mergedTail, nextDetectedKind ?? located.session.lastAiCliKind ?? located.session.aiCliKind)
  sessionOutputTailBySession.set(sessionId, promptReturned || aiReadyReturned ? '' : mergedTail)
  const nextActiveKind = promptReturned ? null : nextDetectedKind
  const nextLastDetectedKind = isConfirmedAiCliInfo(detectedAiInfo)
    ? detectedAiInfo.kind
    : (located.session.lastAiCliKind && located.session.lastAiCliKind !== 'generic-ai'
        ? located.session.lastAiCliKind
        : nextDetectedKind)
  const nextStatus = (promptReturned || aiReadyReturned) ? 'idle' : located.session.status
  setRuntimeSessionOverlay(sessionId, {
    status: nextStatus,
    aiCliKind: nextActiveKind,
    lastAiCliKind: nextLastDetectedKind,
    lastOutputAt: now,
    lastActivityAt: now,
    lastHeartbeatAt: now,
    lastExitCode: completed ? 0 : located.session.lastExitCode ?? null,
    supervisorState: completed || (aiReadyReturned && located.session.hasUserCommand)
      ? 'completed'
      : promptReturned && located.session.hasUserCommand && (located.session.supervisorMode === 'watch' || located.session.supervisorMode === 'auto-resume')
        ? 'completed'
        : promptReturned && located.session.hasUserCommand
          ? 'idle'
          : located.session.supervisorState,
    supervisorNote: completed || (aiReadyReturned && located.session.hasUserCommand) || (promptReturned && located.session.hasUserCommand && (located.session.supervisorMode === 'watch' || located.session.supervisorMode === 'auto-resume'))
      ? tr('任务监督：检测到终端回到可输入状态。', 'Task supervision: the terminal returned to an input-ready state.')
      : located.session.supervisorNote,
  })
  if (nextStatus !== located.session.status) {
    syncRuntimeEntryStatusesForWorkspace(located.workspace)
  }
}

function recordSessionExit(sessionId: string, exitCode: number) {
  const located = locateSessionAcrossWorkspaces(sessionId)
  if (!located) return
  const now = new Date().toISOString()
  sessionOutputTailBySession.delete(sessionId)
  setRuntimeSessionOverlay(sessionId, {
    status: 'idle',
    aiCliKind: null,
    lastAiCliKind: located.session.lastAiCliKind ?? located.session.aiCliKind ?? null,
    lastExitCode: exitCode,
    lastActivityAt: now,
    lastHeartbeatAt: now,
    supervisorState: exitCode === 0 && located.session.hasUserCommand
      ? 'completed'
      : exitCode !== 0
        ? 'needs-human'
        : located.session.supervisorState,
  })
  syncRuntimeEntryStatusesForWorkspace(located.workspace)
}

function launchModeLabel(mode?: TerminalEntry['launchMode']) {
  if (mode === 'open-only') return t('launchMode.openOnly')
  if (mode === 'prefill') return t('launchMode.prefill')
  if (mode === 'execute') return t('launchMode.execute')
  if (mode === 'switch-or-create') return t('launchMode.switchOrCreate')
  return t('workspace.notSet')
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
        h('strong', tr('快捷命令', 'Quick commands')),
        h('small', entry?.name || tr('工作区最近命令', 'Workspace recent commands')),
      ]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: t('common.actions.close'),
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
      : h('div', { class: 'command-panel__empty' }, tr('当前终端还没有可复用命令。', 'No reusable commands are available for the current terminal yet.')),
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
        title: tr('插入命令', 'Insert command'),
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          void insertPaneText(pane, command)
        },
      }, [h(AppIcon, { name: 'bolt', size: 12 })]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: tr('执行命令', 'Execute command'),
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          void executePaneText(pane, command)
        },
      }, [h(AppIcon, { name: 'play', size: 12 })]),
      h('button', {
        type: 'button',
        class: ['icon-btn icon-btn--mini', { 'icon-btn--active': favorite }],
        title: favorite ? tr('取消收藏', 'Remove from favorites') : tr('收藏命令', 'Favorite command'),
        disabled: !favoriteScope,
        onClick: (event: MouseEvent) => {
          event.stopPropagation()
          toggleFavoriteCommand(pane, command)
        },
      }, [h(AppIcon, { name: 'star', size: 12 })]),
      h('button', {
        type: 'button',
        class: 'icon-btn icon-btn--mini',
        title: tr('复制命令', 'Copy command'),
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
    const activeSession = activePaneSession(pane)
    const activeAiInfo = aiCliInfoForSession(selectedWorkspace.value, pane, activeSession)
    return h('section', {
        'data-pane-id': pane.id,
        class: [
          'pane',
          {
            'pane--running': paneHasRunningSession(pane),
            'pane--selected': activeRuntimePaneId.value === pane.id,
            'pane--ai-cli': activeAiInfo.isAi,
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
            ...paneSessions(pane).map((session) => {
              const sessionAiInfo = aiCliInfoForSession(selectedWorkspace.value, pane, session)
              const sessionBrandKind = resolvedAiBrandKind(selectedWorkspace.value, sessionAiInfo)
              return h('div', {
                key: session.id,
                role: 'button',
                tabindex: 0,
                'data-session-id': session.id,
                'data-ai-cli-kind': sessionAiInfo.kind,
                class: ['terminal-window-tab', {
                  'terminal-window-tab--active': activePaneSession(pane)?.id === session.id,
                  'terminal-window-tab--dragging': draggingSession.value?.sourceSessionId === session.id,
                  'terminal-window-tab--supervised': sessionIsSupervised(session),
                  'terminal-window-tab--ai-cli': sessionAiInfo.isAi,
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
                shouldShowAiSessionStyling(sessionAiInfo) && sessionBrandKind
                  ? renderAiBrandIcon(sessionBrandKind, 'tab-leading', sessionAiInfo.iconName, 16)
                  : h(AppIcon, { name: shouldShowAiSessionStyling(sessionAiInfo) ? sessionAiInfo.iconName : 'terminal', size: 14 }),
                h('span', sessionDisplayName(selectedWorkspace.value, pane, session, sessionAiInfo)),
                null,
                h('small', sessionStatusLabel(session)),
                h('button', {
                  type: 'button',
                  class: 'terminal-window-tab__reload',
                  title: tr('重新加载当前终端', 'Reload current terminal'),
                  role: 'button',
                  tabindex: 0,
                  'data-no-drag': 'true',
                  onClick: (event: MouseEvent) => { event.stopPropagation(); reloadPaneSession(pane.id, session.id) },
                  onPointerdown: (event: PointerEvent) => { event.stopPropagation() },
                }, [h(AppIcon, { name: 'refresh', size: 12 })]),
                h('button', {
                  type: 'button',
                  class: 'terminal-window-tab__close',
                  title: tr('关闭当前终端', 'Close current terminal'),
                  role: 'button',
                  tabindex: 0,
                  'data-no-drag': 'true',
                  onClick: (event: MouseEvent) => { event.stopPropagation(); removePaneSession(pane.id, session.id) },
                  onPointerdown: (event: PointerEvent) => { event.stopPropagation() },
                }, [h(AppIcon, { name: 'close', size: 12 })]),
              ])
            }),
            h('button', {
              type: 'button',
              class: 'terminal-window-tab terminal-window-tab--add',
              title: tr('在当前分组新建终端', 'Create terminal in current group'),
              'data-no-drag': 'true',
              onClick: (event: MouseEvent) => { event.stopPropagation(); createPaneSession(pane.id) },
            }, [h(AppIcon, { name: 'plus', size: 13 })]),
          ]),
          isConfirmedAiCliInfo(activeAiInfo)
            ? h('div', {
                class: 'pane-ai-pill',
                title: activeSession?.supervisorNote || tr(`${activeAiInfo.label}：可在此查看历史或固定到辅助层。`, `${activeAiInfo.label}: View history here or pin it to the assist layer.`),
              }, [
                h('span', { class: ['pane-ai-pill__badge', 'ai-tool-badge', `ai-tool-badge--${activeAiInfo.tone}`, { 'ai-tool-badge--icon-only': !aiCliBadgeText(activeAiInfo) }] }, [
                  renderAiBrandIcon(activeAiInfo.kind, 'badge', activeAiInfo.iconName),
                  aiCliBadgeText(activeAiInfo) ? h('span', aiCliBadgeText(activeAiInfo)) : null,
                ]),
                h('span', { class: 'pane-ai-pill__label' }, activeAiInfo.label),
                h('button', {
                  type: 'button',
                  class: 'pane-ai-pill__action',
                  'data-no-drag': 'true',
                  onClick: (event: MouseEvent) => {
                    event.stopPropagation()
                    openAiHistoryDrawer.value = true
                  },
                }, tr('历史', 'History')),
              ])
            : null,
          h('div', { class: 'pane__actions' }, [
            h('button', {
              type: 'button',
              class: ['icon-btn icon-btn--bolt', { 'icon-btn--active': activeCommandPanelPaneId.value === pane.id }],
              title: tr('快捷命令', 'Quick commands'),
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
              title: tr('Pane 分组操作', 'Pane group actions'),
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
            key: `${activeSession?.id || pane.id}-${terminalReloadVersions.value[activeSession?.id || pane.id] ?? 0}`,
            sessionId: activeSession?.id || pane.id,
            sessionName: activeSession?.name || pane.name,
            workingDirectory: terminalSessionWorkingDirectory(pane),
            shellLabel: terminalSessionShellLabel(pane),
            fontFamily: terminalFontFamily.value,
            fontSize: terminalFontSize.value,
            active: activeRuntimePaneId.value === pane.id,
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
        aiCliKind: null,
        lastAiCliKind: null,
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
  showToast(
    direction === 'horizontal' ? tr('已拆分到右侧', 'Split to the right') : tr('已拆分到下方', 'Split below'),
    tr(`新建分组：${nextPane.name}`, `New group: ${nextPane.name}`),
  )
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
  if (direction === 'horizontal') return t('workspace.splitHorizontal')
  if (direction === 'vertical') return t('workspace.splitVertical')
  return t('workspace.splitSingle')
}

function paneSessions(pane: PaneNode): PaneTerminalSession[] {
  if (pane.sessions) {
    return pane.sessions.map((session) => mergeRuntimeSessionOverlay(session))
  }

  return [
    {
      id: `${pane.id}-session-main`,
      name: pane.name,
      pathLabel: pane.pathLabel,
      terminalEntryId: pane.terminalEntryId,
      status: 'idle' as const,
      aiCliKind: null,
      lastAiCliKind: null,
    },
  ].map((session) => mergeRuntimeSessionOverlay(session))
}

function mergeRuntimeSessionOverlay(session: PaneTerminalSession): PaneTerminalSession {
  const overlay = runtimeSessionOverlays.value[session.id]
  if (!overlay) return session
  return {
    ...session,
    ...overlay,
    status: (overlay.status ?? session.status) as PaneTerminalSession['status'],
  }
}

function setRuntimeSessionOverlay(sessionId: string, patch: Partial<PaneTerminalSession>) {
  runtimeSessionOverlays.value = {
    ...runtimeSessionOverlays.value,
    [sessionId]: {
      ...(runtimeSessionOverlays.value[sessionId] ?? {}),
      ...patch,
    },
  }
}

function clearRuntimeSessionOverlay(sessionId: string) {
  if (!(sessionId in runtimeSessionOverlays.value)) return
  const next = { ...runtimeSessionOverlays.value }
  delete next[sessionId]
  runtimeSessionOverlays.value = next
}

function syncRuntimeEntryStatusesForWorkspace(workspace: WorkspaceCard | undefined) {
  if (!workspace) return
  const nextStatuses: Record<string, TerminalEntry['status']> = {}
  workspace.terminalEntries.forEach((entry) => {
    const running = workspace.tabs.some((tab) =>
      flattenLeafPanes(tab.panes).some((pane) =>
        paneSessions(pane).some((session) => session.terminalEntryId === entry.id && session.status === 'running'),
      ),
    )
    nextStatuses[entry.id] = running ? 'running' : 'idle'
  })
  runtimeEntryStatusOverlays.value = nextStatuses
}

function syncRuntimeEntryStatusesForCurrentWorkspaces() {
  const nextStatuses: Record<string, TerminalEntry['status']> = {}
  workspaces.value.forEach((workspace) => {
    workspace.terminalEntries.forEach((entry) => {
      const running = workspace.tabs.some((tab) =>
        flattenLeafPanes(tab.panes).some((pane) =>
          paneSessions(pane).some((session) => session.terminalEntryId === entry.id && session.status === 'running'),
        ),
      )
      nextStatuses[entry.id] = running ? 'running' : 'idle'
    })
  })
  runtimeEntryStatusOverlays.value = nextStatuses
}

function activePaneSession(pane: PaneNode) {
  const sessions = paneSessions(pane)
  const runtimeSessionId = runtimeActiveSessionIds.value[pane.id]
  return sessions.find((session) => session.id === runtimeSessionId)
    ?? sessions.find((session) => session.id === pane.activeSessionId)
    ?? sessions[0]
}

function sessionStatusLabel(session?: PaneTerminalSession) {
  if (!session) return t('workspace.sessionIdle')
  if (session.status === 'running') return t('ai.states.running')
  if (entryById(session.terminalEntryId)?.lastCommand || entryById(session.terminalEntryId)?.defaultCommand) return t('workspace.sessionWaiting')
  return t('workspace.sessionIdle')
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
  const label = tab.name.trim() || tr('未命名项目', 'Untitled project')
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

  if (tone === 'running') return t('ai.states.running')
  if (tone === 'waiting') return t('workspace.sessionWaiting')
  return t('workspace.sessionIdle')
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
    if (systemStatus.cpu === '检测中' || systemStatus.memory === '检测中' || systemStatus.gpu === '检测中' || systemStatus.cpu === 'Checking' || systemStatus.memory === 'Checking' || systemStatus.gpu === 'Checking') {
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
    showToast(t('toast.themeUpdated'), t('toast.themeUpdatedMsg', { name: themeDisplayName(theme) }))
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

function applyLocaleFromSettings(nextLocale: AppLocale) {
  if (currentLocale.value === nextLocale) return
  locale.value = nextLocale
  setAppLocale(nextLocale)
}

function applyTerminalFontSizeFromSettings(size: number) {
  setTerminalFontSize(size)
  showToast(t('toast.terminalFontSizeUpdated'), `${size}px`)
}

function applyTerminalFontFamilyFromSettings(font: string) {
  terminalFontFamily.value = font
  showToast(t('toast.terminalFontUpdated'), font)
}

function applyRestoreStrategyFromSettings(strategy: RestoreCommandStrategy) {
  restoreCommandStrategy.value = strategy
  showToast(t('toast.restoreStrategyUpdated'), restoreCommandStrategyLabel(strategy))
}

function applySystemRefreshModeFromSettings(mode: 'manual' | '5s' | '10s' | '30s') {
  setSystemRefreshMode(mode)
  showToast(t('toast.refreshRateUpdated'), mode === 'manual' ? t('refresh.manual') : t('refresh.autoInterval', { interval: mode }))
}

function toggleEnvironmentVisibilityFromSettings(name: string) {
  toggleEnvironmentVisibility(name)
  showToast(t('toast.environmentVisibilityUpdated'), name)
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
  showToast(t('toast.diagnosticsReset'), t('toast.diagnosticsResetMsg'))
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




