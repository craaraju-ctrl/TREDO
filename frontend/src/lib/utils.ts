import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/** Merge Tailwind classes with conflict resolution */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Format a number as currency */
export function formatCurrency(value: number, decimals = 2): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(value);
}

/** Format a percentage */
export function formatPercent(value: number, decimals = 1): string {
  return `${value >= 0 ? '+' : ''}${value.toFixed(decimals)}%`;
}

/** Format a number with compact notation (e.g., 1.5K, 2.3M) */
export function formatCompact(value: number): string {
  return new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
  }).format(value);
}

/** Format a timestamp to a time string */
export function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

/** Format a timestamp to a date string */
export function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/** Clamp a number between min and max */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/** Generate a unique ID */
export function generateId(): string {
  return Math.random().toString(36).substring(2, 11);
}

/** Get a color class based on a numeric value (positive = green, negative = red) */
export function getChangeColor(value: number): string {
  if (value > 0) return 'text-cyber-green';
  if (value < 0) return 'text-red-400';
  return 'text-slate-400';
}

/** Get a background class for direction badges */
export function getDirectionBg(direction: string): string {
  switch (direction) {
    case 'Bullish': return 'bg-cyber-green/10 text-cyber-green border-cyber-green/30';
    case 'Bearish': return 'bg-red-500/10 text-red-400 border-red-500/30';
    default: return 'bg-slate-500/10 text-slate-400 border-slate-500/30';
  }
}
