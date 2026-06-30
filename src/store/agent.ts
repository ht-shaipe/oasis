import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// ── Types ─────────────────────────────────────────────────────────

export interface AgentInfo {
  id: string
  display_name: string
  version: string
  icon: string
  enabled: boolean
}

export interface AgentStatus {
  id: string
  display_name: string
  icon: string
  installed: boolean
  version: string | null
  error: string | null
  install_hint: string | null
  native_install_command: string | null
}

export interface ProjectEntry {
  name: string
  path: string
  encoded_name: string
  session_count: number
  last_active: string | null
  agent_id: string
  initialized: boolean
}

export interface SessionInfo {
  id: string
  display_name: string | null
  started_at: string | null
  last_active: string | null
  message_count: number
}

export interface SessionMessage {
  role: string
  content: string
  timestamp: number | null
}

export interface ChatResult {
  agent_id: string
  session_id: string
  process_id: number
}

// ── Stream types ──────────────────────────────────────────────────

export type ContentBlock =
  | { type: 'text'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: unknown }
  | { type: 'tool_result'; tool_use_id: string; content: unknown; is_error: boolean }
  | { type: 'thinking'; thinking: string }

export interface AgentStreamChunk {
  agent_id: string
  session_id: string
  event_type: string
  data: {
    kind: string
    delta?: string
    call_id?: string
    tool?: string
    input?: unknown
    output?: unknown
    is_error?: boolean
    session_id?: string
    reason?: string
    message?: string
    recoverable?: boolean
    usage?: { input_tokens?: number; output_tokens?: number; total_cost?: number }
    content?: ContentBlock[]
  }
}

export interface ToolCall {
  id: string
  name: string
  input: unknown
  output?: unknown
  isError?: boolean
}

export interface AgentMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  thinking: string
  timestamp: number
  streaming: boolean
  error: string
  tools: ToolCall[]
  contentBlocks: ContentBlock[]
}

export interface AgentSession {
  id: string
  projectPath: string
  messages: AgentMessage[]
  isStreaming: boolean
}

// ── Store ─────────────────────────────────────────────────────────

export const useAgentStore = defineStore('agent', () => {
  // Agent registry
  const agents = ref<AgentInfo[]>([])
  const agentStatuses = ref<AgentStatus[]>([])
  const activeAgentId = ref<string>('claude-code')

  // Projects
  const projects = ref<ProjectEntry[]>([])
  const activeProjectPath = ref<string | null>(null)

  // Sessions & messages
  const sessions = ref<SessionInfo[]>([])
  const activeSessionId = ref<string | null>(null)
  const sessionMessages = ref<SessionMessage[]>([])

  // Active chat sessions (streaming)
  const activeSessions = ref<Map<string, AgentSession>>(new Map())

  // Event listener
  let unlisten: UnlistenFn | null = null

  // ── Computed ──────────────────────────────────────────────────

  const activeAgent = computed(() =>
    agents.value.find(a => a.id === activeAgentId.value) || null
  )

  const activeProject = computed(() =>
    projects.value.find(p => p.path === activeProjectPath.value) || null
  )

  const installedAgents = computed(() =>
    agentStatuses.value.filter(a => a.installed)
  )

  const activeAgentStatus = computed(() =>
    agentStatuses.value.find(a => a.id === activeAgentId.value) || null
  )

  const activeChatSession = computed(() => {
    if (!activeSessionId.value) return null
    return activeSessions.value.get(activeSessionId.value) || null
  })

  // ── Actions: agents ───────────────────────────────────────────

  async function loadAgents() {
    try {
      agents.value = await invoke<AgentInfo[]>('agent_list')
    } catch (e) {
      console.error('Failed to load agents:', e)
    }
  }

  async function loadAgentStatuses() {
    try {
      agentStatuses.value = await invoke<AgentStatus[]>('agent_refresh_health')
    } catch (e) {
      console.error('Failed to load agent statuses:', e)
    }
  }

  async function setActiveAgent(id: string) {
    try {
      await invoke('agent_set_active', { id })
      activeAgentId.value = id
      await loadProjects()
    } catch (e) {
      console.error('Failed to set active agent:', e)
    }
  }

  // ── Actions: projects ─────────────────────────────────────────

  async function loadProjects() {
    try {
      projects.value = await invoke<ProjectEntry[]>('agent_scan_all_projects')
    } catch (e) {
      console.error('Failed to load projects:', e)
    }
  }

  async function addProject(path: string) {
    try {
      const result = await invoke<ProjectEntry | null>('agent_add_project', { path })
      if (result) {
        projects.value.unshift(result)
        return result
      }
    } catch (e) {
      console.error('Failed to add project:', e)
    }
    return null
  }

  async function initProject(path: string) {
    try {
      await invoke('agent_init_project', { projectPath: path })
      await loadProjects()
    } catch (e) {
      console.error('Failed to init project:', e)
    }
  }

  function selectProject(path: string) {
    activeProjectPath.value = path
    activeSessionId.value = null
    sessionMessages.value = []
    loadSessions(path)
  }

  // ── Actions: sessions ─────────────────────────────────────────

  async function loadSessions(projectPath: string) {
    try {
      sessions.value = await invoke<SessionInfo[]>('agent_list_sessions', { projectPath })
    } catch (e) {
      console.error('Failed to load sessions:', e)
    }
  }

  async function loadSessionMessages(sessionId: string, projectPath: string) {
    try {
      sessionMessages.value = await invoke<SessionMessage[]>('agent_load_session', {
        sessionId,
        projectPath,
      })
    } catch (e) {
      console.error('Failed to load session messages:', e)
    }
  }

  async function selectSession(sessionId: string) {
    activeSessionId.value = sessionId
    if (activeProjectPath.value) {
      await loadSessionMessages(sessionId, activeProjectPath.value)
    }
  }

  // ── Actions: chat ─────────────────────────────────────────────

  async function sendMessage(text: string): Promise<boolean> {
    if (!activeProjectPath.value) return false

    const sessionId = `pending-${Date.now()}`

    try {
      const result = await invoke<ChatResult>('agent_send_message', {
        projectPath: activeProjectPath.value,
        sessionId: null,
        message: text,
      })

      const session: AgentSession = {
        id: result.session_id,
        projectPath: activeProjectPath.value!,
        messages: [],
        isStreaming: true,
      }
      activeSessions.value.set(result.session_id, session)
      activeSessionId.value = result.session_id

      return true
    } catch (e) {
      console.error('Failed to send message:', e)
      return false
    }
  }

  async function abortChat(sessionId: string) {
    try {
      await invoke('agent_abort', { sessionId })
      const session = activeSessions.value.get(sessionId)
      if (session) {
        session.isStreaming = false
      }
    } catch (e) {
      console.error('Failed to abort:', e)
    }
  }

  async function openInTerminal(projectPath: string, sessionId?: string) {
    try {
      await invoke('agent_open_terminal', {
        projectPath,
        resumeSessionId: sessionId || null,
      })
    } catch (e) {
      console.error('Failed to open terminal:', e)
    }
  }

  // ── Stream event handling ─────────────────────────────────────

  function pushStreamChunk(chunk: AgentStreamChunk) {
    const sid = chunk.session_id
    let session = activeSessions.value.get(sid)

    if (!session) {
      // Create session on first chunk
      session = {
        id: sid,
        projectPath: activeProjectPath.value || '',
        messages: [],
        isStreaming: true,
      }
      activeSessions.value.set(sid, session)
    }

    const data = chunk.data

    switch (data.kind) {
      case 'text_delta': {
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant' && last.streaming) {
          last.content += data.delta || ''
        } else {
          session.messages.push({
            id: `msg-${Date.now()}`,
            role: 'assistant',
            content: data.delta || '',
            thinking: '',
            timestamp: Date.now(),
            streaming: true,
            error: '',
            tools: [],
            contentBlocks: [],
          })
        }
        break
      }

      case 'thinking': {
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant' && last.streaming) {
          last.thinking += data.delta || ''
        } else {
          session.messages.push({
            id: `msg-${Date.now()}`,
            role: 'assistant',
            content: '',
            thinking: data.delta || '',
            timestamp: Date.now(),
            streaming: true,
            error: '',
            tools: [],
            contentBlocks: [],
          })
        }
        break
      }

      case 'tool_use_start': {
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant') {
          last.tools.push({
            id: data.call_id!,
            name: data.tool!,
            input: data.input,
          })
          last.contentBlocks.push({
            type: 'tool_use',
            id: data.call_id!,
            name: data.tool!,
            input: data.input,
          })
        }
        break
      }

      case 'tool_use_result': {
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant') {
          const tool = last.tools.find(t => t.id === data.call_id)
          if (tool) {
            tool.output = data.output
            tool.isError = data.is_error
          }
          last.contentBlocks.push({
            type: 'tool_result',
            tool_use_id: data.call_id!,
            content: data.output!,
            is_error: data.is_error || false,
          })
        }
        break
      }

      case 'session_resolved': {
        const realId = data.session_id
        if (realId && realId !== sid) {
          const session = activeSessions.value.get(sid)
          if (session) {
            activeSessions.value.delete(sid)
            activeSessions.value.set(realId, session)
            session.id = realId
            activeSessionId.value = realId
          }
        }
        break
      }

      case 'turn_complete': {
        session.isStreaming = false
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant') {
          last.streaming = false
        }
        break
      }

      case 'error': {
        const last = session.messages.length > 0
          ? session.messages[session.messages.length - 1]
          : null
        if (last && last.role === 'assistant') {
          last.error = data.message || 'Unknown error'
          if (!data.recoverable) {
            last.streaming = false
            session.isStreaming = false
          }
        }
        break
      }
    }

    // Force reactivity
    activeSessions.value = new Map(activeSessions.value)
  }

  function setupStreamListener() {
    if (unlisten) return

    listen<AgentStreamChunk>('agent-stream-chunk', (event) => {
      pushStreamChunk(event.payload)
    }).then(fn => {
      unlisten = fn
    }).catch(err => {
      console.error('Failed to setup stream listener:', err)
    })
  }

  function teardownStreamListener() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  // ── Init ──────────────────────────────────────────────────────

  async function initialize() {
    setupStreamListener()
    await Promise.all([loadAgents(), loadAgentStatuses(), loadProjects()])
    try {
      const active = await invoke<string>('agent_get_active')
      activeAgentId.value = active
    } catch {
      // use default
    }
  }

  return {
    // state
    agents,
    agentStatuses,
    activeAgentId,
    projects,
    activeProjectPath,
    sessions,
    activeSessionId,
    sessionMessages,
    activeSessions,
    // computed
    activeAgent,
    activeProject,
    installedAgents,
    activeAgentStatus,
    activeChatSession,
    // actions
    loadAgents,
    loadAgentStatuses,
    setActiveAgent,
    loadProjects,
    addProject,
    initProject,
    selectProject,
    loadSessions,
    loadSessionMessages,
    selectSession,
    sendMessage,
    abortChat,
    openInTerminal,
    initialize,
    teardownStreamListener,
  }
})
