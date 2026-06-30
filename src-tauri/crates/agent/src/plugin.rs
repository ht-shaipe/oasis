//! Agent plugin system — the core abstraction for CLI-based AI agents.
//!
//! Each CLI agent (Claude Code, Codex, etc.) implements `AgentPlugin`.
//! `AgentRegistry` manages the collection and routes to the active one.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::normalized::NormalizedEvent;

// ── Public types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
    pub version: String,
    pub icon: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentHealth {
    pub installed: bool,
    pub version: Option<String>,
    pub error: Option<String>,
    pub binary_path: Option<String>,
    pub last_checked_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatus {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub installed: bool,
    pub version: Option<String>,
    pub error: Option<String>,
    pub install_hint: Option<String>,
    pub native_install_command: Option<String>,
}

/// Arguments for building a CLI chat command.
#[derive(Debug, Clone)]
pub struct ChatArgs {
    pub project_path: String,
    pub session_id: Option<String>,
    pub message: String,
}

/// Result returned from agent_send_message.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatResult {
    pub agent_id: String,
    pub session_id: String,
    pub process_id: u32,
}

// ── AgentPlugin trait ─────────────────────────────────────────────

/// Every CLI agent must implement this trait.
#[async_trait::async_trait]
pub trait AgentPlugin: Send + Sync {
    /// Basic metadata.
    fn info(&self) -> AgentInfo;

    /// Check if agent binary is installed and return health info (sync).
    fn probe_sync(&self) -> AgentHealth;

    /// Async probe — default delegates to sync.
    async fn probe(&self) -> AgentHealth {
        self.probe_sync()
    }

    /// Hint shown when agent is not installed.
    fn install_hint(&self) -> Option<String> {
        None
    }

    /// Shell command to install this agent.
    fn native_install_command(&self) -> Option<String> {
        None
    }

    // ── Chat ─────────────────────────────────────────────────

    /// Build the tokio Command for starting a chat session.
    fn build_chat_command(&self, args: &ChatArgs) -> tokio::process::Command;

    /// Whether this agent uses ACP (persistent JSON-RPC) protocol.
    fn uses_acp(&self) -> bool {
        false
    }

    /// Returns (binary, args) for ACP subprocess. Only called when uses_acp() is true.
    fn acp_command(&self) -> (&str, Vec<&str>) {
        ("", vec![])
    }

    /// Whether to pipe stdin for abort control sequences.
    fn wants_stdin_pipe(&self) -> bool {
        self.abort_sequence().is_some()
    }

    /// Control sequence sent to stdin to trigger abort.
    fn abort_sequence(&self) -> Option<&'static [u8]> {
        None
    }

    /// Grace period after sending abort sequence before force-killing.
    fn abort_grace_period(&self) -> std::time::Duration {
        std::time::Duration::from_millis(1200)
    }

    /// Build a terminal resume command for the given session.
    fn build_resume_command(&self, session_id: &str) -> String;

    /// Parse a raw JSON Line from stdout into normalized events.
    fn normalize_stream_event(&self, event: &serde_json::Value) -> Vec<NormalizedEvent>;

    // ── Session management ───────────────────────────────────

    /// List sessions for a project (scans JSONL files).
    fn list_sessions(&self, project_path: &str) -> Result<Vec<SessionInfo>, String>;

    /// Load messages from a session JSONL file.
    fn load_session_messages(
        &self,
        session_id: &str,
        project_path: &str,
    ) -> Result<Vec<SessionMessage>, String>;

    // ── Project management ───────────────────────────────────

    /// Discover projects used by this agent.
    fn scan_projects(&self) -> Vec<ProjectEntry>;

    /// Add a project manually.
    fn add_project(&self, path: &str) -> Option<ProjectEntry>;

    // ── Terminal ─────────────────────────────────────────────

    /// Open project in native terminal, optionally resuming a session.
    fn open_in_terminal(
        &self,
        project_path: &str,
        resume_session_id: Option<&str>,
    ) -> Result<u32, String>;

    /// Initialize a project for this agent (creates config dirs if needed).
    fn init_project(&self, project_path: &str) -> Result<bool, String>;
}

// ── Supporting types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub display_name: Option<String>,
    pub started_at: Option<String>,
    pub last_active: Option<String>,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: String,
    pub encoded_name: String,
    pub session_count: usize,
    pub last_active: Option<String>,
    pub agent_id: String,
    pub initialized: bool,
}

// ── AgentRegistry ─────────────────────────────────────────────────

pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn AgentPlugin>>,
    active_id: String,
    health_cache: HashMap<String, AgentHealth>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let claude = crate::adapters::claude_code::ClaudeCodeAgent::new();
        let id = claude.info().id.clone();
        let mut agents: HashMap<String, Box<dyn AgentPlugin>> = HashMap::new();
        agents.insert(id.clone(), Box::new(claude));

        let codex = crate::adapters::codex::CodexAdapter::new();
        let codex_id = codex.info().id.clone();
        agents.insert(codex_id, Box::new(codex));

        Self {
            agents,
            active_id: id,
            health_cache: HashMap::new(),
        }
    }

    pub fn active(&self) -> &dyn AgentPlugin {
        self.agents
            .get(&self.active_id)
            .map(|a| a.as_ref())
            .expect("active agent must exist")
    }

    pub fn active_id(&self) -> &str {
        &self.active_id
    }

    pub fn set_active(&mut self, id: &str) -> Result<(), String> {
        if self.agents.contains_key(id) {
            self.active_id = id.to_string();
            Ok(())
        } else {
            Err(format!("agent not found: {id}"))
        }
    }

    pub fn get(&self, id: &str) -> Option<&dyn AgentPlugin> {
        self.agents.get(id).map(|a| a.as_ref())
    }

    /// List all agent metadata.
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.agents.values().map(|a| a.info()).collect()
    }

    /// Probe all agents and return health + metadata.
    pub fn list_agent_statuses(&self) -> Vec<AgentStatus> {
        self.agents
            .values()
            .map(|a| {
                let info = a.info();
                let health = a.probe_sync();
                AgentStatus {
                    id: info.id,
                    display_name: info.display_name,
                    icon: info.icon,
                    installed: health.installed,
                    version: health.version,
                    error: health.error,
                    install_hint: a.install_hint(),
                    native_install_command: a.native_install_command(),
                }
            })
            .collect()
    }

    /// Probe all agents on a new task (for startup readiness check).
    pub async fn refresh_health(&self) {
        for agent in self.agents.values() {
            let _ = agent.probe().await;
        }
    }

    /// Probe all agents, cache results, and return statuses using cache.
    pub fn refresh_health_cache(&mut self) -> Vec<AgentStatus> {
        for (id, agent) in &self.agents {
            let health = agent.probe_sync();
            self.health_cache.insert(id.clone(), health);
        }
        self.list_agent_statuses_cached()
    }

    /// Return statuses using cached health (no probing).
    pub fn list_agent_statuses_cached(&self) -> Vec<AgentStatus> {
        self.agents
            .values()
            .map(|a| {
                let info = a.info();
                let cached = self.health_cache.get(&info.id);
                let (installed, version, error) = match cached {
                    Some(h) => (h.installed, h.version.clone(), h.error.clone()),
                    None => (false, None, Some("not probed yet".to_string())),
                };
                AgentStatus {
                    id: info.id,
                    display_name: info.display_name,
                    icon: info.icon,
                    installed,
                    version,
                    error,
                    install_hint: a.install_hint(),
                    native_install_command: a.native_install_command(),
                }
            })
            .collect()
    }

    /// Scan projects across all installed agents.
    pub fn scan_all_projects(&self) -> Vec<ProjectEntry> {
        let mut all = Vec::new();
        for agent in self.agents.values() {
            let health = self.health_cache.get(&agent.info().id);
            let is_installed = health.map(|h| h.installed).unwrap_or(false);
            if is_installed {
                all.extend(agent.scan_projects());
            }
        }
        all.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        all
    }

    /// Scan projects for a specific agent.
    pub fn scan_projects_for(&self, agent_id: &str) -> Vec<ProjectEntry> {
        self.agents
            .get(agent_id)
            .map(|a| a.scan_projects())
            .unwrap_or_default()
    }
}
