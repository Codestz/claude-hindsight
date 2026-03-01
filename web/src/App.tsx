import { Suspense, useState, useEffect } from 'react';
import { Sidebar, SIDEBAR_W, SIDEBAR_COLLAPSED_W, BREAKPOINT } from '@/components/layout/Sidebar';
import { CommandPalette } from '@/components/layout/CommandPalette';
import { AppRoutes } from './router';

export function App() {
  const [cmdOpen, setCmdOpen] = useState(false);
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    const check = () => setCollapsed(window.innerWidth < BREAKPOINT);
    check();
    window.addEventListener("resize", check);
    return () => window.removeEventListener("resize", check);
  }, []);

  // ⌘K / Ctrl+K shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCmdOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const marginLeft = collapsed ? SIDEBAR_COLLAPSED_W : SIDEBAR_W;

  return (
    <>
      <Sidebar onSearch={() => setCmdOpen(true)} />
      <main
        style={{
          marginLeft,
          minHeight: '100vh',
          transition: 'margin-left 0.2s ease',
        }}
      >
        <Suspense
          fallback={
            <div
              style={{
                height: '60vh',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '14px',
                color: 'var(--text-3)',
                fontFamily: 'var(--font-sans)',
              }}
            >
              Loading…
            </div>
          }
        >
          <AppRoutes />
        </Suspense>
      </main>
      <CommandPalette open={cmdOpen} onClose={() => setCmdOpen(false)} />
    </>
  );
}
