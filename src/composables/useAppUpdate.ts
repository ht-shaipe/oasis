import { ref, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface ReleaseNoteSection {
  title: string
  items: string[]
}

export interface UpdateInfo {
  version: string
  download_url: string
  release_notes: ReleaseNoteSection[]
  published_at: string
}

export interface CheckUpdateResult {
  has_update: boolean
  current_version: string
  latest_version: string
  update_info: UpdateInfo | null
}

export type DownloadStatus = 'idle' | 'downloading' | 'completed' | 'error'

export function useAppUpdate() {
  const hasUpdate = ref(false)
  const currentVersion = ref('')
  const latestVersion = ref('')
  const updateInfo = ref<UpdateInfo | null>(null)
  const checking = ref(false)
  const downloadProgress = ref(0)
  const downloadStatus = ref<DownloadStatus>('idle')
  const downloadPath = ref('')
  const errorMsg = ref('')

  let unlistenProgress: UnlistenFn | null = null

  const checkForUpdate = async () => {
    checking.value = true
    errorMsg.value = ''
    try {
      const result = await invoke<CheckUpdateResult>('check_update')
      hasUpdate.value = result.has_update
      currentVersion.value = result.current_version
      latestVersion.value = result.latest_version
      updateInfo.value = result.update_info
      return result
    } catch (e) {
      errorMsg.value = String(e)
      return null
    } finally {
      checking.value = false
    }
  }

  const startDownload = async () => {
    if (!updateInfo.value?.download_url) return
    downloadStatus.value = 'downloading'
    downloadProgress.value = 0
    errorMsg.value = ''

    try {
      const path = await invoke<string>('download_update', {
        url: updateInfo.value.download_url,
      })
      downloadPath.value = path
      downloadStatus.value = 'completed'
    } catch (e) {
      downloadStatus.value = 'error'
      errorMsg.value = String(e)
    }
  }

  const openDownloadPage = async () => {
    if (!updateInfo.value?.download_url) return
    try {
      await invoke('plugin:opener|open_url', {
        url: updateInfo.value.download_url,
      })
    } catch {
      window.open(updateInfo.value.download_url, '_blank')
    }
  }

  const setupProgressListener = async () => {
    unlistenProgress = await listen<number>('update-download-progress', (event) => {
      downloadProgress.value = event.payload
    })
  }

  onMounted(async () => {
    await setupProgressListener()
  })

  onBeforeUnmount(() => {
    unlistenProgress?.()
  })

  return {
    hasUpdate,
    currentVersion,
    latestVersion,
    updateInfo,
    checking,
    downloadProgress,
    downloadStatus,
    downloadPath,
    errorMsg,
    checkForUpdate,
    startDownload,
    openDownloadPage,
  }
}
