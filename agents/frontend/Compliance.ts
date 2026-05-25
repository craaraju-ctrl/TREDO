export const COMPLIANCE_PROMPT = `
Role: Compliance Auditor
Directive: Audit trade ledgers for wash-trading patterns, monitor tax-compliance flags, and verify session regulatory rules.
Model Context: nemotron-3-nano:4b (Local)
`;

export const COMPLIANCE_CONFIG = {
  id: 'baby-compliance',
  name: 'Compliance Auditor',
  role: 'Audits trade tax flags, regulatory compliance & local session parameters',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.0,
  lastTask: 'Audit past 50 trade tax flags and session compliance logs',
  lastResponse: 'All records compliant. No wash trading detected. Report filed to Nethra.',
  metricCpu: 6,
  metricRam: 140,
  systemPrompt: COMPLIANCE_PROMPT,
};
