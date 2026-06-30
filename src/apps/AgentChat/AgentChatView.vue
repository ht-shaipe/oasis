<template>
  <div class="agent-chat">
    <!-- Sidebar: Project & Session list -->
    <div class="agent-sidebar">
      <div class="sidebar-section">
        <div class="sidebar-header">
          <span>Agent</span>
          <el-button size="small" text @click="handleRefresh">刷新</el-button>
        </div>
        <AgentHealthPanel
          :statuses="store.agentStatuses"
          :active-id="store.activeAgentId"
          @select="(id: string) => store.setActiveAgent(id)"
          @install="handleInstall"
        />
      </div>

      <!-- Project list -->
      <div class="sidebar-section">
        <div class="sidebar-header">
          <span>项目</span>
        </div>
        <el-scrollbar class="project-list">
          <div
            v-for="project in store.projects"
            :key="project.encoded_name + '-' + project.agent_id"
            class="project-item"
            :class="{ active: store.activeProjectPath === project.path }"
            @click="store.selectProject(project.path)"
          >
            <div class="project-name">
              {{ project.name }}
              <span class="project-agent-badge">{{ project.agent_id }}</span>
            </div>
            <div class="project-meta">
              {{ project.session_count }} 会话
              <span v-if="project.last_active"> · {{ project.last_active.slice(0, 10) }}</span>
            </div>
          </div>
          <div v-if="store.projects.length === 0" class="empty-hint">
            暂无项目，请在终端运行 claude 初始化
          </div>
        </el-scrollbar>
      </div>

      <!-- Session list -->
      <div v-if="store.activeProjectPath" class="sidebar-section session-section">
        <div class="sidebar-header">会话</div>
        <el-scrollbar class="session-list">
          <div
            v-for="session in store.sessions"
            :key="session.id"
            class="session-item"
            :class="{ active: store.activeSessionId === session.id }"
            @click="store.selectSession(session.id)"
          >
            <div class="session-name">{{ session.display_name || session.id.slice(0, 12) }}</div>
            <div class="session-meta">
              {{ session.message_count }} 条消息
              <span v-if="session.last_active"> · {{ session.last_active.slice(0, 10) }}</span>
            </div>
          </div>
        </el-scrollbar>
      </div>
    </div>

    <!-- Main chat area -->
    <div class="agent-main">
      <!-- Session history view -->
      <template v-if="store.sessionMessages.length > 0 && !store.activeChatSession">
        <div class="chat-header">
          <span class="chat-title">会话历史</span>
          <el-button size="small" text @click="handleOpenTerminal(store.activeProjectPath!, store.activeSessionId!)">
            在终端中打开
          </el-button>
        </div>
        <el-scrollbar class="chat-messages" ref="messagesRef">
          <div
            v-for="(msg, i) in store.sessionMessages"
            :key="i"
            class="message"
            :class="'message-' + msg.role"
          >
            <div class="message-role">{{ msg.role === 'user' ? '你' : store.activeAgent?.display_name }}</div>
            <div class="message-content" v-html="renderMarkdown(msg.content)"></div>
          </div>
        </el-scrollbar>
      </template>

      <!-- Active streaming chat -->
      <template v-else-if="store.activeChatSession">
        <div class="chat-header">
          <span class="chat-title">对话中</span>
          <el-button
            v-if="store.activeChatSession.isStreaming"
            size="small"
            type="danger"
            text
            @click="store.abortChat(store.activeChatSession!.id)"
          >
            停止
          </el-button>
        </div>
        <el-scrollbar class="chat-messages" ref="messagesRef">
          <div
            v-for="msg in store.activeChatSession.messages"
            :key="msg.id"
            class="message"
            :class="'message-' + msg.role"
          >
            <div class="message-role">
              {{ msg.role === 'user' ? '你' : store.activeAgent?.display_name }}
              <span v-if="msg.streaming" class="streaming-dot">●</span>
            </div>

            <!-- Thinking section -->
            <div v-if="msg.thinking" class="thinking-block">
              <el-collapse>
                <el-collapse-item title="思考过程">
                  <pre class="thinking-content">{{ msg.thinking }}</pre>
                </el-collapse-item>
              </el-collapse>
            </div>

            <!-- Text content -->
            <div v-if="msg.content" class="message-content" v-html="renderMarkdown(msg.content)"></div>

            <!-- Tool call cards -->
            <AgentToolCard
              v-for="tool in msg.tools"
              :key="tool.id"
              :tool="tool"
            />

            <!-- Error -->
            <div v-if="msg.error" class="message-error">{{ msg.error }}</div>
          </div>
        </el-scrollbar>
      </template>

      <!-- Empty state -->
      <div v-else class="chat-empty">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
            <path d="M12 2a3 3 0 0 0-3 3v1H6a3 3 0 0 0-3 3v8a3 3 0 0 0 3 3h12a3 3 0 0 0 3-3V9a3 3 0 0 0-3-3h-3V5a3 3 0 0 0-3-3z"/>
          </svg>
        </div>
        <h3>选择项目开始对话</h3>
        <p>从左侧选择一个已初始化的项目，然后用 {{ store.activeAgent?.display_name || 'Agent' }} 开始对话</p>
      </div>

      <!-- Input -->
      <div class="chat-input-area" v-if="store.activeProjectPath">
        <div class="input-wrapper">
          <textarea
            ref="inputRef"
            v-model="inputText"
            class="input-field"
            placeholder="输入消息..."
            :disabled="isStreaming"
            :rows="1"
            @keydown="handleKeydown"
            @input="autoResize"
          />
          <button
            class="send-button"
            :class="{ active: inputText.trim() && !isStreaming }"
            :disabled="!inputText.trim() || isStreaming"
            @click="handleSend"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M2 21l21-9L2 3v7l15 2-15 2v7z"/>
            </svg>
          </button>
        </div>
        <div class="input-hint">Enter 发送 · Shift+Enter 换行{{ isStreaming ? ' · AI 回复中...' : '' }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { marked } from 'marked'
import { useAgentStore } from '@/store/agent'
import AgentToolCard from './components/AgentToolCard.vue'
import AgentHealthPanel from './components/AgentHealthPanel.vue'

const store = useAgentStore()
const inputText = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const messagesRef = ref<InstanceType<typeof import('element-plus')['ElScrollbar']> | null>(null)

const isStreaming = computed(() =>
  store.activeChatSession?.isStreaming ?? false
)

onMounted(() => {
  store.initialize()
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

function renderMarkdown(content: string): string {
  if (!content) return ''
  return marked(content) as string
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

async function handleSend() {
  const text = inputText.value.trim()
  if (!text || isStreaming.value) return

  inputText.value = ''
  nextTick(() => autoResize())

  // Add user message
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
}

function handleInstall(agentId: string) {
  const status = store.agentStatuses.find(a => a.id === agentId)
  if (status?.install_hint) {
    window.open(`terminal://run?cmd=${encodeURIComponent(status.native_install_command || status.install_hint)}`, '_blank')
  }
}

function handleOpenTerminal(path: string, sessionId: string) {
  store.openInTerminal(path, sessionId)
}

function autoResize() {
  const el = inputRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 120) + 'px'
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
.agent-chat {
  display: flex;
  height: 100%;
  background: var(--color-bg-glass);
}

.agent-sidebar {
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-card-border);
  display: flex;
  flex-direction: column;
  background: var(--color-sidebar-bg);
}

.sidebar-section {
  border-bottom: 1px solid var(--color-card-border);
}

.session-section {
  flex: 1;
  min-height: 0;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  font-size: var(--app-font-11);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
}

.agent-not-installed {
  padding: 8px 14px 12px;
}

.project-list,
.session-list {
  max-height: 200px;
}

.project-item,
.session-item {
  padding: 8px 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.project-item:hover,
.session-item:hover {
  background: var(--color-hover-bg);
}

.project-item.active,
.session-item.active {
  background: var(--color-selected-bg);
}

.project-name,
.session-name {
  font-size: 13px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-agent-badge {
  font-size: 13px;
  color: var(--color-text-tertiary);
  background: var(--color-hover-bg);
  border-radius: 4px;
  padding: 0 4px;
  margin-left: 6px;
}

.project-meta,
.session-meta {
  font-size: var(--app-font-11);
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.empty-hint {
  padding: 16px 14px;
  font-size: var(--app-font-12);
  color: var(--color-text-tertiary);
  text-align: center;
}

/* Main */
.agent-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 20px;
  border-bottom: 1px solid var(--color-card-border);
  flex-shrink: 0;
}

.chat-title {
  font-size: var(--app-font-13);
  font-weight: 600;
  color: var(--color-text-primary);
}

.chat-messages {
  flex: 1;
  padding: 12px 0;
}

.message {
  padding: 8px 20px;
}

.message-user {
  background: var(--color-message-user-bg);
}

.message-role {
  font-size: var(--app-font-12);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.streaming-dot {
  color: #007AFF;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.message-content {
  font-size: var(--app-font-14);
  line-height: 1.6;
  color: var(--color-text-primary);
}

.message-content :deep(pre) {
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 8px;
  padding: 12px;
  overflow-x: auto;
}

.message-content :deep(code) {
  font-size: var(--app-font-13);
}

.message-error {
  color: var(--el-color-danger);
  font-size: var(--app-font-13);
  margin-top: 4px;
}

.thinking-block {
  margin-bottom: 8px;
}

.thinking-content {
  font-size: var(--app-font-12);
  color: var(--color-text-tertiary);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--color-text-tertiary);
}

.chat-empty h3 {
  font-size: var(--app-font-15);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.chat-empty p {
  font-size: var(--app-font-13);
  margin: 0;
  text-align: center;
  max-width: 320px;
}

/* Input */
.chat-input-area {
  padding: 12px 20px 8px;
  border-top: 1px solid var(--color-card-border);
  background: var(--color-sidebar-bg);
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  background: var(--color-input-bg);
  border: 1px solid var(--color-input-border);
  border-radius: 20px;
  padding: 6px 8px 6px 16px;
  transition: border-color 0.2s;
}

.input-wrapper:focus-within {
  border-color: #007AFF;
}

.input-field {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--app-font-14);
  line-height: 1.5;
  resize: none;
  max-height: 120px;
  min-height: 24px;
  font-family: inherit;
  padding: 0;
}

.input-field::placeholder {
  color: var(--color-text-tertiary);
}

.send-button {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: #e5e5e7;
  color: #999;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.send-button.active {
  background: #007AFF;
  color: #fff;
}

.send-button:disabled {
  cursor: not-allowed;
}

.input-hint {
  padding: 4px 16px 0;
  font-size: var(--app-font-11);
  color: var(--color-text-tertiary);
}
</style>
