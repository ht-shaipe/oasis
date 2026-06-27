import { invoke, Channel } from '@tauri-apps/api/core'

export interface StreamChunk {
  content: string
  reasoningContent?: string
  isOver: boolean
  usage?: {
    promptTokens: number
    completionTokens: number
    totalTokens: number
  }
}

export interface ChatRequest {
  model_id: string
  messages: Array<{ role: string; content: string }>
  temperature?: number
  max_tokens?: number
}

export interface StreamOptions {
  onToken: (token: string) => void
  onReasoning: (token: string) => void
  onComplete: (fullContent: string) => void
  onError: (error: string) => void
  onUsage?: (usage: StreamChunk['usage']) => void
}

export async function streamChat(
  request: ChatRequest,
  options: StreamOptions,
): Promise<void> {
  const { onToken, onReasoning, onComplete, onError, onUsage } = options

  let fullContent = ''
  let streamEnded = false

  const channel = new Channel<StreamChunk>()

  channel.onmessage = (chunk: StreamChunk) => {
    if (chunk.reasoningContent) {
      onReasoning(chunk.reasoningContent)
    }
    if (chunk.content) {
      fullContent += chunk.content
      onToken(chunk.content)
    }

    if (chunk.isOver) {
      streamEnded = true
      if (chunk.usage) {
        onUsage?.(chunk.usage)
      }
      onComplete(fullContent)
    }
  }

  try {
    await invoke('ai_chat_stream', {
      request: {
        model_id: request.model_id,
        messages: request.messages,
        temperature: request.temperature ?? null,
        max_tokens: request.max_tokens ?? null,
      },
      channel,
    })

    if (!streamEnded) {
      onComplete(fullContent)
    }
  } catch (error: any) {
    onError(error?.toString() || '请求失败')
  }
}
