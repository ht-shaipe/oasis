import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface KnowledgeStatus {
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

export interface LLMModel {
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

export interface LocalEmbeddingModel {
  id: string
  name: string
  dimensions: number
  quantized: boolean
  sizeMb: number
  license: string
  downloaded: boolean
  isCustom: boolean
}

export interface DownloadProgressPayload {
  modelId: string
  fileName: string
  current: number
  total: number
  percentage: number
  status: string
}

export interface IndexProgressPayload {
  phase: string
  current: number
  total: number
  message: string
}

const status = ref<KnowledgeStatus | null>(null)
const remoteModels = ref<LLMModel[]>([])
const localModels = ref<LocalEmbeddingModel[]>([])
const embedMode = ref<'local' | 'remote'>('local')
const selectedModelId = ref('')
const localModelId = ref('')
const isIndexing = ref(false)
const incrementalMode = ref(true)
const downloadingId = ref<string | null>(null)
const downloadProgress = reactive<Record<string, DownloadProgressPayload>>({})
const indexProgress = ref<IndexProgressPayload>({ phase: '', current: 0, total: 0, message: '' })

let initPromise: Promise<void> | null = null
let unlistenProgress: UnlistenFn | null = null
let unlistenIndexProgress: UnlistenFn | null = null

function handleProgressEvent(event: { payload: DownloadProgressPayload }) {
  const p = event.payload
  downloadProgress[p.modelId] = p
  if (p.status === 'complete' || p.status === 'cancelled') {
    setTimeout(() => {
      delete downloadProgress[p.modelId]
    }, 3000)
  }
}

function handleIndexProgress(event: { payload: IndexProgressPayload }) {
  indexProgress.value = event.payload
}

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
    const config = await invoke<{ activeLocalModelId: string | null }>('get_embedding_config')
    if (config.activeLocalModelId) {
      localModelId.value = config.activeLocalModelId
    } else {
      const downloaded = models.filter((m) => m.downloaded)
      if (downloaded.length > 0) {
        localModelId.value = downloaded[0].id
      }
    }
  } catch (e) {
    console.error('Failed to load local models:', e)
  }
}

async function ensureInit() {
  if (initPromise) return initPromise
  initPromise = (async () => {
    unlistenProgress = await listen<DownloadProgressPayload>('embed-download-progress', handleProgressEvent)
    unlistenIndexProgress = await listen<IndexProgressPayload>('index-progress', handleIndexProgress)
    await Promise.all([loadStatus(), loadRemoteModels(), loadLocalModels()])
  })()
  return initPromise
}

function cleanup() {
  unlistenProgress?.()
  unlistenIndexProgress?.()
  unlistenProgress = null
  unlistenIndexProgress = null
}

export function useKnowledge() {
  const { t } = useI18n()

  const phaseLabel = computed(() => {
    const phaseMap: Record<string, string> = {
      scanning: t('knowledge.indexPhaseScanning'),
      embedding: t('knowledge.indexPhaseEmbedding'),
      building_index: t('knowledge.indexPhaseBuildingIndex'),
    }
    return phaseMap[indexProgress.value.phase] || t('knowledge.indexingProgress')
  })

  const activeLocalModel = computed(() => {
    if (!localModelId.value) return null
    return localModels.value.find((m) => m.id === localModelId.value) || null
  })

  const inactiveModels = computed(() => {
    return localModels.value.filter((m) => m.id !== localModelId.value)
  })

  return {
    status,
    remoteModels,
    localModels,
    embedMode,
    selectedModelId,
    localModelId,
    isIndexing,
    incrementalMode,
    downloadingId,
    downloadProgress,
    indexProgress,
    phaseLabel,
    activeLocalModel,
    inactiveModels,
    loadStatus,
    loadRemoteModels,
    loadLocalModels,
    ensureInit,
    cleanup,
  }
}
