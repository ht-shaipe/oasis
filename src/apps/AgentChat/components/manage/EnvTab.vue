<template>
  <div class="env-tab">
    <div class="env-header">
      <h2 class="env-title">环境检查</h2>
      <div class="env-actions">
        <button class="icon-btn" @click="handleRefresh" title="刷新">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
        </button>
      </div>
    </div>

    <el-scrollbar class="env-scroll">
      <div class="env-section">
        <div class="env-section-title">运行时</div>
        <EnvCheckPanel />
      </div>

      <div class="env-section">
        <div class="env-section-title">
          本地 Agent CLI
          <span class="ka-count">{{ store.knownAgents.length }}</span>
        </div>
        <p class="env-section-desc">检测本地已安装的 AI Agent 命令行工具及版本</p>

        <div v-if="installedAgents.length" class="agent-group">
          <div class="agent-group-label">已安装 ({{ installedAgents.length }})</div>
          <div class="known-agents-grid">
            <div
              v-for="agent in installedAgents"
              :key="agent.id"
              class="known-agent-card installed"
            >
              <div class="ka-header">
                <div class="ka-status-dot installed" />
                <span class="ka-name">{{ agent.display_name }}</span>
                <span v-if="agent.version" class="ka-version">{{ shortVersion(agent.version) }}</span>
              </div>
              <div class="ka-desc">{{ agent.description }}</div>
              <div class="ka-binary">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
                </svg>
                <span>{{ agent.binary }}</span>
              </div>
              <div class="ka-actions">
                <a
                  v-if="agent.home_url"
                  class="ka-link"
                  href="#"
                  @click.prevent="openUrl(agent.home_url)"
                >
                  主页
                </a>
              </div>
            </div>
          </div>
        </div>

        <div v-if="notInstalledAgents.length" class="agent-group">
          <div class="agent-group-label">未安装 ({{ notInstalledAgents.length }})</div>
          <div class="known-agents-grid">
            <div
              v-for="agent in notInstalledAgents"
              :key="agent.id"
              class="known-agent-card"
            >
              <div class="ka-header">
                <div class="ka-status-dot" />
                <span class="ka-name">{{ agent.display_name }}</span>
              </div>
              <div class="ka-desc">{{ agent.description }}</div>
              <div class="ka-actions">
                <button
                  v-if="agent.install_command"
                  class="ka-install-btn"
                  @click="handleInstallKnown(agent.install_command)"
                >
                  安装
                </button>
                <a
                  v-if="agent.home_url"
                  class="ka-link"
                  href="#"
                  @click.prevent="openUrl(agent.home_url)"
                >
                  详情
                </a>
                <span v-if="agent.install_hint && !agent.install_command" class="ka-hint">{{ agent.install_hint }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { useAgentStore } from '@/store/agent'
import EnvCheckPanel from '../EnvCheckPanel.vue'

const store = useAgentStore()

const installedAgents = computed(() => store.knownAgents.filter(a => a.installed))
const notInstalledAgents = computed(() => store.knownAgents.filter(a => !a.installed))

onMounted(() => {
  store.checkEnvironment()
  store.loadAgentStatuses()
  store.probeKnownAgents()
})

function handleRefresh() {
  store.checkEnvironment()
  store.loadAgentStatuses()
  store.probeKnownAgents()
}

function handleInstallAgent(agentId: string) {
  const status = store.agentStatuses.find(a => a.id === agentId)
  if (status?.install_hint) {
    window.open(`terminal://run?cmd=${encodeURIComponent(status.native_install_command || status.install_hint)}`, '_blank')
  }
}

async function openUrl(url: string) {
  try {
    await invoke('plugin:opener|open_url', { url })
  } catch {
    window.open(url, '_blank')
  }
}

async function handleInstallKnown(command: string) {
  try {
    await invoke('agent_install_agent', { command })
    ElMessage.success('安装命令已执行')
    setTimeout(() => store.probeKnownAgents(), 3000)
  } catch (e) {
    ElMessage.error(`安装失败: ${e}`)
  }
}

function shortVersion(ver: string): string {
  const firstLine = ver.split('\n')[0] || ver
  return firstLine.length > 40 ? firstLine.slice(0, 40) + '…' : firstLine
}
</script>

<style scoped>
.env-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.env-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 24px 0;
}

.env-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.env-actions {
  display: flex;
  gap: 8px;
}

.icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid var(--color-card-border);
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.env-scroll {
  flex: 1;
  padding: 16px 24px 24px;
}

.env-section {
  margin-bottom: 28px;
}

.env-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.env-section-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin: 0 0 10px;
}

.ka-count {
  font-size: 12px;
  font-weight: 400;
  background: var(--color-sidebar-item-hover);
  padding: 1px 6px;
  border-radius: 8px;
  color: var(--color-text-tertiary);
}

.agent-group {
  margin-bottom: 16px;
}

.agent-group-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  margin-bottom: 8px;
}

.known-agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}

.known-agent-card {
  padding: 12px 14px;
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
  background: var(--color-card-bg);
  transition: all 0.15s;
}

.known-agent-card.installed {
  border-color: rgba(52, 199, 89, 0.3);
}

.known-agent-card:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.04);
}

.ka-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.ka-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--el-color-danger-light-5);
  flex-shrink: 0;
}

.ka-status-dot.installed {
  background: #34C759;
  box-shadow: 0 0 4px rgba(52, 199, 89, 0.4);
}

.ka-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.ka-version {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-family: monospace;
  background: var(--color-sidebar-item-hover);
  padding: 1px 5px;
  border-radius: 4px;
  margin-left: auto;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ka-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 6px;
  line-height: 1.4;
}

.ka-binary {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-family: monospace;
  color: #34C759;
  margin-bottom: 6px;
}

.ka-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.ka-install-btn {
  padding: 3px 10px;
  border-radius: 5px;
  border: none;
  background: #007AFF;
  color: #fff;
  font-size: 12px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.ka-install-btn:hover {
  opacity: 0.85;
}

.ka-link {
  font-size: 12px;
  color: #007AFF;
  text-decoration: none;
}

.ka-link:hover {
  text-decoration: underline;
}

.ka-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-family: monospace;
}
</style>