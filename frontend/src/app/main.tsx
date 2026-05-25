import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';
import { ToastProvider } from '../components/ui/Toast';
import './index.css';

// --- Capacitor dynamic API interceptor monkey-patch ---
try {
  const originalFetch = window.fetch;
  window.fetch = function (input, init) {
    if (typeof input === 'string' && input.startsWith('/api/')) {
      try {
        const stored = localStorage.getItem('tredo_settings_api_base_url');
        if (stored) {
          const baseUrl = JSON.parse(stored);
          if (baseUrl) {
            const cleanBase = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
            input = `${cleanBase}${input}`;
          }
        }
      } catch (err) {
        console.warn('[Fetch Rewriter] Failed to parse backend API base URL:', err);
      }
    } else if (input instanceof Request && input.url.startsWith('/api/')) {
      try {
        const stored = localStorage.getItem('tredo_settings_api_base_url');
        if (stored) {
          const baseUrl = JSON.parse(stored);
          if (baseUrl) {
            const cleanBase = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
            const newUrl = `${cleanBase}${input.url}`;
            input = new Request(newUrl, input);
          }
        }
      } catch (err) {
        console.warn('[Fetch Rewriter] Failed to parse backend API base URL:', err);
      }
    }
    return originalFetch(input, init);
  };
} catch (e) {
  console.error('[Fetch Rewriter] Initialization error:', e);
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ToastProvider>
      <App />
    </ToastProvider>
  </React.StrictMode>
);
