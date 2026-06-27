<template>
    <MacWindow
        ref="macWindowRef"
        title="Finder"
        :isMinimized="isMinimized"
        @close="closeApp"
        @minimize="toggleMinimize"
        :width="1000"
        :height="600"
    >
        <div class="finder-container">
            <div class="finder-sidebar">
                <div class="sidebar-section">
                    <div class="section-title">{{ t('finder.favorites') }}</div>
                    <div
                        v-for="item in sidebarItems"
                        :key="item.key"
                        :class="['sidebar-item', { active: activeSidebar === item.key }]"
                        @click="navigateToSidebar(item)"
                    >
                        <el-icon><component :is="item.icon" /></el-icon>
                        <span>{{ item.label }}</span>
                    </div>
                </div>
                <div class="sidebar-section">
                    <div class="section-title">{{ t('knowledge.title') }}</div>
                    <div
                        :class="['sidebar-item', { active: activeSidebar === 'knowledge' }]"
                        @click="activeSidebar = 'knowledge'"
                    >
                        <el-icon><Collection /></el-icon>
                        <span>{{ t('knowledge.indexStatus') }}</span>
                    </div>
                    <div
                        :class="['sidebar-item', { active: activeSidebar === 'semantic-search' }]"
                        @click="activeSidebar = 'semantic-search'"
                    >
                        <el-icon><Search /></el-icon>
                        <span>{{ t('knowledge.semanticSearch') }}</span>
                    </div>
                </div>
            </div>

            <div class="finder-content">
                <KnowledgePanel v-if="activeSidebar === 'knowledge' || activeSidebar === 'semantic-search'" />

                <template v-else>
                <div class="finder-toolbar">
                    <div class="view-controls">
                        <el-radio-group v-model="viewMode" size="small">
                            <el-radio-button value="grid">
                                <el-icon><Grid /></el-icon>
                            </el-radio-button>
                            <el-radio-button value="list">
                                <el-icon><List /></el-icon>
                            </el-radio-button>
                        </el-radio-group>
                    </div>

                    <div class="path-navigator">
                        <template v-for="(seg, idx) in pathSegments" :key="idx">
                            <el-button link size="small" @click="navigateToIndex(idx)">{{ seg.name }}</el-button>
                            <span v-if="idx < pathSegments.length - 1" class="path-separator">/</span>
                        </template>
                    </div>

                    <div class="search-box">
                        <el-input
                            v-model="searchQuery"
                            :placeholder="t('finder.searchPlaceholder')"
                            :prefix-icon="Search"
                            clearable
                            size="small"
                        />
                    </div>
                </div>

                <div v-if="loading" class="loading-container">
                    <el-icon class="loading-icon"><Loading /></el-icon>
                    <p>{{ t('finder.loading') }}</p>
                </div>

                <el-empty v-else-if="filteredEntries.length === 0 && !searchQuery" :description="t('finder.emptyFolder')" />

                <el-scrollbar v-else-if="viewMode === 'grid'" class="files-container grid">
                    <div
                        v-if="canGoBack"
                        class="file-item back-item"
                        @click="goBack"
                    >
                        <div class="file-icon back-icon">
                            <el-icon><Back /></el-icon>
                        </div>
                        <div class="file-name">..</div>
                    </div>
                    <div
                        v-for="entry in filteredEntries"
                        :key="entry.path"
                        class="file-item"
                        @click="handleClick(entry)"
                        @dblclick="handleDblClick(entry)"
                    >
                        <div :class="['file-icon', entry.is_dir ? 'folder-icon' : 'doc-icon']">
                            <el-icon><component :is="entry.is_dir ? Folder : Document" /></el-icon>
                        </div>
                        <div class="file-name">{{ entry.name }}</div>
                        <div v-if="!entry.is_dir" class="file-meta">{{ formatFileSize(entry.size) }}</div>
                    </div>
                </el-scrollbar>

                <el-scrollbar v-else class="files-container list">
                    <el-table
                        :data="filteredEntries"
                        style="width: 100%"
                        @row-click="handleClick"
                        @row-dblclick="handleDblClick"
                        @sort-change="handleSortChange"
                    >
                        <el-table-column :label="t('finder.name')" min-width="260" prop="name" sortable="custom">
                            <template #default="{ row }">
                                <div class="list-item-name">
                                    <el-icon :class="row.is_dir ? 'folder-color' : 'doc-color'">
                                        <component :is="row.is_dir ? Folder : Document" />
                                    </el-icon>
                                    <span>{{ row.name }}</span>
                                </div>
                            </template>
                        </el-table-column>
                        <el-table-column :label="t('finder.modified')" width="180" prop="modified" sortable="custom">
                            <template #default="{ row }">
                                {{ formatDate(row.modified) }}
                            </template>
                        </el-table-column>
                        <el-table-column :label="t('finder.kind')" width="120" prop="kind" sortable="custom">
                            <template #default="{ row }">
                                {{ row.is_dir ? t('finder.folderKind') : (row.extension || t('finder.fileKind')) }}
                            </template>
                        </el-table-column>
                        <el-table-column :label="t('finder.size')" width="120" prop="size" sortable="custom">
                            <template #default="{ row }">
                                {{ row.is_dir ? '--' : formatFileSize(row.size) }}
                            </template>
                        </el-table-column>
                    </el-table>
                </el-scrollbar>

                <div class="finder-statusbar">
                    {{ filteredEntries.length }} {{ t('finder.itemCount') }}
                    <span v-if="currentDir"> — {{ currentDir }}</span>
                </div>
                </template>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import MacWindow from '@/components/common/MacWindow.vue'
import KnowledgePanel from './Finder/components/KnowledgePanel.vue'
import {
    Folder, Document, Loading, Grid, List, Search,
    HomeFilled, Download, Files, Back, Collection,
} from '@element-plus/icons-vue'

const { t } = useI18n()

interface DirEntry {
    name: string
    path: string
    is_dir: boolean
    size: number
    modified: number
    extension: string
}

const props = defineProps({
    isMinimized: { type: Boolean, default: false },
    browserFingerprint: { type: String, default: '' },
})

const emit = defineEmits(['close', 'minimize', 'openApp', 'openEditor'])

const loading = ref(false)
const entries = ref<DirEntry[]>([])
const currentDir = ref('')
const workspaceDir = ref('')
const searchQuery = ref('')
const viewMode = ref('grid')
const activeSidebar = ref('workspace')
const historyStack = ref<string[]>([])
const sortKey = ref<'name' | 'modified' | 'kind' | 'size' | ''>('')
const sortOrder = ref<'ascending' | 'descending' | ''>('')

const sidebarItems = computed(() => [
    { key: 'workspace', label: t('finder.workspace'), icon: Files, path: workspaceDir.value },
    { key: 'home', label: t('finder.home'), icon: HomeFilled, path: '' },
    { key: 'desktop', label: t('finder.desktop'), icon: Folder, path: '' },
    { key: 'documents', label: t('finder.documents'), icon: Folder, path: '' },
    { key: 'downloads', label: t('finder.downloads'), icon: Download, path: '' },
])

const canGoBack = computed(() => {
    if (!currentDir.value || !workspaceDir.value) return false
    return currentDir.value !== workspaceDir.value
})

const pathSegments = computed(() => {
    if (!currentDir.value) return []
    const parts = currentDir.value.split('/').filter(Boolean)
    let accumulated = ''
    return parts.map((name, _idx) => {
        accumulated += '/' + name
        return { name, path: accumulated }
    })
})

const filteredEntries = computed(() => {
    let result = searchQuery.value
        ? entries.value.filter((e) => e.name.toLowerCase().includes(searchQuery.value.toLowerCase()))
        : entries.value

    if (sortKey.value && sortOrder.value) {
        const dir = sortOrder.value === 'ascending' ? 1 : -1
        result = [...result].sort((a, b) => {
            let va: string | number = ''
            let vb: string | number = ''
            switch (sortKey.value) {
                case 'name': va = a.name.toLowerCase(); vb = b.name.toLowerCase(); break
                case 'modified': va = a.modified; vb = b.modified; break
                case 'kind': va = a.is_dir ? '' : (a.extension || ''); vb = b.is_dir ? '' : (b.extension || ''); break
                case 'size': va = a.is_dir ? -1 : a.size; vb = b.is_dir ? -1 : b.size; break
            }
            if (va < vb) return -1 * dir
            if (va > vb) return 1 * dir
            return 0
        })
    }

    return result
})

function handleSortChange({ prop, order }: { prop: string; order: 'ascending' | 'descending' | null }) {
    sortKey.value = (prop as 'name' | 'modified' | 'kind' | 'size' | '') || ''
    sortOrder.value = order || ''
}

const closeApp = () => emit('close')
const toggleMinimize = () => emit('minimize')

async function loadDir(dirPath: string) {
    loading.value = true
    try {
        const list = await invoke<DirEntry[]>('read_dir_entries', { dirPath })
        entries.value = list
        currentDir.value = dirPath
    } catch (e: unknown) {
        ElMessage.error(`${t('finder.readDirFailed')}: ${e}`)
    } finally {
        loading.value = false
    }
}

async function navigateToSidebar(item: { key: string; path: string }) {
    activeSidebar.value = item.key
    let path = item.path
    if (!path) {
        try {
            const home = await invoke<string>('get_workspace_dir')
            const homeBase = home.split('/').slice(0, -1).join('/')
            switch (item.key) {
                case 'home': path = homeBase || home; break
                case 'desktop': path = homeBase + '/Desktop'; break
                case 'documents': path = homeBase + '/Documents'; break
                case 'downloads': path = homeBase + '/Downloads'; break
            }
        } catch {
            return
        }
    }
    if (path) {
        historyStack.value = []
        await loadDir(path)
    }
}

function navigateToIndex(idx: number) {
    const target = pathSegments.value[idx]?.path
    if (target) {
        const current = currentDir.value
        historyStack.value.push(current)
        loadDir(target)
    }
}

function goBack() {
    if (historyStack.value.length > 0) {
        const prev = historyStack.value.pop()!
        loadDir(prev)
    } else {
        const parts = currentDir.value.split('/').filter(Boolean)
        if (parts.length > 1) {
            const parent = '/' + parts.slice(0, -1).join('/')
            loadDir(parent)
        }
    }
}

function handleClick(_entry: DirEntry) {
    // single click: select (no-op for now)
}

function handleDblClick(entry: DirEntry) {
    if (entry.is_dir) {
        historyStack.value.push(currentDir.value)
        loadDir(entry.path)
    } else {
        invoke('plugin:opener|open_path', { path: entry.path }).catch(() => {})
    }
}

function formatDate(timestamp: number): string {
    if (!timestamp) return ''
    const d = new Date(timestamp * 1000)
    return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`
}

function formatFileSize(size: number): string {
    if (size < 1024) return size + ' B'
    if (size < 1024 * 1024) return (size / 1024).toFixed(1) + ' KB'
    if (size < 1024 * 1024 * 1024) return (size / (1024 * 1024)).toFixed(1) + ' MB'
    return (size / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

onMounted(async () => {
    try {
        workspaceDir.value = await invoke<string>('get_workspace_dir')
        activeSidebar.value = 'workspace'
        await loadDir(workspaceDir.value)
    } catch (e: unknown) {
        ElMessage.error(`${t('finder.readDirFailed')}: ${e}`)
    }
})

const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null)
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront(),
})
</script>

<style scoped>
.finder-container {
    display: flex;
    height: 100%;
    background-color: var(--color-sidebar-bg);
}

.finder-sidebar {
    width: 180px;
    background-color: var(--color-sidebar-bg);
    border-right: 1px solid var(--color-window-titlebar-border);
    padding: 10px 0;
    overflow-y: auto;
}

.sidebar-section {
    margin-bottom: 20px;
}

.section-title {
    padding: 0 16px;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-tertiary);
    margin-bottom: 5px;
    text-transform: uppercase;
}

.sidebar-item {
    display: flex;
    align-items: center;
    padding: 7px 16px;
    cursor: pointer;
    border-radius: 6px;
    margin: 1px 6px;
    font-size: 13px;
}

.sidebar-item .el-icon {
    margin-right: 8px;
    font-size: 16px;
    color: var(--color-text-secondary);
}

.sidebar-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
}

.sidebar-item.active {
    background-color: rgba(0, 123, 255, 0.1);
    color: #007bff;
}

.sidebar-item.active .el-icon {
    color: #007bff;
}

.finder-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.finder-toolbar {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background-color: var(--color-sidebar-bg);
    border-bottom: 1px solid var(--color-window-titlebar-border);
    height: 46px;
}

.view-controls {
    display: flex;
    align-items: center;
}

.path-navigator {
    margin: 0 16px;
    flex: 1;
    display: flex;
    align-items: center;
    font-size: 13px;
    overflow: hidden;
}

.path-separator {
    margin: 0 4px;
    color: var(--color-text-tertiary);
}

.search-box {
    width: 180px;
    flex-shrink: 0;
}

.files-container {
    flex: 1;
    background-color: #fff;
}

.files-container.grid :deep(.el-scrollbar__wrap) {
    padding: 0;
}

.files-container.grid :deep(.el-scrollbar__view) {
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    grid-gap: 8px;
    align-content: start;
}

.files-container.list :deep(.el-scrollbar__wrap) {
    padding: 0;
}

.files-container.list :deep(.el-table__body-wrapper) {
    overflow: hidden;
}

.files-container.list :deep(.el-table__inner-wrapper) {
    overflow: hidden;
}

.file-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    cursor: pointer;
    padding: 10px 6px;
    border-radius: 8px;
    transition: background-color 0.15s;
    user-select: none;
}

.file-item:hover {
    background-color: rgba(0, 0, 0, 0.04);
}

.file-icon {
    font-size: 40px;
    margin-bottom: 6px;
}

.folder-icon {
    color: #54aeff;
}

.doc-icon {
    color: #8b949e;
}

.back-icon {
    color: var(--color-text-tertiary);
    font-size: 32px;
}

.back-item {
    opacity: 0.7;
}

.back-item:hover {
    opacity: 1;
}

.file-name {
    font-size: 12px;
    word-break: break-word;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.3;
}

.file-meta {
    font-size: 11px;
    color: var(--color-text-tertiary);
    margin-top: 2px;
}

.list-item-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
}

.list-item-name .el-icon {
    font-size: 18px;
}

.folder-color {
    color: #54aeff;
}

.doc-color {
    color: #8b949e;
}

.finder-statusbar {
    height: 24px;
    background-color: var(--color-sidebar-bg);
    border-top: 1px solid var(--color-window-titlebar-border);
    font-size: 12px;
    color: var(--color-text-tertiary);
    padding: 0 16px;
    display: flex;
    align-items: center;
}

.loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    width: 100%;
}

.loading-icon {
    font-size: 32px;
    color: #409eff;
    animation: spin 2s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

.files-container :deep(.el-table) {
    --el-table-bg-color: #fff;
    --el-table-tr-bg-color: #fff;
}

.files-container :deep(.el-table th.el-table__cell) {
    background-color: #fafafa;
    font-size: 12px;
    color: var(--color-text-tertiary);
}
</style>
