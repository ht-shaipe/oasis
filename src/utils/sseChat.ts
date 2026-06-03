/**
 * 流式聊天工具
 * 优先通过 Tauri Channel 调用后端 ai_chat_stream，实现真正的 SSE 流式传输
 */

import { invoke, Channel } from '@tauri-apps/api/core'

export interface StreamChunk {
  content: string
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
  onComplete: (fullContent: string) => void
  onError: (error: string) => void
  onUsage?: (usage: StreamChunk['usage']) => void
}

/**
 * 通过 Tauri Channel 发起流式聊天
 * 后端 ai_chat_stream 命令逐块推送 token
 */
export async function streamChat(
  request: ChatRequest,
  options: StreamOptions,
): Promise<void> {
  const { onToken, onComplete, onError, onUsage } = options

  let fullContent = ''
  let streamEnded = false

  const channel = new Channel<StreamChunk>()

  channel.onmessage = (chunk: StreamChunk) => {
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

    // 如果 invoke 正常返回但流没有正常结束标记（防止极端情况）
    if (!streamEnded) {
      onComplete(fullContent)
    }
  } catch (error: any) {
    onError(error?.toString() || '请求失败')
  }
}
