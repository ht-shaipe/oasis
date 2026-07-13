<template>
  <div class="agent-app">
    <div class="agent-titlebar">
      <div class="titlebar-tabs">
        <button
          v-for="item in navItems"
          :key="item.page"
          class="titlebar-tab"
          :class="{ active: currentPage === item.page }"
          @click="currentPage = item.page"
        >
          <component :is="item.icon" />
          <span>{{ t(item.labelKey) }}</span>
        </button>
      </div>
    </div>

    <div class="agent-body">
      <ChatPage
        v-if="currentPage === 'chat'"
        @switch-project="currentPage = 'manage'"
      />
      <ManagePage
        v-else-if="currentPage === 'manage'"
        @back="currentPage = 'chat'"
        @enter-project="handleEnterProject"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAgentStore } from '@/store/agent'
import type { ProjectEntry } from '@/store/agent'
import ChatPage from './components/ChatPage.vue'
import ManagePage from './components/ManagePage.vue'

const { t } = useI18n()
const store = useAgentStore()
const currentPage = ref<'chat' | 'manage'>('chat')

const FolderIcon = {
  render() {
    return h('svg', { width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5 }, [
      h('path', { d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' })
    ])
  }
}

const SettingsIcon = {
  render() {
    return h('svg', { width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2 }, [
      h('circle', { cx: '12', cy: '12', r: '3' }),
      h('path', { d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z' })
    ])
  }
}

const navItems = [
  { page: 'chat' as const, icon: FolderIcon, labelKey: 'agent.navChat' },
  { page: 'manage' as const, icon: SettingsIcon, labelKey: 'agent.navManage' },
]

function handleEnterProject(project: ProjectEntry) {
  store.selectProject(project.path)
  currentPage.value = 'chat'
}
</script>

<style scoped>
.agent-app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-glass);
}

.agent-titlebar {
  display: flex;
  align-items: center;
  height: 36px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--color-card-border);
  background: var(--color-layer-1, var(--color-sidebar-bg));
  padding-left: 76px;
}

.titlebar-tabs {
  display: flex;
  gap: 2px;
}

.titlebar-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 14px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: var(--app-font-13);
  cursor: pointer;
  transition: all 0.15s;
}

.titlebar-tab:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.titlebar-tab.active {
  background: var(--color-sidebar-item-active);
  color: var(--color-text-primary);
  font-weight: 500;
}

.agent-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>
