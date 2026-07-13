<template>
  <div class="manage-page">
    <div class="manage-tabs">
      <button class="tab-back" @click="$emit('back')" title="返回对话">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/>
        </svg>
      </button>
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-item"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
        :title="tab.label"
      >
        <component :is="tab.icon" />
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </div>

    <div class="manage-content">
      <ProjectsTab v-if="activeTab === 'projects'" @enter-project="(p: ProjectEntry) => $emit('enter-project', p)" />
      <ConfigTab v-else-if="activeTab === 'config'" />
      <CommandsTab v-else-if="activeTab === 'commands'" />
      <EnvTab v-else-if="activeTab === 'env'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, h } from 'vue'
import type { ProjectEntry } from '@/store/agent'
import ProjectsTab from './manage/ProjectsTab.vue'
import ConfigTab from './manage/ConfigTab.vue'
import CommandsTab from './manage/CommandsTab.vue'
import EnvTab from './manage/EnvTab.vue'

defineEmits<{
  'back': []
  'enter-project': [project: ProjectEntry]
}>()

type TabId = 'projects' | 'config' | 'commands' | 'env'

const activeTab = ref<TabId>('projects')

const FolderIcon = {
  render() {
    return h('svg', { width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.5 }, [
      h('path', { d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' })
    ])
  }
}

const SettingsIcon = {
  render() {
    return h('svg', { width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2 }, [
      h('circle', { cx: '12', cy: '12', r: '3' }),
      h('path', { d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z' })
    ])
  }
}

const RocketIcon = {
  render() {
    return h('svg', { width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2 }, [
      h('path', { d: 'M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z' }),
      h('path', { d: 'm12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z' }),
      h('path', { d: 'M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0' }),
      h('path', { d: 'M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5' })
    ])
  }
}

const ActivityIcon = {
  render() {
    return h('svg', { width: 16, height: 16, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2 }, [
      h('polyline', { points: '22 12 18 12 15 21 9 3 6 12 2 12' })
    ])
  }
}

const tabs: { id: TabId; icon: typeof FolderIcon; label: string }[] = [
  { id: 'projects', icon: FolderIcon, label: '项目' },
  { id: 'config', icon: SettingsIcon, label: '配置' },
  { id: 'commands', icon: RocketIcon, label: '命令' },
  { id: 'env', icon: ActivityIcon, label: '环境' },
]
</script>

<style scoped>
.manage-page {
  display: flex;
  height: 100%;
}

.manage-tabs {
  width: 64px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 0;
  gap: 4px;
  border-right: 1px solid var(--color-sidebar-border);
  background: var(--color-layer-1, var(--color-sidebar-bg));
}

.tab-back {
  width: 40px;
  height: 32px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
  transition: all 0.15s;
}

.tab-back:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.tab-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 48px;
  padding: 8px 0;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s;
}

.tab-item:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.tab-item.active {
  background: var(--color-sidebar-item-active);
  color: var(--color-text-primary);
  font-weight: 500;
}

.tab-label {
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
  text-align: center;
}

.manage-content {
  flex: 1;
  overflow-y: auto;
}
</style>
