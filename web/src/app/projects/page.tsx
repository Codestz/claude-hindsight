"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  type ColumnDef,
  type SortingState,
  flexRender,
} from "@tanstack/react-table";
import { api } from "@/lib/api";
import type { ProjectStats } from "@/lib/types";
import { formatBytes, timeAgo } from "@/lib/utils";
import { Header } from "@/components/layout/Header";

const columns: ColumnDef<ProjectStats>[] = [
  {
    accessorKey: "project_name",
    header: "Project",
    cell: ({ row }) => (
      <Link
        href={`/projects/detail?name=${encodeURIComponent(row.original.project_name)}`}
        className="hover:underline mono text-sm"
        style={{ color: "var(--accent)", fontWeight: 600 }}
      >
        {row.original.project_name}
      </Link>
    ),
  },
  {
    accessorKey: "session_count",
    header: "Sessions",
    cell: ({ getValue }) => (
      <span className="mono tabular" style={{ color: "var(--text-1)" }}>{(getValue() as number).toLocaleString()}</span>
    ),
    size: 90,
  },
  {
    accessorKey: "total_size",
    header: "Size",
    cell: ({ getValue }) => (
      <span className="mono text-sm tabular" style={{ color: "var(--text-2)" }}>{formatBytes(getValue() as number)}</span>
    ),
    size: 90,
  },
  {
    accessorKey: "last_activity",
    header: "Last active",
    cell: ({ getValue }) => {
      const v = getValue() as number | null;
      return v ? (
        <span className="mono text-sm" style={{ color: "var(--text-2)" }}>{timeAgo(v)}</span>
      ) : (
        <span className="mono text-xs" style={{ color: "var(--text-3)" }}>—</span>
      );
    },
    size: 110,
  },
];

export default function ProjectsPage() {
  const [projects, setProjects] = useState<ProjectStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [sorting, setSorting] = useState<SortingState>([{ id: "last_activity", desc: true }]);

  useEffect(() => {
    api.projects()
      .then(setProjects)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const table = useReactTable({
    data: projects,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  return (
    <div className="flex flex-col flex-1">
      <Header
        title="Projects"
        subtitle={`${projects.length} project${projects.length !== 1 ? "s" : ""}`}
      />

      <div className="flex-1 p-4">
        {/* Section header */}
        <div className="flex items-center gap-4 mb-4">
          <span className="label" style={{ color: "var(--accent)", fontSize: "10px" }}>PROJECTS</span>
          <div style={{ flex: 1, height: "1px", background: "var(--border)" }} />
        </div>

        <div style={{ background: "var(--border)" }}>
          {loading ? (
            <div className="mono text-center py-16 animate-pulse" style={{ color: "var(--text-3)", background: "var(--bg)" }}>Loading projects…</div>
          ) : (
            <table className="w-full text-sm" style={{ borderCollapse: "collapse", background: "var(--bg)" }}>
              <thead>
                {table.getHeaderGroups().map((hg) => (
                  <tr key={hg.id} style={{ borderBottom: "1px solid var(--border)" }}>
                    {hg.headers.map((header) => (
                      <th
                        key={header.id}
                        className="text-left px-5 py-2 mono cursor-pointer select-none"
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
                      <td key={cell.id} className="px-5 py-3">
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {!loading && projects.length === 0 && (
            <div className="text-center py-16" style={{ background: "var(--bg)" }}>
              <div className="mono text-sm mb-2" style={{ color: "var(--text-2)" }}>No projects indexed yet</div>
              <div className="mono text-xs" style={{ color: "var(--text-3)" }}>Run hindsight init to discover sessions</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
