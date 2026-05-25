import { useState, useCallback, useRef, useEffect } from 'react';

interface UseApiFetchResult<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  reset: () => void;
}

interface UseApiFetchOptions {
  /** Auto-fetch on mount */
  immediate?: boolean;
  /** Callback on success */
  onSuccess?: (data: any) => void;
  /** Callback on error */
  onError?: (error: string) => void;
}

/**
 * Generic API fetch hook with comprehensive state management.
 * Handles loading, error, and data states consistently across the app.
 */
export function useApiFetch<T = any>(
  url: string,
  options: UseApiFetchOptions = {}
): UseApiFetchResult<T> {
  const { immediate = true, onSuccess, onError } = options;
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(immediate);
  const [error, setError] = useState<string | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const fetchData = useCallback(async () => {
    // Cancel previous request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    const controller = new AbortController();
    abortControllerRef.current = controller;

    setLoading(true);
    setError(null);

    try {
      const response = await fetch(url, {
        signal: controller.signal,
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      
      if (!controller.signal.aborted) {
        setData(result);
        onSuccess?.(result);
      }
    } catch (err: any) {
      if (err.name === 'AbortError') return;
      
      const message = err.message || 'An unexpected error occurred';
      if (!controller.signal.aborted) {
        setError(message);
        onError?.(message);
      }
    } finally {
      if (!controller.signal.aborted) {
        setLoading(false);
      }
    }
  }, [url, onSuccess, onError]);

  const refetch = useCallback(async () => {
    await fetchData();
  }, [fetchData]);

  const reset = useCallback(() => {
    setData(null);
    setLoading(false);
    setError(null);
  }, []);

  useEffect(() => {
    if (immediate) {
      fetchData();
    }
    return () => {
      abortControllerRef.current?.abort();
    };
  }, [immediate, fetchData]);

  return { data, loading, error, refetch, reset };
}

/**
 * Mutation hook for POST/PUT/DELETE operations
 */
export function useApiMutation<T = any>() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const mutate = useCallback(async (
    url: string,
    method: 'POST' | 'PUT' | 'DELETE' = 'POST',
    body?: any
  ): Promise<T | null> => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: body ? JSON.stringify(body) : undefined,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      return result as T;
    } catch (err: any) {
      const message = err.message || 'An unexpected error occurred';
      setError(message);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  return { mutate, loading, error };
}

/** Check backend health */
export function useBackendHealth() {
  return useApiFetch<{ status: string }>('/api/health', { immediate: false });
}
