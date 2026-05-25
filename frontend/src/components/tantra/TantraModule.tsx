import { useState, useEffect } from 'react';
import { cn } from '../../lib/utils';

import { BabyAgent, DEFAULT_BABY_AGENTS } from '../../agents';

interface CommLog {
  id: string;
  timestamp: string;
  from: string;
  to: string;
  message: string;
  direction: 'in' | 'out' | 'system';
}

const INITIAL_COMM_LOGS: CommLog[] = [
  {
    id: 'c-1',
    timestamp: new Date(Date.now() - 30000).toLocaleTimeString(),
    from: 'Nethra Swarm',
    to: 'Risk Shield',
    message: 'Audit current portfolio margin allocation bounds for incoming BTC position sizing.',
    direction: 'out',
  },
  {
    id: 'c-2',
    timestamp: new Date(Date.now() - 28000).toLocaleTimeString(),
    from: 'Risk Shield',
    to: 'Nethra Swarm',
    message: 'Margin exposure assessed at 12.4%. Standard risk index is 0.38. VaR limits secure. Allocation approved.',
    direction: 'in',
  },
  {
    id: 'c-3',
    timestamp: new Date(Date.now() - 20000).toLocaleTimeString(),
    from: 'Nethra Swarm',
    to: 'News Sentinel',
    message: 'Process flash news regarding US Federal Reserve interest rate cut announcement.',
    direction: 'out',
  },
  {
    id: 'c-4',
    timestamp: new Date(Date.now() - 18000).toLocaleTimeString(),
    from: 'News Sentinel',
    to: 'Nethra Swarm',
    message: 'Analyzed rate cut bulletin. Sentiment score is high bullish (95% conviction) on BTC-USD. Recommending exposure.',
    direction: 'in',
  },
  {
    id: 'c-5',
    timestamp: new Date(Date.now() - 10000).toLocaleTimeString(),
    from: 'Nethra Swarm',
    to: 'Tredo Executor',
    message: 'Dispatch limit order to execute buying transaction on watchlisted BTC-USD.',
    direction: 'out',
  },
  {
    id: 'c-6',
    timestamp: new Date(Date.now() - 8000).toLocaleTimeString(),
    from: 'Tredo Executor',
    to: 'Nethra Swarm',
    message: 'Binance API limit order posted. Price: $77,430. Status: FILLED. Portfolio record updated.',
    direction: 'in',
  }
];

export function TantraModule() {
  const [babyAgents, setBabyAgents] = useState<BabyAgent[]>(DEFAULT_BABY_AGENTS);
  const [logs, setLogs] = useState<CommLog[]>(INITIAL_COMM_LOGS);
  const [selectedAgent, setSelectedAgent] = useState<BabyAgent | null>(null);
  
  // Custom operational task dispatcher
  const [selectedAgentId, setSelectedAgentId] = useState('baby-tredo');
  const [operatorObjective, setOperatorObjective] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);

  // Live telemetry metrics
  const [systemLoad, setSystemLoad] = useState({
    cpu: 28,
    ram: 42,
    net: 114,
  });

  // Keep system metrics dynamic
  useEffect(() => {
    const interval = setInterval(() => {
      setSystemLoad({
        cpu: Math.round(20 + Math.random() * 15),
        ram: Math.round(40 + Math.random() * 3),
        net: Math.round(90 + Math.random() * 40),
      });

      // Randomly fluctuate baby agent telemetry slightly
      setBabyAgents(prev => prev.map(agent => ({
        ...agent,
        metricCpu: Math.max(2, Math.min(95, agent.metricCpu + Math.round((Math.random() - 0.5) * 4))),
        metricRam: Math.max(100, Math.min(1024, agent.metricRam + Math.round((Math.random() - 0.5) * 10))),
      })));
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  // Handle command dispatching & simulation
  const handleDispatchObjective = (e: React.FormEvent) => {
    e.preventDefault();
    if (!operatorObjective.trim() || isProcessing) return;

    setIsProcessing(true);
    const targetAgent = babyAgents.find(a => a.id === selectedAgentId);
    if (!targetAgent) { setIsProcessing(false); return; }

    // Update baby agent state to executing
    setBabyAgents(prev => prev.map(a => a.id === targetAgent.id ? { ...a, status: 'executing' } : a));

    const newLog1: CommLog = {
      id: `c-dispatch-${Date.now()}-1`,
      timestamp: new Date().toLocaleTimeString(),
      from: 'Nethra Swarm',
      to: targetAgent.name,
      message: operatorObjective.trim(),
      direction: 'out',
    };

    setLogs(prev => [...prev, newLog1]);
    const objText = operatorObjective.trim();
    setOperatorObjective('');

    // Simulate baby agent processing and returning work
    setTimeout(() => {
      let mockReply = '';
      if (targetAgent.id === 'baby-tredo') {
        mockReply = `Order processed successfully. Executed transaction matching objective: "${objText}". Target assets reconciled.`;
      } else if (targetAgent.id === 'baby-risk') {
        mockReply = `Audit check COMPLETE. Validated risk parameters for objective "${objText}". Metrics remain secure.`;
      } else if (targetAgent.id === 'baby-news') {
        mockReply = `Scraped and classified macro metrics relevant to objective "${objText}". Recommending neutral-positive action.`;
      } else {
        mockReply = `Bridge connection telemetry clean. Active locks cleared. Re-aligned services for objective "${objText}".`;
      }

      const newLog2: CommLog = {
        id: `c-dispatch-${Date.now()}-2`,
        timestamp: new Date().toLocaleTimeString(),
        from: targetAgent.name,
        to: 'Nethra Swarm',
        message: mockReply,
        direction: 'in',
      };

      setLogs(prev => [...prev, newLog2]);
      setBabyAgents(prev => prev.map(a => 
        a.id === targetAgent.id 
          ? { 
              ...a, 
              status: 'idle', 
              lastTask: objText,
              lastResponse: mockReply 
            } 
          : a
      ));
      setIsProcessing(false);
    }, 2500);
  };

  return (
    <div className="flex h-full gap-6 overflow-hidden select-none" role="region" aria-label="Nethra Swarm and Coworker Coordinator">
      
      {/* LEFT COLUMN: Workspace Live Status & Bridge Diagnostics */}
      <div className="w-80 flex flex-col gap-5 overflow-hidden shrink-0">
        
        {/* Workspace live monitor */}
        <div className="glass-panel rounded-xl p-5 flex flex-col overflow-hidden relative">
          <div className="absolute top-0 right-0 w-20 h-20 bg-cyber-green/5 rounded-full blur-xl" />
          
          <div className="flex justify-between items-center mb-4 border-b border-cyber-border/20 pb-2">
            <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300">WORKSPACE LIVE MONITOR</h3>
            <span className="text-[8px] bg-cyber-green/10 border border-cyber-green/30 text-cyber-green font-mono px-2 py-0.5 rounded-full animate-pulse">ACTIVE</span>
          </div>

          <div className="space-y-3.5 text-[10px] font-mono">
            <div>
              <span className="text-slate-500 block mb-1">ACTIVE DIRECTORY</span>
              <span className="text-slate-300 font-bold block truncate bg-cyber-dark/40 px-2.5 py-1.5 rounded border border-cyber-border/20 text-[9px]">
                /home/varma/Freebuff
              </span>
            </div>

            <div className="grid grid-cols-2 gap-3 text-[9px] pt-1">
              <div className="bg-cyber-dark/40 border border-cyber-border/20 rounded p-2 text-center">
                <span className="text-slate-500 block mb-0.5">CPU CORE LOAD</span>
                <span className="text-cyber-green text-sm font-bold">{systemLoad.cpu}%</span>
              </div>
              <div className="bg-cyber-dark/40 border border-cyber-border/20 rounded p-2 text-center">
                <span className="text-slate-500 block mb-0.5">SWARM RAM</span>
                <span className="text-cyber-purple text-sm font-bold">{systemLoad.ram}%</span>
              </div>
            </div>

            {/* Active indexed files */}
            <div className="border-t border-cyber-border/10 pt-3">
              <span className="text-slate-500 block mb-2 tracking-wider">SWARM INDEXED CHANNELS</span>
              <div className="space-y-1.5 max-h-[140px] overflow-y-auto pr-1 scrollbar-cyber text-[9px]">
                <div className="flex items-center justify-between p-1.5 bg-cyber-dark/30 rounded border border-cyber-border/10 hover:border-cyber-purple/20 transition-all">
                  <span className="text-slate-300 truncate">crates/tredo-core/src/types.rs</span>
                  <span className="text-cyber-green">Synced</span>
                </div>
                <div className="flex items-center justify-between p-1.5 bg-cyber-dark/30 rounded border border-cyber-border/10 hover:border-cyber-purple/20 transition-all">
                  <span className="text-slate-300 truncate">frontend/src/atoms/state.ts</span>
                  <span className="text-cyber-green">Synced</span>
                </div>
                <div className="flex items-center justify-between p-1.5 bg-cyber-dark/30 rounded border border-cyber-border/10 hover:border-cyber-purple/20 transition-all">
                  <span className="text-slate-300 truncate">frontend/src/app/Journal.tsx</span>
                  <span className="text-cyber-purple">Modified</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Global LLM active options */}
        <div className="flex-1 glass-panel rounded-xl p-5 flex flex-col overflow-hidden relative">
          <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">
            SWARM CONTEXT MODELS
          </h3>
          <div className="flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-cyber text-[9px] font-mono">
            <div className="p-2.5 bg-cyber-purple/5 border border-cyber-purple/20 rounded-lg flex items-center justify-between">
              <div className="flex flex-col gap-0.5">
                <span className="text-slate-300 font-bold">gemini-2.0-flash</span>
                <span className="text-slate-500 text-[8px]">Primary Swarm Orchestration</span>
              </div>
              <span className="text-cyber-purple font-bold">Cloud</span>
            </div>
            


            <div className="p-2.5 bg-cyber-green/5 border border-cyber-green/20 rounded-lg flex items-center justify-between">
              <div className="flex flex-col gap-0.5">
                <span className="text-slate-300 font-bold">nemotron-3-nano:4b</span>
                <span className="text-slate-500 text-[8px]">Reasoning & Micro-Agent Executors</span>
              </div>
              <span className="text-cyber-green font-bold">Local</span>
            </div>
          </div>
        </div>

      </div>

      {/* CENTER COLUMN: Live Interactive Swarm Node Tree */}
      <div className="flex-1 glass-panel rounded-xl p-5 flex flex-col overflow-hidden relative">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-48 h-48 bg-cyber-purple/5 rounded-full blur-3xl pointer-events-none" />
        
        <div className="flex justify-between items-center mb-4 border-b border-cyber-border/20 pb-2 shrink-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-bold font-mono tracking-wider text-slate-300">NETHRA HIERARCHICAL SWARM TREE</span>
            <span className="text-[8px] bg-cyber-purple/20 border border-cyber-purple/30 text-cyber-purple font-mono px-2 py-0.5 rounded-full">LIVE TELEMETRY</span>
          </div>
          <span className="text-[9px] font-mono text-slate-500">CLICK ANY NODE TO INSPECT DETAILS</span>
        </div>

        {/* Tree Render Panel */}
        <div className="flex-1 flex flex-col justify-center items-center relative min-h-[300px] bg-cyber-dark/20 border border-cyber-border/10 rounded-xl overflow-hidden p-6 mb-4">
          
          {/* NETHRA COMMANDER (ROOT NODE) */}
          <div className="flex flex-col items-center z-10">
            <div 
              onClick={() => setSelectedAgent({
                id: 'nethra-root',
                name: 'Nethra Swarm (Commander)',
                role: 'Swarm Commander, orchestrator, decision maker, and workflow router.',
                status: 'idle',
                assignedLLM: 'gemini-2.0-flash (Cloud)',
                temperature: 0.4,
                lastTask: 'Supervise hierarchical agent coordination and dispatch sub-tasks.',
                lastResponse: 'Awaiting operator directive or automated alert dispatch.',
                metricCpu: 15,
                metricRam: 512,
              })}
              className={cn(
                "group relative px-6 py-3.5 rounded-xl border flex flex-col items-center cursor-pointer transition-all duration-300",
                "bg-cyber-purple/10 border-cyber-purple hover:border-pink-500 hover:shadow-[0_0_20px_rgba(168,85,247,0.4)]",
                selectedAgent?.id === 'nethra-root' ? 'ring-2 ring-pink-500 shadow-[0_0_25px_rgba(168,85,247,0.5)]' : ''
              )}
            >
              <div className="absolute -top-1.5 bg-cyber-purple text-white font-bold font-mono text-[7px] px-1.5 py-0.5 rounded tracking-widest">COMMANDER</div>
              <span className="text-xs font-bold font-mono text-slate-200 tracking-wide">👑 Nethra Swarm</span>
              <span className="text-[8px] font-mono text-cyber-purple mt-1 tracking-wider uppercase">Active Coordinator</span>
            </div>
            
            {/* SVG Connecting Lines */}
            <div className="w-96 h-16 relative">
              <svg className="absolute w-full h-full left-0 top-0 pointer-events-none" xmlns="http://www.w3.org/2000/svg">
                {/* Connecting lines from Commander (center top) to 4 children (distributed left to right) */}
                <path d="M 192,0 L 48,64" stroke="#a855f7" strokeWidth="1.5" strokeOpacity="0.4" fill="none" />
                <path d="M 192,0 L 144,64" stroke="#a855f7" strokeWidth="1.5" strokeOpacity="0.4" fill="none" />
                <path d="M 192,0 L 240,64" stroke="#a855f7" strokeWidth="1.5" strokeOpacity="0.4" fill="none" />
                <path d="M 192,0 L 336,64" stroke="#a855f7" strokeWidth="1.5" strokeOpacity="0.4" fill="none" />
                
                {/* Active signals moving down */}
                <circle r="3" fill="#00e676" className="animate-[ping_2s_infinite]">
                  <animateMotion dur="2.5s" repeatCount="indefinite" path="M 192,0 L 48,64" />
                </circle>
                <circle r="3" fill="#a855f7" className="animate-[ping_2s_infinite]">
                  <animateMotion dur="2s" repeatCount="indefinite" path="M 192,0 L 144,64" />
                </circle>
                <circle r="3" fill="#ffc107" className="animate-[ping_2s_infinite]">
                  <animateMotion dur="3s" repeatCount="indefinite" path="M 192,0 L 240,64" />
                </circle>
                <circle r="3" fill="#00bcd4" className="animate-[ping_2s_infinite]">
                  <animateMotion dur="2.2s" repeatCount="indefinite" path="M 192,0 L 336,64" />
                </circle>
              </svg>
            </div>

            {/* BABY AGENTS (CHILDREN NODES) */}
            <div className="flex gap-4 items-center">
              {babyAgents.map((agent) => (
                <div
                  key={agent.id}
                  onClick={() => setSelectedAgent(agent)}
                  className={cn(
                    "group relative px-4 py-2.5 rounded-lg border flex flex-col items-center cursor-pointer transition-all duration-300 bg-cyber-dark/80",
                    agent.status === 'executing'
                      ? "border-cyber-green bg-cyber-green/5 shadow-[0_0_15px_rgba(0,230,118,0.25)] animate-pulse"
                      : "border-cyber-border/50 hover:border-cyber-purple/50 hover:bg-cyber-purple/5",
                    selectedAgent?.id === agent.id ? "ring-2 ring-cyber-purple border-cyber-purple shadow-[0_0_15px_rgba(168,85,247,0.2)]" : ""
                  )}
                >
                  <div className="absolute -top-1 bg-cyber-border text-slate-400 font-bold font-mono text-[6px] px-1.5 rounded tracking-wide">BABY AGENT</div>
                  <span className="text-[10px] font-bold font-mono text-slate-300 mt-1">{agent.name}</span>
                  <span className="text-[7px] font-mono text-slate-500 mt-0.5 truncate max-w-[80px]">{agent.assignedLLM.split(' ')[0]}</span>
                  
                  {/* Status dot */}
                  <span className={cn(
                    "absolute -right-1 -top-1 w-2 h-2 rounded-full",
                    agent.status === 'executing' ? 'bg-cyber-green animate-ping' :
                    agent.status === 'idle' ? 'bg-cyber-purple' : 'bg-slate-600'
                  )} />
                </div>
              ))}
            </div>
          </div>

          {/* Node Diagnostics Panel */}
          {selectedAgent && (
            <div className="absolute bottom-3 left-3 right-3 bg-cyber-dark/95 border border-cyber-purple/30 rounded-lg p-3.5 animate-slide-up text-[9px] font-mono space-y-2">
              <div className="flex justify-between items-center border-b border-cyber-border/20 pb-1.5">
                <span className="text-slate-200 font-bold text-xs">{selectedAgent.name}</span>
                <div className="flex items-center gap-2">
                  <span className="text-slate-500">LLM: <code className="text-cyber-purple">{selectedAgent.assignedLLM}</code></span>
                  <button onClick={() => setSelectedAgent(null)} className="text-slate-400 hover:text-slate-200">✕</button>
                </div>
              </div>
              
              <p className="text-slate-400 leading-relaxed"><strong className="text-slate-500">ROLE:</strong> {selectedAgent.role}</p>
              
              <div className="grid grid-cols-2 gap-3 pt-1">
                <div>
                  <span className="text-slate-500 block font-bold">LAST OBJECTIVE</span>
                  <span className="text-slate-300 block truncate">{selectedAgent.lastTask}</span>
                </div>
                <div>
                  <span className="text-slate-500 block font-bold">LAST WORK RETURNED</span>
                  <span className="text-cyber-green block truncate">{selectedAgent.lastResponse}</span>
                </div>
              </div>
              
              <div className="flex gap-4 pt-1 text-[8px] text-slate-500">
                <span>CPU: {selectedAgent.metricCpu}%</span>
                <span>Memory Allocation: {selectedAgent.metricRam}MB</span>
                <span>Temperature: {selectedAgent.temperature}</span>
              </div>
            </div>
          )}
        </div>

        {/* Live Swarm Commander Console */}
        <div className="shrink-0 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4">
          <h4 className="text-[10px] font-bold font-mono text-slate-300 mb-2 uppercase tracking-wider">
            ⚡ Swarm Commander Directive Interface
          </h4>
          <form onSubmit={handleDispatchObjective} className="flex gap-3 text-[10px] font-mono">
            <select
              value={selectedAgentId}
              onChange={(e) => setSelectedAgentId(e.target.value)}
              className="bg-cyber-dark border border-cyber-border/40 rounded px-2 text-slate-300 focus:outline-none focus:border-cyber-purple text-[10px]"
            >
              {babyAgents.map(a => (
                <option key={a.id} value={a.id}>{a.name}</option>
              ))}
            </select>
            
            <input
              type="text"
              required
              value={operatorObjective}
              onChange={(e) => setOperatorObjective(e.target.value)}
              placeholder="Enter objective for Nethra to assign to Baby Agent..."
              className="flex-1 bg-cyber-dark border border-cyber-border/40 rounded px-3 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple text-[10px]"
            />
            
            <button
              type="submit"
              disabled={isProcessing}
              className="px-4 py-1.5 bg-cyber-purple hover:bg-cyber-purple/90 text-white font-bold rounded transition-all"
            >
              {isProcessing ? 'DISPATCHING...' : 'DISPATCH'}
            </button>
          </form>
        </div>

      </div>

      {/* RIGHT COLUMN: Live LLM Communication Streams */}
      <div className="w-80 glass-panel rounded-xl p-5 flex flex-col overflow-hidden shrink-0 relative">
        <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">
          LIVE LLM SWARM COMMUNICATIONS
        </h3>

        <div className="flex-1 overflow-y-auto space-y-3 pr-1 scrollbar-cyber text-[9px] font-mono">
          {logs.map((log) => {
            const isOut = log.direction === 'out';
            return (
              <div 
                key={log.id} 
                className={cn(
                  "p-2.5 rounded-lg border",
                  isOut 
                    ? "bg-cyber-purple/5 border-cyber-purple/20 ml-2" 
                    : "bg-cyber-green/5 border-cyber-green/20 mr-2"
                )}
              >
                <div className="flex justify-between items-center text-[7px] text-slate-500 mb-1">
                  <span className={cn("font-bold", isOut ? "text-cyber-purple" : "text-cyber-green")}>
                    {log.from} {isOut ? '➔' : '➔'} {log.to}
                  </span>
                  <span>{log.timestamp}</span>
                </div>
                <p className="text-slate-300 leading-normal">{log.message}</p>
                
                {/* Visual returning token */}
                {!isOut && (
                  <span className="text-[7px] text-cyber-purple font-semibold mt-1 block">
                    ✓ Tasks completed. Work returned to Nethra.
                  </span>
                )}
              </div>
            );
          })}
        </div>

        <div className="mt-4 border-t border-cyber-border/20 pt-3 text-[8px] font-mono text-slate-500 flex justify-between items-center">
          <span>Bridge: Sethu Core v1.8.0</span>
          <span className="flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-cyber-green animate-pulse" />
            Live Sync
          </span>
        </div>
      </div>

    </div>
  );
}
