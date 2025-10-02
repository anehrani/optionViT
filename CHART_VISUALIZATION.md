# 📊 Interactive Greeks Chart Visualization

## Overview

The Interactive Greeks Chart provides a comprehensive visual analysis of portfolio Greeks and value projections across different underlying price levels. All metrics are plotted on a **normalized 0-100 scale** in a single, easy-to-understand chart.

## 🎯 What's Visualized

The chart displays three key metrics simultaneously:

### 1. **Normalized Delta (Blue Line)**
- Represents directional exposure at each price level
- Shows how Delta changes as the underlying price moves
- Linear relationship affected by Gamma

### 2. **Normalized Gamma (Orange Line)**  
- Represents convexity (rate of Delta change)
- Shows portfolio's sensitivity to price movements
- Higher Gamma = faster Delta changes

### 3. **Normalized Value Projection (Green Area)**
- Represents expected P&L at each price level
- Non-linear curve due to Gamma effects
- Filled area shows profit/loss regions

## 📐 Chart Features

### Interactive Elements
- **Hover Tooltips**: Detailed information at each price point
  - Actual underlying price
  - Percentage change from current spot
  - Actual Greek values (not just normalized)
  - P&L projection in dollars
  
- **Legend**: Toggle visibility of each metric
- **Responsive Design**: Adapts to screen size
- **Smooth Lines**: Cubic interpolation for visual clarity

### Price Range
- **Default Range**: -20% to +20% from current spot price
- **Resolution**: 41 data points (1% increments)
- **X-Axis**: Shows underlying prices ($48K - $72K for BTC at $60K)
- **Y-Axis**: Normalized scale 0-100 for fair comparison

## 🔢 Normalization Formula

To compare metrics with different scales, all values are normalized:

```javascript
// Find maximum absolute value for each metric
maxDelta = max(|delta values|)
maxGamma = max(|gamma values|)
maxValue = max(|value projection|)

// Normalize to 0-100 scale (50 = center line)
normalizedDelta = (delta / maxDelta) × 50 + 50
normalizedGamma = (gamma / maxGamma) × 50 + 50
normalizedValue = (value / maxValue) × 50 + 50
```

**Why 50 + 50?**
- Centers the data around 50
- Positive values go to 100
- Negative values go to 0
- Makes comparison intuitive

## 📊 Reading the Chart

### Delta Line Interpretation

**Upward Slope (Positive Gamma)**:
```
  100 │         ╱╱╱
      │       ╱╱
   50 │     ╱    ← Delta increases with price
      │   ╱╱
    0 │ ╱╱
      └─────────────
```
- Long options portfolio
- Benefits from price increases
- Delta grows as price rises

**Downward Slope (Negative Gamma)**:
```
  100 │ ╲╲
      │   ╲╲
   50 │     ╲    ← Delta decreases with price
      │       ╲╲
    0 │         ╲╲╲
      └─────────────
```
- Short options portfolio
- Loses on large moves
- Delta shrinks as price rises

### Gamma Line Interpretation

**Flat Line**:
- Consistent Gamma across prices
- Simplified model assumption
- Real Gamma peaks at-the-money

**Peak at Center**:
- ATM options have highest Gamma
- More realistic scenario
- Would show bell curve shape

### Value Projection Interpretation

**Convex Curve (Positive)**:
```
  100 │     ╱‾‾‾╲
      │   ╱       ╲
   50 │─────┼─────── Current Price
      │              
    0 │               
      └─────────────
```
- Long straddle/strangle
- Profits from large moves
- Symmetric profit potential

**Concave Curve (Negative)**:
```
  100 │               
      │              
   50 │─────┼─────── Current Price
      │   ╲       ╱
    0 │     ╲___╱
      └─────────────
```
- Short straddle/strangle
- Maximum profit at current price
- Losses on large moves

## 🎨 Chart Info Panel

Below the chart, key metrics are displayed:

| Metric | Description | Example |
|--------|-------------|---------|
| **Current Spot** | Assumed underlying price | $60,000 |
| **Price Range** | Chart display range | $48,000 - $72,000 |
| **Max Delta** | Largest Delta value | 50.00 |
| **Max Gamma** | Largest Gamma value | 0.0150 |
| **Max P&L Impact** | Potential profit/loss | ±$4,200,000 |

## 🔍 Use Cases

### 1. Risk Assessment
**Question**: How much can I lose if BTC drops 10%?

**Answer**: 
1. Find -10% on X-axis
2. Read Green line value
3. Hover for actual P&L dollar amount

### 2. Hedge Analysis
**Question**: Is my portfolio balanced?

**Answer**:
- Blue line near 50 = Delta neutral
- Blue line > 50 = Net long
- Blue line < 50 = Net short

### 3. Gamma Scalping
**Question**: Where is my Gamma exposure highest?

**Answer**:
- Orange line peaks = high Gamma zones
- Flat orange line = consistent Gamma
- Use to plan dynamic hedging

### 4. Scenario Planning
**Question**: What if BTC rallies to $70K?

**Answer**:
1. Locate $70K on X-axis
2. Check all three lines
3. See Delta, Gamma, and P&L at that level

## 🎓 Advanced Interpretation

### Delta-Gamma Relationship

The chart visually shows the mathematical relationship:

```
Delta at price P = Initial Delta + Gamma × (P - P₀)

Where:
- P₀ = Current spot price
- P = Target price
- Gamma = Rate of Delta change
```

**Visual Cue**: The slope of the blue line equals the orange line value!

### Value Projection Formula

The green curve represents:

```
P&L = Delta × ΔP + 0.5 × Gamma × ΔP²

Where:
- ΔP = Price change from current
- Linear term (Delta × ΔP) = straight line
- Quadratic term (0.5 × Gamma × ΔP²) = curve
```

**Visual Cue**: Deviation from straight line = Gamma effect!

## 💡 Pro Tips

### 1. Compare Strategies
- Fetch data for different filters
- Compare chart shapes
- Identify risk-reward profiles

### 2. Find Break-Even Points
- Where green line crosses 50
- Those prices = no profit/loss
- Useful for strategy planning

### 3. Monitor Gamma Risk
- High orange line = high risk
- Need dynamic hedging
- Consider reducing exposure

### 4. Visualize Time Decay
- Currently static snapshot
- In production: animate over time
- See how curves flatten approaching expiry

## 🚀 Technical Implementation

### Chart Library
**Chart.js v4.4.0**
- Lightweight and fast
- Responsive design
- Rich customization options
- CDN delivery (no installation)

### Data Generation
```javascript
// 41 price points from -20% to +20%
for (let i = 0; i < 41; i++) {
    const percentChange = -20 + i;
    const price = currentSpot * (1 + percentChange / 100);
    
    // Calculate Delta with Gamma adjustment
    const deltaShift = gamma × (price - currentSpot);
    const adjustedDelta = baseDelta + deltaShift;
    
    // Calculate P&L
    const priceChange = price - currentSpot;
    const pnl = delta × priceChange + 
                0.5 × gamma × priceChange²;
}
```

### Normalization Process
```javascript
// Find maximum absolute values
const maxDelta = Math.max(...deltaValues.map(Math.abs));
const maxGamma = Math.max(...gammaValues.map(Math.abs));
const maxValue = Math.max(...valueProjValues.map(Math.abs));

// Normalize to 0-100 scale centered at 50
const normalizedDelta = deltaValues.map(v => 
    (v / maxDelta) × 50 + 50
);
```

## 🎨 Color Scheme

| Metric | Color | RGB | Meaning |
|--------|-------|-----|---------|
| Delta | Blue | `rgb(59, 130, 246)` | Directional exposure |
| Gamma | Orange | `rgb(234, 88, 12)` | Convexity |
| Value | Green | `rgb(16, 185, 129)` | P&L projection |

Colors chosen for:
- Maximum contrast
- Colorblind accessibility
- Professional appearance

## 📱 Responsive Design

### Desktop (> 992px)
- Full 500px height chart
- All tooltips visible
- Complete legend

### Tablet (768px - 992px)
- 400px height chart
- Compact legend
- Horizontal scroll if needed

### Mobile (< 768px)
- 300px height chart
- Stacked legend
- Touch-friendly tooltips

## 🔧 Customization Options

### Future Enhancements

1. **Price Range Slider**
```javascript
// Allow user to set custom range
const rangeSlider = document.getElementById('priceRange');
rangeSlider.addEventListener('change', (e) => {
    updateChartRange(e.target.value);
});
```

2. **Time Animation**
```javascript
// Animate how Greeks change over time
function animateTimeDecay(days) {
    for (let t = 0; t < days; t++) {
        setTimeout(() => {
            updateGreeksWithDecay(t);
            renderGreeksChart(adjustedGreeks);
        }, t * 100);
    }
}
```

3. **Volatility Surface**
```javascript
// 3D chart showing IV × Strike × Expiry
const surface3D = new Chart(ctx, {
    type: 'surface',
    data: generateVolSurfaceData()
});
```

4. **Export Functionality**
```javascript
// Download chart as PNG
function exportChart() {
    const url = greeksChartInstance.toBase64Image();
    downloadImage(url, 'greeks-chart.png');
}
```

## ⚠️ Limitations

### Current Implementation

1. **Static Gamma**: Assumes constant Gamma across prices
   - Real Gamma peaks at ATM
   - Future: Calculate actual Gamma at each price

2. **Simplified Delta**: Linear Delta adjustment
   - Real Delta follows cumulative normal distribution
   - Future: Use proper Black-Scholes formula

3. **No Time Decay**: Doesn't show Theta effects
   - Options lose value over time
   - Future: Add time slider

4. **Single Underlying**: Assumes one asset
   - Real portfolios may have multiple assets
   - Future: Multi-asset correlation charts

### Browser Compatibility

✅ **Supported**:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

❌ **Not Supported**:
- IE 11 (Chart.js v4 requirement)
- Very old mobile browsers

## 📚 Learning Resources

### Understanding Greeks
- [Options Greeks Primer](https://www.optionsplaybook.com/options-introduction/what-are-the-greeks/)
- [Visualizing Greeks](https://www.investopedia.com/articles/optioninvestor/09/visual-guide-greeks.asp)

### Chart.js Documentation
- [Chart.js Official Docs](https://www.chartjs.org/docs/latest/)
- [Line Chart Examples](https://www.chartjs.org/docs/latest/charts/line.html)
- [Tooltip Customization](https://www.chartjs.org/docs/latest/configuration/tooltip.html)

### Mathematical Background
- Black-Scholes Model
- Greek Sensitivities
- Portfolio Greeks Aggregation

## 🎯 Summary

The Interactive Greeks Chart provides:

✅ **Visual Risk Assessment**: See portfolio behavior at all price levels
✅ **Normalized Comparison**: Fair comparison of different metrics
✅ **Interactive Exploration**: Hover for detailed information
✅ **Professional Visualization**: Publication-ready charts
✅ **Real-Time Updates**: Automatic refresh with new data

**Key Innovation**: All metrics on **one normalized chart** allows instant pattern recognition and risk assessment that would be impossible with separate charts or tables.

---

**Transform complex Greeks data into visual intelligence! 📊📈**
