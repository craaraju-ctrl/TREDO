import { useCallback } from 'react';
import { cn } from '../../lib/utils';
import { MODULE_TABS } from '../../lib/constants';
import type { ModuleTab } from '../../lib/constants';

interface NavigationProps {
  activeTab: ModuleTab;
  onTabChange: (tab: ModuleTab) => void;
}

export function Navigation({ activeTab, onTabChange }: NavigationProps) {
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, tabId: ModuleTab) => {
      const currentIndex = MODULE_TABS.findIndex((t) => t.id === activeTab);
      let nextIndex: number | null = null;

      if (e.key === 'ArrowRight') {
        nextIndex = (currentIndex + 1) % MODULE_TABS.length;
      } else if (e.key === 'ArrowLeft') {
        nextIndex = (currentIndex - 1 + MODULE_TABS.length) % MODULE_TABS.length;
      }

      if (nextIndex !== null) {
        e.preventDefault();
        onTabChange(MODULE_TABS[nextIndex].id);
      } else if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        onTabChange(tabId);
      }
    },
    [activeTab, onTabChange]
  );

  return (
    <nav
      className="flex justify-start px-6 py-2 bg-cyber-dark/80 border-b border-cyber-border/30 z-10 shrink-0"
      role="tablist"
      aria-label="Main navigation"
    >
      {MODULE_TABS.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          onKeyDown={(e) => handleKeyDown(e, tab.id)}
          role="tab"
          aria-selected={activeTab === tab.id}
          aria-controls={`panel-${tab.id}`}
          tabIndex={activeTab === tab.id ? 0 : -1}
          title={`${tab.label} — ${tab.description}${tab.id === 'Chat' ? '' : ''}`}
          className={cn(
            'px-5 py-2.5 text-sm font-semibold font-mono rounded-t-lg transition-all duration-300',
            'hover:text-slate-100 focus-visible:ring-2 focus-visible:ring-cyber-purple focus-visible:ring-inset',
            activeTab === tab.id
              ? 'text-cyber-purple bg-cyber-panel/70 border-t border-x border-cyber-border/50 shadow-sm'
              : 'text-slate-400 hover:text-slate-200 border-transparent'
          )}
        >
          <span className="hidden sm:inline mr-2" aria-hidden="true">{tab.icon}</span>
          {tab.label}
        </button>
      ))}
    </nav>
  );
}
