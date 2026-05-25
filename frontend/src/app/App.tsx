import { useEffect } from 'react';
import { useAtom } from 'jotai';
import { activeModuleAtom } from '../atoms/state';
import { Header } from '../components/layout/Header';
import { Navigation } from '../components/layout/Navigation';
import { ChatModule } from '../components/chat/ChatModule';
import { TredoModule } from '../components/tredo/TredoModule';
import { TantraModule } from '../components/tantra/TantraModule';
import { ToastContainer } from '../components/ui/Toast';
import Settings from './Settings';
import Journal from './Journal';
import type { ModuleTab } from '../lib/constants';

export default function App() {
  const [activeTab, setActiveTab] = useAtom(activeModuleAtom);

  // Persist active tab to localStorage
  useEffect(() => {
    try { localStorage.setItem('tredo_settings_active_module', JSON.stringify(activeTab)); } catch {}
  }, [activeTab]);

  const handleTabChange = (tab: ModuleTab) => {
    setActiveTab(tab);
  };

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-cyber-dark text-slate-100 select-none">
      <Header />
      <Navigation activeTab={activeTab} onTabChange={handleTabChange} />
      <ToastContainer />

      <main className="flex-1 min-h-0 overflow-hidden p-6 bg-cyber-dark/80" role="main">
        {activeTab === 'Chat' && (
          <div key="panel-chat" id="panel-Chat" role="tabpanel" aria-label="Chat module" className="h-full">
            <ChatModule />
          </div>
        )}
        {activeTab === 'Tredo' && (
          <div key="panel-tredo" id="panel-Tredo" role="tabpanel" aria-label="Trading module" className="h-full">
            <TredoModule />
          </div>
        )}
        {activeTab === 'Tantra' && (
          <div key="panel-tantra" id="panel-Tantra" role="tabpanel" aria-label="Systems monitoring module" className="h-full">
            <TantraModule />
          </div>
        )}
        {activeTab === 'Journal' && (
          <div key="panel-journal" id="panel-Journal" role="tabpanel" aria-label="Journal module" className="h-full">
            <Journal />
          </div>
        )}
        {activeTab === 'Settings' && (
          <div key="panel-settings" id="panel-Settings" role="tabpanel" aria-label="Settings module" className="h-full">
            <Settings />
          </div>
        )}
      </main>
    </div>
  );
}
