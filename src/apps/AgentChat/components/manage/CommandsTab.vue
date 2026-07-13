<template>
  <div class="commands-tab">
    <div class="commands-header">
      <h2 class="commands-title">命令</h2>
    </div>
    <el-scrollbar class="commands-scroll">
      <div class="cmd-section">
        <div class="cmd-section-header">
          <span>预设命令</span>
        </div>
        <div
          v-for="preset in store.commandPresets"
          :key="preset.name"
          class="cmd-item"
          @click="runPreset(preset)"
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
          <span v-if="runningCmd === preset.name" class="cmd-running">运行中...</span>
        </div>
        <div v-if="store.commandPresets.length === 0" class="cmd-empty">暂无预设命令</div>
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
            <el-button size="small" @click="toggleAddForm">取消</el-button>
            <el-button size="small" type="primary" @click="saveCustom">保存</el-button>
          </div>
        </div>

        <div
          v-for="cmd in store.customCommands"
          :key="cmd.id"
          class="cmd-item"
        >
          <div class="cmd-item-main" @click="runCustom(cmd)">
            <div class="cmd-item-name">{{ cmd.name }}</div>
            <div class="cmd-item-desc">{{ cmd.description }}</div>
            <div v-if="cmd.cwd" class="cmd-item-cwd">{{ cmd.cwd }}</div>
          </div>
          <div class="cmd-item-actions">
            <el-button
              size="small"
              text
              @click.stop="editCustom(cmd)"
            >
              编辑
            </el-button>
            <el-button
              size="small"
              text
              type="danger"
              @click.stop="deleteCustom(cmd.id)"
            >
              删除
            </el-button>
          </div>
        </div>
        <div v-if="store.customCommands.length === 0 && !showAddForm" class="cmd-empty">暂无自定义命令</div>
      </div>
    </el-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAgentStore } from '@/store/agent'
import type { CustomCommand, AgentCommandPreset } from '@/store/agent'

const store = useAgentStore()
const showAddForm = ref(false)
const editingCmd = ref<CustomCommand | null>(null)
const runningCmd = ref<string | null>(null)
const newCmd = ref({ name: '', description: '', command: '', cwd: '' })

onMounted(() => {
  store.loadCommandPresets()
  store.loadCustomCommands()
})

function toggleAddForm() {
  showAddForm.value = !showAddForm.value
  if (!showAddForm.value) {
    newCmd.value = { name: '', description: '', command: '', cwd: '' }
    editingCmd.value = null
  }
}

async function runPreset(preset: AgentCommandPreset) {
  runningCmd.value = preset.name
  try {
    await store.runInTerminal(preset.command)
  } finally {
    setTimeout(() => { runningCmd.value = null }, 2000)
  }
}

async function runCustom(cmd: CustomCommand) {
  runningCmd.value = cmd.name
  try {
    await store.runInTerminal(cmd.command, cmd.cwd || undefined)
  } finally {
    setTimeout(() => { runningCmd.value = null }, 2000)
  }
}

function editCustom(cmd: CustomCommand) {
  editingCmd.value = cmd
  newCmd.value = { name: cmd.name, description: cmd.description, command: cmd.command, cwd: cmd.cwd }
  showAddForm.value = true
}

async function saveCustom() {
  const { name, description, command, cwd } = newCmd.value
  if (!name.trim() || !command.trim()) return
  try {
    await store.saveCustomCommand({
      id: editingCmd.value?.id || '',
      name: name.trim(),
      description: description.trim(),
      command: command.trim(),
      cwd: cwd.trim(),
    })
    newCmd.value = { name: '', description: '', command: '', cwd: '' }
    showAddForm.value = false
    editingCmd.value = null
  } catch {}
}

async function deleteCustom(id: string) {
  try {
    await store.deleteCustomCommand(id)
  } catch {}
}
</script>

<style scoped>
.commands-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.commands-header {
  padding: 24px 24px 0;
}

.commands-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 16px;
}

.commands-scroll {
  flex: 1;
  padding: 0 24px 24px;
}

.cmd-section {
  border-bottom: 1px solid var(--color-card-border);
  margin-bottom: 16px;
  padding-bottom: 16px;
}

.cmd-section:last-child {
  border-bottom: none;
}

.cmd-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 8px;
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
}

.cmd-item {
  display: flex;
  align-items: center;
  padding: 7px 0;
  cursor: pointer;
  transition: background 0.15s;
  border-radius: 6px;
}

.cmd-item:hover {
  background: var(--color-sidebar-item-hover);
}

.cmd-item-main {
  flex: 1;
  min-width: 0;
  padding: 0 8px;
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
  font-size: 12px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}

.cmd-item-cwd {
  font-size: 11px;
  font-family: monospace;
  color: var(--color-text-tertiary);
  opacity: 0.6;
  margin-top: 2px;
}

.cmd-item-actions {
  flex-shrink: 0;
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
  padding-right: 4px;
}

.cmd-item:hover .cmd-item-actions {
  opacity: 1;
}

.cmd-badge {
  font-size: 11px;
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

.cmd-running {
  font-size: 12px;
  color: #007AFF;
  flex-shrink: 0;
}

.cmd-empty {
  padding: 12px 8px;
  font-size: 13px;
  color: var(--color-text-tertiary);
  text-align: center;
}

.cmd-add-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 8px 10px;
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  margin-bottom: 8px;
}

.cmd-add-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
