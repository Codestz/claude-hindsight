// TypeScript mirrors of Rust DTOs

export interface ProjectStats {
  project_name: string;
  session_count: number;
  total_size: number;
  last_activity: number | null;
}

export interface GlobalAnalytics {
  total_sessions: number;
  sessions_this_week: number;
  sessions_today: number;
  total_size: number;
  total_projects: number;
  subagent_count: number;
  avg_session_size: number;
  most_active_project: string | null;
  top_tools: [string, number][];
  total_tokens: number;
  total_cost: number;
  total_errors: number;
}

export interface ProjectAnalytics {
  project_name: string;
  total_sessions: number;
  sessions_this_week: number;
  sessions_today: number;
  total_size: number;
  subagent_count: number;
  avg_session_size: number;
  top_tools: [string, number][];
  last_activity: number | null;
  total_tokens: number;
  total_cost: number;
  total_errors: number;
}

export interface SessionFile {
  session_id: string;
  project_name: string;
  file_size: number;
  created_at: number;
  modified_at: number;
  has_subagents: boolean;
  total_tokens: number;
  estimated_cost: number;
  model: string | null;
  error_count: number;
  first_message: string | null;
}

export interface NodeResponse {
  uuid: string | null;
  node_type: string;
  label: string;
  color: string;
  summary: string;
  depth: number;
  has_error: boolean;
  timestamp: number | null;
  children: NodeResponse[];
  [key: string]: unknown;
}

export interface TreeResponse {
  roots: NodeResponse[];
  total_nodes: number;
  max_depth: number;
}
