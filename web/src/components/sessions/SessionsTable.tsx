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
        className="text-accent-cyan mono text-xs hover:underline"
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
        <div className="text-text-primary text-sm truncate max-w-xs">
          {row.original.first_message ?? <span className="text-text-muted italic">No message</span>}
        </div>
        <div className="text-text-muted text-xs">{row.original.project_name}</div>
      </div>
    ),
  },
  {
    accessorKey: "total_tokens",
    header: "Tokens",
    cell: ({ getValue }) => (
      <span className="text-accent-green mono text-sm">{formatTokens(getValue() as number)}</span>
    ),
    size: 80,
  },
  {
    accessorKey: "estimated_cost",
    header: "Cost",
    cell: ({ getValue }) => (
      <span className="text-accent-yellow mono text-sm">{formatCost(getValue() as number)}</span>
    ),
    size: 80,
  },
  {
    accessorKey: "error_count",
    header: "Errors",
    cell: ({ getValue }) => {
      const v = getValue() as number;
      return v > 0 ? (
        <span className="text-accent-red mono text-xs">{v}</span>
      ) : (
        <span className="text-text-muted text-xs">—</span>
      );
    },
    size: 60,
  },
  {
    accessorKey: "modified_at",
    header: "Last active",
    cell: ({ getValue }) => (
      <span className="text-text-muted text-xs">{timeAgo(getValue() as number)}</span>
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
      <table className="w-full text-sm border-collapse">
        <thead>
          {table.getHeaderGroups().map((hg) => (
            <tr key={hg.id} style={{ borderBottom: "1px solid rgba(255,255,255,0.06)" }}>
              {hg.headers.map((header) => (
                <th
                  key={header.id}
                  className="text-left px-4 py-3 text-text-muted text-xs font-medium uppercase tracking-wider cursor-pointer select-none hover:text-text-primary transition-colors"
                  style={{ width: header.getSize() }}
                  onClick={header.column.getToggleSortingHandler()}
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
              style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }}
            >
              {row.getVisibleCells().map((cell) => (
                <td
                  key={cell.id}
                  className="px-4 py-3 group-hover:bg-white/[0.02]"
                >
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {sessions.length === 0 && (
        <div className="text-center py-16 text-text-muted">No sessions found</div>
      )}
    </div>
  );
}
