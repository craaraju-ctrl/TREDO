import { useEffect, useState } from 'react';
import { cn } from '../../lib/utils';

interface BadgeProps {
  variant?: 'success' | 'danger' | 'warning' | 'info' | 'neutral';
  children: React.ReactNode;
  className?: string;
  title?: string;
}

const variantStyles = {
  success: 'bg-cyber-green/10 text-cyber-green border-cyber-green/30',
  danger: 'bg-red-500/10 text-red-400 border-red-500/30',
  warning: 'bg-amber-500/10 text-amber-400 border-amber-500/30',
  info: 'bg-cyber-purple/10 text-cyber-purple border-cyber-purple/30',
  neutral: 'bg-slate-500/10 text-slate-400 border-slate-500/30',
};

export function Badge({ variant = 'neutral', children, className, title }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold font-mono border',
        variantStyles[variant],
        className
      )}
      title={title}
    >
      {children}
    </span>
  );
}

interface ProgressBarProps {
  value: number; // 0-100
  variant?: 'success' | 'danger' | 'warning' | 'info' | 'gradient';
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

const barVariants = {
  success: 'bg-cyber-green',
  danger: 'bg-red-500',
  warning: 'bg-amber-400',
  info: 'bg-cyber-purple',
  gradient: 'bg-gradient-to-r from-cyber-purple to-cyber-green',
};

const barSizes = { sm: 'h-1', md: 'h-1.5', lg: 'h-2' };

export function ProgressBar({
  value,
  variant = 'gradient',
  size = 'md',
  showLabel = false,
  className,
}: ProgressBarProps) {
  const clampedValue = Math.min(100, Math.max(0, value));
  const [width, setWidth] = useState(0);

  useEffect(() => {
    // Animate in
    const timer = setTimeout(() => setWidth(clampedValue), 50);
    return () => clearTimeout(timer);
  }, [clampedValue]);

  return (
    <div className={cn('flex items-center gap-2', className)}>
      <div className={cn('progress-bar flex-1', barSizes[size])} role="progressbar" aria-valuenow={clampedValue} aria-valuemin={0} aria-valuemax={100}>
        <div
          className={cn('progress-bar-fill', barVariants[variant])}
          style={{ width: `${width}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-[9px] font-mono text-slate-400 tabular-nums">{clampedValue.toFixed(0)}%</span>
      )}
    </div>
  );
}

interface StatusDotProps {
  status: 'active' | 'inactive' | 'warning' | 'danger';
  label?: string;
  className?: string;
}

const dotVariants = {
  active: 'status-dot-active',
  inactive: 'status-dot-inactive',
  warning: 'status-dot-warning',
  danger: 'status-dot-danger',
};

export function StatusDot({ status, label, className }: StatusDotProps) {
  return (
    <span className={cn('inline-flex items-center gap-1.5', className)} aria-label={label || status}>
      <span className={dotVariants[status]} />
      {label && <span className="text-[10px] font-mono text-slate-400">{label}</span>}
    </span>
  );
}
