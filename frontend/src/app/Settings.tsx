import { useState, type ReactNode } from 'react';
import { useAtom } from 'jotai';
import {
  settingsModelsAtom,
  settingsApiKeysAtom,
  settingsAgentsAtom,
  settingsSkillsAtom,
  settingsPromptsAtom,
  settingsToolsAtom,
  availableSkillsAtom,
  SettingsModel,
  SettingsApiKey,
  SettingsAgent,
  SettingsSkill,
  SettingsPrompt,
  SettingsTool,
} from '../atoms/state';
import JournalSection from '../components/settings/JournalSection';

type SettingsTab =
  | 'overview'
  | 'trades'
  | 'decisions'
  | 'strategies'
  | 'tasks'
  | 'jobs'
  | 'calendar'
  | 'schedule'
  | 'alerts'
  | 'models'
  | 'api-keys'
  | 'agents'
  | 'skills'
  | 'prompts'
  | 'tools'
  | 'connection';

export default function Settings() {
  const [activeSection, setActiveSection] = useState<SettingsTab>('overview');

  const [models, setModels] = useAtom(settingsModelsAtom);
  const [apiKeys, setApiKeys] = useAtom(settingsApiKeysAtom);
  const [agents, setAgents] = useAtom(settingsAgentsAtom);
  const [skills, setSkills] = useAtom(settingsSkillsAtom);
  const [prompts, setPrompts] = useAtom(settingsPromptsAtom);
  const [tools, setTools] = useAtom(settingsToolsAtom);
  const [availableSkills] = useAtom(availableSkillsAtom);

  const auditTabs: { id: SettingsTab; label: string; icon: string }[] = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'trades', label: 'Trades', icon: '💰' },
    { id: 'decisions', label: 'Decisions', icon: '🧠' },
    { id: 'strategies', label: 'Strategies', icon: '📈' },
    { id: 'tasks', label: 'Tasks', icon: '✅' },
    { id: 'jobs', label: 'Jobs', icon: '⚡' },
    { id: 'calendar', label: 'Calendar', icon: '📅' },
    { id: 'schedule', label: 'Schedule', icon: '⏰' },
    { id: 'alerts', label: 'Alerts', icon: '🔔' },
  ];

  const configTabs: { id: SettingsTab; label: string; icon: string }[] = [
    { id: 'models', label: 'Models', icon: '🧠' },
    { id: 'api-keys', label: 'API Keys', icon: '🔑' },
    { id: 'agents', label: 'Agents', icon: '🤖' },
    { id: 'skills', label: 'Skills', icon: '⚡' },
    { id: 'prompts', label: 'Prompts', icon: '📝' },
    { id: 'tools', label: 'Tools', icon: '🔧' },
    { id: 'connection', label: 'Connection', icon: '🌐' },
  ];

  const isJournalTab = auditTabs.some((t) => t.id === activeSection);

  return (
    <div className="flex h-full gap-6">
      {/* Left sidebar — settings navigation */}
      <div className="w-56 flex flex-col gap-1 pr-2 border-r border-cyber-border/40 shrink-0 overflow-y-auto scrollbar-cyber">
        <h2 className="text-[9px] font-bold font-mono tracking-widest text-slate-500 mb-2 px-2 uppercase">PERFORMANCE & AUDIT</h2>
        {auditTabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveSection(tab.id)}
            className={`flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-mono transition-all duration-200 ${
              activeSection === tab.id
                ? 'bg-cyber-purple/20 border border-cyber-purple/40 text-cyber-purple shadow-sm'
                : 'text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/30'
            }`}
          >
            <span className="text-sm">{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}

        <h2 className="text-[9px] font-bold font-mono tracking-widest text-slate-500 my-3 px-2 uppercase border-t border-cyber-border/20 pt-3">SYSTEM CONFIGURATION</h2>
        {configTabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveSection(tab.id)}
            className={`flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-mono transition-all duration-200 ${
              activeSection === tab.id
                ? 'bg-cyber-purple/20 border border-cyber-purple/40 text-cyber-purple shadow-sm'
                : 'text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/30'
            }`}
          >
            <span className="text-sm">{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Main content area */}
      <div className="flex-1 overflow-y-auto pr-4 scrollbar-cyber">
        {isJournalTab && (
          <JournalSection activeTab={activeSection as any} />
        )}
        {activeSection === 'models' && (
          <SettingsModelsSection models={models} setModels={setModels} />
        )}
        {activeSection === 'api-keys' && (
          <SettingsApiKeysSection apiKeys={apiKeys} setApiKeys={setApiKeys} />
        )}
        {activeSection === 'agents' && (
          <SettingsAgentsSection agents={agents} setAgents={setAgents} />
        )}
        {activeSection === 'skills' && (
          <SettingsSkillsSection
            skills={skills}
            setSkills={setSkills}
            availableSkills={availableSkills}
          />
        )}
        {activeSection === 'prompts' && (
          <SettingsPromptsSection prompts={prompts} setPrompts={setPrompts} />
        )}
        {activeSection === 'tools' && (
          <SettingsToolsSection tools={tools} setTools={setTools} />
        )}
        {activeSection === 'connection' && (
          <SettingsConnectionSection />
        )}
      </div>
    </div>
  );
}

// ── Models Section ────────────────────────────────────────────────────────

function SettingsModelsSection({
  models,
  setModels,
}: {
  models: SettingsModel[];
  setModels: (m: SettingsModel[]) => void;
}) {
  const [modelFilter, setModelFilter] = useState<'all' | 'cloud' | 'local'>('all');

  const addModel = () => {
    const newModel: SettingsModel = {
      id: `model-${Date.now()}`,
      name: 'New Model',
      provider: 'ollama',
      endpoint: 'http://localhost:11434',
      model_name: 'llama3.2',
      api_key_ref: '',
      active: false,
      max_tokens: 4096,
      temperature: 0.7,
      category: 'local',
    };
    setModels([...models, newModel]);
    saveSettings('models', [...models, newModel]);
  };

  const updateModel = (id: string, updates: Partial<SettingsModel>) => {
    setModels(models.map((m) => (m.id === id ? { ...m, ...updates } : m)));
    saveSettings('models', models.map((m) => (m.id === id ? { ...m, ...updates } : m)));
  };

  const removeModel = (id: string) => {
    setModels(models.filter((m) => m.id !== id));
    saveSettings('models', models.filter((m) => m.id !== id));
  };

  return (
    <SectionWrapper title="AI Models" description="Configure LLM providers and model endpoints">
      {/* Cloud / Local filter tabs */}
      <div className="flex gap-2 mb-4">
        {(['all', 'cloud', 'local'] as const).map((cat) => (
          <button
            key={cat}
            onClick={() => setModelFilter(cat)}
            className={`text-[10px] font-bold font-mono px-3 py-1 rounded border transition-all ${
              modelFilter === cat
                ? 'bg-cyber-purple/20 text-cyber-purple border-cyber-purple/40'
                : 'text-slate-500 border-transparent hover:text-slate-300'
            }`}
          >
            {cat === 'all' ? 'ALL' : cat === 'cloud' ? '☁️ CLOUD' : '💻 LOCAL'}
          </button>
        ))}
      </div>

      <div className="space-y-3">
        {models.filter(m => modelFilter === 'all' || m.category === modelFilter).map((model) => (
          <div key={model.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 space-y-3">
            <div className="flex justify-between items-center">
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={model.name}
                  onChange={(e) => updateModel(model.id, { name: e.target.value })}
                  className="bg-transparent border-b border-cyber-border/40 text-slate-200 font-bold font-mono text-xs px-2 py-1 focus:outline-none focus:border-cyber-purple"
                />
                {/* Category badge */}
                <span className={`text-[8px] px-1.5 py-0.5 rounded font-mono border ${
                  model.category === 'cloud' ? 'bg-cyber-purple/10 text-cyber-purple border-cyber-purple/30' : 'bg-cyber-green/10 text-cyber-green border-cyber-green/30'
                }`}>
                  {model.category === 'cloud' ? '☁️ CLOUD' : '💻 LOCAL'}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${model.active ? 'bg-cyber-green' : 'bg-slate-500'}`} />
                <button onClick={() => updateModel(model.id, { active: !model.active })}
                  className={`text-[9px] px-2 py-0.5 rounded border font-mono ${
                    model.active
                      ? 'bg-cyber-green/10 text-cyber-green border-cyber-green/30'
                      : 'bg-slate-500/10 text-slate-400 border-slate-500/30'
                  }`}>
                  {model.active ? 'ACTIVE' : 'INACTIVE'}
                </button>
                <button onClick={() => removeModel(model.id)} className="text-red-400/60 hover:text-red-400 text-xs">✕</button>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3 text-[10px] font-mono">
              <div>
                <label className="text-slate-500 block mb-1">Provider</label>
                <select value={model.provider} onChange={(e) => updateModel(model.id, { provider: e.target.value as any })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple">
                  <option value="ollama">Ollama</option>
                  <option value="gemini">Gemini</option>
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Category</label>
                <div className="flex gap-2 mt-1">
                  <button
                    onClick={() => updateModel(model.id, { category: 'cloud' })}
                    className={`flex-1 px-2 py-1.5 rounded text-[9px] font-bold font-mono border transition-all ${
                      model.category === 'cloud'
                        ? 'bg-cyber-purple/20 text-cyber-purple border-cyber-purple/40'
                        : 'bg-cyber-dark text-slate-500 border-cyber-border/40 hover:text-slate-300'
                    }`}
                  >
                    ☁️ Cloud
                  </button>
                  <button
                    onClick={() => updateModel(model.id, { category: 'local' })}
                    className={`flex-1 px-2 py-1.5 rounded text-[9px] font-bold font-mono border transition-all ${
                      model.category === 'local'
                        ? 'bg-cyber-green/20 text-cyber-green border-cyber-green/40'
                        : 'bg-cyber-dark text-slate-500 border-cyber-border/40 hover:text-slate-300'
                    }`}
                  >
                    💻 Local
                  </button>
                </div>
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Model Name</label>
                <input type="text" value={model.model_name} onChange={(e) => updateModel(model.id, { model_name: e.target.value })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Endpoint URL</label>
                <input type="text" value={model.endpoint} onChange={(e) => updateModel(model.id, { endpoint: e.target.value })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">API Key</label>
                <input type="password" value={model.api_key_ref} onChange={(e) => updateModel(model.id, { api_key_ref: e.target.value })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Max Tokens</label>
                <input type="number" value={model.max_tokens} onChange={(e) => updateModel(model.id, { max_tokens: parseInt(e.target.value) || 4096 })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Temperature</label>
                <div className="flex items-center gap-2">
                  <input type="range" min="0" max="2" step="0.1" value={model.temperature}
                    onChange={(e) => updateModel(model.id, { temperature: parseFloat(e.target.value) })}
                    className="flex-1 accent-cyber-purple" />
                  <span className="text-slate-300 w-8 text-right">{model.temperature.toFixed(1)}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
        <button onClick={addModel}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all">
          + Add Model
        </button>
      </div>
    </SectionWrapper>
  );
}

// ── API Keys Section ──────────────────────────────────────────────────────

function SettingsApiKeysSection({
  apiKeys,
  setApiKeys,
}: {
  apiKeys: SettingsApiKey[];
  setApiKeys: (k: SettingsApiKey[]) => void;
}) {
  const [apiKeyFilter, setApiKeyFilter] = useState<'all' | 'exchange' | 'news' | 'ai'>('all');

  const addKey = () => {
    const newKey: SettingsApiKey = {
      id: `key-${Date.now()}`,
      service: 'new-service',
      key: '',
      active: false,
      category: 'ai',
    };
    setApiKeys([...apiKeys, newKey]);
    saveSettings('api_keys', [...apiKeys, newKey]);
  };

  const updateKey = (id: string, updates: Partial<SettingsApiKey>) => {
    setApiKeys(apiKeys.map((k) => (k.id === id ? { ...k, ...updates } : k)));
    saveSettings('api_keys', apiKeys.map((k) => (k.id === id ? { ...k, ...updates } : k)));
  };

  const removeKey = (id: string) => {
    setApiKeys(apiKeys.filter((k) => k.id !== id));
    saveSettings('api_keys', apiKeys.filter((k) => k.id !== id));
  };

  return (
    <SectionWrapper title="API Keys" description="Manage API keys for external services">
      {/* Filter tabs */}
      <div className="flex gap-2 mb-4">
        {(['all', 'exchange', 'news', 'ai'] as const).map((cat) => (
          <button
            key={cat}
            onClick={() => setApiKeyFilter(cat)}
            className={`text-[10px] font-bold font-mono px-3 py-1 rounded border transition-all ${
              apiKeyFilter === cat
                ? 'bg-cyber-purple/20 text-cyber-purple border-cyber-purple/40'
                : 'text-slate-500 border-transparent hover:text-slate-300'
            }`}
          >
            {cat === 'all' ? 'ALL' : cat.toUpperCase()}
          </button>
        ))}
      </div>

      <div className="space-y-3">
        {apiKeys.filter(k => apiKeyFilter === 'all' || k.category === apiKeyFilter).map((key) => (
          <div key={key.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 flex items-center gap-4">
            <div className="flex-1 space-y-2">
              <div className="flex items-center gap-2">
                <input type="text" value={key.service}
                  onChange={(e) => updateKey(key.id, { service: e.target.value })}
                  className="bg-transparent border-b border-cyber-border/40 text-slate-300 font-mono text-xs px-2 py-1 focus:outline-none focus:border-cyber-purple flex-1"
                  placeholder="Service name (e.g., gemini, openai, binance)" />
                {/* Category badge */}
                <span className={`text-[8px] px-1.5 py-0.5 rounded font-mono border ${
                  key.category === 'exchange' ? 'bg-amber-500/10 text-amber-400 border-amber-500/30' :
                  key.category === 'news' ? 'bg-cyan-500/10 text-cyan-400 border-cyan-500/30' :
                  'bg-cyber-purple/10 text-cyber-purple border-cyber-purple/30'
                }`}>
                  {key.category === 'exchange' ? '📊 EXCHANGE' : key.category === 'news' ? '📰 NEWS' : '🤖 AI'}
                </span>
              </div>
              <div className="flex gap-2">
                <input type="password" value={key.key}
                  onChange={(e) => updateKey(key.id, { key: e.target.value })}
                  className="flex-1 bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 font-mono text-[10px] focus:outline-none focus:border-cyber-purple"
                  placeholder="API Key (stored locally, never sent to cloud)" />
                <select
                  value={key.category}
                  onChange={(e) => updateKey(key.id, { category: e.target.value as any })}
                  className="bg-cyber-dark border border-cyber-border/40 rounded px-1.5 py-1 text-[9px] font-mono text-slate-300 focus:outline-none focus:border-cyber-purple"
                >
                  <option value="exchange">Exchange</option>
                  <option value="news">News</option>
                  <option value="ai">AI</option>
                </select>
              </div>
            </div>
            <button onClick={() => updateKey(key.id, { active: !key.active })}
              className={`px-2.5 py-1 rounded text-[9px] font-mono border ${
                key.active
                  ? 'bg-cyber-green/10 text-cyber-green border-cyber-green/30'
                  : 'bg-slate-500/10 text-slate-400 border-slate-500/30'
              }`}>
              {key.active ? 'ENABLED' : 'DISABLED'}
            </button>
            <button onClick={() => removeKey(key.id)} className="text-red-400/60 hover:text-red-400 text-xs">✕</button>
          </div>
        ))}
        <button onClick={addKey}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all">
          + Add API Key
        </button>
      </div>
    </SectionWrapper>
  );
}

// ── Agents Section ────────────────────────────────────────────────────────

function SettingsAgentsSection({
  agents,
  setAgents,
}: {
  agents: SettingsAgent[];
  setAgents: (a: SettingsAgent[]) => void;
}) {
  const addAgent = () => {
    const newAgent: SettingsAgent = {
      id: `agent-${Date.now()}`,
      name: 'New Agent',
      role: 'analyst',
      model_id: '',
      system_prompt: 'You are a helpful trading assistant.',
      temperature: 0.7,
      max_tokens: 2048,
      active: false,
      tools: [],
    };
    setAgents([...agents, newAgent]);
    saveSettings('agents', [...agents, newAgent]);
  };

  const updateAgent = (id: string, updates: Partial<SettingsAgent>) => {
    setAgents(agents.map((a) => (a.id === id ? { ...a, ...updates } : a)));
    saveSettings('agents', agents.map((a) => (a.id === id ? { ...a, ...updates } : a)));
  };

  const removeAgent = (id: string) => {
    setAgents(agents.filter((a) => a.id !== id));
    saveSettings('agents', agents.filter((a) => a.id !== id));
  };

  return (
    <SectionWrapper title="Agents" description="Configure AI agent personas and behavior">
      <div className="space-y-3">
        {agents.map((agent) => (
          <div key={agent.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 space-y-3">
            <div className="flex justify-between items-center">
              <input type="text" value={agent.name}
                onChange={(e) => updateAgent(agent.id, { name: e.target.value })}
                className="bg-transparent border-b border-cyber-border/40 text-slate-200 font-bold font-mono text-xs px-2 py-1 focus:outline-none focus:border-cyber-purple" />
              <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${agent.active ? 'bg-cyber-green' : 'bg-slate-500'}`} />
                <button onClick={() => updateAgent(agent.id, { active: !agent.active })}
                  className={`text-[9px] px-2 py-0.5 rounded border font-mono ${
                    agent.active ? 'bg-cyber-green/10 text-cyber-green border-cyber-green/30' : 'bg-slate-500/10 text-slate-400 border-slate-500/30'
                  }`}>{agent.active ? 'ACTIVE' : 'INACTIVE'}</button>
                <button onClick={() => removeAgent(agent.id)} className="text-red-400/60 hover:text-red-400 text-xs">✕</button>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3 text-[10px] font-mono">
              <div>
                <label className="text-slate-500 block mb-1">Role</label>
                <select value={agent.role} onChange={(e) => updateAgent(agent.id, { role: e.target.value })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple">
                  <option value="analyst">Analyst</option>
                  <option value="trader">Trader</option>
                  <option value="risk-manager">Risk Manager</option>
                  <option value="researcher">Researcher</option>
                  <option value="executor">Executor</option>
                </select>
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Model ID</label>
                <input type="text" value={agent.model_id} onChange={(e) => updateAgent(agent.id, { model_id: e.target.value })}
                  placeholder="model-id or 'default'"
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div className="col-span-2">
                <label className="text-slate-500 block mb-1">System Prompt</label>
                <textarea value={agent.system_prompt} onChange={(e) => updateAgent(agent.id, { system_prompt: e.target.value })}
                  rows={3}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 font-mono text-[10px] focus:outline-none focus:border-cyber-purple resize-none" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Temperature</label>
                <input type="number" min="0" max="2" step="0.1" value={agent.temperature}
                  onChange={(e) => updateAgent(agent.id, { temperature: parseFloat(e.target.value) })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Max Tokens</label>
                <input type="number" value={agent.max_tokens}
                  onChange={(e) => updateAgent(agent.id, { max_tokens: parseInt(e.target.value) || 2048 })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
            </div>
          </div>
        ))}
        <button onClick={addAgent}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all">
          + Add Agent
        </button>
      </div>
    </SectionWrapper>
  );
}

// ── Skills Section ────────────────────────────────────────────────────────

function SettingsSkillsSection({
  skills,
  setSkills,
  availableSkills,
}: {
  skills: SettingsSkill[];
  setSkills: (s: SettingsSkill[]) => void;
  availableSkills: string[];
}) {
  const toggleSkill = (id: string) => {
    const updated = skills.map((s) =>
      s.id === id ? { ...s, enabled: !s.enabled } : s
    );
    setSkills(updated);
    saveSettings('skills', updated);
  };

  const updateSkill = (id: string, updates: Partial<SettingsSkill>) => {
    const updated = skills.map((s) => (s.id === id ? { ...s, ...updates } : s));
    setSkills(updated);
    saveSettings('skills', updated);
  };

  // If no skills loaded, use availableSkills from backend
  const displaySkills = skills.length > 0 ? skills : availableSkills.map((name, i) => ({
    id: `skill-${i}`,
    name,
    enabled: true,
    weight: 1.0,
    min_confidence: 0.0,
    params: {} as Record<string, number>,
  }));

  return (
    <SectionWrapper title="Skills" description="Toggle and configure trading skills">
      <div className="space-y-2">
        {displaySkills.map((skill) => (
          <div key={skill.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3 flex items-center gap-4">
            <button
              onClick={() => toggleSkill(skill.id)}
              className={`w-10 h-5 rounded-full transition-colors relative ${
                skill.enabled ? 'bg-cyber-green/40' : 'bg-slate-700'
              }`}
            >
              <div className={`w-4 h-4 rounded-full bg-white absolute top-0.5 transition-all ${
                skill.enabled ? 'left-5' : 'left-0.5'
              }`} />
            </button>
            <div className="flex-1">
              <span className="text-xs font-mono text-slate-300">{skill.name}</span>
            </div>
            <div className="flex items-center gap-3 text-[10px] font-mono">
              <div>
                <label className="text-slate-500 text-[8px] block">Weight</label>
                <input type="number" min="0" max="2" step="0.1" value={skill.weight}
                  onChange={(e) => updateSkill(skill.id, { weight: parseFloat(e.target.value) || 1.0 })}
                  className="w-14 bg-cyber-dark border border-cyber-border/40 rounded px-1.5 py-0.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 text-[8px] block">Min Conf</label>
                <input type="number" min="0" max="1" step="0.05" value={skill.min_confidence}
                  onChange={(e) => updateSkill(skill.id, { min_confidence: parseFloat(e.target.value) || 0.0 })}
                  className="w-14 bg-cyber-dark border border-cyber-border/40 rounded px-1.5 py-0.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
            </div>
          </div>
        ))}
      </div>
    </SectionWrapper>
  );
}

// ── Prompts Section ───────────────────────────────────────────────────────

function SettingsPromptsSection({
  prompts,
  setPrompts,
}: {
  prompts: SettingsPrompt[];
  setPrompts: (p: SettingsPrompt[]) => void;
}) {
  const addPrompt = () => {
    const newPrompt: SettingsPrompt = {
      id: `prompt-${Date.now()}`,
      name: 'New Prompt',
      template: 'Analyze the following market data: {{data}}',
      variables: ['data'],
      category: 'analysis',
    };
    setPrompts([...prompts, newPrompt]);
    saveSettings('prompts', [...prompts, newPrompt]);
  };

  const updatePrompt = (id: string, updates: Partial<SettingsPrompt>) => {
    setPrompts(prompts.map((p) => (p.id === id ? { ...p, ...updates } : p)));
    saveSettings('prompts', prompts.map((p) => (p.id === id ? { ...p, ...updates } : p)));
  };

  const removePrompt = (id: string) => {
    setPrompts(prompts.filter((p) => p.id !== id));
    saveSettings('prompts', prompts.filter((p) => p.id !== id));
  };

  return (
    <SectionWrapper title="Prompts" description="Customize AI prompt templates with variables">
      <div className="space-y-3">
        {prompts.map((prompt) => (
          <div key={prompt.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 space-y-3">
            <div className="flex justify-between items-center">
              <input type="text" value={prompt.name}
                onChange={(e) => updatePrompt(prompt.id, { name: e.target.value })}
                className="bg-transparent border-b border-cyber-border/40 text-slate-200 font-bold font-mono text-xs px-2 py-1 focus:outline-none focus:border-cyber-purple" />
              <div className="flex items-center gap-2">
                <select value={prompt.category} onChange={(e) => updatePrompt(prompt.id, { category: e.target.value })}
                  className="bg-cyber-dark border border-cyber-border/40 rounded px-2 py-0.5 text-[9px] font-mono text-slate-400 focus:outline-none focus:border-cyber-purple">
                  <option value="analysis">Analysis</option>
                  <option value="trading">Trading</option>
                  <option value="risk">Risk</option>
                  <option value="research">Research</option>
                  <option value="system">System</option>
                </select>
                <button onClick={() => removePrompt(prompt.id)} className="text-red-400/60 hover:text-red-400 text-xs">✕</button>
              </div>
            </div>
            <textarea value={prompt.template} onChange={(e) => updatePrompt(prompt.id, { template: e.target.value })}
              rows={4}
              className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-3 py-2 text-slate-300 font-mono text-[10px] focus:outline-none focus:border-cyber-purple resize-none" />
            <div className="flex items-center gap-2 text-[9px] font-mono text-slate-500">
              <span>Variables:</span>
              {prompt.variables.map((v) => (
                <span key={v} className="px-1.5 py-0.5 bg-cyber-purple/10 text-cyber-purple rounded border border-cyber-purple/20">{v}</span>
              ))}
            </div>
          </div>
        ))}
        <button onClick={addPrompt}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all">
          + Add Prompt Template
        </button>
      </div>
    </SectionWrapper>
  );
}

// ── Tools Section ─────────────────────────────────────────────────────────

function SettingsToolsSection({
  tools,
  setTools,
}: {
  tools: SettingsTool[];
  setTools: (t: SettingsTool[]) => void;
}) {
  const addTool = () => {
    const newTool: SettingsTool = {
      id: `tool-${Date.now()}`,
      name: 'New Tool',
      description: 'Tool description',
      endpoint: '',
      active: false,
      params: [],
    };
    setTools([...tools, newTool]);
    saveSettings('tools', [...tools, newTool]);
  };

  const updateTool = (id: string, updates: Partial<SettingsTool>) => {
    setTools(tools.map((t) => (t.id === id ? { ...t, ...updates } : t)));
    saveSettings('tools', tools.map((t) => (t.id === id ? { ...t, ...updates } : t)));
  };

  const removeTool = (id: string) => {
    setTools(tools.filter((t) => t.id !== id));
    saveSettings('tools', tools.filter((t) => t.id !== id));
  };

  return (
    <SectionWrapper title="Tools" description="Configure agent-callable tools and endpoints">
      <div className="space-y-3">
        {tools.map((tool) => (
          <div key={tool.id} className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-4 space-y-3">
            <div className="flex justify-between items-center">
              <input type="text" value={tool.name}
                onChange={(e) => updateTool(tool.id, { name: e.target.value })}
                className="bg-transparent border-b border-cyber-border/40 text-slate-200 font-bold font-mono text-xs px-2 py-1 focus:outline-none focus:border-cyber-purple" />
              <div className="flex items-center gap-2">
                <button onClick={() => updateTool(tool.id, { active: !tool.active })}
                  className={`text-[9px] px-2 py-0.5 rounded border font-mono ${
                    tool.active ? 'bg-cyber-green/10 text-cyber-green border-cyber-green/30' : 'bg-slate-500/10 text-slate-400 border-slate-500/30'
                  }`}>{tool.active ? 'ENABLED' : 'DISABLED'}</button>
                <button onClick={() => removeTool(tool.id)} className="text-red-400/60 hover:text-red-400 text-xs">✕</button>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3 text-[10px] font-mono">
              <div className="col-span-2">
                <label className="text-slate-500 block mb-1">Description</label>
                <input type="text" value={tool.description} onChange={(e) => updateTool(tool.id, { description: e.target.value })}
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Endpoint / Command</label>
                <input type="text" value={tool.endpoint} onChange={(e) => updateTool(tool.id, { endpoint: e.target.value })}
                  placeholder="/api/... or shell command"
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
              <div>
                <label className="text-slate-500 block mb-1">Parameters (comma-separated)</label>
                <input type="text" value={tool.params.join(', ')} onChange={(e) => updateTool(tool.id, { params: e.target.value.split(',').map(s => s.trim()).filter(Boolean) })}
                  placeholder="symbol, amount, price"
                  className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple" />
              </div>
            </div>
          </div>
        ))}
        <button onClick={addTool}
          className="w-full py-2.5 border border-dashed border-cyber-border/50 rounded-xl text-xs font-mono text-slate-500 hover:text-cyber-purple hover:border-cyber-purple/40 transition-all">
          + Add Tool
        </button>
      </div>
    </SectionWrapper>
  );
}

// ── Local Storage Persistence ─────────────────────────────────────────────

function saveSettings(key: string, data: any) {
  try {
    localStorage.setItem(`tredo_settings_${key}`, JSON.stringify(data));
  } catch {
    // localStorage may not be available
  }
}

// ── Section Wrapper ───────────────────────────────────────────────────────

function SectionWrapper({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div>
      <div className="mb-6">
        <h3 className="text-sm font-bold font-mono text-slate-200">{title}</h3>
        <p className="text-[10px] font-mono text-slate-500 mt-1">{description}</p>
      </div>
      {children}
    </div>
  );
}

// ── Connection Section ──────────────────────────────────────────────────

function SettingsConnectionSection() {
  const [apiBaseUrl, setApiBaseUrl] = useState(() => {
    try {
      const stored = localStorage.getItem('tredo_settings_api_base_url');
      return stored ? JSON.parse(stored) : '';
    } catch {
      return '';
    }
  });

  const saveUrl = (val: string) => {
    setApiBaseUrl(val);
    try {
      localStorage.setItem('tredo_settings_api_base_url', JSON.stringify(val));
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <SectionWrapper title="Backend Connection" description="Configure the API host address for remote client orchestration">
      <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5 space-y-4">
        <div>
          <label className="text-slate-400 block mb-2 font-mono text-[10px]">BACKEND API BASE URL</label>
          <input
            type="text"
            value={apiBaseUrl}
            onChange={(e) => saveUrl(e.target.value)}
            className="w-full bg-cyber-dark border border-cyber-border/40 rounded-lg px-3 py-2 text-slate-200 font-mono text-xs focus:outline-none focus:border-cyber-purple transition-all font-mono"
            placeholder="e.g. http://192.168.1.5:8080 or http://10.0.2.2:8080"
          />
          <p className="text-[9px] font-mono text-slate-500 mt-2 leading-relaxed">
            Leave blank to use relative paths (default for local desktop web browser). 
            For Android builds, configure this to your backend server's local IP address (e.g. <code className="text-cyber-purple bg-cyber-purple/10 px-1 rounded font-mono">http://192.168.1.15:8080</code>) or standard emulator loopback endpoint (<code className="text-cyber-purple bg-cyber-purple/10 px-1 rounded font-mono">http://10.0.2.2:8080</code>).
          </p>
        </div>
        
        <div className="border-t border-cyber-border/20 pt-4 flex items-center gap-3">
          <div className="w-2 h-2 rounded-full bg-cyber-green animate-pulse" />
          <span className="text-[9px] font-mono text-slate-400">
            Transparent relative request rewrite engine active
          </span>
        </div>
      </div>
    </SectionWrapper>
  );
}
