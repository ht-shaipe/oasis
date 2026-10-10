<template>
    <div class="tool-panel">
        <div class="panel-body">            <div class="toolbar">
                <div class="toolbar-left">
                    <el-button @click="pickDir" :icon="FolderOpened">{{ t('toolbox.imageCompress.selectDir') }}</el-button>
                    <el-button @click="pickFiles" :icon="Folder">{{ t('toolbox.imageCompress.selectFiles') }}</el-button>
                    <el-button v-if="images.length > 0" link type="danger" @click="clearImages">{{ t('toolbox.imageCompress.clear') }}</el-button>
                </div>
                <div class="toolbar-right">
                    <span class="config-label">{{ t('toolbox.imageCompress.quality') }}</span>
                    <el-slider v-model="quality" :min="1" :max="100" :show-tooltip="true" style="width: 80px" />
                    <span class="config-value">{{ quality }}%</span>
                    <span class="config-label">{{ t('toolbox.imageCompress.scale') }}</span>
                    <el-select v-model="scale" style="width: 90px" clearable :placeholder="t('toolbox.imageCompress.noScale')">
                        <el-option :label="t('toolbox.imageCompress.noScale')" :value="null" />
                        <el-option label="25%" :value="25" />
                        <el-option label="50%" :value="50" />
                        <el-option label="75%" :value="75" />
                        <el-option label="80%" :value="80" />
                        <el-option label="90%" :value="90" />
                    </el-select>
                    <span class="config-label">{{ t('toolbox.outputFormat') }}</span>
                    <el-select v-model="format" style="width: 90px">
                        <el-option :label="t('toolbox.imageCompress.keepOriginal')" :value="null" />
                        <el-option label="JPEG" value="jpeg" />
                        <el-option label="PNG" value="png" />
                        <el-option label="WebP" value="webp" />
                    </el-select>
                    <el-button type="primary" @click="compress" :loading="loading" :disabled="images.length === 0">
                        {{ loading ? t('toolbox.imageCompress.compressing') : t('toolbox.run') }}
                    </el-button>
                </div>
            </div>

            <div v-if="loading || doneCount > 0" class="progress-bar-area">
                <el-progress
                    :percentage="images.length > 0 ? Math.round((doneCount + images.filter(i => i.status === 'error').length) / images.length * 100) : 0"
                    :stroke-width="4"
                    :show-text="true"
                    :format="() => `${doneCount + images.filter(i => i.status === 'error').length} / ${images.length}`"
                />
                <span v-if="totalOriginal > 0" class="progress-summary">
                    {{ formatSize(totalOriginal) }} → {{ formatSize(totalCompressed) }}
                    ( -{{ totalRatio.toFixed(1) }}% )
                </span>
            </div>

            <div v-if="images.length > 0" class="image-list">
                <div
                    v-for="(item, idx) in images"
                    :key="item.path"
                    class="image-row"
                    :class="{ done: item.status === 'done', error: item.status === 'error' }">
                    <div class="image-info">
                        <span class="image-name" :title="item.path">{{ item.name }}</span>
                        <span class="image-meta">{{ item.width }}×{{ item.height }}</span>
                        <template v-if="item.status === 'done' && item.compressedWidth > 0">
                            <span class="size-arrow">→</span>
                            <span class="image-meta compressed">{{ item.compressedWidth }}×{{ item.compressedHeight }}</span>
                        </template>
                        <span class="image-format">{{ item.format.toUpperCase() }}</span>
                    </div>
                    <div class="image-sizes">
                        <template v-if="item.status === 'pending'">
                            <span class="size-original">{{ formatSize(item.originalSize) }}</span>
                        </template>
                        <template v-else-if="item.status === 'compressing'">
                            <span class="size-original">{{ formatSize(item.originalSize) }}</span>
                            <span class="compressing-indicator">
                                <el-icon class="is-loading"><Loading /></el-icon>
                            </span>
                        </template>
                        <template v-else-if="item.status === 'done'">
                            <span class="size-original strikethrough">{{ formatSize(item.originalSize) }}</span>
                            <span class="size-arrow">→</span>
                            <span class="size-compressed">{{ formatSize(item.compressedSize) }}</span>
                            <span class="size-ratio">-{{ item.ratio.toFixed(1) }}%</span>
                        </template>
                        <template v-else-if="item.status === 'error'">
                            <span class="size-original">{{ formatSize(item.originalSize) }}</span>
                            <span class="size-error" :title="item.errorMsg">✗</span>
                        </template>
                    </div>
                    <el-button
                        v-if="!loading"
                        link
                        size="small"
                        class="row-remove"
                        @click="removeImage(idx)">×</el-button>
                </div>
            </div>

            <div v-else class="empty-hint">
                {{ t('toolbox.imageCompress.emptyHint') }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Folder, FolderOpened, Loading } from '@element-plus/icons-vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { useImageCompress } from '../composables/tools/useImageCompress';

const { t } = useI18n();
const { images, quality, format, scale, loading, totalOriginal, totalCompressed, totalRatio, doneCount, scanDir, scanFiles, removeImage, clearImages, compress } =
    useImageCompress();

function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

async function pickDir() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await scanDir(dir as string, false);
}

async function pickFiles() {
    const files = await open({
        multiple: true,
        directory: false,
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'tiff'] }],
    });
    if (files && Array.isArray(files)) {
        await scanFiles(files as string[]);
    } else if (files) {
        await scanFiles([files as string]);
    }
}
</script>

<style scoped>
.panel-body {
    display: flex;
    flex-direction: column;
    gap: 0;
    height: 100%;
    overflow: hidden;
}

.toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--color-card-border, rgba(0, 0, 0, 0.06));
    flex-wrap: wrap;
    flex-shrink: 0;
}

.toolbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
}

.toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
}

.config-label {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-tertiary, #999);
    white-space: nowrap;
}

.config-value {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-secondary, #666);
    min-width: 32px;
}

.progress-bar-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    flex-shrink: 0;
}

.progress-bar-area :deep(.el-progress) {
    flex: 1;
}

.progress-bar-area :deep(.el-progress__text) {
    font-size: var(--app-font-12, 12px) !important;
    min-width: 50px;
}

.progress-summary {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-secondary, #666);
    white-space: nowrap;
}

.image-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-top: 8px;
}

.image-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    transition: background 0.12s;
}

.image-row:hover {
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.04));
}

.image-row.done {
    background: rgba(45, 140, 60, 0.04);
}

.image-row.error {
    background: rgba(212, 48, 48, 0.04);
}

.image-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
}

.image-name {
    font-size: var(--app-font-13, 13px);
    color: var(--color-text-primary, #333);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.image-meta {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-tertiary, #999);
    white-space: nowrap;
}

.image-meta.compressed {
    color: #2d8c3c;
}

.image-format {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-tertiary, #999);
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.04));
    padding: 1px 5px;
    border-radius: 3px;
}

.image-sizes {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
    font-size: var(--app-font-12, 12px);
}

.size-original {
    color: var(--color-text-secondary, #666);
}

.size-original.strikethrough {
    text-decoration: line-through;
    opacity: 0.6;
}

.size-arrow {
    color: var(--color-text-tertiary, #999);
}

.size-compressed {
    color: #2d8c3c;
    font-weight: 500;
}

.size-ratio {
    color: #2d8c3c;
    font-weight: 600;
}

.size-error {
    color: #d43030;
    cursor: help;
}

.compressing-indicator {
    color: var(--color-menu-hover, #0078d7);
    display: flex;
    align-items: center;
}

.row-remove {
    opacity: 0;
    transition: opacity 0.15s;
    color: var(--color-text-tertiary, #999);
}

.image-row:hover .row-remove {
    opacity: 1;
}

.empty-hint {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-13, 13px);
}
</style>
