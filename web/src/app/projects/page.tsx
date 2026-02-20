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
        className="text-accent-cyan hover:underline font-medium"
      >
        {row.original.project_name}
      </Link>
    ),
  },
  {
    accessorKey: "session_count",
    header: "Sessions",
    cell: ({ getValue }) => (
      <span className="text-text-primary mono">{(getValue() as number).toLocaleString()}</span>
    ),
    size: 90,
  },
  {
    accessorKey: "total_size",
    header: "Size",
    cell: ({ getValue }) => (
      <span className="text-text-muted mono text-sm">{formatBytes(getValue() as number)}</span>
    ),
    size: 90,
  },
  {
    accessorKey: "last_activity",
    header: "Last active",
    cell: ({ getValue }) => {
      const v = getValue() as number | null;
      return v ? (
        <span className="text-text-muted text-sm">{timeAgo(v)}</span>
      ) : (
        <span className="text-text-muted text-xs">—</span>
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

      <div className="flex-1 p-6">
        <div className="card rounded-lg overflow-hidden">
          {loading ? (
            <div className="text-center py-16 text-text-muted animate-pulse">Loading projects…</div>
          ) : (
            <table className="w-full text-sm border-collapse">
              <thead>
                {table.getHeaderGroups().map((hg) => (
                  <tr key={hg.id} style={{ borderBottom: "1px solid rgba(255,255,255,0.06)" }}>
                    {hg.headers.map((header) => (
                      <th
                        key={header.id}
                        className="text-left px-5 py-3 text-text-muted text-xs font-medium uppercase tracking-wider cursor-pointer select-none hover:text-text-primary transition-colors"
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
                      <td key={cell.id} className="px-5 py-3 group-hover:bg-white/[0.02]">
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {!loading && projects.length === 0 && (
            <div className="text-center py-16">
              <div className="text-text-muted text-sm mb-2">No projects indexed yet</div>
              <div className="text-text-muted text-xs">Run hindsight init to discover sessions</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
