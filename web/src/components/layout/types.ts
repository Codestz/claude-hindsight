/**
 * Types for layout components.
 */

export interface NavItem {
  to: string;
  label: string;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  section?: string;
}

export interface SidebarProps {
  onSearch: () => void;
}

export interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

export type CommandResultItem =
  | { type: "session"; id: string; project: string; message: string; when: number }
  | { type: "project"; name: string; sessions: number }
  | { type: "page"; label: string; to: string };
