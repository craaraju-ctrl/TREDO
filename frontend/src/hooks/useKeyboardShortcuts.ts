import { useEffect, useCallback } from 'react';
import type { ModuleTab } from '../lib/constants';
import { KEYBOARD_SHORTCUTS } from '../lib/constants';

interface ShortcutHandlers {
  onNavigate: (tab: ModuleTab) => void;
  onNewChat?: () => void;
  onClosePanel?: () => void;
}

/** Hook to register global keyboard shortcuts */
export function useKeyboardShortcuts({ onNavigate, onNewChat, onClosePanel }: ShortcutHandlers) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in an input
      const target = event.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable
      ) {
        // Allow Escape to work even in inputs
        if (event.key !== 'Escape') return;
      }

      const shortcut = KEYBOARD_SHORTCUTS[event.key as keyof typeof KEYBOARD_SHORTCUTS];
      if (!shortcut) return;

      event.preventDefault();

      if ('module' in shortcut && shortcut.module) {
        onNavigate(shortcut.module as ModuleTab);
      } else if ('action' in shortcut && shortcut.action === 'newChat' && onNewChat) {
        onNewChat();
      } else if ('action' in shortcut && shortcut.action === 'closePanel' && onClosePanel) {
        onClosePanel();
      }
    },
    [onNavigate, onNewChat, onClosePanel]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

/** Hook to manage focus trapping within a modal/dialog */
export function useFocusTrap(containerRef: React.RefObject<HTMLElement | null>, active: boolean) {
  useEffect(() => {
    if (!active || !containerRef.current) return;

    const container = containerRef.current;
    const focusableElements = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstFocusable = focusableElements[0];
    const lastFocusable = focusableElements[focusableElements.length - 1];

    // Focus the first element
    firstFocusable?.focus();

    const handleTabKey = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable?.focus();
        }
      } else {
        if (document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable?.focus();
        }
      }
    };

    document.addEventListener('keydown', handleTabKey);
    return () => document.removeEventListener('keydown', handleTabKey);
  }, [containerRef, active]);
}
