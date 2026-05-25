import { useState, useEffect, useRef, useCallback } from 'react';
import { useAtom } from 'jotai';
import { createChart, CandlestickSeries, HistogramSeries, LineSeries, ColorType, CrosshairMode, LineStyle } from 'lightweight-charts';
import { cn, formatCurrency } from '../../lib/utils';
import { Badge } from '../ui/Badge';
import { TIMEFRAMES, ORDER_TYPES, ORDER_SIDES, AMOUNT_PRESETS, LEDGER_TABS } from '../../lib/constants';
import {
  watchlistAtom,
  basePricesAtom,
  selectedAssetAtom,
  portfolioValueAtom,
  cashBalanceAtom,
  openOrdersAtom,
  tradesHistoryAtom,
  priceHistoryAtom,
  systemAlertsAtom,
  serverLogsAtom,
  autoTradingStateAtom,
  performanceStatsAtom,
  serverActiveAtom,
  type OpenOrder,
  type TradeRecord,
  type Candlestick,
} from '../../atoms/state';

// ── Constants ─────────────────────────────────────────────────────────────

const DEFAULT_BASE_PRICES: Record<string, number> = {
  // Crypto
  'BTC-USD': 77430.0,
  'ETH-USD': 3450.0,
  'SOL-USD': 142.5,
  'ADA-USD': 0.58,
  'XRP-USD': 1.15,
  'DOT-USD': 6.20,
  'DOGE-USD': 0.16,
  'LTC-USD': 84.50,
  'LINK-USD': 15.20,
  'AVAX-USD': 28.40,
  'TRX-USD': 0.12,
  'SHIB-USD': 0.000018,
  'TON-USD': 5.50,
  'SUI-USD': 1.85,
  'NEAR-USD': 5.20,
  // US Stocks
  'AAPL': 185.20,
  'TSLA': 178.50,
  'MSFT': 415.60,
  'NVDA': 910.30,
  'AMZN': 182.40,
  'GOOG': 172.80,
  'META': 485.40,
  'AMD': 164.20,
  'NFLX': 610.50,
  'MS': 92.30,
  'JPM': 195.40,
  'V': 272.50,
  'DIS': 112.40,
  'WMT': 60.20,
  'COST': 725.60,
  // Indian Stocks
  'NSE:RELIANCE': 2450.0,
  'NSE:TCS': 3850.0,
  'NSE:HDFCBANK': 1520.0,
  'NSE:INFY': 1430.0,
  'NSE:ICICIBANK': 1120.0,
  'NSE:SBIN': 740.0,
  'NSE:BHARTIALRT': 1210.0,
  'NSE:ITC': 430.0,
  'NSE:LTIM': 4850.0,
  'NSE:LT': 3520.0,
  'NSE:HINDUNILVR': 2240.0,
  'NSE:SUNPHARMA': 1540.0,
  'NSE:KOTAKBANK': 1720.0,
  'NSE:AXISBANK': 1060.0,
  'NSE:TATASTEEL': 145.0,
  // Commodities
  'XAU-USD': 2352.0,
  'XAG-USD': 28.40,
  'USOIL': 78.50,
  'NGAS': 2.45
};

// ── Type Definitions ──────────────────────────────────────────────────────

interface DrawingLine {
  id: string;
  type: 'SUPPORT' | 'RESISTANCE';
  price: number;
  color: string;
}

type LedgerTab = 'OPEN' | 'HISTORY' | 'ASSETS' | 'SWARM';

// ── Main Tredo Module ─────────────────────────────────────────────────────

export function TredoModule() {
  // ── Shared atoms ──────────────────────────────────────────────────────────
  const [watchlist, setWatchlist] = useAtom(watchlistAtom);
  const [basePrices, setBasePrices] = useAtom(basePricesAtom);
  const [selectedAsset, setSelectedAsset] = useAtom(selectedAssetAtom);
  const [portfolioVal, setPortfolioVal] = useAtom(portfolioValueAtom);
  const [cash, setCash] = useAtom(cashBalanceAtom);
  const [openOrders, setOpenOrders] = useAtom(openOrdersAtom);
  const [tradesHistory, setTradesHistory] = useAtom(tradesHistoryAtom);
  const [priceHistory, setPriceHistory] = useAtom(priceHistoryAtom);
  const [alerts, setAlerts] = useAtom(systemAlertsAtom);
  const [, setLogs] = useAtom(serverLogsAtom);
  const [autoTradingState, setAutoTradingState] = useAtom(autoTradingStateAtom);
  const [perfStats, setPerfStats] = useAtom(performanceStatsAtom);
  const [serverActive] = useAtom(serverActiveAtom);

  // ── Local UI State ────────────────────────────────────────────────────────
  const [currentPrice, setCurrentPrice] = useState(DEFAULT_BASE_PRICES[selectedAsset] || 100);
  const [orderType, setOrderType] = useState<'LIMIT' | 'MARKET'>('LIMIT');
  const [limitPriceInput, setLimitPriceInput] = useState(currentPrice.toString());
  const [amountInput, setAmountInput] = useState('0.1');
  const [orderSide, setOrderSide] = useState<'BUY' | 'SELL'>('BUY');
  const [bottomTab, setBottomTab] = useState<LedgerTab>('OPEN');
  const [selectedTimeframe, setSelectedTimeframe] = useState<'1m' | '5m' | '15m' | '1h' | '1d'>('5m');
  const [activeIndicators, setActiveIndicators] = useState<Record<string, boolean>>({
    SMA: false, EMA: false, BB: false, VWAP: false,
  });
  const [drawingLines, setDrawingLines] = useState<DrawingLine[]>([]);
  const [isAddingAsset, setIsAddingAsset] = useState(false);
  const [newAssetSymbol, setNewAssetSymbol] = useState('');
  const [newAssetPrice, setNewAssetPrice] = useState('');
  const [watchlistCollapsed, setWatchlistCollapsed] = useState(false);
  const [open24hPrices, setOpen24hPrices] = useState<Record<string, number>>({});
  const [flashTickers, setFlashTickers] = useState<Record<string, 'up' | 'down' | null>>({});
  const [indicatorPeriods, setIndicatorPeriods] = useState({ SMA: 9, EMA: 12, BB: 20 });
  const [showIndicatorSettings, setShowIndicatorSettings] = useState<string | null>(null);
  const [chartType, setChartType] = useState<'LOCAL' | 'TRADINGVIEW'>('TRADINGVIEW');

  const getTradingViewSymbol = (asset: string) => {
    const upper = asset.toUpperCase().trim();
    
    // 1. COMMODITIES
    if (upper.includes('XAU') || upper.includes('GOLD')) return 'OANDA:XAUUSD';
    if (upper.includes('XAG') || upper.includes('SILVER')) return 'OANDA:XAGUSD';
    if (upper.includes('USOIL') || upper.includes('WTI') || upper.includes('CRUDE')) return 'TVC:USOIL';
    if (upper.includes('NGAS') || upper.includes('NATGAS')) return 'TVC:NGAS';
    
    // 2. CRYPTO
    const isCrypto = 
      upper.endsWith('-USD') || 
      upper.endsWith('-USDT') || 
      ['BTC', 'ETH', 'SOL', 'ADA', 'XRP', 'DOT', 'DOGE', 'LTC', 'LINK'].some(c => upper.startsWith(c));
    
    if (isCrypto) {
      const cleanCrypto = upper.replace('-USD', '').replace('-USDT', '').replace('/', '');
      return `BINANCE:${cleanCrypto}USDT`;
    }
    
    // 3. STOCKS & INDEXES
    const nasdaqStocks = ['AAPL', 'TSLA', 'MSFT', 'AMZN', 'GOOG', 'NVDA', 'NFLX', 'META', 'AMD', 'INTC'];
    const cleanStock = upper.replace('-USD', '').replace('-USDT', '');
    if (nasdaqStocks.includes(cleanStock)) {
      return `NASDAQ:${cleanStock}`;
    }
    
    return cleanStock;
  };

  // ── Refs ──────────────────────────────────────────────────────────────────

  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<any>(null);
  const candleSeriesRef = useRef<any>(null);
  const volSeriesRef = useRef<any>(null);
  const smaSeriesRef = useRef<any>(null);
  const emaSeriesRef = useRef<any>(null);
  const bbUpperRef = useRef<any>(null);
  const bbLowerRef = useRef<any>(null);
  const vwapSeriesRef = useRef<any>(null);
  const priceLinesRef = useRef<any[]>([]);

  // ── Indicator period edit handler ───────────────────────────────────────────
  const handleIndicatorPeriodChange = useCallback((indicator: string, value: number) => {
    setIndicatorPeriods((prev) => ({ ...prev, [indicator]: Math.max(1, value) }));
  }, []);

  // ── Sync defaults on symbol change ────────────────────────────────────────
  useEffect(() => {
    const base = basePrices[selectedAsset] || DEFAULT_BASE_PRICES[selectedAsset] || 100;
    setCurrentPrice(base);
    setLimitPriceInput(base.toString());
  }, [selectedAsset, basePrices]);

  // ── Initialize price history ─────────────────────────────────────────────
  useEffect(() => {
    setPriceHistory((prev) => {
      if (prev[selectedAsset] && prev[selectedAsset].length > 0) return prev;
      const startPrice = basePrices[selectedAsset] || DEFAULT_BASE_PRICES[selectedAsset] || 100;
      const candles: Candlestick[] = [];
      let lastClose = startPrice - 100;
      for (let i = 0; i < 40; i++) {
        const open = lastClose;
        const drift = (Math.random() - 0.48) * (startPrice * 0.003);
        const close = open + drift;
        const high = Math.max(open, close) + Math.random() * (startPrice * 0.001);
        const low = Math.min(open, close) - Math.random() * (startPrice * 0.001);
        candles.push({
          time: Date.now() - (40 - i) * 10000,
          open, high, low, close,
          volume: Math.random() * 5 + 0.5,
        });
        lastClose = close;
      }
      return { ...prev, [selectedAsset]: candles };
    });

    // Open 24h prices for change calc
    setOpen24hPrices((prev) => {
      if (!prev[selectedAsset]) {
        return { ...prev, [selectedAsset]: (basePrices[selectedAsset] || DEFAULT_BASE_PRICES[selectedAsset] || 100) * 0.99 };
      }
      return prev;
    });
  }, [selectedAsset, basePrices, setPriceHistory]);

  // ── Real-time price simulator ──────────────────────────────────────────
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentPrice((prevPrice) => {
        const base = basePrices[selectedAsset] || DEFAULT_BASE_PRICES[selectedAsset] || 100;
        const volatility = base * 0.0006;
        const change = (Math.random() - 0.49) * volatility;
        const nextPrice = Math.max(0.01, prevPrice + change);

        // Update latest candle
        setPriceHistory((history) => {
          const candles = [...(history[selectedAsset] || [])];
          if (candles.length > 0) {
            const lastCandle = { ...candles[candles.length - 1] };
            lastCandle.close = nextPrice;
            if (nextPrice > lastCandle.high) lastCandle.high = nextPrice;
            if (nextPrice < lastCandle.low) lastCandle.low = nextPrice;
            candles[candles.length - 1] = lastCandle;
          }
          return { ...history, [selectedAsset]: candles };
        });

        // Recent trades feed
        setTradesHistory((prevTrades) => {
          const tradeSize = Math.random() * 2 + 0.01;
          const nextTrade: TradeRecord = {
            id: `trade_${Date.now()}_${Math.random().toString(36).substring(2, 10)}`,
            symbol: selectedAsset,
            side: Math.random() > 0.5 ? 'BUY' as const : 'SELL' as const,
            price: nextPrice,
            amount: Number(tradeSize.toFixed(4)),
            timestamp: Date.now(),
          };
          return [nextTrade, ...prevTrades.slice(0, 19)];
        });

        // Flash tickers for price change — clear previous tick flash, then set current
        // (React 18 batches both into one render, so only the final state is visible)
        setFlashTickers((prev) => ({ ...prev, [selectedAsset]: null }));
        setFlashTickers((prev) => ({
          ...prev,
          [selectedAsset]: nextPrice > prevPrice ? 'up' : nextPrice < prevPrice ? 'down' : null,
        }));

        // Match limit orders
        setOpenOrders((orders) => {
          const remaining: OpenOrder[] = [];
          orders.forEach((order) => {
            if (order.symbol !== selectedAsset) { remaining.push(order); return; }
            const isFilled =
              (order.side === 'BUY' && nextPrice <= order.price) ||
              (order.side === 'SELL' && nextPrice >= order.price);
            if (isFilled) {
              const totalCost = order.price * order.amount;
              if (order.side === 'BUY') setPortfolioVal((v) => v + totalCost * 0.02);
              else { setCash((c) => c + totalCost); setPortfolioVal((v) => v + totalCost * 0.01); }
              setLogs((prev) => [`[INFO] Limit Order FILLED: ${order.side} ${order.amount} ${order.symbol} at ${order.price.toFixed(2)}`, ...prev]);
              setAlerts((prev) => [{
                alertId: `F-${Date.now()}-${Math.random().toString(36).substring(2, 10)}`,
                source: 'ExecutionEngine',
                severity: 'Info',
                message: `Matched ${order.side} order for ${order.amount} ${order.symbol} at $${order.price.toFixed(2)}`,
                timestamp: Date.now(),
              }, ...prev]);
            } else remaining.push(order);
          });
          return remaining;
        });

        return nextPrice;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [selectedAsset, basePrices, setPriceHistory, setTradesHistory, setOpenOrders, setLogs, setAlerts, setCash, setPortfolioVal]);

  // ── Professional TradingView lightweight-charts Engine ─────────────────────
  // Init chart once on mount
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: '#64748b',
        fontFamily: "'JetBrains Mono', 'Courier New', monospace",
        fontSize: 10,
      },
      grid: {
        vertLines: { color: 'rgba(255,255,255,0.03)', style: LineStyle.Dotted },
        horzLines: { color: 'rgba(255,255,255,0.04)', style: LineStyle.Dotted },
      },
      crosshair: {
        mode: CrosshairMode.Normal,
        vertLine: { color: 'rgba(255,255,255,0.2)', style: LineStyle.Dashed, labelBackgroundColor: '#1e293b' },
        horzLine: { color: 'rgba(255,255,255,0.2)', style: LineStyle.Dashed, labelBackgroundColor: '#1e293b' },
      },
      rightPriceScale: {
        borderColor: 'rgba(255,255,255,0.06)',
        textColor: '#64748b',
        scaleMargins: { top: 0.08, bottom: 0.25 },
      },
      timeScale: {
        borderColor: 'rgba(255,255,255,0.06)',
        timeVisible: true,
        secondsVisible: false,
        tickMarkFormatter: (time: any) => {
          const d = new Date(time * 1000);
          return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        },
      },
      handleScroll: { vertTouchDrag: true },
      handleScale: { axisPressedMouseMove: true },
    });

    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderUpColor: '#22c55e',
      borderDownColor: '#ef4444',
      wickUpColor: '#22c55e',
      wickDownColor: '#ef4444',
    });

    const volSeries = chart.addSeries(HistogramSeries, {
      color: '#22c55e',
      priceFormat: { type: 'volume' },
      priceScaleId: 'vol',
    });
    chart.priceScale('vol').applyOptions({
      scaleMargins: { top: 0.8, bottom: 0 },
    });

    chartInstanceRef.current = chart;
    candleSeriesRef.current = candleSeries;
    volSeriesRef.current = volSeries;

    const resizeObserver = new ResizeObserver(() => {
      if (chartContainerRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
          height: chartContainerRef.current.clientHeight,
        });
      }
    });
    resizeObserver.observe(chartContainerRef.current);

    return () => {
      resizeObserver.disconnect();
      chart.remove();
      chartInstanceRef.current = null;
    };
  }, []);

  // Feed price history into the chart whenever it updates
  useEffect(() => {
    const chart = chartInstanceRef.current;
    const candleSeries = candleSeriesRef.current;
    const volSeries = volSeriesRef.current;
    if (!chart || !candleSeries || !volSeries) return;

    const rawCandles = priceHistory[selectedAsset] || [];
    if (rawCandles.length === 0) return;

    const sortedCandles = [...rawCandles].sort((a, b) => a.time - b.time);

    // Deduplicate by time (keep last)
    const seen = new Map<number, Candlestick>();
    sortedCandles.forEach(c => seen.set(Math.floor(c.time / 1000), c));
    const candles = Array.from(seen.entries())
      .sort((a, b) => a[0] - b[0])
      .map(([t, c]) => ({ time: t as any, open: c.open, high: c.high, low: c.low, close: c.close, volume: c.volume }));

    try {
      candleSeries.setData(candles);
      volSeries.setData(candles.map((c: any) => ({
        time: c.time,
        value: c.volume,
        color: c.close >= c.open ? 'rgba(34,197,94,0.4)' : 'rgba(239,68,68,0.4)',
      })));
    } catch (e) {
      console.error("CHART SET DATA ERROR:", e);
    }

    // --- Remove old indicator series ---
    if (smaSeriesRef.current) { try { chart.removeSeries(smaSeriesRef.current); } catch { /* */ } smaSeriesRef.current = null; }
    if (emaSeriesRef.current) { try { chart.removeSeries(emaSeriesRef.current); } catch { /* */ } emaSeriesRef.current = null; }
    if (bbUpperRef.current) { try { chart.removeSeries(bbUpperRef.current); } catch { /* */ } bbUpperRef.current = null; }
    if (bbLowerRef.current) { try { chart.removeSeries(bbLowerRef.current); } catch { /* */ } bbLowerRef.current = null; }
    if (vwapSeriesRef.current) { try { chart.removeSeries(vwapSeriesRef.current); } catch { /* */ } vwapSeriesRef.current = null; }

    const closes = candles.map((c: any) => c.close);

    // SMA
    if (activeIndicators.SMA) {
      const period = indicatorPeriods.SMA;
      const smaSeries = chart.addSeries(LineSeries, { color: '#00bcd4', lineWidth: 1, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
      const smaData: any[] = [];
      candles.forEach((c: any, idx: number) => {
        if (idx >= period - 1) {
          const slice = closes.slice(idx - period + 1, idx + 1);
          smaData.push({ time: c.time, value: slice.reduce((a: number, b: number) => a + b, 0) / period });
        }
      });
      try { smaSeries.setData(smaData); } catch { /* */ }
      smaSeriesRef.current = smaSeries;
    }

    // EMA
    if (activeIndicators.EMA) {
      const period = indicatorPeriods.EMA;
      const k = 2 / (period + 1);
      const emaSeries = chart.addSeries(LineSeries, { color: '#ff007f', lineWidth: 1, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
      const emaData: any[] = [];
      let prevEma = closes[0] || 0;
      candles.forEach((c: any, idx: number) => {
        const ema = idx === 0 ? prevEma : c.close * k + prevEma * (1 - k);
        prevEma = ema;
        emaData.push({ time: c.time, value: ema });
      });
      try { emaSeries.setData(emaData); } catch { /* */ }
      emaSeriesRef.current = emaSeries;
    }

    // Bollinger Bands
    if (activeIndicators.BB) {
      const period = indicatorPeriods.BB;
      const upperSeries = chart.addSeries(LineSeries, { color: 'rgba(251,191,36,0.7)', lineWidth: 1, priceLineVisible: false, lastValueVisible: false, crosshairMarkerVisible: false });
      const lowerSeries = chart.addSeries(LineSeries, { color: 'rgba(251,191,36,0.7)', lineWidth: 1, priceLineVisible: false, lastValueVisible: false, crosshairMarkerVisible: false });
      const upperData: any[] = [], lowerData: any[] = [];
      candles.forEach((c: any, idx: number) => {
        if (idx >= period - 1) {
          const slice = closes.slice(idx - period + 1, idx + 1);
          const mean = slice.reduce((a: number, b: number) => a + b, 0) / period;
          const std = Math.sqrt(slice.reduce((a: number, b: number) => a + Math.pow(b - mean, 2), 0) / period);
          upperData.push({ time: c.time, value: mean + 2 * std });
          lowerData.push({ time: c.time, value: mean - 2 * std });
        }
      });
      try { upperSeries.setData(upperData); lowerSeries.setData(lowerData); } catch { /* */ }
      bbUpperRef.current = upperSeries;
      bbLowerRef.current = lowerSeries;
    }

    // VWAP
    if (activeIndicators.VWAP) {
      const vwapSeries = chart.addSeries(LineSeries, { color: 'rgba(255,255,255,0.7)', lineWidth: 2, lineStyle: LineStyle.Dashed, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
      let cumPV = 0, cumVol = 0;
      const vwapData: any[] = [];
      candles.forEach((c: any) => {
        const tp = (c.high + c.low + c.close) / 3;
        cumPV += tp * c.volume;
        cumVol += c.volume;
        vwapData.push({ time: c.time, value: cumPV / cumVol });
      });
      try { vwapSeries.setData(vwapData); } catch { /* */ }
      vwapSeriesRef.current = vwapSeries;
    }

    // Remove old price lines and redraw support/resistance
    priceLinesRef.current.forEach(pl => { try { candleSeries.removePriceLine(pl); } catch { /* */ } });
    priceLinesRef.current = [];
    drawingLines.forEach(line => {
      const pl = candleSeries.createPriceLine({
        price: line.price,
        color: line.color,
        lineWidth: 1,
        lineStyle: LineStyle.Dashed,
        axisLabelVisible: true,
        title: line.type,
      });
      priceLinesRef.current.push(pl);
    });

    // Live price line
    const livePriceLine = candleSeries.createPriceLine({
      price: currentPrice,
      color: 'rgba(251,191,36,0.8)',
      lineWidth: 1,
      lineStyle: LineStyle.Dotted,
      axisLabelVisible: true,
      title: 'LIVE',
    });
    priceLinesRef.current.push(livePriceLine);

  }, [priceHistory, selectedAsset, selectedTimeframe, currentPrice, activeIndicators, indicatorPeriods, drawingLines]);

  // ── L2 Data Generator ─────────────────────────────────────────────────────
  const generateL2 = useCallback(() => {
    const base = basePrices[selectedAsset] || DEFAULT_BASE_PRICES[selectedAsset] || 100;
    const step = base * 0.0003;
    const bids = [];
    const asks = [];
    for (let i = 1; i <= 6; i++) {
      asks.push({ price: currentPrice + i * step, amount: Math.random() * 3.5 + 0.1 });
      bids.push({ price: currentPrice - i * step, amount: Math.random() * 3.5 + 0.1 });
    }
    return { bids, asks: asks.reverse() };
  }, [currentPrice, selectedAsset, basePrices]);

  const l2Data = generateL2();

  // ── Order Execution ───────────────────────────────────────────────────────
  const handlePlaceOrder = useCallback(() => {
    const price = orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice;
    const amount = Number(amountInput);
    if (isNaN(price) || price <= 0 || isNaN(amount) || amount <= 0) return;

    const totalCost = price * amount;
    if (orderSide === 'BUY' && totalCost > cash) {
      alert('Insufficient cash balance to place this order!');
      return;
    }

    if (orderType === 'LIMIT') {
      const nextOrder: OpenOrder = {
        id: `order_${Date.now()}_${Math.random().toString(36).substring(2, 10)}`,
        symbol: selectedAsset,
        side: orderSide,
        type: 'LIMIT',
        price,
        amount,
        filled: 0,
        timestamp: Date.now(),
      };
      if (orderSide === 'BUY') setCash((c) => c - totalCost);
      setOpenOrders((prev) => [nextOrder, ...prev]);
      setLogs((prev) => [`[INFO] Limit Order Placed: ${orderSide} ${amount} ${selectedAsset} at ${price.toFixed(2)}`, ...prev]);
      setAlerts((prev) => [{
        alertId: `L-${Date.now()}-${Math.random().toString(36).substring(2, 10)}`,
        source: 'ExecutionEngine',
        severity: 'Info',
        message: `Placed Limit ${orderSide} order for ${amount} ${selectedAsset} at $${price.toFixed(2)}`,
        timestamp: Date.now(),
      }, ...prev]);
    } else {
      if (orderSide === 'BUY') { setCash((c) => c - totalCost); setPortfolioVal((v) => v + totalCost * 0.02); }
      else { setCash((c) => c + totalCost); setPortfolioVal((v) => v + totalCost * 0.01); }
      setLogs((prev) => [`[INFO] Market Order Executed: ${orderSide} ${amount} ${selectedAsset} at ${currentPrice.toFixed(2)}`, ...prev]);
      setAlerts((prev) => [{
        alertId: `M-${Date.now()}-${Math.random().toString(36).substring(2, 10)}`,
        source: 'ExecutionEngine',
        severity: 'Info',
        message: `Executed Market ${orderSide} order for ${amount} ${selectedAsset} at $${currentPrice.toFixed(2)}`,
        timestamp: Date.now(),
      }, ...prev]);
    }
  }, [orderType, limitPriceInput, currentPrice, amountInput, orderSide, cash, selectedAsset, setCash, setOpenOrders, setPortfolioVal, setLogs, setAlerts]);

  const handleCancelOrder = useCallback((orderId: string) => {
    setOpenOrders((orders) => {
      const order = orders.find((o) => o.id === orderId);
      if (order && order.side === 'BUY') setCash((c) => c + order.price * order.amount);
      setLogs((prev) => [`[INFO] Limit Order Cancelled: ${orderId}`, ...prev]);
      return orders.filter((o) => o.id !== orderId);
    });
  }, [setOpenOrders, setCash, setLogs]);

  // ── Auto Trading Controls ─────────────────────────────────────────────────
  const handleStartAutoTrade = useCallback(async () => {
    try {
      await fetch('/api/autotrade/start', { method: 'POST' });
      setLogs((prev) => [`[AutoTrading] Autonomous trading loop STARTED`, ...prev]);
      const res = await fetch('/api/autotrade/status');
      const data = await res.json();
      if (data.status === 'success') setAutoTradingState(data.trading_state);
    } catch { /* ignore */ }
  }, [setLogs, setAutoTradingState]);

  const handleStopAutoTrade = useCallback(async () => {
    try {
      await fetch('/api/autotrade/stop', { method: 'POST' });
      setLogs((prev) => [`[AutoTrading] Autonomous trading loop STOPPED`, ...prev]);
      const res = await fetch('/api/autotrade/status');
      const data = await res.json();
      if (data.status === 'success') setAutoTradingState(data.trading_state);
    } catch { /* ignore */ }
  }, [setLogs, setAutoTradingState]);

  // ── Fetch auto-trading state periodically ─────────────────────────────────
  useEffect(() => {
    const fetchState = async () => {
      try {
        const res = await fetch('/api/autotrade/status');
        const data = await res.json();
        if (data.status === 'success') setAutoTradingState(data.trading_state);
      } catch { /* skip */ }
      try {
        const res = await fetch('/api/journal/stats');
        const data = await res.json();
        if (data.status === 'success') setPerfStats(data.stats);
      } catch { /* skip */ }
    };
    fetchState();
    const interval = setInterval(fetchState, 10000);
    return () => clearInterval(interval);
  }, [setAutoTradingState, setPerfStats]);

  // ── Add Asset ─────────────────────────────────────────────────────────────
  const handleAddAsset = useCallback(() => {
    const symbol = newAssetSymbol.trim().toUpperCase();
    const priceVal = parseFloat(newAssetPrice);
    if (!symbol || isNaN(priceVal) || priceVal <= 0) {
      setLogs((prev) => [`[ERROR] Whitelist Registration Failed: Invalid Symbol or Base Price`, ...prev]);
      return;
    }
    if (watchlist.includes(symbol)) {
      setLogs((prev) => [`[WARNING] Symbol ${symbol} already exists`, ...prev]);
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
    setLogs((prev) => [`[INFO] Registered Symbol: ${symbol} at $${priceVal.toFixed(2)}`, ...prev]);
    setNewAssetSymbol('');
    setNewAssetPrice('');
    setIsAddingAsset(false);
  }, [newAssetSymbol, newAssetPrice, watchlist, basePrices, setWatchlist, setBasePrices, setSelectedAsset, setLogs]);

  // ── Draw Support/Resistance ───────────────────────────────────────────────
  const addSupportLine = useCallback(() => {
    setDrawingLines((prev) => [...prev, {
      id: `line-${Date.now()}-${Math.random().toString(36).substring(2, 10)}`,
      type: 'SUPPORT',
      price: currentPrice,
      color: '#00e676',
    }]);
    setLogs((prev) => [`[INFO] Placed Support Line at $${currentPrice.toFixed(2)}`, ...prev]);
  }, [currentPrice, setLogs]);

  const addResistanceLine = useCallback(() => {
    setDrawingLines((prev) => [...prev, {
      id: `line-${Date.now()}-${Math.random().toString(36).substring(2, 10)}`,
      type: 'RESISTANCE',
      price: currentPrice,
      color: '#ef4444',
    }]);
    setLogs((prev) => [`[INFO] Placed Resistance Line at $${currentPrice.toFixed(2)}`, ...prev]);
  }, [currentPrice, setLogs]);

  const clearDrawingLines = useCallback(() => {
    setDrawingLines([]);
    setLogs((prev) => [`[INFO] Cleared all drawing levels from chart`, ...prev]);
  }, [setLogs]);

  // ── Format helpers ─────────────────────────────────────────────────────
  const formatPrice = (v: number) => v.toFixed(2);

  // ── Compute spread ─────────────────────────────────────────────────────────
  const bestAsk = l2Data.asks.length > 0 ? l2Data.asks[l2Data.asks.length - 1].price : 0;
  const bestBid = l2Data.bids.length > 0 ? l2Data.bids[0].price : 0;
  const spread = bestAsk - bestBid;
  const spreadBps = bestBid > 0 ? (spread / bestBid) * 10000 : 0;

  // ── Max amounts for depth visual ──────────────────────────────────────────
  const maxAskAmount = Math.max(...l2Data.asks.map((a) => a.amount), 1);
  const maxBidAmount = Math.max(...l2Data.bids.map((b) => b.amount), 1);

  // ── Render ────────────────────────────────────────────────────────────────
  return (
    <div className="relative h-full overflow-hidden">
      {!serverActive && (
        <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-slate-950/85 backdrop-blur-md transition-all duration-500 border border-rose-500/20 rounded-2xl">
          <div className="w-16 h-16 rounded-full bg-rose-950/30 border border-rose-500/30 flex items-center justify-center text-rose-500 text-3xl shadow-[0_0_15px_rgba(244,63,94,0.15)] mb-4 animate-pulse">
            ⏻
          </div>
          <h2 className="text-xl font-bold font-mono text-rose-400 tracking-widest mb-1.5 uppercase">
            Sethu Bridge Severed
          </h2>
          <p className="text-[10px] text-slate-500 font-mono tracking-wider max-w-xs text-center leading-relaxed font-semibold">
            The core orchestrator is offline. Activate the server power toggle in the cockpit control bar to reconnect telemetry feeds.
          </p>
        </div>
      )}
      <div className={cn("grid grid-cols-12 gap-6 h-full overflow-hidden", !serverActive && "opacity-25 pointer-events-none transition-opacity duration-500")} role="region" aria-label="Trading module">
      {/* COLUMN 1: Watchlist & Recent Trades (3 cols, collapsible) */}
      <div className={cn(watchlistCollapsed ? 'col-span-1' : 'col-span-3', 'flex flex-col gap-6 h-full overflow-hidden transition-all duration-300')}>
        {/* Watchlist Panel */}
        <div className="glass-panel rounded-xl p-4 flex flex-col h-[48%] overflow-hidden relative">
          <div className="flex justify-between items-center mb-3">
            {!watchlistCollapsed && (
              <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400" id="watchlist-heading">WATCHLIST</h3>
            )}
            <div className="flex items-center ml-auto gap-1.5">
              {!watchlistCollapsed && (
                <button
                  onClick={() => setIsAddingAsset(!isAddingAsset)}
                  className="text-[10px] font-mono font-bold text-cyber-purple hover:text-white px-2 py-0.5 bg-cyber-purple/10 hover:bg-cyber-purple/35 rounded border border-cyber-purple/30 transition-all"
                  aria-label="Add asset to watchlist"
                >
                  ✙ Add
                </button>
              )}
              <button
                onClick={() => setWatchlistCollapsed(!watchlistCollapsed)}
                className="btn-icon text-[10px]"
                aria-label={watchlistCollapsed ? 'Expand watchlist' : 'Collapse watchlist'}
              >
                {watchlistCollapsed ? '▶' : '◀'}
              </button>
            </div>
          </div>

          {/* Add Asset Form */}
          {isAddingAsset && !watchlistCollapsed && (
            <div className="absolute top-12 left-3 right-3 bg-cyber-dark border border-cyber-purple/50 rounded-xl p-3.5 shadow-xl z-20 font-mono text-xs animate-slide-up">
              <h4 className="text-[10px] font-bold text-cyber-purple uppercase tracking-wider mb-2.5">Register Symbol</h4>
              <div className="space-y-2.5">
                <input
                  type="text"
                  value={newAssetSymbol}
                  onChange={(e) => setNewAssetSymbol(e.target.value.toUpperCase().replace(/\s/g, ''))}
                  placeholder="e.g. TSLA or DOGE-USD"
                  className="input-cyber"
                  aria-label="Symbol ticker"
                />
                <input
                  type="number"
                  value={newAssetPrice}
                  onChange={(e) => setNewAssetPrice(e.target.value)}
                  placeholder="e.g. 185.50"
                  className="input-cyber"
                  aria-label="Base price"
                />
                <div className="flex gap-2 pt-1">
                  <button onClick={handleAddAsset} className="btn-primary flex-1">REGISTER</button>
                  <button onClick={() => { setNewAssetSymbol(''); setNewAssetPrice(''); setIsAddingAsset(false); }} className="btn-secondary flex-1">CANCEL</button>
                </div>
              </div>
            </div>
          )}

          <div className="flex-1 overflow-y-auto space-y-1.5 pr-1 scrollbar-cyber" role="list" aria-labelledby="watchlist-heading">
            {watchlist.map((asset) => {
              const price = basePrices[asset] || DEFAULT_BASE_PRICES[asset] || 100;
              const open24h = open24hPrices[asset] || price;
              const pct = open24h > 0 ? ((price - open24h) / open24h) * 100 : 0;
              const flash = flashTickers[asset];

              return (
                <button
                  key={asset}
                  onClick={() => setSelectedAsset(asset)}
                  className={cn(
                    'w-full text-left rounded-lg font-mono text-xs flex transition-all duration-300 relative overflow-hidden',
                    watchlistCollapsed ? 'px-1 py-3 justify-center' : 'px-3 py-2.5 justify-between items-center',
                    selectedAsset === asset
                      ? 'bg-cyber-purple/25 border border-cyber-purple/40 text-cyber-purple shadow-[0_0_12px_rgba(157,78,221,0.15)]'
                      : 'bg-cyber-panel/20 border border-transparent text-slate-400 hover:text-slate-200 hover:bg-cyber-panel/40',
                    flash === 'up' && 'bg-green-500/10 border-green-500/30',
                    flash === 'down' && 'bg-red-500/10 border-red-500/30'
                  )}
                >
                  {watchlistCollapsed ? (
                    <span className="font-bold text-[10px] uppercase">{asset.split('-')[0]}</span>
                  ) : (
                    <>
                      <div className="flex items-center gap-2">
                        <span className="font-bold">{asset}</span>
                        <svg className="w-10 h-5 opacity-70" viewBox="0 0 50 20">
                          <polyline
                            fill="none"
                            stroke={pct >= 0 ? '#22c55e' : '#ef4444'}
                            strokeWidth="1.2"
                            points={flash === 'up' ? "0,15 10,12 20,10 30,13 40,8 50,5" : flash === 'down' ? "0,5 10,8 20,12 30,10 40,14 50,18" : "0,12 10,11 20,13 30,12 40,11 50,12"}
                          />
                        </svg>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-slate-100">${formatPrice(price)}</span>
                        <Badge variant={pct >= 0 ? 'success' : 'danger'}>{pct >= 0 ? '+' : ''}{pct.toFixed(2)}%</Badge>
                      </div>
                    </>
                  )}
                </button>
              );
            })}
          </div>
        </div>

        {/* Recent Trades Feed */}
        <div className="glass-panel rounded-xl p-4 flex flex-col h-[52%] overflow-hidden">
          {!watchlistCollapsed && (
            <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">RECENT MARKET TRADES</h3>
          )}
          <div className="flex-grow overflow-hidden relative">
            {!watchlistCollapsed && (
              <div className="flex justify-between text-[10px] font-mono text-slate-500 border-b border-cyber-border/40 pb-1.5 mb-1">
                <span className="font-semibold tracking-wider">Price</span>
                <span className="font-semibold tracking-wider text-right">Amount</span>
                <span className="font-semibold tracking-wider text-right">Time</span>
              </div>
            )}
            <div className="h-full overflow-y-auto space-y-0.5 scrollbar-cyber">
              {tradesHistory.filter(t => t.symbol === selectedAsset).slice(0, 50).map((trade) => (
                <div key={trade.id} className="flex justify-between font-mono py-0.5 text-[10px] border-b border-cyber-border/5 hover:bg-white/5 transition-colors">
                  {watchlistCollapsed ? (
                    <span className={cn('font-bold', trade.side === 'BUY' ? 'text-cyber-green' : 'text-red-400')}>
                      ${formatPrice(trade.price)}
                    </span>
                  ) : (
                    <>
                      <span className={cn(trade.side === 'BUY' ? 'text-cyber-green font-semibold' : 'text-red-400 font-semibold')}>
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
              {tradesHistory.filter(t => t.symbol === selectedAsset).length === 0 && (
                <p className="text-[10px] text-slate-500 text-center py-4 font-mono">No trades yet</p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* COLUMN 2: Chart & Ledger */}
      <div className={cn(watchlistCollapsed ? 'col-span-8' : 'col-span-6', 'flex flex-col gap-6 h-full overflow-hidden transition-all duration-300')}>
        {/* Chart Panel */}
        <div className="flex-1 glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
          <div className="flex justify-between items-center border-b border-cyber-border/40 pb-3 mb-3">
            <div className="flex items-center gap-3">
              <h3 className="text-sm font-bold font-mono text-slate-200">{selectedAsset}</h3>
              <Badge variant="info" className="animate-pulse">Live</Badge>
            </div>
            <div className="flex items-center gap-3 text-xs font-mono">
              <span className="text-slate-400">Current:</span>
              <span className="px-2.5 py-1 bg-cyber-dark/80 rounded-lg border border-cyber-green/40 text-cyber-green font-bold shadow-[0_0_15px_rgba(0,230,118,0.15)] tabular-nums transition-all duration-300">
                ${formatPrice(currentPrice)}
              </span>
            </div>
          </div>

          {/* Chart Controls */}
          <div className="flex flex-wrap items-center justify-between gap-2 bg-cyber-dark/30 border border-cyber-border/20 rounded-lg p-2 mb-3 text-xs font-mono z-10">
            {/* Chart Type Toggle */}
            <div className="flex items-center gap-1 bg-cyber-panel/50 p-1 rounded border border-cyber-border/30">
              <span className="text-[10px] text-slate-500 font-bold px-1 uppercase">Chart:</span>
              <button
                onClick={() => setChartType('LOCAL')}
                className={cn(
                  'px-2 py-0.5 rounded text-[10px] font-bold transition-all',
                  chartType === 'LOCAL'
                    ? 'bg-cyber-purple/20 border border-cyber-purple/50 text-cyber-purple'
                    : 'text-slate-400 hover:text-slate-200'
                )}
              >
                Local
              </button>
              <button
                onClick={() => setChartType('TRADINGVIEW')}
                className={cn(
                  'px-2 py-0.5 rounded text-[10px] font-bold transition-all',
                  chartType === 'TRADINGVIEW'
                    ? 'bg-cyber-purple/20 border border-cyber-purple/50 text-cyber-purple'
                    : 'text-slate-400 hover:text-slate-200'
                )}
              >
                TradingView
              </button>
            </div>

            {/* Timeframes */}
            <div className="flex items-center gap-1 bg-cyber-panel/50 p-1 rounded border border-cyber-border/30">
              <span className="text-[10px] text-slate-500 font-bold px-1 uppercase">TF:</span>
              {TIMEFRAMES.map((tf) => (
                <button
                  key={tf}
                  onClick={() => setSelectedTimeframe(tf)}
                  className={cn(
                    'px-2 py-0.5 rounded text-[10px] font-bold transition-all',
                    selectedTimeframe === tf
                      ? 'bg-cyber-purple/20 border border-cyber-purple/50 text-cyber-purple'
                      : 'text-slate-400 hover:text-slate-200'
                  )}
                >
                  {tf}
                </button>
              ))}
            </div>

            {/* Indicators */}
            {chartType === 'LOCAL' && (
              <div className="flex items-center gap-1 bg-cyber-panel/50 p-1 rounded border border-cyber-border/30">
                <span className="text-[10px] text-slate-500 font-bold px-1 uppercase">Ind:</span>
                {Object.entries(activeIndicators).map(([ind, active]) => (
                  <div key={ind} className="relative inline-flex items-center gap-0.5">
                    <button
                      onClick={() => setActiveIndicators((prev) => ({ ...prev, [ind]: !prev[ind] }))}
                      className={cn(
                        'px-2 py-0.5 rounded text-[10px] font-bold transition-all',
                        active
                          ? ind === 'SMA' ? 'bg-cyan-500/25 border border-cyan-500/50 text-cyan-400'
                          : ind === 'EMA' ? 'bg-pink-500/25 border border-pink-500/50 text-pink-400'
                          : ind === 'BB' ? 'bg-amber-500/25 border border-amber-500/50 text-amber-400'
                          : 'bg-slate-100/20 border border-slate-100/50 text-slate-100'
                          : 'text-slate-400 hover:text-slate-200'
                      )}
                    >
                      {ind === 'SMA' ? `SMA (${indicatorPeriods.SMA})` : ind === 'EMA' ? `EMA (${indicatorPeriods.EMA})` : ind === 'BB' ? `BB (${indicatorPeriods.BB},2)` : 'VWAP'}
                    </button>
                    {ind !== 'VWAP' && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setShowIndicatorSettings(showIndicatorSettings === ind ? null : ind);
                        }}
                        className={cn(
                          'p-0.5 rounded text-[9px] transition-all',
                          showIndicatorSettings === ind ? 'text-cyber-purple' : 'text-slate-500 hover:text-slate-300'
                        )}
                        title={`Configure ${ind} period`}
                      >
                        ⚙
                      </button>
                    )}
                    {showIndicatorSettings === ind && (
                      <div className="absolute top-7 left-1/2 -translate-x-1/2 bg-cyber-dark border border-cyber-purple/40 rounded-lg p-2.5 z-40 space-y-1.5 shadow-xl font-mono text-[10px] min-w-[120px]">
                        <div className="flex items-center justify-between gap-2">
                          <span className="text-slate-400 text-[9px]">Period:</span>
                          <input
                            type="number"
                            min="1"
                            max="200"
                            className="w-14 bg-cyber-panel border border-cyber-border rounded text-center text-slate-200 font-bold py-0.5 text-[10px] focus:outline-none focus:border-cyber-purple"
                            value={indicatorPeriods[ind as keyof typeof indicatorPeriods]}
                            onChange={(e) => handleIndicatorPeriodChange(ind, Number(e.target.value))}
                          />
                        </div>
                        <button
                          onClick={() => setShowIndicatorSettings(null)}
                          className="w-full text-center bg-cyber-purple/20 hover:bg-cyber-purple/40 text-cyber-purple font-bold py-0.5 rounded text-[9px] transition-all"
                        >
                          Apply
                        </button>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* Drawing Tools */}
            {chartType === 'LOCAL' && (
              <div className="flex items-center gap-1">
                <button onClick={addSupportLine} className="px-2 py-0.5 bg-cyber-green/10 hover:bg-cyber-green/20 text-cyber-green border border-cyber-green/30 rounded text-[10px] font-bold transition-all">
                  + Support
                </button>
                <button onClick={addResistanceLine} className="px-2 py-0.5 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30 rounded text-[10px] font-bold transition-all">
                  + Resistance
                </button>
                {drawingLines.length > 0 && (
                  <button onClick={clearDrawingLines} className="btn-icon text-[10px]" title="Clear all lines">✕</button>
                )}
              </div>
            )}
          </div>

          {/* Canvas Chart Wrapper */}
          <div className="flex-1 relative bg-cyber-dark/60 border border-cyber-border/40 rounded-lg overflow-hidden min-h-[200px] shadow-inner">
            {chartType === 'LOCAL' ? (
              <>
                <div ref={chartContainerRef} className="absolute inset-0 w-full h-full" aria-label={`Candlestick chart for ${selectedAsset}`} />
                {(!priceHistory[selectedAsset] || priceHistory[selectedAsset].length === 0) && (
                  <div className="absolute inset-0 flex items-center justify-center">
                    <div className="flex flex-col items-center gap-2">
                      <span className="w-6 h-6 border-2 border-cyber-purple/50 border-t-cyber-purple rounded-full animate-spin" />
                      <span className="text-[10px] font-mono text-slate-500">Loading chart data...</span>
                    </div>
                  </div>
                )}
              </>
            ) : (
              <iframe
                src={`https://s.tradingview.com/widgetembed/?symbol=${getTradingViewSymbol(selectedAsset)}&interval=5&theme=dark&style=1&timezone=exchange&show_popup_button=1&popup_width=1000&popup_height=650&locale=en`}
                className="w-full h-full border-0 absolute inset-0 rounded-lg"
                allowFullScreen
                title="TradingView Advanced Chart"
              />
            )}
          </div>
        </div>

        {/* Bottom Ledger */}
        <div className="h-56 glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
          <div className="flex gap-3 border-b border-cyber-border/30 pb-2 mb-3">
            {LEDGER_TABS.map((tab) => (
              <button
                key={tab}
                onClick={() => setBottomTab(tab)}
                className={cn(
                  'text-xs font-bold font-mono tracking-wider px-3 py-1 rounded transition-colors',
                  bottomTab === tab
                    ? 'bg-cyber-purple/20 text-cyber-purple border border-cyber-purple/40'
                    : 'text-slate-400 hover:text-slate-200'
                )}
              >
                {tab === 'OPEN' ? 'OPEN ORDERS' : tab === 'HISTORY' ? 'SYSTEM ALERTS' : tab === 'ASSETS' ? 'LEDGER ASSETS' : 'AGENT SWARM'}
              </button>
            ))}
          </div>

          <div className="flex-1 overflow-y-auto text-xs font-mono scrollbar-cyber">
            {bottomTab === 'OPEN' && (
              openOrders.length === 0 ? (
                <p className="text-center py-6 text-slate-500">No active open limit orders.</p>
              ) : (
                <table className="data-table">
                  <thead>
                    <tr><th>Symbol</th><th>Side</th><th>Price</th><th>Amount</th><th>Total</th><th>Action</th></tr>
                  </thead>
                  <tbody>
                    {openOrders.map((order) => (
                      <tr key={order.id}>
                        <td className="text-slate-300 font-bold">{order.symbol}</td>
                        <td className={cn('font-bold', order.side === 'BUY' ? 'text-cyber-green' : 'text-red-400')}>{order.side}</td>
                        <td className="text-slate-300">${formatPrice(order.price)}</td>
                        <td className="text-slate-300">{order.amount}</td>
                        <td className="text-cyber-purple font-semibold">${(order.price * order.amount).toFixed(2)}</td>
                        <td><button onClick={() => handleCancelOrder(order.id)} className="btn-danger text-[10px] px-2 py-0.5">CANCEL</button></td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )
            )}
            {bottomTab === 'HISTORY' && (
              <div className="space-y-1">
                {alerts.map((alert) => (
                  <div key={alert.alertId} className="flex justify-between border-b border-cyber-border/10 py-1 text-[10px]">
                    <span className="text-slate-300 truncate mr-2">{alert.message}</span>
                    <span className="text-slate-500 shrink-0">{new Date(alert.timestamp).toLocaleTimeString()}</span>
                  </div>
                ))}
                {alerts.length === 0 && <p className="text-center py-6 text-slate-500">No alerts</p>}
              </div>
            )}
            {bottomTab === 'ASSETS' && (
              <div className="grid grid-cols-3 gap-4 p-2">
                <div className="stat-card">
                  <span className="text-[10px] text-slate-400">NET WORTH</span>
                  <p className="text-base font-bold text-cyber-green mt-1 tabular-nums">{formatCurrency(portfolioVal)}</p>
                </div>
                <div className="stat-card">
                  <span className="text-[10px] text-slate-400">LIQUID CASH</span>
                  <p className="text-base font-bold text-slate-200 mt-1 tabular-nums">{formatCurrency(cash)}</p>
                </div>
                <div className="stat-card">
                  <span className="text-[10px] text-slate-400">ENGINE</span>
                  <p className="text-xs font-bold text-cyber-purple mt-2">Sethu SwarmCoordinator</p>
                </div>
              </div>
            )}
            {bottomTab === 'SWARM' && (
              <div className="grid grid-cols-12 gap-4 h-full min-h-[140px] overflow-hidden">
                {/* Visual Agent Graph */}
                <div className="col-span-7 bg-cyber-dark/40 rounded-xl border border-cyber-border/40 p-2.5 flex flex-col justify-between overflow-hidden relative">
                  <div className="flex justify-between items-center mb-1 z-10">
                    <span className="text-[9px] font-bold text-cyber-purple font-mono uppercase tracking-wider flex items-center gap-1.5">
                      <span className="w-1.5 h-1.5 rounded-full bg-cyber-purple animate-ping" />
                      Hierarchical Swarm Consensus Topology
                    </span>
                    <span className="text-[8px] text-slate-500 font-mono">Consensus: 88.4% (nemetron:4b)</span>
                  </div>

                  {/* Nodes & Connections Grid */}
                  <div className="flex-1 flex items-center justify-center relative min-h-[90px]">
                    {/* Glowing Connections SVG background */}
                    <svg className="absolute inset-0 w-full h-full pointer-events-none" style={{ filter: 'drop-shadow(0 0 6px rgba(157,78,221,0.2))' }}>
                      <line x1="50%" y1="50%" x2="20%" y2="20%" stroke="#06b6d4" strokeWidth="1" strokeDasharray="3,3" />
                      <line x1="50%" y1="50%" x2="80%" y2="20%" stroke="#10b981" strokeWidth="1" strokeDasharray="3,3" />
                      <line x1="50%" y1="50%" x2="20%" y2="80%" stroke="#f59e0b" strokeWidth="1" strokeDasharray="3,3" />
                      <line x1="50%" y1="50%" x2="80%" y2="80%" stroke="#f43f5e" strokeWidth="1" strokeDasharray="3,3" />
                    </svg>

                    {/* Nethra Coordinator (Center) */}
                    <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col items-center z-10">
                      <div className="w-12 h-12 rounded-full bg-gradient-to-br from-cyber-purple to-pink-500 flex items-center justify-center font-bold text-white shadow-[0_0_20px_rgba(157,78,221,0.6)] animate-pulse border border-white/20 text-[9px]">
                        NETHRA
                      </div>
                      <span className="text-[7px] text-slate-400 font-mono mt-0.5">Commander</span>
                    </div>

                    {/* Director 1: Tech (Top-Left) */}
                    <div className="absolute left-[15%] top-[10%] flex flex-col items-center">
                      <div className="w-8 h-8 rounded-lg bg-cyan-950/80 border border-cyan-500/50 flex items-center justify-center text-cyan-400 font-bold shadow-[0_0_8px_rgba(6,182,212,0.2)] text-[7px] leading-none text-center">
                        BABY<br/>TECH
                      </div>
                      <span className="text-[6px] text-slate-400 mt-0.5 font-mono">Weight: 35%</span>
                    </div>

                    {/* Director 2: Risk (Top-Right) */}
                    <div className="absolute right-[15%] top-[10%] flex flex-col items-center">
                      <div className="w-8 h-8 rounded-lg bg-emerald-950/80 border border-emerald-500/50 flex items-center justify-center text-emerald-400 font-bold shadow-[0_0_8px_rgba(16,185,129,0.2)] text-[7px] leading-none text-center">
                        BABY<br/>RISK
                      </div>
                      <span className="text-[6px] text-slate-400 mt-0.5 font-mono">Weight: 25%</span>
                    </div>

                    {/* Director 3: Port (Bottom-Left) */}
                    <div className="absolute left-[15%] bottom-[10%] flex flex-col items-center">
                      <div className="w-8 h-8 rounded-lg bg-amber-950/80 border border-amber-500/50 flex items-center justify-center text-amber-400 font-bold shadow-[0_0_8px_rgba(245,158,11,0.2)] text-[7px] leading-none text-center">
                        BABY<br/>PORT
                      </div>
                      <span className="text-[6px] text-slate-400 mt-0.5 font-mono">Weight: 20%</span>
                    </div>

                    {/* Director 4: Mkt (Bottom-Right) */}
                    <div className="absolute right-[15%] bottom-[10%] flex flex-col items-center">
                      <div className="w-8 h-8 rounded-lg bg-rose-950/80 border border-rose-500/50 flex items-center justify-center text-rose-400 font-bold shadow-[0_0_8px_rgba(244,63,94,0.2)] text-[7px] leading-none text-center">
                        BABY<br/>MKT
                      </div>
                      <span className="text-[6px] text-slate-400 mt-0.5 font-mono">Weight: 20%</span>
                    </div>
                  </div>
                </div>

                {/* CoT operations terminal */}
                <div className="col-span-5 bg-cyber-dark/60 rounded-xl border border-cyber-border/40 p-2.5 flex flex-col overflow-hidden">
                  <div className="flex justify-between items-center border-b border-cyber-border/20 pb-1 mb-1.5">
                    <span className="text-[9px] font-bold text-cyber-purple font-mono uppercase tracking-wider flex items-center gap-1">
                      ⛓ Chain-of-Thought Operations Log
                    </span>
                    <span className="text-[7px] text-cyber-green font-mono">100% Rust Swarm</span>
                  </div>
                  <div className="flex-1 overflow-y-auto space-y-1.5 text-[8px] font-mono scrollbar-cyber text-slate-300 pr-1 leading-normal">
                    <div className="text-cyan-400 flex items-start gap-1">
                      <span className="shrink-0 text-slate-600">[08:45:01]</span>
                      <span>[TECH] Bollinger bands compress, RSI at 43. MACD histogram ticking upward. Firing 18/25 bull skills. Conviction: 72%</span>
                    </div>
                    <div className="text-emerald-400 flex items-start gap-1">
                      <span className="shrink-0 text-slate-600">[08:45:02]</span>
                      <span>[RISK] VaR index safe. 24h drawdowns minimized. Max exposure ratio holds. Firing risk evaluation. Conviction: 95%</span>
                    </div>
                    <div className="text-amber-400 flex items-start gap-1">
                      <span className="shrink-0 text-slate-600">[08:45:02]</span>
                      <span>[PORT] Diversification balance verified. Cash reserves healthy. Firing rebalance suggestions. Conviction: 80%</span>
                    </div>
                    <div className="text-rose-400 flex items-start gap-1">
                      <span className="shrink-0 text-slate-600">[08:45:03]</span>
                      <span>[MKT] Volume spreads tighten, orderbook skew indicates buy-pressure. Firing market intelligence. Conviction: 82%</span>
                    </div>
                    <div className="text-cyber-purple font-semibold flex items-start gap-1 border-t border-cyber-border/10 pt-1">
                      <span className="shrink-0 text-slate-600">[08:45:04]</span>
                      <span>[NETHRA] Swarm consensus: Strong Bullish conviction (88.4%). Executing strategic limit BUY order. CoT completed.</span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* COLUMN 3: Auto-Trading, Order Book, Order Form */}
      <div className="col-span-3 flex flex-col gap-6 h-full overflow-hidden">
        {/* Auto-Trading Control */}
        <div className="glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
          <div className="flex justify-between items-center border-b border-cyber-border/30 pb-2 mb-3">
            <h3 className="text-xs font-bold font-mono tracking-wider text-cyber-purple">AUTONOMOUS TRADING</h3>
            <div className="flex items-center gap-2">
              <span className={cn('w-2 h-2 rounded-full', autoTradingState?.enabled ? 'bg-cyber-green animate-pulse' : 'bg-slate-500')} />
              <span className="text-[10px] font-mono text-slate-400">{autoTradingState?.enabled ? 'ACTIVE' : 'PAUSED'}</span>
            </div>
          </div>

          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2 text-[10px] font-mono">
              <span className="text-slate-500">Mode:</span>
              <Badge variant={autoTradingState?.paper_trading ? 'warning' : 'danger'}>
                {autoTradingState?.paper_trading ? 'PAPER' : 'REAL'}
              </Badge>
            </div>
            <div className="flex gap-2">
              <button onClick={handleStartAutoTrade} disabled={autoTradingState?.enabled}
                className={cn(autoTradingState?.enabled ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-success')}>
                START
              </button>
              <button onClick={handleStopAutoTrade} disabled={!autoTradingState?.enabled}
                className={cn(!autoTradingState?.enabled ? 'btn-secondary opacity-50 cursor-not-allowed' : 'btn-danger')}>
                STOP
              </button>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2 text-[9px] font-mono mb-3">
            <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
              <span className="text-slate-500 block">Balance</span>
              <span className="text-cyber-green font-bold tabular-nums">${autoTradingState?.balance?.toLocaleString() ?? '100,000'}</span>
            </div>
            <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
              <span className="text-slate-500 block">Positions</span>
              <span className="text-slate-200 font-bold">{autoTradingState?.open_positions?.length ?? 0}</span>
            </div>
            <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
              <span className="text-slate-500 block">Drawdown</span>
              <span className={cn('font-bold', (autoTradingState?.current_drawdown_pct ?? 0) > 10 ? 'text-red-400' : 'text-slate-200')}>
                {autoTradingState?.current_drawdown_pct?.toFixed(1) ?? '0.0'}%
              </span>
            </div>
            <div className="bg-cyber-dark/40 rounded p-2 border border-cyber-border/30">
              <span className="text-slate-500 block">Interval</span>
              <span className="text-cyber-purple font-bold">{autoTradingState?.analysis_interval_secs ?? 300}s</span>
            </div>
          </div>

          {autoTradingState?.last_outcomes && autoTradingState.last_outcomes.length > 0 && (
            <div className="mb-2">
              <h4 className="text-[9px] font-bold text-slate-500 mb-1 font-mono">RECENT DECISIONS</h4>
              <div className="max-h-20 overflow-y-auto space-y-1 scrollbar-cyber">
                {autoTradingState.last_outcomes.slice(-5).reverse().map((outcome, i) => (
                  <div key={i} className="flex items-center text-[8px] font-mono bg-cyber-dark/30 rounded px-2 py-1 gap-1">
                    <span className="text-slate-400 truncate max-w-[60px]">{outcome.symbol}</span>
                    <span className={cn('font-bold shrink-0', outcome.action?.Buy ? 'text-cyber-green' : outcome.action?.Sell ? 'text-red-400' : outcome.action?.Hold ? 'text-slate-400' : 'text-yellow-400')}>
                      {outcome.action?.Buy ? 'BUY' : outcome.action?.Sell ? 'SELL' : outcome.action?.Hold ? 'HOLD' : 'SKIP'}
                    </span>
                    <span className="text-slate-500 truncate max-w-[40px]">{outcome.regime ?? '—'}</span>
                    <span className={cn('ml-auto shrink-0', outcome.conviction > 0 ? 'text-cyber-green' : 'text-red-400')}>{(outcome.conviction * 100).toFixed(0)}%</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {perfStats && (
            <div className="border-t border-cyber-border/20 pt-3">
              <h4 className="text-[9px] font-bold text-slate-500 mb-1.5 font-mono">PERFORMANCE</h4>
              <div className="grid grid-cols-3 gap-1.5">
                <div className="stat-card p-1.5 text-center">
                  <span className="text-[10px] font-bold text-cyber-green block">{perfStats.win_rate?.toFixed(1) ?? '0.0'}%</span>
                  <span className="text-[7px] text-slate-500 font-mono">Win Rate</span>
                </div>
                <div className="stat-card p-1.5 text-center">
                  <span className="text-[10px] font-bold text-cyber-purple block">{perfStats.total_trades ?? 0}</span>
                  <span className="text-[7px] text-slate-500 font-mono">Trades</span>
                </div>
                <div className="stat-card p-1.5 text-center">
                  <span className={cn('text-[10px] font-bold block', perfStats.total_pnl >= 0 ? 'text-cyber-green' : 'text-red-400')}>
                    ${perfStats.total_pnl.toFixed(0)}
                  </span>
                  <span className="text-[7px] text-slate-500 font-mono">Total P&L</span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Order Book */}
        <div className="glass-panel rounded-xl p-4 flex flex-col flex-1 overflow-hidden min-h-[120px]">
          <h3 className="text-xs font-bold font-mono tracking-wider text-slate-400 mb-2">ORDER BOOK</h3>
          <div className="flex-1 flex flex-col overflow-hidden text-[10px] font-mono">
            {/* Asks */}
            <div className="flex-1 overflow-y-auto space-y-0.5 mb-1 flex flex-col justify-end scrollbar-cyber">
              {l2Data.asks.map((ask, idx) => (
                <div key={idx} className="flex justify-between py-0.5 relative hover:bg-white/5 px-2 transition-colors">
                  <div className="absolute right-0 top-0 bottom-0 bg-red-500/10 pointer-events-none transition-all"
                    style={{ width: `${(ask.amount / maxAskAmount) * 100}%` }} />
                  <span className="text-red-400 z-10 font-bold">${formatPrice(ask.price)}</span>
                  <span className="text-slate-300 z-10">{ask.amount.toFixed(4)}</span>
                </div>
              ))}
            </div>

            {/* Spread */}
            <div className="py-1.5 border-y border-cyber-border/40 text-center font-mono my-1 text-cyber-purple font-semibold bg-cyber-panel/30 flex justify-between px-3 text-[10px]">
              <span>SPREAD</span>
              <span>${formatPrice(spread)} ({spreadBps.toFixed(1)} bps)</span>
            </div>

            {/* Bids */}
            <div className="flex-1 overflow-y-auto space-y-0.5 mt-1 scrollbar-cyber">
              {l2Data.bids.map((bid, idx) => (
                <div key={idx} className="flex justify-between py-0.5 relative hover:bg-white/5 px-2 transition-colors">
                  <div className="absolute right-0 top-0 bottom-0 bg-cyber-green/10 pointer-events-none transition-all"
                    style={{ width: `${(bid.amount / maxBidAmount) * 100}%` }} />
                  <span className="text-cyber-green z-10 font-bold">${formatPrice(bid.price)}</span>
                  <span className="text-slate-300 z-10">{bid.amount.toFixed(4)}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Order Form */}
        <div className="glass-panel rounded-xl p-4 flex flex-col overflow-hidden">
          <div className="flex border-b border-cyber-border/40 pb-2 mb-3 justify-between items-center">
            <div className="flex gap-1.5">
              {ORDER_TYPES.map((t) => (
                <button key={t} onClick={() => setOrderType(t)}
                  className={cn('text-[10px] font-bold font-mono px-2 py-0.5 rounded transition-all',
                    orderType === t ? 'bg-cyber-panel border border-cyber-border text-slate-200' : 'text-slate-500 hover:text-slate-300'
                  )}>{t}</button>
              ))}
            </div>
            <div className="flex bg-cyber-panel p-0.5 rounded border border-cyber-border/40">
              {ORDER_SIDES.map((s) => (
                <button key={s} onClick={() => setOrderSide(s)}
                  className={cn('text-[10px] font-bold font-mono px-3 py-0.5 rounded transition-all',
                    orderSide === s
                      ? s === 'BUY' ? 'bg-cyber-green/20 text-cyber-green shadow-[0_0_8px_rgba(34,197,94,0.2)]' : 'bg-red-500/20 text-red-400 shadow-[0_0_8px_rgba(239,68,68,0.2)]'
                      : 'text-slate-500'
                  )}>{s}</button>
              ))}
            </div>
          </div>

          <div className="flex-1 flex flex-col justify-between text-xs font-mono space-y-3">
            <div className="space-y-2">
              {orderType === 'LIMIT' && limitPriceInput && (
                <div className="text-[9px] font-mono text-slate-500 bg-cyber-dark/40 rounded-lg p-2 border border-cyber-border/30 flex justify-between items-center">
                  <span>ORDER TOTAL:</span>
                  <span className="text-cyber-purple font-bold tabular-nums">
                    {formatCurrency(Number(limitPriceInput) * Number(amountInput || 0))}
                  </span>
                </div>
              )}
              <div className="flex justify-between items-center text-[10px] text-slate-500">
                <span>AVAILABLE BALANCE:</span>
                <span className="text-slate-200 font-bold tabular-nums">${cash.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
              </div>

              {orderType === 'LIMIT' && (
                <div>
                  <label className="text-[10px] text-slate-500 block mb-1">LIMIT PRICE (USD)</label>
                  <div className="flex">
                    <button onClick={() => setLimitPriceInput((p) => Math.max(0, Number(p) - Number(p) * 0.001).toFixed(2))}
                      className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-l text-[10px] font-bold">−</button>
                    <input type="text" value={limitPriceInput} onChange={(e) => setLimitPriceInput(e.target.value)}
                      className="w-full bg-cyber-dark/60 border-y border-cyber-border text-center text-slate-200 focus:outline-none focus:border-cyber-purple font-mono py-1" />
                    <button onClick={() => setLimitPriceInput((p) => (Number(p) + Number(p) * 0.001).toFixed(2))}
                      className="bg-cyber-panel border border-cyber-border text-slate-300 px-2 py-1 rounded-r text-[10px] font-bold">+</button>
                  </div>
                </div>
              )}

              <div>
                <label className="text-[10px] text-slate-500 block mb-1">QUANTITY</label>
                <input type="text" value={amountInput} onChange={(e) => setAmountInput(e.target.value)}
                  className="input-cyber text-center" />
              </div>

              <div className="grid grid-cols-4 gap-1">
                {AMOUNT_PRESETS.map((pct) => (
                  <button key={pct} onClick={() => {
                    const price = orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice;
                    if (price > 0) setAmountInput(((cash * pct) / price).toFixed(4));
                  }}
                    className="bg-cyber-panel hover:bg-cyber-border/40 text-slate-400 border border-cyber-border rounded py-1 text-[9px] font-bold transition-all">
                    {pct * 100}%
                  </button>
                ))}
              </div>

              <div className="border-t border-cyber-border/20 pt-2 flex justify-between text-[10px] text-slate-500">
                <span>EST. TOTAL:</span>
                <span className="text-cyber-purple font-bold tabular-nums">
                  {formatCurrency((orderType === 'LIMIT' ? Number(limitPriceInput) : currentPrice) * Number(amountInput || 0))}
                </span>
              </div>
            </div>

            <button onClick={handlePlaceOrder}
              className={cn('w-full py-2.5 font-bold font-mono rounded text-sm transition-all shadow-lg',
                orderSide === 'BUY'
                  ? 'bg-cyber-green/20 hover:bg-cyber-green/30 text-cyber-green border border-cyber-green/40 shadow-green hover:shadow-[0_0_15px_rgba(34,197,94,0.25)]'
                  : 'bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/40 hover:shadow-[0_0_15px_rgba(239,68,68,0.25)]'
              )}>
              PLACE {orderSide} {orderType} ORDER
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  );
}
