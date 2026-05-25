import { useEffect, useRef, useCallback, useState } from 'react';
import { cn } from '../../lib/utils';
import { useFocusTrap } from '../../hooks/useKeyboardShortcuts';
import {
  useToast,
  ToastContext,
} from '../../hooks/useToast';
import type {
  Toast,
  ToastInput,
  ToastType,
  ToastContextValue,
} from '../../hooks/useToast';

// ── Toast Provider ─────────────────────────────────────────────────────────

let toastCounter = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((input: ToastInput) => {
    const id = `toast-${++toastCounter}-${Date.now()}`;
    const newToast: Toast = { ...input, id };

    setToasts((prev) => [...prev, newToast]);

    const duration = input.duration ?? 4000;
    if (duration > 0) {
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, duration);
    }

    return id;
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const clearToasts = useCallback(() => {
    setToasts([]);
  }, []);

  const contextValue: ToastContextValue = { toasts, addToast, removeToast, clearToasts };

  return (
    <ToastContext.Provider value={contextValue}>
      {children}
    </ToastContext.Provider>
  );
}

// ── Toast Container ────────────────────────────────────────────────────────
const toastStyles: Record<ToastType, { bg: string; icon: string; border: string }> = {
  success: { bg: 'bg-cyber-green/10', icon: '✓', border: 'border-cyber-green/30' },
  error: { bg: 'bg-red-500/10', icon: '✕', border: 'border-red-500/30' },
  warning: { bg: 'bg-amber-500/10', icon: '⚠', border: 'border-amber-500/30' },
  info: { bg: 'bg-cyber-purple/10', icon: 'ℹ', border: 'border-cyber-purple/30' },
};
function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: (id: string) => void }) {
  const style = toastStyles[toast.type];

  return (
    <div
      className={cn(
        'flex items-start gap-3 p-3 rounded-lg border backdrop-blur-xl shadow-lg',
        'animate-slide-up min-w-[280px] max-w-[380px]',
        style.bg,
        style.border
      )}
      role="alert"
      aria-live="polite"
    >
      <span className="text-sm mt-0.5" aria-hidden="true">{style.icon}</span>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-semibold font-mono text-slate-200">{toast.title}</p>
        {toast.message && (
          <p className="text-[10px] font-mono text-slate-400 mt-0.5 leading-relaxed">{toast.message}</p>
        )}
      </div>
      <button
        onClick={() => onDismiss(toast.id)}
        className="text-slate-500 hover:text-slate-300 transition-colors text-xs shrink-0"
        aria-label="Dismiss notification"
      >
        ✕
      </button>
    </div>
  );
}

export function ToastContainer() {
  const { toasts, removeToast } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed top-4 right-4 z-[9999] flex flex-col gap-2 pointer-events-none"
      aria-label="Notifications"
    >
      {toasts.map((toast) => (
        <div key={toast.id} className="pointer-events-auto">
          <ToastItem toast={toast} onDismiss={removeToast} />
        </div>
      ))}
    </div>
  );
}

// ── Modal ──────────────────────────────────────────────────────────────────

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  className?: string;
}

export function Modal({ open, onClose, title, children, className }: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);
  useFocusTrap(modalRef, open);

  useEffect(() => {
    if (open) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [open]);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) onClose();
    },
    [onClose]
  );

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && open) onClose();
    };
    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-[9998] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in"
      onClick={handleBackdropClick}
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <div
        ref={modalRef}
        className={cn(
          'glass-panel rounded-xl max-w-lg w-full mx-4 max-h-[85vh] flex flex-col animate-scale-in',
          className
        )}
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-cyber-border/40">
          <h2 className="text-sm font-bold font-mono text-slate-200">{title}</h2>
          <button
            onClick={onClose}
            className="btn-icon"
            aria-label="Close dialog"
          >
            ✕
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-5 scrollbar-cyber">
          {children}
        </div>
      </div>
    </div>
  );
}
