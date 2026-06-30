<template>
  <div class="agent-health-panel">
    <div
      v-for="status in statuses"
      :key="status.id"
      class="health-row"
      :class="{ active: status.id === activeId, 'not-installed': !status.installed }"
      @click="$emit('select', status.id)"
    >
      <div class="health-indicator">
        <span v-if="status.installed" class="dot installed" />
        <span v-else class="dot missing" />
      </div>
      <div class="health-info">
        <div class="health-name">{{ status.display_name }}</div>
        <div v-if="status.version" class="health-version">v{{ status.version }}</div>
        <div v-if="status.error" class="health-error">{{ status.error }}</div>
      </div>
      <div v-if="!status.installed && status.native_install_command" class="health-action">
        <el-button size="small" text type="primary" @click.stop="$emit('install', status.id)">
          安装
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AgentStatus } from '@/store/agent'

defineProps<{
  statuses: AgentStatus[]
  activeId: string
}>()

defineEmits<{
  select: [id: string]
  install: [id: string]
}>()
</script>

<style scoped>
.agent-health-panel {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.health-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background 0.15s;
  border-radius: 6px;
  margin: 0 4px;
}

.health-row:hover {
  background: var(--color-hover-bg);
}

.health-row.active {
  background: var(--color-selected-bg);
}

.health-indicator {
  flex-shrink: 0;
}

.dot {
  display: block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.dot.installed {
  background: #34C759;
  box-shadow: 0 0 4px rgba(52, 199, 89, 0.4);
}

.dot.missing {
  background: var(--el-color-danger-light-5);
}

.health-info {
  flex: 1;
  min-width: 0;
}

.health-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.health-version {
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.health-error {
  font-size: 13px;
  color: var(--el-color-danger);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.health-action {
  flex-shrink: 0;
}
</style>
