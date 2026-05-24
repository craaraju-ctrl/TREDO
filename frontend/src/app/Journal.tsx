import { useState, useEffect } from 'react';
import { useAtom } from 'jotai';
import {
  performanceStatsAtom,
  autoTradingStateAtom,
  PerformanceStats,
} from '../atoms/state';

interface Trade {
  id: string;
  symbol: string;
  side: string;
  entry_price: number;
  exit_price: number | null;
  quantity: number;
  pnl: number | null;
  pnl_pct: number | null;
  conviction_at_entry: number;
  entry_reasoning: string;
  exit_reasoning: string | null;
  market_regime: string;
  strategies_used: string;
  open_time: string;
  close_time: string | null;
  is_open: boolean;
}

interface Decision {
  id: string;
  symbol: string;
  timestamp: string;
  overall_conviction: number;
  overall_direction: string;
  market_regime: string;
  action_taken: string;
  reason: string;
  bullish_signals: number;
  bearish_signals: number;
  neutral_signals: number;
}

type JournalTab = 'overview' | 'trades' | 'decisions' | 'strategies';

export default function Journal() {
  const [perfStats] = useAtom(performanceStatsAtom);
  const [autoTradingState] = useAtom(autoTradingStateAtom);
  const [activeTab, setActiveTab] = useState<JournalTab>('overview');
  const [trades, setTrades] = useState<Trade[]>([]);
  const [decisions, setDecisions] = useState<Decision[]>([]);
  const [strategyWinRates, setStrategyWinRates] = useState<Record<string, number>>({});
  const [regimeWinRates, setRegimeWinRates] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchJournalData();
    const interval = setInterval(fetchJournalData, 15000);
    return () => clearInterval(interval);
  }, []);

  const fetchJournalData = async () => {
    try {
      const [tradesRes, decisionsRes, swrRes, rwrRes] = await Promise.all([
        fetch('/api/journal/trades', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ limit: 50, offset: 0 }),
        }),
        fetch('/api/journal/decisions'),
        fetch('/api/journal/strategy-win-rates'),
        fetch('/api/journal/regime-win-rates'),
      ]);

      if (tradesRes.ok) {
        const data = await tradesRes.json();
        if (data.status === 'success') setTrades(data.trades);
      }
      if (decisionsRes.ok) {
        const data = await decisionsRes.json();
        if (data.status === 'success') setDecisions(data.decisions);
      }
      if (swrRes.ok) {
        const data = await swrRes.json();
        if (data.status === 'success') setStrategyWinRates(data.strategy_win_rates);
      }
      if (rwrRes.ok) {
        const data = await rwrRes.json();
        if (data.status === 'success') setRegimeWinRates(data.regime_win_rates);
      }
    } catch {
      console.warn('Journal API not available');
    } finally {
      setLoading(false);
    }
  };

  const tabs: { id: JournalTab; label: string; icon: string }[] = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'trades', label: 'Trades', icon: '💰' },
    { id: 'decisions', label: 'Decisions', icon: '🧠' },
    { id: 'strategies', label: 'Strategies', icon: '📈' },
  ];

  return (
    <div className="flex h-full gap-6">
      {/* Left sidebar */}
      <div className="w-48 flex flex-col gap-2 pr-2 border-r border-cyber-border/40">
        <h2 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-3 px-2">JOURNAL</h2>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-xs font-mono transition-all ${
              activeTab === tab.id
                ? 'bg-cyber-purple/20 border border-cyber-purple/40 text-cyber-purple'
                : 'text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/30'
            }`}
          >
            <span className="text-sm">{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-y-auto pr-4">
        {activeTab === 'overview' && (
          <OverviewTab stats={perfStats} autoTradingState={autoTradingState} />
        )}
        {activeTab === 'trades' && (
          <TradesTab trades={trades} loading={loading} />
        )}
        {activeTab === 'decisions' && (
          <DecisionsTab decisions={decisions} loading={loading} />
        )}
        {activeTab === 'strategies' && (
          <StrategiesTab
            strategyWinRates={strategyWinRates}
            regimeWinRates={regimeWinRates}
            trades={trades}
          />
        )}
      </div>
    </div>
  );
}

// ── Overview Tab ──────────────────────────────────────────────────────────

function OverviewTab({
  stats,
  autoTradingState,
}: {
  stats: PerformanceStats | null;
  autoTradingState: any;
}) {
  const statCards = stats ? [
    { label: 'Total Trades', value: stats.total_trades, color: 'text-slate-200', suffix: '' },
    { label: 'Win Rate', value: `${stats.win_rate.toFixed(1)}%`, color: stats.win_rate >= 50 ? 'text-cyber-green' : 'text-red-400', suffix: '' },
    { label: 'Total P&L', value: `$${stats.total_pnl.toFixed(0)}`, color: stats.total_pnl >= 0 ? 'text-cyber-green' : 'text-red-400', suffix: '' },
    { label: 'Profit Factor', value: stats.profit_factor.toFixed(2), color: stats.profit_factor >= 1.5 ? 'text-cyber-green' : stats.profit_factor >= 1.0 ? 'text-yellow-400' : 'text-red-400', suffix: '' },
    { label: 'Sharpe Ratio', value: stats.sharpe_ratio.toFixed(2), color: stats.sharpe_ratio >= 1.0 ? 'text-cyber-green' : stats.sharpe_ratio >= 0.5 ? 'text-yellow-400' : 'text-red-400', suffix: '' },
    { label: 'Max Drawdown', value: `${stats.max_drawdown.toFixed(1)}%`, color: stats.max_drawdown < 10 ? 'text-cyber-green' : stats.max_drawdown < 20 ? 'text-yellow-400' : 'text-red-400', suffix: '' },
    { label: 'Avg Win', value: `$${stats.avg_win.toFixed(0)}`, color: 'text-cyber-green', suffix: '' },
    { label: 'Avg Loss', value: `$${stats.avg_loss.toFixed(0)}`, color: 'text-red-400', suffix: '' },
  ] : [];

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Performance Overview</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Real-time trading performance metrics from the trade journal</p>
      </div>

      {/* Performance Gauges */}
      {stats && (
        <div className="grid grid-cols-4 gap-4 mb-6">
          {statCards.map((card, i) => (
            <div key={i} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4">
              <span className="text-[9px] font-mono text-slate-500 block mb-1">{card.label}</span>
              <span className={`text-xl font-bold font-mono ${card.color}`}>{card.value}</span>
              {card.label === 'Win Rate' && (
                <div className="w-full bg-slate-800 rounded-full h-1.5 mt-2 overflow-hidden">
                  <div
                    className={`h-full rounded-full ${stats.win_rate >= 50 ? 'bg-cyber-green' : 'bg-red-500'}`}
                    style={{ width: `${Math.min(100, stats.win_rate)}%` }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Win/Loss Distribution */}
      {stats && (
        <div className="grid grid-cols-2 gap-6 mb-6">
          <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
            <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Win / Loss Distribution</h4>
            <div className="flex items-end gap-2 h-32">
              <div className="flex-1 flex flex-col items-center">
                <span className="text-cyber-green text-lg font-bold font-mono">{stats.winning_trades}</span>
                <div className="w-full bg-cyber-green/20 rounded-t mt-1" style={{ height: `${Math.min(100, (stats.winning_trades / Math.max(stats.total_trades, 1)) * 100)}%` }} />
                <span className="text-[9px] text-slate-500 font-mono mt-1">Wins</span>
              </div>
              <div className="flex-1 flex flex-col items-center">
                <span className="text-red-400 text-lg font-bold font-mono">{stats.losing_trades}</span>
                <div className="w-full bg-red-500/20 rounded-t mt-1" style={{ height: `${Math.min(100, (stats.losing_trades / Math.max(stats.total_trades, 1)) * 100)}%` }} />
                <span className="text-[9px] text-slate-500 font-mono mt-1">Losses</span>
              </div>
            </div>
          </div>

          {/* Auto-Trading Status */}
          <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
            <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Auto-Trading Status</h4>
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-mono text-slate-500">Engine</span>
                <span className={`text-[10px] font-mono font-bold px-2 py-0.5 rounded border ${
                  autoTradingState?.enabled
                    ? 'text-cyber-green border-cyber-green/30 bg-cyber-green/10'
                    : 'text-slate-400 border-slate-500/30 bg-slate-500/10'
                }`}>{autoTradingState?.enabled ? 'RUNNING' : 'PAUSED'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-mono text-slate-500">Mode</span>
                <span className={`text-[10px] font-mono font-bold ${autoTradingState?.paper_trading ? 'text-yellow-400' : 'text-red-400'}`}>
                  {autoTradingState?.paper_trading ? 'PAPER' : 'REAL'}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-mono text-slate-500">Current Balance</span>
                <span className="text-[10px] font-mono text-cyber-green font-bold">${autoTradingState?.balance?.toLocaleString() ?? '100,000'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-mono text-slate-500">Drawdown</span>
                <span className={`text-[10px] font-mono font-bold ${
                  (autoTradingState?.current_drawdown_pct ?? 0) > 10 ? 'text-red-400' : 'text-slate-300'
                }`}>{autoTradingState?.current_drawdown_pct?.toFixed(1) ?? '0.0'}%</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-mono text-slate-500">Open Positions</span>
                <span className="text-[10px] font-mono text-slate-300">{autoTradingState?.open_positions?.length ?? 0}</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {!stats && (
        <div className="text-center py-12 text-slate-500 font-mono text-xs">
          No trade data yet. Start the auto-trading loop to generate performance metrics.
        </div>
      )}
    </div>
  );
}

// ── Trades Tab ────────────────────────────────────────────────────────────

function TradesTab({ trades, loading }: { trades: Trade[]; loading: boolean }) {
  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Trade History</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">All recorded trades from the auto-trading system</p>
      </div>
      {trades.length === 0 ? (
        <div className="text-center py-12 text-slate-500 font-mono text-xs">
          {loading ? 'Loading trades...' : 'No trades recorded yet.'}
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-[10px] font-mono text-left">
            <thead>
              <tr className="text-slate-500 border-b border-cyber-border/40">
                <th className="pb-2 pr-3">Time</th>
                <th className="pb-2 pr-3">Symbol</th>
                <th className="pb-2 pr-3">Side</th>
                <th className="pb-2 pr-3">Entry</th>
                <th className="pb-2 pr-3">Exit</th>
                <th className="pb-2 pr-3">Qty</th>
                <th className="pb-2 pr-3">P&L</th>
                <th className="pb-2 pr-3">P&L%</th>
                <th className="pb-2 pr-3">Conviction</th>
                <th className="pb-2 pr-3">Regime</th>
                <th className="pb-2">Status</th>
              </tr>
            </thead>
            <tbody>
              {trades.map((trade) => (
                <tr key={trade.id} className="border-b border-cyber-border/10 hover:bg-cyber-panel/20">
                  <td className="py-2 pr-3 text-slate-400">{new Date(trade.open_time).toLocaleDateString()}</td>
                  <td className="py-2 pr-3 text-slate-300 font-bold">{trade.symbol}</td>
                  <td className={`py-2 pr-3 font-bold ${trade.side === 'BUY' ? 'text-cyber-green' : 'text-red-400'}`}>{trade.side}</td>
                  <td className="py-2 pr-3 text-slate-300">${trade.entry_price.toFixed(2)}</td>
                  <td className="py-2 pr-3 text-slate-300">{trade.exit_price ? `$${trade.exit_price.toFixed(2)}` : '—'}</td>
                  <td className="py-2 pr-3 text-slate-300">{trade.quantity.toFixed(4)}</td>
                  <td className={`py-2 pr-3 font-bold ${trade.pnl && trade.pnl >= 0 ? 'text-cyber-green' : 'text-red-400'}`}>
                    {trade.pnl ? `${trade.pnl >= 0 ? '+' : ''}$${trade.pnl.toFixed(2)}` : '—'}
                  </td>
                  <td className={`py-2 pr-3 ${trade.pnl_pct && trade.pnl_pct >= 0 ? 'text-cyber-green' : 'text-red-400'}`}>
                    {trade.pnl_pct ? `${trade.pnl_pct >= 0 ? '+' : ''}${trade.pnl_pct.toFixed(2)}%` : '—'}
                  </td>
                  <td className="py-2 pr-3">
                    <span className={trade.conviction_at_entry > 0 ? 'text-cyber-green' : 'text-red-400'}>
                      {(trade.conviction_at_entry * 100).toFixed(0)}%
                    </span>
                  </td>
                  <td className="py-2 pr-3 text-slate-400">{trade.market_regime}</td>
                  <td className="py-2">
                    <span className={`text-[8px] px-1.5 py-0.5 rounded font-mono ${
                      trade.is_open
                        ? 'bg-yellow-500/10 text-yellow-400 border border-yellow-500/30'
                        : 'bg-slate-500/10 text-slate-400 border border-slate-500/30'
                    }`}>
                      {trade.is_open ? 'OPEN' : 'CLOSED'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// ── Decisions Tab ─────────────────────────────────────────────────────────

function DecisionsTab({ decisions, loading }: { decisions: Decision[]; loading: boolean }) {
  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Decision Log</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Every decision made by the autonomous trading loop</p>
      </div>
      {decisions.length === 0 ? (
        <div className="text-center py-12 text-slate-500 font-mono text-xs">
          {loading ? 'Loading decisions...' : 'No decisions recorded yet.'}
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-[10px] font-mono text-left">
            <thead>
              <tr className="text-slate-500 border-b border-cyber-border/40">
                <th className="pb-2 pr-3">Time</th>
                <th className="pb-2 pr-3">Symbol</th>
                <th className="pb-2 pr-3">Action</th>
                <th className="pb-2 pr-3">Conviction</th>
                <th className="pb-2 pr-3">Direction</th>
                <th className="pb-2 pr-3">Regime</th>
                <th className="pb-2 pr-3">Bullish</th>
                <th className="pb-2 pr-3">Bearish</th>
                <th className="pb-2">Reason</th>
              </tr>
            </thead>
            <tbody>
              {decisions.map((dec) => (
                <tr key={dec.id} className="border-b border-cyber-border/10 hover:bg-cyber-panel/20">
                  <td className="py-2 pr-3 text-slate-400">{new Date(dec.timestamp).toLocaleTimeString()}</td>
                  <td className="py-2 pr-3 text-slate-300 font-bold">{dec.symbol}</td>
                  <td className={`py-2 pr-3 font-bold ${
                    dec.action_taken === 'BUY' ? 'text-cyber-green' :
                    dec.action_taken === 'SELL' ? 'text-red-400' :
                    'text-slate-400'
                  }`}>{dec.action_taken}</td>
                  <td className={`py-2 pr-3 font-bold ${dec.overall_conviction > 0 ? 'text-cyber-green' : 'text-red-400'}`}>
                    {(dec.overall_conviction * 100).toFixed(0)}%
                  </td>
                  <td className="py-2 pr-3">
                    <span className={`text-[8px] px-1.5 py-0.5 rounded ${
                      dec.overall_direction === 'Bullish' ? 'bg-cyber-green/10 text-cyber-green' :
                      dec.overall_direction === 'Bearish' ? 'bg-red-500/10 text-red-400' :
                      'bg-slate-500/10 text-slate-400'
                    }`}>{dec.overall_direction}</span>
                  </td>
                  <td className="py-2 pr-3 text-slate-400">{dec.market_regime}</td>
                  <td className="py-2 pr-3 text-cyber-green">{dec.bullish_signals}</td>
                  <td className="py-2 pr-3 text-red-400">{dec.bearish_signals}</td>
                  <td className="py-2 text-slate-400 max-w-[200px] truncate">{dec.reason}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// ── Strategies Tab ────────────────────────────────────────────────────────

function StrategiesTab({
  strategyWinRates,
  regimeWinRates,
  trades,
}: {
  strategyWinRates: Record<string, number>;
  regimeWinRates: Record<string, number>;
  trades: Trade[];
}) {
  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Strategy & Regime Analysis</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Win rates broken down by strategy and market regime</p>
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Strategy Win Rates */}
        <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
          <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Strategy Win Rates</h4>
          {Object.keys(strategyWinRates).length === 0 ? (
            <p className="text-[10px] font-mono text-slate-500 text-center py-8">No strategy data yet</p>
          ) : (
            <div className="space-y-3">
              {Object.entries(strategyWinRates)
                .sort(([, a], [, b]) => b - a)
                .map(([strategy, rate]) => (
                  <div key={strategy} className="flex items-center gap-3">
                    <span className="text-[10px] font-mono text-slate-300 w-36 truncate">{strategy}</span>
                    <div className="flex-1 bg-slate-800 rounded-full h-2 overflow-hidden">
                      <div
                        className={`h-full rounded-full ${rate >= 50 ? 'bg-cyber-green' : 'bg-red-500'}`}
                        style={{ width: `${Math.min(100, rate)}%` }}
                      />
                    </div>
                    <span className={`text-[9px] font-mono font-bold w-10 text-right ${rate >= 50 ? 'text-cyber-green' : 'text-red-400'}`}>
                      {rate.toFixed(0)}%
                    </span>
                  </div>
                ))}
            </div>
          )}
        </div>

        {/* Regime Win Rates */}
        <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
          <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Regime Win Rates</h4>
          {Object.keys(regimeWinRates).length === 0 ? (
            <p className="text-[10px] font-mono text-slate-500 text-center py-8">No regime data yet</p>
          ) : (
            <div className="space-y-3">
              {Object.entries(regimeWinRates)
                .sort(([, a], [, b]) => b - a)
                .map(([regime, rate]) => (
                  <div key={regime} className="flex items-center gap-3">
                    <span className="text-[10px] font-mono text-slate-300 w-36 truncate capitalize">{regime.replace(/_/g, ' ')}</span>
                    <div className="flex-1 bg-slate-800 rounded-full h-2 overflow-hidden">
                      <div
                        className={`h-full rounded-full ${rate >= 50 ? 'bg-cyber-green' : 'bg-red-500'}`}
                        style={{ width: `${Math.min(100, rate)}%` }}
                      />
                    </div>
                    <span className={`text-[9px] font-mono font-bold w-10 text-right ${rate >= 50 ? 'text-cyber-green' : 'text-red-400'}`}>
                      {rate.toFixed(0)}%
                    </span>
                  </div>
                ))}
            </div>
          )}
        </div>
      </div>

      {/* Per-Trade P&L Chart */}
      {trades.length > 0 && (
        <div className="mt-6 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
          <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Trade P&L Waterfall</h4>
          <div className="flex items-end gap-1 h-40 overflow-x-auto pb-2">
            {[...trades].reverse().map((trade) => {
              const pnl = trade.pnl || 0;
              const maxAbs = Math.max(
                ...trades.map((t) => Math.abs(t.pnl || 0)),
                1
              );
              const height = Math.max(2, (Math.abs(pnl) / maxAbs) * 100);
              return (
                <div key={trade.id} className="flex flex-col items-center min-w-[16px]">
                  <div
                    className={`w-3 rounded-t ${pnl >= 0 ? 'bg-cyber-green' : 'bg-red-500'}`}
                    style={{ height: `${height}%` }}
                    title={`${trade.symbol}: ${pnl >= 0 ? '+' : ''}$${pnl.toFixed(2)}`}
                  />
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
