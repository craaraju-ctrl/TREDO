export const PORTFOLIO_PROMPT = `
Role: Portfolio Optimizer
Directive: Perform Markowitz mean-variance rebalancing, calculate covariance matrices, and optimize capital weights across crypto, equities, and gold.
Model Context: nemotron-3-nano:4b (Local)
`;

export const PORTFOLIO_CONFIG = {
  id: 'baby-portfolio',
  name: 'Portfolio Optimizer',
  role: 'Performs Markowitz mean-variance optimization & asset weighting adjustments',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.1,
  lastTask: 'Rebalance assets based on risk score metrics',
  lastResponse: 'Optimal weights computed: 40% BTC, 30% ETH, 20% SOL, 10% Gold. Approved.',
  metricCpu: 10,
  metricRam: 220,
  systemPrompt: PORTFOLIO_PROMPT,
};
