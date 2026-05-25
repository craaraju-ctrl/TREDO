import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act, cleanup } from '@testing-library/react/pure';
import { getDefaultStore } from 'jotai';
import {
  cashBalanceAtom,
  openOrdersAtom,
  tradesHistoryAtom,
  systemAlertsAtom,
  serverLogsAtom,
  portfolioValueAtom,
  priceHistoryAtom,
  selectedAssetAtom,
  watchlistAtom,
  basePricesAtom,
  autoTradingStateAtom,
  performanceStatsAtom,
} from '../atoms/state';
import { TredoModule } from '../components/tredo/TredoModule';

const jotaiStore = getDefaultStore();

// ── Mocks ─────────────────────────────────────────────────────────────────

// Mock fetch for API calls.
// Important: We use mockImplementation with a deferred pattern (setTimeout 10ms)
// instead of mockResolvedValue. This ensures fetch promises resolve inside the
// fake timer system, so the initial fetchState() call in the 10s interval effect
// completes its async state updates inside act() when timers are advanced.
// Using mockResolvedValue would cause promises to resolve as microtasks outside
// act(), triggering "not wrapped in act(...)" warnings.
const mockFetch = vi.fn();
mockFetch.mockImplementation(() =>
  new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        ok: true,
        json: () => Promise.resolve({ status: 'success', trading_state: null, stats: null }),
      });
    }, 10);
  })
);
global.fetch = mockFetch;

// Mock canvas context methods used by the chart renderer
const mockCtx = new Proxy({} as any, {
  get(_target, prop) {
    if (prop === 'measureText') {
      return () => ({ width: 10, height: 10 });
    }
    if (prop === 'createLinearGradient') {
      return () => ({
        addColorStop: () => {},
      });
    }
    if (prop === 'canvas') {
      return document.createElement('canvas');
    }
    return vi.fn();
  }
});

beforeEach(() => {
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(
    (_contextId: string) => mockCtx as unknown as CanvasRenderingContext2D
  );
});


// ── Helpers ───────────────────────────────────────────────────────────────

/** Get a default mock auto-trading state for tests */
function mockAutoTradingState(overrides: Record<string, any> = {}) {
  return {
    enabled: false,
    paper_trading: true,
    symbols: ['BTC-USD'],
    analysis_interval_secs: 300,
    last_analysis: null,
    next_analysis: null,
    last_outcomes: [
      {
        symbol: 'BTC-USD',
        timestamp: '2025-05-21T12:00:00Z',
        regime: 'bullish_trend',
        action: { Buy: ['limit', 75000, 0.5] },
        conviction: 0.65,
        bullish_signals: 12,
        bearish_signals: 4,
        neutral_signals: 2,
        summary: 'Bullish momentum confirmed',
      },
    ],
    open_positions: ['BTC-USD'],
    current_drawdown_pct: 3.2,
    balance: 105000.50,
    performance: null,
    ...overrides,
  };
}

/** Get a default mock performance stats */
function mockPerfStats(overrides: Record<string, any> = {}) {
  return {
    total_trades: 47,
    winning_trades: 28,
    losing_trades: 19,
    win_rate: 59.6,
    total_pnl: 3200.00,
    avg_win: 215.00,
    avg_loss: -120.00,
    profit_factor: 1.85,
    max_drawdown: 8.4,
    sharpe_ratio: 1.2,
    ...overrides,
  };
}

/** Render TredoModule with clean state and flush pending React microtasks.
 * React 18's createRoot schedules some state updates as microtasks (via
 * useSyncExternalStore from Jotai). The async act(() => {}) pattern flushes
 * those microtasks so state updates complete inside the act() scope. */
async function renderModule() {
  const result = render(<TredoModule />);
  // Flush both synchronous and microtask-scheduled React state updates
  await act(async () => {});
  
  // Toggle to Local chart mode to support standard technical indicators and canvas DOM assertions used in tests
  const localBtn = screen.queryByText('Local');
  if (localBtn) {
    fireEvent.click(localBtn);
    await act(async () => {});
  }
  
  return result;
}

/** Mock fetch with custom JSON body using the deferred (10ms) promise pattern.
 * Ensures fetch resolves inside the fake timer system, avoiding act() warnings. */
function mockFetchWith(data: Record<string, any>) {
  mockFetch.mockImplementation(() =>
    new Promise((resolve) => {
      setTimeout(() => {
        resolve({ ok: true, json: () => Promise.resolve(data) });
      }, 10);
    })
  );
}

/** Advance timers past the 10s fetch interval and flush pending microtasks.
 * Advances extra to trigger the deferred (10ms) fetch promises and price simulator.
 * Note: vi.advanceTimersByTimeAsync internally wraps timer callbacks in act(),
 * so we must NOT wrap it in another act() call to avoid nesting issues. */
async function advanceToFetch() {
  await vi.advanceTimersByTimeAsync(10001);
  await vi.advanceTimersByTimeAsync(2000);
  // Flush any microtask-scheduled React state updates
  await vi.advanceTimersByTimeAsync(50);
}

// ── Tests ─────────────────────────────────────────────────────────────────

describe('TredoModule', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    // Don't mock Math.random — the deterministic mock causes ID collisions in
    // timer-triggered state updates that call Math.random frequently, wrapping
    // the mock sequence and producing duplicate IDs.
    mockFetch.mockClear();
    // Default deferred implementation (overridden per-test via mockFetchWith)
    localStorage.clear();
  });

  afterEach(async () => {
    // Unmount the component inside act() to suppress warnings from React 18
    // microtask-based state updates during unmount. The /pure import disables
    // @testing-library/react's auto-cleanup so we manage it here manually.
    await act(async () => {
      cleanup();
    });

    // Flush any remaining React microtasks
    await act(async () => {});

    // Now safe to switch timer mode (component is unmounted)
    vi.useRealTimers();
    vi.restoreAllMocks();

    // Reset all Jotai atoms to their initial values
    jotaiStore.set(cashBalanceAtom, 100000);
    jotaiStore.set(openOrdersAtom, []);
    jotaiStore.set(tradesHistoryAtom, []);
    jotaiStore.set(systemAlertsAtom, []);
    jotaiStore.set(serverLogsAtom, []);
    jotaiStore.set(portfolioValueAtom, 100000);
    jotaiStore.set(priceHistoryAtom, {});
    jotaiStore.set(selectedAssetAtom, 'BTC-USD');
    jotaiStore.set(watchlistAtom, ['BTC-USD', 'ETH-USD', 'SOL-USD', 'XAU-USD']);
    jotaiStore.set(basePricesAtom, {
      'BTC-USD': 77430.0,
      'ETH-USD': 3450.0,
      'SOL-USD': 142.5,
      'XAU-USD': 2352.0,
    });
    jotaiStore.set(autoTradingStateAtom, null);
    jotaiStore.set(performanceStatsAtom, null);

    localStorage.clear();
  });

  // ── Basic Render ────────────────────────────────────────────────────────

  it('renders the trading module with all major sections', async () => {
    await renderModule();
    expect(screen.getByText('WATCHLIST')).toBeInTheDocument();
    expect(screen.getByText('RECENT MARKET TRADES')).toBeInTheDocument();
    expect(screen.getByText('ORDER BOOK')).toBeInTheDocument();
    expect(screen.getByText('AUTONOMOUS TRADING')).toBeInTheDocument();
  });

  it('renders the watchlist with default symbols', async () => {
    await renderModule();
    // BTC-USD appears in watchlist AND chart header — use getAllByText
    expect(screen.getAllByText('BTC-USD').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('ETH-USD')).toBeInTheDocument();
    expect(screen.getByText('SOL-USD')).toBeInTheDocument();
    expect(screen.getByText('XAU-USD')).toBeInTheDocument();
  });

  it('shows Live badge on the chart panel', async () => {
    await renderModule();
    expect(screen.getByText('Live')).toBeInTheDocument();
  });

  // ── Indicator Period Settings ───────────────────────────────────────────

  describe('Indicator Period Settings', () => {
    it('displays indicators with correct period format', async () => {
      await renderModule();
      expect(screen.getByText('SMA (9)')).toBeInTheDocument();
      expect(screen.getByText('EMA (12)')).toBeInTheDocument();
      expect(screen.getByText('BB (20,2)')).toBeInTheDocument();
      expect(screen.getByText('VWAP')).toBeInTheDocument();
    });

    it('shows gear icon for SMA, EMA, BB indicators but not VWAP', async () => {
      await renderModule();
      expect(screen.getByTitle('Configure SMA period')).toBeInTheDocument();
      expect(screen.getByTitle('Configure EMA period')).toBeInTheDocument();
      expect(screen.getByTitle('Configure BB period')).toBeInTheDocument();
      expect(screen.queryByTitle('Configure VWAP period')).not.toBeInTheDocument();
    });

    it('opens period configuration popup when SMA gear icon is clicked', async () => {
      await renderModule();
      // Click the SMA gear icon
      fireEvent.click(screen.getByTitle('Configure SMA period'));

      // The popup should be visible with a period input (no waitFor needed - sync)
      expect(screen.getByText('Period:')).toBeInTheDocument();
      expect(screen.getByText('Apply')).toBeInTheDocument();
      expect(screen.getByDisplayValue('9')).toBeInTheDocument();
    });

    it('updates SMA period when changed in the popup', async () => {
      await renderModule();
      fireEvent.click(screen.getByTitle('Configure SMA period'));
      const periodInput = screen.getByDisplayValue('9');
      fireEvent.change(periodInput, { target: { value: '14' } });
      fireEvent.click(screen.getByText('Apply'));

      expect(screen.queryByText('Period:')).not.toBeInTheDocument();
      expect(screen.getByText('SMA (14)')).toBeInTheDocument();
    });

    it('closes popup and shows new display text for EMA period change', async () => {
      await renderModule();
      fireEvent.click(screen.getByTitle('Configure EMA period'));
      const periodInput = screen.getByDisplayValue('12');
      fireEvent.change(periodInput, { target: { value: '26' } });
      fireEvent.click(screen.getByText('Apply'));

      expect(screen.getByText('EMA (26)')).toBeInTheDocument();
    });

    it('toggles indicator active state on click', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('SMA (9)'));
      expect(screen.getByText('SMA (9)')).toBeInTheDocument();
    });
  });

  // ── Order Placement ─────────────────────────────────────────────────────

  describe('Order Placement', () => {
    it('shows order type toggle buttons', async () => {
      await renderModule();
      expect(screen.getByText('LIMIT')).toBeInTheDocument();
      expect(screen.getByText('MARKET')).toBeInTheDocument();
    });

    it('shows order side toggle buttons', async () => {
      await renderModule();
      expect(screen.getByText('BUY')).toBeInTheDocument();
      expect(screen.getByText('SELL')).toBeInTheDocument();
    });

    it('switches order type from LIMIT to MARKET', async () => {
      await renderModule();
      expect(screen.getByText('LIMIT PRICE (USD)')).toBeInTheDocument();

      fireEvent.click(screen.getByText('MARKET'));

      expect(screen.queryByText('LIMIT PRICE (USD)')).not.toBeInTheDocument();
    });

    it('switches order side between BUY and SELL', async () => {
      await renderModule();
      expect(screen.getByText(/PLACE BUY/)).toBeInTheDocument();

      fireEvent.click(screen.getByText('SELL'));
      expect(screen.getByText(/PLACE SELL/)).toBeInTheDocument();

      fireEvent.click(screen.getByText('BUY'));
      expect(screen.getByText(/PLACE BUY/)).toBeInTheDocument();
    });

    it('displays the available balance', async () => {
      await renderModule();
      expect(screen.getByText('AVAILABLE BALANCE:')).toBeInTheDocument();
      // Default cash is 100,000 — should have $100,000.00
      expect(screen.getByText('$100,000.00')).toBeInTheDocument();
    });

    it('shows quantity input field', async () => {
      await renderModule();
      expect(screen.getByText('QUANTITY')).toBeInTheDocument();
    });

    it('shows amount preset percentage buttons', async () => {
      await renderModule();
      expect(screen.getByText('25%')).toBeInTheDocument();
      expect(screen.getByText('50%')).toBeInTheDocument();
      expect(screen.getByText('75%')).toBeInTheDocument();
      expect(screen.getByText('100%')).toBeInTheDocument();
    });

    it('calculates amount when 25% preset is clicked', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('25%'));
      const amountInput = screen.getByDisplayValue(/0\.3/);
      expect(amountInput).toBeInTheDocument();
    });

    it('calculates amount when 50% preset is clicked', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('50%'));
      const amountInput = screen.getByDisplayValue(/0\.6/);
      expect(amountInput).toBeInTheDocument();
    });

    it('shows estimated total in the order form', async () => {
      await renderModule();
      // EST. TOTAL is shown with default values
      expect(screen.getByText('EST. TOTAL:')).toBeInTheDocument();
    });

    it('shows limit price input with +/- buttons for LIMIT orders', async () => {
      await renderModule();
      expect(screen.getByText('LIMIT PRICE (USD)')).toBeInTheDocument();

      // The limit price input should show the current price
      const limitInput = screen.getByDisplayValue(/77430/);
      expect(limitInput).toBeInTheDocument();

      // The decrement and increment buttons
      const decBtn = screen.getByText('−');
      const incBtn = screen.getByText('+');
      expect(decBtn).toBeInTheDocument();
      expect(incBtn).toBeInTheDocument();
    });

    it('adjusts limit price down with the minus button', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('−'));
      const limitInput = screen.getByDisplayValue(/77352/);
      expect(limitInput).toBeInTheDocument();
    });

    it('adjusts limit price up with the plus button', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('+'));
      const limitInput = screen.getByDisplayValue(/77507/);
      expect(limitInput).toBeInTheDocument();
    });

    it('shows the PLACE ORDER button with correct side and type', async () => {
      await renderModule();
      // Default: BUY LIMIT
      const orderBtn = screen.getByText('PLACE BUY LIMIT ORDER');
      expect(orderBtn).toBeInTheDocument();
    });

    it('places a market order and shows confirmation in logs', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('MARKET'));
      fireEvent.click(screen.getByText('SELL'));
      fireEvent.click(screen.getByText('PLACE SELL MARKET ORDER'));
      fireEvent.click(screen.getByText('SYSTEM ALERTS'));
      expect(screen.getByText(/Executed Market SELL order/)).toBeInTheDocument();
    });

    it('places a limit order and shows it in open orders', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));

      // BTC-USD appears in watchlist AND chart AND open orders — use getAllByText
      expect(screen.getAllByText(/BTC-USD/).length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText('CANCEL')).toBeInTheDocument();
    });
  });

  // ── Auto-Trading Controls ───────────────────────────────────────────────

  describe('Auto-Trading Controls', () => {
    it('shows the autonomous trading section', async () => {
      await renderModule();
      expect(screen.getByText('AUTONOMOUS TRADING')).toBeInTheDocument();
    });

    it('shows START and STOP buttons with correct disabled states', async () => {
      await renderModule();
      const startBtn = screen.getByText('START');
      const stopBtn = screen.getByText('STOP');

      expect(startBtn).toBeInTheDocument();
      expect(stopBtn).toBeInTheDocument();

      // By default, trading is disabled, so START should be enabled and STOP disabled
      expect(startBtn).not.toBeDisabled();
      expect(stopBtn).toBeDisabled();
    });

    it('shows PAUSED status indicator by default', async () => {
      await renderModule();
      expect(screen.getByText('PAUSED')).toBeInTheDocument();
    });

    it('shows REAL mode badge by default when autoTradingState is null', async () => {
      await renderModule();
      // autoTradingState starts null, so autoTradingState?.paper_trading is
      // undefined (falsy) → the Badge shows 'REAL'
      expect(screen.getByText('REAL')).toBeInTheDocument();
    });

    it('displays balance, positions, drawdown, and interval when auto-trading data is loaded', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState(),
        stats: null,
      });
      await renderModule();
      await advanceToFetch();

      // balance: 105000.50 → toLocaleString() returns "105,000.5" with $ prefix
      expect(screen.getByText((c) => c.startsWith('$105'))).toBeInTheDocument();
      // positions length is 1
      expect(screen.getByText((c) => c === '1')).toBeInTheDocument();
      expect(screen.getByText('3.2%')).toBeInTheDocument();
      expect(screen.getByText('300s')).toBeInTheDocument();
    });

    it('switches to ACTIVE status and enables STOP when auto-trading is enabled', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState({ enabled: true }),
        stats: null,
      });
      await renderModule();
      await advanceToFetch();

      expect(screen.getByText('ACTIVE')).toBeInTheDocument();
      expect(screen.getByText('STOP')).not.toBeDisabled();
      expect(screen.getByText('START')).toBeDisabled();
    });

    it('shows RECENT DECISIONS with symbol, action, regime, and conviction', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState(),
        stats: null,
      });
      await renderModule();
      await advanceToFetch();

      expect(screen.getByText('RECENT DECISIONS')).toBeInTheDocument();
      expect(screen.getByText('bullish_trend')).toBeInTheDocument();
      // Multiple 'BUY' and 'BTC-USD' elements exist — use getAllByText
      expect(screen.getAllByText('BUY').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('BTC-USD').length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText('65%')).toBeInTheDocument();
    });

    it('displays performance stats when available', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState(),
        stats: mockPerfStats(),
      });
      await renderModule();
      await advanceToFetch();

      expect(screen.getByText('PERFORMANCE')).toBeInTheDocument();
      expect(screen.getByText('59.6%')).toBeInTheDocument();
      expect(screen.getByText('47')).toBeInTheDocument();
      expect(screen.getByText('$3200')).toBeInTheDocument();
    });

    it('calls the auto-trade start API when START is clicked', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('START'));

      // Flush pending state updates from the async handler chain
      await act(async () => {
        await Promise.resolve();
      });

      expect(mockFetch).toHaveBeenCalledWith('/api/autotrade/start', { method: 'POST' });
    });

    it('calls the auto-trade stop API when STOP is clicked', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState({ enabled: true }),
        stats: null,
      });
      await renderModule();
      await advanceToFetch();

      expect(screen.getByText('STOP')).not.toBeDisabled();

      fireEvent.click(screen.getByText('STOP'));

      // Flush pending state updates from the async handler chain
      await act(async () => {
        await Promise.resolve();
      });

      expect(mockFetch).toHaveBeenCalledWith('/api/autotrade/stop', { method: 'POST' });
    });

    it('periodically fetches auto-trading state', async () => {
      await renderModule();
      // Initial fetch
      await advanceToFetch();
      expect(mockFetch).toHaveBeenCalledWith('/api/autotrade/status');
      expect(mockFetch).toHaveBeenCalledWith('/api/journal/stats');

      // Advance another interval — use act-wrapped helper
      mockFetch.mockClear();
      await advanceToFetch();
      expect(mockFetch).toHaveBeenCalledWith('/api/autotrade/status');
    });

    it('shows REAL mode badge when paper_trading is false', async () => {
      mockFetchWith({
        status: 'success',
        trading_state: mockAutoTradingState({ paper_trading: false }),
        stats: null,
      });
      await renderModule();
      await advanceToFetch();

      expect(screen.getByText('REAL')).toBeInTheDocument();
    });

    it('handles fetch errors gracefully', async () => {
      mockFetch.mockImplementation(() =>
        new Promise((_, reject) => {
          setTimeout(() => reject(new Error('Network error')), 10);
        })
      );
      await renderModule();

      // Should not throw — component catches errors
      await advanceToFetch();
      expect(screen.getByText('AUTONOMOUS TRADING')).toBeInTheDocument();
    });
  });

  // ── Empty State ──────────────────────────────────────────────────────────

  describe('Empty State', () => {
    it('shows "No trades yet" when no trades for the selected asset', async () => {
      await renderModule();
      expect(screen.getByText('No trades yet')).toBeInTheDocument();
    });

    it('shows "No active open limit orders" when no orders', async () => {
      await renderModule();
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('shows default auto-trading fallback values when state is null', async () => {
      await renderModule();
      expect(screen.getByText('$100,000')).toBeInTheDocument();
      expect(screen.getByText('0.0%')).toBeInTheDocument();
      expect(screen.getByText('300s')).toBeInTheDocument();
    });

    it('shows PAUSED status and REAL mode badge when autoTradingState is null', async () => {
      await renderModule();
      expect(screen.getByText('PAUSED')).toBeInTheDocument();
      expect(screen.getByText('REAL')).toBeInTheDocument();
      expect(screen.getByText('START')).not.toBeDisabled();
      expect(screen.getByText('STOP')).toBeDisabled();
    });

    it('renders the bottom ledger ASSETS tab with default values', async () => {
      await renderModule();
      fireEvent.click(screen.getByText('LEDGER ASSETS'));
      expect(screen.getByText('NET WORTH')).toBeInTheDocument();
      expect(screen.getByText('LIQUID CASH')).toBeInTheDocument();
      expect(screen.getByText('ENGINE')).toBeInTheDocument();
    });
  });

  // ── Error Handling ────────────────────────────────────────────────────────

  describe('Error Handling', () => {
    beforeEach(() => {
      vi.stubGlobal('alert', vi.fn());
    });

    afterEach(() => {
      vi.unstubAllGlobals();
    });

    it('does not place order with invalid price (NaN)', async () => {
      await renderModule();
      const limitInput = screen.getByDisplayValue(/77430/);
      fireEvent.change(limitInput, { target: { value: 'abc' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      // No order placed — empty orders message remains
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('does not place order with invalid price (zero)', async () => {
      await renderModule();
      const limitInput = screen.getByDisplayValue(/77430/);
      fireEvent.change(limitInput, { target: { value: '0' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('does not place order with invalid price (negative)', async () => {
      await renderModule();
      const limitInput = screen.getByDisplayValue(/77430/);
      fireEvent.change(limitInput, { target: { value: '-50' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('does not place order with invalid amount (NaN)', async () => {
      await renderModule();
      const amountInput = screen.getByDisplayValue('0.1');
      fireEvent.change(amountInput, { target: { value: 'abc' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('does not place order with zero amount', async () => {
      await renderModule();
      const amountInput = screen.getByDisplayValue('0.1');
      fireEvent.change(amountInput, { target: { value: '0' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('does not place order with negative amount', async () => {
      await renderModule();
      const amountInput = screen.getByDisplayValue('0.1');
      fireEvent.change(amountInput, { target: { value: '-1' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(screen.getByText('No active open limit orders.')).toBeInTheDocument();
    });

    it('shows alert when placing BUY order exceeding cash balance', async () => {
      await renderModule();
      // Set amount large enough to exceed $100,000 cash
      // Price ~77430 * 2 = ~154,860 > 100,000
      const amountInput = screen.getByDisplayValue('0.1');
      fireEvent.change(amountInput, { target: { value: '2' } });
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      expect(alert).toHaveBeenCalledWith('Insufficient cash balance to place this order!');
    });

    it('handles adding a duplicate asset gracefully', async () => {
      await renderModule();
      fireEvent.click(screen.getByLabelText('Add asset to watchlist'));
      const symbolInput = screen.getByLabelText('Symbol ticker');
      const priceInput = screen.getByLabelText('Base price');
      fireEvent.change(symbolInput, { target: { value: 'BTC-USD' } });
      fireEvent.change(priceInput, { target: { value: '77000' } });
      fireEvent.click(screen.getByText('REGISTER'));
      // Form should close gracefully for duplicate
      expect(screen.queryByPlaceholderText('e.g. TSLA or DOGE-USD')).not.toBeInTheDocument();
    });

    it('handles adding asset with empty symbol and stays open', async () => {
      await renderModule();
      fireEvent.click(screen.getByLabelText('Add asset to watchlist'));
      // Click REGISTER without entering anything
      fireEvent.click(screen.getByText('REGISTER'));
      // Form should stay open due to invalid input
      expect(screen.getByPlaceholderText('e.g. TSLA or DOGE-USD')).toBeInTheDocument();
    });

    it('cancels adding asset and closes the form', async () => {
      await renderModule();
      fireEvent.click(screen.getByLabelText('Add asset to watchlist'));
      fireEvent.click(screen.getByText('CANCEL'));
      expect(screen.queryByPlaceholderText('e.g. TSLA or DOGE-USD')).not.toBeInTheDocument();
    });

    it('handles MARKET order with invalid amount gracefully', async () => {
      await renderModule();
      // Switch to MARKET
      fireEvent.click(screen.getByText('MARKET'));
      // Set invalid amount
      const amountInput = screen.getByDisplayValue('0.1');
      fireEvent.change(amountInput, { target: { value: 'abc' } });
      fireEvent.click(screen.getByText('PLACE BUY MARKET ORDER'));
      // Should remain on order form (no crash, no alert called)
      expect(screen.getByText('PLACE BUY MARKET ORDER')).toBeInTheDocument();
    });
  });

  // ── Large Dataset ──────────────────────────────────────────────────────────

  describe('Large Dataset', () => {
    it('renders correctly with many open orders', async () => {
      await renderModule();
      // Place 10 limit orders (each costs ~$7,743, well within $100,000)
      for (let i = 0; i < 10; i++) {
        fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      }
      // Should show 10 orders with CANCEL buttons
      const cancelButtons = screen.getAllByText('CANCEL');
      expect(cancelButtons.length).toBe(10);

      // Table should show the correct columns
      // Note: 'Price' and 'Amount' also appear in RECENT MARKET TRADES header — use getAllByText
      expect(screen.getByText('Symbol')).toBeInTheDocument();
      expect(screen.getByText('Side')).toBeInTheDocument();
      expect(screen.getAllByText('Price').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('Amount').length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText('Total')).toBeInTheDocument();
      expect(screen.getByText('Action')).toBeInTheDocument();
    });

    it('generates trades via price simulator and populates trade feed', async () => {
      await renderModule();
      // Should start empty
      expect(screen.getByText('No trades yet')).toBeInTheDocument();

      // Advance 5 seconds to generate ~5 trades from the 1s price simulator
      await act(async () => {
        await vi.advanceTimersByTimeAsync(5000);
      });

      // "No trades yet" should be gone
      expect(screen.queryByText('No trades yet')).not.toBeInTheDocument();
      // Trade rows should be visible (price/time columns present in trade feed)
      // Note: 'Price' and 'Amount' also appear in OPEN ORDERS table — use getAllByText
      expect(screen.getAllByText('Price').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('Amount').length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText('Time')).toBeInTheDocument();
    });

    it('renders many alerts in the SYSTEM ALERTS ledger tab', async () => {
      await renderModule();
      // Place many MARKET orders to generate alerts
      fireEvent.click(screen.getByText('MARKET'));
      for (let i = 0; i < 5; i++) {
        fireEvent.click(screen.getByText('PLACE BUY MARKET ORDER'));
      }

      // Switch to alerts tab
      fireEvent.click(screen.getByText('SYSTEM ALERTS'));

      // Should see multiple alert messages
      const alertMessages = screen.getAllByText(/Executed Market BUY order/);
      expect(alertMessages.length).toBe(5);
    });

    it('toggles between open orders, alerts, and assets tabs with large data', async () => {
      await renderModule();
      // Place some limit orders
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));
      fireEvent.click(screen.getByText('PLACE BUY LIMIT ORDER'));

      // Check OPEN tab shows 3 orders
      expect(screen.getAllByText('CANCEL').length).toBe(3);

      // Switch to SYSTEM ALERTS — limit orders should generate alerts
      fireEvent.click(screen.getByText('SYSTEM ALERTS'));
      expect(screen.getAllByText(/Placed Limit BUY order/).length).toBe(3);

      // Switch to LEDGER ASSETS
      fireEvent.click(screen.getByText('LEDGER ASSETS'));
      expect(screen.getByText('NET WORTH')).toBeInTheDocument();
      expect(screen.getByText('LIQUID CASH')).toBeInTheDocument();
    });
  });
});
