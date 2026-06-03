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
        <h3>开始新对话</h3>
        <p>选择一个已配置的 LLM 模型后开始聊天</p>
        <el-button type="primary" @click="handleNewConversation">新建对话</el-button>
      </div>

      <!-- 聊天区 -->
      <template v-else>
        <div class="chat-header">
          <span class="chat-title">{{ store.activeConversation.title }}</span>
          <el-select
            v-model="currentModelId"
            size="small"
            placeholder="选择模型"
            style="width: 200px"
            @change="onModelChange"
          >
            <el-option
              v-for="m in enabledModels"
              :key="m.id"
              :label="m.name"
              :value="m.id"
            />
          </el-select>
        </div>

        <div class="chat-messages" ref="messagesRef">
          <ChatMessage
            v-for="msg in store.activeConversation.messages"
            :key="msg.id"
            :role="msg.role"
            :content="msg.content"
            :streaming="msg.streaming"
            :error="msg.error"
          />
          <div v-if="store.isStreaming && !streamingMessage" class="chat-loading">
            <span class="loading-dot">思考中</span>
          </div>
        </div>

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
import { invoke } from '@tauri-apps/api/core'
import { useChatStore } from '@/store/chat'
import { streamChat } from '@/utils/sseChat'
import ConversationList from './components/ConversationList.vue'
import ChatMessage from './components/ChatMessage.vue'
import ChatInput from './components/ChatInput.vue'

const store = useChatStore()

const messagesRef = ref<HTMLElement | null>(null)
const inputRef = ref<InstanceType<typeof ChatInput> | null>(null)
const currentModelId = ref('')

// 从 Rust 后端加载的模型列表
interface LLMModel {
  id: string
  name: string
  provider: string
  model_id: string
  base_url: string
  api_key: string
  auth_type: string
  enabled: boolean
}

const models = ref<LLMModel[]>([])

const enabledModels = computed(() => models.value.filter((m) => m.enabled))

onMounted(async () => {
  try {
    models.value = await invoke<LLMModel[]>('get_llm_models')
  } catch (e) {
    console.error('加载 LLM 模型列表失败:', e)
  }
})

// 当前正在流式输出的 AI 消息
const streamingMessage = computed(() =>
  store.activeConversation?.messages.find((m) => m.streaming)
)

// 滚动到底部
function scrollToBottom() {
  nextTick(() => {
    if (messagesRef.value) {
      messagesRef.value.scrollTop = messagesRef.value.scrollHeight
    }
  })
}

// 监听消息变化自动滚动
watch(
  () => store.activeConversation?.messages.length,
  () => scrollToBottom()
)

watch(
  () => store.activeConversation?.messages.slice(-1)?.[0]?.content,
  () => scrollToBottom()
)

function handleNewConversation() {
  store.createConversation()
  currentModelId.value = enabledModels.value[0]?.id || ''
  nextTick(() => inputRef.value?.focus())
}

async function handleDeleteConversation(id: string) {
  try {
    await ElMessageBox.confirm('确定要删除此对话吗？', '删除对话', { type: 'warning' })
    store.deleteConversation(id)
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
    handleNewConversation()
    // 等待新对话创建后再发送
    await nextTick()
    return handleSend(text)
  }

  if (!currentModelId.value) {
    ElMessage.warning('请先选择一个模型')
    return
  }

  const model = enabledModels.value.find((m) => m.id === currentModelId.value)
  if (!model) {
    ElMessage.warning('所选模型不可用')
    return
  }

  // 添加用户消息
  store.addMessage(conv.id, {
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
    timestamp: Date.now(),
  })

  // 创建 AI 消息占位
  const aiMsgId = crypto.randomUUID()
  store.addMessage(conv.id, {
    id: aiMsgId,
    role: 'assistant',
    content: '',
    timestamp: Date.now(),
    streaming: true,
  })

  store.isStreaming = true
  scrollToBottom()

  // 构建消息历史
  const history = conv.messages
    .filter((m) => m.role !== 'system' && m.id !== aiMsgId)
    .map((m) => ({ role: m.role, content: m.content }))

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
  overflow-y: auto;
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
</style>
