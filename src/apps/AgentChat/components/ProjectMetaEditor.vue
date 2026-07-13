<template>
  <div class="project-meta-editor">
    <div class="meta-field">
      <label>名称</label>
      <el-input
        v-model="form.customName"
        size="small"
        :placeholder="projectPath.split('/').pop() || projectPath"
        @blur="scheduleSave"
      />
    </div>

    <div class="meta-field">
      <label>标签</label>
      <div class="tags-row">
        <el-tag
          v-for="tag in form.tags"
          :key="tag"
          size="small"
          closable
          @close="removeTag(tag)"
        >
          {{ tag }}
        </el-tag>
        <el-input
          v-if="showTagInput"
          ref="tagInputRef"
          v-model="newTag"
          size="small"
          class="tag-input"
          @keyup.enter="addTag"
          @blur="addTag"
        />
        <el-button v-else size="small" text type="primary" @click="openTagInput">
          +
        </el-button>
      </div>
    </div>

    <div class="meta-field">
      <label>备注</label>
      <el-input
        v-model="form.notes"
        type="textarea"
        size="small"
        :rows="3"
        resize="none"
        @blur="scheduleSave"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ref, reactive, watch, onMounted, nextTick } from 'vue'

const props = defineProps<{
  projectPath: string
  encodedName: string
}>()

interface ProjectMetaMap {
  [key: string]: { tags?: string[] | null; notes?: string | null; custom_name?: string | null }
}

const form = reactive({
  customName: '',
  tags: [] as string[],
  notes: '',
})

const showTagInput = ref(false)
const newTag = ref('')
const tagInputRef = ref<InstanceType<typeof import('element-plus')['ElInput']>>()

let saveTimer: ReturnType<typeof setTimeout> | null = null

function scheduleSave() {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(save, 300)
}

async function save() {
  try {
    await invoke('agent_save_project_meta', {
      encodedName: props.encodedName,
      meta: {
        custom_name: form.customName || null,
        tags: form.tags.length ? form.tags : null,
        notes: form.notes || null,
      },
    })
  } catch (e) {
    console.error('save project meta failed', e)
  }
}

async function load() {
  try {
    const metas = await invoke<ProjectMetaMap>('agent_load_project_metas')
    const meta = metas[props.encodedName]
    if (meta) {
      form.customName = meta.custom_name ?? ''
      form.tags = meta.tags ?? []
      form.notes = meta.notes ?? ''
    }
  } catch (e) {
    console.error('load project metas failed', e)
  }
}

function removeTag(tag: string) {
  form.tags = form.tags.filter(t => t !== tag)
  scheduleSave()
}

function addTag() {
  const trimmed = newTag.value.trim()
  if (trimmed && !form.tags.includes(trimmed)) {
    form.tags.push(trimmed)
    scheduleSave()
  }
  newTag.value = ''
  showTagInput.value = false
}

function openTagInput() {
  showTagInput.value = true
  nextTick(() => {
    tagInputRef.value?.focus()
  })
}

watch(() => props.encodedName, load)

onMounted(load)
</script>

<style scoped>
.project-meta-editor {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 10px 12px;
  width: 260px;
}

.meta-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-field label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-tertiary);
}

.tags-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.tag-input {
  width: 80px;
}

.project-meta-editor :deep(.el-input__inner),
.project-meta-editor :deep(.el-textarea__inner) {
  font-size: 13px;
}

.project-meta-editor :deep(.el-tag) {
  font-size: 13px;
}
</style>
