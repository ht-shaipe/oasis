import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  reasoningContent?: string
  timestamp: number
  streaming?: boolean
  error?: string
  tokenUsage?: { promptTokens: number; completionTokens: number; totalTokens: number }
}

export interface Conversation {
  id: string
  title: string
  messages: ChatMessage[]
  modelId: string
  createdAt: number
  updatedAt: number
  messageCount?: number
}

interface BackendConversation {
  id: string
  title: string
  model_id: string
  message_count: number
  created_at: string
  updated_at: string
}

interface BackendConversationDetail {
  conversation: {
    id: string
    title: string
    model_id: string
    created_at: string
    updated_at: string
  }
  messages: Array<{
    id: string
    conversation_id: string
    role: string
    content: string
    reasoning_content?: string
    token_usage?: string
    created_at: string
  }>
}

function parseIsoToTs(iso: string): number {
  return new Date(iso).getTime() || Date.now()
}

function parseTokenUsage(raw?: string): ChatMessage['tokenUsage'] {
  if (!raw) return undefined
  try {
    const parsed = JSON.parse(raw)
    return {
      promptTokens: parsed.promptTokens ?? parsed.prompt_tokens ?? 0,
      completionTokens: parsed.completionTokens ?? parsed.completion_tokens ?? 0,
      totalTokens: parsed.totalTokens ?? parsed.total_tokens ?? 0,
    }
  } catch {
    return undefined
  }
}

function backendDetailToConversation(detail: BackendConversationDetail): Conversation {
  const conv = detail.conversation
  return {
    id: conv.id,
    title: conv.title,
    modelId: conv.model_id,
    createdAt: parseIsoToTs(conv.created_at),
    updatedAt: parseIsoToTs(conv.updated_at),
    messages: detail.messages.map((m) => ({
      id: m.id,
      role: m.role as 'user' | 'assistant',
      content: m.content,
      reasoningContent: m.reasoning_content || undefined,
      tokenUsage: parseTokenUsage(m.token_usage),
      timestamp: parseIsoToTs(m.created_at),
    })),
  }
}

export const useChatStore = defineStore('chat', () => {
  const conversations = ref<Conversation[]>([])
  const activeConversationId = ref<string | null>(null)
  const isStreaming = ref(false)
  const isLoaded = ref(false)

  const activeConversation = computed(() =>
    conversations.value.find((c) => c.id === activeConversationId.value) || null
  )

  const sortedConversations = computed(() =>
    [...conversations.value].sort((a, b) => b.updatedAt - a.updatedAt)
  )

  async function loadConversations() {
    if (isLoaded.value) return
    try {
      const list = await invoke<BackendConversation[]>('list_conversations')
      conversations.value = list.map((c) => ({
        id: c.id,
        title: c.title,
        modelId: c.model_id,
        createdAt: parseIsoToTs(c.created_at),
        updatedAt: parseIsoToTs(c.updated_at),
        messages: [],
        messageCount: c.message_count,
      }))
      isLoaded.value = true
    } catch (e) {
      console.error('Failed to load conversations:', e)
    }
  }

  async function loadConversationMessages(id: string) {
    const conv = conversations.value.find((c) => c.id === id)
    if (!conv || conv.messages.length > 0) return
    try {
      const detail = await invoke<BackendConversationDetail>('get_conversation', { id })
      const parsed = backendDetailToConversation(detail)
      conv.messages = parsed.messages
    } catch (e) {
      console.error('Failed to load conversation messages:', e)
    }
  }

  async function createConversation(modelId?: string): Promise<Conversation> {
    const id = crypto.randomUUID()
    const title = '新对话'
    const model_id = modelId || ''

    try {
      await invoke('create_conversation', {
        conversation: { id, title, model_id },
      })
    } catch (e) {
      console.error('Failed to create conversation in backend:', e)
    }

    const conv: Conversation = {
      id,
      title,
      messages: [],
      modelId: model_id,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }
    conversations.value.unshift(conv)
    activeConversationId.value = conv.id
    return conv
  }

  async function selectConversation(id: string) {
    activeConversationId.value = id
    await loadConversationMessages(id)
  }

  async function deleteConversation(id: string) {
    const index = conversations.value.findIndex((c) => c.id === id)
    if (index === -1) return
    conversations.value.splice(index, 1)
    if (activeConversationId.value === id) {
      activeConversationId.value = conversations.value[0]?.id || null
    }
    try {
      await invoke('delete_conversation', { id })
    } catch (e) {
      console.error('Failed to delete conversation:', e)
    }
  }

  async function addMessage(convId: string, msg: ChatMessage) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    conv.messages.push(msg)
    conv.updatedAt = Date.now()

    if (conv.messages.filter((m) => m.role === 'user').length === 1 && msg.role === 'user') {
      conv.title = msg.content.slice(0, 30) + (msg.content.length > 30 ? '...' : '')
      try {
        await invoke('update_conversation_title', { id: convId, title: conv.title })
      } catch (e) {
        console.error('Failed to update title:', e)
      }
    }

    if (!msg.streaming) {
      try {
        await invoke('save_message', {
          message: {
            id: msg.id,
            conversation_id: convId,
            role: msg.role,
            content: msg.content,
            reasoning_content: msg.reasoningContent || null,
            token_usage: msg.tokenUsage ? JSON.stringify(msg.tokenUsage) : null,
          },
        })
      } catch (e) {
        console.error('Failed to save message:', e)
      }
    }
  }

  async function updateMessageContent(convId: string, msgId: string, content: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.content = content
    conv.updatedAt = Date.now()
  }

  async function updateMessageReasoning(convId: string, msgId: string, reasoning: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.reasoningContent = (msg.reasoningContent || '') + reasoning
  }

  async function setMessageStreaming(convId: string, msgId: string, streaming: boolean) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.streaming = streaming

    if (!streaming) {
      try {
        await invoke('save_message', {
          message: {
            id: msg.id,
            conversation_id: convId,
            role: msg.role,
            content: msg.content,
            reasoning_content: msg.reasoningContent || null,
            token_usage: msg.tokenUsage ? JSON.stringify(msg.tokenUsage) : null,
          },
        })
      } catch (e) {
        console.error('Failed to save completed message:', e)
      }
    }
  }

  function setMessageError(convId: string, msgId: string, error: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (!conv) return
    const msg = conv.messages.find((m) => m.id === msgId)
    if (!msg) return
    msg.error = error
    msg.streaming = false
  }

  async function setTitle(convId: string, title: string) {
    const conv = conversations.value.find((c) => c.id === convId)
    if (conv) {
      conv.title = title
      try {
        await invoke('update_conversation_title', { id: convId, title })
      } catch (e) {
        console.error('Failed to update title:', e)
      }
    }
  }

  return {
    conversations,
    activeConversationId,
    activeConversation,
    sortedConversations,
    isStreaming,
    isLoaded,
    loadConversations,
    loadConversationMessages,
    createConversation,
    selectConversation,
    deleteConversation,
    addMessage,
    updateMessageContent,
    updateMessageReasoning,
    setMessageStreaming,
    setMessageError,
    setTitle,
  }
})
