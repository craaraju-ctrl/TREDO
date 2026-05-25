export const RISK_PROMPT = `
Role: Risk Shield
Directive: Perform deep-reasoning, margin allocation checks, Value-at-Risk (VaR) evaluations, and auto-stop rule audits.
Model Context: gemini-2.0-flash (Cloud) - Deep Reasoning & Exclusivity.
`;

export const RISK_CONFIG = {
  id: 'baby-risk',
  name: 'Risk Shield',
  role: 'Performs real-time VaR, margin limit checks & auto-stop allocation rules',
  status: 'idle' as const,
  assignedLLM: 'gemini-2.0-flash (Cloud)',
  temperature: 0.0,
  lastTask: 'Audit portfolio margin ratio exposure safety threshold',
  lastResponse: 'Margin exposure 12.4% (SECURE). VaR check approved. Work returned to Nethra.',
  metricCpu: 4,
  metricRam: 120,
  systemPrompt: RISK_PROMPT,
};
