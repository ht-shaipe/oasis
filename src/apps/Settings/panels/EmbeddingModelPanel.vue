<template>
    <div class="section-panel">
        <div class="flex items-center justify-between mb-3">
            <h2 class="section-heading mb-0">{{ t('settings.embedding.title') }}</h2>
        </div>

        <div class="text-[13px] text-[var(--color-text-secondary)] mb-4 leading-7">
            {{ t('settings.embedding.description') }}
        </div>

        <div class="flex items-center gap-3 mb-4 px-3 py-2 bg-[var(--color-card-bg)] rounded-2 border border-solid border-[var(--color-card-border)]">
            <span class="text-[13px] text-[var(--color-text-secondary)] shrink-0">{{ t('settings.embedding.mode') }}</span>
            <el-radio-group v-model="embedMode" size="small" @change="handleModeChange">
                <el-radio-button value="local">{{ t('settings.embedding.localMode') }}</el-radio-button>
                <el-radio-button value="remote">{{ t('settings.embedding.remoteMode') }}</el-radio-button>
            </el-radio-group>
            <el-switch
                v-model="autoActivate"
                size="small"
                :active-text="t('settings.embedding.autoActivate')"
                @change="handleAutoActivateChange"
                class="ml-auto"
            />
        </div>

        <div class="mb-4">
            <div class="mb-2">
                <el-input
                    v-model="hfSearchQuery"
                    :placeholder="t('settings.embedding.hfSearchPlaceholder')"
                    size="small"
                    clearable
                    @keyup.enter="handleHfSearch"
                >
                    <template #prefix>
                        <el-icon><Search /></el-icon>
                    </template>
                    <template #append>
                        <el-button :loading="hfSearching" @click="handleHfSearch">
                            {{ t('settings.embedding.hfSearch') }}
                        </el-button>
                    </template>
                </el-input>
            </div>

            <div v-if="hfSearchResults.length > 0" class="border border-solid border-[var(--color-card-border)] rounded-2 bg-[var(--color-card-bg)]">
                <div class="flex items-center justify-between px-3 py-2 text-[13px] text-[var(--color-text-secondary)] border-b border-solid border-[var(--color-card-border)]">
                    <span>{{ t('settings.embedding.hfResultsCount', { count: hfSearchResults.length }) }}</span>
                    <el-button size="small" link type="primary" @click="hfSearchResults = []">
                        {{ t('common.close') }}
                    </el-button>
                </div>
                <el-scrollbar class="max-h-300px">
                    <div
                        v-for="m in hfSearchResults"
                        :key="m.modelId"
                        class="hf-result-item"
                    >
                        <div class="min-w-0 flex-1">
                            <div class="text-[13px] text-[var(--color-text-primary)] font-500 flex items-center gap-1.5 flex-wrap">
                                <span>{{ m.modelId }}</span>
                                <span v-if="m.pipelineTag" class="hf-tag">{{ m.pipelineTag }}</span>
                                <span v-if="m.libraryName" class="hf-tag lib">{{ m.libraryName }}</span>
                            </div>
                            <div class="flex gap-2 text-[11px] text-[var(--color-text-tertiary)] mt-0.5">
                                <span v-if="m.downloads">{{ formatNumber(m.downloads) }} downloads</span>
                                <span v-if="m.likes">{{ formatNumber(m.likes) }} likes</span>
                                <span v-if="m.tags.includes('onnx')" class="text-primary font-600">ONNX</span>
                            </div>
                        </div>
                        <div class="shrink-0">
                            <template v-if="!isModelKnown(m.modelId)">
                                <el-button
                                    type="primary"
                                    size="small"
                                    :loading="addingFromHf === m.modelId"
                                    @click="handleAddFromHf(m.modelId)"
                                >
                                    {{ t('settings.embedding.hfAddAndDownload') }}
                                </el-button>
                            </template>
                            <template v-else-if="isModelDownloaded(m.modelId)">
                                <span class="text-[12px] font-500 text-primary">{{ t('settings.embedding.hfDownloaded') }}</span>
                                <el-button
                                    type="danger"
                                    size="small"
                                    link
                                    @click="isModelCustom(m.modelId) ? handleRemoveCustom(m.modelId) : handleDelete(m.modelId)"
                                >
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                            <template v-else>
                                <span class="text-[12px] text-[var(--color-text-tertiary)]">{{ t('settings.embedding.hfAlreadyAdded') }}</span>
                                <el-button
                                    v-if="isModelCustom(m.modelId)"
                                    type="danger"
                                    size="small"
                                    link
                                    @click="handleRemoveCustom(m.modelId)"
                                >
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                        </div>
                    </div>
                </el-scrollbar>
            </div>
        </div>

        <div v-if="embedMode === 'local' && models.length > 0" class="section-divider">
            <span>{{ t('settings.embedding.myModels') }}</span>
        </div>

        <div v-if="embedMode === 'local' && models.length > 0" class="grid grid-cols-1 gap-3">
            <div
                v-for="model in models"
                :key="model.id"
                class="model-card"
                :class="{
                    active: activeModelId === model.id && model.downloaded,
                    downloaded: model.downloaded && activeModelId !== model.id,
                    pending: !model.downloaded,
                    downloading: !model.downloaded && downloadingId === model.id,
                }"
            >
                <div class="flex items-center gap-2">
                    <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                    <span v-if="model.quantized" class="quant-tag">Q</span>
                    <span v-if="model.isCustom" class="custom-tag">Custom</span>
                    <span v-if="activeModelId === model.id && model.downloaded" class="status-tag status-active">
                        <span class="running-dot"></span>
                        {{ t('settings.embedding.running') }}
                    </span>
                    <span v-else-if="model.downloaded" class="status-tag status-downloaded">
                        {{ t('settings.embedding.downloaded') }}
                    </span>
                    <span v-else class="status-tag status-pending">
                        {{ t('settings.embedding.pendingDownload') }}
                    </span>
                </div>
                <div class="flex gap-2.5 text-[12px] text-[var(--color-text-tertiary)]">
                    <span class="whitespace-nowrap">{{ model.dimensions }}D</span>
                    <span class="whitespace-nowrap">~{{ model.sizeMb }}MB</span>
                    <span class="whitespace-nowrap">{{ model.license }}</span>
                </div>
                <div class="text-[12px] text-[var(--color-text-secondary)] leading-5">{{ model.description }}</div>

                <el-progress
                    v-if="downloadProgress[model.id] && downloadProgress[model.id].status === 'downloading'"
                    :percentage="downloadProgress[model.id].percentage"
                    :stroke-width="4"
                    :show-text="true"
                    :format="(p: number) => `${p}%`"
                    class="my-1"
                />

                <div v-if="downloadProgress[model.id]?.status === 'cancelled'" class="text-[12px] text-[#e6a23c]">
                    {{ t('settings.embedding.downloadCancelled') }}
                </div>

                <div class="flex items-center gap-2 mt-0.5">
                    <template v-if="!model.downloaded">
                        <el-button
                            type="primary"
                            size="small"
                            :loading="downloadingId === model.id"
                            @click="handleDownload(model.id)"
                        >
                            {{ t('settings.embedding.download') }}
                        </el-button>
                        <el-button
                            v-if="downloadingId === model.id"
                            size="small"
                            type="warning"
                            plain
                            @click="handleCancelDownload"
                        >
                            {{ t('settings.embedding.cancel') }}
                        </el-button>
                        <el-button
                            type="danger"
                            size="small"
                            link
                            @click="model.isCustom ? handleRemoveCustom(model.id) : handleHideModel(model.id)"
                        >
                            <el-icon><Delete /></el-icon>
                        </el-button>
                    </template>
                    <template v-else>
                        <el-button
                            v-if="activeModelId !== model.id"
                            type="success"
                            size="small"
                            plain
                            @click="handleSetActive(model.id)"
                        >
                            {{ t('settings.embedding.activate') }}
                        </el-button>
                        <el-button
                            v-if="model.isCustom"
                            type="danger"
                            size="small"
                            link
                            @click="handleRemoveCustom(model.id)"
                        >
                            <el-icon><Delete /></el-icon>
                        </el-button>
                        <el-button
                            v-if="!model.isCustom"
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

        <div v-if="embedMode === 'local' && hiddenModels.length > 0" class="section-divider">
            <span>{{ t('settings.embedding.hiddenModels') }}</span>
        </div>

        <div v-if="embedMode === 'local' && hiddenModels.length > 0" class="text-[12px] text-[var(--color-text-tertiary)] mb-3">
            {{ t('settings.embedding.hiddenModelsHint') }}
        </div>

        <div v-if="embedMode === 'local' && hiddenModels.length > 0" class="grid grid-cols-2 gap-3">
            <div
                v-for="model in hiddenModels"
                :key="model.id"
                class="model-card hidden"
            >
                <div class="flex items-center gap-2">
                    <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                    <span class="hidden-tag">{{ t('settings.embedding.hidden') }}</span>
                </div>
                <div class="flex gap-2.5 text-[12px] text-[var(--color-text-tertiary)]">
                    <span class="whitespace-nowrap">{{ model.dimensions }}D</span>
                    <span class="whitespace-nowrap">~{{ model.sizeMb }}MB</span>
                    <span class="whitespace-nowrap">{{ model.license }}</span>
                </div>
                <div class="flex items-center gap-2">
                    <el-button
                        type="primary"
                        size="small"
                        plain
                        @click="handleRestoreModel(model.id)"
                    >
                        {{ t('settings.embedding.restore') }}
                    </el-button>
                </div>
            </div>
        </div>

        <el-dialog v-model="showAddCustom" :title="t('settings.embedding.addCustom')" width="480px">
            <el-form :model="customForm" label-width="100px" size="small">
                <el-form-item :label="t('settings.embedding.customModelId')">
                    <el-input v-model="customForm.modelId" placeholder="e.g. Qwen/Qwen-embedding" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customName')">
                    <el-input v-model="customForm.name" placeholder="Display name" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customDimensions')">
                    <el-input-number v-model="customForm.dimensions" :min="1" :max="8192" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customOnnxFile')">
                    <el-input v-model="customForm.onnxFile" placeholder="e.g. model.onnx" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customQuantized')">
                    <el-switch v-model="customForm.quantized" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customSizeMb')">
                    <el-input-number v-model="customForm.sizeMb" :min="1" :max="10000" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customLicense')">
                    <el-input v-model="customForm.license" placeholder="MIT / Apache-2.0 / ..." />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customDescription')">
                    <el-input v-model="customForm.description" type="textarea" :rows="2" />
                </el-form-item>
                <el-form-item :label="t('settings.embedding.customAdditionalFiles')">
                    <el-input v-model="customForm.additionalFilesStr" placeholder="tokenizer.json, config.json (comma-separated)" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button size="small" @click="showAddCustom = false">{{ t('common.cancel') }}</el-button>
                <el-button size="small" type="primary" :loading="addingCustom" @click="handleAddCustom">
                    {{ t('settings.embedding.addCustom') }}
                </el-button>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Delete, Search } from '@element-plus/icons-vue'

const { t } = useI18n()

interface LocalEmbeddingModel {
    id: string
    name: string
    dimensions: number
    quantized: boolean
    sizeMb: number
    license: string
    description: string
    downloaded: boolean
    isCustom: boolean
    onnxFile: string
    additionalFiles: string[]
}

interface EmbedConfig {
    mode: 'Local' | 'Remote'
    activeLocalModelId: string | null
    activeRemoteModelId: string | null
    customModels: CustomModelEntry[]
    autoActivate: boolean
    hiddenBuiltinIds: string[]
}

interface CustomModelEntry {
    modelId: string
    name: string
    dimensions: number
    quantized: boolean
    sizeMb: number
    license: string
    description: string
    onnxFile: string
    additionalFiles: string[]
}

interface DownloadProgressPayload {
    modelId: string
    fileName: string
    current: number
    total: number
    percentage: number
    status: string
}

interface HfModelSearchResult {
    modelId: string
    author: string
    downloads: number
    likes: number
    pipelineTag: string | null
    tags: string[]
    libraryName: string | null
    createdAt: string | null
}

interface HfModelFileInfo {
    rfilename: string
}

interface HfModelInfo {
    modelId: string
    siblings: HfModelFileInfo[]
    tags: string[]
    pipelineTag: string | null
    libraryName: string | null
    downloads: number | null
    likes: number | null
}

const models = ref<LocalEmbeddingModel[]>([])
const embedMode = ref<'local' | 'remote'>('local')
const activeModelId = ref<string | null>(null)
const autoActivate = ref(true)
const downloadingId = ref<string | null>(null)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})
const showAddCustom = ref(false)
const addingCustom = ref(false)
const customForm = reactive({
    modelId: '',
    name: '',
    dimensions: 768,
    onnxFile: 'model.onnx',
    quantized: false,
    sizeMb: 100,
    license: 'MIT',
    description: '',
    additionalFilesStr: 'tokenizer.json,config.json,tokenizer_config.json',
})

const hfSearchQuery = ref('')
const hfSearching = ref(false)
const hfSearchResults = ref<HfModelSearchResult[]>([])
const addingFromHf = ref<string | null>(null)
const hiddenModels = ref<LocalEmbeddingModel[]>([])

const downloadedModels = computed(() => models.value.filter((m) => m.downloaded))

const knownModelIds = computed(() => new Set(models.value.map((m) => m.id)))
const downloadedModelIds = computed(() => new Set(downloadedModels.value.map((m) => m.id)))
const customModelIds = computed(() => new Set(models.value.filter((m) => m.isCustom).map((m) => m.id)))

function isModelKnown(modelId: string): boolean {
    return knownModelIds.value.has(modelId)
}

function isModelDownloaded(modelId: string): boolean {
    return downloadedModelIds.value.has(modelId)
}

function isModelCustom(modelId: string): boolean {
    return customModelIds.value.has(modelId)
}

let unlistenProgress: UnlistenFn | null = null

async function loadModels() {
    try {
        models.value = await invoke<LocalEmbeddingModel[]>('list_available_embedding_models')
    } catch (e) {
        console.error('Failed to load embedding models:', e)
    }
}

async function loadConfig() {
    try {
        const config = await invoke<EmbedConfig>('get_embedding_config')
        embedMode.value = config.mode.toLowerCase() as 'local' | 'remote'
        activeModelId.value = config.activeLocalModelId
        autoActivate.value = config.autoActivate
    } catch (e) {
        console.error('Failed to load embed config:', e)
    }
}

async function loadHiddenModels() {
    try {
        hiddenModels.value = await invoke<LocalEmbeddingModel[]>('list_hidden_embedding_models')
    } catch (e) {
        console.error('Failed to load hidden models:', e)
    }
}

async function handleModeChange() {
    try {
        await invoke('set_embed_mode', { mode: embedMode.value })
        await loadConfig()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleAutoActivateChange() {
    try {
        await invoke('set_auto_activate', { autoActivate: autoActivate.value })
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleDownload(modelId: string) {
    downloadingId.value = modelId
    try {
        await invoke('download_embedding_model', { modelId, autoActivate: autoActivate.value || undefined })
        ElMessage.success(t('settings.embedding.downloadSuccess'))
        await Promise.all([loadModels(), loadConfig()])
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        downloadingId.value = null
        delete downloadProgress[modelId]
    }
}

async function handleCancelDownload() {
    try {
        await invoke('cancel_embedding_download')
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleSetActive(modelId: string) {
    try {
        await invoke('set_active_embedding_model', { modelId })
        ElMessage.success(t('settings.embedding.activateSuccess'))
        await loadConfig()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleDelete(modelId: string) {
    try {
        await ElMessageBox.confirm(
            t('settings.embedding.deleteConfirm'),
            t('settings.embedding.deleteModel'),
            { type: 'warning' }
        )
        await invoke('delete_embedding_model', { modelId })
        ElMessage.success(t('settings.embedding.deleteSuccess'))
        await Promise.all([loadModels(), loadConfig()])
    } catch {
        // cancelled
    }
}

async function handleRemoveCustom(modelId: string) {
    try {
        await ElMessageBox.confirm(
            t('settings.embedding.removeCustomConfirm'),
            t('settings.embedding.removeCustom'),
            { type: 'warning' }
        )
        await invoke('remove_custom_embedding_model', { modelId })
        ElMessage.success(t('settings.embedding.removeCustomSuccess'))
        await Promise.all([loadModels(), loadConfig(), loadHiddenModels()])
    } catch {
        // cancelled
    }
}

async function handleHideModel(modelId: string) {
    try {
        await invoke('hide_embedding_model', { modelId })
        ElMessage.success(t('settings.embedding.hideSuccess'))
        await Promise.all([loadModels(), loadConfig(), loadHiddenModels()])
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleRestoreModel(modelId: string) {
    try {
        await invoke('restore_embedding_model', { modelId })
        ElMessage.success(t('settings.embedding.restoreSuccess'))
        await Promise.all([loadModels(), loadConfig(), loadHiddenModels()])
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleHfSearch() {
    hfSearching.value = true
    try {
        const query = hfSearchQuery.value.trim() || undefined
        hfSearchResults.value = await invoke<HfModelSearchResult[]>('search_hf_embedding_models', {
            query,
            limit: 30,
        })
        if (hfSearchResults.value.length === 0) {
            ElMessage.info(t('settings.embedding.hfNoResults'))
        }
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        hfSearching.value = false
    }
}

function findOnnxFile(siblings: HfModelFileInfo[]): string {
    const onnxFiles = siblings
        .map((f) => f.rfilename)
        .filter((n) => n.endsWith('.onnx'))

    const quantized = onnxFiles.find((n) =>
        n.includes('quantized') || n.includes('INT8') || n.includes('int8') || n.includes('Q8')
    )
    if (quantized) return quantized

    const modelOnnx = onnxFiles.find((n) => n === 'model.onnx' || n.endsWith('/model.onnx'))
    if (modelOnnx) return modelOnnx

    return onnxFiles[0] || 'model.onnx'
}

function findAdditionalFiles(siblings: HfModelFileInfo[]): string[] {
    const names = siblings.map((f) => f.rfilename)
    const candidates = [
        'tokenizer.json', 'config.json', 'special_tokens_map.json',
        'tokenizer_config.json', 'tokenizer.model', 'sentence_bert_config.json',
        'modules.json', 'config_sentence_transformers.json',
    ]
    const found: string[] = []
    for (const c of candidates) {
        if (names.includes(c)) {
            found.push(c)
        } else {
            const match = names.find((n) => n.endsWith('/' + c))
            if (match) found.push(match)
        }
    }
    return found
}

async function handleAddFromHf(modelId: string) {
    addingFromHf.value = modelId
    try {
        const info = await invoke<HfModelInfo>('get_hf_model_info', { modelId })

        const onnxFile = findOnnxFile(info.siblings)
        const additionalFiles = findAdditionalFiles(info.siblings)
        const hasOnnx = info.siblings.some((f) => f.rfilename.endsWith('.onnx'))
        const isQuantized = onnxFile.includes('quantized') || onnxFile.includes('INT8') || onnxFile.includes('int8')
        const license = info.tags.find((tag) => tag.startsWith('license:'))?.replace('license:', '') || 'unknown'

        const displayName = modelId.split('/').pop() || modelId

        await invoke('add_custom_embedding_model', {
            model: {
                modelId,
                name: displayName,
                dimensions: 768,
                quantized: isQuantized,
                sizeMb: 50,
                license,
                description: hasOnnx
                    ? `${info.pipelineTag || 'embedding'} model from HuggingFace`
                    : `⚠️ No ONNX file detected — may not work with local inference`,
                onnxFile,
                additionalFiles,
            },
        })

        ElMessage.success(t('settings.embedding.hfAdded'))

        await loadModels()

        if (hasOnnx) {
            await handleDownload(modelId)
        }
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        addingFromHf.value = null
    }
}

async function handleAddCustom() {
    if (!customForm.modelId.trim()) {
        ElMessage.warning(t('settings.embedding.customModelIdRequired'))
        return
    }
    addingCustom.value = true
    try {
        const additionalFiles = customForm.additionalFilesStr
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean)

        await invoke('add_custom_embedding_model', {
            model: {
                modelId: customForm.modelId.trim(),
                name: customForm.name.trim() || customForm.modelId.trim(),
                dimensions: customForm.dimensions,
                quantized: customForm.quantized,
                sizeMb: customForm.sizeMb,
                license: customForm.license.trim(),
                description: customForm.description.trim(),
                onnxFile: customForm.onnxFile.trim(),
                additionalFiles,
            },
        })
        ElMessage.success(t('settings.embedding.addCustomSuccess'))
        showAddCustom.value = false
        resetCustomForm()
        await loadModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        addingCustom.value = false
    }
}

function resetCustomForm() {
    customForm.modelId = ''
    customForm.name = ''
    customForm.dimensions = 768
    customForm.onnxFile = 'model.onnx'
    customForm.quantized = false
    customForm.sizeMb = 100
    customForm.license = 'MIT'
    customForm.description = ''
    customForm.additionalFilesStr = 'tokenizer.json,config.json,tokenizer_config.json'
}

function formatNumber(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K'
    return String(n)
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

onMounted(async () => {
    unlistenProgress = await listen<DownloadProgressPayload>('embed-download-progress', handleProgressEvent)
    await Promise.all([loadModels(), loadConfig(), loadHiddenModels()])
})

onUnmounted(() => {
    unlistenProgress?.()
})
</script>

<style scoped>
.hf-result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-card-border);
    transition: background 0.1s;
}

.hf-result-item:last-child {
    border-bottom: none;
}

.hf-result-item:hover {
    background: rgba(0, 0, 0, 0.02);
}

.hf-tag {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(64, 158, 255, 0.08);
    color: #409eff;
    font-weight: 500;
}

.hf-tag.lib {
    background: rgba(103, 194, 58, 0.08);
    color: #67c23a;
}

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

.model-card.hidden {
    opacity: 0.55;
}

.quant-tag {
    font-size: 11px;
    font-weight: 600;
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.12);
    padding: 1px 6px;
    border-radius: 3px;
}

.custom-tag {
    font-size: 11px;
    font-weight: 600;
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
    padding: 1px 6px;
    border-radius: 3px;
}

.hidden-tag {
    font-size: 11px;
    font-weight: 600;
    color: #909399;
    background: rgba(144, 147, 153, 0.1);
    padding: 1px 6px;
    border-radius: 3px;
    margin-left: auto;
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
