/** Module tab identifiers */
export type ModuleTab = 'Chat' | 'Tredo' | 'Tantra' | 'Journal' | 'Settings';

/** Module tab configuration */
export const MODULE_TABS: { id: ModuleTab; label: string; icon: string; description: string }[] = [
  { id: 'Chat', label: 'Chat', icon: '💬', description: 'Multi-agent communication hub' },
  { id: 'Tredo', label: 'Tredo', icon: '📊', description: 'Exchange trading dashboard' },
  { id: 'Tantra', label: 'Nethra', icon: '🛡️', description: 'Nethra swarm & coworker orchestration' },
  { id: 'Journal', label: 'News', icon: '📰', description: 'Real-time global news feed & sentiment analysis' },
  { id: 'Settings', label: 'Settings', icon: '⚙️', description: 'System configuration' },
] as const;

/** Available AI models */
export const AVAILABLE_MODELS = [
  { value: 'nemotron-3-nano:4b', label: 'nemotron-3-nano:4b (Local)' },
  { value: 'gemini-2.0-flash', label: 'gemini-2.0-flash (Cloud)' },
] as const;

export const AVAILABLE_AGENTS = [
  { value: 'Nethra Swarm', label: 'Nethra Swarm (Rust)', description: 'Autonomous Hierarchical Swarm Coordinator' },
  { value: 'Risk Manager', label: 'Risk Manager', description: 'Safety Check' },
  { value: 'Tantra Monitor', label: 'Tantra Monitor', description: 'Systems Specialist' },
] as const;

/** Base prices for mock data */
export const BASE_PRICES: Record<string, number> = {
  'BTC-USD': 77430.0,
  'ETH-USD': 3450.0,
  'SOL-USD': 142.5,
  'XAU-USD': 2352.0,
};

/** Default watchlist */
export const DEFAULT_WATCHLIST = ['BTC-USD', 'ETH-USD', 'SOL-USD', 'XAU-USD'];

/** Keyboard shortcuts configuration */
export const KEYBOARD_SHORTCUTS = {
  '1': { module: 'Chat' as ModuleTab, description: 'Switch to Chat' },
  '2': { module: 'Tredo' as ModuleTab, description: 'Switch to Tredo' },
  '3': { module: 'Tantra' as ModuleTab, description: 'Switch to Nethra' },
  '4': { module: 'Journal' as ModuleTab, description: 'Switch to News' },
  '5': { module: 'Settings' as ModuleTab, description: 'Switch to Settings' },
  'n': { action: 'newChat', description: 'New chat session' },
  'Escape': { action: 'closePanel', description: 'Close active panel' },
} as const;

/** Timeframes for charts */
export const TIMEFRAMES = ['1m', '5m', '15m', '1h', '1d'] as const;

/** Order types */
export const ORDER_TYPES = ['LIMIT', 'MARKET'] as const;

/** Order sides */
export const ORDER_SIDES = ['BUY', 'SELL'] as const;

/** Percentage presets for order amount */
export const AMOUNT_PRESETS = [0.25, 0.5, 0.75, 1.0] as const;

/** Bottom ledger tabs */
export const LEDGER_TABS = ['OPEN', 'HISTORY', 'ASSETS', 'SWARM'] as const;
