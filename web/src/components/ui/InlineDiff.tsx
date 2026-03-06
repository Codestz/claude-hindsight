type DiffLine = { type: "same" | "removed" | "added"; line: string };

/** LCS-based line diff. O(n²) — fine for small skill/agent bodies. */
function computeDiff(a: string, b: string): DiffLine[] {
  const la = a.split("\n");
  const lb = b.split("\n");
  const m = la.length;
  const n = lb.length;

  // Build LCS length table
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (la[i - 1] === lb[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Trace back to produce diff
  const result: DiffLine[] = [];
  let i = m, j = n;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && la[i - 1] === lb[j - 1]) {
      result.push({ type: "same", line: la[i - 1] });
      i--; j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      result.push({ type: "added", line: lb[j - 1] });
      j--;
    } else {
      result.push({ type: "removed", line: la[i - 1] });
      i--;
    }
  }
  result.reverse();
  return result;
}

interface SideSpec {
  label: string;
  body: string;
}

interface InlineDiffProps {
  left: SideSpec;
  right: SideSpec;
}

export function InlineDiff({ left, right }: InlineDiffProps) {
  const lines = computeDiff(left.body, right.body);

  return (
    <div
      style={{
        border: "1px solid var(--border-1)",
        borderRadius: "var(--radius-md)",
        overflow: "hidden",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
      }}
    >
      {/* Column headers */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          borderBottom: "1px solid var(--border-1)",
        }}
      >
        <div
          style={{
            padding: "6px 12px",
            color: "var(--text-3)",
            fontSize: "11px",
            borderRight: "1px solid var(--border-1)",
          }}
        >
          {left.label}
        </div>
        <div style={{ padding: "6px 12px", color: "var(--text-3)", fontSize: "11px" }}>
          {right.label}
        </div>
      </div>

      {/* Diff lines */}
      <div style={{ maxHeight: "400px", overflowY: "auto" }}>
        {lines.map((line, idx) => {
          const bg =
            line.type === "removed"
              ? "color-mix(in srgb, var(--red, #f87171) 10%, transparent)"
              : line.type === "added"
              ? "color-mix(in srgb, var(--green) 10%, transparent)"
              : "transparent";
          const color =
            line.type === "removed"
              ? "var(--red, #f87171)"
              : line.type === "added"
              ? "var(--green)"
              : "var(--text-3)";
          const prefix =
            line.type === "removed" ? "−" : line.type === "added" ? "+" : " ";

          // For same lines, show in both columns; for removed only left, for added only right
          return (
            <div
              key={idx}
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                background: bg,
              }}
            >
              {/* Left cell */}
              <div
                style={{
                  padding: "1px 12px",
                  color,
                  whiteSpace: "pre",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  borderRight: "1px solid var(--border-1)",
                }}
              >
                {line.type !== "added" ? `${prefix} ${line.line}` : ""}
              </div>
              {/* Right cell */}
              <div
                style={{
                  padding: "1px 12px",
                  color,
                  whiteSpace: "pre",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                {line.type !== "removed" ? `${prefix} ${line.line}` : ""}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
