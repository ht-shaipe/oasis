<template>
    <AppDialog
        v-model="dialogVisible"
        :title="t('update.title')"
        width="480px"
        custom-class="update-dialog"
        :show-close="false"
        :close-on-backdrop="false"
        @close="handleClose">
        <div class="update-container">
            <div class="update-header">
                <h3 class="update-title">{{ t('update.newVersionAvailable') }}</h3>
                <button class="close-btn" @click="handleClose">
                    <el-icon :size="18"><Close /></el-icon>
                </button>
            </div>

            <div class="update-version-info">
                <span class="version-text">
                    Oasis {{ latestVersion }} {{ t('update.published') }}，{{ t('update.currentVersion') }} {{ currentVersion }}
                </span>
            </div>

            <div class="update-notes" v-if="updateInfo?.release_notes?.length">
                <div
                    v-for="(section, idx) in updateInfo.release_notes"
                    :key="idx"
                    class="note-section">
                    <div class="note-section-title" v-if="section.title">{{ section.title }}</div>
                    <ul class="note-list">
                        <li v-for="(item, i) in section.items" :key="i" class="note-item">
                            <span class="note-bullet">•</span>
                            <span class="note-text" v-html="renderItem(item)"></span>
                        </li>
                    </ul>
                </div>
            </div>

            <div class="update-footer">
                <el-button class="footer-btn secondary-btn" @click="handleOpenDownloadPage">
                    {{ t('update.openDownloadPage') }}
                </el-button>
                <el-button
                    class="footer-btn primary-btn"
                    :class="{ 'is-downloading': downloadStatus === 'downloading' }"
                    :disabled="downloadStatus === 'downloading' || downloadStatus === 'completed'"
                    @click="handleDownload">
                    <el-icon v-if="downloadStatus === 'downloading'" class="download-spin">
                        <Loading />
                    </el-icon>
                    <template v-if="downloadStatus === 'downloading'">
                        {{ t('update.downloading') }} {{ downloadProgress }}%
                    </template>
                    <template v-else-if="downloadStatus === 'completed'">
                        {{ t('update.downloadComplete') }}
                    </template>
                    <template v-else-if="downloadStatus === 'error'">
                        {{ t('update.downloadFailed') }}
                    </template>
                    <template v-else>
                        {{ t('update.downloadNow') }}
                    </template>
                </el-button>
            </div>
        </div>
    </AppDialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Close, Loading } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import AppDialog from '@/components/common/AppDialog.vue'
import type { UpdateInfo } from '@/composables/useAppUpdate'

const { t } = useI18n()

const props = defineProps<{
    visible: boolean
    currentVersion: string
    latestVersion: string
    updateInfo: UpdateInfo | null
    downloadProgress: number
    downloadStatus: 'idle' | 'downloading' | 'completed' | 'error'
}>()

const emit = defineEmits<{
    (e: 'update:visible', value: boolean): void
    (e: 'download'): void
    (e: 'openDownloadPage'): void
}>()

const dialogVisible = computed({
    get: () => props.visible,
    set: (val) => emit('update:visible', val),
})

const handleClose = () => {
    emit('update:visible', false)
}

const handleDownload = () => {
    emit('download')
}

const handleOpenDownloadPage = () => {
    emit('openDownloadPage')
}

const renderItem = (item: string) => {
    return item.replace(
        /#(\d+)/g,
        '<a href="https://github.com/ht-shaipe/oasis/issues/$1" target="_blank" class="issue-link">#$1</a>'
    ).replace(
        /\[(.*?)\]\((.*?)\)/g,
        '<a href="$2" target="_blank" class="issue-link">$1</a>'
    )
}
</script>

<style scoped>
.update-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.update-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.update-title {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
    color: var(--color-text-primary);
}

.close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
}

.close-btn:hover {
    background: var(--color-input-bg);
    color: var(--color-text-primary);
}

.update-version-info {
    font-size: 13px;
    color: var(--color-text-secondary);
}

.version-text {
    line-height: 1.5;
}

.update-notes {
    max-height: 280px;
    overflow-y: auto;
    padding: 12px 16px;
    background: var(--color-input-bg);
    border-radius: 8px;
    border: 1px solid var(--color-border);
}

.update-notes::-webkit-scrollbar {
    width: 6px;
}

.update-notes::-webkit-scrollbar-thumb {
    background: var(--color-text-tertiary);
    border-radius: 3px;
}

.note-section + .note-section {
    margin-top: 14px;
}

.note-section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 6px;
}

.note-list {
    list-style: none;
    margin: 0;
    padding: 0;
}

.note-item {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--color-text-secondary);
}

.note-bullet {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
}

.note-text {
    word-break: break-word;
}

.note-text :deep(.issue-link) {
    color: #42b883;
    text-decoration: none;
}

.note-text :deep(.issue-link:hover) {
    text-decoration: underline;
}

.update-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding-top: 4px;
}

.footer-btn {
    min-width: 120px;
    height: 36px;
    border-radius: 8px;
    font-size: 13px;
}

.secondary-btn {
    background: var(--color-input-bg);
    border-color: var(--color-border);
    color: var(--color-text-primary);
}

.secondary-btn:hover {
    border-color: #42b883;
    color: #42b883;
}

.primary-btn {
    background: #42b883;
    border-color: #42b883;
    color: #fff;
}

.primary-btn:hover {
    background: #369a6d;
    border-color: #369a6d;
}

.primary-btn.is-downloading {
    opacity: 0.85;
}

.download-spin {
    animation: spin 1s linear infinite;
    margin-right: 6px;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}
</style>
