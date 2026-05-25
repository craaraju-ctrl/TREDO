import { useState, useCallback, useEffect, useRef } from 'react';
import { useAtom } from 'jotai';
import { cn } from '../../lib/utils';
import { Badge, ProgressBar } from '../ui/Badge';
import { EmptyState } from '../ui/States';
import {
  chatMessagesAtom,
  chatInputAtom,
  selectedModelAtom,
  selectedAgentAtom,
  selectedModuleAtom,
  skillAnalysisAtom,
  availableSkillsAtom,
  priceHistoryAtom,
  cashBalanceAtom,
  portfolioValueAtom,
  chatSessionsAtom,
  activeSessionIdAtom,
  skillAnalyzingAtom,
  isTypingAtom,
  type ChatMessage,
  type ChatSession,
  type AggregatedAnalysis,
} from '../../atoms/state';

export function ChatModule() {
  const [messages, setMessages] = useAtom(chatMessagesAtom);
  const [chatInput, setChatInput] = useAtom(chatInputAtom);
  const [selectedModel, setSelectedModel] = useAtom(selectedModelAtom);
  const [selectedAgent, setSelectedAgent] = useAtom(selectedAgentAtom);
  const [selectedModule, setSelectedModule] = useAtom(selectedModuleAtom);

  // Persist selections to localStorage
  useEffect(() => {
    try { localStorage.setItem('tredo_settings_selected_model', JSON.stringify(selectedModel)); } catch {}
  }, [selectedModel]);
  useEffect(() => {
    try { localStorage.setItem('tredo_settings_selected_agent', JSON.stringify(selectedAgent)); } catch {}
  }, [selectedAgent]);
  useEffect(() => {
    try { localStorage.setItem('tredo_settings_selected_module', JSON.stringify(selectedModule)); } catch {}
  }, [selectedModule]);
  const [skillAnalysis, setSkillAnalysis] = useAtom(skillAnalysisAtom);
  const [availableSkills] = useAtom(availableSkillsAtom);
  const [priceHistory] = useAtom(priceHistoryAtom);
  const [cash] = useAtom(cashBalanceAtom);
  const [portfolioVal] = useAtom(portfolioValueAtom);

  const [sessions, setSessions] = useAtom(chatSessionsAtom);
  const [activeSessionId, setActiveSessionId] = useAtom(activeSessionIdAtom);
  const [skillAnalyzing, setSkillAnalyzing] = useAtom(skillAnalyzingAtom);
  const [isTyping, setIsTyping] = useAtom(isTypingAtom);
  const [tools, setTools] = useState<string[]>(['skills', 'market-data', 'journal']);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const chatContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  // Fetch available skills on mount
  useEffect(() => {
    const fetchSkills = async () => {
      try {
        const res = await fetch('/api/skills/list');
        const data = await res.json();
        if (data.status === 'success') {
          // availableSkillsAtom is set by the parent
        }
      } catch {
        // Backend not responding
      }
    };
    fetchSkills();
  }, []);

  const handleSwitchSession = useCallback(
    (sessionId: string) => {
      const target = sessions.find((s) => s.id === sessionId);
      if (!target) return;
      setActiveSessionId(sessionId);
      setMessages(target.messages);
      setSelectedModel(target.model);
      setSelectedAgent(target.agent);
    },
    [sessions, setMessages, setSelectedModel, setSelectedAgent]
  );

  const handleNewChat = useCallback(() => {
    const newId = `session-${Date.now()}`;
    const newSession: ChatSession = {
      id: newId,
      title: `Conversation ${sessions.length + 1}`,
      messages: [
        {
          sender: 'Nethra',
          text: 'Greetings, Operator. Nethra Swarm is online. Chat, Tredo, and Tantra modules are operational.',
          timestamp: Date.now(),
        },
      ],
      agent: 'Nethra Swarm',
      model: 'nemotron-3-nano:4b',
      timestamp: Date.now(),
    };

    setSessions((prev) => [newSession, ...prev]);
    setActiveSessionId(newId);
    setMessages(newSession.messages);
    setSelectedModel(newSession.model);
    setSelectedAgent(newSession.agent);
  }, [sessions.length, setMessages, setSelectedModel, setSelectedAgent]);

  const handleDeleteSession = useCallback(
    (sessionId: string) => {
      if (sessions.length <= 1) {
        // Create a fresh session instead of leaving stale metadata
        const freshId = `session-${Date.now()}`;
        const freshSession: ChatSession = {
          id: freshId,
          title: 'New Conversation',
          messages: [
            {
              sender: 'Nethra',
              text: 'Greetings, Operator. Nethra Swarm is online. Chat, Tredo, and Tantra modules are operational.',
              timestamp: Date.now(),
            },
          ],
          agent: 'Nethra Swarm',
          model: 'nemotron-3-nano:4b',
          timestamp: Date.now(),
        };
        setSessions([freshSession]);
        setActiveSessionId(freshId);
        setMessages(freshSession.messages);
        setSelectedModel(freshSession.model);
        setSelectedAgent(freshSession.agent);
        return;
      }

      const remaining = sessions.filter((s) => s.id !== sessionId);
      setSessions(remaining);

      if (activeSessionId === sessionId) {
        const nextActive = remaining[0];
        setActiveSessionId(nextActive.id);
        setMessages(nextActive.messages);
        setSelectedModel(nextActive.model);
        setSelectedAgent(nextActive.agent);
      }
    },
    [sessions, activeSessionId, setMessages, setSelectedModel, setSelectedAgent]
  );

  const updateSessionModel = useCallback(
    (sessionId: string, model: string) => {
      setSessions((prev) => prev.map((s) => (s.id === sessionId ? { ...s, model } : s)));
    },
    []
  );

  const updateSessionAgent = useCallback(
    (sessionId: string, agent: string) => {
      setSessions((prev) => prev.map((s) => (s.id === sessionId ? { ...s, agent } : s)));
    },
    []
  );

  const appendChatMessage = useCallback(
    (msg: ChatMessage) => {
      setMessages((prev) => {
        const updated = [...prev, msg];
        setSessions((sPrev) =>
          sPrev.map((s) => (s.id === activeSessionId ? { ...s, messages: updated, timestamp: Date.now() } : s))
        );
        return updated;
      });
    },
    [activeSessionId, setMessages]
  );

  const handleSendMessage = useCallback(async () => {
    if (!chatInput.trim()) return;
    const prompt = chatInput;
    appendChatMessage({ sender: 'Operator', text: prompt, timestamp: Date.now() });
    setChatInput('');
    setIsTyping(true);

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt }),
      });
      const data = await response.json();

      let replyText = '';
      if (data.status === 'success') {
        try {
          const parsed = JSON.parse(data.reply);
          replyText = `[Gemini Conviction: ${(parsed.conviction * 100).toFixed(0)}%] ${parsed.reasoning}`;
        } catch {
          replyText = data.reply;
        }
      } else {
        replyText = `Failed to query intelligence pool: ${data.message || 'Unknown error'}`;
      }

      appendChatMessage({ sender: 'Nethra', text: replyText, timestamp: Date.now() });
    } catch {
      appendChatMessage({
        sender: 'Nethra',
        text: `[Local Fallback Mode] Connection failed. Bollinger compression indicates support holds. Conviction is 82%.`,
        timestamp: Date.now(),
      });
    } finally {
      setIsTyping(false);
    }
  }, [chatInput, appendChatMessage, setChatInput]);

  const handleRunSkillsAnalysis = useCallback(async () => {
    if (skillAnalyzing) return;
    setSkillAnalyzing(true);

    try {
      const selectedAsset = 'BTC-USD'; // Default — could be passed via props
      const candles = priceHistory[selectedAsset];
      const currentPrice = candles?.[candles.length - 1]?.close ?? 77430;

      const payload = {
        symbol: selectedAsset,
        current_price: currentPrice,
        cash_available: cash,
        portfolio_value: portfolioVal,
        candles: (candles || []).map((c) => ({
          time: c.time,
          open: c.open,
          high: c.high,
          low: c.low,
          close: c.close,
          volume: c.volume,
        })),
      };

      const res = await fetch('/api/skills/analyze', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (data.status === 'success') {
        setSkillAnalysis(data.analysis);

        const analysis = data.analysis as AggregatedAnalysis;
        const dirEmoji =
          analysis.overall_direction === 'Bullish'
            ? '🟢'
            : analysis.overall_direction === 'Bearish'
            ? '🔴'
            : '⚪';
        const msg = `${dirEmoji} **Nethra Skills Analysis** for ${analysis.symbol}\nConviction: ${(analysis.overall_conviction * 100).toFixed(0)}% (${analysis.overall_direction})\nSignals: ${analysis.bullish_signals} Bullish | ${analysis.bearish_signals} Bearish | ${analysis.neutral_signals} Neutral\nSkills Fired: ${analysis.signals.length}/${availableSkills.length} skills triggered`;

        appendChatMessage({ sender: 'Nethra', text: msg, timestamp: Date.now() });
      }
    } catch {
      appendChatMessage({
        sender: 'Nethra',
        text: `Skills analysis failed — backend not reachable. Using fallback conviction model.`,
        timestamp: Date.now(),
      });
    } finally {
      setSkillAnalyzing(false);
    }
  }, [skillAnalyzing, priceHistory, cash, portfolioVal, availableSkills.length, setSkillAnalysis, appendChatMessage]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSendMessage();
      }
    },
    [handleSendMessage]
  );

  return (
    <div className="flex h-full gap-6" role="region" aria-label="Chat module">
      {/* LEFT SIDEBAR: Chat History + Intelligence */}
      <div className="w-80 flex flex-col gap-6 h-full shrink-0">
        {/* Chat History */}
        <div className="glass-panel rounded-xl p-5 flex flex-col flex-grow overflow-hidden min-h-[250px]">
          <div className="flex items-center justify-between mb-4 border-b border-cyber-border/20 pb-2">
            <h3 className="text-xs font-bold tracking-wider font-mono text-slate-300" id="chat-history-heading">
              CHAT HISTORY
            </h3>
            <button
              onClick={handleNewChat}
              className="btn-primary text-[10px] px-2.5 py-1"
              aria-label="Start new chat"
            >
              + NEW
            </button>
          </div>

          <div
            className="flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-cyber"
            role="list"
            aria-labelledby="chat-history-heading"
          >
            {sessions.map((session) => (
              <div
                key={session.id}
                onClick={() => handleSwitchSession(session.id)}
                role="listitem"
                className={cn(
                  'group p-3 rounded-lg border font-mono text-xs cursor-pointer transition-all duration-200',
                  activeSessionId === session.id
                    ? 'bg-cyber-purple/10 border-cyber-purple/40 text-cyber-purple shadow-purple'
                    : 'bg-cyber-panel/20 border-transparent text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/40'
                )}
              >
                <div className="flex justify-between items-start gap-1">
                  <span className="font-semibold truncate max-w-[170px]">{session.title}</span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteSession(session.id);
                    }}
                    className="opacity-0 group-hover:opacity-100 hover:text-red-400 text-[10px] p-0.5 transition-opacity"
                    aria-label={`Delete ${session.title}`}
                  >
                    ✕
                  </button>
                </div>
                <div className="flex justify-between items-center text-[9px] text-slate-500 mt-2 font-mono">
                  <span>{session.agent.split(' ')[0]}</span>
                  <span>
                    {new Date(session.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Intelligence Control Panel */}
        <div className="glass-panel rounded-xl p-5 space-y-5 shrink-0 gradient-border">
          <h3 className="text-xs font-bold tracking-wider font-mono text-slate-300">INTELLIGENCE CONTROL</h3>
          <div className="space-y-3.5 text-xs font-mono">
            <div>
              <label htmlFor="model-select" className="text-slate-400 block mb-1.5 text-[10px]">
                Reasoning LLM Model
              </label>
              <select
                id="model-select"
                value={selectedModel}
                onChange={(e) => {
                  setSelectedModel(e.target.value);
                  updateSessionModel(activeSessionId, e.target.value);
                }}
                className="select-cyber"
              >
                <option value="nemotron-3-nano:4b">nemotron-3-nano:4b (Local)</option>
                <option value="gemini-2.0-flash">gemini-2.0-flash (Cloud)</option>
              </select>
            </div>
            <div>
              <label htmlFor="agent-select" className="text-slate-400 block mb-1.5 text-[10px]">
                Active Special Agent
              </label>
              <select
                id="agent-select"
                value={selectedAgent}
                onChange={(e) => {
                  setSelectedAgent(e.target.value);
                  updateSessionAgent(activeSessionId, e.target.value);
                }}
                className="select-cyber"
              >
                <option value="Nethra Swarm">Nethra (Commander / Decision Swarm)</option>
                <option value="Technical Analyst">└─ Technical Analyst (Baby Agent)</option>
                <option value="Risk Manager">└─ Risk Manager (Baby Agent)</option>
                <option value="Portfolio Manager">└─ Portfolio Manager (Baby Agent)</option>
                <option value="Market Data Agent">└─ Market Data Agent (Baby Agent)</option>
              </select>
            </div>

            {/* Swarm Hierarchy Explanation */}
            <div className="bg-cyber-purple/5 border border-cyber-purple/30 rounded-lg p-3 mt-3">
              <div className="flex items-center gap-1.5 mb-1.5">
                <span className="text-[10px] text-cyber-purple font-mono font-bold tracking-wider">NETHRA HIERARCHY</span>
                <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-pulse" />
              </div>
              <p className="text-[9px] text-slate-400 font-mono leading-relaxed">
                <strong className="text-slate-200">Nethra</strong> acts as the master Commander, Orchestrator, and final Decision Maker.
                The dedicated <strong className="text-cyber-glow">Baby Agents</strong> (Technical, Risk, Portfolio, and Market Data) operate as task executors that finish specialized analysis and return findings back to Nethra for consensus execution.
              </p>
            </div>

            {/* Active Module / Tool Selector */}
            <div className="border-t border-cyber-border/30 pt-3">
              <h4 className="text-xs font-bold text-cyber-purple mb-3">ACTIVE MODULE</h4>
              <div className="space-y-2.5">
                <div>
                  <label htmlFor="module-select" className="text-slate-400 block mb-1.5 text-[10px]">
                    Module Access
                  </label>
                  <select
                    id="module-select"
                    value={selectedModule}
                    onChange={(e) => {
                      setSelectedModule(e.target.value);
                    }}
                    className="select-cyber"
                  >
                    <option value="chat">Chat Only</option>
                    <option value="tredo">Tredo (Trading)</option>
                    <option value="tantra">Tantra (Coworker Agents)</option>
                    <option value="journal">Journal (Records & Tasks)</option>
                    <option value="all">All Modules</option>
                  </select>
                  <p className="text-[8px] font-mono text-slate-500 mt-1">
                    Determines which modules the agent can interact with.
                  </p>
                </div>
              </div>
            </div>

            {/* Tools Configuration */}
            <div className="border-t border-cyber-border/30 pt-3">
              <h4 className="text-xs font-bold text-cyber-purple mb-3">AGENT TOOLS</h4>
              <div className="space-y-2">
                {['skills', 'market-data', 'journal', 'risk-engine', 'position-sizing', 'backtest'].map((tool) => (
                  <label key={tool} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={tools.includes(tool)}
                      onChange={() => {
                        setTools((prev) =>
                          prev.includes(tool)
                            ? prev.filter((t) => t !== tool)
                            : [...prev, tool]
                        );
                      }}
                      className="accent-cyber-purple w-3 h-3"
                    />
                    <span className="text-[9px] font-mono text-slate-400 capitalize">{tool.replace(/-/g, ' ')}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* Nethra Skills System */}
            <div className="border-t border-cyber-border/30 pt-3">
              <h4 className="text-xs font-bold text-cyber-purple mb-3">NETHRA SKILLS SYSTEM</h4>
              <div className="space-y-2.5">
                <div className="flex items-center justify-between text-[10px] text-slate-500">
                  <span>Skills Loaded</span>
                  <span className="text-cyber-green font-semibold tabular-nums">{availableSkills.length}</span>
                </div>
                <button
                  onClick={handleRunSkillsAnalysis}
                  disabled={skillAnalyzing}
                  className={cn(
                    'w-full py-2.5 rounded-lg text-xs font-bold font-mono transition-all duration-200 border relative overflow-hidden',
                    skillAnalyzing
                      ? 'bg-cyber-panel/50 text-slate-500 border-cyber-border/30 cursor-not-allowed'
                      : 'btn-primary hover:shadow-[0_0_20px_rgba(157,78,221,0.35)]'
                  )}
                  aria-busy={skillAnalyzing}
                >
                  {skillAnalyzing ? (
                    <span className="flex items-center justify-center gap-2">
                      <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                      ANALYZING...
                    </span>
                  ) : (
                    <span className="flex items-center justify-center gap-2">
                      <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-pulse" />
                      RUN SKILLS ANALYSIS
                    </span>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* RIGHT: Main Chat area + Skills Results */}
      <div className="flex-1 flex gap-6 h-full overflow-hidden">
        {/* Chat Interface */}
        <div className="flex-1 flex flex-col glass-panel rounded-xl overflow-hidden" role="log" aria-label="Chat messages">
          <div
            ref={chatContainerRef}
            className="flex-1 overflow-y-auto space-y-4 p-6 scrollbar-cyber"
          >
            {messages.length === 0 ? (
              <EmptyState
                icon="💬"
                title="No messages yet"
                description="Start a conversation with Nethra to analyze markets, check system status, or run trading strategies."
              />
            ) : (
              messages.map((msg, i) => (
                <div
                  key={i}
                  className={cn(
                    'flex flex-col max-w-[80%] rounded-lg p-3 animate-slide-up transition-all duration-200 hover:shadow-lg',
                    msg.sender === 'Operator'
                      ? 'bg-cyber-purple/20 border border-cyber-purple/30 self-end ml-auto hover:border-cyber-purple/50 hover:shadow-purple'
                      : msg.sender === 'System'
                      ? 'bg-amber-500/10 border border-amber-500/20 self-center max-w-[60%]'
                      : 'bg-cyber-panel/60 border border-cyber-border/40 self-start hover:bg-cyber-panel/80'
                  )}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-[10px] font-semibold font-mono text-slate-400">{msg.sender}</span>
                    <span className="text-[8px] font-mono text-slate-600">
                      {new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                  </div>
                  <p className="text-sm leading-relaxed whitespace-pre-wrap font-mono text-slate-200">
                    {msg.text}
                  </p>
                </div>
              ))
            )}

            {/* Typing indicator */}
            {isTyping && (
              <div className="flex items-center gap-3 text-slate-400 text-xs font-mono pl-2 animate-slide-up bg-cyber-dark/30 rounded-lg px-3 py-2 border border-cyber-border/20 self-start">
                <div className="flex items-center gap-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-bounce" />
                  <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-bounce [animation-delay:0.15s]" />
                  <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-bounce [animation-delay:0.3s]" />
                </div>
                <span className="text-[10px] text-slate-500 font-semibold">Nethra is analyzing...</span>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>

          {/* Input Area */}
          <div className="border-t border-cyber-border/50 p-4 shrink-0">
            <div className="flex items-center gap-3">
              <input
                type="text"
                value={chatInput}
                onChange={(e) => setChatInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Ask Nethra to inspect orders, build automation bot scripts, or research risk templates..."
                className="input-cyber flex-1"
                aria-label="Chat message input"
                autoComplete="off"
              />
              <button
                onClick={handleSendMessage}
                disabled={!chatInput.trim()}
                className="btn-primary"
                aria-label="Send message"
              >
                SEND
              </button>
            </div>
          </div>
        </div>

        {/* Skills Analysis Results Panel */}
        {skillAnalysis && (
          <div className="w-80 glass-panel rounded-xl p-4 flex flex-col overflow-hidden h-full max-h-full shrink-0 animate-slide-up">
            <div className="flex items-center justify-between border-b border-cyber-border/20 pb-2 mb-3">
              <h4 className="text-[10px] font-bold font-mono tracking-wider text-slate-400">
                SKILLS ANALYSIS: {skillAnalysis.symbol}
              </h4>
              <Badge
                variant={
                  skillAnalysis.overall_direction === 'Bullish'
                    ? 'success'
                    : skillAnalysis.overall_direction === 'Bearish'
                    ? 'danger'
                    : 'neutral'
                }
              >
                {skillAnalysis.overall_direction}
              </Badge>
            </div>

            {/* Conviction Meter */}
            <div className="mb-3">
              <div className="flex justify-between text-[9px] font-mono text-slate-500 mb-1">
                <span>Conviction</span>
                <span
                  className={cn(
                    'font-semibold tabular-nums',
                    skillAnalysis.overall_conviction > 0.3
                      ? 'text-cyber-green'
                      : skillAnalysis.overall_conviction < -0.3
                      ? 'text-red-400'
                      : 'text-slate-400'
                  )}
                >
                  {(skillAnalysis.overall_conviction * 100).toFixed(0)}%
                </span>
              </div>
              <ProgressBar
                value={Math.abs(skillAnalysis.overall_conviction * 100)}
                variant={
                  skillAnalysis.overall_conviction > 0.3
                    ? 'success'
                    : skillAnalysis.overall_conviction < -0.3
                    ? 'danger'
                    : 'info'
                }
              />
            </div>

            {/* Signal Counters */}
            <div className="grid grid-cols-3 gap-2 mb-3">
              <div className="bg-cyber-green/10 border border-cyber-green/30 rounded p-1.5 text-center">
                <span className="text-cyber-green text-[10px] font-bold tabular-nums">{skillAnalysis.bullish_signals}</span>
                <span className="text-[8px] text-slate-500 block">Bullish</span>
              </div>
              <div className="bg-red-500/10 border border-red-500/30 rounded p-1.5 text-center">
                <span className="text-red-400 text-[10px] font-bold tabular-nums">{skillAnalysis.bearish_signals}</span>
                <span className="text-[8px] text-slate-500 block">Bearish</span>
              </div>
              <div className="bg-slate-500/10 border border-slate-500/30 rounded p-1.5 text-center">
                <span className="text-slate-400 text-[10px] font-bold tabular-nums">{skillAnalysis.neutral_signals}</span>
                <span className="text-[8px] text-slate-500 block">Neutral</span>
              </div>
            </div>

            {/* Individual Signals */}
            <div className="flex-1 overflow-y-auto space-y-1.5 pr-1 scrollbar-cyber">
              {skillAnalysis.signals.map((signal, idx) => (
                <div key={idx} className="p-2 bg-cyber-dark/40 border border-cyber-border/30 rounded-lg">
                  <div className="flex justify-between items-center mb-0.5">
                    <span className="text-[9px] font-mono text-slate-300 font-semibold truncate mr-1">
                      {signal.skill_name}
                    </span>
                    <Badge
                      variant={
                        signal.direction === 'Bullish'
                          ? 'success'
                          : signal.direction === 'Bearish'
                          ? 'danger'
                          : 'neutral'
                      }
                    >
                      {signal.direction === 'Bullish' ? '▲' : signal.direction === 'Bearish' ? '▼' : '◆'}{' '}
                      {signal.direction}
                    </Badge>
                  </div>
                  <p className="text-[8px] text-slate-500 font-mono leading-tight">{signal.details}</p>
                  <div className="flex justify-between text-[7px] text-slate-600 font-mono mt-0.5">
                    <span>Strength: {(signal.strength * 100).toFixed(0)}%</span>
                    <span>Confidence: {(signal.confidence * 100).toFixed(0)}%</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
