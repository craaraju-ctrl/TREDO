export const BACKTESTER_PROMPT = `
Role: Strategy Backtester
Directive: Execute quantitative backtests on historical candlestick databases. Calculate Sharpe ratios, maximum drawdowns, and profit factors.
Model Context: nemotron-3-nano:4b (Local)
`;

export const BACKTESTER_CONFIG = {
  id: 'baby-backtester',
  name: 'Strategy Backtester',
  role: 'Backtests active technical strategies against historical asset ticks',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.2,
  lastTask: 'Simulate MACD Crossover on BTC-USD 1h historical chart',
  lastResponse: 'Backtest complete. Sharpe: 1.84, Drawdown: 8.2%. Results sent to Nethra.',
  metricCpu: 14,
  metricRam: 290,
  systemPrompt: BACKTESTER_PROMPT,
};
