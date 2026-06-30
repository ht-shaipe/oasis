<template>
    <div class="section-panel">
        <div class="flex items-center justify-between mb-3">
            <h2 class="section-heading mb-0">{{ t('settings.localModel.title') }}</h2>
        </div>

        <div class="text-[13px] text-[var(--color-text-secondary)] mb-4 leading-7">
            {{ t('settings.localModel.description') }}
        </div>

        <div v-if="models.length > 0" class="section-divider">
            <span>{{ t('settings.localModel.available') }}</span>
        </div>

        <div v-if="models.length > 0" class="grid grid-cols-1 gap-3">
            <div
                v-for="model in sortedModels"
                :key="model.id"
                class="model-card"
                :class="{
                    active: model.isActive,
                    loaded: model.isLoaded && !model.isActive,
                    downloaded: model.downloaded && !model.isActive && !model.isLoaded,
                    pending: !model.downloaded,
                    downloading: !model.downloaded && downloadingId === model.id,
                }"
            >
                <div class="flex items-center gap-2">
                    <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                    <span class="quant-tag">{{ model.paramsBillions }}B</span>
                    <span v-if="model.isActive" class="status-tag status-active">
                        <span class="running-dot"></span>
                        {{ t('settings.localModel.running') }}
                    </span>
                    <span v-else-if="model.isLoaded" class="status-tag status-loaded">
                        {{ t('settings.localModel.loaded') }}
                    </span>
                    <span v-else-if="model.downloaded" class="status-tag status-downloaded">
                        {{ t('settings.localModel.downloaded') }}
                    </span>
                    <span v-else class="status-tag status-pending">
                        {{ t('settings.localModel.pendingDownload') }}
                    </span>
                </div>
                <div class="flex gap-2.5 text-[12px] text-[var(--color-text-tertiary)]">
                    <span class="whitespace-nowrap">~{{ model.sizeMb >= 1000 ? (model.sizeMb / 1000).toFixed(1) + 'GB' : model.sizeMb + 'MB' }}</span>
                    <span class="whitespace-nowrap">Q4_K_M</span>
                    <span class="whitespace-nowrap">{{ model.license }}</span>
                </div>
                <div class="text-[12px] text-[var(--color-text-secondary)] leading-5">{{ model.description }}</div>

                <div v-if="loadingModel" class="text-[12px] text-[#e6a23c] flex items-center gap-1">
                    <span class="running-dot" style="background:#e6a23c"></span>
                    {{ t('settings.localModel.loadingModel') }}
                </div>

                <el-progress
                    v-if="downloadProgress[model.id] && downloadProgress[model.id].status === 'downloading'"
                    :percentage="downloadProgress[model.id].percentage"
                    :stroke-width="4"
                    :show-text="true"
                    :format="(p: number) => `${p}%`"
                    class="my-1"
                />

                <div v-if="downloadProgress[model.id]?.status === 'cancelled'" class="text-[12px] text-[#e6a23c]">
                    {{ t('settings.localModel.downloadCancelled') }}
                </div>

                <div class="flex items-center gap-2 mt-0.5">
                    <template v-if="!model.downloaded">
                        <el-button
                            type="primary"
                            size="small"
                            :loading="downloadingId === model.id"
                            @click="handleDownload(model.id)"
                        >
                            {{ t('settings.localModel.download') }}
                        </el-button>
                        <el-button
                            v-if="downloadingId === model.id"
                            size="small"
                            type="warning"
                            plain
                            @click="handleCancelDownload"
                        >
                            {{ t('settings.localModel.cancel') }}
                        </el-button>
                    </template>
                    <template v-else>
                        <el-button
                            v-if="!model.isActive"
                            type="success"
                            size="small"
                            plain
                            :loading="activatingId === model.id"
                            @click="handleActivate(model.id)"
                        >
                            {{ t('settings.localModel.activate') }}
                        </el-button>
                        <el-button
                            v-if="model.isActive"
                            type="warning"
                            size="small"
                            plain
                            @click="handleUnload"
                        >
                            {{ t('settings.localModel.unload') }}
                        </el-button>
                        <el-button
                            type="danger"
                            size="small"
                            link
                            @click="handleDelete(model.id)"
                        >
                            <el-icon><Delete /></el-icon>
                        </el-button>
                    </template>
                </div>
            </div>
        </div>

        <div v-if="models.length === 0" class="text-[13px] text-[var(--color-text-tertiary)]">
            {{ t('settings.localModel.noModels') }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'

const { t } = useI18n()

interface LocalChatModel {
    id: string
    name: string
    paramsBillions: number
    sizeMb: number
    license: string
    description: string
    downloaded: boolean
    isActive: boolean
    isLoaded: boolean
}

interface DownloadProgressPayload {
    modelId: string
    fileName: string
    current: number
    total: number
    percentage: number
    status: string
}

interface ModelLoadState {
    modelId: string
    status: string
    message: string | null
}

const models = ref<LocalChatModel[]>([])
const downloadingId = ref<string | null>(null)
const activatingId = ref<string | null>(null)
const loadingModel = ref(false)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})

const sortedModels = computed(() => {
    return [...models.value].sort((a, b) => {
        if (a.isActive && !b.isActive) return -1
        if (!a.isActive && b.isActive) return 1
        if (a.isLoaded && !b.isLoaded) return -1
        if (!a.isLoaded && b.isLoaded) return 1
        if (a.downloaded && !b.downloaded) return -1
        if (!a.downloaded && b.downloaded) return 1
        return a.paramsBillions - b.paramsBillions
    })
})

let unlistenProgress: UnlistenFn | null = null
let unlistenLoad: UnlistenFn | null = null

async function loadModels() {
    try {
        models.value = await invoke<LocalChatModel[]>('list_local_chat_models')
    } catch (e) {
        console.error('Failed to load local chat models:', e)
    }
}

async function handleDownload(modelId: string) {
    downloadingId.value = modelId
    try {
        await invoke('download_local_chat_model', { modelId, autoActivate: true })
        ElMessage.success(t('settings.localModel.downloadSuccess'))
        await loadModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        downloadingId.value = null
        delete downloadProgress[modelId]
    }
}

async function handleCancelDownload() {
    try {
        await invoke('cancel_local_chat_download')
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleActivate(modelId: string) {
    activatingId.value = modelId
    loadingModel.value = true
    try {
        await invoke('set_active_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.activateSuccess'))
        await loadModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        activatingId.value = null
        loadingModel.value = false
    }
}

async function handleUnload() {
    try {
        await invoke('unload_local_chat_model')
        ElMessage.success(t('settings.localModel.unloadSuccess'))
        await loadModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleDelete(modelId: string) {
    try {
        await ElMessageBox.confirm(
            t('settings.localModel.deleteConfirm'),
            t('settings.localModel.deleteModel'),
            { type: 'warning' }
        )
        await invoke('delete_local_chat_model', { modelId })
        ElMessage.success(t('settings.localModel.deleteSuccess'))
        await loadModels()
    } catch {
        // cancelled
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

function handleLoadEvent(event: { payload: ModelLoadState }) {
    const p = event.payload
    loadingModel.value = p.status === 'loading'
    if (p.status === 'ready' || p.status === 'error') {
        loadModels()
    }
}

onMounted(async () => {
    unlistenProgress = await listen<DownloadProgressPayload>('local-llm-download-progress', handleProgressEvent)
    unlistenLoad = await listen<ModelLoadState>('local-llm-load-progress', handleLoadEvent)
    await loadModels()
})

onUnmounted(() => {
    unlistenProgress?.()
    unlistenLoad?.()
})
</script>

<style scoped>
.section-divider {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    color: var(--color-text-tertiary);
    font-size: 12px;
}

.section-divider::before,
.section-divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--color-card-border);
}

.model-card {
    padding: 14px 16px;
    border-radius: 10px;
    background: var(--color-card-bg);
    border: 1.5px solid var(--color-card-border);
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.model-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.model-card.active {
    border-color: #42b883;
    background: rgba(66, 184, 131, 0.04);
    box-shadow: 0 0 0 1px rgba(66, 184, 131, 0.15);
}

.model-card.loaded {
    border-color: rgba(64, 158, 255, 0.35);
    background: rgba(64, 158, 255, 0.02);
}

.model-card.downloaded {
    border-color: rgba(66, 184, 131, 0.35);
    background: rgba(66, 184, 131, 0.02);
}

.model-card.pending {
    border-color: rgba(0, 0, 0, 0.06);
    background: rgba(0, 0, 0, 0.01);
}

.model-card.downloading {
    border-color: #e6a23c;
    background: rgba(230, 162, 60, 0.04);
}

.quant-tag {
    font-size: 11px;
    font-weight: 600;
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.12);
    padding: 1px 6px;
    border-radius: 3px;
}

.status-tag {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
}

.status-tag.status-active {
    color: #42b883;
    background: rgba(66, 184, 131, 0.12);
}

.status-tag.status-loaded {
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
}

.status-tag.status-downloaded {
    color: #67c23a;
    background: rgba(103, 194, 58, 0.08);
}

.status-tag.status-pending {
    color: #909399;
    background: rgba(144, 147, 153, 0.08);
}

.running-dot {
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
</style>
