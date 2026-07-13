<template>
  <div class="agent-msg" :class="[role]">
    <div class="agent-msg-avatar" v-if="role === 'assistant'">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2a4 4 0 0 1 4 4v2a4 4 0 0 1-8 0V6a4 4 0 0 1 4-4z"/>
        <path d="M6 12h12a4 4 0 0 1 4 4v2a4 4 0 0 1-8 0v-2H10v2a4 4 0 0 1-8 0v-2a4 4 0 0 1 4-4z"/>
      </svg>
    </div>
    <div class="agent-msg-body">
      <div class="agent-msg-bubble" :class="{ 'user-bubble': role === 'user', 'ai-bubble': role === 'assistant' }">
        <div v-if="thinking" class="thinking-section">
          <div class="thinking-toggle" @click="thinkingExpanded = !thinkingExpanded">
            <svg class="thinking-arrow" :class="{ expanded: thinkingExpanded }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
            <span class="thinking-label">思考过程</span>
          </div>
          <div v-if="thinkingExpanded" class="thinking-content" v-html="renderMarkdown(thinking)"></div>
        </div>

        <div v-if="content" class="markdown-body" v-html="renderMarkdown(content)"></div>

        <AgentToolCard
          v-for="tool in tools"
          :key="tool.id"
          :tool="tool"
        />

        <div v-if="error" class="agent-msg-error">{{ error }}</div>

        <div v-if="streaming && !content && !thinking" class="streaming-hint">
          <span class="streaming-dot">●</span> 思考中...
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { marked } from 'marked'
import type { ToolCall } from '@/store/agent'
import AgentToolCard from './AgentToolCard.vue'

const props = defineProps<{
  role: 'user' | 'assistant'
  content: string
  thinking?: string
  tools?: ToolCall[]
  error?: string
  streaming?: boolean
}>()

const thinkingExpanded = ref(false)

function renderMarkdown(text: string): string {
  if (!text) return ''
  try {
    return marked.parse(text) as string
  } catch {
    return text.replace(/\n/g, '<br>')
  }
}
</script>

<style scoped>
.agent-msg {
  display: flex;
  gap: 10px;
  padding: 8px 20px;
  max-width: 100%;
}

.agent-msg.user {
  flex-direction: row-reverse;
}

.agent-msg-avatar {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 2px;
}

.agent-msg-body {
  min-width: 0;
  max-width: 75%;
}

.agent-msg-bubble {
  padding: 10px 14px;
  border-radius: 16px;
  font-size: var(--app-font-14);
  line-height: 1.6;
  word-break: break-word;
}

.user-bubble {
  background: #007AFF;
  color: #fff;
  border-bottom-right-radius: 6px;
}

.ai-bubble {
  background: var(--color-card-bg);
  color: var(--color-text-primary);
  border: 1px solid var(--color-card-border);
  border-bottom-left-radius: 6px;
}

.thinking-section {
  margin-bottom: 8px;
  border-left: 2px solid var(--color-link, #007AFF);
  padding-left: 10px;
}

.thinking-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  padding: 2px 0;
  color: var(--color-text-tertiary);
  transition: color 0.15s;
}

.thinking-toggle:hover {
  color: var(--color-text-secondary);
}

.thinking-arrow {
  transition: transform 0.2s;
}

.thinking-arrow.expanded {
  transform: rotate(180deg);
}

.thinking-label {
  font-size: var(--app-font-13);
  font-style: italic;
}

.thinking-content {
  margin-top: 6px;
  color: var(--color-text-tertiary);
  font-size: var(--app-font-13);
  white-space: pre-wrap;
  word-break: break-word;
}

.agent-msg-error {
  margin-top: 6px;
  padding: 6px 10px;
  background: rgba(245, 108, 108, 0.1);
  border-radius: 8px;
  color: var(--el-color-danger);
  font-size: var(--app-font-12);
}

.streaming-hint {
  color: var(--color-text-tertiary);
  font-size: var(--app-font-13);
  display: flex;
  align-items: center;
  gap: 6px;
}

.streaming-dot {
  color: #007AFF;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.markdown-body :deep(p) {
  margin: 0 0 8px;
}
.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(pre) {
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 8px;
  padding: 12px;
  overflow-x: auto;
}
.markdown-body :deep(code) {
  font-size: var(--app-font-13);
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
}
.markdown-body :deep(p > code) {
  background: var(--code-bg-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
}
</style>
