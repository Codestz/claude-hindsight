/**
 * Shared primitive components used by all tool displays.
 */

import type { TokenUsage } from "@/lib/types";

/** Labeled section wrapper with uppercase mono label. */
export function ContentSection({
  label, color, children,
}: { label: string; color?: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: "24px" }}>
      <div style={{
        fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 700,
        letterSpacing: "0.1em", textTransform: "uppercase",
        color: color ?? "var(--text-3)", marginBottom: "10px",
      }}>
        {label}
      </div>
      {children}
    </div>
  );
}

/** Token usage footer (input, output, cache write, cache read). */
export function TokenFooter({ usage }: { usage: TokenUsage }) {
  const items = [
    { label: "In",          value: usage.input_tokens },
    { label: "Out",         value: usage.output_tokens },
    { label: "Cache write", value: usage.cache_creation_input_tokens },
    { label: "Cache read",  value: usage.cache_read_input_tokens },
  ].filter((i) => i.value != null && i.value > 0);

  if (items.length === 0) return null;

  return (
    <div style={{
      display: "flex", gap: "20px", paddingTop: "18px",
      borderTop: "1px solid var(--border-1)", marginTop: "8px",
      background: "color-mix(in srgb, var(--sky) 3%, var(--bg-1))",
      margin: "8px -24px 0", padding: "18px 24px 0",
    }}>
      {items.map((item) => (
        <div key={item.label} style={{ display: "flex", gap: "6px", alignItems: "baseline" }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "11px",
            color: "var(--text-3)", textTransform: "uppercase", letterSpacing: "0.06em",
          }}>{item.label}</span>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "13px",
            fontVariantNumeric: "tabular-nums", color: "var(--cyan)",
          }}>{item.value!.toLocaleString()}</span>
        </div>
      ))}
    </div>
  );
}

/** Placeholder for empty tool results. */
export function EmptyResult({ label }: { label?: string }) {
  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "center",
      padding: "20px", gap: "8px",
      background: "var(--bg-2)", border: "1px solid var(--border-1)",
      borderRadius: "var(--radius-md)",
    }}>
      <span style={{ color: "var(--text-3)", fontSize: "16px" }}>&#8709;</span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "12px", color: "var(--text-3)", letterSpacing: "0.05em" }}>
        {label ?? "no result"}
      </span>
    </div>
  );
}
