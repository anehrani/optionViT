/// Greeks calculation module for options portfolio analysis
/// 
/// This module provides functions to calculate collective Greeks (Delta, Gamma)
/// and price projections for a portfolio of options instruments.

use serde_json::Value;
use std::f64::consts::PI;

/// Represents the collective Greeks for a portfolio
#[derive(Debug, Clone)]
pub struct CollectiveGreeks {
    pub total_delta: f64,
    pub total_gamma: f64,
    pub weighted_delta: f64,
    pub weighted_gamma: f64,
    pub total_notional: f64,
    pub instrument_count: usize,
}

/// Calculate the collective Greeks from a list of instruments
/// 
/// # Arguments
/// * `instruments` - JSON array of option instruments with market data
/// 
/// # Returns
/// * `CollectiveGreeks` - Aggregated Greeks for the portfolio
pub fn calculate_collective_greeks(instruments: &[Value]) -> CollectiveGreeks {
    let mut total_delta = 0.0;
    let mut total_gamma = 0.0;
    let mut total_notional = 0.0;
    let mut count = 0;

    for instrument in instruments {
        // Get open interest (position size)
        let open_interest = instrument
            .get("open_interest")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if open_interest <= 0.0 {
            continue;
        }

        // Get mark price (current option price)
        let mark_price = instrument
            .get("mark_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Get mark IV (implied volatility)
        let mark_iv = instrument
            .get("mark_iv")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Determine if it's a call or put
        let instrument_name = instrument
            .get("instrument_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let is_call = instrument_name.ends_with("-C");

        // Simple approximation: If we have IV, estimate Greeks
        // For more accurate calculations, you'd need underlying price, strike, time to expiry
        if mark_iv > 0.0 && mark_price > 0.0 {
            // Simplified Delta estimation (ATM options have delta ~0.5 for calls, -0.5 for puts)
            // This is a rough approximation - in production, use proper Black-Scholes
            let estimated_delta = if is_call { 0.5 } else { -0.5 };
            
            // Simplified Gamma estimation (highest for ATM options)
            // Gamma is typically highest around 0.05-0.15 for ATM options
            let estimated_gamma = 0.1;

            // Weight by position size
            let position_delta = estimated_delta * open_interest;
            let position_gamma = estimated_gamma * open_interest;
            let position_notional = mark_price * open_interest;

            total_delta += position_delta;
            total_gamma += position_gamma;
            total_notional += position_notional;
            count += 1;
        }
    }

    let weighted_delta = if count > 0 {
        total_delta / count as f64
    } else {
        0.0
    };

    let weighted_gamma = if count > 0 {
        total_gamma / count as f64
    } else {
        0.0
    };

    CollectiveGreeks {
        total_delta,
        total_gamma,
        weighted_delta,
        weighted_gamma,
        total_notional,
        instrument_count: count,
    }
}

/// Calculate price projection based on underlying price movement
/// 
/// # Arguments
/// * `greeks` - The collective Greeks of the portfolio
/// * `price_change_percent` - Expected price change in percentage (e.g., 1.0 for 1% up, -2.0 for 2% down)
/// * `underlying_price` - Current underlying asset price
/// 
/// # Returns
/// * Tuple of (linear_projection, gamma_adjusted_projection)
pub fn calculate_price_projection(
    greeks: &CollectiveGreeks,
    price_change_percent: f64,
    underlying_price: f64,
) -> (f64, f64) {
    // Use actual underlying price passed as parameter
    let price_change = underlying_price * (price_change_percent / 100.0);

    // Linear projection: P&L = Delta × Price Change
    let linear_projection = greeks.total_delta * price_change;

    // Gamma-adjusted projection: P&L = Delta × ΔP + 0.5 × Gamma × ΔP²
    let gamma_adjustment = 0.5 * greeks.total_gamma * price_change * price_change;
    let gamma_adjusted_projection = linear_projection + gamma_adjustment;

    (linear_projection, gamma_adjusted_projection)
}

/// Calculate advanced Black-Scholes Greeks (for future implementation)
/// This is a placeholder for more accurate Greeks calculation
#[allow(dead_code)]
fn black_scholes_greeks(
    spot_price: f64,
    strike: f64,
    time_to_expiry: f64,
    volatility: f64,
    risk_free_rate: f64,
    is_call: bool,
) -> (f64, f64) {
    // Placeholder for Black-Scholes implementation
    // Would calculate d1, d2, and derive Delta and Gamma
    let d1 = (spot_price.ln() - strike.ln() 
        + (risk_free_rate + volatility.powi(2) / 2.0) * time_to_expiry)
        / (volatility * time_to_expiry.sqrt());
    
    let d2 = d1 - volatility * time_to_expiry.sqrt();
    
    // Standard normal PDF
    let n_prime_d1 = (-(d1.powi(2)) / 2.0).exp() / (2.0 * PI).sqrt();
    
    // Delta
    let delta = if is_call {
        norm_cdf(d1)
    } else {
        norm_cdf(d1) - 1.0
    };
    
    // Gamma (same for calls and puts)
    let gamma = n_prime_d1 / (spot_price * volatility * time_to_expiry.sqrt());
    
    (delta, gamma)
}

/// Standard normal cumulative distribution function (CDF)
#[allow(dead_code)]
fn norm_cdf(x: f64) -> f64 {
    (1.0 + erf(x / 2.0_f64.sqrt())) / 2.0
}

/// Error function approximation
#[allow(dead_code)]
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_collective_greeks_empty() {
        let instruments: Vec<Value> = vec![];
        let greeks = calculate_collective_greeks(&instruments);
        
        assert_eq!(greeks.total_delta, 0.0);
        assert_eq!(greeks.total_gamma, 0.0);
        assert_eq!(greeks.instrument_count, 0);
    }

    #[test]
    fn test_price_projection() {
        let greeks = CollectiveGreeks {
            total_delta: 10.0,
            total_gamma: 0.5,
            weighted_delta: 0.5,
            weighted_gamma: 0.025,
            total_notional: 100000.0,
            instrument_count: 20,
        };

        let underlying_price = 100000.0; // Example price for test
        let (linear, gamma_adjusted) = calculate_price_projection(&greeks, 1.0, underlying_price);
        
        assert!(linear > 0.0);
        assert!(gamma_adjusted != linear);
    }
}
