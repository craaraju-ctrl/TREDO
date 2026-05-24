import { useState, useEffect, useRef } from 'react';
import { useAtom } from 'jotai';
import {
  activeModuleAtom,
  chatMessagesAtom,
  chatInputAtom,
  selectedModelAtom,
  selectedAgentAtom,
  watchlistAtom,
  selectedAssetAtom,
  portfolioValueAtom,
  cashBalanceAtom,
  systemAlertsAtom,
  serverLogsAtom,
  metricsAtom,
  openOrdersAtom,
  tradesHistoryAtom,
  priceHistoryAtom,
  calendarEventsAtom,
  coworkerTasksAtom,
  newsFeedAtom,
  portfolioHealthAtom,
  skillAnalysisAtom,
  availableSkillsAtom,
  autoTradingStateAtom,
  performanceStatsAtom,
  OpenOrder,
  TradeRecord,
  Candlestick,
  AggregatedAnalysis,
} from '../atoms/state';
import Settings from './Settings';
import Journal from './Journal';

const BASE_PRICES: Record<string, number> = {
  'BTC-USD': 77430.0,
  'ETH-USD': 3450.0,
  'SOL-USD': 142.5,
  'XAU-USD': 2352.0,
};

export default function App() {
  const [activeTab, setActiveTab] = useAtom(activeModuleAtom);

  // Chat States
  const [messages, setMessages] = useAtom(chatMessagesAtom);
  const [chatInput, setChatInput] = useAtom(chatInputAtom);
  const [selectedModel, setSelectedModel] = useAtom(selectedModelAtom);
  const [selectedAgent, setSelectedAgent] = useAtom(selectedAgentAtom);

  // Chat History / Sessions State
  interface ChatSession {
    id: string;
    title: string;
    messages: typeof messages;
    agent: string;
    model: string;
    timestamp: number;
  }

  const [sessions, setSessions] = useState<ChatSession[]>([
    {
      id: 'session-1',
      title: 'Sethu System Health & Intel',
      messages: [
        {
          sender: 'Hermes',
          text: 'Greetings, Operator. Sethu bridge is online. Chat, Tredo, and Tantra modules are operational.',
          timestamp: Date.now() - 3600000,
        }
      ],
      agent: 'Hermes Tredo',
      model: 'qwen3.5:0.8b',
      timestamp: Date.now() - 3600000,
    },
    {
      id: 'session-2',
      title: 'BTC Conviction Analysis',
      messages: [
        {
          sender: 'Operator',
          text: 'Run skills analysis on BTC-USD.',
          timestamp: Date.now() - 1800000,
        },
        {
          sender: 'Hermes',
          text: '🟢 **Hermes Skills Analysis** for BTC-USD\nConviction: 41% (Bullish)\nSignals: 20 Bullish | 6 Bearish | 3 Neutral\nSkills Fired: 29/32 skills triggered',
          timestamp: Date.now() - 1795000,
        }
      ],
      agent: 'Hermes Tredo',
      model: 'qwen3.5:0.8b',
      timestamp: Date.now() - 1800000,
    },
    {
      id: 'session-3',
      title: 'Tantra Risk Policy Review',
      messages: [
        {
          sender: 'Operator',
          text: 'What is the current safety coordination index?',
          timestamp: Date.now() - 600000,
        },
        {
          sender: 'Hermes',
          text: 'Safety coordinator index is set to HIGH_GUARD due to active calendar DND schedule.',
          timestamp: Date.now() - 590000,
        }
      ],
      agent: 'Risk Manager',
      model: 'nemotron-3-nano:4b',
      timestamp: Date.now() - 600000,
    }
  ]);

  const [activeSessionId, setActiveSessionId] = useState<string>('session-1');

  // Session Switch helper:
  const handleSwitchSession = (sessionId: string) => {
    const target = sessions.find(s => s.id === sessionId);
    if (!target) return;
    setActiveSessionId(sessionId);
    setMessages(target.messages);
    setSelectedModel(target.model);
    setSelectedAgent(target.agent);
  };

  // Add a new session helper:
  const handleNewChat = () => {
    const newId = `session-${Date.now()}`;
    const newSession: ChatSession = {
      id: newId,
      title: `Conversation ${sessions.length + 1}`,
      messages: [
        {
          sender: 'Hermes',
          text: 'Greetings, Operator. Sethu bridge is online. Chat, Tredo, and Tantra modules are operational.',
          timestamp: Date.now(),
        }
      ],
      agent: 'Hermes Tredo',
      model: 'qwen3.5:0.8b',
      timestamp: Date.now(),
    };

    setSessions(prev => [newSession, ...prev]);
    setActiveSessionId(newId);
    setMessages(newSession.messages);
    setSelectedModel(newSession.model);
    setSelectedAgent(newSession.agent);
  };

  // Delete a session helper:
  const handleDeleteSession = (sessionId: string) => {
    if (sessions.length <= 1) {
      setMessages([
        {
          sender: 'Hermes',
          text: 'Greetings, Operator. Sethu bridge is online. Chat, Tredo, and Tantra modules are operational.',
          timestamp: Date.now(),
        }
      ]);
      return;
    }

    const remaining = sessions.filter(s => s.id !== sessionId);
    setSessions(remaining);
    
    if (activeSessionId === sessionId) {
      const nextActive = remaining[0];
      setActiveSessionId(nextActive.id);
      setMessages(nextActive.messages);
      setSelectedModel(nextActive.model);
      setSelectedAgent(nextActive.agent);
    }
  };

  // Helper to update active session settings:
  const updateSessionModel = (sessionId: string, model: string) => {
    setSessions(prev => prev.map(s => s.id === sessionId ? { ...s, model } : s));
  };
  const updateSessionAgent = (sessionId: string, agent: string) => {
    setSessions(prev => prev.map(s => s.id === sessionId ? { ...s, agent } : s));
  };

  // Tredo Shared States
  const [watchlist] = useAtom(watchlistAtom);
  const [selectedAsset, setSelectedAsset] = useAtom(selectedAssetAtom);
  const [portfolioVal, setPortfolioVal] = useAtom(portfolioValueAtom);
  const [cash, setCash] = useAtom(cashBalanceAtom);
  const [openOrders, setOpenOrders] = useAtom(openOrdersAtom);
  const [tradesHistory, setTradesHistory] = useAtom(tradesHistoryAtom);
  const [priceHistory, setPriceHistory] = useAtom(priceHistoryAtom);

  // Tantra States
  const [alerts, setAlerts] = useAtom(systemAlertsAtom);
  const [logs, setLogs] = useAtom(serverLogsAtom);
  const [metrics] = useAtom(metricsAtom);
  const [calendarEvents, setCalendarEvents] = useAtom(calendarEventsAtom);
  const [coworkerTasks, setCoworkerTasks] = useAtom(coworkerTasksAtom);
  const [newsFeed] = useAtom(newsFeedAtom);
  const [portfolioHealth] = useAtom(portfolioHealthAtom);
  const [dndActive, setDndActive] = useState(true);

  // Skills States
  const [skillAnalysis, setSkillAnalysis] = useAtom(skillAnalysisAtom);
  const [availableSkills, setAvailableSkills] = useAtom(availableSkillsAtom);
  const [skillAnalyzing, setSkillAnalyzing] = useState(false);

  // Auto-Trading States
  const [autoTradingState, setAutoTradingState] = useAtom(autoTradingStateAtom);
  const [perfStats, setPerfStats] = useAtom(performanceStatsAtom);

  // Fetch available skills on mount
  useEffect(() => {
    const fetchSkills = async () => {
      try {
        const res = await fetch('/api/skills/list');
        const data = await res.json();
        if (data.status === 'success') {
          setAvailableSkills(data.skills);
        }
      } catch {
        console.warn('Backend not responding for skills list');
      }
    };
    fetchSkills();
  }, []);

  // --- Exchange Interactive Local States ---
  const [currentPrice, setCurrentPrice] = useState(BASE_PRICES[selectedAsset]);
  const [orderType, setOrderType] = useState<'LIMIT' | 'MARKET'>('LIMIT');
  const [limitPriceInput, setLimitPriceInput] = useState(BASE_PRICES[selectedAsset].toString());
  const [amountInput, setAmountInput] = useState('0.1');
  const [orderSide, setOrderSide] = useState<'BUY' | 'SELL'>('BUY');
  const [bottomTab, setBottomTab] = useState<'OPEN' | 'HISTORY' | 'ASSETS'>('OPEN');

  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Synchronize dynamic defaults when selected symbol changes
  useEffect(() => {
    const base = BASE_PRICES[selectedAsset] || 100.0;
    setCurrentPrice(base);
    setLimitPriceInput(base.toString());
  }, [selectedAsset]);

  // --- Real-time Price, Trades, and Order Book Simulator ---
  useEffect(() => {
    // 1. Pre-populate mock historical price series for chart on load
    setPriceHistory((prev) => {
      if (prev[selectedAsset] && prev[selectedAsset].length > 0) return prev;
      
      const startPrice = BASE_PRICES[selectedAsset];
      const candles: Candlestick[] = [];
      let lastClose = startPrice - 100;
      
      for (let i = 0; i < 40; i++) {
        const open = lastClose;
        const drift = (Math.random() - 0.48) * (startPrice * 0.003);
        const close = open + drift;
        const high = Math.max(open, close) + Math.random() * (startPrice * 0.001);
        const low = Math.min(open, close) - Math.random() * (startPrice * 0.001);
        candles.push({
          time: Date.now() - (40 - i) * 10000,
          open,
          high,
          low,
          close,
          volume: Math.random() * 5 + 0.5,
        });
        lastClose = close;
      }
      return { ...prev, [selectedAsset]: candles };
    });

    // 2. Continuous real-time fluctuate loop (1 second interval)
    const intervalId = setInterval(() => {
      setCurrentPrice((prevPrice) => {
        const volatility = BASE_PRICES[selectedAsset] * 0.0006;
        const change = (Math.random() - 0.49) * volatility;
        const nextPrice = Math.max(0.01, prevPrice + change);

        // Update latest candle close
        setPriceHistory((history) => {
          const candles = [...(history[selectedAsset] || [])];
          if (candles.length > 0) {
            const lastCandle = { ...candles[candles.length - 1] };
            lastCandle.close = nextPrice;
            if (nextPrice > lastCandle.high) lastCandle.high = nextPrice;
            if (nextPrice < lastCandle.low) lastCandle.low = nextPrice;
            candles[candles.length - 1] = lastCandle;
          }
          return { ...history, [selectedAsset]: candles };
        });

        // Add to Recent Trades feed
        setTradesHistory((prevTrades) => {
          const tradeSize = Math.random() * 2 + 0.01;
          const nextTrade: TradeRecord = {
            id: Math.random().toString(36).substring(7),
            symbol: selectedAsset,
            side: Math.random() > 0.5 ? 'BUY' : 'SELL',
            price: nextPrice,
            amount: Number(tradeSize.toFixed(4)),
            timestamp: Date.now(),
          };
          return [nextTrade, ...prevTrades.slice(0, 19)];
        });

        // Match Open Limit Orders
        setOpenOrders((orders) => {
          const remaining: OpenOrder[] = [];
          orders.forEach((order) => {
            if (order.symbol !== selectedAsset) {
              remaining.push(order);
              return;
            }

            const isFilled =
              (order.side === 'BUY' && nextPrice <= order.price) ||
              (order.side === 'SELL' && nextPrice >= order.price);

            if (isFilled) {
              // Deduct/Add optimistically
              const totalCost = order.price * order.amount;
              if (order.side === 'BUY') {
                // Adjust asset value in portfolio
                setPortfolioVal((v) => v + totalCost * 0.02); // optimistic yield bump
              } else {
                setCash((c) => c + totalCost);
                setPortfolioVal((v) => v + totalCost * 0.01);
              }

              // Log details to systems logger
              setLogs((prevLogs) => [
                `[INFO] Limit Order FILLED: ${order.side} ${order.amount} ${order.symbol} at ${order.price.toFixed(2)}`,
                ...prevLogs,
              ]);

              // Push system notifications alert
              setAlerts((prevAlerts) => [
                {
                  alertId: `F-${Math.random().toString(36).substring(4)}`,
                  source: 'ExecutionEngine',
                  severity: 'Info',
                  message: `Matched ${order.side} order for ${order.amount} ${order.symbol} at $${order.price.toFixed(2)}`,
                  timestamp: Date.now(),
                },
                ...prevAlerts,
              ]);
            } else {
              remaining.push(order);
            }
          });
          return remaining;
        });

        return nextPrice;
      });
    }, 1000);

    return () => clearInterval(intervalId);
  }, [selectedAsset, setPriceHistory, setTradesHistory, setOpenOrders, setLogs, setAlerts, setCash, setPortfolioVal]);

  // --- Interactive Canvas Renderer for Candlestick Price & Vol Chart ---
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    
    // Support high DPI screens
    canvas.width = width * 2;
    canvas.height = height * 2;
    ctx.scale(2, 2);

    ctx.clearRect(0, 0, width, height);

    const candles = priceHistory[selectedAsset] || [];
    if (candles.length === 0) {
      ctx.fillStyle = '#64748b';
      ctx.font = '13px monospace';
      ctx.textAlign = 'center';
      ctx.fillText('Initializing L2 stream ticks...', width / 2, height / 2);
      return;
    }

    const minPrice = Math.min(...candles.map((c) => c.low)) * 0.9992;
    const maxPrice = Math.max((Math.max(...candles.map((c) => c.high))), currentPrice) * 1.0008;
    const priceRange = maxPrice - minPrice;

    // Draw gridlines
    ctx.strokeStyle = 'rgba(30, 41, 59, 0.4)';
    ctx.lineWidth = 0.5;
    for (let i = 1; i < 6; i++) {
      const y = (height / 6) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    const candleWidth = Math.max(2, (width * 0.85) / candles.length - 2);
    const spacing = 2;

    // Draw candles
    candles.forEach((candle, idx) => {
      const x = idx * (candleWidth + spacing) + 30;
      const yOpen = height - ((candle.open - minPrice) / priceRange) * height;
      const yClose = height - ((candle.close - minPrice) / priceRange) * height;
      const yHigh = height - ((candle.high - minPrice) / priceRange) * height;
      const yLow = height - ((candle.low - minPrice) / priceRange) * height;

      const isGreen = candle.close >= candle.open;
      const color = isGreen ? '#00e676' : '#ef4444';

      // Draw wick
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x + candleWidth / 2, yHigh);
      ctx.lineTo(x + candleWidth / 2, yLow);
      ctx.stroke();

      // Draw body
      ctx.fillStyle = color;
      const bodyHeight = Math.max(1, Math.abs(yClose - yOpen));
      ctx.fillRect(x, Math.min(yOpen, yClose), candleWidth, bodyHeight);
    });

    // Draw current ticking price indicator line
    const yCurrent = height - ((currentPrice - minPrice) / priceRange) * height;
    ctx.strokeStyle = 'rgba(157, 78, 221, 0.7)';
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(0, yCurrent);
    ctx.lineTo(width - 70, yCurrent);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw Price Badge tag on the right y-axis
    ctx.fillStyle = 'rgba(18, 24, 38, 0.9)';
    ctx.strokeStyle = '#9d4edd';
    ctx.lineWidth = 1;
    ctx.fillRect(width - 68, yCurrent - 10, 65, 20);
    ctx.strokeRect(width - 68, yCurrent - 10, 65, 20);

    ctx.fillStyle = '#00e676';
    ctx.font = 'bold 9px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(currentPrice.toFixed(2), width - 35, yCurrent + 4);
  }, [priceHistory, selectedAsset, currentPrice]);

  // Simulated Level 2 Bids and Asks generator based on volatility offsets
  const generateL2BidsAndAsks = () => {
    const bids = [];
    const asks = [];
    const step = BASE_PRICES[selectedAsset] * 0.0003;
    
    for (let i = 1; i <= 6; i++) {
      asks.push({
        price: currentPrice + i * step,
        amount: Math.random() * 3.5 + 0.1,
      });
      bids.push({
        price: currentPrice - i * step,
        amount: Math.random() * 3.5 + 0.1,
      });
    }
    // Sort asks descending to show expensive asks at high layout, similar to exchanges
    return { bids, asks: asks.reverse() };
  };

  const l2Data = generateL2BidsAndAsks();

  // --- Order Execution Actions ---
  const handlePlaceOrder = () => {
    const price = orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice;
    const amount = Number(amountInput);
    if (isNaN(price) || price <= 0 || isNaN(amount) || amount <= 0) return;

    const totalCost = price * amount;
    if (orderSide === 'BUY' && totalCost > cash) {
      alert('Insufficient cash balance to place this order!');
      return;
    }

    if (orderType === 'LIMIT') {
      const nextOrder: OpenOrder = {
        id: Math.random().toString(36).substring(7),
        symbol: selectedAsset,
        side: orderSide,
        type: 'LIMIT',
        price,
        amount,
        filled: 0,
        timestamp: Date.now(),
      };

      if (orderSide === 'BUY') {
        setCash((c) => c - totalCost);
      }

      setOpenOrders((prev) => [nextOrder, ...prev]);
      
      setLogs((prevLogs) => [
        `[INFO] Limit Order Placed: ${orderSide} ${amount} ${selectedAsset} at ${price.toFixed(2)}`,
        ...prevLogs,
      ]);
    } else {
      // MARKET order fills instantly!
      if (orderSide === 'BUY') {
        setCash((c) => c - totalCost);
        setPortfolioVal((v) => v + totalCost * 0.02);
      } else {
        setCash((c) => c + totalCost);
        setPortfolioVal((v) => v + totalCost * 0.01);
      }

      setLogs((prevLogs) => [
        `[INFO] Market Order Executed: ${orderSide} ${amount} ${selectedAsset} at ${currentPrice.toFixed(2)}`,
        ...prevLogs,
      ]);

      setAlerts((prevAlerts) => [
        {
          alertId: `M-${Math.random().toString(36).substring(4)}`,
          source: 'ExecutionEngine',
          severity: 'Info',
          message: `Executed Market ${orderSide} order for ${amount} ${selectedAsset} at $${currentPrice.toFixed(2)}`,
          timestamp: Date.now(),
        },
        ...prevAlerts,
      ]);
    }
  };

  const handleCancelOrder = (orderId: string) => {
    setOpenOrders((orders) => {
      const order = orders.find((o) => o.id === orderId);
      if (order && order.side === 'BUY') {
        // Return held cash balance optimistically
        setCash((c) => c + order.price * order.amount);
      }
      
      setLogs((prevLogs) => [
        `[INFO] Limit Order Cancelled: ${orderId}`,
        ...prevLogs,
      ]);
      
      return orders.filter((o) => o.id !== orderId);
    });
  };

  // Fetch auto-trading state periodically
  useEffect(() => {
    const fetchAutoTradeState = async () => {
      try {
        const res = await fetch('/api/autotrade/status');
        const data = await res.json();
        if (data.status === 'success') {
          setAutoTradingState(data.trading_state);
        }
      } catch {
        // Backend not available — skip
      }

      try {
        const res = await fetch('/api/journal/stats');
        const data = await res.json();
        if (data.status === 'success') {
          setPerfStats(data.stats);
        }
      } catch {
        // Skip
      }
    };

    fetchAutoTradeState();
    const interval = setInterval(fetchAutoTradeState, 10000);
    return () => clearInterval(interval);
  }, []);

  // Auto-trading control handlers
  const handleStartAutoTrade = async () => {
    try {
      await fetch('/api/autotrade/start', { method: 'POST' });
      setLogs((prev) => [`[AutoTrading] Autonomous trading loop STARTED`, ...prev]);
      // Refresh state
      const res = await fetch('/api/autotrade/status');
      const data = await res.json();
      if (data.status === 'success') setAutoTradingState(data.trading_state);
    } catch (err) {
      console.error('Failed to start auto-trading:', err);
    }
  };

  const handleStopAutoTrade = async () => {
    try {
      await fetch('/api/autotrade/stop', { method: 'POST' });
      setLogs((prev) => [`[AutoTrading] Autonomous trading loop STOPPED`, ...prev]);
      const res = await fetch('/api/autotrade/status');
      const data = await res.json();
      if (data.status === 'success') setAutoTradingState(data.trading_state);
    } catch (err) {
      console.error('Failed to stop auto-trading:', err);
    }
  };

  // Load TANTRA data from backend
  useEffect(() => {
    const fetchTantraData = async () => {
      try {
        const statusRes = await fetch('/api/tantra/status');
        const statusData = await statusRes.json();
        if (statusData.status === 'success') {
          setDndActive(statusData.dnd_active);
        }

        const calRes = await fetch('/api/tantra/calendar');
        const calData = await calRes.json();
        if (calData.status === 'success') {
          setCalendarEvents(calData.events);
        }

        const tasksRes = await fetch('/api/tantra/tasks');
        const tasksData = await tasksRes.json();
        if (tasksData.status === 'success') {
          setCoworkerTasks(tasksData.tasks);
        }
      } catch (err) {
        console.warn('Backend not responding to Tantra calls, using dynamic mock defaults.');
      }
    };
    
    fetchTantraData();
  }, [activeTab]);

  const handleToggleDnd = async () => {
    const newDnd = !dndActive;
    setDndActive(newDnd);
    try {
      await fetch('/api/tantra/dnd', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active: newDnd }),
      });
      
      setLogs((prev) => [
        `[TantraService] User manual DND schedule guard ${newDnd ? 'ENABLED' : 'DISABLED'}`,
        ...prev,
      ]);
      setAlerts((prev) => [
        {
          alertId: `A-${Math.random()}`,
          source: 'TantraService',
          severity: newDnd ? 'Warning' : 'Info',
          message: `Calendar DND schedule guard has been manually toggled ${newDnd ? 'ON (Trading Suspended)' : 'OFF'}`,
          timestamp: Date.now(),
        },
        ...prev,
      ]);
    } catch (err) {
      console.error(err);
    }
  };

  const handleResolveTask = async (taskId: string) => {
    setCoworkerTasks((prev) => prev.filter((t) => t.id !== taskId));
    try {
      await fetch('/api/tantra/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: taskId }),
      });
      
      setLogs((prev) => [
        `[TantraService] Completed operator safety task review: ${taskId}`,
        ...prev,
      ]);
      setAlerts((prev) => [
        {
          alertId: `A-${Math.random()}`,
          source: 'TantraService',
          severity: 'Info',
          message: `Operator resolved and approved coworker workflow action: ${taskId}`,
          timestamp: Date.now(),
        },
        ...prev,
      ]);
    } catch (err) {
      console.error(err);
    }
  };

  // Helper to append a chat message and keep the active session in sync
  const appendChatMessage = (msg: { sender: 'Operator' | 'Hermes' | 'System'; text: string; timestamp: number }) => {
    setMessages((prev) => {
      const updated = [...prev, msg];
      setSessions((sPrev) => sPrev.map(s => s.id === activeSessionId ? { ...s, messages: updated, timestamp: Date.now() } : s));
      return updated;
    });
  };

  // Chat message submit
  const handleSendMessage = async () => {
    if (!chatInput.trim()) return;
    const prompt = chatInput;
    appendChatMessage({ sender: 'Operator', text: prompt, timestamp: Date.now() });
    setChatInput('');

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

      appendChatMessage({
        sender: 'Hermes',
        text: replyText,
        timestamp: Date.now(),
      });
    } catch (err) {
      appendChatMessage({
        sender: 'Hermes',
        text: `[Local Fallback Mode] Connection failed. Bollinger compression indicates support holds. Conviction is 82%.`,
        timestamp: Date.now(),
      });
    }
  };

  // Skill analysis handler
  const handleRunSkillsAnalysis = async () => {
    if (skillAnalyzing) return;
    setSkillAnalyzing(true);
    
    try {
      const candles = priceHistory[selectedAsset];
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
        
        // Also post a summary to chat
        const analysis = data.analysis as AggregatedAnalysis;
        const dirEmoji = analysis.overall_direction === 'Bullish' ? '🟢' : analysis.overall_direction === 'Bearish' ? '🔴' : '⚪';
        const msg = `${dirEmoji} **Hermes Skills Analysis** for ${analysis.symbol}
Conviction: ${(analysis.overall_conviction * 100).toFixed(0)}% (${analysis.overall_direction})
Signals: ${analysis.bullish_signals} Bullish | ${analysis.bearish_signals} Bearish | ${analysis.neutral_signals} Neutral
Skills Fired: ${analysis.signals.length}/${availableSkills.length} skills triggered`;
        
        appendChatMessage({ sender: 'Hermes', text: msg, timestamp: Date.now() });
        
        setLogs((prev) => [
          `[HermesSkills] Analysis complete for ${analysis.symbol}: ${analysis.overall_direction} (${(analysis.overall_conviction * 100).toFixed(1)}% conviction)`,
          ...prev,
        ]);
      }
    } catch (err) {
      console.error('Skills analysis failed:', err);
      appendChatMessage({
        sender: 'Hermes',
        text: `Skills analysis failed — backend not reachable. Using fallback conviction model.`,
        timestamp: Date.now(),
      });
    } finally {
      setSkillAnalyzing(false);
    }
  };

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-cyber-dark text-slate-100 select-none">
      {/* 1. Main System Header */}
      <header className="flex items-center justify-between px-6 py-4 glass-panel border-b border-cyber-border shadow-lg z-10">
        <div className="flex items-center space-x-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyber-purple to-cyber-glow flex items-center justify-center font-bold text-lg shadow-purple">
            A
          </div>
          <div>
            <h1 className="text-lg font-bold tracking-wider font-mono bg-gradient-to-r from-slate-100 to-slate-400 bg-clip-text text-transparent">
              ARKM COCKPIT
            </h1>
            <p className="text-xs text-cyber-purple font-mono">Sethu Bridge Core v1.0.0</p>
          </div>
        </div>

        <div className="flex items-center space-x-6 text-xs font-mono">
          <div className="flex items-center space-x-2">
            <span className="w-2 h-2 rounded-full bg-cyber-green animate-pulse" />
            <span className="text-slate-400">Sethu Link:</span>
            <span className="text-cyber-green font-bold">ACTIVE</span>
          </div>
          <div className="hidden md:flex items-center space-x-4 border-l border-cyber-border pl-6">
            <div>
              <span className="text-slate-400">CPU:</span>{' '}
              <span className="text-cyber-purple">{metrics.cpu}%</span>
            </div>
            <div>
              <span className="text-slate-400">RAM:</span>{' '}
              <span className="text-cyber-purple">{metrics.memory}%</span>
            </div>
          </div>
        </div>
      </header>

      {/* 2. Navigation Tabs */}
      <nav className="flex justify-start px-6 py-2 bg-cyber-dark border-b border-cyber-border/40 font-mono z-10">
        {(['Chat', 'Tredo', 'Tantra', 'Journal', 'Settings'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-2 text-sm font-semibold rounded-t-lg transition-all duration-300 relative ${
              activeTab === tab
                ? 'text-cyber-purple bg-cyber-panel/50 border-t border-x border-cyber-border'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            {tab}
            {activeTab === tab && (
              <span className="absolute bottom-0 left-0 w-full h-[2px] bg-cyber-purple" />
            )}
          </button>
        ))}
      </nav>

      <main className="flex-1 overflow-hidden p-6 bg-cyber-dark/80">
        {activeTab === 'Chat' && (
          <div className="flex h-full gap-6">
            {/* LEFT SIDEBAR: Chat History (Top) + Intelligence (Bottom) */}
            <div className="w-80 flex flex-col gap-6 h-full shrink-0">
              
              {/* Left Top: Chat History */}
              <div className="glass-panel rounded-xl p-5 flex flex-col flex-grow overflow-hidden min-h-[250px]">
                <div className="flex items-center justify-between mb-4 border-b border-cyber-border/20 pb-2">
                  <h3 className="text-xs font-bold tracking-wider font-mono text-slate-300">CHAT HISTORY</h3>
                  <button
                    onClick={handleNewChat}
                    className="px-2.5 py-1 bg-cyber-purple/10 hover:bg-cyber-purple/20 border border-cyber-purple/30 rounded text-[10px] font-mono text-cyber-purple transition-all duration-200"
                  >
                    + NEW
                  </button>
                </div>
                
                <div className="flex-1 overflow-y-auto space-y-2 pr-1 select-none">
                  {sessions.map((session) => (
                    <div
                      key={session.id}
                      onClick={() => handleSwitchSession(session.id)}
                      className={`group p-3 rounded-lg border font-mono text-xs cursor-pointer transition-all duration-200 ${
                        activeSessionId === session.id
                          ? 'bg-cyber-purple/10 border-cyber-purple/40 text-cyber-purple shadow-purple'
                          : 'bg-cyber-panel/20 border-transparent text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/40'
                      }`}
                    >
                      <div className="flex justify-between items-start gap-1">
                        <span className="font-semibold truncate max-w-[170px]">{session.title}</span>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteSession(session.id);
                          }}
                          className="opacity-0 group-hover:opacity-100 hover:text-red-400 text-[10px] p-0.5"
                          title="Delete Chat"
                        >
                          ✕
                        </button>
                      </div>
                      <div className="flex justify-between items-center text-[9px] text-slate-500 mt-2 font-mono">
                        <span>{session.agent.split(' ')[0]}</span>
                        <span>{new Date(session.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Left Bottom: Intelligence Control Panel */}
              <div className="glass-panel rounded-xl p-5 space-y-5 shrink-0">
                <h3 className="text-xs font-bold tracking-wider font-mono text-slate-300">INTELLIGENCE CONTROL</h3>
                <div className="space-y-3.5 text-xs font-mono">
                  <div>
                    <label className="text-slate-400 block mb-1">Local Ollama Model</label>
                    <select
                      value={selectedModel}
                      onChange={(e) => {
                        setSelectedModel(e.target.value);
                        updateSessionModel(activeSessionId, e.target.value);
                      }}
                      className="w-full bg-cyber-dark border border-cyber-border rounded px-3 py-2 focus:outline-none focus:border-cyber-purple text-slate-200"
                    >
                      <option value="qwen3.5:0.8b">qwen3.5:0.8b</option>
                      <option value="nemotron-3-nano:4b">nemotron-3-nano:4b</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-slate-400 block mb-1">Active Special Agent</label>
                    <select
                      value={selectedAgent}
                      onChange={(e) => {
                        setSelectedAgent(e.target.value);
                        updateSessionAgent(activeSessionId, e.target.value);
                      }}
                      className="w-full bg-cyber-dark border border-cyber-border rounded px-3 py-2 focus:outline-none focus:border-cyber-purple text-slate-200"
                    >
                      <option value="Hermes Tredo">Hermes Tredo (Trading Analyst)</option>
                      <option value="Risk Manager">Risk Manager (Safety Check)</option>
                      <option value="Tantra Monitor">Tantra Monitor (Systems Specialist)</option>
                    </select>
                  </div>
                  
                  {/* Hermes Skills System Launcher */}
                  <div className="border-t border-cyber-border/30 pt-3">
                    <h4 className="text-xs font-bold text-cyber-purple mb-2">HERMES SKILLS SYSTEM</h4>
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-[10px] text-slate-500">
                        <span>Skills Loaded</span>
                        <span className="text-cyber-green font-semibold">{availableSkills.length}</span>
                      </div>
                      <button
                        onClick={handleRunSkillsAnalysis}
                        disabled={skillAnalyzing}
                        className={`w-full py-2.5 rounded-lg text-xs font-bold font-mono transition-all border ${
                          skillAnalyzing
                            ? 'bg-cyber-panel/50 text-slate-500 border-cyber-border/30'
                            : 'bg-cyber-purple/20 hover:bg-cyber-purple/30 text-cyber-purple border-cyber-purple/40 shadow-purple'
                        }`}
                      >
                        {skillAnalyzing ? 'ANALYZING...' : 'RUN SKILLS ANALYSIS'}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* MIDDLE/RIGHT: Main Chat area & dynamic Skills Results panel */}
            <div className="flex-1 flex gap-6 h-full overflow-hidden">
              
              {/* Chat Interface */}
              <div className="flex-1 flex flex-col glass-panel rounded-xl overflow-hidden p-6 h-full">
                <div className="flex-1 overflow-y-auto space-y-4 mb-4 pr-2">
                  {messages.map((msg, i) => (
                    <div
                      key={i}
                      className={`flex flex-col max-w-[80%] rounded-lg p-3 ${
                        msg.sender === 'Operator'
                          ? 'bg-cyber-purple/20 border border-cyber-purple/30 self-end ml-auto'
                          : 'bg-cyber-panel/60 border border-cyber-border/40 self-start'
                      }`}
                    >
                      <span className="text-[10px] text-slate-400 font-mono mb-1">{msg.sender}</span>
                      <p className="text-sm leading-relaxed whitespace-pre-wrap">{msg.text}</p>
                    </div>
                  ))}
                </div>

                <div className="flex items-center space-x-3 border-t border-cyber-border/50 pt-4 shrink-0">
                  <input
                    type="text"
                    value={chatInput}
                    onChange={(e) => setChatInput(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSendMessage()}
                    placeholder="Ask Hermes to inspect orders, build automation bot scripts, or research risk templates..."
                    className="flex-1 bg-cyber-dark/60 border border-cyber-border rounded-lg px-4 py-3 text-sm focus:outline-none focus:border-cyber-purple font-mono"
                  />
                  <button
                    onClick={handleSendMessage}
                    className="px-6 py-3 bg-cyber-purple hover:bg-cyber-glow transition-colors duration-200 rounded-lg text-sm font-semibold font-mono"
                  >
                    SEND
                  </button>
                </div>
              </div>

              {/* Skills Analysis Results Panel */}
              {skillAnalysis && (
                <div className="w-80 glass-panel rounded-xl p-4 flex flex-col overflow-hidden h-full max-h-full shrink-0">
                  <div className="flex items-center justify-between border-b border-cyber-border/20 pb-2 mb-3">
                    <h4 className="text-[10px] font-bold font-mono tracking-wider text-slate-400">
                      SKILLS ANALYSIS: {skillAnalysis.symbol}
                    </h4>
                    <span className={`text-[10px] font-bold font-mono px-2 py-0.5 rounded border ${
                      skillAnalysis.overall_direction === 'Bullish'
                        ? 'text-cyber-green border-cyber-green/40 bg-cyber-green/10'
                        : skillAnalysis.overall_direction === 'Bearish'
                        ? 'text-red-400 border-red-400/40 bg-red-500/10'
                        : 'text-slate-400 border-slate-400/40 bg-slate-500/10'
                    }`}>
                      {skillAnalysis.overall_direction}
                    </span>
                  </div>

                  {/* Conviction Meter */}
                  <div className="mb-3">
                    <div className="flex justify-between text-[9px] font-mono text-slate-500 mb-1">
                      <span>Conviction</span>
                      <span className={skillAnalysis.overall_conviction > 0 ? 'text-cyber-green' : 'text-red-400'}>
                        {(skillAnalysis.overall_conviction * 100).toFixed(0)}%
                      </span>
                    </div>
                    <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all ${
                          skillAnalysis.overall_conviction > 0.3 ? 'bg-cyber-green'
                          : skillAnalysis.overall_conviction < -0.3 ? 'bg-red-500'
                          : 'bg-slate-400'
                        }`}
                        style={{
                          width: `${Math.abs(skillAnalysis.overall_conviction * 100)}%`,
                          marginLeft: skillAnalysis.overall_conviction < 0 ? `${((1 - Math.abs(skillAnalysis.overall_conviction)) / 2) * 100}%` : '0%',
                        }}
                      />
                    </div>
                  </div>

                  {/* Signal Counters */}
                  <div className="grid grid-cols-3 gap-2 mb-3">
                    <div className="bg-cyber-green/10 border border-cyber-green/30 rounded p-1.5 text-center">
                      <span className="text-cyber-green text-[10px] font-bold">{skillAnalysis.bullish_signals}</span>
                      <span className="text-[8px] text-slate-500 block">Bullish</span>
                    </div>
                    <div className="bg-red-500/10 border border-red-500/30 rounded p-1.5 text-center">
                      <span className="text-red-400 text-[10px] font-bold">{skillAnalysis.bearish_signals}</span>
                      <span className="text-[8px] text-slate-500 block">Bearish</span>
                    </div>
                    <div className="bg-slate-500/10 border border-slate-500/30 rounded p-1.5 text-center">
                      <span className="text-slate-400 text-[10px] font-bold">{skillAnalysis.neutral_signals}</span>
                      <span className="text-[8px] text-slate-500 block">Neutral</span>
                    </div>
                  </div>

                  {/* Individual Signals */}
                  <div className="flex-1 overflow-y-auto space-y-1.5 pr-1">
                    {skillAnalysis.signals.map((signal, idx) => (
                      <div key={idx} className="p-2 bg-cyber-dark/40 border border-cyber-border/30 rounded-lg">
                        <div className="flex justify-between items-center mb-0.5">
                          <span className="text-[9px] font-mono text-slate-300 font-semibold truncate mr-1">
                            {signal.skill_name}
                          </span>
                          <span className={`text-[8px] font-bold font-mono px-1 py-0.5 rounded shrink-0 ${
                            signal.direction === 'Bullish'
                              ? 'text-cyber-green bg-cyber-green/10'
                              : signal.direction === 'Bearish'
                              ? 'text-red-400 bg-red-500/10'
                              : 'text-slate-400 bg-slate-500/10'
                          }`}>
                            {signal.direction === 'Bullish' ? '▲' : signal.direction === 'Bearish' ? '▼' : '◆'} {signal.direction}
                          </span>
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
        )}

        {/* --- FULL TRADING EXCHANGE COCKPIT --- */}
        {activeTab === 'Tredo' && (
          <div className="grid grid-cols-12 gap-6 h-full overflow-hidden">
            
            {/* COLUMN 1: Watchlist & Recent Trades Feed (3 cols) */}
            <div className="col-span-3 flex flex-col gap-6 h-full overflow-hidden">
              {/* Asset Watchlist */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[45%] overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">WATCHLIST</h3>
                <div className="flex-1 overflow-y-auto space-y-1.5 pr-1">
                  {watchlist.map((asset) => (
                    <button
                      key={asset}
                      onClick={() => setSelectedAsset(asset)}
                      className={`w-full text-left px-3 py-2.5 rounded-lg font-mono text-xs flex justify-between items-center transition-all ${
                        selectedAsset === asset
                          ? 'bg-cyber-purple/20 border border-cyber-purple/40 text-cyber-purple'
                          : 'bg-cyber-panel/30 border border-transparent text-slate-400 hover:text-slate-200'
                      }`}
                    >
                      <span>{asset}</span>
                      <span className="text-cyber-green font-semibold">+1.42%</span>
                    </button>
                  ))}
                </div>
              </div>

              {/* Scrolling Executed Trades Feed */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[55%] overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">RECENT MARKET TRADES</h3>
                <div className="flex-grow overflow-hidden relative">
                  <table className="w-full text-[10px] font-mono text-left">
                    <thead>
                      <tr className="text-slate-500 border-b border-cyber-border/40">
                        <th className="pb-1.5">Price</th>
                        <th className="pb-1.5 text-right">Amount</th>
                        <th className="pb-1.5 text-right">Time</th>
                      </tr>
                    </thead>
                  </table>
                  <div className="h-full overflow-y-auto pr-1 space-y-1 mt-1 text-[10px]">
                    {tradesHistory.filter(t => t.symbol === selectedAsset).map((trade) => (
                      <div key={trade.id} className="flex justify-between font-mono py-0.5 border-b border-cyber-border/10">
                        <span className={trade.side === 'BUY' ? 'text-cyber-green' : 'text-red-400'}>
                          {trade.price.toFixed(2)}
                        </span>
                        <span className="text-slate-300 text-right">{trade.amount.toFixed(4)}</span>
                        <span className="text-slate-500 text-right">
                          {new Date(trade.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* COLUMN 2: Live Canvas Candlestick Chart & Bottom Console Tab Ledger (6 cols) */}
            <div className="col-span-6 flex flex-col gap-6 h-full overflow-hidden">
              {/* Candlestick Analytics Workspace */}
              <div className="flex-1 glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
                <div className="flex justify-between items-center border-b border-cyber-border/40 pb-3 mb-3">
                  <div className="flex items-center space-x-3">
                    <h3 className="text-sm font-bold font-mono tracking-wider text-slate-200">
                      {selectedAsset}
                    </h3>
                    <span className="text-xs text-cyber-purple font-mono px-2 py-0.5 bg-cyber-purple/10 rounded">
                      Live Candlesticks
                    </span>
                  </div>
                  <div className="flex space-x-3 text-xs font-mono items-center">
                    <span className="text-slate-400">Current Value:</span>
                    <span className="px-2.5 py-1 bg-cyber-panel rounded border border-cyber-border text-cyber-green font-bold shadow-green">
                      ${currentPrice.toFixed(2)}
                    </span>
                  </div>
                </div>

                {/* Pure Canvas Candle Chart */}
                <div className="flex-1 relative bg-cyber-dark/40 border border-cyber-border/40 rounded-lg overflow-hidden">
                  <canvas ref={canvasRef} className="absolute inset-0 w-full h-full" />
                </div>
              </div>

              {/* Bottom Ledger Dashboard Console */}
              <div className="h-56 glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
                <div className="flex space-x-3 border-b border-cyber-border/30 pb-2 mb-3">
                  {(['OPEN', 'HISTORY', 'ASSETS'] as const).map((tab) => (
                    <button
                      key={tab}
                      onClick={() => setBottomTab(tab)}
                      className={`text-xs font-bold font-mono tracking-wider px-3 py-1 rounded transition-colors ${
                        bottomTab === tab
                          ? 'bg-cyber-purple/20 text-cyber-purple border border-cyber-purple/40'
                          : 'text-slate-400 hover:text-slate-200'
                      }`}
                    >
                      {tab === 'OPEN' ? 'OPEN ORDERS' : tab === 'HISTORY' ? 'SYSTEM ALERTS' : 'LEDGER ASSETS'}
                    </button>
                  ))}
                </div>

                <div className="flex-1 overflow-y-auto text-xs font-mono">
                  {bottomTab === 'OPEN' && (
                    <table className="w-full text-left">
                      <thead>
                        <tr className="text-slate-500 border-b border-cyber-border/20 text-[10px]">
                          <th>Symbol</th>
                          <th>Side</th>
                          <th>Price</th>
                          <th>Amount</th>
                          <th>Total</th>
                          <th>Action</th>
                        </tr>
                      </thead>
                      <tbody>
                        {openOrders.length === 0 ? (
                          <tr>
                            <td colSpan={6} className="text-center py-6 text-slate-500">
                              No active open limit orders. Place one below.
                            </td>
                          </tr>
                        ) : (
                          openOrders.map((order) => (
                            <tr key={order.id} className="border-b border-cyber-border/10 py-2">
                              <td className="py-2 text-slate-300 font-bold">{order.symbol}</td>
                              <td className={`py-2 font-bold ${order.side === 'BUY' ? 'text-cyber-green' : 'text-red-400'}`}>
                                {order.side}
                              </td>
                              <td className="py-2 text-slate-300">${order.price.toFixed(2)}</td>
                              <td className="py-2 text-slate-300">{order.amount}</td>
                              <td className="py-2 text-cyber-purple font-semibold">
                                ${(order.price * order.amount).toFixed(2)}
                              </td>
                              <td className="py-2">
                                <button
                                  onClick={() => handleCancelOrder(order.id)}
                                  className="px-2.5 py-1 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 rounded text-[10px] transition-colors"
                                >
                                  CANCEL
                                </button>
                              </td>
                            </tr>
                          ))
                        )}
                      </tbody>
                    </table>
                  )}

                  {bottomTab === 'HISTORY' && (
                    <div className="space-y-1.5">
                      {alerts.map((alert) => (
                        <div key={alert.alertId} className="flex justify-between border-b border-cyber-border/10 py-1">
                          <span className="text-slate-300">{alert.message}</span>
                          <span className="text-slate-500 text-[10px]">
                            {new Date(alert.timestamp).toLocaleTimeString()}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}

                  {bottomTab === 'ASSETS' && (
                    <div className="grid grid-cols-3 gap-4 p-2">
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">TOTAL NET WORTH</span>
                        <p className="text-base font-bold text-cyber-green mt-1">${portfolioVal.toLocaleString()}</p>
                      </div>
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">LIQUID CASH</span>
                        <p className="text-base font-bold text-slate-200 mt-1">${cash.toLocaleString()}</p>
                      </div>
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">ACTIVE TRADING ENGINE</span>
                        <p className="text-xs font-bold text-cyber-purple mt-2">Sethu ExecutionEngine</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* COLUMN 3: Dual-sided real-time L2 Order Book & Buy/Sell Order ticket Desk (3 cols) */}
            <div className="col-span-3 flex flex-col gap-6 h-full overflow-hidden">
              {/* Auto-Trading Control Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
                <div className="flex justify-between items-center border-b border-cyber-border/30 pb-2 mb-3">
                  <h3 className="text-xs font-bold font-mono tracking-wider text-cyber-purple">AUTONOMOUS TRADING</h3>
                  <div className="flex items-center space-x-2">
                    <span className={`w-2 h-2 rounded-full ${autoTradingState?.enabled ? 'bg-cyber-green animate-pulse' : 'bg-slate-500'}`} />
                    <span className="text-[10px] font-mono text-slate-400">
                      {autoTradingState?.enabled ? 'ACTIVE' : 'PAUSED'}
                    </span>
                  </div>
                </div>
                
                {/* Controls */}
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-2 text-[10px] font-mono">
                    <span className="text-slate-500">Mode:</span>
                    <span className={`px-2 py-0.5 rounded border text-[9px] ${
                      autoTradingState?.paper_trading
                        ? 'bg-yellow-500/10 text-yellow-400 border-yellow-500/30'
                        : 'bg-red-500/10 text-red-400 border-red-500/30'
                    }`}>
                      {autoTradingState?.paper_trading ? 'PAPER' : 'REAL'}
                    </span>
                  </div>
                  <div className="flex space-x-2">
                    <button
                      onClick={handleStartAutoTrade}
                      disabled={autoTradingState?.enabled}
                      className={`px-3 py-1.5 text-[10px] font-bold font-mono rounded transition-all border ${
                        autoTradingState?.enabled
                          ? 'bg-cyber-green/10 text-cyber-green/50 border-cyber-green/20 cursor-not-allowed'
                          : 'bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border-cyber-green/40 shadow-green'
                      }`}
                    >
                      START
                    </button>
                    <button
                      onClick={handleStopAutoTrade}
                      disabled={!autoTradingState?.enabled}
                      className={`px-3 py-1.5 text-[10px] font-bold font-mono rounded transition-all border ${
                        !autoTradingState?.enabled
                          ? 'bg-red-500/10 text-red-400/50 border-red-500/20 cursor-not-allowed'
                          : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border-red-500/40'
                      }`}
                    >
                      STOP
                    </button>
                  </div>
                </div>

                {/* Status info */}
                <div className="grid grid-cols-2 gap-2 text-[9px] font-mono">
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Balance</span>
                    <span className="text-cyber-green font-bold">
                      ${autoTradingState?.balance?.toLocaleString() ?? '100,000'}
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Positions</span>
                    <span className="text-slate-200 font-bold">
                      {autoTradingState?.open_positions?.length ?? 0}
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Drawdown</span>
                    <span className={`font-bold ${(autoTradingState?.current_drawdown_pct ?? 0) > 10 ? 'text-red-400' : 'text-slate-200'}`}>
                      {autoTradingState?.current_drawdown_pct?.toFixed(1) ?? '0.0'}%
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Interval</span>
                    <span className="text-cyber-purple font-bold">
                      {autoTradingState?.analysis_interval_secs ?? 300}s
                    </span>
                  </div>
                </div>

                {/* Last decisions */}
                {autoTradingState?.last_outcomes && autoTradingState.last_outcomes.length > 0 && (
                  <div className="mt-3">
                    <h4 className="text-[9px] font-bold text-slate-500 mb-1.5 font-mono">RECENT DECISIONS</h4>
                    <div className="max-h-24 overflow-y-auto space-y-1">
                      {autoTradingState.last_outcomes.slice(-5).reverse().map((outcome, i) => (
                        <div key={i} className="flex justify-between items-center text-[8px] font-mono bg-cyber-dark/30 rounded px-2 py-1">
                          <span className="text-slate-400 truncate max-w-[100px]">{outcome.symbol}</span>
                          <span className={`font-bold ${
                            outcome.action?.Buy ? 'text-cyber-green' :
                            outcome.action?.Sell ? 'text-red-400' :
                            outcome.action?.Hold ? 'text-slate-400' : 'text-yellow-400'
                          }`}>
                            {outcome.action?.Buy ? 'BUY' : outcome.action?.Sell ? 'SELL' : outcome.action?.Hold ? 'HOLD' : 'SKIP'}
                          </span>
                          <span className="text-slate-500">{outcome.regime}</span>
                          <span className={outcome.conviction > 0 ? 'text-cyber-green' : 'text-red-400'}>
                            {(outcome.conviction * 100).toFixed(0)}%
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Performance Stats */}
                {perfStats && (
                  <div className="mt-3 border-t border-cyber-border/20 pt-3">
                    <h4 className="text-[9px] font-bold text-slate-500 mb-1.5 font-mono">PERFORMANCE</h4>
                    <div className="grid grid-cols-3 gap-1.5">
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className="text-[10px] font-bold text-cyber-green block">{perfStats.win_rate.toFixed(1)}%</span>
                        <span className="text-[7px] text-slate-500 font-mono">Win Rate</span>
                      </div>
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className="text-[10px] font-bold text-cyber-purple block">{perfStats.total_trades}</span>
                        <span className="text-[7px] text-slate-500 font-mono">Trades</span>
                      </div>
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className={`text-[10px] font-bold block ${perfStats.total_pnl >= 0 ? 'text-cyber-green' : 'text-red-400'}`}>
                          ${perfStats.total_pnl.toFixed(0)}
                        </span>
                        <span className="text-[7px] text-slate-500 font-mono">Total P&L</span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
              {/* Vertical Level 2 Orderbook Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[50%] overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">ORDER BOOK</h3>
                <div className="flex-1 flex flex-col overflow-hidden text-[10px] font-mono">
                  {/* Asks (Sell orders) red */}
                  <div className="flex-1 overflow-y-auto space-y-0.5 mb-1 flex flex-col justify-end">
                    {l2Data.asks.map((ask, idx) => (
                      <div key={idx} className="flex justify-between py-0.5 relative">
                        <div
                          className="absolute right-0 top-0 bottom-0 bg-red-500/5 pointer-events-none"
                          style={{ width: `${Math.min(100, (ask.amount / 3.5) * 100)}%` }}
                        />
                        <span className="text-red-400 z-10">${ask.price.toFixed(2)}</span>
                        <span className="text-slate-400 z-10 text-right">{ask.amount.toFixed(4)}</span>
                      </div>
                    ))}
                  </div>

                  {/* Spread indicator bar */}
                  <div className="py-1 border-y border-cyber-border/40 text-center font-mono my-1 text-cyber-purple font-semibold bg-cyber-panel/30">
                    Spread: 0.05%
                  </div>

                  {/* Bids (Buy orders) green */}
                  <div className="flex-1 overflow-y-auto space-y-0.5 mt-1">
                    {l2Data.bids.map((bid, idx) => (
                      <div key={idx} className="flex justify-between py-0.5 relative">
                        <div
                          className="absolute right-0 top-0 bottom-0 bg-cyber-green/5 pointer-events-none"
                          style={{ width: `${Math.min(100, (bid.amount / 3.5) * 100)}%` }}
                        />
                        <span className="text-cyber-green z-10">${bid.price.toFixed(2)}</span>
                        <span className="text-slate-400 z-10 text-right">{bid.amount.toFixed(4)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Order Form Widget Desk Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[50%] overflow-hidden">
                <div className="flex border-b border-cyber-border/40 pb-2 mb-3 justify-between">
                  <div className="flex space-x-1.5">
                    {(['LIMIT', 'MARKET'] as const).map((t) => (
                      <button
                        key={t}
                        onClick={() => setOrderType(t)}
                        className={`text-[10px] font-bold font-mono px-2 py-0.5 rounded ${
                          orderType === t ? 'bg-cyber-panel border border-cyber-border text-slate-200' : 'text-slate-500 hover:text-slate-300'
                        }`}
                      >
                        {t}
                      </button>
                    ))}
                  </div>
                  <div className="flex space-x-1">
                    <button
                      onClick={() => setOrderSide('BUY')}
                      className={`text-[10px] font-bold font-mono px-2.5 py-0.5 rounded ${
                        orderSide === 'BUY' ? 'bg-cyber-green/20 text-cyber-green border border-cyber-green/30' : 'text-slate-500'
                      }`}
                    >
                      BUY
                    </button>
                    <button
                      onClick={() => setOrderSide('SELL')}
                      className={`text-[10px] font-bold font-mono px-2.5 py-0.5 rounded ${
                        orderSide === 'SELL' ? 'bg-red-500/20 text-red-400 border border-red-500/30' : 'text-slate-500'
                      }`}
                    >
                      SELL
                    </button>
                  </div>
                </div>

                <div className="flex-1 flex flex-col justify-between text-xs font-mono space-y-3">
                  <div className="space-y-2">
                    {orderType === 'LIMIT' && (
                      <div>
                        <label className="text-[10px] text-slate-500 block mb-1">LIMIT PRICE (USD)</label>
                        <div className="flex">
                          <button
                            onClick={() => setLimitPriceInput((p) => Math.max(0, Number(p) - 1).toString())}
                            className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-l text-[10px]"
                          >
                            -
                          </button>
                          <input
                            type="text"
                            value={limitPriceInput}
                            onChange={(e) => setLimitPriceInput(e.target.value)}
                            className="w-full bg-cyber-dark/60 border-y border-cyber-border text-center text-slate-200 focus:outline-none focus:border-cyber-purple font-mono py-1"
                          />
                          <button
                            onClick={() => setLimitPriceInput((p) => (Number(p) + 1).toString())}
                            className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-r text-[10px]"
                          >
                            +
                          </button>
                        </div>
                      </div>
                    )}

                    <div>
                      <label className="text-[10px] text-slate-500 block mb-1">QUANTITY</label>
                      <input
                        type="text"
                        value={amountInput}
                        onChange={(e) => setAmountInput(e.target.value)}
                        className="w-full bg-cyber-dark/60 border border-cyber-border rounded text-center text-slate-200 focus:outline-none focus:border-cyber-purple font-mono py-1"
                      />
                    </div>

                    {/* Percentage buttons */}
                    <div className="grid grid-cols-4 gap-1">
                      {[0.25, 0.5, 0.75, 1.0].map((pct) => (
                        <button
                          key={pct}
                          onClick={() => {
                            const maxCost = cash;
                            const price = orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice;
                            if (price > 0) {
                              const qty = (maxCost * pct) / price;
                              setAmountInput(qty.toFixed(4));
                            }
                          }}
                          className="bg-cyber-panel hover:bg-cyber-border/40 text-slate-400 border border-cyber-border rounded py-0.5 text-[9px]"
                        >
                          {pct * 100}%
                        </button>
                      ))}
                    </div>

                    <div className="border-t border-cyber-border/20 pt-2 flex justify-between text-[10px] text-slate-500">
                      <span>EST. TOTAL VALUE:</span>
                      <span className="text-cyber-purple font-bold">
                        ${((orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice) * Number(amountInput || 0)).toFixed(2)}
                      </span>
                    </div>
                  </div>

                  <button
                    onClick={handlePlaceOrder}
                    className={`w-full py-2.5 font-bold font-mono rounded text-sm transition-all shadow-md ${
                      orderSide === 'BUY'
                        ? 'bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border border-cyber-green/40 shadow-green'
                        : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/40'
                    }`}
                  >
                    PLACE {orderSide} {orderType} ORDER
                  </button>
                </div>
              </div>
            </div>
            
          </div>
        )}

        {activeTab === 'Journal' && (
          <div className="flex h-full">
            <Journal />
          </div>
        )}

        {activeTab === 'Settings' && (
          <div className="flex h-full">
            <Settings />
          </div>
        )}

        {activeTab === 'Tantra' && (
          <div className="flex h-full gap-6 overflow-hidden">
            {/* 1. Left safety guard desk (Calendar + News) */}
            <div className="w-96 flex flex-col gap-6 overflow-hidden">
              {/* Google Calendar DND guard card */}
              <div className="flex-1 glass-panel rounded-xl p-5 flex flex-col overflow-hidden">
                <div className="flex justify-between items-center mb-4 border-b border-cyber-border/20 pb-2">
                  <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300">CALENDAR DND GUARD</h3>
                  <div className="flex items-center space-x-2">
                    <span className={`w-2.5 h-2.5 rounded-full ${dndActive ? 'bg-red-500 animate-pulse shadow-red' : 'bg-cyber-green shadow-green'}`} />
                    <span className="text-[10px] font-mono text-slate-400">{dndActive ? 'TRADING BLOCKED' : 'EXECUTION LIVE'}</span>
                  </div>
                </div>

                <div className="mb-4 bg-cyber-dark/40 border border-cyber-border/40 rounded-lg p-3 flex items-center justify-between">
                  <div className="flex flex-col gap-0.5">
                    <span className="text-[10px] font-mono text-slate-500">MANUAL SHIELD OVERRIDE</span>
                    <span className="text-xs font-bold text-slate-300 font-mono">{dndActive ? 'Suspended' : 'Armed'}</span>
                  </div>
                  <button
                    onClick={handleToggleDnd}
                    className={`px-3 py-1.5 rounded font-mono text-xs font-bold transition-all border ${
                      dndActive 
                        ? 'bg-cyber-purple/20 hover:bg-cyber-purple/30 text-cyber-purple border-cyber-purple/40 shadow-purple'
                        : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border-red-500/40'
                    }`}
                  >
                    {dndActive ? 'RESUME TRADING' : 'FORCE PAUSE DND'}
                  </button>
                </div>

                <div className="flex-1 overflow-y-auto space-y-3 pr-1">
                  {calendarEvents.map((evt) => (
                    <div key={evt.id} className="p-3 bg-cyber-panel/40 border border-cyber-border/40 rounded-lg flex flex-col gap-1.5">
                      <div className="flex justify-between items-center text-[10px] font-mono">
                        <span className="text-cyber-glow">{evt.start} - {evt.end}</span>
                        <span className={`px-1.5 py-0.5 rounded-full border text-[9px] ${
                          evt.isDnd 
                            ? 'bg-red-500/10 text-red-400 border-red-500/20' 
                            : 'bg-slate-500/10 text-slate-400 border-slate-500/20'
                        }`}>
                          {evt.isDnd ? 'DND GUARD' : 'STANDARD'}
                        </span>
                      </div>
                      <h4 className="text-xs font-bold text-slate-300">{evt.title}</h4>
                    </div>
                  ))}
                </div>
              </div>

              {/* Live News Feed Card */}
              <div className="h-64 glass-panel rounded-xl p-5 flex flex-col overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">COWORKER NEWS FEED</h3>
                <div className="flex-1 overflow-y-auto space-y-3 pr-1">
                  {newsFeed.map((news) => (
                    <div key={news.id} className="p-2.5 bg-cyber-dark/40 border border-cyber-border/40 rounded-lg flex flex-col gap-1">
                      <div className="flex justify-between items-center text-[9px] font-mono">
                        <span className="text-slate-500">{news.source}</span>
                        <span className={`px-1.5 rounded-full border text-[8px] ${
                          news.impact === 'HIGH' 
                            ? 'bg-red-500/20 text-red-400 border-red-500/30' 
                            : 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
                        }`}>
                          {news.impact} IMPACT
                        </span>
                      </div>
                      <p className="text-[11px] text-slate-300 leading-snug">{news.headline}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* 2. Center coworker workflow queue & Portfolio Health */}
            <div className="flex-1 flex flex-col gap-6 overflow-hidden">
              {/* Coworker interactive workflow queue */}
              <div className="flex-1 glass-panel rounded-xl p-5 flex flex-col overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-4 border-b border-cyber-border/20 pb-2">
                  PENDING COWORKER COORDINATION QUEUE ({coworkerTasks.length})
                </h3>

                {coworkerTasks.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center text-slate-500 font-mono text-xs gap-2">
                    <span className="text-cyber-green text-lg animate-pulse">✓</span>
                    <span>All coworker tasks resolved. System fully synchronized.</span>
                  </div>
                ) : (
                  <div className="flex-1 overflow-y-auto space-y-3 pr-1">
                    {coworkerTasks.map((task) => (
                      <div key={task.id} className="p-4 bg-cyber-panel/60 border border-cyber-border rounded-xl flex items-center justify-between gap-4">
                        <div className="flex-1 flex flex-col gap-1.5">
                          <div className="flex items-center space-x-2">
                            <span className={`px-2 py-0.5 rounded text-[9px] font-mono border ${
                              task.priority === 'HIGH' 
                                ? 'bg-red-500/10 text-red-400 border-red-500/20' 
                                : 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20'
                            }`}>
                              {task.priority}
                            </span>
                            <span className="text-[10px] font-mono text-slate-500">{task.category}</span>
                          </div>
                          <h4 className="text-xs font-bold text-slate-200">{task.title}</h4>
                          <p className="text-[11px] text-slate-400 leading-relaxed">{task.description}</p>
                        </div>
                        <button
                          onClick={() => handleResolveTask(task.id)}
                          className="px-4 py-2 bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border border-cyber-green/40 shadow-green text-[11px] font-mono font-bold rounded transition-all shrink-0"
                        >
                          APPROVE & ALIGN
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Portfolio exposure gauges */}
              <div className="h-48 glass-panel rounded-xl p-5 flex flex-col justify-between">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">PORTFOLIO EXPOSURE HEALTH</h3>
                <div className="grid grid-cols-4 gap-4 flex-1 items-center">
                  <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3.5 flex flex-col gap-1">
                    <span className="text-[9px] font-mono text-slate-500">MARGIN EXPOSURE</span>
                    <span className="text-lg font-bold text-cyber-purple font-mono">{portfolioHealth.marginRatio}%</span>
                    <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                      <div className="bg-cyber-purple h-1.5 rounded-full" style={{ width: `${portfolioHealth.marginRatio}%` }} />
                    </div>
                  </div>
                  <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3.5 flex flex-col gap-1">
                    <span className="text-[9px] font-mono text-slate-500">RISK INDEX (VOL)</span>
                    <span className="text-lg font-bold text-cyber-glow font-mono">{portfolioHealth.riskIndex} <span className="text-[10px] text-slate-600">/ 1.00</span></span>
                    <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                      <div className="bg-cyber-glow h-1.5 rounded-full" style={{ width: `${portfolioHealth.riskIndex * 100}%` }} />
                    </div>
                  </div>
                  <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3.5 flex flex-col gap-1">
                    <span className="text-[9px] font-mono text-slate-500">DAILY YIELD ACCRUAL</span>
                    <span className="text-lg font-bold text-cyber-green font-mono">+{portfolioHealth.dailyYield}%</span>
                    <span className="text-[9px] text-slate-500 font-mono">Realized P&L: +$2,450</span>
                  </div>
                  <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3.5 flex flex-col gap-1">
                    <span className="text-[9px] font-mono text-slate-500">RISK VALUE LIMIT (VaR)</span>
                    <span className="text-lg font-bold text-red-400 font-mono">${portfolioHealth.valueAtRisk.toLocaleString()}</span>
                    <span className="text-[9px] text-red-400/80 font-mono">Max allowable loss guard</span>
                  </div>
                </div>
              </div>
            </div>

            {/* 3. Right pane Alarms & Logs */}
            <div className="w-80 flex flex-col gap-6 overflow-hidden">
              <div className="flex-1 glass-panel rounded-xl p-5 flex flex-col overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">SYSTEM ALARMS</h3>
                <div className="flex-1 overflow-y-auto space-y-2 pr-1 font-mono text-[10px]">
                  {alerts.map((alert) => (
                    <div key={alert.alertId} className="p-2.5 bg-cyber-panel/40 border border-cyber-border/40 rounded-lg flex flex-col gap-1">
                      <div className="flex justify-between items-center">
                        <span className="text-[9px] text-cyber-purple font-bold">{alert.source}</span>
                        <span className={`px-1.5 py-0.5 rounded-full border text-[8px] ${
                          alert.severity === 'Critical' 
                            ? 'bg-red-500/20 text-red-400 border-red-500/30' 
                            : 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30'
                        }`}>
                          {alert.severity}
                        </span>
                      </div>
                      <p className="text-slate-300 leading-relaxed">{alert.message}</p>
                    </div>
                  ))}
                </div>
              </div>

              <div className="h-64 glass-panel rounded-xl p-5 flex flex-col overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-300 mb-3 border-b border-cyber-border/20 pb-2">COWORKER CORE AUDIT</h3>
                <div className="flex-1 bg-cyber-dark/80 border border-cyber-border rounded-lg p-3 font-mono text-[9px] overflow-y-auto space-y-1 text-cyber-green">
                  {logs.map((log, i) => (
                    <div key={i} className="leading-normal">{log}</div>
                  ))}
                  <div>$ tail -f sethu_bridge.log...</div>
                </div>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
