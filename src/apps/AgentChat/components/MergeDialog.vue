<template>
  <AppDialog
    v-model="dialogVisible"
    title="项目合并 / 拆分"
    width="560px"
    append-to-body
    destroy-on-close
    @closed="handleClosed">
    <el-tabs v-model="activeTab">
      <el-tab-pane label="合并" name="merge">
        <div class="form-section">
          <div class="form-label">主项目</div>
          <el-select
            v-model="primaryProject"
            placeholder="选择主项目"
            style="width: 100%">
            <el-option
              v-for="p in projects"
              :key="p.encoded_name"
              :label="p.name"
              :value="p.encoded_name" />
          </el-select>
        </div>
        <div class="form-section">
          <div class="form-label">次要项目</div>
          <el-select
            v-model="secondaryProjects"
            placeholder="选择要合并的项目"
            multiple
            style="width: 100%">
            <el-option
              v-for="p in availableSecondaries"
              :key="p.encoded_name"
              :label="p.name"
              :value="p.encoded_name" />
          </el-select>
        </div>
        <div class="footer-actions">
          <el-button
            type="primary"
            :disabled="!primaryProject || secondaryProjects.length === 0"
            :loading="merging"
            @click="handleMerge">
            合并
          </el-button>
        </div>
      </el-tab-pane>

      <el-tab-pane label="拆分" name="split">
        <div class="form-section">
          <div class="form-label">已合并项目</div>
          <el-select
            v-model="splitTarget"
            placeholder="选择要拆分的项目"
            style="width: 100%">
            <el-option
              v-for="p in mergedProjectList"
              :key="p.encoded_name"
              :label="p.name"
              :value="p.encoded_name" />
          </el-select>
        </div>
        <div class="footer-actions">
          <el-button
            type="primary"
            :disabled="!splitTarget"
            :loading="splitting"
            @click="handleSplit(splitTarget!)">
            拆分
          </el-button>
        </div>
      </el-tab-pane>
    </el-tabs>

    <div v-if="Object.keys(merges).length > 0" class="merges-section">
      <div class="merges-header">当前合并</div>
      <div
        v-for="(secondaries, primary) in merges"
        :key="primary"
        class="merge-item">
        <div class="merge-info">
          <span class="merge-primary">{{ getProjectName(primary) }}</span>
          <span class="merge-arrow">←</span>
          <span class="merge-secondary">{{ secondaries.map(getProjectName).join(', ') }}</span>
        </div>
        <el-button
          size="small"
          type="danger"
          text
          :loading="splitting"
          @click="handleSplit(primary)">
          拆分
        </el-button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import AppDialog from '@/components/common/AppDialog.vue'

interface ProjectInfo {
  name: string
  encoded_name: string
  path: string
}

const props = defineProps<{
  visible: boolean
  projects: ProjectInfo[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'merged'): void
  (e: 'split'): void
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const activeTab = ref('merge')
const primaryProject = ref('')
const secondaryProjects = ref<string[]>([])
const splitTarget = ref('')
const merging = ref(false)
const splitting = ref(false)
const merges = ref<Record<string, string[]>>({})

const availableSecondaries = computed(() =>
  props.projects.filter((p) => p.encoded_name !== primaryProject.value),
)

const mergedProjectList = computed(() =>
  props.projects.filter((p) => Object.keys(merges.value).includes(p.encoded_name)),
)

const getProjectName = (encodedName: string): string => {
  const p = props.projects.find((item) => item.encoded_name === encodedName)
  return p ? p.name : encodedName
}

const loadMerges = async () => {
  try {
    merges.value = await invoke<Record<string, string[]>>('agent_get_project_merges')
  } catch {
    merges.value = {}
  }
}

const handleMerge = async () => {
  try {
    await ElMessageBox.confirm(
      `将 ${secondaryProjects.value.map(getProjectName).join('、')} 合并到 ${getProjectName(primaryProject.value)}？`,
      '确认合并',
      { type: 'warning' },
    )
  } catch {
    return
  }

  merging.value = true
  try {
    await invoke('agent_merge_projects', {
      primary: primaryProject.value,
      secondaries: secondaryProjects.value,
    })
    ElMessage.success('合并成功')
    primaryProject.value = ''
    secondaryProjects.value = []
    await loadMerges()
    emit('merged')
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : '合并失败')
  } finally {
    merging.value = false
  }
}

const handleSplit = async (primary: string) => {
  try {
    await ElMessageBox.confirm(
      `将 ${getProjectName(primary)} 拆分还原？`,
      '确认拆分',
      { type: 'warning' },
    )
  } catch {
    return
  }

  splitting.value = true
  try {
    await invoke('agent_split_project', { primary })
    ElMessage.success('拆分成功')
    splitTarget.value = ''
    await loadMerges()
    emit('split')
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : '拆分失败')
  } finally {
    splitting.value = false
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) loadMerges()
  },
)

const handleClosed = () => {
  activeTab.value = 'merge'
  primaryProject.value = ''
  secondaryProjects.value = []
  splitTarget.value = ''
  merging.value = false
  splitting.value = false
  merges.value = {}
}
</script>

<style scoped>
.form-section {
  margin-bottom: 16px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 6px;
}

.footer-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

.merges-section {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.merges-header {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 8px;
}

.merge-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  margin-bottom: 6px;
}

.merge-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  min-width: 0;
  overflow: hidden;
}

.merge-primary {
  font-weight: 500;
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.merge-arrow {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.merge-secondary {
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
