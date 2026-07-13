<template>
  <div class="env-check-panel">
    <div
      v-for="item in items"
      :key="item.label"
      class="env-row"
    >
      <el-icon :size="16" :color="item.installed ? '#34C759' : 'var(--el-color-danger)'">
        <Check v-if="item.installed" />
        <Close v-else />
      </el-icon>
      <div class="env-info">
        <span class="env-label">{{ item.label }}</span>
        <span v-if="item.version" class="env-version">{{ item.version }}</span>
        <span v-else-if="!item.installed" class="env-missing">未安装</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Check, Close } from '@element-plus/icons-vue'

interface EnvResult {
  node_installed: boolean
  node_version: string | null
  npm_installed: boolean
  npm_version: string | null
  python_installed: boolean
  python_version: string | null
}

interface EnvItem {
  label: string
  installed: boolean
  version: string | null
}

const items = ref<EnvItem[]>([])

onMounted(async () => {
  try {
    const env = await invoke<EnvResult>('agent_check_environment')
    items.value = [
      { label: 'Node.js', installed: env.node_installed, version: env.node_version },
      { label: 'npm', installed: env.npm_installed, version: env.npm_version },
      { label: 'Python3', installed: env.python_installed, version: env.python_version },
    ]
  } catch {
    items.value = [
      { label: 'Node.js', installed: false, version: null },
      { label: 'npm', installed: false, version: null },
      { label: 'Python3', installed: false, version: null },
    ]
  }
})
</script>

<style scoped>
.env-check-panel {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.env-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-radius: 6px;
  margin: 0 4px;
}

.env-info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.env-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.env-version {
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.env-missing {
  font-size: 13px;
  color: var(--el-color-danger-light-3);
}
</style>
