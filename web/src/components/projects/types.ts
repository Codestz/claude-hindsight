/**
 * Types for project components.
 */

import type { ProjectStats } from "@/lib/types";

export interface ProjectRowProps {
  project: ProjectStats;
  maxSize: number;
}

export interface ProjectTableProps {
  projects: ProjectStats[];
}
