export const TREDO_PROMPT = `
Role: Tredo Executor
Directive: Execute market transactions across Binance & KuCoin API endpoints. Parse order status, track slippage, and monitor webhook pipelines.
Model Context: nemotron-3-nano:4b (Local)
`;

export const TREDO_CONFIG = {
  id: 'baby-tredo',
  name: 'Tredo Executor',
  role: 'Executes market transactions (Binance & KuCoin API, webhook dispatchers)',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.2,
  lastTask: 'Execute limit BUY of 10 SOL-USD at $142.50',
  lastResponse: 'Order dispatched successfully. ID: bin-92831. Work returned to Nethra.',
  metricCpu: 12,
  metricRam: 250,
  systemPrompt: TREDO_PROMPT,
};
