import { atom } from 'jotai';
import { OrderBook, Trade, TantraAlert } from '../../../protocols/ts';

// Navigation active module tab
export type ActiveModule = 'Chat' | 'Tredo' | 'Tantra' | 'Journal' | 'Settings';
export const activeModuleAtom = atom<ActiveModule>('Chat');

// --- CHAT STATE ---
export interface ChatMessage {
  sender: 'Operator' | 'Hermes' | 'System';
  text: string;
  timestamp: number;
}
export const chatMessagesAtom = atom<ChatMessage[]>([
  {
    sender: 'Hermes',
    text: 'Greetings, Operator. Sethu bridge is online. Chat, Tredo, and Tantra modules are operational.',
    timestamp: Date.now(),
  }
]);
export const chatInputAtom = atom<string>('');
export const selectedModelAtom = atom<string>('qwen3.5:0.8b');
export const selectedAgentAtom = atom<string>('Hermes Tredo');

// --- TREDO STATE ---
export interface OpenOrder {
  id: string;
  symbol: string;
  side: 'BUY' | 'SELL';
  type: 'LIMIT' | 'MARKET';
  price: number;
  amount: number;
  filled: number;
  timestamp: number;
}

export interface TradeRecord {
  id: string;
  symbol: string;
  side: 'BUY' | 'SELL';
  price: number;
  amount: number;
  timestamp: number;
}

export interface Candlestick {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export const watchlistAtom = atom<string[]>(['BTC-USD', 'ETH-USD', 'SOL-USD', 'XAU-USD']);
export const selectedAssetAtom = atom<string>('BTC-USD');
export const orderBookAtom = atom<OrderBook | null>(null);
export const activeTradesAtom = atom<Trade[]>([]);
export const portfolioValueAtom = atom<number>(100000.00);
export const cashBalanceAtom = atom<number>(100000.00);
export const openOrdersAtom = atom<OpenOrder[]>([]);
export const tradesHistoryAtom = atom<TradeRecord[]>([]);
export const priceHistoryAtom = atom<Record<string, Candlestick[]>>({});

// --- TANTRA STATE ---
export interface CalendarEvent {
  id: string;
  title: string;
  start: string;
  end: string;
  isDnd: boolean;
  status: 'PENDING' | 'ACTIVE' | 'PASSED';
}

export interface CoworkerTask {
  id: string;
  title: string;
  priority: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW';
  status: 'PENDING' | 'RESOLVED';
  description: string;
  category: 'Risk Review' | 'Calendar Block' | 'Trade Verify' | 'System Health';
}

export interface NewsHeadline {
  id: string;
  headline: string;
  source: string;
  impact: 'HIGH' | 'MEDIUM' | 'LOW';
  timestamp: number;
  symbolRelated?: string;
}

export interface PortfolioHealth {
  marginRatio: number;
  riskIndex: number;
  dailyYield: number;
  valueAtRisk: number;
  systemSafetyStatus: 'SECURE' | 'WARNING' | 'CRITICAL';
}

export const systemAlertsAtom = atom<TantraAlert[]>([
  {
    alertId: 'A-1',
    source: 'StateCache',
    severity: 'Info',
    message: 'Active state cache synchronized via SQLite.',
    timestamp: Date.now(),
  }
]);
export const serverLogsAtom = atom<string[]>([
  '[INFO] Actix Server listening on 127.0.0.1:8080',
  '[INFO] StateCache initialized with SQLite vault.',
]);
export const metricsAtom = atom<{ cpu: number; memory: number; tps: number }>({
  cpu: 12.5,
  memory: 45.2,
  tps: 0,
});

export const calendarEventsAtom = atom<CalendarEvent[]>([
  {
    id: 'cal-1',
    title: 'Operator Weekly Risk & Alignment Meeting',
    start: '09:00',
    end: '10:00',
    isDnd: true,
    status: 'ACTIVE',
  },
  {
    id: 'cal-2',
    title: 'Binance API Maintenance Upgrade Window',
    start: '14:00',
    end: '15:30',
    isDnd: false,
    status: 'PENDING',
  }
]);

export const coworkerTasksAtom = atom<CoworkerTask[]>([
  {
    id: 'task-1',
    title: 'Approve Exposure Limit Increase (SOL-USD)',
    priority: 'HIGH',
    status: 'PENDING',
    description: 'Autonomous agent requests allocation increase from 10% to 15% collateral.',
    category: 'Risk Review',
  },
  {
    id: 'task-2',
    title: 'Inspect In-flight Collateral Overlap',
    priority: 'MEDIUM',
    status: 'PENDING',
    description: 'Bids on BTC and ETH exceed standard concurrent threshold warning.',
    category: 'Trade Verify',
  }
]);

export const newsFeedAtom = atom<NewsHeadline[]>([
  {
    id: 'news-1',
    headline: 'US Fed Announces Interest Rate Cut of 25bps',
    source: 'Bloomberg',
    impact: 'HIGH',
    timestamp: Date.now() - 600000,
    symbolRelated: 'BTC-USD',
  },
  {
    id: 'news-2',
    headline: 'KuCoin Token KCS Integrates New Bridge Network',
    source: 'CoinDesk',
    impact: 'MEDIUM',
    timestamp: Date.now() - 1200000,
    symbolRelated: 'SOL-USD',
  }
]);

export const portfolioHealthAtom = atom<PortfolioHealth>({
  marginRatio: 12.4,
  riskIndex: 0.38,
  dailyYield: 2.45,
  valueAtRisk: 3450.00,
  systemSafetyStatus: 'SECURE',
});

// --- SKILLS STATE ---
export interface SkillSignal {
  skill_id: string;
  skill_name: string;
  direction: 'Bullish' | 'Bearish' | 'Neutral';
  strength: number;
  confidence: number;
  details: string;
  indicators: Record<string, number>;
  time_frame: string;
}

export interface AggregatedAnalysis {
  symbol: string;
  current_price: number;
  signals: SkillSignal[];
  overall_conviction: number;
  overall_direction: 'Bullish' | 'Bearish' | 'Neutral';
  bullish_signals: number;
  bearish_signals: number;
  neutral_signals: number;
  timestamp: string;
}

export const skillAnalysisAtom = atom<AggregatedAnalysis | null>(null);
export const availableSkillsAtom = atom<string[]>([]);

// --- AUTO-TRADING STATE ---
export interface AutoTradingState {
  enabled: boolean;
  paper_trading: boolean;
  symbols: string[];
  analysis_interval_secs: number;
  last_analysis: string | null;
  next_analysis: string | null;
  last_outcomes: DecisionOutcome[];
  open_positions: string[];
  current_drawdown_pct: number;
  balance: number;
  performance: PerformanceStats | null;
}

export interface DecisionOutcome {
  symbol: string;
  timestamp: string;
  regime: string;
  action: TradeActionInfo;
  conviction: number;
  bullish_signals: number;
  bearish_signals: number;
  neutral_signals: number;
  summary: string;
}

export interface TradeActionInfo {
  Buy?: [string, number, number];
  Sell?: [string, number, number];
  Hold?: [string];
  Skip?: [string, string];
}

export interface PerformanceStats {
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate: number;
  total_pnl: number;
  avg_win: number;
  avg_loss: number;
  profit_factor: number;
  max_drawdown: number;
  sharpe_ratio: number;
}

export const autoTradingStateAtom = atom<AutoTradingState | null>(null);
export const performanceStatsAtom = atom<PerformanceStats | null>(null);

// --- SETTINGS STATE ---
export interface SettingsModel {
  id: string;
  name: string;
  provider: 'ollama' | 'gemini' | 'openai' | 'anthropic' | 'custom';
  endpoint: string;
  model_name: string;
  api_key_ref: string;
  active: boolean;
  max_tokens: number;
  temperature: number;
}

export interface SettingsApiKey {
  id: string;
  service: string;
  key: string;
  active: boolean;
}

export interface SettingsAgent {
  id: string;
  name: string;
  role: string;
  model_id: string;
  system_prompt: string;
  temperature: number;
  max_tokens: number;
  active: boolean;
  tools: string[];
}

export interface SettingsSkill {
  id: string;
  name: string;
  enabled: boolean;
  weight: number;
  min_confidence: number;
  params: Record<string, number>;
}

export interface SettingsPrompt {
  id: string;
  name: string;
  template: string;
  variables: string[];
  category: string;
}

export interface SettingsTool {
  id: string;
  name: string;
  description: string;
  endpoint: string;
  active: boolean;
  params: string[];
}

// Helper to load from localStorage
function loadSettings<T>(key: string, defaults: T): T {
  try {
    const stored = localStorage.getItem(`arkm_settings_${key}`);
    if (stored) return JSON.parse(stored);
  } catch {}
  return defaults;
}

export const settingsModelsAtom = atom<SettingsModel[]>(
  loadSettings<SettingsModel[]>('models', [
    {
      id: 'model-default',
      name: 'Default Ollama',
      provider: 'ollama',
      endpoint: 'http://localhost:11434',
      model_name: 'llama3.2',
      api_key_ref: '',
      active: true,
      max_tokens: 4096,
      temperature: 0.7,
    },
    {
      id: 'model-hermes',
      name: 'Hermes Gemini',
      provider: 'gemini',
      endpoint: 'https://generativelanguage.googleapis.com/v1beta/models',
      model_name: 'gemini-2.0-flash',
      api_key_ref: '',
      active: true,
      max_tokens: 8192,
      temperature: 0.3,
    },
  ])
);

export const settingsApiKeysAtom = atom<SettingsApiKey[]>(
  loadSettings<SettingsApiKey[]>('api_keys', [
    { id: 'key-gemini', service: 'gemini', key: '', active: false },
    { id: 'key-openai', service: 'openai', key: '', active: false },
    { id: 'key-binance', service: 'binance', key: '', active: false },
    { id: 'key-kucoin', service: 'kucoin', key: '', active: false },
  ])
);

export const settingsAgentsAtom = atom<SettingsAgent[]>(
  loadSettings<SettingsAgent[]>('agents', [
    {
      id: 'agent-hermes',
      name: 'Hermes Tredo',
      role: 'analyst',
      model_id: 'model-hermes',
      system_prompt: 'You are Hermes, an expert trading analyst. Analyze market data and provide clear trading signals with conviction scores.',
      temperature: 0.3,
      max_tokens: 2048,
      active: true,
      tools: ['skills', 'market-data', 'journal'],
    },
    {
      id: 'agent-risk',
      name: 'Risk Manager',
      role: 'risk-manager',
      model_id: 'model-default',
      system_prompt: 'You are a risk management specialist. Evaluate trade proposals and ensure they comply with risk parameters.',
      temperature: 0.2,
      max_tokens: 2048,
      active: true,
      tools: ['risk-engine', 'position-sizing'],
    },
  ])
);

export const settingsSkillsAtom = atom<SettingsSkill[]>(
  loadSettings<SettingsSkill[]>('skills', [])
);

export const settingsPromptsAtom = atom<SettingsPrompt[]>(
  loadSettings<SettingsPrompt[]>('prompts', [
    {
      id: 'prompt-analysis',
      name: 'Market Analysis',
      template: 'Analyze the following market data for {{symbol}}:\nCurrent Price: ${{price}}\nSignals: {{signals}}\nRegime: {{regime}}\n\nProvide a trading recommendation with conviction level (0-100%).',
      variables: ['symbol', 'price', 'signals', 'regime'],
      category: 'analysis',
    },
    {
      id: 'prompt-risk',
      name: 'Risk Assessment',
      template: 'Evaluate the risk of this trade:\nSymbol: {{symbol}}\nSide: {{side}}\nSize: {{size}}\nPortfolio: {{portfolio}}\n\nIs this trade within acceptable risk parameters?',
      variables: ['symbol', 'side', 'size', 'portfolio'],
      category: 'risk',
    },
  ])
);

export const settingsToolsAtom = atom<SettingsTool[]>(
  loadSettings<SettingsTool[]>('tools', [
    {
      id: 'tool-market-data',
      name: 'Market Data Fetcher',
      description: 'Fetch real-time market data and candles from Yahoo Finance',
      endpoint: '/api/market/candles',
      active: true,
      params: ['symbol', 'timeframe'],
    },
    {
      id: 'tool-skills',
      name: 'Hermes Skills Engine',
      description: 'Run the full Hermes skills analysis on a symbol',
      endpoint: '/api/skills/analyze',
      active: true,
      params: ['symbol', 'current_price', 'candles'],
    },
    {
      id: 'tool-journal',
      name: 'Trade Journal',
      description: 'Record and query trade history with performance stats',
      endpoint: '/api/journal',
      active: true,
      params: ['action', 'data'],
    },
    {
      id: 'tool-backtest',
      name: 'Backtesting Engine',
      description: 'Run historical backtests on trading strategies',
      endpoint: '/api/backtest/run',
      active: true,
      params: ['symbol', 'strategy', 'start_date', 'end_date'],
    },
  ])
);
