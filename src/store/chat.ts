import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  streaming?: boolean
  error?: string
}

export interface Conversation {
  id: string
  title: string
  messages: ChatMessage[]
  modelId: string
  createdAt: number
  updatedAt: number
}

export const useChatStore = defineStore('chat', () => {
  const conversations = ref<Conversation[]>([])
  const activeConversationId = ref<string | null>(null)
  const isStreaming = ref(false)

  const activeConversation = computed(() =>
    conversations.value.find((c) => c.id === activeConversationId.value) || null
  )

  const sortedConversations = computed(() =>
    [...conversations.value].sort((a, b) => b.updatedAt - a.updatedAt)
  )

  function createConversation(modelId?: string): Conversation {
    const conv: Conversation = {
      id: crypto.randomUUID(),
      title: '新对话',
      messages: [],
      modelId: modelId || '',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }
    conversations.value.unshift(conv)
    activeConversationId.value = conv.id
    return conv
  }

  function selectConversation(id: string) {
    activeConversationId.value = id
  }

  function deleteConversation(id: string) {
    const index = conversations.value.findIndex((c) => c.id === id)
    if (index === -1) return
    conversations.value.splice(index, 1)
    if (activeConversationId.value === id) {
      activeConversationId.value = conversations.value[0]?.id || null
    }
  }

  function addMessage(convId: string, msg: ChatMessage) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    conv.messages.push(msg)
    conv.updatedAt = Date.now()
    // 自动用第一条用户消息作为标题
    if (conv.messages.filter((m) => m.role === 'user').length === 1 && msg.role === 'user') {
      conv.title = msg.content.slice(0, 30) + (msg.content.length > 30 ? '...' : '')
    }
  }

  function updateMessageContent(convId: string, msgId: string, content: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.content = content
    conv.updatedAt = Date.now()
  }

  function setMessageStreaming(convId: string, msgId: string, streaming: boolean) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.streaming = streaming
  }

  function setMessageError(convId: string, msgId: string, error: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.error = error
    msg.streaming = false
  }

  function setTitle(convId: string, title: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (conv) {
      conv.title = title
    }
  }

  return {
    conversations,
    activeConversationId,
    activeConversation,
    sortedConversations,
    isStreaming,
    createConversation,
    selectConversation,
    deleteConversation,
    addMessage,
    updateMessageContent,
    setMessageStreaming,
    setMessageError,
    setTitle,
  }
})
