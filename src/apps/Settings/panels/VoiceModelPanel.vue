<template>
    <div class="section-panel">
        <div class="flex items-center justify-between mb-3">
            <h2 class="section-heading mb-0">{{ t('settings.voice.title') }}</h2>
        </div>

        <div class="text-[13px] text-[var(--color-text-secondary)] mb-4 leading-7">
            {{ t('settings.voice.description') }}
        </div>

        <el-tabs v-model="activeTab" class="voice-tabs">
            <el-tab-pane :label="t('settings.voice.tabRemote')" name="remote">
                <div class="remote-config">
                    <el-form :model="remoteConfig" label-width="100px" size="small">
                        <el-form-item :label="t('settings.voice.provider')">
                            <el-select v-model="remoteConfig.provider" :placeholder="t('settings.voice.selectProvider')" style="width: 100%" @change="onProviderChange">
                                <el-option v-for="p in providers" :key="p.code" :value="p.code" :label="p.name" />
                            </el-select>
                        </el-form-item>
                        <el-form-item :label="t('llm.baseUrl')">
                            <el-input v-model="remoteConfig.base_url" :placeholder="t('llm.baseUrlPlaceholder')" />
                        </el-form-item>
                        <el-form-item :label="t('llm.apiKey')">
                            <el-input v-model="remoteConfig.api_key" :placeholder="t('llm.apiKeyPlaceholder')" type="password" show-password />
                        </el-form-item>

                        <div class="section-divider">
                            <span>{{ t('settings.voice.ttsSection') }}</span>
                        </div>

                        <el-form-item :label="t('settings.voice.ttsModel')">
                            <el-select v-model="remoteConfig.tts_model_id" :placeholder="t('settings.voice.selectTtsModel')" filterable allow-create style="width: 100%">
                                <el-option v-for="m in ttsProviderModels" :key="m.id" :value="m.id" :label="m.name" />
                            </el-select>
                        </el-form-item>
                        <el-form-item :label="t('settings.voice.voice')">
                            <el-select v-model="remoteConfig.voice" :placeholder="t('settings.voice.selectVoice')" filterable allow-create style="width: 100%">
                                <el-option v-for="v in ttsVoices" :key="v" :value="v" :label="v" />
                            </el-select>
                        </el-form-item>
                        <el-form-item :label="t('settings.voice.speed')">
                            <el-slider v-model="remoteConfig.speed" :min="0.25" :max="4.0" :step="0.25" show-input />
                        </el-form-item>

                        <div class="section-divider">
                            <span>{{ t('settings.voice.sttSection') }}</span>
                        </div>

                        <el-form-item :label="t('settings.voice.sttModel')">
                            <el-select v-model="remoteConfig.stt_model_id" :placeholder="t('settings.voice.selectSttModel')" filterable allow-create style="width: 100%">
                                <el-option v-for="m in sttProviderModels" :key="m.id" :value="m.id" :label="m.name" />
                            </el-select>
                        </el-form-item>
                        <el-form-item :label="t('settings.voice.language')">
                            <el-select v-model="remoteConfig.language" :placeholder="t('settings.voice.selectLanguage')" filterable allow-create style="width: 100%">
                                <el-option value="zh" label="中文" />
                                <el-option value="en" label="English" />
                                <el-option value="ja" label="日本語" />
                                <el-option value="ko" label="한국어" />
                                <el-option value="auto" :label="t('settings.voice.autoDetect')" />
                            </el-select>
                        </el-form-item>

                        <el-form-item>
                            <el-button type="primary" size="small" :loading="saving" @click="saveRemoteConfig">
                                {{ t('app.save') }}
                            </el-button>
                            <el-button size="small" :loading="testing" @click="testTts">
                                {{ t('settings.voice.test') }}
                            </el-button>
                        </el-form-item>
                    </el-form>
                </div>
            </el-tab-pane>

            <el-tab-pane :label="t('settings.voice.tabLocal')" name="local">
                <div class="mb-3">
                    <el-input
                        v-model="hfSearchQuery"
                        :placeholder="t('settings.voice.hfSearchPlaceholder')"
                        size="small"
                        clearable
                        @keyup.enter="handleHfSearch"
                    >
                        <template #prefix>
                            <el-icon><Search /></el-icon>
                        </template>
                        <template #append>
                            <el-button :loading="hfSearching" @click="handleHfSearch">
                                {{ t('settings.voice.hfSearch') }}
                            </el-button>
                        </template>
                    </el-input>
                </div>

                <div v-if="hfResults.length > 0" class="border border-solid border-[var(--color-card-border)] rounded-2 bg-[var(--color-card-bg)]">
                    <div class="flex items-center justify-between px-3 py-2 text-[13px] text-[var(--color-text-secondary)] border-b border-solid border-[var(--color-card-border)]">
                        <span>{{ t('settings.voice.hfResultsCount', { count: hfResults.length }) }}</span>
                        <el-button size="small" link type="primary" @click="hfResults = []">
                            {{ t('app.close') }}
                        </el-button>
                    </div>
                    <el-scrollbar class="max-h-300px">
                        <div v-for="m in hfResults" :key="m.modelId" class="hf-result-item">
                            <div class="min-w-0 flex-1">
                                <div class="text-[13px] text-[var(--color-text-primary)] font-500 flex items-center gap-1.5 flex-wrap">
                                    <span>{{ m.modelId }}</span>
                                    <span v-if="m.pipelineTag" class="hf-tag">{{ m.pipelineTag }}</span>
                                    <span v-if="m.libraryName" class="hf-tag lib">{{ m.libraryName }}</span>
                                </div>
                                <div class="flex gap-2 text-[11px] text-[var(--color-text-tertiary)] mt-0.5">
                                    <span v-if="m.downloads">{{ formatNumber(m.downloads) }} downloads</span>
                                    <span v-if="m.likes">{{ formatNumber(m.likes) }} likes</span>
                                </div>
                            </div>
                            <div class="shrink-0">
                                <template v-if="!isModelKnown(m.modelId)">
                                    <el-button type="primary" size="small" :loading="addingFromHf === m.modelId" @click="handleAddFromHf(m.modelId, m.pipelineTag)">
                                        {{ t('settings.voice.hfAddAndDownload') }}
                                    </el-button>
                                </template>
                                <template v-else-if="isModelDownloaded(m.modelId)">
                                    <span class="text-[12px] font-500 text-primary">{{ t('settings.voice.hfDownloaded') }}</span>
                                    <el-button type="danger" size="small" link @click="isModelCustom(m.modelId) ? handleRemoveCustom(m.modelId) : handleDeleteModel(m.modelId)">
                                        <el-icon><Delete /></el-icon>
                                    </el-button>
                                </template>
                                <template v-else>
                                    <span class="text-[12px] text-[var(--color-text-tertiary)]">{{ t('settings.voice.hfAlreadyAdded') }}</span>
                                </template>
                            </div>
                        </div>
                    </el-scrollbar>
                </div>

                <div v-if="localModels.length > 0" class="section-divider mt-4">
                    <span>{{ t('settings.voice.myModels') }}</span>
                </div>

                <div v-if="localModels.length > 0" class="grid grid-cols-1 gap-3">
                    <div
                        v-for="model in sortedModels"
                        :key="model.id"
                        class="model-card"
                        :class="{
                            active: isActiveModel(model.id) && model.downloaded,
                            downloaded: model.downloaded && !isActiveModel(model.id),
                            pending: !model.downloaded,
                            downloading: !model.downloaded && downloadingId === model.id,
                        }"
                    >
                        <div class="flex items-center gap-2">
                            <span class="text-[14px] text-[var(--color-text-primary)] font-600">{{ model.name }}</span>
                            <span class="voice-type-tag" :class="model.voiceType">{{ model.voiceType === 'tts' ? 'TTS' : 'STT' }}</span>
                            <span v-if="model.isCustom" class="custom-tag">Custom</span>
                            <span v-if="isActiveModel(model.id) && model.downloaded" class="status-tag status-active">
                                <span class="running-dot"></span>
                                {{ t('settings.voice.running') }}
                            </span>
                            <span v-else-if="model.downloaded" class="status-tag status-downloaded">
                                {{ t('settings.voice.downloaded') }}
                            </span>
                            <span v-else class="status-tag status-pending">
                                {{ t('settings.voice.pendingDownload') }}
                            </span>
                        </div>
                        <div class="flex gap-2.5 text-[12px] text-[var(--color-text-tertiary)]">
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

                        <div class="flex items-center gap-2 mt-0.5">
                            <template v-if="!model.downloaded">
                                <el-button type="primary" size="small" :loading="downloadingId === model.id" @click="handleDownload(model.id)">
                                    {{ t('settings.voice.download') }}
                                </el-button>
                                <el-button v-if="downloadingId === model.id" size="small" type="warning" plain @click="handleCancelDownload">
                                    {{ t('settings.voice.cancel') }}
                                </el-button>
                                <el-button type="danger" size="small" link @click="model.isCustom ? handleRemoveCustom(model.id) : handleDeleteModel(model.id)">
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                            <template v-else>
                                <el-button v-if="!isActiveModel(model.id)" type="success" size="small" plain @click="handleSetActive(model.id)">
                                    {{ t('settings.voice.activate') }}
                                </el-button>
                                <el-button type="danger" size="small" link @click="model.isCustom ? handleRemoveCustom(model.id) : handleDeleteModel(model.id)">
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </template>
                        </div>
                    </div>
                </div>

                <div v-if="localModels.length === 0" class="llm-empty">
                    {{ t('settings.voice.noModels') }}
                </div>
            </el-tab-pane>
        </el-tabs>

        <el-dialog v-model="showCustomDialog" :title="t('settings.voice.addCustom')" width="480px">
            <el-form :model="customForm" label-width="100px" size="small">
                <el-form-item :label="t('settings.voice.voiceType')">
                    <el-radio-group v-model="customForm.voiceType">
                        <el-radio-button value="tts">TTS</el-radio-button>
                        <el-radio-button value="stt">STT</el-radio-button>
                    </el-radio-group>
                </el-form-item>
                <el-form-item :label="t('settings.voice.customModelId')">
                    <el-input v-model="customForm.modelId" placeholder="e.g. openai/whisper-tiny" />
                </el-form-item>
                <el-form-item :label="t('settings.voice.customName')">
                    <el-input v-model="customForm.name" placeholder="Display name" />
                </el-form-item>
                <el-form-item :label="t('settings.voice.customSizeMb')">
                    <el-input-number v-model="customForm.sizeMb" :min="1" :max="10000" />
                </el-form-item>
                <el-form-item :label="t('settings.voice.customLicense')">
                    <el-input v-model="customForm.license" placeholder="MIT / Apache-2.0 / ..." />
                </el-form-item>
                <el-form-item :label="t('settings.voice.customDescription')">
                    <el-input v-model="customForm.description" type="textarea" :rows="2" />
                </el-form-item>
            </el-form>
            <template #footer>
                <el-button size="small" @click="showCustomDialog = false">{{ t('app.cancel') }}</el-button>
                <el-button size="small" type="primary" :loading="addingCustom" @click="handleAddCustom">
                    {{ t('app.confirm') }}
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

const activeTab = ref('remote')

interface VoiceLocalModel {
    id: string
    name: string
    voiceType: 'tts' | 'stt'
    sizeMb: number
    license: string
    description: string
    downloaded: boolean
    isCustom: boolean
}

interface VoiceRemoteConfig {
    provider: string
    base_url: string
    api_key: string
    tts_model_id: string
    stt_model_id: string
    voice: string
    speed: number
    language: string
}

interface ProviderOption {
    code: string
    name: string
    base_url: string
}

interface ModelOption {
    id: string
    name: string
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

const remoteConfig = reactive<VoiceRemoteConfig>({
    provider: 'openai',
    base_url: 'https://api.openai.com/v1',
    api_key: '',
    tts_model_id: 'tts-1',
    stt_model_id: 'whisper-1',
    voice: 'alloy',
    speed: 1.0,
    language: 'auto',
})

const providers = ref<ProviderOption[]>([
    { code: 'openai', name: 'OpenAI', base_url: 'https://api.openai.com/v1' },
    { code: 'azure', name: 'Azure Speech', base_url: '' },
    { code: 'volcengine', name: '火山引擎', base_url: 'https://openspeech.bytedance.com/api/v1' },
    { code: 'alibaba', name: '阿里云', base_url: 'https://nls-gateway-cn-shanghai.aliyuncs.com' },
    { code: 'custom', name: 'Custom / OpenAI Compatible', base_url: '' },
])

const ttsProviderModels = ref<ModelOption[]>([
    { id: 'tts-1', name: 'TTS-1' },
    { id: 'tts-1-hd', name: 'TTS-1 HD' },
])

const sttProviderModels = ref<ModelOption[]>([
    { id: 'whisper-1', name: 'Whisper-1' },
])

const ttsVoices = ref<string[]>(['alloy', 'echo', 'fable', 'onyx', 'nova', 'shimmer'])

const activeTtsModelId = ref<string | null>(null)
const activeSttModelId = ref<string | null>(null)
const localModels = ref<VoiceLocalModel[]>([])
const downloadingId = ref<string | null>(null)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})
const saving = ref(false)
const testing = ref(false)

const hfSearchQuery = ref('')
const hfSearching = ref(false)
const hfResults = ref<HfModelSearchResult[]>([])
const addingFromHf = ref<string | null>(null)

const showCustomDialog = ref(false)
const addingCustom = ref(false)
const customForm = reactive({
    voiceType: 'tts' as 'tts' | 'stt',
    modelId: '',
    name: '',
    sizeMb: 100,
    license: 'MIT',
    description: '',
})

const sortedModels = computed(() => {
    return [...localModels.value].sort((a, b) => {
        const aActive = isActiveModel(a.id) && a.downloaded
        const bActive = isActiveModel(b.id) && b.downloaded
        if (aActive && !bActive) return -1
        if (!aActive && bActive) return 1
        if (a.downloaded && !b.downloaded) return -1
        if (!a.downloaded && b.downloaded) return 1
        if (a.voiceType === 'tts' && b.voiceType === 'stt') return -1
        if (a.voiceType === 'stt' && b.voiceType === 'tts') return 1
        return 0
    })
})

const knownIds = computed(() => new Set(localModels.value.map(m => m.id)))
const downloadedIds = computed(() => new Set(localModels.value.filter(m => m.downloaded).map(m => m.id)))
const customIds = computed(() => new Set(localModels.value.filter(m => m.isCustom).map(m => m.id)))

function isModelKnown(id: string) { return knownIds.value.has(id) }
function isModelDownloaded(id: string) { return downloadedIds.value.has(id) }
function isModelCustom(id: string) { return customIds.value.has(id) }
function isActiveModel(id: string) {
    const model = localModels.value.find(m => m.id === id)
    if (!model) return false
    return model.voiceType === 'tts' ? activeTtsModelId.value === id : activeSttModelId.value === id
}

function onProviderChange(code: string) {
    const p = providers.value.find(p => p.code === code)
    if (p) remoteConfig.base_url = p.base_url
    if (code === 'openai') {
        ttsProviderModels.value = [{ id: 'tts-1', name: 'TTS-1' }, { id: 'tts-1-hd', name: 'TTS-1 HD' }]
        sttProviderModels.value = [{ id: 'whisper-1', name: 'Whisper-1' }]
        ttsVoices.value = ['alloy', 'echo', 'fable', 'onyx', 'nova', 'shimmer']
    } else if (code === 'volcengine') {
        ttsProviderModels.value = [{ id: 'tts', name: 'TTS' }]
        sttProviderModels.value = [{ id: 'streaming_asr', name: 'Streaming ASR' }]
        ttsVoices.value = ['zh_female_cancan', 'zh_male_zhiboshuangkuai']
    } else if (code === 'alibaba') {
        ttsProviderModels.value = [{ id: 'cosyvoice', name: 'CosyVoice' }]
        sttProviderModels.value = [{ id: 'paraformer', name: 'Paraformer' }]
        ttsVoices.value = []
    } else {
        ttsProviderModels.value = []
        sttProviderModels.value = []
        ttsVoices.value = []
    }
}

async function loadConfig() {
    try {
        const config = await invoke<{ activeTtsModelId: string | null; activeSttModelId: string | null; remote: VoiceRemoteConfig }>('get_voice_config')
        activeTtsModelId.value = config.activeTtsModelId
        activeSttModelId.value = config.activeSttModelId
        if (config.remote) {
            Object.assign(remoteConfig, config.remote)
        }
    } catch {
        // use defaults
    }
}

async function loadLocalModels() {
    try {
        localModels.value = await invoke<VoiceLocalModel[]>('list_voice_models')
    } catch {
        // empty
    }
}

async function saveRemoteConfig() {
    saving.value = true
    try {
        await invoke('save_voice_config', { config: { remote: remoteConfig } })
        ElMessage.success(t('settings.voice.saveSuccess'))
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        saving.value = false
    }
}

async function testTts() {
    testing.value = true
    try {
        await invoke('test_tts', { config: remoteConfig })
        ElMessage.success(t('settings.voice.testSuccess'))
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        testing.value = false
    }
}

async function handleHfSearch() {
    hfSearching.value = true
    try {
        hfResults.value = await invoke<HfModelSearchResult[]>('search_hf_voice_models', {
            query: hfSearchQuery.value.trim() || undefined,
            limit: 30,
        })
        if (hfResults.value.length === 0) {
            ElMessage.info(t('settings.voice.hfNoResults'))
        }
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        hfSearching.value = false
    }
}

function inferVoiceType(pipelineTag: string | null): 'tts' | 'stt' {
    if (!pipelineTag) return 'tts'
    const tag = pipelineTag.toLowerCase()
    if (tag.includes('asr') || tag.includes('speech-to-text') || tag.includes('automatic-speech-recognition')) return 'stt'
    return 'tts'
}

async function handleAddFromHf(modelId: string, pipelineTag: string | null) {
    addingFromHf.value = modelId
    const voiceType = inferVoiceType(pipelineTag)
    try {
        await invoke('add_custom_voice_model', {
            model: {
                modelId,
                voiceType,
                name: modelId.split('/').pop() || modelId,
                sizeMb: 100,
                license: 'unknown',
                description: `${voiceType.toUpperCase()} model from HuggingFace`,
            },
        })
        ElMessage.success(t('settings.voice.hfAdded'))
        await loadLocalModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        addingFromHf.value = null
    }
}

async function handleDownload(modelId: string) {
    downloadingId.value = modelId
    try {
        await invoke('download_voice_model', { modelId })
        ElMessage.success(t('settings.voice.downloadSuccess'))
        await Promise.all([loadLocalModels(), loadConfig()])
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        downloadingId.value = null
        delete downloadProgress[modelId]
    }
}

async function handleCancelDownload() {
    try { await invoke('cancel_voice_download') } catch { /* ignore */ }
}

async function handleSetActive(modelId: string) {
    const model = localModels.value.find(m => m.id === modelId)
    if (!model) return
    try {
        await invoke('set_active_voice_model', { modelId, voiceType: model.voiceType })
        ElMessage.success(t('settings.voice.activateSuccess'))
        await loadConfig()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    }
}

async function handleDeleteModel(modelId: string) {
    try {
        await ElMessageBox.confirm(t('settings.voice.deleteConfirm'), t('settings.voice.deleteModel'), { type: 'warning' })
        await invoke('delete_voice_model', { modelId })
        ElMessage.success(t('settings.voice.deleteSuccess'))
        await Promise.all([loadLocalModels(), loadConfig()])
    } catch { /* cancelled */ }
}

async function handleRemoveCustom(modelId: string) {
    try {
        await ElMessageBox.confirm(t('settings.voice.removeCustomConfirm'), t('settings.voice.removeCustom'), { type: 'warning' })
        await invoke('remove_custom_voice_model', { modelId })
        ElMessage.success(t('settings.voice.removeCustomSuccess'))
        await loadLocalModels()
    } catch { /* cancelled */ }
}

async function handleAddCustom() {
    if (!customForm.modelId.trim()) {
        ElMessage.warning(t('settings.voice.customModelIdRequired'))
        return
    }
    addingCustom.value = true
    try {
        await invoke('add_custom_voice_model', {
            model: {
                modelId: customForm.modelId.trim(),
                voiceType: customForm.voiceType,
                name: customForm.name.trim() || customForm.modelId.trim(),
                sizeMb: customForm.sizeMb,
                license: customForm.license.trim(),
                description: customForm.description.trim(),
            },
        })
        ElMessage.success(t('settings.voice.addCustomSuccess'))
        showCustomDialog.value = false
        customForm.voiceType = 'tts'
        customForm.modelId = ''
        customForm.name = ''
        customForm.sizeMb = 100
        customForm.license = 'MIT'
        customForm.description = ''
        await loadLocalModels()
    } catch (e: unknown) {
        ElMessage.error(`${e}`)
    } finally {
        addingCustom.value = false
    }
}

function formatNumber(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K'
    return String(n)
}

let unlistenProgress: UnlistenFn | null = null

function handleProgressEvent(event: { payload: DownloadProgressPayload }) {
    const p = event.payload
    downloadProgress[p.modelId] = p
    if (p.status === 'complete' || p.status === 'cancelled') {
        setTimeout(() => { delete downloadProgress[p.modelId] }, 3000)
    }
}

onMounted(async () => {
    try { unlistenProgress = await listen<DownloadProgressPayload>('voice-download-progress', handleProgressEvent) } catch { /* ok */ }
    await Promise.all([loadConfig(), loadLocalModels()])
})

onUnmounted(() => {
    unlistenProgress?.()
})
</script>

<style scoped>
.voice-tabs :deep(.el-tabs__header) {
    margin-bottom: 16px;
}

.voice-tabs :deep(.el-tabs__item) {
    font-size: var(--app-font-13);
    font-weight: 500;
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
    margin: 16px 0 12px;
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

.voice-type-tag {
    font-size: 11px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 3px;
}

.voice-type-tag.tts {
    color: #409eff;
    background: rgba(64, 158, 255, 0.1);
}

.voice-type-tag.stt {
    color: #e6a23c;
    background: rgba(230, 162, 60, 0.1);
}

.custom-tag {
    font-size: 11px;
    font-weight: 600;
    color: #909399;
    background: rgba(144, 147, 153, 0.1);
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

.llm-empty {
    padding: 48px 0;
    text-align: center;
    font-size: 14px;
    color: var(--color-text-tertiary);
}
</style>
