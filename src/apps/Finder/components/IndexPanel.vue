<template>
  <div class="index-panel">
    <div class="index-header">
      <h4>{{ t('knowledge.indexStatus') }}</h4>
    </div>

    <div v-if="status" class="index-stats">
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

    <div v-if="status?.isIndexing || isIndexing" class="indexing-progress">
      <el-icon class="progress-icon"><Loading /></el-icon>
      <div class="progress-info">
        <span class="progress-phase">{{ phaseLabel }}</span>
        <el-progress
          v-if="indexProgress.total > 0"
          :percentage="Math.round((indexProgress.current / indexProgress.total) * 100)"
          :stroke-width="4"
          :show-text="true"
          :format="() => `${indexProgress.current}/${indexProgress.total}`"
          size="small"
        />
        <span v-if="indexProgress.message" class="progress-message">{{ indexProgress.message }}</span>
      </div>
    </div>

    <div class="index-actions">
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
        <div v-if="activeLocalModel" class="active-model-card">
          <div class="active-model-header">
            <span class="active-model-name">{{ activeLocalModel.name }}</span>
            <span v-if="activeLocalModel.quantized" class="quant-badge">Q</span>
            <span class="active-model-status">
              <span class="status-dot"></span>
              {{ t('knowledge.currentModel') }}
            </span>
          </div>
          <div class="active-model-meta">
            <span>{{ activeLocalModel.dimensions }}D</span>
            <span>~{{ activeLocalModel.sizeMb }}MB</span>
            <span>{{ activeLocalModel.license }}</span>
          </div>
        </div>
        <div v-else class="no-active-model-hint">
          <span>{{ t('knowledge.noLocalModel') }}</span>
          <el-button size="small" type="primary" link @click="openEmbeddingSettings">
            {{ t('knowledge.goToSettings') }}
          </el-button>
        </div>

        <div v-if="inactiveModels.length > 0" class="inactive-models-section">
          <div class="inactive-models-toggle" @click="showInactiveModels = !showInactiveModels">
            <el-icon :class="{ rotated: showInactiveModels }"><ArrowRight /></el-icon>
            <span>{{ t('knowledge.otherModels', { count: inactiveModels.length }) }}</span>
          </div>
          <el-scrollbar v-if="showInactiveModels" max-height="160px">
            <div class="inactive-model-list">
              <div
                v-for="m in inactiveModels"
                :key="m.id"
                class="inactive-model-row"
              >
                <div class="inactive-model-info">
                  <span class="inactive-model-name">{{ m.name }}</span>
                  <span v-if="m.quantized" class="quant-badge">Q</span>
                  <span class="inactive-model-meta">{{ m.dimensions }}D · ~{{ m.sizeMb }}MB</span>
                </div>
                <div class="inactive-model-action">
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
                    v-else
                    type="success"
                    size="small"
                    plain
                    @click="handleActivateLocal(m.id)"
                  >
                    {{ t('knowledge.activateModel') }}
                  </el-button>
                </div>
              </div>
            </div>
          </el-scrollbar>
        </div>
      </template>

      <div class="incremental-toggle">
        <el-switch v-model="incrementalMode" size="small" />
        <span class="incremental-label">{{ t('knowledge.incrementalMode') }}</span>
      </div>

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

    <div v-if="status && status.totalFiles > 0" class="index-actions">
      <el-button size="small" type="danger" plain @click="handleClearIndex" style="width: 100%">
        {{ t('knowledge.clearIndex') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Loading, ArrowRight } from '@element-plus/icons-vue'
import { useKnowledge } from '../composables/useKnowledge'

const { t } = useI18n()

const {
  status,
  remoteModels,
  embedMode,
  selectedModelId,
  localModelId,
  isIndexing,
  incrementalMode,
  downloadingId,
  indexProgress,
  phaseLabel,
  activeLocalModel,
  inactiveModels,
  loadStatus,
  loadLocalModels,
  ensureInit,
  cleanup,
} = useKnowledge()

const showInactiveModels = ref(false)

function openEmbeddingSettings() {
  const event = new CustomEvent('open-app', { detail: 'settings' })
  window.dispatchEvent(event)
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
  indexProgress.value = { phase: 'scanning', current: 0, total: 0, message: '' }
  try {
    const result = await invoke<{
      indexedFiles: number
      skippedFiles: number
      skippedUnchanged: number
      deletedFiles: number
      totalChunks: number
      elapsedSecs: number
    }>('start_indexing', { params: { mode, modelId, incremental: incrementalMode.value } })
    if (incrementalMode.value && result.skippedUnchanged > 0) {
      ElMessage.success(
        t('knowledge.indexCompleteIncremental', {
          files: result.indexedFiles,
          unchanged: result.skippedUnchanged,
          chunks: result.totalChunks,
          secs: result.elapsedSecs.toFixed(1),
        })
      )
    } else {
      ElMessage.success(
        t('knowledge.indexComplete', {
          files: result.indexedFiles,
          chunks: result.totalChunks,
          secs: result.elapsedSecs.toFixed(1),
        })
      )
    }
    await loadStatus()
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  } finally {
    isIndexing.value = false
    indexProgress.value = { phase: '', current: 0, total: 0, message: '' }
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

async function handleClearIndex() {
  try {
    await ElMessageBox.confirm(
      t('knowledge.clearConfirm'),
      t('knowledge.clearIndex'),
      { type: 'warning' }
    )
    await invoke('delete_knowledge_index')
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

onMounted(() => ensureInit())
onUnmounted(() => cleanup())
</script>

<style scoped>
.index-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.index-header h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.index-stats {
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
  align-items: flex-start;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(64, 158, 255, 0.08);
  border-radius: 6px;
  font-size: 13px;
  color: #409eff;
}

.progress-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.progress-phase {
  font-weight: 500;
}

.progress-message {
  font-size: 12px;
  color: var(--color-text-tertiary, #909399);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.progress-icon {
  animation: spin 2s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.index-actions {
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

.incremental-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
}

.incremental-label {
  font-size: 13px;
  color: var(--color-text-secondary);
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

.active-model-card {
  padding: 10px 12px;
  border: 1.5px solid #42b883;
  border-radius: 8px;
  background: rgba(66, 184, 131, 0.04);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.active-model-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.active-model-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.active-model-status {
  font-size: 12px;
  font-weight: 500;
  color: #42b883;
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #42b883;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.active-model-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.quant-badge {
  font-size: 10px;
  font-weight: 600;
  color: #e6a23c;
  background: rgba(230, 162, 60, 0.1);
  padding: 0 4px;
  border-radius: 2px;
}

.no-active-model-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding: 8px 10px;
  background: rgba(230, 162, 60, 0.06);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.inactive-models-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.inactive-models-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  cursor: pointer;
  padding: 2px 0;
  user-select: none;
}

.inactive-models-toggle:hover {
  color: var(--color-text-secondary);
}

.inactive-models-toggle .el-icon {
  transition: transform 0.2s;
  font-size: 12px;
}

.inactive-models-toggle .el-icon.rotated {
  transform: rotate(90deg);
}

.inactive-model-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.inactive-model-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  background: var(--color-card-bg);
}

.inactive-model-info {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.inactive-model-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.inactive-model-meta {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.inactive-model-action {
  flex-shrink: 0;
}
</style>
