<template>
  <div class="command-panel">
    <div class="cmd-section">
      <div class="cmd-section-header">
        <span>预设命令</span>
      </div>
      <el-scrollbar max-height="220px">
        <div
          v-for="preset in presets"
          :key="preset.name"
          class="cmd-item"
          @click="runPreset(preset.command)"
        >
          <div class="cmd-item-main">
            <div class="cmd-item-name">
              {{ preset.name }}
              <span v-if="preset.is_launch" class="cmd-badge launch">启动</span>
              <span v-if="preset.is_resume" class="cmd-badge resume">恢复</span>
              <span v-if="preset.is_init" class="cmd-badge init">初始化</span>
            </div>
            <div class="cmd-item-desc">{{ preset.description }}</div>
          </div>
        </div>
        <div v-if="presets.length === 0" class="cmd-empty">暂无预设命令</div>
      </el-scrollbar>
    </div>

    <div class="cmd-section">
      <div class="cmd-section-header">
        <span>自定义命令</span>
        <el-button size="small" text type="primary" @click="toggleAddForm">
          {{ showAddForm ? '取消' : '添加' }}
        </el-button>
      </div>

      <div v-if="showAddForm" class="cmd-add-form">
        <el-input v-model="newCmd.name" placeholder="名称" size="small" />
        <el-input v-model="newCmd.description" placeholder="描述" size="small" />
        <el-input v-model="newCmd.command" placeholder="命令" size="small" />
        <el-input v-model="newCmd.cwd" placeholder="工作目录（可选）" size="small" />
        <div class="cmd-add-actions">
          <el-button size="small" type="primary" @click="saveCustom">保存</el-button>
        </div>
      </div>

      <el-scrollbar max-height="220px">
        <div
          v-for="cmd in customCommands"
          :key="cmd.id"
          class="cmd-item"
        >
          <div class="cmd-item-main" @click="runCustom(cmd)">
            <div class="cmd-item-name">{{ cmd.name }}</div>
            <div class="cmd-item-desc">{{ cmd.description }}</div>
          </div>
          <el-button
            size="small"
            text
            type="danger"
            class="cmd-delete-btn"
            @click.stop="deleteCustom(cmd.id)"
          >
            <el-icon :size="14"><Delete /></el-icon>
          </el-button>
        </div>
        <div v-if="customCommands.length === 0 && !showAddForm" class="cmd-empty">暂无自定义命令</div>
      </el-scrollbar>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Delete } from '@element-plus/icons-vue'

interface CommandPreset {
  name: string
  description: string
  command: string
  is_launch: boolean
  is_resume: boolean
  is_init: boolean
}

interface CustomCommand {
  id: string
  name: string
  description: string
  command: string
  cwd: string
}

const presets = ref<CommandPreset[]>([])
const customCommands = ref<CustomCommand[]>([])
const showAddForm = ref(false)
const newCmd = ref({ name: '', description: '', command: '', cwd: '' })

onMounted(async () => {
  await loadPresets()
  await loadCustomCommands()
})

async function loadPresets() {
  try {
    presets.value = await invoke<CommandPreset[]>('agent_command_presets')
  } catch {
    presets.value = []
  }
}

async function loadCustomCommands() {
  try {
    customCommands.value = await invoke<CustomCommand[]>('agent_list_custom_commands')
  } catch {
    customCommands.value = []
  }
}

async function runPreset(command: string) {
  try {
    await invoke('agent_run_in_terminal', { command, cwd: null })
  } catch {}
}

async function runCustom(cmd: CustomCommand) {
  try {
    await invoke('agent_run_in_terminal', { command: cmd.command, cwd: cmd.cwd || null })
  } catch {}
}

async function saveCustom() {
  const { name, description, command, cwd } = newCmd.value
  if (!name.trim() || !command.trim()) return
  try {
    await invoke('agent_save_custom_command', {
      cmd: { id: '', name: name.trim(), description: description.trim(), command: command.trim(), cwd: cwd.trim() }
    })
    newCmd.value = { name: '', description: '', command: '', cwd: '' }
    showAddForm.value = false
    await loadCustomCommands()
  } catch {}
}

async function deleteCustom(id: string) {
  try {
    await invoke('agent_delete_custom_command', { id })
    await loadCustomCommands()
  } catch {}
}

function toggleAddForm() {
  showAddForm.value = !showAddForm.value
  if (!showAddForm.value) {
    newCmd.value = { name: '', description: '', command: '', cwd: '' }
  }
}
</script>

<style scoped>
.command-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.cmd-section {
  border-bottom: 1px solid var(--color-card-border);
}

.cmd-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
}

.cmd-item {
  display: flex;
  align-items: center;
  padding: 7px 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.cmd-item:hover {
  background: var(--color-hover-bg);
}

.cmd-item-main {
  flex: 1;
  min-width: 0;
}

.cmd-item-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.cmd-item-desc {
  font-size: 13px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}

.cmd-badge {
  font-size: 13px;
  padding: 0 4px;
  border-radius: 3px;
  font-weight: 500;
}

.cmd-badge.launch {
  color: #34C759;
  background: rgba(52, 199, 89, 0.1);
}

.cmd-badge.resume {
  color: #007AFF;
  background: rgba(0, 122, 255, 0.1);
}

.cmd-badge.init {
  color: #FF9500;
  background: rgba(255, 149, 0, 0.1);
}

.cmd-delete-btn {
  flex-shrink: 0;
  margin-left: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.cmd-item:hover .cmd-delete-btn {
  opacity: 1;
}

.cmd-empty {
  padding: 12px 14px;
  font-size: 13px;
  color: var(--color-text-tertiary);
  text-align: center;
}

.cmd-add-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 14px 10px;
  border-bottom: 1px solid var(--color-card-border);
}

.cmd-add-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
