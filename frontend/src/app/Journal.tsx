import { useState, useEffect } from 'react';
import { useAtom } from 'jotai';
import { cn } from '../lib/utils';
import { newsFeedAtom, type NewsHeadline } from '../atoms/state';

// Additional 10 default, high-fidelity headlines covering global assets
const INITIAL_NEWS_HEADLINES: NewsHeadline[] = [
  {
    id: 'news-1',
    headline: 'US Federal Reserve Announces Interest Rate Cut of 25bps, Markets Rally',
    source: 'Bloomberg',
    impact: 'HIGH',
    timestamp: Date.now() - 600000, // 10m ago
    symbolRelated: 'BTC-USD',
  },
  {
    id: 'news-2',
    headline: 'SEC Formally Approves Multi-Asset Index Futures for Spot Tickers',
    source: 'Reuters',
    impact: 'HIGH',
    timestamp: Date.now() - 1500000, // 25m ago
    symbolRelated: 'ETH-USD',
  },
  {
    id: 'news-3',
    headline: 'Reliance Industries Launches Large-Scale Green Hydrogen Hub in Gujarat',
    source: 'Economic Times',
    impact: 'MEDIUM',
    timestamp: Date.now() - 3600000, // 1h ago
    symbolRelated: 'NSE:RELIANCE',
  },
  {
    id: 'news-4',
    headline: 'Gold Futures (GC=F) Breach $2,400 Resistance Level Amid Safe-Haven Flows',
    source: 'MarketWatch',
    impact: 'HIGH',
    timestamp: Date.now() - 5400000, // 1.5h ago
    symbolRelated: 'XAU-USD',
  },
  {
    id: 'news-5',
    headline: 'OPEC+ Details Strategy to Extend Crude Oil Production Cuts Into Next Quarter',
    source: 'Platts',
    impact: 'HIGH',
    timestamp: Date.now() - 7200000, // 2h ago
    symbolRelated: 'USOIL',
  },
  {
    id: 'news-6',
    headline: 'Solana (SOL) Network Active Addresses Surpass All-Time High in DeFi Surge',
    source: 'CoinDesk',
    impact: 'MEDIUM',
    timestamp: Date.now() - 10800000, // 3h ago
    symbolRelated: 'SOL-USD',
  },
  {
    id: 'news-7',
    headline: 'Silver Futures Rebound 3.4% as Industrial Solar Panel Demand Accelerates',
    source: 'Bloomberg',
    impact: 'MEDIUM',
    timestamp: Date.now() - 14400000, // 4h ago
    symbolRelated: 'XAG-USD',
  },
  {
    id: 'news-8',
    headline: 'NIFTY 50 Index Reaches Historic Milestone Led by Tech and Energy Sectors',
    source: 'NSE India',
    impact: 'HIGH',
    timestamp: Date.now() - 18000000, // 5h ago
    symbolRelated: 'NSE:NIFTY50',
  },
  {
    id: 'news-9',
    headline: 'Natural Gas Storage Inventories Fall Beyond Standard Seasonal Expectations',
    source: 'EIA',
    impact: 'LOW',
    timestamp: Date.now() - 25200000, // 7h ago
    symbolRelated: 'NGAS',
  },
  {
    id: 'news-10',
    headline: 'Apple Inc. Unveils Specialized Neural Core Processor Supporting Local LLMs',
    source: 'TechCrunch',
    impact: 'MEDIUM',
    timestamp: Date.now() - 32400000, // 9h ago
    symbolRelated: 'AAPL',
  }
];

export default function Journal() {
  const [newsFeed, setNewsFeed] = useAtom(newsFeedAtom);
  const [filterImpact, setFilterImpact] = useState<'ALL' | 'HIGH' | 'MEDIUM' | 'LOW'>('ALL');
  const [filterSymbol, setFilterSymbol] = useState<string>('ALL');
  const [searchQuery, setSearchQuery] = useState('');
  
  // Custom mock news creation states
  const [newHeadline, setNewHeadline] = useState('');
  const [newSource, setNewSource] = useState('Nethra Analytics');
  const [newImpact, setNewImpact] = useState<'HIGH' | 'MEDIUM' | 'LOW'>('MEDIUM');
  const [newSymbol, setNewSymbol] = useState('BTC-USD');
  const [showAddForm, setShowAddForm] = useState(false);

  // Time Zones
  const [sessionTimes, setSessionTimes] = useState({
    newYork: '',
    london: '',
    tokyo: '',
    mumbai: '',
  });

  // Load initial headlines if empty or short
  useEffect(() => {
    if (newsFeed.length <= 2) {
      setNewsFeed(INITIAL_NEWS_HEADLINES);
    }
  }, [newsFeed, setNewsFeed]);

  // Update market clocks every second
  useEffect(() => {
    const updateClocks = () => {
      const formatTime = (timeZone: string) => {
        return new Intl.DateTimeFormat('en-US', {
          timeZone,
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
          hour12: false,
        }).format(new Date());
      };
      
      setSessionTimes({
        newYork: formatTime('America/New_York'),
        london: formatTime('Europe/London'),
        tokyo: formatTime('Asia/Tokyo'),
        mumbai: formatTime('Asia/Kolkata'),
      });
    };
    updateClocks();
    const interval = setInterval(updateClocks, 1000);
    return () => clearInterval(interval);
  }, []);

  // Filter headlines
  const filteredNews = newsFeed.filter((item) => {
    const matchesImpact = filterImpact === 'ALL' || item.impact === filterImpact;
    const matchesSymbol = filterSymbol === 'ALL' || item.symbolRelated === filterSymbol;
    const matchesSearch = item.headline.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          item.source.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          (item.symbolRelated && item.symbolRelated.toLowerCase().includes(searchQuery.toLowerCase()));
    return matchesImpact && matchesSymbol && matchesSearch;
  });

  // Calculate Sentiment Scores
  const totalItems = filteredNews.length || 1;
  const highImpactCount = filteredNews.filter(n => n.impact === 'HIGH').length;
  const mediumImpactCount = filteredNews.filter(n => n.impact === 'MEDIUM').length;
  const lowImpactCount = filteredNews.filter(n => n.impact === 'LOW').length;

  // Sentiment ratio: 65% base + random or custom based on items
  const bullishRatio = Math.min(95, Math.max(25, Math.round(
    ((highImpactCount * 1.5 + mediumImpactCount * 1.1 + lowImpactCount * 0.8) / (totalItems * 1.2)) * 100
  )));
  const bearishRatio = 100 - bullishRatio;

  const handleAddNews = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newHeadline.trim()) return;

    const flashNews: NewsHeadline = {
      id: `news-${Date.now()}`,
      headline: newHeadline.trim(),
      source: newSource.trim() || 'Nethra Flash',
      impact: newImpact,
      timestamp: Date.now(),
      symbolRelated: newSymbol.trim(),
    };

    setNewsFeed([flashNews, ...newsFeed]);
    setNewHeadline('');
    setShowAddForm(false);
  };

  const getImpactBadge = (impact: 'HIGH' | 'MEDIUM' | 'LOW') => {
    switch (impact) {
      case 'HIGH':
        return <span className="bg-red-950/80 border border-red-500/50 text-red-400 font-bold font-mono px-2 py-0.5 rounded text-[8px] tracking-wide animate-pulse">HIGH IMPACT</span>;
      case 'MEDIUM':
        return <span className="bg-amber-950/80 border border-amber-500/50 text-amber-400 font-bold font-mono px-2 py-0.5 rounded text-[8px] tracking-wide">MED IMPACT</span>;
      case 'LOW':
        return <span className="bg-emerald-950/80 border border-emerald-500/50 text-emerald-400 font-bold font-mono px-2 py-0.5 rounded text-[8px] tracking-wide">LOW IMPACT</span>;
    }
  };

  return (
    <div className="flex flex-col gap-6 h-full overflow-hidden select-none">
      {/* Upper Global Sessions Bar */}
      <div className="grid grid-cols-4 gap-4 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-3 shrink-0">
        <div className="flex items-center justify-between px-3 py-1 border-r border-cyber-border/20">
          <div className="flex flex-col">
            <span className="text-[9px] font-mono text-slate-500 tracking-wider">NEW YORK (NYSE)</span>
            <span className="text-xs font-mono font-bold text-slate-300 mt-0.5">{sessionTimes.newYork}</span>
          </div>
          <span className="text-[8px] px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-cyber-green font-bold font-mono">OPEN</span>
        </div>
        <div className="flex items-center justify-between px-3 py-1 border-r border-cyber-border/20">
          <div className="flex flex-col">
            <span className="text-[9px] font-mono text-slate-500 tracking-wider">LONDON (LSE)</span>
            <span className="text-xs font-mono font-bold text-slate-300 mt-0.5">{sessionTimes.london}</span>
          </div>
          <span className="text-[8px] px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-cyber-green font-bold font-mono">OPEN</span>
        </div>
        <div className="flex items-center justify-between px-3 py-1 border-r border-cyber-border/20">
          <div className="flex flex-col">
            <span className="text-[9px] font-mono text-slate-500 tracking-wider">TOKYO (TSE)</span>
            <span className="text-xs font-mono font-bold text-slate-300 mt-0.5">{sessionTimes.tokyo}</span>
          </div>
          <span className="text-[8px] px-1.5 py-0.5 rounded bg-red-500/10 border border-red-500/30 text-red-400 font-bold font-mono">CLOSED</span>
        </div>
        <div className="flex items-center justify-between px-3 py-1">
          <div className="flex flex-col">
            <span className="text-[9px] font-mono text-slate-500 tracking-wider">MUMBAI (NSE)</span>
            <span className="text-xs font-mono font-bold text-slate-300 mt-0.5">{sessionTimes.mumbai}</span>
          </div>
          <span className="text-[8px] px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-cyber-green font-bold font-mono">OPEN</span>
        </div>
      </div>

      <div className="flex-1 grid grid-cols-12 gap-6 min-h-0">
        {/* Left Side: Sentiment Dashboard & Mock flash creator */}
        <div className="col-span-4 flex flex-col gap-5 min-h-0 overflow-y-auto pr-1 scrollbar-cyber">
          {/* Sentiment Meter Card */}
          <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5 relative overflow-hidden shrink-0">
            <div className="absolute top-0 right-0 w-24 h-24 bg-cyber-purple/10 rounded-full blur-xl pointer-events-none" />
            <h3 className="text-xs font-bold font-mono text-cyber-purple tracking-wider flex items-center gap-1.5 uppercase">
              <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-ping" />
              Nethra AI Sentiment Engine
            </h3>
            
            <div className="mt-5 flex flex-col items-center">
              <span className="text-2xl font-bold font-mono text-cyber-green tracking-tight">{bullishRatio}%</span>
              <span className="text-[8px] font-mono text-slate-500 mt-0.5 uppercase tracking-wide">Overall Bullish Conviction</span>
              
              {/* Slider Indicator Dial */}
              <div className="w-full h-2 bg-gradient-to-r from-red-500 via-amber-400 to-cyber-green rounded-full mt-4 relative">
                <div 
                  className="absolute w-4 h-4 bg-white rounded-full border-2 border-cyber-dark -top-1 shadow-[0_0_8px_rgba(255,255,255,0.8)] transition-all duration-500"
                  style={{ left: `calc(${bullishRatio}% - 8px)` }}
                />
              </div>
              
              <div className="flex justify-between w-full text-[8px] font-mono text-slate-500 mt-2">
                <span>BEARISH SKEW</span>
                <span>NEUTRAL ZONE</span>
                <span>BULLISH SKEW</span>
              </div>
            </div>

            {/* Sub-distribution bars */}
            <div className="mt-6 border-t border-cyber-border/20 pt-4 space-y-2.5 text-[9px] font-mono">
              <div className="flex justify-between items-center text-slate-400">
                <span>Bullish Volume Ratio</span>
                <span className="text-cyber-green font-bold">{bullishRatio}%</span>
              </div>
              <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                <div className="bg-cyber-green h-full rounded-full transition-all" style={{ width: `${bullishRatio}%` }} />
              </div>

              <div className="flex justify-between items-center text-slate-400">
                <span>Bearish Volume Ratio</span>
                <span className="text-red-400 font-bold">{bearishRatio}%</span>
              </div>
              <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                <div className="bg-red-500 h-full rounded-full transition-all" style={{ width: `${bearishRatio}%` }} />
              </div>

              <div className="flex justify-between items-center text-slate-400 mt-2">
                <span>Fear & Greed Sentiment</span>
                <span className="text-amber-400 font-bold">64 (Greed)</span>
              </div>
              <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                <div className="bg-amber-400 h-full rounded-full transition-all" style={{ width: '64%' }} />
              </div>
            </div>
          </div>

          {/* Flash News Injection Form */}
          <div className="bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5 shrink-0 transition-all duration-300">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-xs font-bold font-mono text-slate-300 uppercase tracking-wider">
                📢 Feed Mock Flash News
              </h3>
              <button 
                onClick={() => setShowAddForm(!showAddForm)}
                className="text-[9px] font-mono bg-cyber-purple/20 border border-cyber-purple/30 text-cyber-purple px-2 py-0.5 rounded hover:bg-cyber-purple/30 transition-all"
              >
                {showAddForm ? 'COLLAPSE' : 'EXPAND'}
              </button>
            </div>

            {showAddForm && (
              <form onSubmit={handleAddNews} className="space-y-3.5 animate-slide-up text-[10px] font-mono">
                <div>
                  <label className="text-slate-500 block mb-1">Headline Text</label>
                  <textarea
                    rows={3}
                    value={newHeadline}
                    onChange={(e) => setNewHeadline(e.target.value)}
                    required
                    placeholder="e.g. BTC Spot Inflows Exceed $1.2B in Single Session..."
                    className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2.5 py-1.5 text-slate-300 focus:outline-none focus:border-cyber-purple resize-none text-[10px]"
                  />
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-slate-500 block mb-1">Source</label>
                    <input
                      type="text"
                      value={newSource}
                      onChange={(e) => setNewSource(e.target.value)}
                      placeholder="e.g. Bloomberg"
                      className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2.5 py-1.5 text-slate-300 focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="text-slate-500 block mb-1">Related Asset</label>
                    <input
                      type="text"
                      value={newSymbol}
                      onChange={(e) => setNewSymbol(e.target.value)}
                      placeholder="e.g. BTC-USD"
                      className="w-full bg-cyber-dark border border-cyber-border/40 rounded px-2.5 py-1.5 text-slate-300 focus:outline-none"
                    />
                  </div>
                </div>

                <div>
                  <label className="text-slate-500 block mb-1">Impact Level</label>
                  <div className="flex gap-2">
                    {(['HIGH', 'MEDIUM', 'LOW'] as const).map((impact) => (
                      <button
                        key={impact}
                        type="button"
                        onClick={() => setNewImpact(impact)}
                        className={cn(
                          'flex-1 py-1 rounded text-[9px] font-bold font-mono border transition-all',
                          newImpact === impact
                            ? impact === 'HIGH' ? 'bg-red-500/20 text-red-400 border-red-500/50 shadow-sm'
                              : impact === 'MEDIUM' ? 'bg-amber-500/20 text-amber-400 border-amber-500/50 shadow-sm'
                              : 'bg-emerald-500/20 text-emerald-400 border-emerald-500/50 shadow-sm'
                            : 'bg-cyber-dark text-slate-500 border-cyber-border/40 hover:text-slate-300'
                        )}
                      >
                        {impact}
                      </button>
                    ))}
                  </div>
                </div>

                <button
                  type="submit"
                  className="w-full py-2 bg-gradient-to-r from-cyber-purple to-pink-600 hover:from-cyber-purple/90 hover:to-pink-600/90 text-white font-bold rounded-lg shadow-lg shadow-cyber-purple/20 transition-all uppercase text-[9px] tracking-wider mt-2"
                >
                  Broadcast Market News
                </button>
              </form>
            )}
            
            {!showAddForm && (
              <p className="text-[9px] font-mono text-slate-500 leading-relaxed">
                Inject custom macroeconomic bulletins, network upgrades, or policy rulings to evaluate Swarm alignment, base volatility, and active trading triggers.
              </p>
            )}
          </div>
        </div>

        {/* Right Side: News feed panel */}
        <div className="col-span-8 bg-cyber-dark/40 border border-cyber-border/40 rounded-xl p-5 flex flex-col min-h-0 overflow-hidden">
          {/* Header Actions */}
          <div className="flex justify-between items-center border-b border-cyber-border/20 pb-3 mb-4 shrink-0 gap-3">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold font-mono text-slate-200">Neural News Feeder</span>
              <span className="text-[8px] font-mono px-2 py-0.5 rounded bg-cyber-purple/20 text-cyber-purple border border-cyber-purple/30">
                {filteredNews.length} Headlines
              </span>
            </div>
            
            {/* Search Box */}
            <input
              type="text"
              placeholder="Search news, tickers, sources..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-cyber-dark/80 border border-cyber-border/40 rounded px-2.5 py-1 text-[9px] font-mono text-slate-300 w-44 focus:outline-none focus:border-cyber-purple"
            />
          </div>

          {/* Filters Bar */}
          <div className="flex flex-wrap gap-2.5 mb-4 shrink-0 items-center justify-between border-b border-cyber-border/10 pb-3">
            {/* Filter by impact */}
            <div className="flex items-center gap-1.5">
              <span className="text-[9px] font-mono text-slate-500 uppercase">IMPACT:</span>
              {(['ALL', 'HIGH', 'MEDIUM', 'LOW'] as const).map((imp) => (
                <button
                  key={imp}
                  onClick={() => setFilterImpact(imp)}
                  className={cn(
                    'text-[8px] font-bold font-mono px-2 py-0.5 rounded border transition-all',
                    filterImpact === imp
                      ? 'bg-cyber-purple/20 text-cyber-purple border-cyber-purple/40'
                      : 'text-slate-500 border-transparent hover:text-slate-300'
                  )}
                >
                  {imp}
                </button>
              ))}
            </div>

            {/* Filter by Related Asset */}
            <div className="flex items-center gap-1.5">
              <span className="text-[9px] font-mono text-slate-500 uppercase">ASSET:</span>
              <select
                value={filterSymbol}
                onChange={(e) => setFilterSymbol(e.target.value)}
                className="bg-cyber-dark border border-cyber-border/40 rounded px-2 py-0.5 text-[8px] font-mono text-slate-300 focus:outline-none focus:border-cyber-purple"
              >
                <option value="ALL">ALL ASSETS</option>
                <option value="BTC-USD">BTC-USD</option>
                <option value="ETH-USD">ETH-USD</option>
                <option value="SOL-USD">SOL-USD</option>
                <option value="XAU-USD">XAU-USD (Gold)</option>
                <option value="USOIL">Crude Oil</option>
                <option value="XAG-USD">Silver</option>
                <option value="NSE:RELIANCE">RELIANCE</option>
              </select>
            </div>
          </div>

          {/* Headlines List */}
          <div className="flex-1 overflow-y-auto space-y-2.5 pr-1 scrollbar-cyber">
            {filteredNews.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-20 text-slate-500 font-mono text-xs gap-2">
                <span className="text-2xl">📰</span>
                <span>No headlines match current criteria. Add new news flash!</span>
              </div>
            ) : (
              filteredNews.map((news) => (
                <div 
                  key={news.id} 
                  className="p-3.5 bg-cyber-dark/30 border border-cyber-border/20 rounded-xl hover:border-cyber-border/40 transition-all flex items-start gap-4 animate-slide-up hover:bg-cyber-panel/10"
                >
                  {/* Left Impact Dot indicator */}
                  <div className="mt-1.5 flex items-center justify-center shrink-0">
                    <span className={cn(
                      'w-2 h-2 rounded-full ring-4 shadow-[0_0_8px_currentColor]',
                      news.impact === 'HIGH' ? 'bg-red-500 text-red-500/20 ring-red-500/10' :
                      news.impact === 'MEDIUM' ? 'bg-amber-400 text-amber-400/20 ring-amber-400/10' :
                      'bg-emerald-500 text-emerald-500/20 ring-emerald-500/10'
                    )} />
                  </div>

                  {/* News Content */}
                  <div className="flex-1 space-y-1">
                    <div className="flex justify-between items-center gap-2">
                      <span className="text-[9px] font-mono text-slate-500">
                        {news.source} · {new Date(news.timestamp).toLocaleTimeString()}
                      </span>
                      <div className="flex items-center gap-2">
                        {news.symbolRelated && (
                          <span className="text-[8px] px-1.5 py-0.5 rounded bg-cyber-purple/10 text-cyber-purple border border-cyber-purple/20 font-bold font-mono">
                            {news.symbolRelated}
                          </span>
                        )}
                        {getImpactBadge(news.impact)}
                      </div>
                    </div>
                    <p className="text-xs font-semibold text-slate-200 leading-relaxed font-mono">
                      {news.headline}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
