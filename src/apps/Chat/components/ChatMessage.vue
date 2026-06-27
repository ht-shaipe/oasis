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
        <div v-if="reasoningContent" class="reasoning-section">
          <div class="reasoning-toggle" @click="showReasoning = !showReasoning">
            <svg class="reasoning-arrow" :class="{ expanded: showReasoning }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
            <span class="reasoning-label">{{ showReasoning ? t('chat.hideThinking') : t('chat.showThinking') }}</span>
          </div>
          <div v-if="showReasoning" class="reasoning-content" v-html="renderedReasoning" />
        </div>
        <div v-if="streaming && !reasoningContent" class="message-content streaming-content">
          {{ content }}
          <span class="cursor-blink">|</span>
        </div>
        <div v-else-if="streaming && reasoningContent && !content" class="message-content streaming-content reasoning-streaming">
          {{ reasoningContent }}
          <span class="cursor-blink">|</span>
        </div>
        <div v-else class="message-content markdown-body" ref="markdownRef" v-html="renderedContent" />
        <div v-if="error" class="message-error">{{ error }}</div>
      </div>
      <div v-if="!streaming && content" class="message-actions">
        <el-tooltip :content="t('chat.copyMessage')" placement="top" :show-after="500">
          <button class="action-btn" @click="handleCopyMessage">
            <svg v-if="!messageCopied" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
          </button>
        </el-tooltip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { marked } from 'marked'
import hljs from 'highlight.js'

const { t } = useI18n()

const props = defineProps<{
  role: 'user' | 'assistant'
  content: string
  reasoningContent?: string
  streaming?: boolean
  error?: string
}>()

const showReasoning = ref(false)
const messageCopied = ref(false)
const markdownRef = ref<HTMLElement | null>(null)

const renderer = new marked.Renderer()
renderer.code = function ({ text, lang }: { text: string; lang?: string; escaped?: boolean }) {
  const langAttr = lang ? ` class="hljs language-${lang}"` : ' class="hljs"'
  let highlighted: string
  if (lang && hljs.getLanguage(lang)) {
    highlighted = hljs.highlight(text, { language: lang }).value
  } else {
    highlighted = hljs.highlightAuto(text).value
  }
  const langLabel = lang ? `<span class="code-lang">${lang}</span>` : ''
  return `<div class="code-block-wrapper"><div class="code-block-header">${langLabel}<button class="code-copy-btn" title="${t('chat.copyCode')}"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div><pre><code${langAttr}>${highlighted}</code></pre></div>`
}

marked.setOptions({
  breaks: true,
  gfm: true,
  renderer,
})

const renderedContent = computed(() => {
  if (!props.content) return ''
  try {
    return marked.parse(props.content) as string
  } catch {
    return props.content.replace(/\n/g, '<br>')
  }
})

const renderedReasoning = computed(() => {
  if (!props.reasoningContent) return ''
  try {
    return marked.parse(props.reasoningContent) as string
  } catch {
    return props.reasoningContent.replace(/\n/g, '<br>')
  }
})

function copyText(text: string): Promise<void> {
  if (navigator.clipboard && navigator.clipboard.writeText) {
    return navigator.clipboard.writeText(text)
  }
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.select()
  return new Promise<void>((resolve, reject) => {
    document.execCommand('copy') ? resolve() : reject()
    document.body.removeChild(textarea)
  })
}

function handleCopyMessage() {
  copyText(props.content).then(() => {
    messageCopied.value = true
    setTimeout(() => { messageCopied.value = false }, 2000)
  })
}

function setupCodeCopyButtons() {
  if (!markdownRef.value) return
  const btns = markdownRef.value.querySelectorAll('.code-copy-btn')
  btns.forEach((btn) => {
    btn.removeEventListener('click', onCodeCopyClick)
    btn.addEventListener('click', onCodeCopyClick)
  })
}

function onCodeCopyClick(e: Event) {
  const btn = e.currentTarget as HTMLElement
  const wrapper = btn.closest('.code-block-wrapper')
  const codeEl = wrapper?.querySelector('code')
  if (!codeEl) return
  const code = codeEl.textContent || ''
  copyText(code).then(() => {
    const originalSvg = btn.innerHTML
    btn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`
    btn.classList.add('copied')
    setTimeout(() => {
      btn.innerHTML = originalSvg
      btn.classList.remove('copied')
    }, 2000)
  })
}

watch(renderedContent, () => {
  nextTick(setupCodeCopyButtons)
})

watch(() => props.streaming, (streaming) => {
  if (!streaming) {
    nextTick(setupCodeCopyButtons)
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

.reasoning-streaming {
  color: var(--color-text-tertiary);
  font-style: italic;
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
  color: var(--danger);
  font-size: var(--app-font-12);
}

.reasoning-section {
  margin-bottom: 8px;
  border-left: 2px solid var(--color-link);
  padding-left: 10px;
}

.reasoning-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  padding: 2px 0;
  color: var(--color-text-tertiary);
  transition: color 0.15s;
}

.reasoning-toggle:hover {
  color: var(--color-text-secondary);
}

.reasoning-arrow {
  transition: transform 0.2s;
}

.reasoning-arrow.expanded {
  transform: rotate(180deg);
}

.reasoning-label {
  font-size: var(--app-font-13);
  font-style: italic;
}

.reasoning-content {
  margin-top: 6px;
  color: var(--color-text-tertiary);
  font-size: var(--app-font-13);
}

.reasoning-content :deep(p) {
  margin: 0 0 4px;
}

.reasoning-content :deep(p:last-child) {
  margin-bottom: 0;
}

.message-actions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.chat-message:hover .message-actions {
  opacity: 1;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover {
  background: var(--color-card-bg);
  color: var(--color-text-primary);
}

.action-btn svg {
  display: block;
}

.chat-message.user .action-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

/* Markdown */
.markdown-body :deep(p) {
  margin: 0 0 8px;
}
.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(.code-block-wrapper) {
  position: relative;
  background: var(--code-bg-secondary);
  border: 1px solid var(--code-border);
  border-radius: 6px;
  margin: 8px 0;
}
.markdown-body :deep(.code-block-header) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px 4px 12px;
  border-bottom: 1px solid var(--code-border);
  background: rgba(0, 0, 0, 0.04);
}
.markdown-body :deep(.code-lang) {
  font-size: 11px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  pointer-events: none;
  line-height: 20px;
}
.markdown-body :deep(.code-copy-btn) {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s;
  padding: 0;
}
.markdown-body :deep(.code-copy-btn:hover) {
  background: rgba(0, 0, 0, 0.08);
  color: var(--color-text-primary);
}
.markdown-body :deep(.code-copy-btn.copied) {
  color: #67c23a;
}
.markdown-body :deep(.code-copy-btn svg) {
  display: block;
}
.markdown-body :deep(pre) {
  background: transparent;
  border: none;
  border-radius: 0 0 6px 6px;
  padding: 12px;
  overflow-x: auto;
  margin: 0;
  font-size: var(--app-font-13);
}
.markdown-body :deep(pre::-webkit-scrollbar) {
  height: 6px;
}
.markdown-body :deep(pre::-webkit-scrollbar-track) {
  background: transparent;
}
.markdown-body :deep(pre::-webkit-scrollbar-thumb) {
  background: rgba(144, 147, 153, 0.3);
  border-radius: 3px;
}
.markdown-body :deep(pre::-webkit-scrollbar-thumb:hover) {
  background: rgba(144, 147, 153, 0.5);
}
.markdown-body :deep(pre code) {
  display: block;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  font-size: var(--app-font-13);
}
.markdown-body :deep(.hljs) {
  background: transparent;
  padding: 0;
  color: var(--color-text-primary);
}
.markdown-body :deep(.hljs-keyword),
.markdown-body :deep(.hljs-selector-tag),
.markdown-body :deep(.hljs-built_in) {
  color: #c678dd;
}
.markdown-body :deep(.hljs-string),
.markdown-body :deep(.hljs-attr),
.markdown-body :deep(.hljs-template-variable) {
  color: #98c379;
}
.markdown-body :deep(.hljs-comment),
.markdown-body :deep(.hljs-doctag) {
  color: #5c6370;
  font-style: italic;
}
.markdown-body :deep(.hljs-number),
.markdown-body :deep(.hljs-literal) {
  color: #d19a66;
}
.markdown-body :deep(.hljs-title),
.markdown-body :deep(.hljs-section),
.markdown-body :deep(.hljs-function) {
  color: #61afef;
}
.markdown-body :deep(.hljs-type),
.markdown-body :deep(.hljs-params) {
  color: #e5c07b;
}
.markdown-body :deep(.hljs-meta) {
  color: #56b6c2;
}
.markdown-body :deep(.hljs-symbol),
.markdown-body :deep(.hljs-bullet) {
  color: #e06c75;
}
.markdown-body :deep(.hljs-addition) {
  color: #98c379;
  background: rgba(152, 195, 121, 0.1);
}
.markdown-body :deep(.hljs-deletion) {
  color: #e06c75;
  background: rgba(224, 108, 117, 0.1);
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
