import re

filepath = "/home/varma/Freebuff/tredo/frontend/src/components/tredo/TredoModule.tsx"
with open(filepath, "r") as f:
    content = f.read()

# Fix imports
content = content.replace(
    "import { createChart, ColorType, CrosshairMode, LineStyle } from 'lightweight-charts';",
    "import { createChart, CandlestickSeries, HistogramSeries, LineSeries, ColorType, CrosshairMode, LineStyle } from 'lightweight-charts';"
)

# Fix add methods
content = content.replace("chart.addCandlestickSeries(", "chart.addSeries(CandlestickSeries, ")
content = content.replace("chart.addHistogramSeries(", "chart.addSeries(HistogramSeries, ")
content = content.replace("chart.addLineSeries(", "chart.addSeries(LineSeries, ")

with open(filepath, "w") as f:
    f.write(content)

print("Fixed lightweight-charts v5 typings in TredoModule.tsx")
