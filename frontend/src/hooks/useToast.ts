import { useContext, createContext } from 'react';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

export interface ToastInput {
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

export interface ToastContextValue {
  toasts: Toast[];
  addToast: (toast: ToastInput) => string;
  removeToast: (id: string) => void;
  clearToasts: () => void;
}

export const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}

export function useToastActions() {
  const { addToast } = useToast();

  const success = (title: string, message?: string) => addToast({ type: 'success', title, message });

  const error = (title: string, message?: string) => addToast({ type: 'error', title, message });

  const warning = (title: string, message?: string) => addToast({ type: 'warning', title, message });

  const info = (title: string, message?: string) => addToast({ type: 'info', title, message });

  return { success, error, warning, info };
}
