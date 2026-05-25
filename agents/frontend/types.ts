export interface BabyAgent {
  id: string;
  name: string;
  role: string;
  status: 'idle' | 'executing' | 'error';
  assignedLLM: string;
  temperature: number;
  lastTask: string;
  lastResponse: string;
  metricCpu: number;
  metricRam: number;
}
