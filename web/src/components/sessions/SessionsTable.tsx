"use client";

import { useState } from "react";
import Link from "next/link";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  type ColumnDef,
  type SortingState,
  flexRender,
} from "@tanstack/react-table";
import type { SessionFile } from "@/lib/types";
import { formatCost, formatTokens, shortId, timeAgo } from "@/lib/utils";

interface SessionsTableProps {
  sessions: SessionFile[];
}

const columns: ColumnDef<SessionFile>[] = [
  {
    accessorKey: "session_id",
    header: "ID",
    cell: ({ row }) => (
      <Link
        href={`/sessions?id=${row.original.session_id}`}
        className="mono text-xs hover:underline"
        style={{ color: "var(--accent)" }}
      >
        {shortId(row.original.session_id)}
      </Link>
    ),
    size: 90,
  },
  {
    accessorKey: "first_message",
    header: "Session",
    cell: ({ row }) => (
      <div>
        <div className="text-sm truncate max-w-xs" style={{ color: "var(--text-1)" }}>
          {row.original.first_message ?? <span className="italic" style={{ color: "var(--text-3)" }}>No message</span>}
        </div>
        <div className="mono text-xs" style={{ color: "var(--text-3)" }}>{row.original.project_name}</div>
      </div>
    ),
  },
  {
    accessorKey: "total_tokens",
    header: "Tokens",
    cell: ({ getValue }) => (
      <span className="mono text-sm tabular" style={{ color: "var(--cyan)" }}>{formatTokens(getValue() as number)}</span>
    ),
    size: 80,
  },
  {
    accessorKey: "estimated_cost",
    header: "Cost",
    cell: ({ getValue }) => (
      <span className="mono text-sm tabular" style={{ color: "var(--amber)" }}>{formatCost(getValue() as number)}</span>
    ),
    size: 80,
  },
  {
    accessorKey: "error_count",
    header: "Errors",
    cell: ({ getValue }) => {
      const v = getValue() as number;
      return v > 0 ? (
        <span className="tbadge tbadge-err">{v}</span>
      ) : (
        <span className="mono text-xs" style={{ color: "var(--text-3)" }}>—</span>
      );
    },
    size: 60,
  },
  {
    accessorKey: "modified_at",
    header: "Last active",
    cell: ({ getValue }) => (
      <span className="mono text-xs" style={{ color: "var(--text-3)" }}>{timeAgo(getValue() as number)}</span>
    ),
    size: 100,
  },
];

export function SessionsTable({ sessions }: SessionsTableProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: "modified_at", desc: true },
  ]);

  const table = useReactTable({
    data: sessions,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  return (
    <div className="overflow-auto">
      <table className="w-full text-sm" style={{ borderCollapse: "collapse" }}>
        <thead>
          {table.getHeaderGroups().map((hg) => (
            <tr key={hg.id} style={{ borderBottom: "1px solid var(--border)" }}>
              {hg.headers.map((header) => (
                <th
                  key={header.id}
                  className="text-left px-4 py-2 cursor-pointer select-none mono"
                  style={{
                    width: header.getSize(),
                    color: "var(--text-3)",
                    fontSize: "9px",
                    textTransform: "uppercase",
                    letterSpacing: "0.12em",
                    fontWeight: 700,
                  }}
                  onClick={header.column.getToggleSortingHandler()}
                  onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-2)"; }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.color = "var(--text-3)"; }}
                >
                  <div className="flex items-center gap-1">
                    {flexRender(header.column.columnDef.header, header.getContext())}
                    {header.column.getIsSorted() === "asc" && " ↑"}
                    {header.column.getIsSorted() === "desc" && " ↓"}
                  </div>
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr
              key={row.id}
              className="group transition-colors"
              style={{ borderBottom: "1px solid var(--border)" }}
              onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--bg-2)"; }}
              onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
            >
              {row.getVisibleCells().map((cell) => (
                <td
                  key={cell.id}
                  className="px-4 py-3"
                  style={{ color: "var(--text-2)", fontSize: "13px" }}
                >
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {sessions.length === 0 && (
        <div className="text-center py-16 mono" style={{ color: "var(--text-3)" }}>No sessions found</div>
      )}
    </div>
  );
}
