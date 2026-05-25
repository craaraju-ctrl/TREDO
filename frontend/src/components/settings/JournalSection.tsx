import { useState, useEffect, useCallback } from 'react';
import { useAtom } from 'jotai';
import { cn } from '../../lib/utils';
import { Badge, StatusDot } from '../ui/Badge';
import {
  performanceStatsAtom,
  autoTradingStateAtom,
  systemAlertsAtom,
  serverLogsAtom,
  calendarEventsAtom,
  coworkerTasksAtom,
  newsFeedAtom,
  portfolioHealthAtom,
  type PerformanceStats,
  type CalendarEvent,
  type CoworkerTask,
  type NewsHeadline,
  type PortfolioHealth,
  type TantraAlert,
} from '../../atoms/state';

// ── Interfaces ─────────────────────────────────────────────────────────────

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

interface Job {
  id: string;
  title: string;
  description: string;
  frequency: 'once' | 'daily' | 'weekly' | 'monthly';
  status: 'scheduled' | 'running' | 'completed' | 'failed';
  lastRun: string | null;
  nextRun: string | null;
  agent: string;
}

type JournalTab = 'overview' | 'trades' | 'decisions' | 'strategies' | 'tasks' | 'jobs' | 'calendar' | 'schedule' | 'alerts';

// ── Default Data ───────────────────────────────────────────────────────────

const DEFAULT_CALENDAR_EVENTS: CalendarEvent[] = [
  { id: 'cal-1', title: 'Operator Weekly Risk & Alignment Meeting', start: '09:00', end: '10:00', isDnd: true, status: 'ACTIVE' },
  { id: 'cal-2', title: 'Binance API Maintenance Upgrade Window', start: '14:00', end: '15:30', isDnd: false, status: 'PENDING' },
  { id: 'cal-3', title: 'End of Day Reconciliation', start: '16:00', end: '17:00', isDnd: false, status: 'PENDING' },
  { id: 'cal-4', title: 'Mid-Week Strategy Review', start: '11:00', end: '12:00', isDnd: true, status: 'PENDING' },
];

const DEFAULT_COWORKER_TASKS: CoworkerTask[] = [
  { id: 'task-1', title: 'Approve Exposure Limit Increase (SOL-USD)', priority: 'HIGH', status: 'PENDING', description: 'Autonomous agent requests allocation increase from 10% to 15% collateral.', category: 'Risk Review' },
  { id: 'task-2', title: 'Inspect In-flight Collateral Overlap', priority: 'MEDIUM', status: 'PENDING', description: 'Bids on BTC and ETH exceed standard concurrent threshold warning.', category: 'Trade Verify' },
  { id: 'task-3', title: 'Review New Exchange Integration Proposal', priority: 'LOW', status: 'PENDING', description: 'KuCoin integration request for additional trading pairs.', category: 'System Health' },
  { id: 'task-4', title: 'Update Risk Parameters for High Volatility Regime', priority: 'CRITICAL', status: 'PENDING', description: 'VIX spike detected — margin requirements need adjustment.', category: 'Risk Review' },
];

const DEFAULT_JOBS: Job[] = [
  { id: 'job-1', title: 'Hourly Market Scan', description: 'Scan all watchlist symbols for new signals', frequency: 'daily', status: 'running', lastRun: new Date(Date.now() - 3600000).toISOString(), nextRun: new Date(Date.now() + 3600000).toISOString(), agent: 'Nethra Swarm' },
  { id: 'job-2', title: 'Risk Parameter Recalibration', description: 'Recalculate VaR and margin requirements', frequency: 'daily', status: 'scheduled', lastRun: new Date(Date.now() - 86400000).toISOString(), nextRun: new Date(Date.now() + 43200000).toISOString(), agent: 'Risk Manager' },
  { id: 'job-3', title: 'Weekly Performance Report', description: 'Generate weekly P&L and win-rate report', frequency: 'weekly', status: 'scheduled', lastRun: new Date(Date.now() - 604800000).toISOString(), nextRun: new Date(Date.now() + 259200000).toISOString(), agent: 'Nethra Swarm' },
  { id: 'job-4', title: 'Model Fine-tuning Cycle', description: 'Fine-tune conviction models on new data', frequency: 'monthly', status: 'completed', lastRun: new Date(Date.now() - 1209600000).toISOString(), nextRun: new Date(Date.now() + 1814400000).toISOString(), agent: 'Tantra Monitor' },
];

// ── Main Journal Component ─────────────────────────────────────────────────

export default function JournalSection({
  activeTab: propActiveTab,
  setActiveTab: propSetActiveTab,
}: {
  activeTab?: JournalTab;
  setActiveTab?: (t: JournalTab) => void;
} = {}) {
  const [perfStats] = useAtom(performanceStatsAtom);
  const [autoTradingState] = useAtom(autoTradingStateAtom);
  const [internalActiveTab, setInternalActiveTab] = useState<JournalTab>('overview');
  
  const activeTab = propActiveTab || internalActiveTab;
  const setActiveTab = propSetActiveTab || setInternalActiveTab;
  const [trades, setTrades] = useState<Trade[]>([]);
  const [decisions, setDecisions] = useState<Decision[]>([]);
  const [strategyWinRates, setStrategyWinRates] = useState<Record<string, number>>({});
  const [regimeWinRates, setRegimeWinRates] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(true);

  // Tantra-derived state
  const [alerts, setAlerts] = useAtom(systemAlertsAtom);
  const [logs, setLogs] = useAtom(serverLogsAtom);
  const [calendarEvents, setCalendarEvents] = useAtom(calendarEventsAtom);
  const [coworkerTasks, setCoworkerTasks] = useAtom(coworkerTasksAtom);
  const [newsFeed] = useAtom(newsFeedAtom);
  const [portfolioHealth] = useAtom(portfolioHealthAtom);
  const [dndActive, setDndActive] = useState(true);
  const [jobs, setJobs] = useState<Job[]>(DEFAULT_JOBS);

  useEffect(() => {
    fetchJournalData();
    fetchTantraData();
    const interval = setInterval(() => {
      fetchJournalData();
      fetchTantraData();
    }, 15000);
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

  const fetchTantraData = async () => {
    try {
      const [statusRes, calRes, tasksRes] = await Promise.all([
        fetch('/api/tantra/status'),
        fetch('/api/tantra/calendar'),
        fetch('/api/tantra/tasks'),
      ]);

      if (statusRes.ok) {
        const statusData = await statusRes.json();
        if (statusData.status === 'success') setDndActive(statusData.dnd_active);
      }
      if (calRes.ok) {
        const calData = await calRes.json();
        if (calData.status === 'success') setCalendarEvents(calData.events);
      }
      if (tasksRes.ok) {
        const tasksData = await tasksRes.json();
        if (tasksData.status === 'success') setCoworkerTasks(tasksData.tasks);
      }
    } catch {
      // Backend not available — use defaults
      if (calendarEvents.length === 0) setCalendarEvents(DEFAULT_CALENDAR_EVENTS);
      if (coworkerTasks.length === 0) setCoworkerTasks(DEFAULT_COWORKER_TASKS);
    }
  };

  const tabs: { id: JournalTab; label: string; icon: string; count?: number }[] = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'trades', label: 'Trades', icon: '💰' },
    { id: 'decisions', label: 'Decisions', icon: '🧠' },
    { id: 'strategies', label: 'Strategies', icon: '📈' },
    { id: 'tasks', label: 'Tasks', icon: '✅', count: coworkerTasks.filter(t => t.status === 'PENDING').length },
    { id: 'jobs', label: 'Jobs', icon: '⚡', count: jobs.filter(j => j.status === 'running' || j.status === 'scheduled').length },
    { id: 'calendar', label: 'Calendar', icon: '📅' },
    { id: 'schedule', label: 'Schedule', icon: '⏰' },
    { id: 'alerts', label: 'Alerts', icon: '🔔', count: alerts.length },
  ];

  const activeContent = (
    <>
      {activeTab === 'overview' && (
        <OverviewTab stats={perfStats} autoTradingState={autoTradingState} portfolioHealth={portfolioHealth} />
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
      {activeTab === 'tasks' && (
        <TasksTab tasks={coworkerTasks} setTasks={setCoworkerTasks} logs={logs} setLogs={setLogs} alerts={alerts} setAlerts={setAlerts} />
      )}
      {activeTab === 'jobs' && (
        <JobsTab jobs={jobs} setJobs={setJobs} />
      )}
      {activeTab === 'calendar' && (
        <CalendarTab
          events={calendarEvents}
          setEvents={setCalendarEvents}
          dndActive={dndActive}
          setDndActive={setDndActive}
          setLogs={setLogs}
        />
      )}
      {activeTab === 'schedule' && (
        <ScheduleTab
          events={calendarEvents}
          dndActive={dndActive}
          setDndActive={setDndActive}
          newsFeed={newsFeed}
        />
      )}
      {activeTab === 'alerts' && (
        <AlertsTab alerts={alerts} logs={logs} portfolioHealth={portfolioHealth} />
      )}
    </>
  );

  if (propActiveTab) {
    return activeContent;
  }

  return (
    <div className="flex h-full gap-6">
      {/* Left sidebar */}
      <div className="w-52 flex flex-col gap-1 pr-2 border-r border-cyber-border/40 overflow-y-auto shrink-0">
        <h2 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-3 px-2">JOURNAL</h2>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-xs font-mono transition-all duration-200 ${
              activeTab === tab.id
                ? 'bg-cyber-purple/20 border border-cyber-purple/40 text-cyber-purple shadow-sm'
                : 'text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/30'
            }`}
          >
            <span className="text-sm">{tab.icon}</span>
            <span className="flex-1 text-left">{tab.label}</span>
            {tab.count !== undefined && tab.count > 0 && (
              <span className="text-[8px] bg-cyber-purple/30 text-cyber-purple font-bold px-1.5 py-0.5 rounded-full">
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-y-auto pr-4 scrollbar-cyber">
        {activeContent}
      </div>
    </div>
  );
}

// ── Overview Tab ──────────────────────────────────────────────────────────

function OverviewTab({
  stats,
  autoTradingState,
  portfolioHealth,
}: {
  stats: PerformanceStats | null;
  autoTradingState: any;
  portfolioHealth: PortfolioHealth;
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
        <h3 className="text-sm font-bold font-mono text-slate-200">Journal Overview</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Real-time trading performance, portfolio health & system status</p>
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

      {/* Win/Loss + Portfolio Health */}
      <div className="grid grid-cols-2 gap-6 mb-6">
        {/* Win/Loss Distribution */}
        {stats && (
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
        )}

        {/* Portfolio Health */}
        <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5">
          <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Portfolio Exposure Health</h4>
          <div className="grid grid-cols-2 gap-3">
            <HealthCard
              label="MARGIN EXPOSURE"
              value={`${portfolioHealth.marginRatio}%`}
              progress={portfolioHealth.marginRatio}
              variant="info"
            />
            <HealthCard
              label="RISK INDEX (VOL)"
              value={`${portfolioHealth.riskIndex.toFixed(2)}`}
              suffix="/ 1.00"
              progress={portfolioHealth.riskIndex * 100}
              variant="warning"
            />
            <HealthCard
              label="DAILY YIELD"
              value={`+${portfolioHealth.dailyYield}%`}
              subtext="+$2,450 P&L"
              variant="success"
              progress={Math.min(100, portfolioHealth.dailyYield * 10)}
            />
            <HealthCard
              label="VaR LIMIT"
              value={`$${portfolioHealth.valueAtRisk.toLocaleString()}`}
              subtext="Max loss guard"
              variant="danger"
              progress={Math.min(100, (portfolioHealth.valueAtRisk / 10000) * 100)}
            />
          </div>
        </div>
      </div>

      {/* Auto-Trading Status */}
      {stats && (
        <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5 mb-6">
          <h4 className="text-xs font-bold font-mono text-slate-300 mb-4">Auto-Trading Status</h4>
          <div className="grid grid-cols-5 gap-4">
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
              <span className="text-[10px] font-mono text-slate-500">Balance</span>
              <span className="text-[10px] font-mono text-cyber-green font-bold">${autoTradingState?.balance?.toLocaleString() ?? '100,000'}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-[10px] font-mono text-slate-500">Drawdown</span>
              <span className={`text-[10px] font-mono font-bold ${
                (autoTradingState?.current_drawdown_pct ?? 0) > 10 ? 'text-red-400' : 'text-slate-300'
              }`}>{autoTradingState?.current_drawdown_pct?.toFixed(1) ?? '0.0'}%</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-[10px] font-mono text-slate-500">Positions</span>
              <span className="text-[10px] font-mono text-slate-300">{autoTradingState?.open_positions?.length ?? 0}</span>
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

// ── Tasks Tab (from Tantra) ──────────────────────────────────────────────

function TasksTab({
  tasks,
  setTasks,
  logs,
  setLogs,
  alerts,
  setAlerts,
}: {
  tasks: CoworkerTask[];
  setTasks: (t: CoworkerTask[]) => void;
  logs: string[];
  setLogs: (l: string[]) => void;
  alerts: TantraAlert[];
  setAlerts: (a: TantraAlert[]) => void;
}) {
  const handleResolveTask = useCallback(async (taskId: string) => {
    setTasks(tasks.filter((t) => t.id !== taskId));
    try {
      await fetch('/api/tantra/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: taskId }),
      });
      setLogs([`[TantraService] Completed task: ${taskId}`, ...logs]);
      setAlerts([{
        alertId: `A-${Math.random()}`,
        source: 'TantraService',
        severity: 'Info',
        message: `Operator resolved coworker task: ${taskId}`,
        timestamp: Date.now(),
      }, ...alerts]);
    } catch {
      console.warn('Failed to resolve task');
    }
  }, [tasks, logs, alerts]);

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Coworker Task Queue</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Pending coworker coordination tasks requiring operator review</p>
      </div>

      {tasks.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-slate-500 font-mono text-xs gap-2">
          <span className="text-cyber-green text-3xl animate-pulse">✓</span>
          <span>All tasks resolved. System fully synchronized.</span>
        </div>
      ) : (
        <div className="space-y-3">
          {tasks.map((task) => (
            <div key={task.id} className="p-4 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl flex items-start justify-between gap-4 animate-slide-up">
              <div className="flex-1 space-y-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <Badge variant={task.priority === 'HIGH' || task.priority === 'CRITICAL' ? 'danger' : 'warning'}>
                    {task.priority}
                  </Badge>
                  <span className="text-[10px] font-mono text-slate-500">{task.category}</span>
                  <span className={cn(
                    'text-[8px] px-1.5 py-0.5 rounded border font-mono',
                    task.status === 'PENDING'
                      ? 'bg-amber-500/10 text-amber-400 border-amber-500/30'
                      : 'bg-cyber-green/10 text-cyber-green border-cyber-green/30'
                  )}>
                    {task.status}
                  </span>
                </div>
                <h4 className="text-xs font-bold text-slate-200">{task.title}</h4>
                <p className="text-[11px] text-slate-400 leading-relaxed">{task.description}</p>
              </div>
              {task.status === 'PENDING' && (
                <button
                  onClick={() => handleResolveTask(task.id)}
                  className="btn-success shrink-0 text-[11px]"
                  aria-label={`Approve task: ${task.title}`}
                >
                  APPROVE & ALIGN
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ── Jobs Tab ───────────────────────────────────────────────────────────────

function JobsTab({ jobs, setJobs }: { jobs: Job[]; setJobs: (j: Job[]) => void }) {
  const toggleJobStatus = (jobId: string) => {
    setJobs(jobs.map((j) =>
      j.id === jobId
        ? { ...j, status: j.status === 'running' ? 'scheduled' : 'running' }
        : j
    ));
  };

  const statusColors: Record<string, string> = {
    running: 'bg-cyber-green/10 text-cyber-green border-cyber-green/30',
    scheduled: 'bg-cyber-purple/10 text-cyber-purple border-cyber-purple/30',
    completed: 'bg-slate-500/10 text-slate-400 border-slate-500/30',
    failed: 'bg-red-500/10 text-red-400 border-red-500/30',
  };

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Scheduled Jobs</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Recurring tasks and automated job schedules</p>
      </div>

      <div className="space-y-3">
        {jobs.map((job) => (
          <div key={job.id} className="p-4 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl flex items-start justify-between gap-4">
            <div className="flex-1 space-y-2">
              <div className="flex items-center gap-2 flex-wrap">
                <StatusDot
                  status={job.status === 'running' ? 'active' : job.status === 'failed' ? 'danger' : 'inactive'}
                />
                <span className={`text-[8px] px-1.5 py-0.5 rounded border font-mono ${statusColors[job.status]}`}>
                  {job.status.toUpperCase()}
                </span>
                <span className="text-[10px] font-mono text-slate-500">{job.frequency.toUpperCase()}</span>
                <span className="text-[10px] font-mono text-cyber-purple">{job.agent}</span>
              </div>
              <h4 className="text-xs font-bold text-slate-200">{job.title}</h4>
              <p className="text-[11px] text-slate-400 leading-relaxed">{job.description}</p>
              <div className="flex gap-4 text-[9px] font-mono text-slate-500">
                {job.lastRun && <span>Last: {new Date(job.lastRun).toLocaleString()}</span>}
                {job.nextRun && <span>Next: {new Date(job.nextRun).toLocaleString()}</span>}
              </div>
            </div>
            <button
              onClick={() => toggleJobStatus(job.id)}
              className={`btn-primary shrink-0 text-[10px] ${job.status === 'completed' ? 'opacity-50 cursor-not-allowed' : ''}`}
              disabled={job.status === 'completed'}
            >
              {job.status === 'running' ? 'PAUSE' : job.status === 'scheduled' ? 'RUN NOW' : 'COMPLETED'}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Calendar Tab ──────────────────────────────────────────────────────────

function CalendarTab({
  events,
  setEvents,
  dndActive,
  setDndActive,
  setLogs,
}: {
  events: CalendarEvent[];
  setEvents: (e: CalendarEvent[]) => void;
  dndActive: boolean;
  setDndActive: (d: boolean) => void;
  setLogs: (l: string[] | ((prev: string[]) => string[])) => void;
}) {
  const handleToggleDnd = useCallback(async () => {
    const newDnd = !dndActive;
    setDndActive(newDnd);
    try {
      await fetch('/api/tantra/dnd', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active: newDnd }),
      });
      setLogs((prev: string[]) => [`[TantraService] DND schedule ${newDnd ? 'ENABLED' : 'DISABLED'}`, ...prev]);
    } catch {
      console.warn('Failed to toggle DND');
    }
  }, [dndActive]);

  const addEvent = () => {
    const newEvent: CalendarEvent = {
      id: `cal-${Date.now()}`,
      title: 'New Event',
      start: '12:00',
      end: '13:00',
      isDnd: false,
      status: 'PENDING',
    };
    setEvents([...events, newEvent]);
  };

  const toggleDndEvent = (eventId: string) => {
    setEvents(events.map((e) =>
      e.id === eventId ? { ...e, isDnd: !e.isDnd } : e
    ));
  };

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Calendar & DND Guard</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Schedule management with automatic Do-Not-Disturb trading blocks</p>
      </div>

      {/* DND Status Bar */}
      <div className="mb-6 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <StatusDot
            status={dndActive ? 'danger' : 'active'}
            label={dndActive ? 'TRADING BLOCKED' : 'EXECUTION LIVE'}
          />
          <div>
            <span className="text-[10px] font-mono text-slate-500 block">DND SCHEDULE GUARD</span>
            <span className="text-xs font-bold text-slate-300 font-mono">{dndActive ? 'Trading Suspended' : 'Armed & Active'}</span>
          </div>
        </div>
        <button
          onClick={handleToggleDnd}
          className={cn(
            'px-3 py-1.5 rounded font-mono text-xs font-bold transition-all border',
            dndActive
              ? 'bg-cyber-purple/20 hover:bg-cyber-purple/30 text-cyber-purple border-cyber-purple/40'
              : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border-red-500/40'
          )}
        >
          {dndActive ? 'RESUME TRADING' : 'FORCE PAUSE'}
        </button>
      </div>

      {/* Events */}
      <div className="space-y-3">
        {events.map((evt) => (
          <div key={evt.id} className="p-4 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl">
            <div className="flex justify-between items-center text-[10px] font-mono mb-2">
              <span className="text-cyber-glow font-bold">{evt.start} — {evt.end}</span>
              <div className="flex items-center gap-2">
                <Badge variant={evt.isDnd ? 'danger' : 'neutral'}>
                  {evt.isDnd ? 'DND GUARD' : 'STANDARD'}
                </Badge>
                <Badge variant={evt.status === 'ACTIVE' ? 'warning' : 'info'}>
                  {evt.status}
                </Badge>
                <button
                  onClick={() => toggleDndEvent(evt.id)}
                  className="text-[9px] px-2 py-0.5 rounded border font-mono text-slate-400 hover:text-slate-200 border-cyber-border/40"
                >
                  {evt.isDnd ? '→ STANDARD' : '→ DND'}
                </button>
              </div>
            </div>
            <h4 className="text-xs font-bold text-slate-300">{evt.title}</h4>
            {evt.isDnd && (
              <div className="mt-2 flex items-center gap-1 text-[9px] font-mono text-red-400/80">
                <span className="w-1.5 h-1.5 rounded-full bg-red-400 animate-pulse" />
                Trading automatically blocked during this event
              </div>
            )}
          </div>
        ))}
        <button
          onClick={addEvent}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all"
        >
          + Add Calendar Event
        </button>
      </div>
    </div>
  );
}

// ── Schedule Tab ──────────────────────────────────────────────────────────

function ScheduleTab({
  events,
  dndActive,
  setDndActive,
  newsFeed,
}: {
  events: CalendarEvent[];
  dndActive: boolean;
  setDndActive: (d: boolean) => void;
  newsFeed: NewsHeadline[];
}) {
  const hours = Array.from({ length: 14 }, (_, i) => i + 7); // 7 AM to 8 PM

  const getEventsForHour = (hour: number) => {
    const hh = hour.toString().padStart(2, '0');
    return events.filter((e) => e.start.startsWith(hh));
  };

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">Daily Schedule</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Timeline view of today's events, DND blocks, and news schedule</p>
      </div>

      {/* DND Status */}
      <div className="mb-4 flex items-center gap-3 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3">
        <StatusDot
          status={dndActive ? 'danger' : 'active'}
          label={dndActive ? 'DND ACTIVE — Trading Blocked' : 'Trading Allowed'}
        />
        <button
          onClick={() => setDndActive(!dndActive)}
          className="btn-primary text-[10px] px-2 py-1"
        >
          TOGGLE DND
        </button>
        <span className="text-[9px] font-mono text-slate-500 ml-auto">
          {events.filter(e => e.isDnd).length} DND events today
        </span>
      </div>

      {/* Timeline */}
      <div className="space-y-1">
        {hours.map((hour) => {
          const hourEvents = getEventsForHour(hour);
          const label = hour > 12 ? `${hour - 12}:00 PM` : `${hour}:00 AM`;

          return (
            <div key={hour} className="flex gap-3">
              <div className="w-16 text-right text-[9px] font-mono text-slate-500 pt-2 shrink-0">
                {label}
              </div>
              <div className="flex-1 min-h-[32px] border-t border-cyber-border/20 py-1 relative">
                {hourEvents.length === 0 ? (
                  <div className="h-full" />
                ) : (
                  <div className="flex gap-2">
                    {hourEvents.map((evt) => (
                      <div
                        key={evt.id}
                        className={cn(
                          'px-3 py-1.5 rounded text-[10px] font-mono border flex items-center gap-2',
                          evt.isDnd
                            ? 'bg-red-500/10 border-red-500/30 text-red-300'
                            : 'bg-cyber-purple/10 border-cyber-purple/30 text-cyber-purple'
                        )}
                      >
                        <span className={cn('w-1.5 h-1.5 rounded-full', evt.isDnd ? 'bg-red-400 animate-pulse' : 'bg-cyber-purple')} />
                        <span className="font-semibold">{evt.title}</span>
                        <span className="text-slate-500">{evt.start}-{evt.end}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* News Feed Summary */}
      <div className="mt-6 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4">
        <h4 className="text-xs font-bold font-mono text-slate-300 mb-3">Related News Feed</h4>
        {newsFeed.length === 0 ? (
          <p className="text-[10px] font-mono text-slate-500 text-center py-4">No recent news</p>
        ) : (
          <div className="space-y-2">
            {newsFeed.slice(0, 5).map((news) => (
              <div key={news.id} className="flex items-start gap-2 p-2 bg-cyber-dark/30 rounded-lg">
                <Badge variant={news.impact === 'HIGH' ? 'danger' : 'warning'}>
                  {news.impact}
                </Badge>
                <div className="flex-1">
                  <p className="text-[11px] text-slate-300">{news.headline}</p>
                  <span className="text-[8px] font-mono text-slate-500">{news.source} · {new Date(news.timestamp).toLocaleTimeString()}</span>
                </div>
                {news.symbolRelated && (
                  <span className="text-[8px] font-mono text-cyber-purple">{news.symbolRelated}</span>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ── Alerts Tab ─────────────────────────────────────────────────────────────

function AlertsTab({
  alerts,
  logs,
  portfolioHealth,
}: {
  alerts: TantraAlert[];
  logs: string[];
  portfolioHealth: PortfolioHealth;
}) {
  const [filter, setFilter] = useState<'all' | 'critical' | 'warning' | 'info'>('all');

  const filteredAlerts = alerts.filter((a) => {
    if (filter === 'all') return true;
    return a.severity.toLowerCase() === filter;
  });

  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">System Alerts & Audit Log</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">Real-time system alerts, alarms, and core audit trail</p>
      </div>

      {/* System Safety Status */}
      <div className="mb-6 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <StatusDot
            status={portfolioHealth.systemSafetyStatus === 'SECURE' ? 'active' : portfolioHealth.systemSafetyStatus === 'WARNING' ? 'warning' : 'danger'}
          />
          <div>
            <span className="text-[10px] font-mono text-slate-500 block">SYSTEM SAFETY STATUS</span>
            <span className={cn(
              'text-xs font-bold font-mono',
              portfolioHealth.systemSafetyStatus === 'SECURE' ? 'text-cyber-green' :
              portfolioHealth.systemSafetyStatus === 'WARNING' ? 'text-amber-400' : 'text-red-400'
            )}>
              {portfolioHealth.systemSafetyStatus}
            </span>
          </div>
        </div>
        <div className="flex gap-4 text-[9px] font-mono text-slate-400">
          <span>VaR: ${portfolioHealth.valueAtRisk.toLocaleString()}</span>
          <span>Risk Index: {portfolioHealth.riskIndex.toFixed(2)}</span>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-2 mb-4">
        {(['all', 'critical', 'warning', 'info'] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={cn(
              'text-[10px] font-bold font-mono px-3 py-1 rounded border transition-all',
              filter === f
                ? 'bg-cyber-purple/20 text-cyber-purple border-cyber-purple/40'
                : 'text-slate-500 border-transparent hover:text-slate-300'
            )}
          >
            {f.toUpperCase()} {f === 'all' ? `(${alerts.length})` : `(${alerts.filter(a => a.severity.toLowerCase() === f).length})`}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Alerts */}
        <div className="space-y-2">
          <h4 className="text-[10px] font-bold font-mono text-slate-400 mb-2 uppercase tracking-wider">Active Alarms</h4>
          {filteredAlerts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-slate-500 font-mono text-xs gap-2">
              <span className="text-cyber-green text-2xl">✓</span>
              <span>All clear — no active alerts</span>
            </div>
          ) : (
            filteredAlerts.map((alert) => (
              <div key={alert.alertId} className="p-3 bg-cyber-dark/40 border border-cyber-border/40 rounded-lg">
                <div className="flex justify-between items-center mb-1">
                  <span className="text-[9px] text-cyber-purple font-bold">{alert.source}</span>
                  <Badge variant={alert.severity === 'Critical' ? 'danger' : alert.severity === 'Warning' ? 'warning' : 'info'}>
                    {alert.severity}
                  </Badge>
                </div>
                <p className="text-slate-300 leading-relaxed text-[10px]">{alert.message}</p>
                <span className="text-[7px] text-slate-600 mt-1 block">
                  {new Date(alert.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))
          )}
        </div>

        {/* Audit Logs */}
        <div>
          <h4 className="text-[10px] font-bold font-mono text-slate-400 mb-2 uppercase tracking-wider">Core Audit Log</h4>
          <div className="bg-cyber-dark/80 border border-cyber-border rounded-lg p-3 font-mono text-[9px] max-h-[500px] overflow-y-auto space-y-1">
            {logs.length === 0 ? (
              <span className="text-slate-500">No log entries.</span>
            ) : (
              logs.map((log, i) => (
                <div key={i} className="text-cyber-green/80 leading-normal hover:text-cyber-green transition-colors">
                  <span className="text-slate-600 mr-1">[{i + 1}]</span>
                  {log}
                </div>
              ))
            )}
            <div className="text-slate-600 animate-pulse mt-2">$ tail -f sethu_bridge.log...</div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Health Card Subcomponent ──────────────────────────────────────────────

function HealthCard({
  label,
  value,
  suffix,
  subtext,
  progress,
  variant,
}: {
  label: string;
  value: string;
  suffix?: string;
  subtext?: string;
  progress: number;
  variant: 'success' | 'danger' | 'warning' | 'info';
}) {
  const variantStyles = {
    success: { text: 'text-cyber-green', bg: 'bg-cyber-green' },
    danger: { text: 'text-red-400', bg: 'bg-red-500' },
    warning: { text: 'text-cyber-glow', bg: 'bg-amber-400' },
    info: { text: 'text-cyber-purple', bg: 'bg-cyber-purple' },
  };
  const style = variantStyles[variant];

  return (
    <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3.5 flex flex-col gap-1.5">
      <span className="text-[9px] font-mono text-slate-500">{label}</span>
      <div className="flex items-baseline gap-1">
        <span className={cn('text-lg font-bold font-mono', style.text)}>{value}</span>
        {suffix && <span className="text-[10px] text-slate-600 font-mono">{suffix}</span>}
      </div>
      <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
        <div className={cn('h-full rounded-full transition-all duration-500', style.bg)}
          style={{ width: `${Math.min(100, Math.max(0, progress))}%` }} />
      </div>
      {subtext && <span className="text-[9px] text-slate-500 font-mono">{subtext}</span>}
    </div>
  );
}
