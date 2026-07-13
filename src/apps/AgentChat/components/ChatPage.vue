<template>
  <div class="chat-page">
    <div class="chat-sidebar" :class="{ collapsed: sidebarCollapsed }">
      <template v-if="!sidebarCollapsed">
        <div class="sidebar-section" style="background: var(--color-layer-1, var(--color-sidebar-bg))">
          <div class="project-card" v-if="store.activeProject">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="shrink-0" style="color: var(--icon-folder, #f0ad4e)">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span class="project-card-name" :title="projectDisplayName">{{ projectDisplayName }}</span>
            <button class="switch-btn" @click="$emit('switch-project')">
              <span>切换</span>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
            </button>
          </div>
          <div class="project-card" v-else>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="opacity: 0.4">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span style="opacity: 0.5; flex: 1; font-size: 13px; font-weight: 600">无项目</span>
            <button class="switch-btn" @click="$emit('switch-project')">
              <span>切换</span>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
            </button>
          </div>

          <div class="sidebar-actions">
            <button
              class="new-session-btn"
              :class="{ disabled: !store.activeProjectPath }"
              :disabled="!store.activeProjectPath"
              @click="handleNewSession"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              <span>新对话</span>
            </button>
            <button class="icon-btn-sm" @click="handleRefresh" title="刷新">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
              </svg>
            </button>
            <button class="icon-btn-sm" @click="sidebarCollapsed = true" title="收起">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/>
              </svg>
            </button>
          </div>

          <div class="sidebar-search">
            <el-input
              v-model="searchQuery"
              placeholder="搜索会话..."
              size="small"
              clearable
              class="search-input"
            />
          </div>
        </div>

        <div class="session-list">
          <button
            v-for="session in filteredSessions"
            :key="session.id"
            class="session-item"
            :class="{ active: store.activeSessionId === session.id }"
            @click="store.selectSession(session.id)"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--icon-message, #3b82f6)">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
            </svg>
            <span class="session-name">{{ session.display_name || session.id.slice(0, 8) }}</span>
            <span class="session-time" v-if="session.last_active">{{ formatRelativeTime(session.last_active) }}</span>
          </button>
          <div v-if="store.sessions.length === 0" class="list-empty">
            暂无会话
          </div>
        </div>
      </template>

      <template v-else>
        <div class="collapsed-sidebar" style="background: var(--color-layer-1, var(--color-sidebar-bg))">
          <button class="icon-btn-sm" @click="$emit('switch-project')" :title="store.activeProject?.name || '无项目'">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--icon-folder, #f0ad4e)">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          </button>
          <button class="icon-btn-sm" @click="sidebarCollapsed = false" title="展开">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2"/><line x1="15" y1="3" x2="15" y2="21"/>
            </svg>
          </button>
          <button
            class="icon-btn-sm"
            :class="{ disabled: !store.activeProjectPath }"
            :disabled="!store.activeProjectPath"
            @click="handleNewSession"
            title="新对话"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </button>
        </div>
        <div class="collapsed-body" />
      </template>
    </div>

    <div class="chat-area">
      <template v-if="!store.activeProjectPath">
        <div class="chat-empty-center">
          <div class="empty-icon-lg">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--icon-message, #3b82f6)">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
            </svg>
          </div>
          <span class="empty-hint">未选择项目</span>
          <button class="switch-project-btn" @click="$emit('switch-project')">
            切换项目
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
          </button>
        </div>
      </template>

      <template v-else-if="!store.activeSessionId && !store.activeChatSession">
        <div class="chat-empty-center">
          <h2 class="start-prompt">与 {{ projectDisplayName }} 开始对话</h2>
          <div class="start-composer-footer" v-if="store.activeProject">
            <span class="footer-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--icon-folder, #f0ad4e)">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <span>{{ projectDisplayName }}</span>
            </span>
            <span class="footer-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="3" width="20" height="18" rx="3"/><polyline points="7 10 10 13 7 16"/><line x1="13" y1="16" x2="17" y2="16"/>
              </svg>
              <span>本地模式</span>
            </span>
            <span class="footer-item" v-if="store.activeAgent">
              {{ store.activeAgent.display_name }}
            </span>
          </div>
          <ChatInput
            :disabled="isStreaming"
            placeholder="输入消息..."
            @send="handleSend"
            class="start-input"
          />
        </div>
      </template>

      <template v-else>
        <div class="chat-session-header">
          <span class="session-title">{{ activeSessionName }}</span>
          <span class="session-id" v-if="store.activeSessionId">{{ store.activeSessionId.slice(0, 8) }}</span>
          <div class="session-actions">
            <el-button v-if="isStreaming" size="small" type="danger" text @click="store.abortChat(store.activeChatSession!.id)">停止</el-button>
            <el-button v-if="store.activeSessionId && !store.activeChatSession" size="small" text @click="handleOpenTerminal(store.activeProjectPath!, store.activeSessionId!)">终端</el-button>
          </div>
        </div>

        <el-scrollbar class="chat-messages" ref="messagesRef">
          <template v-if="store.activeChatSession">
            <AgentMessage
              v-for="msg in store.activeChatSession.messages"
              :key="msg.id"
              :role="msg.role as 'user' | 'assistant'"
              :content="msg.content"
              :thinking="msg.thinking"
              :tools="msg.tools"
              :error="msg.error"
              :streaming="msg.streaming"
            />
          </template>
          <template v-else>
            <AgentMessage
              v-for="(msg, i) in store.sessionMessages"
              :key="i"
              :role="msg.role as 'user' | 'assistant'"
              :content="msg.content"
            />
          </template>
        </el-scrollbar>

        <div class="chat-input-area">
          <ChatInput
            :disabled="isStreaming"
            placeholder="输入消息..."
            @send="handleSend"
          />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useAgentStore } from '@/store/agent'
import ChatInput from '@/apps/Chat/components/ChatInput.vue'
import AgentMessage from './AgentMessage.vue'

defineEmits<{
  'switch-project': []
}>()

const store = useAgentStore()
const messagesRef = ref<InstanceType<typeof import('element-plus')['ElScrollbar']> | null>(null)
const searchQuery = ref('')
const sidebarCollapsed = ref(false)

const isStreaming = computed(() => store.activeChatSession?.isStreaming ?? false)

const projectDisplayName = computed(() => {
  if (!store.activeProject) return ''
  const meta = store.projectMetas[store.activeProject.encoded_name]
  return meta?.custom_name || store.activeProject.name
})

const activeSessionName = computed(() => {
  if (!store.activeSessionId) return ''
  const session = store.sessions.find(s => s.id === store.activeSessionId)
  return session?.display_name || store.activeSessionId.slice(0, 8)
})

const filteredSessions = computed(() => {
  let sessions = store.sessions
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    sessions = sessions.filter(s =>
      (s.display_name || s.id).toLowerCase().includes(q)
    )
  }
  return sessions
})

onMounted(() => {
  store.initialize()
  store.loadProjectMetas()
})

onUnmounted(() => {
  store.teardownStreamListener()
})

watch(
  () => store.activeChatSession?.messages.length,
  () => scrollToBottom()
)

watch(
  () => store.activeChatSession?.messages.slice(-1)?.[0]?.content,
  () => scrollToBottom()
)

function handleNewSession() {
  if (!store.activeProjectPath) return
  store.clearActiveSession()
}

async function handleSend(text: string) {
  if (!text || isStreaming.value) return
  if (store.activeChatSession) {
    store.activeChatSession.messages.push({
      id: `user-${Date.now()}`,
      role: 'user',
      content: text,
      thinking: '',
      timestamp: Date.now(),
      streaming: false,
      error: '',
      tools: [],
      contentBlocks: [{ type: 'text', text }],
    })
  }
  await store.sendMessage(text)
  scrollToBottom()
}

function handleRefresh() {
  store.loadProjects()
  store.loadAgentStatuses()
  store.loadProjectMetas()
}

function handleOpenTerminal(path: string, sessionId: string) {
  store.openInTerminal(path, sessionId)
}

function formatRelativeTime(dateStr: string): string {
  const d = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return '刚刚'
  if (diffMin < 60) return `${diffMin}分钟前`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}小时前`
  const diffDay = Math.floor(diffHr / 24)
  if (diffDay < 7) return `${diffDay}天前`
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  return `${mm}-${dd}`
}

function scrollToBottom() {
  nextTick(() => {
    const scrollbar = messagesRef.value
    if (scrollbar) {
      scrollbar.setScrollTop(scrollbar.wrapRef?.scrollHeight ?? 99999)
    }
  })
}
</script>

<style scoped>
.chat-page {
  display: flex;
  height: 100%;
}

.chat-sidebar {
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-sidebar-border);
  background: var(--color-sidebar-bg);
  transition: width 0.15s;
}

.chat-sidebar.collapsed {
  width: 56px;
}

.sidebar-section {
  display: flex;
  flex-direction: column;
}

.project-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 40px;
  border-bottom: 1px solid var(--color-sidebar-border);
}

.project-card-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.switch-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px;
  height: 24px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.switch-btn:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.sidebar-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px 4px;
  height: 44px;
}

.new-session-btn {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  height: 32px;
  padding: 0 8px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}

.new-session-btn:hover:not(.disabled) {
  background: var(--color-sidebar-item-hover);
}

.new-session-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.icon-btn-sm {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.icon-btn-sm:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.sidebar-search {
  padding: 0 12px 8px;
  height: 40px;
}

.search-input :deep(.el-input__wrapper) {
  border-radius: 8px;
  box-shadow: none !important;
  border: 1px solid var(--color-card-border);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 8px 20px;
  border: none;
  border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
}

.session-item:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.session-item.active {
  background: rgba(0, 122, 255, 0.1);
  color: var(--color-text-primary);
  font-weight: 500;
}

.session-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-time {
  font-size: 10px;
  color: var(--color-text-tertiary);
  opacity: 0.5;
  flex-shrink: 0;
}

.list-empty {
  padding: 20px 12px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.collapsed-sidebar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
}

.collapsed-body {
  flex: 1;
}

.chat-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--color-bg-glass);
}

.chat-empty-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 24px;
}

.empty-icon-lg {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  background: var(--color-sidebar-item-hover);
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-hint {
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.switch-project-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: #007AFF;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}

.switch-project-btn:hover {
  background: rgba(0, 122, 255, 0.1);
}

.start-prompt {
  font-size: 32px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0 0 16px;
  text-align: center;
  max-width: 600px;
}

.start-composer-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 16px;
  padding: 10px 16px;
  border-top: 1px solid var(--color-card-border);
  border-radius: 8px;
  background: var(--color-sidebar-item-hover);
  margin-bottom: 16px;
  width: 100%;
  max-width: 680px;
}

.footer-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.footer-item span {
  font-weight: 500;
  color: var(--color-text-primary);
}

.start-input {
  width: 100%;
  max-width: 680px;
}

.chat-session-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 20px;
  height: 44px;
  border-bottom: 1px solid var(--color-card-border);
  background: var(--color-layer-1, var(--color-sidebar-bg));
  flex-shrink: 0;
}

.session-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-id {
  font-size: 11px;
  font-family: monospace;
  color: var(--color-text-tertiary);
  opacity: 0.5;
  flex-shrink: 0;
}

.session-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.chat-messages {
  flex: 1;
}

.chat-input-area {
  flex-shrink: 0;
}
</style>
