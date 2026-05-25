import re

# Load App.tsx contents
filepath = "/home/varma/Freebuff/tredo/frontend/src/app/App.tsx"
with open(filepath, "r") as f:
    content = f.read()

# Let's write the upgraded, fully professional Tredo JSX
upgraded_tredo_jsx = """        {/* --- FULL TRADING EXCHANGE COCKPIT --- */}
        {activeTab === 'Tredo' && (
          <div className="grid grid-cols-12 gap-6 h-full overflow-hidden">
            
            {/* COLUMN 1: Watchlist & Recent Trades Feed (3 cols) */}
            <div className={`${watchlistCollapsed ? 'col-span-1' : 'col-span-3'} flex flex-col gap-6 h-full overflow-hidden transition-all duration-300`}>
              
              {/* Asset Watchlist */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[48%] overflow-hidden relative">
                <div className="flex justify-between items-center mb-3">
                  {!watchlistCollapsed && (
                    <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400">WATCHLIST</h3>
                  )}
                  <div className="flex items-center space-x-1.5 ml-auto">
                    {!watchlistCollapsed && (
                      <button
                        onClick={() => setIsAddingAsset(!isAddingAsset)}
                        className="text-[10px] font-mono font-bold text-cyber-purple hover:text-white px-2 py-0.5 bg-cyber-purple/10 hover:bg-cyber-purple/35 rounded border border-cyber-purple/30 transition-all flex items-center gap-1"
                      >
                        <span>✙</span> Add Asset
                      </button>
                    )}
                    <button
                      onClick={() => setWatchlistCollapsed(!watchlistCollapsed)}
                      className="p-1 hover:bg-cyber-panel rounded text-slate-400 hover:text-slate-200 transition-colors"
                      title={watchlistCollapsed ? "Expand Watchlist" : "Collapse Watchlist"}
                    >
                      {watchlistCollapsed ? "▶" : "◀"}
                    </button>
                  </div>
                </div>

                {/* Inline Whitelist Builder Popover Form */}
                {isAddingAsset && !watchlistCollapsed && (
                  <div className="absolute top-12 left-3 right-3 bg-cyber-dark border border-cyber-purple/50 rounded-xl p-3.5 shadow-xl shadow-cyber-dark/80 z-20 font-mono text-xs">
                    <h4 className="text-[10px] font-bold text-cyber-purple uppercase tracking-wider mb-2.5">
                      Register Whitelist Symbol
                    </h4>
                    <div className="space-y-2.5">
                      <div>
                        <label className="text-[9px] text-slate-400 block mb-1">SYMBOL TICKER</label>
                        <input
                          type="text"
                          value={newAssetSymbol}
                          onChange={(e) => setNewAssetSymbol(e.target.value.toUpperCase().replace(/\\s/g, ''))}
                          placeholder="e.g. TSLA or DOGE-USD"
                          className="bg-cyber-dark/80 border border-cyber-border/40 focus:border-cyber-purple/80 text-xs px-2.5 py-1.5 rounded-lg w-full text-slate-200 outline-none transition-colors"
                        />
                      </div>
                      <div>
                        <label className="text-[9px] text-slate-400 block mb-1">STARTING BASE PRICE ($)</label>
                        <input
                          type="number"
                          value={newAssetPrice}
                          onChange={(e) => setNewAssetPrice(e.target.value)}
                          placeholder="e.g. 185.50"
                          className="bg-cyber-dark/80 border border-cyber-border/40 focus:border-cyber-purple/80 text-xs px-2.5 py-1.5 rounded-lg w-full text-slate-200 outline-none transition-colors"
                        />
                      </div>
                      <div className="flex gap-2 pt-1">
                        <button
                          onClick={() => {
                            const symbol = newAssetSymbol.trim();
                            const priceVal = parseFloat(newAssetPrice);
                            if (!symbol || isNaN(priceVal) || priceVal <= 0) {
                              setLogs((prev) => [`[ERROR] Whitelist Registration Failed: Invalid Symbol or Base Price`, ...prev]);
                              return;
                            }
                            if (watchlist.includes(symbol)) {
                              setLogs((prev) => [`[WARNING] Whitelist Registration Skipped: Symbol ${symbol} already exists`, ...prev]);
                              setIsAddingAsset(false);
                              return;
                            }
                            const updatedWatchlist = [...watchlist, symbol];
                            const updatedBasePrices = { ...basePrices, [symbol]: priceVal };
                            
                            setWatchlist(updatedWatchlist);
                            setBasePrices(updatedBasePrices);
                            
                            localStorage.setItem('tredo_settings_watchlist', JSON.stringify(updatedWatchlist));
                            localStorage.setItem('tredo_settings_base_prices', JSON.stringify(updatedBasePrices));

                            setSelectedAsset(symbol);
                            
                            setLogs((prev) => [
                              `[INFO] Whitelist Registered Symbol: ${symbol} at base price $${priceVal.toFixed(2)}`,
                              ...prev
                            ]);

                            setNewAssetSymbol('');
                            setNewAssetPrice('');
                            setIsAddingAsset(false);
                          }}
                          className="flex-1 px-3 py-1.5 bg-cyber-purple/20 hover:bg-cyber-purple/35 text-cyber-purple border border-cyber-purple/40 rounded-lg font-bold text-center transition-all"
                        >
                          REGISTER
                        </button>
                        <button
                          onClick={() => {
                            setNewAssetSymbol('');
                            setNewAssetPrice('');
                            setIsAddingAsset(false);
                          }}
                          className="px-3 py-1.5 bg-cyber-panel/50 hover:bg-cyber-panel text-slate-400 border border-cyber-border rounded-lg text-center transition-all"
                        >
                          CANCEL
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                <div className="flex-1 overflow-y-auto space-y-1.5 pr-1">
                  {watchlist.map((asset) => {
                    const price = basePrices[asset] || 100.0;
                    const open24h = open24hPrices[asset] || price;
                    const diff = price - open24h;
                    const pct = open24h > 0 ? (diff / open24h) * 100 : 0.0;
                    const flash = flashTickers[asset];

                    // Mini Sparkline polyline coordinates mock based on actual drift
                    const points = flash === 'up' ? "0,15 10,12 20,10 30,13 40,8 50,5" : flash === 'down' ? "0,5 10,8 20,12 30,10 40,14 50,18" : "0,12 10,11 20,13 30,12 40,11 50,12";

                    return (
                      <button
                        key={asset}
                        onClick={() => setSelectedAsset(asset)}
                        className={`w-full text-left rounded-lg font-mono text-xs flex justify-between items-center transition-all duration-300 relative overflow-hidden ${
                          watchlistCollapsed ? 'px-1 py-3 justify-center' : 'px-3 py-2.5'
                        } ${
                          selectedAsset === asset
                            ? 'bg-cyber-purple/25 border border-cyber-purple/40 text-cyber-purple shadow-[0_0_12px_rgba(157,78,221,0.15)]'
                            : 'bg-cyber-panel/20 border border-transparent text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/40'
                        } ${
                          flash === 'up' ? 'bg-green-500/10 border-green-500/30' : flash === 'down' ? 'bg-red-500/10 border-red-500/30' : ''
                        }`}
                      >
                        {watchlistCollapsed ? (
                          <div className="font-bold text-[10px] uppercase truncate">{asset.split('-')[0]}</div>
                        ) : (
                          <>
                            <div className="flex items-center space-x-2">
                              <span className="font-bold tracking-wider">{asset}</span>
                              <svg className="w-10 h-5 opacity-70" viewBox="0 0 50 20">
                                <polyline
                                  fill="none"
                                  stroke={pct >= 0 ? '#22c55e' : '#ef4444'}
                                  strokeWidth="1.2"
                                  points={points}
                                />
                              </svg>
                            </div>
                            <div className="flex items-center space-x-2 text-right">
                              <span className="font-semibold text-slate-100">${formatPrice(price)}</span>
                              <span className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                                pct >= 0 ? 'bg-cyber-green/10 text-cyber-green border border-cyber-green/20' : 'bg-red-500/10 text-red-400 border border-red-500/20'
                              }`}>
                                {pct >= 0 ? '+' : ''}{pct.toFixed(2)}%
                              </span>
                            </div>
                          </>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Scrolling Executed Trades Feed */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[52%] overflow-hidden">
                {!watchlistCollapsed && (
                  <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">RECENT MARKET TRADES</h3>
                )}
                <div className="flex-grow overflow-hidden relative">
                  {!watchlistCollapsed && (
                    <table className="w-full text-[10px] font-mono text-left">
                      <thead>
                        <tr className="text-slate-500 border-b border-cyber-border/40">
                          <th className="pb-1.5">Price</th>
                          <th className="pb-1.5 text-right">Amount</th>
                          <th className="pb-1.5 text-right">Time</th>
                        </tr>
                      </thead>
                    </table>
                  )}
                  <div className="h-full overflow-y-auto pr-1 space-y-1 mt-1 text-[10px]">
                    {tradesHistory.filter(t => t.symbol === selectedAsset).slice(0, 50).map((trade) => (
                      <div key={trade.id} className="flex justify-between font-mono py-0.5 border-b border-cyber-border/5 hover:bg-white/5 transition-all duration-300">
                        {watchlistCollapsed ? (
                          <span className={`font-bold ${trade.side === 'BUY' ? 'text-cyber-green' : 'text-red-400'}`}>
                            ${formatPrice(trade.price)}
                          </span>
                        ) : (
                          <>
                            <span className={trade.side === 'BUY' ? 'text-cyber-green font-semibold' : 'text-red-400 font-semibold'}>
                              {formatPrice(trade.price)}
                            </span>
                            <span className="text-slate-300 text-right">{trade.amount.toFixed(4)}</span>
                            <span className="text-slate-500 text-right">
                              {new Date(trade.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                            </span>
                          </>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* COLUMN 2: Live Canvas Candlestick Chart & Bottom Console Tab Ledger (6 cols) */}
            <div className={`${watchlistCollapsed ? 'col-span-8' : 'col-span-6'} flex flex-col gap-6 h-full overflow-hidden transition-all duration-300`}>
              
              {/* Candlestick Analytics Workspace */}
              <div className="flex-1 glass-panel rounded-xl p-4 flex flex-col overflow-hidden relative">
                
                {/* HUD Header Info */}
                <div className="flex justify-between items-center border-b border-cyber-border/40 pb-3 mb-3">
                  <div className="flex items-center space-x-3">
                    <h3 className="text-sm font-bold font-mono tracking-wider text-slate-200">
                      {selectedAsset}
                    </h3>
                    <span className="text-xs text-cyber-purple font-mono px-2 py-0.5 bg-cyber-purple/10 rounded animate-pulse">
                      Live Binance WS
                    </span>
                  </div>
                  <div className="flex space-x-3 text-xs font-mono items-center">
                    <span className="text-slate-400">Current Value:</span>
                    <span className="px-2.5 py-1 bg-cyber-panel rounded border border-cyber-border text-cyber-green font-bold shadow-green">
                      ${formatPrice(currentPrice)}
                    </span>
                  </div>
                </div>

                {/* Advanced Chart Control & Drawing Dock */}
                <div className="flex flex-wrap items-center justify-between gap-3 bg-cyber-dark/30 border border-cyber-border/20 rounded-lg p-2.5 mb-3 text-xs font-mono relative z-10">
                  
                  {/* Timeframe selector */}
                  <div className="flex items-center space-x-1 bg-cyber-panel/50 p-1 rounded border border-cyber-border/30">
                    <span className="text-[10px] text-slate-500 font-bold px-1.5 uppercase">Timeframe:</span>
                    {(['1m', '5m', '15m', '1h', '1d'] as const).map((tf) => (
                      <button
                        key={tf}
                        onClick={() => setSelectedTimeframe(tf)}
                        className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                          selectedTimeframe === tf
                            ? 'bg-cyber-purple/20 border border-cyber-purple/50 text-cyber-purple'
                            : 'text-slate-400 hover:text-slate-200'
                        }`}
                      >
                        {tf}
                      </button>
                    ))}
                  </div>

                  {/* Technical Indicators Toggles */}
                  <div className="flex items-center space-x-1 bg-cyber-panel/50 p-1 rounded border border-cyber-border/30 relative">
                    <span className="text-[10px] text-slate-500 font-bold px-1.5 uppercase">Indicators:</span>
                    {(['SMA', 'EMA', 'BB', 'VWAP'] as const).map((ind) => (
                      <div key={ind} className="relative inline-block">
                        <button
                          onClick={() =>
                            setActiveIndicators((prev) => ({ ...prev, [ind]: !prev[ind] }))
                          }
                          className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                            activeIndicators[ind]
                              ? ind === 'SMA'
                                ? 'bg-cyan-500/25 border border-cyan-500/50 text-cyan-400'
                                : ind === 'EMA'
                                ? 'bg-pink-500/25 border border-pink-500/50 text-pink-400'
                                : ind === 'BB'
                                ? 'bg-amber-500/25 border border-amber-500/50 text-amber-400'
                                : 'bg-slate-100/20 border border-slate-100/50 text-slate-100'
                              : 'text-slate-400 hover:text-slate-200'
                          }`}
                        >
                          {ind === 'SMA' ? `SMA (${indicatorPeriods.SMA})` : ind === 'EMA' ? `EMA (${indicatorPeriods.EMA})` : ind === 'BB' ? `BB (${indicatorPeriods.BB},2)` : 'VWAP'}
                        </button>
                        
                        {/* Period config settings popup */}
                        {ind !== 'VWAP' && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              setShowIndicatorSettings(showIndicatorSettings === ind ? null : ind);
                            }}
                            className="ml-0.5 p-0.5 hover:bg-white/10 rounded"
                            title="Configure Period"
                          >
                            ⚙
                          </button>
                        )}

                        {showIndicatorSettings === ind && (
                          <div className="absolute top-7 left-0 bg-cyber-dark border border-cyber-purple/40 rounded-lg p-2 z-40 space-y-1.5 shadow-xl font-mono text-[10px]">
                            <div className="flex justify-between items-center space-x-2">
                              <span className="text-slate-400">Period:</span>
                              <input
                                type="number"
                                className="w-10 bg-cyber-panel border border-cyber-border rounded text-center text-slate-200 font-bold"
                                value={indicatorPeriods[ind as keyof typeof indicatorPeriods]}
                                onChange={(e) => {
                                  const val = Math.max(1, Number(e.target.value));
                                  setIndicatorPeriods(prev => ({ ...prev, [ind]: val }));
                                }}
                              />
                            </div>
                            <button
                              onClick={() => setShowIndicatorSettings(null)}
                              className="w-full text-center bg-cyber-purple/20 hover:bg-cyber-purple/40 text-cyber-purple font-bold py-0.5 rounded"
                            >
                              Apply
                            </button>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>

                  {/* Interactive Support / Resistance drawing trigger */}
                  <div className="flex items-center space-x-1.5">
                    <button
                      onClick={() => {
                        const price = currentPrice;
                        setDrawingLines((prev) => [
                          ...prev,
                          {
                            id: `line-\${Math.random().toString(36).substring(4)}`,
                            type: 'SUPPORT',
                            price,
                            color: '#00e676',
                          },
                        ]);
                        setLogs((prev) => [`[INFO] Placed Horizontal Support Line at $\${formatPrice(price)}`, ...prev]);
                      }}
                      className="px-2.5 py-1 bg-cyber-green/10 hover:bg-cyber-green/20 text-cyber-green border border-cyber-green/30 rounded text-[10px] font-bold transition-all"
                    >
                      + Support
                    </button>
                    <button
                      onClick={() => {
                        const price = currentPrice;
                        setDrawingLines((prev) => [
                          ...prev,
                          {
                            id: `line-\${Math.random().toString(36).substring(4)}`,
                            type: 'RESISTANCE',
                            price,
                            color: '#ef4444',
                          },
                        ]);
                        setLogs((prev) => [`[INFO] Placed Horizontal Resistance Line at $\${formatPrice(price)}`, ...prev]);
                      }}
                      className="px-2.5 py-1 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 rounded text-[10px] font-bold transition-all"
                    >
                      + Resistance
                    </button>
                    {drawingLines.length > 0 && (
                      <button
                        onClick={() => {
                          setDrawingLines([]);
                          setLogs((prev) => [`[INFO] Cleared all drawing levels from chart`, ...prev]);
                        }}
                        className="p-1 text-slate-500 hover:text-slate-300 transition-colors"
                        title="Clear all drawing lines"
                      >
                        ✕
                      </button>
                    )}
                  </div>
                </div>

                {/* Lightweight-charts Container */}
                <div className="flex-1 relative bg-cyber-dark/40 border border-cyber-border/40 rounded-lg overflow-hidden min-h-[300px]">
                  <div ref={chartContainerRef} className="absolute inset-0 w-full h-full" />
                </div>
              </div>

              {/* Bottom Ledger Dashboard Console */}
              <div className="h-56 glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
                <div className="flex space-x-3 border-b border-cyber-border/30 pb-2 mb-3">
                  {(['OPEN', 'HISTORY', 'ASSETS'] as const).map((tab) => (
                    <button
                      key={tab}
                      onClick={() => setBottomTab(tab)}
                      className={`text-xs font-bold font-mono tracking-wider px-3 py-1 rounded transition-colors ${
                        bottomTab === tab
                          ? 'bg-cyber-purple/20 text-cyber-purple border border-cyber-purple/40'
                          : 'text-slate-400 hover:text-slate-200'
                      }`}
                    >
                      {tab === 'OPEN' ? 'OPEN ORDERS' : tab === 'HISTORY' ? 'SYSTEM ALERTS' : 'LEDGER ASSETS'}
                    </button>
                  ))}
                </div>

                <div className="flex-1 overflow-y-auto text-xs font-mono">
                  {bottomTab === 'OPEN' && (
                    <table className="w-full text-left">
                      <thead>
                        <tr className="text-slate-500 border-b border-cyber-border/20 text-[10px]">
                          <th>Symbol</th>
                          <th>Side</th>
                          <th>Price</th>
                          <th>Amount</th>
                          <th>Total</th>
                          <th>Action</th>
                        </tr>
                      </thead>
                      <tbody>
                        {openOrders.length === 0 ? (
                          <tr>
                            <td colSpan={6} className="text-center py-6 text-slate-500">
                              No active open limit orders. Place one below.
                            </td>
                          </tr>
                        ) : (
                          openOrders.map((order) => (
                            <tr key={order.id} className="border-b border-cyber-border/10 py-2">
                              <td className="py-2 text-slate-300 font-bold">{order.symbol}</td>
                              <td className={`py-2 font-bold ${order.side === 'BUY' ? 'text-cyber-green' : 'text-red-400'}`}>
                                {order.side}
                              </td>
                              <td className="py-2 text-slate-300">${formatPrice(order.price)}</td>
                              <td className="py-2 text-slate-300">{order.amount}</td>
                              <td className="py-2 text-cyber-purple font-semibold">
                                ${(order.price * order.amount).toFixed(2)}
                              </td>
                              <td className="py-2">
                                <button
                                  onClick={() => handleCancelOrder(order.id)}
                                  className="px-2.5 py-1 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 rounded text-[10px] transition-colors"
                                >
                                  CANCEL
                                </button>
                              </td>
                            </tr>
                          ))
                        )}
                      </tbody>
                    </table>
                  )}

                  {bottomTab === 'HISTORY' && (
                    <div className="space-y-1.5">
                      {alerts.map((alert) => (
                        <div key={alert.alertId} className="flex justify-between border-b border-cyber-border/10 py-1">
                          <span className="text-slate-300">{alert.message}</span>
                          <span className="text-slate-500 text-[10px]">
                            {new Date(alert.timestamp).toLocaleTimeString()}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}

                  {bottomTab === 'ASSETS' && (
                    <div className="grid grid-cols-3 gap-4 p-2">
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">TOTAL NET WORTH</span>
                        <p className="text-base font-bold text-cyber-green mt-1">${portfolioVal.toLocaleString()}</p>
                      </div>
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">LIQUID CASH</span>
                        <p className="text-base font-bold text-slate-200 mt-1">${cash.toLocaleString()}</p>
                      </div>
                      <div className="p-3 bg-cyber-panel/50 rounded-lg border border-cyber-border">
                        <span className="text-[10px] text-slate-400">ACTIVE TRADING ENGINE</span>
                        <p className="text-xs font-bold text-cyber-purple mt-2">Sethu ExecutionEngine</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* COLUMN 3: Dual-sided real-time L2 Order Book & Buy/Sell Order ticket Desk (3 cols) */}
            <div className="col-span-3 flex flex-col gap-6 h-full overflow-hidden">
              {/* Auto-Trading Control Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
                <div className="flex justify-between items-center border-b border-cyber-border/30 pb-2 mb-3">
                  <h3 className="text-xs font-bold font-mono tracking-wider text-cyber-purple">AUTONOMOUS TRADING</h3>
                  <div className="flex items-center space-x-2">
                    <span className={`w-2 h-2 rounded-full \${autoTradingState?.enabled ? 'bg-cyber-green animate-pulse' : 'bg-slate-500'}`} />
                    <span className="text-[10px] font-mono text-slate-400">
                      {autoTradingState?.enabled ? 'ACTIVE' : 'PAUSED'}
                    </span>
                  </div>
                </div>
                
                {/* Controls */}
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-2 text-[10px] font-mono">
                    <span className="text-slate-500">Mode:</span>
                    <span className={`px-2 py-0.5 rounded border text-[9px] \${
                      autoTradingState?.paper_trading
                        ? 'bg-yellow-500/10 text-yellow-400 border-yellow-500/30'
                        : 'bg-red-500/10 text-red-400 border-red-500/30'
                    }`}>
                      {autoTradingState?.paper_trading ? 'PAPER' : 'REAL'}
                    </span>
                  </div>
                  <div className="flex space-x-2">
                    <button
                      onClick={handleStartAutoTrade}
                      disabled={autoTradingState?.enabled}
                      className={`px-3 py-1.5 text-[10px] font-bold font-mono rounded transition-all border \${
                        autoTradingState?.enabled
                          ? 'bg-cyber-green/10 text-cyber-green/50 border-cyber-green/20 cursor-not-allowed'
                          : 'bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border-cyber-green/40 shadow-green'
                      }`}
                    >
                      START
                    </button>
                    <button
                      onClick={handleStopAutoTrade}
                      disabled={!autoTradingState?.enabled}
                      className={`px-3 py-1.5 text-[10px] font-bold font-mono rounded transition-all border \${
                        !autoTradingState?.enabled
                          ? 'bg-red-500/10 text-red-400/50 border-red-500/20 cursor-not-allowed'
                          : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border-red-500/40'
                      }`}
                    >
                      STOP
                    </button>
                  </div>
                </div>

                {/* Status info */}
                <div className="grid grid-cols-2 gap-2 text-[9px] font-mono">
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Balance</span>
                    <span className="text-cyber-green font-bold">
                      \${autoTradingState?.balance?.toLocaleString() ?? '100,000'}
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Positions</span>
                    <span className="text-slate-200 font-bold">
                      {autoTradingState?.open_positions?.length ?? 0}
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Drawdown</span>
                    <span className={`font-bold \${(autoTradingState?.current_drawdown_pct ?? 0) > 10 ? 'text-red-400' : 'text-slate-200'}`}>
                      {autoTradingState?.current_drawdown_pct?.toFixed(1) ?? '0.0'}%
                    </span>
                  </div>
                  <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
                    <span className="text-slate-500 block">Interval</span>
                    <span className="text-cyber-purple font-bold">
                      {autoTradingState?.analysis_interval_secs ?? 300}s
                    </span>
                  </div>
                </div>

                {/* Last decisions */}
                {autoTradingState?.last_outcomes && autoTradingState.last_outcomes.length > 0 && (
                  <div className="mt-3">
                    <h4 className="text-[9px] font-bold text-slate-500 mb-1.5 font-mono">RECENT DECISIONS</h4>
                    <div className="max-h-24 overflow-y-auto space-y-1">
                      {autoTradingState.last_outcomes.slice(-5).reverse().map((outcome, i) => (
                        <div key={i} className="flex justify-between items-center text-[8px] font-mono bg-cyber-dark/30 rounded px-2 py-1">
                          <span className="text-slate-400 truncate max-w-[100px]">{outcome.symbol}</span>
                          <span className={`font-bold \${
                            outcome.action?.Buy ? 'text-cyber-green' :
                            outcome.action?.Sell ? 'text-red-400' :
                            outcome.action?.Hold ? 'text-slate-400' : 'text-yellow-400'
                          }`}>
                            {outcome.action?.Buy ? 'BUY' : outcome.action?.Sell ? 'SELL' : outcome.action?.Hold ? 'HOLD' : 'SKIP'}
                          </span>
                          <span className="text-slate-500">{outcome.regime}</span>
                          <span className={outcome.conviction > 0 ? 'text-cyber-green' : 'text-red-400'}>
                            {(outcome.conviction * 100).toFixed(0)}%
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Performance Stats */}
                {perfStats && (
                  <div className="mt-3 border-t border-cyber-border/20 pt-3">
                    <h4 className="text-[9px] font-bold text-slate-500 mb-1.5 font-mono">PERFORMANCE</h4>
                    <div className="grid grid-cols-3 gap-1.5">
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className="text-[10px] font-bold text-cyber-green block">{perfStats.win_rate.toFixed(1)}%</span>
                        <span className="text-[7px] text-slate-500 font-mono">Win Rate</span>
                      </div>
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className="text-[10px] font-bold text-cyber-purple block">{perfStats.total_trades}</span>
                        <span className="text-[7px] text-slate-500 font-mono">Trades</span>
                      </div>
                      <div className="bg-cyber-dark/40 rounded p-1.5 text-center border border-cyber-border/20">
                        <span className={`text-[10px] font-bold block \${perfStats.total_pnl >= 0 ? 'text-cyber-green' : 'text-red-400'}`}>
                          \${perfStats.total_pnl.toFixed(0)}
                        </span>
                        <span className="text-[7px] text-slate-500 font-mono">Total P&L</span>
                      </div>
                    </div>
                  </div>
                )}
              </div>

              {/* Vertical Level 2 Orderbook Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[50%] overflow-hidden">
                <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">ORDER BOOK</h3>
                <div className="flex-1 flex flex-col overflow-hidden text-[10px] font-mono">
                  
                  {/* Asks (Sell orders) red */}
                  <div className="flex-1 overflow-y-auto space-y-0.5 mb-1 flex flex-col justify-end">
                    {l2Data.asks.map((ask, idx) => {
                      const maxAmount = Math.max(...l2Data.asks.map(a => a.amount), 1);
                      return (
                        <div key={idx} className="flex justify-between py-0.5 relative hover:bg-white/5 px-2 transition-colors">
                          <div
                            className="absolute right-0 top-0 bottom-0 bg-red-500/10 pointer-events-none transition-all duration-300"
                            style={{ width: `\${(ask.amount / maxAmount) * 100}%` }}
                          />
                          <span className="text-red-400 z-10 font-bold">\${formatPrice(ask.price)}</span>
                          <span className="text-slate-300 z-10 text-right">{ask.amount.toFixed(4)}</span>
                        </div>
                      );
                    })}
                  </div>

                  {/* Spread indicator bar */}
                  {l2Data.asks.length > 0 && l2Data.bids.length > 0 && (() => {
                    const bestAsk = l2Data.asks[l2Data.asks.length - 1].price;
                    const bestBid = l2Data.bids[0].price;
                    const spread = bestAsk - bestBid;
                    const spreadBps = bestBid > 0 ? (spread / bestBid) * 10000 : 0;
                    return (
                      <div className="py-1.5 border-y border-cyber-border/40 text-center font-mono my-1 text-cyber-purple font-semibold bg-cyber-panel/30 flex justify-between px-3 text-[10px]">
                        <span>SPREAD</span>
                        <span>\${formatPrice(spread)} ({spreadBps.toFixed(1)} bps)</span>
                      </div>
                    );
                  })()}

                  {/* Bids (Buy orders) green */}
                  <div className="flex-1 overflow-y-auto space-y-0.5 mt-1">
                    {l2Data.bids.map((bid, idx) => {
                      const maxAmount = Math.max(...l2Data.bids.map(b => b.amount), 1);
                      return (
                        <div key={idx} className="flex justify-between py-0.5 relative hover:bg-white/5 px-2 transition-colors">
                          <div
                            className="absolute right-0 top-0 bottom-0 bg-cyber-green/10 pointer-events-none transition-all duration-300"
                            style={{ width: `\${(bid.amount / maxAmount) * 100}%` }}
                          />
                          <span className="text-cyber-green z-10 font-bold">\${formatPrice(bid.price)}</span>
                          <span className="text-slate-300 z-10 text-right">{bid.amount.toFixed(4)}</span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>

              {/* Order Form Widget Desk Panel */}
              <div className="glass-panel rounded-xl p-4 flex flex-col h-[50%] overflow-hidden">
                <div className="flex border-b border-cyber-border/40 pb-2 mb-3 justify-between items-center">
                  <div className="flex space-x-1.5">
                    {(['LIMIT', 'MARKET'] as const).map((t) => (
                      <button
                        key={t}
                        onClick={() => setOrderType(t)}
                        className={`text-[10px] font-bold font-mono px-2 py-0.5 rounded \${
                          orderType === t ? 'bg-cyber-panel border border-cyber-border text-slate-200' : 'text-slate-500 hover:text-slate-300'
                        }`}
                      >
                        {t}
                      </button>
                    ))}
                  </div>
                  <div className="flex bg-cyber-panel p-0.5 rounded border border-cyber-border/40">
                    <button
                      onClick={() => setOrderSide('BUY')}
                      className={`text-[10px] font-bold font-mono px-3 py-0.5 rounded transition-all \${
                        orderSide === 'BUY' ? 'bg-cyber-green/20 text-cyber-green shadow-[0_0_8px_rgba(34,197,94,0.2)]' : 'text-slate-500'
                      }`}
                    >
                      BUY
                    </button>
                    <button
                      onClick={() => setOrderSide('SELL')}
                      className={`text-[10px] font-bold font-mono px-3 py-0.5 rounded transition-all \${
                        orderSide === 'SELL' ? 'bg-red-500/20 text-red-400 shadow-[0_0_8px_rgba(239,68,68,0.2)]' : 'text-slate-500'
                      }`}
                    >
                      SELL
                    </button>
                  </div>
                </div>

                <div className="flex-1 flex flex-col justify-between text-xs font-mono space-y-3">
                  <div className="space-y-2">
                    <div className="flex justify-between items-center text-[10px] text-slate-500">
                      <span>AVAILABLE BALANCE:</span>
                      <span className="text-slate-200 font-bold">\${cash.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                    </div>

                    {orderType === 'LIMIT' && (
                      <div>
                        <label className="text-[10px] text-slate-500 block mb-1">LIMIT PRICE (USD)</label>
                        <div className="flex">
                          <button
                            onClick={() => {
                              const p = Number(limitPriceInput) || currentPrice;
                              const step = p * 0.001;
                              setLimitPriceInput(formatPrice(Math.max(0, p - step)));
                            }}
                            className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-l text-[10px] font-bold"
                          >
                            -
                          </button>
                          <input
                            type="text"
                            value={limitPriceInput}
                            onChange={(e) => setLimitPriceInput(e.target.value)}
                            className="w-full bg-cyber-dark/60 border-y border-cyber-border text-center text-slate-200 focus:outline-none focus:border-cyber-purple font-mono py-1"
                          />
                          <button
                            onClick={() => {
                              const p = Number(limitPriceInput) || currentPrice;
                              const step = p * 0.001;
                              setLimitPriceInput(formatPrice(p + step));
                            }}
                            className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-r text-[10px] font-bold"
                          >
                            +
                          </button>
                        </div>
                      </div>
                    )}

                    <div>
                      <label className="text-[10px] text-slate-500 block mb-1">QUANTITY</label>
                      <input
                        type="text"
                        value={amountInput}
                        onChange={(e) => setAmountInput(e.target.value)}
                        className="w-full bg-cyber-dark/60 border border-cyber-border rounded text-center text-slate-200 focus:outline-none focus:border-cyber-purple font-mono py-1"
                      />
                    </div>

                    {/* Percentage Slider stop buttons */}
                    <div className="grid grid-cols-4 gap-1">
                      {[0.25, 0.5, 0.75, 1.0].map((pct) => (
                        <button
                          key={pct}
                          onClick={() => {
                            const maxCost = cash;
                            const price = orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice;
                            if (price > 0) {
                              const qty = (maxCost * pct) / price;
                              setAmountInput(qty.toFixed(4));
                            }
                          }}
                          className="bg-cyber-panel hover:bg-cyber-border/40 text-slate-400 border border-cyber-border rounded py-1 text-[9px] font-bold transition-all"
                        >
                          {pct * 100}%
                        </button>
                      ))}
                    </div>

                    <div className="border-t border-cyber-border/20 pt-2 flex justify-between text-[10px] text-slate-500">
                      <span>EST. TOTAL VALUE:</span>
                      <span className="text-cyber-purple font-bold">
                        \${((orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice) * Number(amountInput || 0)).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                      </span>
                    </div>
                  </div>

                  <button
                    onClick={handlePlaceOrder}
                    className={`w-full py-2.5 font-bold font-mono rounded text-sm transition-all shadow-lg \${
                      orderSide === 'BUY'
                        ? 'bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border border-cyber-green/40 shadow-green hover:shadow-[0_0_15px_rgba(34,197,94,0.25)]'
                        : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/40 hover:shadow-[0_0_15px_rgba(239,68,68,0.25)]'
                    }`}
                  >
                    PLACE {orderSide} {orderType} ORDER
                  </button>
                </div>
              </div>
            </div>
            
          </div>
        )}"""

# Replace the Tredo Cockpit JSX inside activeTab === 'Tredo'
pattern = r"\{\/\* \-\-\- FULL TRADING EXCHANGE COCKPIT \-\-\- \*\/\}\s+\{activeTab === 'Tredo' && \([\s\S]+?\}\)\;\s+\}\s+([^\n]*)\{activeTab === 'Journal'"
match = re.search(r"\{activeTab === 'Tredo' && \([\s\S]+?\}\)\;\s+\}", content)

if match:
    # Perform exact drop-in replacement
    start_idx, end_idx = match.span()
    new_content = content[:start_idx] + upgraded_tredo_jsx + content[end_idx:]
    with open(filepath, "w") as f:
        f.write(new_content)
    print("Success: App.tsx Tredo JSX block successfully upgraded!")
else:
    # Try alternate match if the first pattern did not match
    match_alt = re.search(r"\{activeTab === 'Tredo' && \([\s\S]+?\)\}", content)
    if match_alt:
        start_idx, end_idx = match_alt.span()
        new_content = content[:start_idx] + upgraded_tredo_jsx + content[end_idx:]
        with open(filepath, "w") as f:
            f.write(new_content)
        print("Success: App.tsx Tredo JSX block successfully upgraded (alt match)!")
    else:
        print("Error: Could not locate 'Tredo' tab JSX block in App.tsx!")
