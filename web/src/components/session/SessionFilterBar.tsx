import { FilterChips } from "@/components/ui/FilterChips";
import type { SessionFilterBarProps } from "./types";
import { FILTER_OPTIONS } from "./config";

export function SessionFilterBar({
  filterTypes,
  onToggleType,
  filterKeyword,
  onKeywordChange,
  autoScroll,
  onToggleAutoScroll,
  activeFilePaths,
  onRemoveFilePath,
  onClearFilePaths,
  viewMode = "list",
  onViewModeChange,
  sortOrder = "newest",
  onSortOrderChange,
  filteredCount,
  totalCount,
}: SessionFilterBarProps) {
  return (
    <div style={{
      flexShrink: 0,
      padding: "4px 0",
      display: "flex", flexDirection: "column", gap: "4px",
    }}>
      <div style={{
        display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap",
      }}>
        <FilterChips
          options={FILTER_OPTIONS}
          active={filterTypes}
          onToggle={onToggleType}
        />

        <div style={{ flex: 1, minWidth: "120px", maxWidth: "280px" }}>
          <div style={{ position: "relative" }}>
            <input
              type="text"
              value={filterKeyword}
              onChange={(e) => onKeywordChange(e.target.value)}
              placeholder="Search messages..."
              style={{
                width: "100%",
                fontFamily: "var(--font-mono)", fontSize: "11px",
                padding: "5px 28px 5px 10px",
                borderRadius: "12px",
                border: "1px solid var(--border-2)",
                background: "var(--bg-2)",
                color: "var(--text-2)", outline: "none",
                caretColor: "var(--indigo)",
                transition: "border-color 0.12s",
              }}
              onFocus={(e) => { e.currentTarget.style.borderColor = "var(--indigo)"; }}
              onBlur={(e) => { e.currentTarget.style.borderColor = "var(--border-2)"; }}
            />
            {filterKeyword && (
              <button
                onClick={() => onKeywordChange("")}
                style={{
                  position: "absolute", right: "6px", top: "50%", transform: "translateY(-50%)",
                  background: "none", border: "none", cursor: "pointer",
                  color: "var(--text-3)", fontSize: "12px", padding: "2px",
                  lineHeight: 1,
                }}
              >
                &times;
              </button>
            )}
          </div>
        </div>

        <button
          onClick={onToggleAutoScroll}
          style={{
            fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
            letterSpacing: "0.05em",
            padding: "4px 12px", borderRadius: "12px",
            border: autoScroll ? "1px solid var(--indigo)" : "1px solid var(--border-2)",
            background: autoScroll ? "rgba(129, 140, 248, 0.10)" : "transparent",
            color: autoScroll ? "var(--indigo)" : "var(--text-3)",
            cursor: "pointer", transition: "all 0.12s",
          }}
        >
          AUTO
        </button>

        {/* View mode toggle */}
        {onViewModeChange && (
          <div style={{ display: "flex", borderRadius: "12px", border: "1px solid var(--border-2)", overflow: "hidden" }}>
            {(["list", "graph"] as const).map((mode) => (
              <button
                key={mode}
                onClick={() => onViewModeChange(mode)}
                style={{
                  fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
                  letterSpacing: "0.05em",
                  padding: "4px 10px",
                  border: "none",
                  background: viewMode === mode ? "rgba(129, 140, 248, 0.10)" : "transparent",
                  color: viewMode === mode ? "var(--indigo)" : "var(--text-3)",
                  cursor: "pointer", transition: "all 0.12s",
                  textTransform: "uppercase",
                }}
              >
                {mode.toUpperCase()}
              </button>
            ))}
          </div>
        )}

        {/* Sort order toggle */}
        {onSortOrderChange && viewMode === "list" && (
          <button
            onClick={() => onSortOrderChange(sortOrder === "newest" ? "oldest" : "newest")}
            title={sortOrder === "newest" ? "Showing newest first" : "Showing oldest first"}
            style={{
              fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
              letterSpacing: "0.05em",
              padding: "4px 12px", borderRadius: "12px",
              border: "1px solid var(--border-2)",
              background: "transparent",
              color: "var(--text-3)",
              cursor: "pointer", transition: "all 0.12s",
              display: "flex", alignItems: "center", gap: "4px",
            }}
          >
            {sortOrder === "newest" ? "NEWEST \u2193" : "OLDEST \u2191"}
          </button>
        )}

        {/* Node count */}
        {totalCount != null && totalCount > 0 && (
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "10px",
            color: filteredCount != null && filteredCount < totalCount ? "var(--indigo)" : "var(--text-3)",
            fontVariantNumeric: "tabular-nums",
            marginLeft: "auto",
          }}>
            {filteredCount != null && filteredCount < totalCount
              ? `${filteredCount} / ${totalCount}`
              : totalCount} nodes
          </span>
        )}
      </div>

      {/* File path filter chips */}
      {activeFilePaths.size > 0 && (
        <div style={{ display: "flex", gap: "4px", flexWrap: "wrap", alignItems: "center" }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: "9px", color: "var(--text-3)",
            textTransform: "uppercase", letterSpacing: "0.08em",
          }}>
            Paths:
          </span>
          {[...activeFilePaths].map((path) => (
            <button
              key={path}
              onClick={() => onRemoveFilePath(path)}
              title={`Remove filter: ${path}`}
              style={{
                fontFamily: "var(--font-mono)", fontSize: "10px",
                padding: "2px 8px", borderRadius: "10px",
                border: "1px solid color-mix(in srgb, var(--emerald) 25%, transparent)",
                background: "color-mix(in srgb, var(--emerald) 8%, transparent)",
                color: "var(--emerald)", cursor: "pointer",
                transition: "all 0.12s", maxWidth: "200px",
                overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
              }}
            >
              {path.split("/").pop()} &times;
            </button>
          ))}
          <button
            onClick={onClearFilePaths}
            style={{
              fontFamily: "var(--font-mono)", fontSize: "9px",
              padding: "2px 6px", borderRadius: "10px",
              border: "1px solid var(--border-1)", background: "transparent",
              color: "var(--text-3)", cursor: "pointer",
            }}
          >
            Clear
          </button>
        </div>
      )}
    </div>
  );
}
