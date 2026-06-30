<template>
  <div class="shell-body">
    <div v-if="command" class="shell-command">
      <span class="shell-prompt">$</span>
      <code class="shell-cmd-text">{{ command }}</code>
    </div>
    <div v-if="workdir" class="shell-meta">
      <span class="meta-label">cwd:</span> {{ workdir }}
    </div>
    <div v-if="timeout" class="shell-meta">
      <span class="meta-label">timeout:</span> {{ timeout }}s
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ input: unknown }>()

const command = computed(() => {
  if (typeof props.input === 'string') return props.input
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.command || obj.cmd || obj.script || '') as string
  }
  return ''
})

const workdir = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.workdir || obj.cwd || obj.working_directory || '') as string
  }
  return ''
})

const timeout = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    const t = obj.timeout || obj.max_duration
    return t ? String(t) : ''
  }
  return ''
})
</script>

<style scoped>
.shell-body {
  font-family: ui-monospace, monospace;
  font-size: 13px;
}

.shell-command {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
}

.shell-prompt {
  color: #007AFF;
  user-select: none;
  flex-shrink: 0;
}

.shell-cmd-text {
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
}

.shell-meta {
  font-size: 13px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
  padding-left: 2px;
}

.meta-label {
  color: var(--color-text-tertiary);
  margin-right: 4px;
}
</style>
