/**
 * Shared types for UI components.
 *
 * All prop interfaces for reusable UI primitives live here.
 */

// ── Card ─────────────────────────────────────────────────────

export interface CardProps {
  children: React.ReactNode;
  glow?: string;
}

// ── CodeRender ───────────────────────────────────────────────

export interface CodeRenderProps {
  content: string;
  language?: string;
  filePath?: string;
  error?: boolean;
  maxHeight?: string;
}

// ── EmptyState ───────────────────────────────────────────────

export interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
}

// ── ErrorState ───────────────────────────────────────────────

export interface ErrorStateProps {
  message?: string | null;
}

// ── FilterChips ──────────────────────────────────────────────

export interface FilterChipsProps {
  options: string[];
  active: Set<string>;
  onToggle: (option: string) => void;
}

// ── HindsightLogo ────────────────────────────────────────────

export interface HindsightLogoProps {
  size?: number;
  glow?: boolean;
}

// ── InlineDiff ───────────────────────────────────────────────

export type DiffLine = { type: "same" | "removed" | "added"; line: string };

export interface SideSpec {
  label: string;
  content: string;
}

export interface InlineDiffProps {
  left: SideSpec;
  right: SideSpec;
}

// ── PageShell ────────────────────────────────────────────────

export interface PageShellProps {
  children: React.ReactNode;
  maxWidth?: string;
}

// ── ResizablePanel ───────────────────────────────────────────

export interface ResizablePanelProps {
  left: React.ReactNode;
  right: React.ReactNode;
  storageKey?: string;
  defaultRatio?: number;
  minLeftPx?: number;
  minRightPx?: number;
  presets?: number[];
}

// ── SectionHeader ────────────────────────────────────────────

export interface SectionHeaderProps {
  icon?: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  title: string;
  count?: number;
  accent?: boolean;
}

// ── StatCard ─────────────────────────────────────────────────

export interface StatCardProps {
  label: string;
  value: string;
  sub?: string;
  accent?: boolean;
  valueColor?: string;
}

// ── Badge ────────────────────────────────────────────────────

export interface BadgeProps {
  children: React.ReactNode;
  color?: string;
  bg?: string;
}
