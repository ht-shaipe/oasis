<template>
  <div class="shell-output">
    <pre class="output-text" :class="{ 'is-error': isError }">{{ outputText }}</pre>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  output: unknown
  isError: boolean
}>()

const outputText = computed(() => {
  if (typeof props.output === 'string') {
    return props.output.length > 2000
      ? props.output.slice(0, 2000) + '\n\n... (truncated)'
      : props.output
  }
  try {
    return JSON.stringify(props.output, null, 2)
  } catch {
    return String(props.output)
  }
})
</script>

<style scoped>
.shell-output {
  font-family: ui-monospace, monospace;
  font-size: 13px;
}

.output-text {
  margin: 0;
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 300px;
  overflow-y: auto;
  color: var(--color-text-secondary);
}

.output-text.is-error {
  color: var(--el-color-danger);
}
</style>
