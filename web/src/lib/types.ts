// ============================================================
// HINDSIGHT — TypeScript Type Layer
//
// Every type mirrors a Rust struct/enum from:
//   src/server/dto.rs          → API response DTOs
//   src/api/responses.rs       → NodeResponse / TreeResponse
//   src/parser/models.rs       → ExecutionNode + nested types
//   src/analyzer/smart_label.rs → node_type values and color tokens
//
// Rule: if the Rust DTO changes, update the matching interface here.
// ============================================================

// ─────────────────────────────────────────────────────────────
// § 1 — ANALYTICS & PROJECT API
//       GET /api/projects
//       GET /api/analytics/global
//       GET /api/analytics/:project
// ─────────────────────────────────────────────────────────────

/** One project entry — from GET /api/projects */
export interface ProjectStats {
  project_name: string;
  session_count: number;       // usize
  total_size: number;          // u64, bytes
  last_activity: number | null; // unix seconds, null if no sessions
}

/** Global dashboard analytics — from GET /api/analytics/global */
export interface GlobalAnalytics {
  total_sessions: number;       // usize
  sessions_this_week: number;   // usize
  sessions_today: number;       // usize
  total_size: number;           // u64, bytes
  total_projects: number;       // usize
  subagent_count: number;       // usize — sessions that spawned sub-agents
  avg_session_size: number;     // u64, bytes
  most_active_project: string | null;
  top_tools: [string, number][]; // Vec<(String, usize)> → [["Read", 312], ...]
  total_errors: number;         // usize
}

/** Per-project analytics — from GET /api/analytics/:project */
export interface ProjectAnalytics {
  project_name: string;
  total_sessions: number;
  sessions_this_week: number;
  sessions_today: number;
  total_size: number;           // bytes
  subagent_count: number;
  avg_session_size: number;     // bytes
  top_tools: [string, number][];
  last_activity: number | null; // unix seconds
  total_errors: number;
}

// ─────────────────────────────────────────────────────────────
// § 2 — SESSION API
//       GET /api/sessions
//       GET /api/sessions/:id
// ─────────────────────────────────────────────────────────────

/** Session file metadata — from GET /api/sessions and /sessions/:id */
export interface SessionFile {
  session_id: string;
  project_name: string;
  file_size: number;            // bytes
  created_at: number;           // unix seconds
  modified_at: number;          // unix seconds
  has_subagents: boolean;
  model: string | null;         // e.g. "claude-sonnet-4-5-20250929"
  error_count: number;
  first_message: string | null; // first user message text, truncated
  source_dir: string;           // config path e.g. "~/.claude/projects"
  subagent_models: string[] | null; // unique models used by subagents
}

// ─────────────────────────────────────────────────────────────
// § 3 — NODE CONTENT TYPES
//       These types model the fields that are FLATTENED into
//       NodeResponse from ExecutionNode (src/parser/models.rs).
//       They appear at the top level of a NodeResponse JSON object.
// ─────────────────────────────────────────────────────────────

/**
 * Token usage breakdown.
 * Claude bills cache creation and cache reads at different rates than
 * regular input tokens — all three are tracked separately.
 */
export interface TokenUsage {
  input_tokens: number | null;
  output_tokens: number | null;
  cache_creation_input_tokens: number | null;
  cache_read_input_tokens: number | null;
}

/**
 * A typed content block inside a message.
 * Matches ContentBlock enum from models.rs (internally tagged with "type").
 */
export type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; thinking: string; signature: string | null }
  | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
  | { type: "tool_result"; tool_use_id: string; content: unknown; is_error: boolean | null }
  | { type: string; [key: string]: unknown }; // forward-compat for future block types

/**
 * Message content — either a legacy plain string or a typed block array.
 * Matches MessageContent enum from models.rs (untagged).
 */
export type MessageContent = string | ContentBlock[];

/** Message object inside a user or assistant node */
export interface NodeMessage {
  id: string | null;
  role: string | null;           // "user" | "assistant" | "system"
  model: string | null;          // e.g. "claude-sonnet-4-5-20250929"
  content: MessageContent | null;
  usage: TokenUsage | null;
  // extra: arbitrary additional message metadata
  [key: string]: unknown;
}

/** Tool use (call) details — present when a node invokes a tool */
export interface NodeToolUse {
  name: string;                  // e.g. "Read", "Write", "Bash"
  input: Record<string, unknown>; // JSON parameters passed to the tool
  id: string | null;
}

/** Tool result (output) details — present on the result node */
export interface NodeToolResult {
  tool_use_id: string | null;
  content: string | null;
  file: {
    file_path: string | null;
    content: string | null;
    num_lines: number | null;
  } | null;
  is_error: boolean | null;
  error: string | null;
  duration_ms: number | null;    // how long the tool took
}

/** Progress update from a long-running operation */
export interface NodeProgress {
  message: string | null;
  percentage: number | null;
}

// ─────────────────────────────────────────────────────────────
// § 4 — NODE TYPE TAXONOMY
//       From src/analyzer/smart_label.rs
// ─────────────────────────────────────────────────────────────

/**
 * Raw node_type values as written in the JSONL transcript.
 * These are the values you'll find in NodeResponse.node_type.
 */
export type NodeType =
  | "user"
  | "assistant"
  | "progress"
  | "file-history-snapshot"
  | "system"
  | "queue-operation";

/**
 * Color token returned by the server's smart_label system.
 * Maps directly to CSS custom properties in the design system:
 *   cyan    → var(--cyan)
 *   green   → var(--green)
 *   magenta → var(--purple)
 *   yellow  → var(--amber)
 *   blue    → var(--info)  [cyan]
 *   gray    → var(--text-3)
 *   white   → var(--text-1)
 */
export type NodeColor =
  | "cyan"      // user message
  | "green"     // assistant text
  | "magenta"   // thinking, file snapshot, agent progress
  | "yellow"    // tool calls, bash, hooks, other progress
  | "blue"      // tool result (user node containing tool output)
  | "gray"      // system events, queue operations
  | "white";    // unknown / fallback

// ─────────────────────────────────────────────────────────────
// § 5 — NODE RESPONSE & TREE API
//       GET /api/sessions/:id/nodes
//
//       NodeResponse merges:
//         - Explicit fields from responses.rs
//         - All ExecutionNode fields (#[serde(flatten)])
//         - All extra HashMap fields (doubly flattened)
//
//       This means fields like `message`, `tool_use`, `token_usage`,
//       `parent_uuid`, etc. all appear at the top level of the JSON.
// ─────────────────────────────────────────────────────────────

export interface NodeResponse {
  // ── Explicit NodeResponse fields ─────────────────────────────
  uuid: string | null;
  node_type: NodeType | string;  // string fallback for future types
  label: string;                 // smart label computed by server
  color: NodeColor | string;     // semantic color token
  summary: string;               // short text preview
  depth: number;                 // 0 = root
  has_error: boolean;
  timestamp: number | null;      // milliseconds (not seconds — note!)
  children: NodeResponse[];

  // ── Flattened from ExecutionNode ─────────────────────────────
  parent_uuid: string | null;

  // The raw "type" field (same value as node_type, different JSON key)
  type: string | null;

  message: NodeMessage | null;
  tool_use: NodeToolUse | null;
  tool_result: NodeToolResult | null;

  // camelCase because Rust uses #[serde(rename = "toolUseResult")]
  toolUseResult: unknown;

  thinking: string | null;
  progress: NodeProgress | null;
  token_usage: TokenUsage | null;

  // ── Backend-computed fields ─────────────────────────────────
  tool_name?: string;       // "Read", "Bash", "Edit" — on tool-call nodes
  file_paths?: string[];    // paths from tool inputs

  // ── Prompt score (0–100) — only present on user nodes
  // Scores >= 40 indicate a meaningful prompt.
  prompt_score?: number;

  // ── Extra / unknown fields (doubly flattened from ExecutionNode.extra)
  // May include: subtype, durationMs, data, snapshot, etc.
  [key: string]: unknown;
}

/** Tree response — from GET /api/sessions/:id/nodes */
export interface TreeResponse {
  roots: NodeResponse[];
  total_nodes: number;
  max_depth: number;
}

// ─────────────────────────────────────────────────────────────
// § 6 — MISC API TYPES
// ─────────────────────────────────────────────────────────────

/** Cross-session prompt entry — from GET /api/prompts */
export interface PromptEntry {
  session_id: string;
  project_name: string;
  prompt_text: string;
  prompt_score: number;
  timestamp: number | null;
  model: string | null;
}

/**
 * 14-day (or N-day) conversation count array.
 * Index 0 = oldest day, last index = today.
 * From GET /api/analytics/global/sparkline?days=N
 */
export type Sparkline = number[];

// ─────────────────────────────────────────────────────────────
// § 7 — OTEL TELEMETRY TYPES
//       GET /api/telemetry/summary
//       GET /api/telemetry/sessions
//       GET /api/otel/session-summary
//       GET /api/otel/global-summary
//       GET /api/otel/logs
//       GET /api/otel/metrics
// ─────────────────────────────────────────────────────────────

/** Aggregated telemetry summary — from GET /api/telemetry/summary */
export interface TelemetrySummary {
  total_sessions: number;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cost_usd: number;
}

/** Per-session telemetry — from GET /api/telemetry/sessions */
export interface SessionTelemetry {
  session_id: string;
  project_name: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cost_usd: number;
}

/** OTEL session summary — from GET /api/otel/session-summary?session_id= */
export interface OtelSessionSummary {
  session_id: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cost_usd: number;
  api_requests: number;
}

/** OTEL global summary — from GET /api/otel/global-summary */
export interface OtelGlobalSummary {
  total_sessions: number;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cost_usd: number;
  api_requests: number;
}

/** OTEL log record — from GET /api/otel/logs */
export interface OtelLogDto {
  received_at: number;
  session_id: string | null;
  event_name: string | null;
  model: string | null;
  cost_usd: number | null;
  input_tokens: number | null;
  output_tokens: number | null;
  cache_read_tokens: number | null;
  cache_creation_tokens: number | null;
  duration_ms: number | null;
  tool_name: string | null;
  success: boolean | null;
  error_message: string | null;
  status_code: number | null;
  severity: string | null;
  body: string | null;
  time_unix_nano: string | null;
}

// ─────────────────────────────────────────────────────────────
// § 8 — HOOK EVENT TYPES
//       GET /api/hooks/tool-events
//       GET /api/hooks/subagent-events
//       GET /api/hooks/permission-events
//       GET /api/hooks/lifecycle-events
//       GET /api/hooks/activity-summary
// ─────────────────────────────────────────────────────────────

export interface HookToolEvent {
  id: number;
  session_id: string;
  occurred_at: number;
  hook_event: string;
  tool_name: string | null;
  tool_input: string | null;
  tool_result: string | null;
  error_message: string | null;
  is_interrupt: boolean | null;
  tool_use_id: string | null;
  cwd: string | null;
}

export interface HookSubagentEvent {
  id: number;
  session_id: string;
  occurred_at: number;
  hook_event: string;
  agent_type: string | null;
  agent_name: string | null;
  cwd: string | null;
}

export interface HookPermissionEvent {
  id: number;
  session_id: string;
  occurred_at: number;
  tool_name: string | null;
  tool_input: string | null;
  cwd: string | null;
}

export interface HookLifecycleEvent {
  id: number;
  session_id: string;
  occurred_at: number;
  event_name: string;
  attributes: string | null;
}

export interface HookActivitySummary {
  total_tool_events: number;
  total_subagent_events: number;
  total_lifecycle_events: number;
  total_permission_events: number;
  tool_event_counts: [string, number][];
  recent_errors: number;
}

// ─────────────────────────────────────────────────────────────
// § 9 — AGENT & SKILL TYPES
//       GET /api/agents
//       GET /api/agents/:name
//       GET /api/skills
//       GET /api/skills/:name
// ─────────────────────────────────────────────────────────────

/** All copies of an agent sharing the same name across scopes — from GET /api/agents and /api/agents/:name */
export interface AgentGroup {
  name: string;
  /** true when every copy has byte-identical body content */
  identical: boolean;
  items: AgentConfig[];
}

/** Agent configuration — from GET /api/agents (inside AgentGroup.items) */
export interface AgentConfig {
  name: string;
  description: string | null;
  tools: string[] | null;
  model: string | null;
  permission_mode: string | null;
  max_turns: number | null;
  skills: string[] | null;
  hooks: unknown | null;
  memory: string | null;
  background: boolean | null;
  isolation: string | null;
  body: string;
  file_path: string;
  scope: string;
  project_name: string | null;
}

/** Supplementary reference or rule file associated with a skill */
export interface SkillReference {
  name: string;
  path: string;
  content: string;
  category: string; // "reference" | "rule"
}

/** All copies of a skill sharing the same name across scopes — from GET /api/skills and /api/skills/:name */
export interface SkillGroup {
  name: string;
  /** true when every copy has byte-identical body content */
  identical: boolean;
  items: SkillConfig[];
}

/** Skill configuration — from GET /api/skills (inside SkillGroup.items) */
export interface SkillConfig {
  name: string;
  description: string | null;
  user_invocable: boolean | null;
  allowed_tools: string[] | null;
  model: string | null;
  context: string | null;
  agent: string | null;
  hooks: unknown | null;
  body: string;
  file_path: string;
  scope: string;
  project_name: string | null;
  references: SkillReference[];
}

// ─────────────────────────────────────────────────────────────
// § 10 — CONVERSATION TIMELINE TYPES
// ─────────────────────────────────────────────────────────────

/** A turn groups consecutive nodes by role boundary */
export interface Turn {
  id: string;
  role: "user" | "assistant" | "system";
  nodes: NodeResponse[];
  timestamp: number | null;
}

/** Token cost for a single user turn */
export interface TurnCost {
  turnIndex: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  totalTokens: number;
  toolCalls: number;
}

/** Aggregated session statistics for the header bar */
export interface SessionStats {
  totalTurns: number;
  toolCalls: number;
  errorCount: number;
  totalTokens: number;
  costUsd: number | null;
  durationMs: number | null;
  turnCosts: TurnCost[];
}
