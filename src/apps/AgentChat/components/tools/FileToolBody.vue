<template>
  <div class="file-body">
    <div v-if="filePath" class="file-path-row">
      <span class="file-icon">📄</span>
      <code class="file-path-text">{{ filePath }}</code>
    </div>
    <div v-if="content" class="file-content-preview">
      <div class="content-label">{{ operationLabel }}</div>
      <pre class="content-text">{{ contentPreview }}</pre>
    </div>
    <div v-if="startLine || endLine" class="file-meta">
      <span v-if="startLine">L{{ startLine }}</span>
      <span v-if="startLine && endLine">-</span>
      <span v-if="endLine">L{{ endLine }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ input: unknown }>()

const filePath = computed(() => {
  if (typeof props.input === 'string') return props.input
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.path || obj.file_path || obj.filename || obj.fileName || '') as string
  }
  return ''
})

const content = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.content || obj.new_content || obj.text || '') as string
  }
  return ''
})

const operationLabel = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    if (obj.old_content || obj.old_string) return 'Diff'
  }
  return content.value ? 'Content' : ''
})

const contentPreview = computed(() => {
  if (!content.value) return ''
  if (content.value.length > 500) {
    return content.value.slice(0, 500) + '\n... (truncated)'
  }
  return content.value
})

const startLine = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.start_line || obj.line_number || obj.startLine || '') as string
  }
  return ''
})

const endLine = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.end_line || obj.endLine || '') as string
  }
  return ''
})
</script>

<style scoped>
.file-body {
  font-family: ui-monospace, monospace;
  font-size: 13px;
}

.file-path-row {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
}

.file-icon {
  font-size: 13px;
  flex-shrink: 0;
}

.file-path-text {
  color: #007AFF;
  word-break: break-all;
}

.content-label {
  font-size: 13px;
  color: var(--color-text-tertiary);
  margin-bottom: 2px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.content-text {
  margin: 0;
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow-y: auto;
}

.file-meta {
  margin-top: 4px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}
</style>
