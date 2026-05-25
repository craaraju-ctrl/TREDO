export const TANTRA_PROMPT = `
Role: System Specialist
Directive: Monitor backend port status (default 8080/3001), detect database locks, verify TCP streams, and auto-scale telemetry index channels.
Model Context: nemotron-3-nano:4b (Local)
`;

export const TANTRA_CONFIG = {
  id: 'baby-tantra',
  name: 'System Specialist',
  role: 'Monitors bridge channels, database locks & auto-scales coworker telemetry',
  status: 'idle' as const,
  assignedLLM: 'nemotron-3-nano:4b (Local)',
  temperature: 0.1,
  lastTask: 'Check active TCP bridge connection on port 8080',
  lastResponse: 'ark-server connection ACTIVE. Zero telemetry dropouts. Work returned.',
  metricCpu: 8,
  metricRam: 180,
  systemPrompt: TANTRA_PROMPT,
};
