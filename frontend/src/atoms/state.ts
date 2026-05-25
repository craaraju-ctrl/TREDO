import { atom } from 'jotai';
import { OrderBook, Trade, TantraAlert } from '../../../protocols/ts';

export type { TantraAlert };

// Navigation active module tab
export type ActiveModule = 'Chat' | 'Tredo' | 'Tantra' | 'Journal' | 'Settings';
export const activeModuleAtom = atom<ActiveModule>(loadSettings<ActiveModule>('active_module', 'Chat'));

// --- CHAT STATE ---
export interface ChatMessage {
  sender: 'Operator' | 'Nethra' | 'System';
  text: string;
  timestamp: number;
}
export const chatMessagesAtom = atom<ChatMessage[]>([
  {
    sender: 'Nethra',
    text: 'Greetings, Operator. Sethu bridge is online. Chat, Tredo, and Tantra modules are operational.',
    timestamp: Date.now(),
  }
]);
export const chatInputAtom = atom<string>('');
export const selectedModelAtom = atom<string>(loadSettings<string>('selected_model', 'nemotron-3-nano:4b'));
export const selectedAgentAtom = atom<string>(loadSettings<string>('selected_agent', 'Nethra Swarm'));
export const selectedModuleAtom = atom<string>(loadSettings<string>('selected_module', 'all'));

// --- CHAT SESSION STATE ---
export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  agent: string;
  model: string;
  timestamp: number;
}

const DEFAULT_SESSIONS: ChatSession[] = [
  {
    id: 'session-1',
    title: 'Sethu System Health & Intel',
    messages: [
      {
        sender: 'Nethra',
        text: 'Greetings, Operator. Nethra Swarm is online. Chat, Tredo, and Tantra modules are operational.',
        timestamp: Date.now() - 3600000,
      },
    ],
    agent: 'Nethra Swarm',
    model: 'nemotron-3-nano:4b',
    timestamp: Date.now() - 3600000,
  },
  {
    id: 'session-2',
    title: 'BTC Conviction Analysis',
    messages: [
      { sender: 'Operator', text: 'Run skills analysis on BTC-USD.', timestamp: Date.now() - 1800000 },
      {
        sender: 'Nethra',
        text: '🟢 **Nethra Skills Analysis** for BTC-USD\nConviction: 41% (Bullish)\nSignals: 20 Bullish | 6 Bearish | 3 Neutral\nSkills Fired: 29/32 skills triggered',
        timestamp: Date.now() - 1795000,
      },
    ],
    agent: 'Nethra Swarm',
    model: 'nemotron-3-nano:4b',
    timestamp: Date.now() - 1800000,
  },
  {
    id: 'session-3',
    title: 'Tantra Risk Policy Review',
    messages: [
      { sender: 'Operator', text: 'What is the current safety coordination index?', timestamp: Date.now() - 600000 },
      {
        sender: 'Nethra',
        text: 'Safety coordinator index is set to HIGH_GUARD due to active calendar DND schedule.',
        timestamp: Date.now() - 590000,
      },
    ],
    agent: 'Risk Manager',
    model: 'nemotron-3-nano:4b',
    timestamp: Date.now() - 600000,
  },
];

export const chatSessionsAtom = atom<ChatSession[]>(DEFAULT_SESSIONS);
export const activeSessionIdAtom = atom<string>('session-1');
export const skillAnalyzingAtom = atom<boolean>(false);
export const isTypingAtom = atom<boolean>(false);

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

// Auto-migrate standard watchlist if old smaller list (length <= 4) is stored
if (typeof window !== 'undefined') {
  try {
    const stored = localStorage.getItem('tredo_settings_watchlist');
    if (stored) {
      const list = JSON.parse(stored);
      if (Array.isArray(list) && list.length <= 4) {
        localStorage.removeItem('tredo_settings_watchlist');
        localStorage.removeItem('tredo_settings_base_prices');
      }
    }
  } catch {}
}

const DEFAULT_WATCHLIST = [
  // Crypto
  'BTC-USD', 'ETH-USD', 'SOL-USD', 'ADA-USD', 'XRP-USD', 
  'DOT-USD', 'DOGE-USD', 'LTC-USD', 'LINK-USD', 'AVAX-USD', 
  'TRX-USD', 'SHIB-USD', 'TON-USD', 'SUI-USD', 'NEAR-USD',
  // US Stocks
  'AAPL', 'TSLA', 'MSFT', 'NVDA', 'AMZN', 
  'GOOG', 'META', 'AMD', 'NFLX', 'MS', 
  'JPM', 'V', 'DIS', 'WMT', 'COST',
  // Indian Stocks (NSE)
  'NSE:RELIANCE', 'NSE:TCS', 'NSE:HDFCBANK', 'NSE:INFY', 'NSE:ICICIBANK', 
  'NSE:SBIN', 'NSE:BHARTIALRT', 'NSE:ITC', 'NSE:LTIM', 'NSE:LT', 
  'NSE:HINDUNILVR', 'NSE:SUNPHARMA', 'NSE:KOTAKBANK', 'NSE:AXISBANK', 'NSE:TATASTEEL',
  // Commodities & Others
  'XAU-USD', 'XAG-USD', 'USOIL', 'NGAS'
];

const DEFAULT_BASE_PRICES = {
  // Crypto
  'BTC-USD': 77430.0,
  'ETH-USD': 3450.0,
  'SOL-USD': 142.5,
  'ADA-USD': 0.58,
  'XRP-USD': 1.15,
  'DOT-USD': 6.20,
  'DOGE-USD': 0.16,
  'LTC-USD': 84.50,
  'LINK-USD': 15.20,
  'AVAX-USD': 28.40,
  'TRX-USD': 0.12,
  'SHIB-USD': 0.000018,
  'TON-USD': 5.50,
  'SUI-USD': 1.85,
  'NEAR-USD': 5.20,
  // US Stocks
  'AAPL': 185.20,
  'TSLA': 178.50,
  'MSFT': 415.60,
  'NVDA': 910.30,
  'AMZN': 182.40,
  'GOOG': 172.80,
  'META': 485.40,
  'AMD': 164.20,
  'NFLX': 610.50,
  'MS': 92.30,
  'JPM': 195.40,
  'V': 272.50,
  'DIS': 112.40,
  'WMT': 60.20,
  'COST': 725.60,
  // Indian Stocks
  'NSE:RELIANCE': 2450.0,
  'NSE:TCS': 3850.0,
  'NSE:HDFCBANK': 1520.0,
  'NSE:INFY': 1430.0,
  'NSE:ICICIBANK': 1120.0,
  'NSE:SBIN': 740.0,
  'NSE:BHARTIALRT': 1210.0,
  'NSE:ITC': 430.0,
  'NSE:LTIM': 4850.0,
  'NSE:LT': 3520.0,
  'NSE:HINDUNILVR': 2240.0,
  'NSE:SUNPHARMA': 1540.0,
  'NSE:KOTAKBANK': 1720.0,
  'NSE:AXISBANK': 1060.0,
  'NSE:TATASTEEL': 145.0,
  // Commodities
  'XAU-USD': 2352.0,
  'XAG-USD': 28.40,
  'USOIL': 78.50,
  'NGAS': 2.45
};

export const watchlistAtom = atom<string[]>(loadSettings<string[]>('watchlist', DEFAULT_WATCHLIST));
export const basePricesAtom = atom<Record<string, number>>(
  loadSettings<Record<string, number>>('base_prices', DEFAULT_BASE_PRICES)
);
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
export const serverActiveAtom = atom<boolean>(true);

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
  category: 'cloud' | 'local';
}

export interface SettingsApiKey {
  id: string;
  service: string;
  key: string;
  active: boolean;
  category: 'exchange' | 'news' | 'ai';
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
    const stored = localStorage.getItem(`tredo_settings_${key}`);
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
      model_name: 'nemotron-3-nano:4b',
      api_key_ref: '',
      active: true,
      max_tokens: 4096,
      temperature: 0.7,
      category: 'local',
    },
    {
      id: 'model-nethra',
      name: 'Nethra Gemini',
      provider: 'gemini',
      endpoint: 'https://generativelanguage.googleapis.com/v1beta/models',
      model_name: 'gemini-2.0-flash',
      api_key_ref: '',
      active: true,
      max_tokens: 8192,
      temperature: 0.3,
      category: 'cloud',
    },
  ])
);

export const settingsApiKeysAtom = atom<SettingsApiKey[]>(
  loadSettings<SettingsApiKey[]>('api_keys', [
    { id: 'key-gemini', service: 'gemini', key: '', active: false, category: 'ai' },
    { id: 'key-openai', service: 'openai', key: '', active: false, category: 'ai' },
    { id: 'key-binance', service: 'binance', key: '', active: false, category: 'exchange' },
    { id: 'key-kucoin', service: 'kucoin', key: '', active: false, category: 'exchange' },
    { id: 'key-news-finnhub', service: 'finnhub.io', key: '', active: false, category: 'news' },
    { id: 'key-news-cryptopanic', service: 'cryptopanic', key: '', active: false, category: 'news' },
    { id: 'key-news-polygon', service: 'polygon.io', key: '', active: false, category: 'news' },
    { id: 'key-news-rss2json', service: 'rss2json', key: '', active: false, category: 'news' },
    { id: 'key-news-gnews', service: 'gnews', key: '', active: false, category: 'news' },
    { id: 'key-news-thenewsapi', service: 'thenewsapi', key: '', active: false, category: 'news' },
    { id: 'key-news-mediastack', service: 'mediastack', key: '', active: false, category: 'news' },
    { id: 'key-news-currents', service: 'currents', key: '', active: false, category: 'news' },
    { id: 'key-news-bloomberg', service: 'bloomberg', key: '', active: false, category: 'news' },
    { id: 'key-news-coindesk', service: 'coindesk', key: '', active: false, category: 'news' },
  ])
);

export const settingsAgentsAtom = atom<SettingsAgent[]>(
  loadSettings<SettingsAgent[]>('agents', [
    {
      id: 'agent-nethra',
      name: 'Nethra Swarm',
      role: 'analyst',
      model_id: 'model-nethra',
      system_prompt: 'You are Nethra, an expert autonomous multi-agent swarm. Analyze market data and provide clear trading signals with conviction scores.',
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
      name: 'Nethra Skills Engine',
      description: 'Run the full Nethra skills analysis on a symbol',
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
