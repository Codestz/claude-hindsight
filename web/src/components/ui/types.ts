/**
 * Shared types for UI components.
 *
 * All prop interfaces for reusable UI primitives live here.
 * Components import from this file — no inline definitions.
 */

import type { ReactNode } from "react";

// ── Badge ────────────────────────────────────────────────────

export type BadgeVariant =
  | "success" | "error" | "warn" | "info" | "purple" | "muted" | "default";

export interface BadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
}

// ── Card ─────────────────────────────────────────────────────

export interface CardProps {
  children: React.ReactNode;
  padding?: string;
  glow?: string;
  animate?: boolean;
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
  suggestion?: string;
}

// ── FilterChips ──────────────────────────────────────────────

export interface FilterChipsProps {
  options: string[];
  active: Set<string>;
  onToggle: (value: string) => void;
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
  body: string;
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
  title: string;
  count?: number;
  action?: { label: string; href: string };
}

// ── StatCard ─────────────────────────────────────────────────

export interface StatCardProps {
  label: string;
  value: string;
  sub?: string;
  accent?: boolean;
  valueColor?: string;
}
