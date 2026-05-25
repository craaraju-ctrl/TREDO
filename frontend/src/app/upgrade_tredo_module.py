import re

filepath = "/home/varma/Freebuff/tredo/frontend/src/components/tredo/TredoModule.tsx"
with open(filepath, "r") as f:
    content = f.read()

# 1. Add import
if "lightweight-charts" not in content:
    content = content.replace(
        "import { useAtom } from 'jotai';",
        "import { useAtom } from 'jotai';\nimport { createChart, ColorType, CrosshairMode, LineStyle } from 'lightweight-charts';"
    )

# 2. Add lightweight-charts refs
refs_target = """  // ── Refs ──────────────────────────────────────────────────────────────────
  const canvasRef = useRef<HTMLCanvasElement>(null);"""
refs_replacement = """  // ── Refs ──────────────────────────────────────────────────────────────────
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<any>(null);
  const candleSeriesRef = useRef<any>(null);
  const volSeriesRef = useRef<any>(null);
  const smaSeriesRef = useRef<any>(null);
  const emaSeriesRef = useRef<any>(null);
  const bbUpperRef = useRef<any>(null);
  const bbLowerRef = useRef<any>(null);
  const vwapSeriesRef = useRef<any>(null);
  const priceLinesRef = useRef<any[]>([]);"""

content = content.replace(refs_target, refs_replacement)

# 3. Replace canvas hook with lightweight charts hook
hook_pattern = r"// ── Canvas chart renderer ─────────────────────────────────────────────────\n  useEffect\(\(\) => \{[\s\S]*?\}, \[priceHistory, selectedAsset, currentPrice, drawingLines\]\);"

new_hook = """// ── Professional TradingView lightweight-charts Engine ─────────────────────
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

    const candleSeries = chart.addCandlestickSeries({
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderUpColor: '#22c55e',
      borderDownColor: '#ef4444',
      wickUpColor: '#22c55e',
      wickDownColor: '#ef4444',
    });

    const volSeries = chart.addHistogramSeries({
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
    } catch { /* ignore ordering errors on rapid updates */ }

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
      const smaSeries = chart.addLineSeries({ color: '#00bcd4', lineWidth: 1, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
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
      const emaSeries = chart.addLineSeries({ color: '#ff007f', lineWidth: 1, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
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
      const upperSeries = chart.addLineSeries({ color: 'rgba(251,191,36,0.7)', lineWidth: 1, priceLineVisible: false, lastValueVisible: false, crosshairMarkerVisible: false });
      const lowerSeries = chart.addLineSeries({ color: 'rgba(251,191,36,0.7)', lineWidth: 1, priceLineVisible: false, lastValueVisible: false, crosshairMarkerVisible: false });
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
      const vwapSeries = chart.addLineSeries({ color: 'rgba(255,255,255,0.7)', lineWidth: 2, lineStyle: LineStyle.Dashed, priceLineVisible: false, lastValueVisible: true, crosshairMarkerVisible: false });
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

  }, [priceHistory, selectedAsset, selectedTimeframe, currentPrice, activeIndicators, indicatorPeriods, drawingLines]);"""

content = re.sub(hook_pattern, new_hook, content)

# 4. Replace canvas ref in JSX
content = content.replace(
    '<canvas ref={canvasRef} className="absolute inset-0 w-full h-full" aria-label={`Candlestick chart for ${selectedAsset}`} />',
    '<div ref={chartContainerRef} className="absolute inset-0 w-full h-full" aria-label={`Candlestick chart for ${selectedAsset}`} />'
)

with open(filepath, "w") as f:
    f.write(content)

print("Upgrade python script executed successfully.")
