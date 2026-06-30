<template>
  <div class="search-panel">
    <div class="search-header">
      <h4>{{ t('knowledge.semanticSearch') }}</h4>
      <span v-if="status && status.embeddedChunks > 0" class="search-index-hint">
        {{ status.embeddedChunks }} / {{ status.totalChunks }} {{ t('knowledge.embeddedChunks') }}
      </span>
    </div>

    <div v-if="!status || status.embeddedChunks === 0" class="no-index-hint">
      <span>{{ t('knowledge.noIndexHint') }}</span>
      <el-button size="small" type="primary" link @click="$emit('goToIndex')">
        {{ t('knowledge.goToIndex') }}
      </el-button>
    </div>

    <template v-else>
      <el-input
        v-model="searchQuery"
        :placeholder="t('knowledge.searchPlaceholder')"
        size="small"
        clearable
        @keyup.enter="handleSearch"
      >
        <template #append>
          <el-button :loading="isSearching" @click="handleSearch">
            <el-icon><Search /></el-icon>
          </el-button>
        </template>
      </el-input>

      <div v-if="searchResults.length > 0" class="search-results">
        <el-scrollbar max-height="400px">
          <div
            v-for="result in searchResults"
            :key="result.chunkIndex + result.relPath"
            class="search-result-item"
          >
            <div class="result-header">
              <span class="result-file">{{ result.relPath }}</span>
              <span class="result-score">{{ (result.score * 100).toFixed(1) }}%</span>
            </div>
            <div class="result-content">{{ result.chunkContent }}</div>
          </div>
        </el-scrollbar>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { useKnowledge } from '../composables/useKnowledge'

const { t } = useI18n()

const emit = defineEmits<{
  goToIndex: []
}>()

interface SearchResult {
  filePath: string
  relPath: string
  chunkContent: string
  chunkIndex: number
  score: number
}

const {
  status,
  ensureInit,
  cleanup,
} = useKnowledge()

const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const isSearching = ref(false)

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  isSearching.value = true
  try {
    searchResults.value = await invoke<SearchResult[]>('semantic_search', {
      query: searchQuery.value,
      topK: 5,
    })
    if (searchResults.value.length === 0) {
      ElMessage.info(t('knowledge.noResults'))
    }
  } catch (e: unknown) {
    ElMessage.error(`${e}`)
  } finally {
    isSearching.value = false
  }
}

onMounted(() => ensureInit())
onUnmounted(() => cleanup())
</script>

<style scoped>
.search-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.search-header h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.search-index-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.no-index-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding: 8px 10px;
  background: rgba(230, 162, 60, 0.06);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.search-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.search-result-item {
  padding: 8px;
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  cursor: default;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.result-file {
  font-size: 12px;
  font-weight: 500;
  color: #007bff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.result-score {
  font-size: 11px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.result-content {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  max-height: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
</style>
