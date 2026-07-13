<template>
  <div class="config-tab">
    <div class="config-header">
      <h2 class="config-title">配置</h2>
      <div class="config-actions">
        <button class="outline-btn" @click="handleExport">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          导出
        </button>
        <button class="outline-btn" @click="handleImport">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          导入
        </button>
      </div>
    </div>
    <el-scrollbar class="config-scroll">
      <AgentConfigPanel />
    </el-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import AgentConfigPanel from '@/apps/Settings/panels/AgentConfigPanel.vue'

async function handleExport() {
  try {
    await invoke('agent_export_config_dialog')
  } catch (e) {
    if (!String(e).includes('USER_CANCELLED')) {
      ElMessage.error('导出失败')
    }
  }
}

async function handleImport() {
  try {
    await invoke('agent_import_config_dialog')
    ElMessage.success('配置已导入')
  } catch (e) {
    if (!String(e).includes('USER_CANCELLED')) {
      ElMessage.error('导入失败')
    }
  }
}
</script>

<style scoped>
.config-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.config-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 24px 0;
}

.config-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.config-actions {
  display: flex;
  gap: 8px;
}

.outline-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-card-border);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.outline-btn:hover {
  border-color: var(--color-text-tertiary);
}

.config-scroll {
  flex: 1;
  padding: 0 24px 24px;
}
</style>
