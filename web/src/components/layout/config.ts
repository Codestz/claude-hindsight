/**
 * Configuration for layout components.
 */

import {
  LayoutDashboard,
  Activity,
  FolderOpen,
  MessageSquare,
  Bot,
  Sparkles,
} from "lucide-react";
import type { NavItem } from "./types";

/** Sidebar width in pixels (expanded). */
export const SIDEBAR_W = 232;

/** Sidebar width when collapsed. */
export const SIDEBAR_COLLAPSED_W = 56;

/** Window width below which sidebar auto-collapses. */
export const SIDEBAR_BREAKPOINT = 1024;

/** Navigation items for the sidebar. Add new routes here. */
export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/activity", label: "Activity", icon: Activity },
  { to: "/projects", label: "Projects", icon: FolderOpen, section: "Analyze" },
  { to: "/prompts", label: "Prompts", icon: MessageSquare },
  { to: "/agents", label: "Agents", icon: Bot, section: "Configure" },
  { to: "/skills", label: "Skills", icon: Sparkles },
];

/** Check if a nav item is active based on current pathname. */
export function isNavActive(to: string, pathname: string): boolean {
  if (to === "/") return pathname === "/";
  return pathname.startsWith(to) || (to === "/projects" && pathname.startsWith("/sessions"));
}
