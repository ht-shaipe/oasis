<template>
  <div class="chat-view">
    <ConversationList
      :conversations="store.sortedConversations"
      :activeId="store.activeConversationId"
      @select="store.selectConversation"
      @delete="handleDeleteConversation"
      @new="handleNewConversation"
      
    />

    <div class="chat-main">
      <!-- 空状态 -->
      <div v-if="!store.activeConversation" class="chat-empty">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
        </div>
        <h3>{{ t('chat.emptyHint') }}</h3>
        <p>{{ t('chat.emptyDesc') }}</p>
        <el-button type="primary" @click="handleNewConversation">{{ t('chat.newChat') }}</el-button>
      </div>

      <!-- 聊天区 -->
      <template v-else>
        <div class="chat-header">
          <span class="chat-title">{{ store.activeConversation.title }}</span>
          <div class="chat-header-actions">
            <el-tooltip :content="ragEnabled ? t('chat.ragEnabled') : t('chat.ragDisabled')" placement="bottom">
              <el-button
                :type="ragEnabled ? 'primary' : 'default'"
                size="small"
                :icon="Reading"
                @click="ragEnabled = !ragEnabled"
                plain
              />
            </el-tooltip>
            <el-select
              v-model="currentModelId"
              size="small"
              :placeholder="t('chat.selectModel')"
              style="width: 220px"
              @change="onModelChange"
            >
              <el-option-group
                v-for="group in modelGroups"
                :key="group.provider"
                :label="group.label"
              >
                <el-option
                  v-for="m in group.models"
                  :key="m.id"
                  :label="m.name"
                  :value="m.id"
                >
                  <span class="model-option-name">{{ m.name }}</span>
                  <span class="model-option-id">{{ m.model_id }}</span>
                </el-option>
              </el-option-group>
            </el-select>
          </div>
        </div>

        <el-scrollbar class="chat-messages" ref="messagesScrollbarRef">
          <ChatMessage
              v-for="msg in store.activeConversation.messages"
              :key="msg.id"
              :role="msg.role"
              :content="msg.content"
              :reasoningContent="msg.reasoningContent"
              :streaming="msg.streaming"
              :error="msg.error"
            />
            <div v-if="store.isStreaming && !streamingMessage" class="chat-loading">
              <span class="loading-dot">{{ t('chat.thinking') }}</span>
            </div>
        </el-scrollbar>

        <ChatInput
          ref="inputRef"
          :disabled="store.isStreaming"
          placeholder="输入消息..."
          @send="handleSend"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Reading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useChatStore } from '@/store/chat'
import { streamChat } from '@/utils/sseChat'
import ConversationList from './components/ConversationList.vue'
import ChatMessage from './components/ChatMessage.vue'
import ChatInput from './components/ChatInput.vue'

const { t } = useI18n()
const store = useChatStore()

const messagesScrollbarRef = ref<InstanceType<typeof import('element-plus')['ElScrollbar']> | null>(null)
const inputRef = ref<InstanceType<typeof ChatInput> | null>(null)
const currentModelId = ref('')
const ragEnabled = ref(false)

interface LLMModel {
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

const models = ref<LLMModel[]>([])

const enabledModels = computed(() => models.value.filter((m) => m.enabled && m.model_type !== 'embedding'))

const providerLabels: Record<string, string> = {
  deepseek: 'DeepSeek',
  chatgpt: 'ChatGPT',
  ollama: 'Ollama',
  kimi: 'Kimi',
  hunyuan: '腾讯混元',
  doubao: '豆包',
  mimo: 'MiMo',
  qwen: '阿里千问',
  zhipu: '智谱',
  wenxin: '文心一言',
  xunfei: '讯飞',
}

const modelGroups = computed(() => {
  const groups: Record<string, { provider: string; label: string; models: LLMModel[] }> = {}
  for (const m of enabledModels.value) {
    const key = m.provider || 'other'
    if (!groups[key]) {
      groups[key] = {
        provider: key,
        label: providerLabels[key] || key,
        models: [],
      }
    }
    groups[key].models.push(m)
  }
  return Object.values(groups)
})

onMounted(async () => {
  await store.loadConversations()
  if (store.sortedConversations.length > 0 && !store.activeConversationId) {
    store.selectConversation(store.sortedConversations[0].id)
  }
  try {
    models.value = await invoke<LLMModel[]>('get_llm_models')
  } catch (e) {
    console.error('加载 LLM 模型列表失败:', e)
  }
  syncCurrentModelId()
})

function syncCurrentModelId() {
  const conv = store.activeConversation
  if (conv && conv.modelId) {
    currentModelId.value = conv.modelId
  } else {
    currentModelId.value = enabledModels.value[0]?.id || ''
  }
}

watch(() => store.activeConversationId, () => {
  syncCurrentModelId()
})

const streamingMessage = computed(() =>
  store.activeConversation?.messages.find((m) => m.streaming)
)

function scrollToBottom() {
  nextTick(() => {
    const scrollbar = messagesScrollbarRef.value
    if (scrollbar) {
      scrollbar.setScrollTop(scrollbar.wrapRef?.scrollHeight ?? 99999)
    }
  })
}

watch(
  () => store.activeConversation?.messages.length,
  () => scrollToBottom()
)

watch(
  () => store.activeConversation?.messages.slice(-1)?.[0]?.content,
  () => scrollToBottom()
)

async function handleNewConversation() {
  const modelId = currentModelId.value || enabledModels.value[0]?.id || ''
  await store.createConversation(modelId)
  currentModelId.value = modelId
  nextTick(() => inputRef.value?.focus())
}

async function handleDeleteConversation(id: string) {
  try {
    await ElMessageBox.confirm(t('chat.deleteConfirm'), t('chat.deleteChat'), { type: 'warning' })
    await store.deleteConversation(id)
  } catch {
    // 取消
  }
}

function onModelChange() {
  if (store.activeConversation) {
    store.activeConversation.modelId = currentModelId.value
  }
}

async function handleSend(text: string) {
  const conv = store.activeConversation
  if (!conv) {
    await handleNewConversation()
    return handleSend(text)
  }

  if (!currentModelId.value) {
    ElMessage.warning(t('chat.modelRequired'))
    return
  }

  const model = enabledModels.value.find((m) => m.id === currentModelId.value)
  if (!model) {
    ElMessage.warning(t('chat.modelUnavailable'))
    return
  }

  const aiMsgId = crypto.randomUUID()

  await store.addMessage(conv.id, {
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
    timestamp: Date.now(),
  })

  await store.addMessage(conv.id, {
    id: aiMsgId,
    role: 'assistant',
    content: '',
    timestamp: Date.now(),
    streaming: true,
  })

  store.isStreaming = true
  scrollToBottom()

  let history: Array<{ role: string; content: string }> = conv.messages
    .filter((m) => m.id !== aiMsgId)
    .map((m) => ({ role: m.role, content: m.content }))

  if (ragEnabled.value) {
    try {
      const results = await invoke<Array<{
        filePath: string
        relPath: string
        chunkContent: string
        chunkIndex: number
        score: number
      }>>('semantic_search', { query: text, topK: 5 })
      if (results.length > 0) {
        const contextParts = results
          .map((r) => `[${r.relPath}]\n${r.chunkContent}`)
          .join('\n\n')
        const systemContent = `${t('chat.ragSystemPrompt')}\n\n${contextParts}`
        history.unshift({ role: 'system' as const, content: systemContent })
      }
    } catch (e) {
      console.error('RAG search failed:', e)
    }
  }

  await streamChat(
    {
      model_id: model.model_id,
      messages: history,
    },
    {
      onToken(token: string) {
        const current = conv.messages.find((m) => m.id === aiMsgId)
        if (current) {
          store.updateMessageContent(conv.id, aiMsgId, current.content + token)
        }
      },
      onReasoning(token: string) {
        store.updateMessageReasoning(conv.id, aiMsgId, token)
      },
      onComplete() {
        store.setMessageStreaming(conv.id, aiMsgId, false)
        store.isStreaming = false
      },
      onError(error: string) {
        store.setMessageError(conv.id, aiMsgId, error)
        store.isStreaming = false
      },
    }
  )
}
</script>

<style scoped>
.chat-view {
  display: flex;
  height: 100%;
  background: var(--color-bg-glass);
}

.chat-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  border-bottom: 1px solid var(--color-card-border);
  flex-shrink: 0;
}

.chat-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chat-title {
  font-size: var(--app-font-13);
  font-weight: 600;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-messages {
  flex: 1;
}

.chat-messages :deep(.el-scrollbar__wrap) {
  padding: 8px 0;
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-tertiary);
}

.chat-empty h3 {
  font-size: var(--app-font-16);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin: 0;
}

.chat-empty p {
  font-size: var(--app-font-13);
  margin: 0;
}

.empty-icon {
  margin-bottom: 8px;
}

.chat-loading {
  padding: 8px 20px;
}

.loading-dot {
  font-size: var(--app-font-13);
  color: var(--color-text-tertiary);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.model-option-name {
  font-size: var(--app-font-13);
  color: var(--color-text-primary);
}

.model-option-id {
  font-size: var(--app-font-12);
  color: var(--color-text-tertiary);
  margin-left: 8px;
}
</style>
