# 📊 Greeks Analysis & Portfolio Risk Documentation

## Overview

The Greeks Analysis feature provides real-time portfolio risk metrics and price projection calculations for cryptocurrency options. This new section calculates collective Delta and Gamma values along with scenario-based profit/loss projections.

## 🎯 Features

### 1. Collective Greeks Calculation
- **Total Delta (Δ)**: Measures the portfolio's directional exposure to underlying price changes
- **Total Gamma (Γ)**: Measures how Delta changes with price movements (convexity)
- **Net Position**: Total notional value of the portfolio in USD
- **Instrument Count**: Number of options with valid Greeks data

### 2. Price Movement Projections
Calculates P&L for four scenarios:
- 🚀 **+5% Up Move**: Bullish scenario
- 📈 **+1% Up Move**: Modest upward movement
- 📉 **-1% Down Move**: Modest downward movement
- 💥 **-5% Down Move**: Bearish scenario

Each projection shows:
- **Total P&L**: Gamma-adjusted profit/loss
- **Linear Component**: Delta × Price Change
- **Gamma Adjustment**: Convexity effect

## 📐 Calculation Methodology

### Delta Estimation

Delta represents the rate of change of option price with respect to underlying price.

**For Call Options**:
```
Moneyness = Spot Price / Strike Price

If moneyness > 1.10:  Delta = 0.8  (Deep ITM)
If moneyness > 1.02:  Delta = 0.6  (ITM)
If moneyness > 0.98:  Delta = 0.5  (ATM)
If moneyness > 0.90:  Delta = 0.3  (OTM)
Else:                 Delta = 0.1  (Deep OTM)
```

**For Put Options**:
```
If moneyness > 1.10:  Delta = -0.2  (Deep OTM)
If moneyness > 1.02:  Delta = -0.4  (OTM)
If moneyness > 0.98:  Delta = -0.5  (ATM)
If moneyness > 0.90:  Delta = -0.7  (ITM)
Else:                 Delta = -0.9  (Deep ITM)
```

### Gamma Estimation

Gamma represents the rate of change of Delta with respect to underlying price.

```
If 0.95 < moneyness < 1.05:  Gamma = 0.015  (ATM)
If 0.90 < moneyness < 1.10:  Gamma = 0.008  (Near the money)
Else:                         Gamma = 0.002  (Far from money)

Adjusted Gamma = Base Gamma / (1 + IV)
```

**Note**: Higher implied volatility (IV) reduces Gamma due to time value effects.

### Position-Weighted Greeks

```rust
Position Delta = Estimated Delta × Open Interest
Position Gamma = Estimated Gamma × Open Interest
Position Notional = Mark Price × Open Interest × Spot Price

Total Delta = Σ(Position Delta)
Total Gamma = Σ(Position Gamma)
Total Notional = Σ(Position Notional)
```

### Price Projection Formula

**Linear Projection** (First-order approximation):
```
P&L = Delta × ΔPrice
```

**Gamma-Adjusted Projection** (Second-order approximation):
```
P&L = Delta × ΔPrice + 0.5 × Gamma × (ΔPrice)²
```

Where:
- `ΔPrice = Spot Price × (Price Change % / 100)`
- For BTC, assumes spot price ≈ $60,000

## 🎨 Visual Design

### Color Scheme
- **Background**: Warm orange gradient (#fff7ed → #ffedd5)
- **Cards**: White with orange borders
- **Text**: Brown tones (#7c2d12, #9a3412)
- **Positive P&L**: Green (#059669)
- **Negative P&L**: Red (#dc2626)

### Layout
- **Greeks Grid**: 4 columns on desktop, 2 on tablet, 1 on mobile
- **Projection Grid**: 3 columns on desktop, 1 on mobile
- **Responsive**: Adapts to all screen sizes

## 📊 Example Calculations

### Scenario: Portfolio with 100 BTC Call Options

**Given**:
- 100 ATM Call Options (Strike = $60,000)
- Open Interest per option = 1.0 BTC
- Current BTC Price = $60,000
- Mark IV = 0.65 (65%)

**Greeks Calculation**:
```
Delta per option (ATM Call) = 0.5
Total Delta = 0.5 × 100 = 50.0

Gamma per option (ATM) = 0.015 / (1 + 0.65) ≈ 0.009
Total Gamma = 0.009 × 100 = 0.9
```

**Price Projection (+5% move)**:
```
Price Change = $60,000 × 5% = $3,000

Linear P&L = 50.0 × $3,000 = $150,000

Gamma Adjustment = 0.5 × 0.9 × ($3,000)² = $4,050,000

Total P&L = $150,000 + $4,050,000 = $4,200,000
```

## 🔍 Interpreting the Results

### Delta Interpretation

| Delta Range | Meaning | Portfolio Characteristic |
|-------------|---------|-------------------------|
| > +50 | Strongly bullish | Profits from price increases |
| +10 to +50 | Moderately bullish | Slight upward bias |
| -10 to +10 | Market neutral | Hedged or balanced |
| -50 to -10 | Moderately bearish | Slight downward bias |
| < -50 | Strongly bearish | Profits from price decreases |

### Gamma Interpretation

| Gamma Value | Meaning | Risk Level |
|-------------|---------|-----------|
| > 1.0 | High convexity | High risk/reward |
| 0.5 - 1.0 | Moderate convexity | Moderate risk |
| 0.1 - 0.5 | Low convexity | Low risk |
| < 0.1 | Minimal convexity | Very low risk |

**Positive Gamma**: 
- Delta increases as price rises (accelerating gains)
- Delta decreases as price falls (decelerating losses)
- Good for long option positions

**Negative Gamma**: 
- Delta decreases as price rises (decelerating gains)
- Delta increases as price falls (accelerating losses)
- Typical for short option positions

## 🎓 Trading Strategies Based on Greeks

### High Positive Delta Portfolio
- **Strategy**: Long calls or short puts
- **Market View**: Bullish
- **Risk**: Significant losses if market falls
- **Action**: Consider hedging with long puts

### High Negative Delta Portfolio
- **Strategy**: Long puts or short calls
- **Market View**: Bearish
- **Risk**: Significant losses if market rises
- **Action**: Consider hedging with long calls

### High Gamma Portfolio
- **Strategy**: Long straddles/strangles
- **Market View**: Expecting large moves
- **Risk**: Time decay if market stays flat
- **Action**: Monitor closely, may need to adjust

### Low Gamma Portfolio
- **Strategy**: Short options or far OTM positions
- **Market View**: Expecting minimal movement
- **Risk**: Unlimited if move is larger than expected
- **Action**: Set stop losses

## ⚙️ Technical Implementation

### Module: `src/greeks.rs`

**Key Functions**:
```rust
pub fn calculate_collective_greeks(instruments: &[Value]) -> CollectiveGreeks
pub fn calculate_price_projection(greeks: &CollectiveGreeks, price_change_percent: f64) -> (f64, f64)
```

**Data Structure**:
```rust
pub struct CollectiveGreeks {
    pub total_delta: f64,
    pub total_gamma: f64,
    pub weighted_delta: f64,
    pub weighted_gamma: f64,
    pub total_notional: f64,
    pub instrument_count: usize,
}
```

### UI Integration: `src/ui.rs`

**JavaScript Functions**:
- `displayGreeks(data)` - Renders Greeks section
- `calculateCollectiveGreeks(instruments)` - Client-side calculation
- `calculatePriceProjection(greeks, priceChangePercent)` - P&L projection

## 🚀 Future Enhancements

### 1. Advanced Greeks
- **Vega (ν)**: Sensitivity to volatility changes
- **Theta (Θ)**: Time decay measurement
- **Rho (ρ)**: Interest rate sensitivity

### 2. Real-Time Updates
- WebSocket integration for live prices
- Auto-recalculation on price changes
- Alert thresholds for risk limits

### 3. Historical Analysis
- Greeks over time charts
- Backtest portfolio performance
- Volatility surface visualization

### 4. Risk Management Tools
- Position limits based on Delta/Gamma
- Hedging suggestions
- Optimal portfolio rebalancing

### 5. Black-Scholes Integration
Replace simplified estimations with full Black-Scholes calculations:
```rust
fn black_scholes_greeks(
    spot_price: f64,
    strike: f64,
    time_to_expiry: f64,
    volatility: f64,
    risk_free_rate: f64,
    is_call: bool,
) -> (f64, f64, f64, f64, f64)  // Delta, Gamma, Vega, Theta, Rho
```

### 6. Custom Scenarios
- User-defined price movements
- Multiple underlying assets
- Correlation effects
- Stress testing

## 📚 Additional Resources

### Learning Materials
- [Options Greeks Explained](https://www.optionsplaybook.com/options-introduction/what-are-the-greeks/)
- [Black-Scholes Model](https://en.wikipedia.org/wiki/Black%E2%80%93Scholes_model)
- [Portfolio Greeks](https://www.investopedia.com/terms/g/greeks.asp)

### Books
- "Options, Futures, and Other Derivatives" by John Hull
- "Option Volatility and Pricing" by Sheldon Natenberg
- "Dynamic Hedging" by Nassim Taleb

### Academic Papers
- Black, F., & Scholes, M. (1973). "The Pricing of Options and Corporate Liabilities"
- Merton, R. C. (1973). "Theory of Rational Option Pricing"

## ⚠️ Important Disclaimers

### Limitations of Current Implementation

1. **Simplified Greeks**: Current calculations use approximations based on moneyness and IV. Production systems should use proper Black-Scholes formulas with actual time to expiry.

2. **Assumed Spot Price**: System assumes BTC spot price of $60,000. Real implementation should fetch live prices.

3. **No Time Decay**: Current implementation doesn't account for Theta (time decay). Options lose value as expiration approaches.

4. **Static IV**: Uses mark IV from Deribit. Doesn't account for volatility smile or skew.

5. **No Correlation**: Treats each option independently. Real portfolios may have correlation effects.

### Risk Warning

⚠️ **This tool is for informational purposes only and should not be considered financial advice.**

- Greeks are estimates and may not reflect actual P&L
- Options trading involves significant risk
- Past performance doesn't guarantee future results
- Always consult with a financial advisor before trading
- Never risk more than you can afford to lose

## 🎯 Summary

The Greeks Analysis section provides:

✅ **Real-time portfolio risk metrics**
✅ **Directional exposure measurement (Delta)**
✅ **Convexity analysis (Gamma)**
✅ **Scenario-based P&L projections**
✅ **Visual, intuitive interface**
✅ **Automatic calculation on data filtering**

This feature transforms raw options data into actionable risk intelligence, helping traders understand their portfolio's sensitivity to market movements and make informed hedging decisions.

---

**Built with 💹 for Professional Options Traders**
