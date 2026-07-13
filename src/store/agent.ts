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

export interface ProjectMeta {
  tags: string[]
  notes: string
  custom_name: string | null
}

export interface CustomCommand {
  id: string
  name: string
  description: string
  command: string
  cwd: string
}

export interface EnvStatus {
  node_installed: boolean
  node_version: string | null
  npm_installed: boolean
  npm_version: string | null
  python_installed: boolean
  python_version: string | null
}

export interface AgentCommandPreset {
  name: string
  description: string
  command: string
  is_launch: boolean
  is_resume: boolean
  is_init: boolean
}

export interface KnownAgent {
  id: string
  display_name: string
  binary: string
  installed: boolean
  version: string | null
  install_hint: string
  install_command: string | null
  home_url: string | null
  description: string
  source: string
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

  function deselectProject() {
    activeProjectPath.value = null
    activeSessionId.value = null
    sessionMessages.value = []
    sessions.value = []
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

  function clearActiveSession() {
    activeSessionId.value = null
    sessionMessages.value = []
  }

  // ── Actions: chat ─────────────────────────────────────────────

  async function sendMessage(text: string): Promise<boolean> {
    if (!activeProjectPath.value) return false

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

  // ── Actions: project meta ────────────────────────────────────

  const projectMetas = ref<Record<string, ProjectMeta>>({})

  async function loadProjectMetas() {
    try {
      projectMetas.value = await invoke<Record<string, ProjectMeta>>('agent_load_project_metas')
    } catch (e) {
      console.error('Failed to load project metas:', e)
    }
  }

  async function saveProjectMeta(encodedName: string, meta: ProjectMeta) {
    try {
      await invoke('agent_save_project_meta', { encodedName, meta })
      projectMetas.value[encodedName] = meta
    } catch (e) {
      console.error('Failed to save project meta:', e)
    }
  }

  async function hideProject(encodedName: string) {
    try {
      await invoke('agent_hide_project', { encodedName })
      projects.value = projects.value.filter(p => p.encoded_name !== encodedName)
    } catch (e) {
      console.error('Failed to hide project:', e)
    }
  }

  async function addManualProject(path: string) {
    try {
      await invoke('agent_add_manual_project', { path })
      await loadProjects()
    } catch (e) {
      console.error('Failed to add manual project:', e)
    }
  }

  // ── Actions: project merges ──────────────────────────────────

  async function mergeProjects(primary: string, secondaries: string[]) {
    try {
      await invoke('agent_merge_projects', { primary, secondaries })
      await loadProjects()
    } catch (e) {
      console.error('Failed to merge projects:', e)
      throw e
    }
  }

  async function splitProject(primary: string) {
    try {
      await invoke('agent_split_project', { primary })
      await loadProjects()
    } catch (e) {
      console.error('Failed to split project:', e)
      throw e
    }
  }

  async function getProjectMerges(): Promise<Record<string, string[]>> {
    try {
      return await invoke<Record<string, string[]>>('agent_get_project_merges')
    } catch (e) {
      console.error('Failed to get project merges:', e)
      return {}
    }
  }

  // ── Actions: commands ────────────────────────────────────────

  const commandPresets = ref<AgentCommandPreset[]>([])
  const customCommands = ref<CustomCommand[]>([])

  async function loadCommandPresets() {
    try {
      commandPresets.value = await invoke<AgentCommandPreset[]>('agent_command_presets')
    } catch (e) {
      console.error('Failed to load command presets:', e)
    }
  }

  async function loadCustomCommands() {
    try {
      customCommands.value = await invoke<CustomCommand[]>('agent_list_custom_commands')
    } catch (e) {
      console.error('Failed to load custom commands:', e)
    }
  }

  async function saveCustomCommand(cmd: CustomCommand) {
    try {
      await invoke('agent_save_custom_command', { cmd })
      await loadCustomCommands()
    } catch (e) {
      console.error('Failed to save custom command:', e)
    }
  }

  async function deleteCustomCommand(id: string) {
    try {
      await invoke('agent_delete_custom_command', { id })
      customCommands.value = customCommands.value.filter(c => c.id !== id)
    } catch (e) {
      console.error('Failed to delete custom command:', e)
    }
  }

  async function runInTerminal(command: string, cwd?: string) {
    try {
      await invoke('agent_run_in_terminal', { command, cwd: cwd || null })
    } catch (e) {
      console.error('Failed to run in terminal:', e)
    }
  }

  // ── Actions: environment ─────────────────────────────────────

  const envStatus = ref<EnvStatus | null>(null)

  async function checkEnvironment() {
    try {
      envStatus.value = await invoke<EnvStatus>('agent_check_environment')
    } catch (e) {
      console.error('Failed to check environment:', e)
    }
  }

  // ── Known agents probe ────────────────────────────────────

  const knownAgents = ref<KnownAgent[]>([])

  async function probeKnownAgents() {
    try {
      knownAgents.value = await invoke<KnownAgent[]>('agent_probe_known_agents')
    } catch (e) {
      console.error('Failed to probe known agents:', e)
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
    projectMetas,
    commandPresets,
    customCommands,
    envStatus,
    // computed
    activeAgent,
    activeProject,
    installedAgents,
    activeAgentStatus,
    activeChatSession,
    // actions: agents
    loadAgents,
    loadAgentStatuses,
    setActiveAgent,
    // actions: projects
    loadProjects,
    addProject,
    initProject,
    selectProject,
    deselectProject,
    loadProjectMetas,
    saveProjectMeta,
    hideProject,
    addManualProject,
    mergeProjects,
    splitProject,
    getProjectMerges,
    // actions: sessions
    loadSessions,
    loadSessionMessages,
    selectSession,
    clearActiveSession,
    // actions: chat
    sendMessage,
    abortChat,
    openInTerminal,
    // actions: commands
    loadCommandPresets,
    loadCustomCommands,
    saveCustomCommand,
    deleteCustomCommand,
    runInTerminal,
    // actions: environment
    checkEnvironment,
    knownAgents,
    probeKnownAgents,
    // init
    initialize,
    teardownStreamListener,
  }
})
