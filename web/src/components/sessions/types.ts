/**
 * Types for session list components.
 */

import type { SessionFile } from "@/lib/types";

export interface SessionRowProps {
  session: SessionFile;
}

export interface SessionTableProps {
  sessions: SessionFile[];
}
