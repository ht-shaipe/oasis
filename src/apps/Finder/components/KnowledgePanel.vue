<template>
  <div class="knowledge-panel">
    <div class="knowledge-header">
      <h4>{{ t('knowledge.indexStatus') }}</h4>
    </div>

    <div v-if="status" class="knowledge-stats">
      <div class="stat-row">
        <span class="stat-label">{{ t('knowledge.totalFiles') }}</span>
        <span class="stat-value">{{ status.totalFiles }}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">{{ t('knowledge.totalChunks') }}</span>
        <span class="stat-value">{{ status.totalChunks }}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">{{ t('knowledge.embeddedChunks') }}</span>
        <span class="stat-value">{{ status.embeddedChunks }}</span>
      </div>
      <div v-if="status.embeddingModel" class="stat-row">
        <span class="stat-label">{{ t('knowledge.embeddingModel') }}</span>
        <span class="stat-value">{{ status.embeddingModel }}</span>
      </div>
      <div v-if="status.lastIndexTime" class="stat-row">
        <span class="stat-label">{{ t('knowledge.lastIndexTime') }}</span>
        <span class="stat-value">{{ formatTime(status.lastIndexTime) }}</span>
      </div>
    </div>

    <div v-if="status?.isIndexing" class="indexing-progress">
      <el-icon class="progress-icon"><Loading /></el-icon>
      <span>{{ t('knowledge.indexingProgress') }}</span>
    </div>

    <div class="knowledge-actions">
      <el-select
        v-model="selectedModelId"
        :placeholder="t('knowledge.selectEmbeddingModel')"
        size="small"
        style="width: 100%"
        :disabled="models.length === 0"
      >
        <el-option
          v-for="m in models"
          :key="m.id"
          :label="m.name"
          :value="m.model_id"
        >
          <span>{{ m.name }}</span>
          <span class="model-id-tag">{{ m.model_id }}</span>
        </el-option>
      </el-select>

      <div v-if="models.length === 0" class="no-embedding-hint">
        {{ t('knowledge.noEmbeddingModel') }}
      </div>

      <el-button
        type="primary"
        size="small"
        :loading="isIndexing"
        :disabled="!selectedModelId"
        @click="handleStartIndexing"
        style="width: 100%"
      >
        {{ isIndexing ? t('knowledge.indexing') : t('knowledge.startIndexing') }}
      </el-button>

      <el-button
        v-if="status?.isIndexing"
        size="small"
        @click="handleStopIndexing"
        style="width: 100%"
      >
        {{ t('knowledge.stopIndexing') }}
      </el-button>
    </div>

    <div class="knowledge-search">
      <h4>{{ t('knowledge.semanticSearch') }}</h4>
      <el-input
        v-model="searchQuery"
        :placeholder="t('knowledge.searchPlaceholder')"
        size="small"
        clearable
        @keyup.enter="handleSearch"
      >
        <template #append>
          <el-button :loading="isSearching" @click="handleSearch">
            <el-icon><Search /></el-icon>
          </el-button>
        </template>
      </el-input>
    </div>

    <div v-if="searchResults.length > 0" class="search-results">
      <div
        v-for="result in searchResults"
        :key="result.chunkIndex + result.relPath"
        class="search-result-item"
      >
        <div class="result-header">
          <span class="result-file">{{ result.relPath }}</span>
          <span class="result-score">{{ (result.score * 100).toFixed(1) }}%</span>
        </div>
        <div class="result-content">{{ result.chunkContent }}</div>
      </div>
    </div>

    <div v-if="status && status.totalFiles > 0" class="knowledge-actions">
      <el-button size="small" type="danger" plain @click="handleClearIndex" style="width: 100%">
        {{ t('knowledge.clearIndex') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Loading, Search } from '@element-plus/icons-vue'

const { t } = useI18n()

interface KnowledgeStatus {
  workspaceDir: string
  totalFiles: number
  totalChunks: number
  embeddedChunks: number
  lastIndexTime: string | null
  isIndexing: boolean
  embeddingModel: string | null
  embeddingDim: number | null
}

interface LLMModel {
  id: string
  name: string
  provider: string
  model_id: string
  base_url: string
  api_key: string
  auth_type: string
  enabled: boolean
  model_type: string
}

interface SearchResult {
  filePath: string
  relPath: string
  chunkContent: string
  chunkIndex: number
  score: number
}

const status = ref<KnowledgeStatus | null>(null)
const models = ref<LLMModel[]>([])
const selectedModelId = ref('')
const isIndexing = ref(false)
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const isSearching = ref(false)

async function loadStatus() {
  try {
    status.value = await invoke<KnowledgeStatus>('get_knowledge_status')
    isIndexing.value = status.value.isIndexing
  } catch (e) {
    console.error('Failed to load knowledge status:', e)
  }
}

async function loadModels() {
  try {
    const all = await invoke<LLMModel[]>('get_llm_models')
    models.value = all.filter((m) => m.enabled && m.model_type === 'embedding')
    if (!selectedModelId.value && models.value.length > 0) {
      selectedModelId.value = models.value[0].model_id
    }
  } catch (e) {
    console.error('Failed to load models:', e)
  }
}

async function handleStartIndexing() {
  if (!selectedModelId.value) {
    ElMessage.warning(t('knowledge.selectModelFirst'))
    return
  }
  isIndexing.value = true
  try {
    const result = await invoke<{
      indexedFiles: number
      skippedFiles: number
      deletedFiles: number
      totalChunks: number
      elapsedSecs: number
    }>('start_indexing', { embeddingModelId: selectedModelId.value })
    ElMessage.success(
      t('knowledge.indexComplete', {
        files: result.indexedFiles,
        chunks: result.totalChunks,
        secs: result.elapsedSecs.toFixed(1),
      })
    )
    await loadStatus()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  } finally {
    isIndexing.value = false
  }
}

async function handleStopIndexing() {
  try {
    await invoke('stop_indexing')
    isIndexing.value = false
    await loadStatus()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  }
}

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  isSearching.value = true
  try {
    searchResults.value = await invoke<SearchResult[]>('semantic_search', {
      query: searchQuery.value,
      topK: 5,
    })
    if (searchResults.value.length === 0) {
      ElMessage.info(t('knowledge.noResults'))
    }
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  } finally {
    isSearching.value = false
  }
}

async function handleClearIndex() {
  try {
    await ElMessageBox.confirm(
      t('knowledge.clearConfirm'),
      t('knowledge.clearIndex'),
      { type: 'warning' }
    )
    await invoke('delete_knowledge_index')
    searchResults.value = []
    await loadStatus()
    ElMessage.success(t('knowledge.clearComplete'))
  } catch {
    // cancelled
  }
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleString()
  } catch {
    return iso
  }
}

onMounted(async () => {
  await Promise.all([loadStatus(), loadModels()])
})

defineExpose({ loadStatus })
</script>

<style scoped>
.knowledge-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.knowledge-header h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.knowledge-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
}

.stat-label {
  color: var(--color-text-secondary);
}

.stat-value {
  color: var(--color-text-primary);
  font-weight: 500;
}

.indexing-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(64, 158, 255, 0.08);
  border-radius: 6px;
  font-size: 13px;
  color: #409eff;
}

.progress-icon {
  animation: spin 2s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.knowledge-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.no-embedding-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding: 6px 8px;
  background: rgba(230, 162, 60, 0.06);
  border-radius: 4px;
  line-height: 1.5;
}

.model-id-tag {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-left: 8px;
}

.knowledge-search h4 {
  margin: 0 0 8px 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.search-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.search-result-item {
  padding: 8px;
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  cursor: default;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.result-file {
  font-size: 12px;
  font-weight: 500;
  color: #007bff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.result-score {
  font-size: 11px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.result-content {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  max-height: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
</style>
