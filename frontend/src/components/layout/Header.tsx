import { useAtom } from 'jotai';
import { metricsAtom, serverActiveAtom } from '../../atoms/state';
import { StatusDot } from '../ui/Badge';

export function Header() {
  const [metrics] = useAtom(metricsAtom);
  const [serverActive, setServerActive] = useAtom(serverActiveAtom);

  return (
    <header
      className="flex items-center justify-between px-6 py-3 glass-panel border-b border-cyber-border/60 shadow-lg z-20 shrink-0"
      role="banner"
    >
      {/* Logo & Title */}
      <div className="flex items-center gap-3">
        <div
          className="w-9 h-9 rounded-lg bg-gradient-to-br from-cyber-purple to-cyber-glow flex items-center justify-center font-bold text-white text-lg shadow-purple"
          aria-hidden="true"
        >
          A
        </div>
        <div>
          <h1 className="text-lg font-bold tracking-wider font-mono bg-gradient-to-r from-slate-100 via-slate-200 to-slate-400 bg-clip-text text-transparent">
            TREDO COCKPIT
          </h1>
          <p className="text-[9px] text-cyber-purple font-mono tracking-wider">Sethu Bridge Core v1.0.0</p>
        </div>
      </div>

      {/* Control Switch & Status Bar */}
      <div className="flex items-center gap-6 text-[10px] font-mono">
        {/* Cyberpunk Server Toggle Switch */}
        <div className="flex items-center gap-2 bg-slate-950/80 border border-cyber-border/50 rounded-lg p-1">
          <span className="text-[8px] text-slate-500 font-bold uppercase tracking-wider pl-1.5 pr-1">SERVER</span>
          <button
            onClick={() => setServerActive(!serverActive)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[9px] font-bold transition-all duration-300 ${
              serverActive
                ? 'bg-emerald-950/60 border border-emerald-500/50 text-emerald-400 shadow-[0_0_8px_rgba(16,185,129,0.2)]'
                : 'bg-rose-950/60 border border-rose-500/50 text-rose-400 shadow-[0_0_8px_rgba(244,63,94,0.2)]'
            }`}
            title={serverActive ? 'Shutdown Core Server' : 'Boot Core Server'}
          >
            <span className={`w-1.5 h-1.5 rounded-full ${serverActive ? 'bg-emerald-400 animate-ping' : 'bg-rose-500'}`} />
            <span className="font-mono tracking-wider text-[8px]">{serverActive ? 'ONLINE' : 'OFFLINE'}</span>
            <span className="text-[10px]">⏻</span>
          </button>
        </div>

        <StatusDot 
          status={serverActive ? 'active' : 'inactive'} 
          label={serverActive ? 'Sethu Link ACTIVE' : 'Sethu Link OFFLINE'} 
        />
        
        <div className="hidden md:flex items-center gap-4 border-l border-cyber-border/40 pl-6">
          <div className="flex items-center gap-1.5">
            <span className="text-slate-500">CPU</span>
            <span className={`font-semibold tabular-nums transition-colors duration-300 ${serverActive ? 'text-cyber-purple' : 'text-rose-500/60'}`}>
              {serverActive ? `${metrics.cpu}%` : '0.0%'}
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="text-slate-500">RAM</span>
            <span className={`font-semibold tabular-nums transition-colors duration-300 ${serverActive ? 'text-cyber-purple' : 'text-rose-500/60'}`}>
              {serverActive ? `${metrics.memory}%` : '0.0%'}
            </span>
          </div>
        </div>
      </div>
    </header>
  );
}
