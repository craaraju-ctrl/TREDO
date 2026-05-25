export const ARBITRAGE_PROMPT = `
Role: Arbitrage Finder
Directive: Scour bid-ask spreads across multiple exchanges simultaneously to identify zero-risk delta pricing discrepancies.
Model Context: nemotron-3-nano:4b (Local)
`;

export const ARBITRAGE_CONFIG = {
  id: 'baby-arbitrage',
  name: 'Arbitrage Finder',
  role: 'Scans cross-exchange order books for immediate premium price mismatches',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.0,
  lastTask: 'Scan Binance vs KuCoin BTC spot order books',
  lastResponse: 'Mismatches scanned. Max spread: 0.02% (below threshold of 0.1%). No trade.',
  metricCpu: 22,
  metricRam: 410,
  systemPrompt: ARBITRAGE_PROMPT,
};
