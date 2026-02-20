import { Sidebar } from "./Sidebar";

interface ShellProps {
  children: React.ReactNode;
}

export function Shell({ children }: ShellProps) {
  return (
    <div className="min-h-screen" style={{ background: "var(--bg-base)" }}>
      <Sidebar />
      <main className="ml-52 min-h-screen flex flex-col">{children}</main>
    </div>
  );
}
