<template>
  <div class="tool-call-card" :class="{ 'is-expanded': expanded }">
    <div class="tool-header" @click="expanded = !expanded">
      <span class="tool-icon">{{ toolIcon }}</span>
      <span class="tool-name">{{ tool.name }}</span>
      <span v-if="tool.output !== undefined && !expanded" class="tool-status">
        {{ tool.isError ? '✗' : '✓' }}
      </span>
      <span v-else-if="tool.output === undefined && !expanded" class="tool-status running">⟳</span>
      <span class="tool-chevron">{{ expanded ? '▾' : '▸' }}</span>
    </div>

    <div v-if="expanded" class="tool-body">
      <div class="tool-section">
        <div class="tool-section-title">输入</div>
        <ShellToolBody v-if="toolCategory === 'shell'" :input="tool.input" />
        <FileToolBody v-else-if="toolCategory === 'file'" :input="tool.input" />
        <SearchToolBody v-else-if="toolCategory === 'search'" :input="tool.input" />
        <pre v-else class="tool-code">{{ formatInput(tool.input) }}</pre>
      </div>

      <div v-if="tool.output !== undefined" class="tool-section">
        <div class="tool-section-title" :class="{ 'is-error': tool.isError }">
          {{ tool.isError ? '错误' : '输出' }}
        </div>
        <ShellToolOutput v-if="toolCategory === 'shell'" :output="tool.output" :is-error="!!tool.isError" />
        <pre v-else class="tool-code" :class="{ 'is-error': tool.isError }">{{ formatOutput(tool.output) }}</pre>
      </div>

      <div v-else class="tool-section">
        <div class="tool-section-title running">执行中...</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ToolCall } from '@/store/agent'
import ShellToolBody from './tools/ShellToolBody.vue'
import FileToolBody from './tools/FileToolBody.vue'
import SearchToolBody from './tools/SearchToolBody.vue'
import ShellToolOutput from './tools/ShellToolOutput.vue'

const props = defineProps<{
  tool: ToolCall
}>()

const expanded = ref(false)

const toolCategory = computed(() => {
  const name = props.tool.name
  if (name === 'Bash' || name === 'bash' || name === 'shell' || name === 'execute_command' || name === 'run_command') return 'shell'
  if (name === 'Read' || name === 'Write' || name === 'Edit' || name === 'edit_file' || name === 'create_file' || name === 'read_file' || name === 'write_file' || name === 'list_directory' || name === 'ListDir') return 'file'
  if (name === 'Grep' || name === 'Glob' || name === 'search_files' || name === 'find_in_files' || name === 'list_files') return 'search'
  return 'other'
})

const toolIcon = computed(() => {
  switch (toolCategory.value) {
    case 'shell': return '⌨️'
    case 'file': return '📄'
    case 'search': return '🔍'
    default: return '🔧'
  }
})

function formatInput(input: unknown): string {
  if (typeof input === 'string') return input
  try {
    return JSON.stringify(input, null, 2)
  } catch {
    return String(input)
  }
}

function formatOutput(output: unknown): string {
  if (typeof output === 'string') {
    if (output.length > 2000) {
      return output.slice(0, 2000) + '\n\n... (truncated)'
    }
    return output
  }
  try {
    return JSON.stringify(output, null, 2)
  } catch {
    return String(output)
  }
}
</script>

<style scoped>
.tool-call-card {
  margin: 8px 0;
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--color-card-bg, rgba(255,255,255,0.05));
}

.tool-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
}

.tool-header:hover {
  background: var(--color-hover-bg);
}

.tool-icon {
  font-size: 14px;
}

.tool-name {
  font-size: 13px;
  font-family: ui-monospace, monospace;
  color: var(--color-text-primary);
  flex: 1;
}

.tool-status {
  font-size: 13px;
  color: var(--el-color-success);
}

.tool-status.running {
  color: #007AFF;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.tool-chevron {
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.tool-body {
  border-top: 1px solid var(--color-card-border);
  padding: 8px 12px 12px;
}

.tool-section {
  margin-bottom: 8px;
}

.tool-section:last-child {
  margin-bottom: 0;
}

.tool-section-title {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-tertiary);
  margin-bottom: 4px;
}

.tool-section-title.is-error {
  color: var(--el-color-danger);
}

.tool-section-title.running {
  color: #007AFF;
}

.tool-code {
  margin: 0;
  font-size: 13px;
  font-family: ui-monospace, monospace;
  color: var(--color-text-secondary);
  background: var(--color-code-bg, #1e1e2e);
  border-radius: 6px;
  padding: 8px 10px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 300px;
  overflow-y: auto;
}

.tool-code.is-error {
  color: var(--el-color-danger);
}
</style>
