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
      <div class="mode-switch">
        <span class="mode-label">{{ t('knowledge.embeddingMode') }}</span>
        <el-radio-group v-model="embedMode" size="small" @change="handleModeChange">
          <el-radio-button value="local">{{ t('knowledge.localMode') }}</el-radio-button>
          <el-radio-button value="remote">{{ t('knowledge.remoteMode') }}</el-radio-button>
        </el-radio-group>
      </div>

      <template v-if="embedMode === 'remote'">
        <el-select
          v-model="selectedModelId"
          :placeholder="t('knowledge.selectEmbeddingModel')"
          size="small"
          style="width: 100%"
          :disabled="remoteModels.length === 0"
        >
          <el-option
            v-for="m in remoteModels"
            :key="m.id"
            :label="m.name"
            :value="m.model_id"
          >
            <span>{{ m.name }}</span>
            <span class="model-id-tag">{{ m.model_id }}</span>
          </el-option>
        </el-select>

        <div v-if="remoteModels.length === 0" class="no-embedding-hint">
          {{ t('knowledge.noEmbeddingModel') }}
        </div>
      </template>

      <template v-else>
        <div class="local-model-section">
          <div
            v-for="m in localModels"
            :key="m.id"
            class="local-model-row"
            :class="{ active: localModelId === m.id }"
          >
            <div class="local-model-info">
              <div class="local-model-name">
                <span>{{ m.name }}</span>
                <span v-if="m.quantized" class="model-id-tag quant-tag">Q</span>
              </div>
              <div class="local-model-meta">
                <span>{{ m.dimensions }}D</span>
                <span>~{{ m.sizeMb }}MB</span>
              </div>
              <el-progress
                v-if="downloadProgress[m.id] && downloadProgress[m.id].status === 'downloading'"
                :percentage="downloadProgress[m.id].percentage"
                :stroke-width="3"
                :show-text="true"
                :format="(p: number) => `${p}%`"
                style="margin-top: 2px"
              />
            </div>
            <div class="local-model-action">
              <el-button
                v-if="!m.downloaded"
                type="primary"
                size="small"
                :loading="downloadingId === m.id"
                @click="handleDownloadLocal(m.id)"
              >
                {{ t('knowledge.downloadModel') }}
              </el-button>
              <el-button
                v-else-if="localModelId !== m.id"
                type="success"
                size="small"
                plain
                @click="handleActivateLocal(m.id)"
              >
                {{ t('knowledge.activateModel') }}
              </el-button>
              <span v-else class="active-label">{{ t('knowledge.currentModel') }}</span>
            </div>
          </div>

          <div v-if="localModels.length === 0" class="no-embedding-hint">
            {{ t('knowledge.noLocalModel') }}
          </div>
        </div>
      </template>

      <el-button
        type="primary"
        size="small"
        :loading="isIndexing"
        :disabled="embedMode === 'remote' ? !selectedModelId : !localModelId"
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
      <el-scrollbar max-height="300px">
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
      </el-scrollbar>
    </div>

    <div v-if="status && status.totalFiles > 0" class="knowledge-actions">
      <el-button size="small" type="danger" plain @click="handleClearIndex" style="width: 100%">
        {{ t('knowledge.clearIndex') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
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
  embeddingMode: string | null
  localModelId: string | null
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

interface LocalEmbeddingModel {
  id: string
  name: string
  dimensions: number
  quantized: boolean
  sizeMb: number
  downloaded: boolean
  isCustom: boolean
}

interface DownloadProgressPayload {
  modelId: string
  fileName: string
  current: number
  total: number
  percentage: number
  status: string
}

interface SearchResult {
  filePath: string
  relPath: string
  chunkContent: string
  chunkIndex: number
  score: number
}

const status = ref<KnowledgeStatus | null>(null)
const remoteModels = ref<LLMModel[]>([])
const localModels = ref<LocalEmbeddingModel[]>([])
const embedMode = ref<'local' | 'remote'>('local')
const selectedModelId = ref('')
const localModelId = ref('')
const isIndexing = ref(false)
const downloadingId = ref<string | null>(null)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const isSearching = ref(false)

async function loadStatus() {
  try {
    status.value = await invoke<KnowledgeStatus>('get_knowledge_status')
    isIndexing.value = status.value.isIndexing
    if (status.value.embeddingMode) {
      embedMode.value = status.value.embeddingMode as 'local' | 'remote'
    }
    if (status.value.localModelId && embedMode.value === 'local') {
      localModelId.value = status.value.localModelId
    }
  } catch (e) {
    console.error('Failed to load knowledge status:', e)
  }
}

async function loadRemoteModels() {
  try {
    const all = await invoke<LLMModel[]>('get_llm_models')
    remoteModels.value = all.filter((m) => m.enabled && m.model_type === 'embedding')
    if (!selectedModelId.value && remoteModels.value.length > 0) {
      selectedModelId.value = remoteModels.value[0].model_id
    }
  } catch (e) {
    console.error('Failed to load remote models:', e)
  }
}

async function loadLocalModels() {
  try {
    const models = await invoke<LocalEmbeddingModel[]>('list_available_embedding_models')
    localModels.value = models
    const downloaded = models.filter((m) => m.downloaded)
    if (!localModelId.value && downloaded.length > 0) {
      localModelId.value = downloaded[0].id
    }
  } catch (e) {
    console.error('Failed to load local models:', e)
  }
}

async function handleModeChange() {
  try {
    await invoke('set_embed_mode', { mode: embedMode.value })
    await loadStatus()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  }
}

async function handleDownloadLocal(modelId: string) {
  downloadingId.value = modelId
  try {
    await invoke('download_embedding_model', { modelId })
    ElMessage.success(t('knowledge.downloadModelSuccess'))
    await loadLocalModels()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  } finally {
    downloadingId.value = null
  }
}

async function handleActivateLocal(modelId: string) {
  try {
    await invoke('set_active_embedding_model', { modelId })
    localModelId.value = modelId
    ElMessage.success(t('knowledge.activateModelSuccess'))
    await loadStatus()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  }
}

async function handleStartIndexing() {
  const mode = embedMode.value
  const modelId = mode === 'local' ? localModelId.value : selectedModelId.value

  if (!modelId) {
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
    }>('start_indexing', { params: { mode, modelId } })
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

function handleProgressEvent(event: { payload: DownloadProgressPayload }) {
  const p = event.payload
  downloadProgress[p.modelId] = p
  if (p.status === 'complete' || p.status === 'cancelled') {
    setTimeout(() => {
      delete downloadProgress[p.modelId]
    }, 3000)
  }
}

let unlistenProgress: UnlistenFn | null = null

onMounted(async () => {
  unlistenProgress = await listen<DownloadProgressPayload>('embed-download-progress', handleProgressEvent)
  await Promise.all([loadStatus(), loadRemoteModels(), loadLocalModels()])
})

onUnmounted(() => {
  unlistenProgress?.()
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

.mode-switch {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mode-label {
  font-size: 13px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
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

.model-id-tag.not-downloaded {
  color: #f56c6c;
}

.model-id-tag.quant-tag {
  color: #e6a23c;
  background: rgba(230, 162, 60, 0.1);
  padding: 0 4px;
  border-radius: 2px;
  font-weight: 600;
}

.local-model-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.local-model-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  background: var(--color-card-bg);
  transition: border-color 0.15s;
}

.local-model-row:hover {
  border-color: var(--color-input-border, #dcdfe6);
}

.local-model-row.active {
  border-color: #42b883;
  background: rgba(66, 184, 131, 0.04);
}

.local-model-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.local-model-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.local-model-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.local-model-action {
  flex-shrink: 0;
}

.active-label {
  font-size: 13px;
  font-weight: 500;
  color: #42b883;
  white-space: nowrap;
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
