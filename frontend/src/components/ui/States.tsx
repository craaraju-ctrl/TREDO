import { cn } from '../../lib/utils';

interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export function EmptyState({ icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center py-12 px-6 text-center',
        className
      )}
      role="status"
    >
      <div className="relative mb-3">
        <div className="absolute inset-0 bg-cyber-purple/5 rounded-full blur-xl" />
        {icon && <span className="text-3xl relative" aria-hidden="true">{icon}</span>}
      </div>
      <h3 className="text-sm font-semibold font-mono text-slate-400 mb-1">{title}</h3>
      {description && (
        <p className="text-[10px] font-mono text-slate-500 max-w-xs leading-relaxed">
          {description}
        </p>
      )}
      {action && (
        <button
          onClick={action.onClick}
          className="btn-primary mt-4"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
  className?: string;
}

export function ErrorState({
  title = 'Something went wrong',
  message,
  onRetry,
  className,
}: ErrorStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center py-12 px-6 text-center',
        className
      )}
      role="alert"
    >
      <span className="text-3xl mb-3" aria-hidden="true">⚠️</span>
      <h3 className="text-sm font-semibold font-mono text-red-400 mb-1">{title}</h3>
      {message && (
        <p className="text-[10px] font-mono text-slate-500 max-w-xs leading-relaxed mb-4">
          {message}
        </p>
      )}
      {onRetry && (
        <button onClick={onRetry} className="btn-secondary">
          Try Again
        </button>
      )}
    </div>
  );
}
