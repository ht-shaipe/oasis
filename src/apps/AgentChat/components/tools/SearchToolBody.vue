<template>
  <div class="search-body">
    <div v-if="pattern" class="search-pattern-row">
      <span class="search-icon">🔍</span>
      <code class="search-pattern">{{ pattern }}</code>
    </div>
    <div v-if="searchPath" class="search-meta">
      <span class="meta-label">in:</span> {{ searchPath }}
    </div>
    <div v-if="fileType" class="search-meta">
      <span class="meta-label">type:</span> {{ fileType }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ input: unknown }>()

const pattern = computed(() => {
  if (typeof props.input === 'string') return props.input
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.pattern || obj.query || obj.search || obj.glob || obj.regex || '') as string
  }
  return ''
})

const searchPath = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.path || obj.directory || obj.cwd || '') as string
  }
  return ''
})

const fileType = computed(() => {
  if (props.input && typeof props.input === 'object') {
    const obj = props.input as Record<string, unknown>
    return (obj.file_type || obj.include || obj.type || '') as string
  }
  return ''
})
</script>

<style scoped>
.search-body {
  font-family: ui-monospace, monospace;
  font-size: 13px;
}

.search-pattern-row {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
}

.search-icon {
  font-size: 13px;
  flex-shrink: 0;
}

.search-pattern {
  color: #FF9500;
  word-break: break-word;
}

.search-meta {
  margin-top: 4px;
  color: var(--color-text-tertiary);
  font-size: 13px;
  padding-left: 2px;
}

.meta-label {
  margin-right: 4px;
}
</style>
