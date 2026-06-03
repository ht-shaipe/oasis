<template>
  <div class="chat-input-area">
    <div class="input-wrapper">
      <textarea
        ref="textareaRef"
        v-model="inputText"
        class="input-field"
        :placeholder="placeholder"
        :disabled="disabled"
        :rows="1"
        @keydown="handleKeydown"
        @input="autoResize"
      />
      <button
        class="send-button"
        :class="{ active: inputText.trim() && !disabled }"
        :disabled="!inputText.trim() || disabled"
        @click="send"
      >
        <svg v-if="!disabled" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M2 21l21-9L2 3v7l15 2-15 2v7z"/>
        </svg>
        <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <line x1="9" y1="9" x2="15" y2="15"/>
          <line x1="15" y1="9" x2="9" y2="15"/>
        </svg>
      </button>
    </div>
    <div class="input-hint">
      Enter 发送 · Shift+Enter 换行
      <span v-if="disabled">· AI 回复中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'

const props = defineProps<{
  placeholder?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'send', text: string): void
}>()

const inputText = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    send()
  }
}

function send() {
  const text = inputText.value.trim()
  if (!text || props.disabled) return
  emit('send', text)
  inputText.value = ''
  nextTick(() => {
    autoResize()
    textareaRef.value?.focus()
  })
}

function autoResize() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 120) + 'px'
}

function focus() {
  textareaRef.value?.focus()
}

defineExpose({ focus })
</script>

<style scoped>
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

.send-button:not(:disabled):active {
  transform: scale(0.92);
}

.input-hint {
  padding: 4px 16px 0;
  font-size: var(--app-font-11);
  color: var(--color-text-tertiary);
}
</style>
