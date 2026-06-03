<template>
  <div class="chat-message" :class="[role, { streaming }]">
    <div class="message-avatar" v-if="role === 'assistant'">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2a4 4 0 0 1 4 4v2a4 4 0 0 1-8 0V6a4 4 0 0 1 4-4z"/>
        <path d="M6 12h12a4 4 0 0 1 4 4v2a4 4 0 0 1-8 0v-2H10v2a4 4 0 0 1-8 0v-2a4 4 0 0 1 4-4z"/>
      </svg>
    </div>
    <div class="message-body">
      <div class="message-bubble" :class="{ 'user-bubble': role === 'user', 'ai-bubble': role === 'assistant' }">
        <!-- 流式输出中：纯文本逐字显示 -->
        <div v-if="streaming" class="message-content streaming-content">
          {{ content }}
          <span class="cursor-blink">|</span>
        </div>
        <!-- 完成：渲染 Markdown -->
        <div v-else class="message-content markdown-body" v-html="renderedContent" />
        <!-- 错误提示 -->
        <div v-if="error" class="message-error">{{ error }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps<{
  role: 'user' | 'assistant'
  content: string
  streaming?: boolean
  error?: string
}>()

// 配置 marked 渲染器
marked.setOptions({
  breaks: true,
  gfm: true,
})

const renderedContent = computed(() => {
  if (!props.content) return ''
  try {
    return marked.parse(props.content) as string
  } catch {
    return props.content.replace(/\n/g, '<br>')
  }
})
</script>

<style scoped>
.chat-message {
  display: flex;
  gap: 10px;
  padding: 8px 20px;
  max-width: 100%;
}

.chat-message.user {
  flex-direction: row-reverse;
}

.message-avatar {
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

.message-body {
  min-width: 0;
  max-width: 75%;
}

.message-bubble {
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

.streaming-content {
  white-space: pre-wrap;
  min-height: 1.6em;
}

.cursor-blink {
  animation: blink 1s step-end infinite;
  color: var(--color-text-primary);
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.message-error {
  margin-top: 6px;
  padding: 6px 10px;
  background: rgba(245, 108, 108, 0.1);
  border-radius: 8px;
  color: var(--color-danger);
  font-size: var(--app-font-12);
}

/* Markdown 渲染样式 */
.markdown-body :deep(p) {
  margin: 0 0 8px;
}
.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(pre) {
  background: var(--code-bg-secondary);
  border: 1px solid var(--code-border);
  border-radius: 6px;
  padding: 12px;
  overflow-x: auto;
  margin: 8px 0;
  font-size: var(--app-font-13);
}
.markdown-body :deep(code) {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  font-size: var(--app-font-13);
}
.markdown-body :deep(p > code) {
  background: var(--code-bg-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
}
.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 20px;
  margin: 8px 0;
}
.markdown-body :deep(li) {
  margin: 4px 0;
}
.markdown-body :deep(blockquote) {
  border-left: 3px solid var(--color-link);
  padding-left: 12px;
  margin: 8px 0;
  color: var(--color-text-secondary);
}
.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 8px 0;
}
.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--color-card-border);
  padding: 6px 10px;
  text-align: left;
}
.markdown-body :deep(th) {
  background: var(--color-card-bg);
}
.markdown-body :deep(a) {
  color: var(--color-link);
}
.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-card-border);
  margin: 12px 0;
}
</style>
