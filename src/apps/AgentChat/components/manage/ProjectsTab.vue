<template>
  <div class="projects-tab">
    <div class="projects-header">
      <h2 class="projects-title">项目</h2>
      <div class="projects-actions">
        <button class="icon-btn" @click="handleRefresh" title="刷新">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
        </button>
        <button class="outline-btn" @click="toggleManagement">
          {{ managementMode ? '退出管理' : '管理' }}
        </button>
        <button class="add-btn" @click="handleAddProject">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          添加项目
        </button>
      </div>
    </div>

    <div class="projects-filters" v-if="store.projects.length > 0">
      <el-input
        v-model="searchQuery"
        placeholder="搜索项目..."
        size="small"
        clearable
        class="filter-search"
      />
      <div class="tag-filter" v-if="allTags.length > 0">
        <button
          v-for="tag in displayTags"
          :key="tag"
          class="tag-pill"
          :class="{ active: selectedTag === tag }"
          @click="selectedTag = selectedTag === tag ? null : tag"
        >
          {{ tag }}
        </button>
        <button v-if="allTags.length > TAG_LIMIT" class="tag-more" @click="tagsExpanded = !tagsExpanded">
          {{ tagsExpanded ? '收起' : `+${allTags.length - TAG_LIMIT}` }}
        </button>
        <button v-if="selectedTag" class="tag-clear" @click="selectedTag = null">清除</button>
      </div>
    </div>

    <div v-if="store.projects.length === 0" class="projects-empty">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--color-text-tertiary); opacity: 0.4">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <p>暂无项目</p>
      <p style="font-size: 12px">在终端运行 claude 初始化项目，或点击添加</p>
    </div>

    <div v-else-if="filteredProjects.length === 0" class="projects-empty">
      <p style="font-size: 13px">未找到匹配项目</p>
    </div>

    <div v-else class="projects-grid">
      <div
        v-for="project in filteredProjects"
        :key="project.encoded_name"
        class="project-card"
        :class="{ selected: selectedProject?.encoded_name === project.encoded_name }"
        @click="handleProjectClick(project)"
      >
        <div class="project-card-header">
          <div class="project-card-check" v-if="managementMode">
            <input type="checkbox" :checked="checkedProjects.has(project.encoded_name)" @click.stop="handleCheck(project.encoded_name)" />
          </div>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color: var(--icon-folder, #f0ad4e)">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          <span class="project-card-name">{{ getProjectDisplayName(project) }}</span>
          <span v-if="!project.initialized" class="init-badge" title="未初始化">!</span>
        </div>
        <div class="project-card-subtitle" v-if="getProjectDisplayName(project) !== project.name">{{ project.name }}</div>
        <div class="project-card-meta">
          <span>{{ project.session_count }} 会话</span>
          <span v-if="project.last_active">{{ project.last_active.slice(0, 10) }}</span>
        </div>
        <div class="project-card-tags" v-if="getProjectTags(project).length > 0">
          <span v-for="tag in getProjectTags(project).slice(0, 2)" :key="tag" class="card-tag" @click.stop="selectedTag = tag">{{ tag }}</span>
          <span v-if="getProjectTags(project).length > 2" class="card-tag-more">+{{ getProjectTags(project).length - 2 }}</span>
        </div>
        <div class="project-card-path" :title="project.path">{{ project.path }}</div>
        <div class="project-card-actions">
          <button v-if="!project.initialized" class="init-btn" @click.stop="handleInit(project.path)">
            初始化
          </button>
          <button class="enter-chat-btn" @click.stop="$emit('enter-project', project)">
            进入对话
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
          </button>
        </div>
      </div>
    </div>

    <div v-if="managementMode && checkedProjects.size >= 2" class="management-bar">
      <span>已选择 {{ checkedProjects.size }} 个项目</span>
      <div class="management-actions">
        <button class="outline-btn" @click="checkedProjects = new Set()">取消选择</button>
        <button class="add-btn" @click="mergeDialogVisible = true">合并项目</button>
      </div>
    </div>

    <div v-if="selectedProject" class="project-detail-panel">
      <div class="detail-header">
        <h3>{{ getProjectDisplayName(selectedProject) }}</h3>
        <button class="icon-btn" @click="selectedProject = null">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      <el-scrollbar class="detail-scroll">
        <div class="detail-info">
          <div class="detail-row">
            <span class="detail-label">路径</span>
            <span class="detail-value mono">{{ selectedProject.path }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">会话数</span>
            <span class="detail-value">{{ selectedProject.session_count }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">最近活跃</span>
            <span class="detail-value">{{ selectedProject.last_active ? selectedProject.last_active.slice(0, 16) : '-' }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">已初始化</span>
            <span class="detail-value" :class="{ 'text-danger': !selectedProject.initialized }">{{ selectedProject.initialized ? '是' : '否' }}</span>
          </div>
        </div>
        <div class="detail-section">
          <div class="detail-section-title">项目信息</div>
          <ProjectMetaEditor :project-path="selectedProject.path" :encoded-name="selectedProject.encoded_name" />
        </div>
        <div class="detail-section">
          <div class="detail-section-title">操作</div>
          <div class="detail-buttons">
            <el-button size="small" @click="handleOpenTerminal(selectedProject.path)">在终端打开</el-button>
            <el-button v-if="!selectedProject.initialized" size="small" type="warning" @click="handleInit(selectedProject.path)">初始化</el-button>
            <el-button size="small" type="danger" @click="handleHide(selectedProject.encoded_name)">移除</el-button>
          </div>
        </div>
      </el-scrollbar>
    </div>

    <MergeDialog
      v-model:visible="mergeDialogVisible"
      :projects="store.projects"
      @merged="handleMerged"
      @split="handleRefresh"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useAgentStore } from '@/store/agent'
import type { ProjectEntry } from '@/store/agent'
import ProjectMetaEditor from '../ProjectMetaEditor.vue'
import MergeDialog from '../MergeDialog.vue'

defineEmits<{
  'enter-project': [project: ProjectEntry]
}>()

const store = useAgentStore()
const searchQuery = ref('')
const selectedTag = ref<string | null>(null)
const tagsExpanded = ref(false)
const managementMode = ref(false)
const checkedProjects = ref<Set<string>>(new Set())
const selectedProject = ref<ProjectEntry | null>(null)
const mergeDialogVisible = ref(false)

const TAG_LIMIT = 8

const allTags = computed(() => {
  const tagSet = new Set<string>()
  Object.values(store.projectMetas).forEach(m => {
    m.tags?.forEach(t => tagSet.add(t))
  })
  return [...tagSet].sort()
})

const displayTags = computed(() => {
  return tagsExpanded.value ? allTags.value : allTags.value.slice(0, TAG_LIMIT)
})

const filteredProjects = computed(() => {
  let projects = store.projects
  if (selectedTag.value) {
    projects = projects.filter(p => store.projectMetas[p.encoded_name]?.tags?.includes(selectedTag.value!))
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    projects = projects.filter(p => {
      const meta = store.projectMetas[p.encoded_name]
      const name = meta?.custom_name?.toLowerCase() || p.name.toLowerCase()
      const tags = meta?.tags?.map(t => t.toLowerCase()) ?? []
      return name.includes(q) || tags.some(t => t.includes(q)) || p.path.toLowerCase().includes(q)
    })
  }
  return projects
})

onMounted(() => {
  store.loadProjectMetas()
})

function getProjectDisplayName(project: ProjectEntry): string {
  const meta = store.projectMetas[project.encoded_name]
  return meta?.custom_name || project.name
}

function getProjectTags(project: ProjectEntry): string[] {
  return store.projectMetas[project.encoded_name]?.tags ?? []
}

function handleRefresh() {
  store.loadProjects()
  store.loadAgentStatuses()
  store.loadProjectMetas()
}

async function handleAddProject() {
  try {
    const { value: path } = await ElMessageBox.prompt(
      '输入项目目录路径',
      '添加项目',
      { confirmButtonText: '添加', cancelButtonText: '取消' },
    )
    if (path?.trim()) {
      await store.addManualProject(path.trim())
      store.loadProjectMetas()
    }
  } catch {
    // cancelled
  }
}

function handleProjectClick(project: ProjectEntry) {
  if (selectedProject.value?.encoded_name === project.encoded_name) {
    selectedProject.value = null
    return
  }
  selectedProject.value = project
}

function handleCheck(encodedName: string) {
  const newChecked = new Set(checkedProjects.value)
  if (newChecked.has(encodedName)) {
    newChecked.delete(encodedName)
  } else {
    newChecked.add(encodedName)
  }
  checkedProjects.value = newChecked
}

function toggleManagement() {
  managementMode.value = !managementMode.value
  checkedProjects.value = new Set()
}

function handleInit(path: string) {
  store.initProject(path)
}

function handleHide(encodedName: string) {
  store.hideProject(encodedName)
  selectedProject.value = null
}

function handleOpenTerminal(path: string) {
  store.openInTerminal(path)
}

function handleMerged() {
  store.loadProjects()
  store.loadProjectMetas()
  checkedProjects.value = new Set()
  managementMode.value = false
}
</script>

<style scoped>
.projects-tab {
  padding: 24px;
  height: 100%;
  overflow-y: auto;
}

.projects-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.projects-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.projects-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid var(--color-card-border);
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.outline-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-card-border);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.outline-btn:hover {
  border-color: var(--color-text-tertiary);
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: none;
  background: #007AFF;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.add-btn:hover {
  opacity: 0.9;
}

.projects-filters {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.filter-search {
  width: 100%;
  max-width: 480px;
}

.filter-search :deep(.el-input__wrapper) {
  border-radius: 10px;
}

.tag-filter {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  max-width: 720px;
  justify-content: center;
}

.tag-pill {
  padding: 4px 10px;
  border-radius: 7px;
  border: none;
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-tertiary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.tag-pill:hover {
  color: var(--color-text-primary);
}

.tag-pill.active {
  background: #007AFF;
  color: #fff;
}

.tag-more, .tag-clear {
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: 12px;
  cursor: pointer;
}

.tag-more:hover, .tag-clear:hover {
  color: var(--color-text-primary);
}

.projects-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  gap: 8px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}

.project-card {
  padding: 14px;
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  background: var(--color-card-bg);
  cursor: pointer;
  transition: all 0.15s;
}

.project-card:hover {
  border-color: rgba(0, 122, 255, 0.3);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.project-card.selected {
  border-color: #007AFF;
}

.project-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.project-card-check {
  display: flex;
  align-items: center;
}

.project-card-check input {
  width: 16px;
  height: 16px;
  accent-color: #007AFF;
}

.project-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.init-badge {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #FF9500;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.project-card-subtitle {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 4px;
  margin-left: 24px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-card-meta {
  display: flex;
  gap: 8px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 4px;
}

.project-card-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}

.card-tag {
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-tertiary);
  font-size: 11px;
  cursor: pointer;
}

.card-tag:hover {
  color: var(--color-text-primary);
}

.card-tag-more {
  padding: 2px 4px;
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.project-card-path {
  font-size: 11px;
  font-family: monospace;
  color: var(--color-text-tertiary);
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 8px;
}

.project-card-actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.init-btn {
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid #FF9500;
  background: transparent;
  color: #FF9500;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.init-btn:hover {
  background: rgba(255, 149, 0, 0.1);
}

.enter-chat-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #007AFF;
  font-size: 12px;
  cursor: pointer;
  margin-left: auto;
  transition: background 0.15s;
}

.enter-chat-btn:hover {
  background: rgba(0, 122, 255, 0.1);
}

.management-bar {
  position: sticky;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--color-card-border);
  background: var(--color-bg-glass);
  z-index: 10;
}

.management-actions {
  display: flex;
  gap: 8px;
}

.project-detail-panel {
  position: fixed;
  right: 0;
  top: 0;
  bottom: 0;
  width: 380px;
  background: var(--color-card-bg);
  border-left: 1px solid var(--color-card-border);
  z-index: 20;
  display: flex;
  flex-direction: column;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.08);
  animation: slideIn 0.15s ease;
}

@keyframes slideIn {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-card-border);
}

.detail-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.detail-scroll {
  flex: 1;
  padding: 16px 20px;
}

.detail-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
}

.detail-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.detail-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  width: 70px;
}

.detail-value {
  font-size: 13px;
  color: var(--color-text-primary);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-value.mono {
  font-family: monospace;
  font-size: 12px;
}

.text-danger {
  color: var(--el-color-danger);
}

.detail-section {
  margin-bottom: 16px;
}

.detail-section-title {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.detail-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
