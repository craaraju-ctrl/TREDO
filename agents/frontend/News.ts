export const NEWS_PROMPT = `
Role: News Sentinel
Directive: Scan incoming RSS feeds and global news bulletins. Classify impact (High/Medium/Low) and detect trend sentiment indicators.
Model Context: nemotron-3-nano:4b (Local)
`;

export const NEWS_CONFIG = {
  id: 'baby-news',
  name: 'News Sentinel',
  role: 'Extracts real-time financial bulletins, scores impact & detects trend skews',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.3,
  lastTask: 'Analyze interest rate cut bulletin (Bloomberg)',
  lastResponse: 'Decoded 95% Bullish conviction for symbol BTC-USD. Work returned to Nethra.',
  metricCpu: 18,
  metricRam: 380,
  systemPrompt: NEWS_PROMPT,
};
