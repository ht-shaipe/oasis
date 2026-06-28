<template>
    <div class="section-panel">
        <div class="llm-header">
            <h2 class="section-heading">{{ t('settings.embedding.title') }}</h2>
        </div>

        <div class="embed-desc">
            {{ t('settings.embedding.description') }}
        </div>

        <div class="mode-switch-row">
            <span class="mode-label">{{ t('settings.embedding.mode') }}</span>
            <el-radio-group v-model="embedMode" size="small" @change="handleModeChange">
                <el-radio-button value="local">{{ t('settings.embedding.localMode') }}</el-radio-button>
                <el-radio-button value="remote">{{ t('settings.embedding.remoteMode') }}</el-radio-button>
            </el-radio-group>
            <el-switch
                v-model="autoActivate"
                size="small"
                :active-text="t('settings.embedding.autoActivate')"
                @change="handleAutoActivateChange"
                style="margin-left: auto"
            />
        </div>

        <div class="hf-search-section">
            <div class="hf-search-bar">
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

            <div v-if="hfSearchResults.length > 0" class="hf-results">
                <div class="hf-results-header">
                    <span>{{ t('settings.embedding.hfResultsCount', { count: hfSearchResults.length }) }}</span>
                    <el-button size="small" link type="primary" @click="hfSearchResults = []">
                        {{ t('common.close') }}
                    </el-button>
                </div>
                <el-scrollbar class="hf-results-scroll" max-height="300px">
                    <div
                        v-for="m in hfSearchResults"
                        :key="m.modelId"
                        class="hf-result-item"
                    >
                        <div class="hf-result-info">
                            <div class="hf-result-name">
                                <span>{{ m.modelId }}</span>
                                <span v-if="m.pipelineTag" class="hf-tag">{{ m.pipelineTag }}</span>
                                <span v-if="m.libraryName" class="hf-tag lib">{{ m.libraryName }}</span>
                            </div>
                            <div class="hf-result-meta">
                                <span v-if="m.downloads">{{ formatNumber(m.downloads) }} downloads</span>
                                <span v-if="m.likes">{{ formatNumber(m.likes) }} likes</span>
                                <span v-if="m.tags.includes('onnx')" class="onnx-badge">ONNX</span>
                            </div>
                        </div>
                        <div class="hf-result-action">
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
                                <span class="downloaded-label">{{ t('settings.embedding.hfDownloaded') }}</span>
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
                                <span class="already-label">{{ t('settings.embedding.hfAlreadyAdded') }}</span>
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

        <div v-if="embedMode === 'local' && models.length > 0" class="model-grid">
            <div
                v-for="model in models"
                :key="model.id"
                class="model-card"
                :class="{
                    active: activeModelId === model.id,
                    downloading: !model.downloaded && downloadingId === model.id,
                }"
            >
                <div class="card-header">
                    <span class="model-name">{{ model.name }}</span>
                    <span v-if="model.quantized" class="quant-tag">Q</span>
                    <span v-if="model.isCustom" class="custom-tag">Custom</span>
                    <span v-if="activeModelId === model.id && model.downloaded" class="running-tag">
                        <span class="running-dot"></span>
                        {{ t('settings.embedding.running') }}
                    </span>
                </div>
                <div class="card-meta">
                    <span class="meta-item">{{ model.dimensions }}D</span>
                    <span class="meta-item">~{{ model.sizeMb }}MB</span>
                    <span class="meta-item">{{ model.license }}</span>
                </div>
                <div class="card-desc">{{ model.description }}</div>

                <el-progress
                    v-if="downloadProgress[model.id] && downloadProgress[model.id].status === 'downloading'"
                    :percentage="downloadProgress[model.id].percentage"
                    :stroke-width="4"
                    :show-text="true"
                    :format="(p: number) => `${p}%`"
                    style="margin: 4px 0"
                />

                <div v-if="downloadProgress[model.id]?.status === 'cancelled'" class="cancelled-hint">
                    {{ t('settings.embedding.downloadCancelled') }}
                </div>

                <div class="card-actions">
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
        await Promise.all([loadModels(), loadConfig()])
    } catch {
        // cancelled
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
    await Promise.all([loadModels(), loadConfig()])
})

onUnmounted(() => {
    unlistenProgress?.()
})
</script>

<style scoped>
.embed-desc {
    font-size: var(--app-font-13);
    color: var(--color-text-secondary);
    margin-bottom: 16px;
    line-height: 1.5;
}

.mode-switch-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    padding: 8px 12px;
    background: var(--color-card-bg);
    border-radius: 8px;
    border: 1px solid var(--color-card-border);
}

.mode-label {
    font-size: 13px;
    color: var(--color-text-secondary);
    flex-shrink: 0;
}

.hf-search-section {
    margin-bottom: 16px;
}

.hf-search-bar {
    margin-bottom: 8px;
}

.hf-results {
    border: 1px solid var(--color-card-border);
    border-radius: 8px;
    background: var(--color-card-bg);
}

.hf-results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--color-text-secondary);
    border-bottom: 1px solid var(--color-card-border);
}

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

.hf-result-info {
    min-width: 0;
    flex: 1;
}

.hf-result-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
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

.hf-result-meta {
    display: flex;
    gap: 8px;
    font-size: 11px;
    color: var(--color-text-tertiary);
    margin-top: 2px;
}

.onnx-badge {
    color: #42b883;
    font-weight: 600;
}

.hf-result-action {
    flex-shrink: 0;
}

.already-label {
    font-size: 12px;
    color: var(--color-text-tertiary);
}

.downloaded-label {
    font-size: 12px;
    font-weight: 500;
    color: #42b883;
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

.model-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
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

.model-card.downloading {
    border-color: #e6a23c;
    background: rgba(230, 162, 60, 0.04);
}

.card-header {
    display: flex;
    align-items: center;
    gap: 8px;
}

.model-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
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

.card-meta {
    display: flex;
    gap: 10px;
    font-size: 12px;
    color: var(--color-text-tertiary);
}

.meta-item {
    white-space: nowrap;
}

.card-desc {
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.4;
}

.cancelled-hint {
    font-size: 12px;
    color: #e6a23c;
}

.card-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
}

.active-label {
    font-size: 13px;
    font-weight: 500;
    color: #42b883;
}

.running-tag {
    font-size: 11px;
    font-weight: 600;
    color: #42b883;
    background: rgba(66, 184, 131, 0.1);
    padding: 2px 8px;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
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

.llm-empty {
    padding: 48px 0;
    text-align: center;
    font-size: 14px;
    color: var(--color-text-tertiary);
}

.llm-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
}

.llm-header .section-heading {
    margin-bottom: 0;
}
</style>
