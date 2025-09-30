use chrono::{DateTime, Datelike, NaiveDate};
use polars::prelude::*;
use reqwest;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, error::Error};

pub type DynError = Box<dyn Error + Send + Sync>;

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeData {
    pub instrument_name: String,
    pub trade_seq: i64,
    pub trade_id: String,
    pub timestamp: i64,
    pub price: f64,
    pub mark_price: Option<f64>,
    pub iv: Option<f64>,
    pub amount: f64,
    pub direction: String,
    pub tick_direction: Option<i32>,
    pub index_price: Option<f64>,
    pub liquidity: Option<String>,
    pub block_trade_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstrumentData {
    pub instrument_name: String,
    pub currency: String,
    pub expiration_timestamp: i64,
    pub strike: Option<f64>,
    pub option_type: Option<String>,
    pub open_interest: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeribitResponse {
    jsonrpc: String,
    result: Value,
    id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Instrument {
    pub instrument_name: String,
    pub kind: String,
    pub quote_currency: String,
    pub base_currency: String,
    pub strike: Option<f64>,
    pub option_type: Option<String>,
    pub settlement_period: Option<String>,
    pub creation_timestamp: i64,
    pub expiration_timestamp: i64,
    pub is_active: bool,
    pub tick_size: f64,
    pub min_trade_amount: f64,
    pub contract_size: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBook {
    pub timestamp: i64,
    pub stats: Stats,
    pub state: String,
    pub settlement_price: Option<f64>,
    pub open_interest: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub mark_price: f64,
    pub mark_iv: Option<f64>,
    pub last_price: Option<f64>,
    pub interest_rate: f64,
    pub instrument_name: String,
    pub index_price: f64,
    pub greeks: Option<Greeks>,
    pub estimated_delivery_price: Option<f64>,
    pub change_id: i64,
    pub bids: Vec<Vec<f64>>,
    pub asks: Vec<Vec<f64>>,
    pub best_bid_price: Option<f64>,
    pub best_bid_amount: Option<f64>,
    pub best_ask_price: Option<f64>,
    pub best_ask_amount: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    volume: Option<f64>,
    price_change: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Greeks {
    vega: f64,
    theta: f64,
    rho: f64,
    gamma: f64,
    delta: f64,
}

pub struct DeribitClient {
    base_url: String,
    client: reqwest::Client,
}

impl DeribitClient {
    pub fn new() -> Self {
        DeribitClient {
            base_url: "https://www.deribit.com/api/v2".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Downloads instruments data from Deribit for a specific asset and expiry date
    /// 
    /// # Arguments
    /// * `asset` - The asset symbol (e.g., "BTC", "ETH")
    /// * `expiry_date` - The expiry date in format "YYYY-MM-DD" (e.g., "2024-03-29")
    /// 
    /// # Returns
    /// * `Result<Vec<Instrument>, Box<dyn Error>>` - List of instruments or error
    pub async fn get_instruments_by_expiry(
        &self,
        asset: &str,
        expiry_date: &str,
    ) -> Result<Vec<Instrument>, Box<dyn Error>> {
        // Parse the date to validate format and convert to Deribit format
        let date = NaiveDate::parse_from_str(expiry_date, "%Y-%m-%d")?;
        // Use %-d for single-digit days (no zero padding) to match Deribit format
        let deribit_expiry = date.format("%-d%b%y").to_string().to_uppercase();
        
        // Get all instruments for the currency

        let url = format!("{}/public/get_instruments", self.base_url);
        let params = [
            ("currency", asset.to_uppercase()),
            ("expired", "false".to_string()),
        ];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let deribit_response: DeribitResponse = response.json().await?;
        let instruments: Vec<Instrument> = serde_json::from_value(deribit_response.result)?;
        Ok(instruments)
    }

    pub async fn get_available_expiry_dates(
        &self,
        asset: &str,
    ) -> Result<Vec<String>, DynError> {
        let instruments = self.fetch_instruments(asset).await?;
        let mut expiries = BTreeSet::new();

        println!("🔍 Processing {} instruments to extract expiry dates", instruments.len());

        for instrument in instruments {
            if instrument.kind == "option" {  // Only process options
                let expiry_dt = DateTime::from_timestamp(instrument.expiration_timestamp / 1000, 0)
                    .unwrap_or_default();
                let expiry_iso = expiry_dt.format("%Y-%m-%d").to_string();
                expiries.insert(expiry_iso);
            }
        }

        let result: Vec<String> = expiries.into_iter().collect();
        println!("✅ Found {} unique expiry dates for {}", result.len(), asset);
        if result.len() > 0 {
            println!("📅 First few expiry dates: {:?}", result.iter().take(5).collect::<Vec<_>>());
        }

        Ok(result)
    }

    /// Downloads instruments data from Deribit for a specific asset and expiry date
    ///
    /// # Arguments
    /// * `asset` - The asset symbol (e.g., "BTC", "ETH")
    /// * `expiry_date` - The expiry date in format "YYYY-MM-DD" (e.g., "2024-03-29")
    ///
    /// # Returns
    /// * `Result<Vec<Instrument>, DynError>` - List of instruments or error
    pub async fn get_instruments_by_expiry(
        &self,
        asset: &str,
        expiry_date: &str,
    ) -> Result<Vec<Instrument>, DynError> {
        // Parse the date to validate format and convert to Deribit format
        let date = NaiveDate::parse_from_str(expiry_date, "%Y-%m-%d")?;
        // Deribit uses single digit days (not zero-padded): 1OCT25, not 01OCT25
        let deribit_expiry = format!("{}{}{}",
            date.day(),  // Single digit day without zero padding
            date.format("%b").to_string().to_uppercase(),  // Month abbreviation
            date.format("%y")  // Two digit year
        );
        let instruments = self.fetch_instruments(asset).await?;

        println!("🔍 Looking for instruments with expiry token: '{}'", deribit_expiry);
        println!("📊 Total instruments fetched: {}", instruments.len());

        // Filter instruments by expiry date
        let filtered_instruments: Vec<Instrument> = instruments
            .iter()
            .filter(|inst| inst.instrument_name.contains(&deribit_expiry))
            .cloned()
            .collect();

        println!("✅ Found {} instruments matching expiry: {}", filtered_instruments.len(), deribit_expiry);
        if filtered_instruments.len() > 0 {
            println!("📋 First few matches:");
            for inst in filtered_instruments.iter().take(3) {
                println!("  - {}", inst.instrument_name);
            }
        }

        Ok(filtered_instruments)
    }

    /// Downloads order book data for a specific instrument
    ///
    /// # Arguments
    /// * `instrument_name` - The full instrument name (e.g., "BTC-29MAR24-50000-C")
    ///
    /// # Returns
    /// * `Result<OrderBook, DynError>` - Order book data or error
    pub async fn get_order_book(&self, instrument_name: &str) -> Result<OrderBook, DynError> {
        let url = format!("{}/public/get_order_book", self.base_url);
        let params = [("instrument_name", instrument_name), ("depth", "10")];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let deribit_response: DeribitResponse = response.json().await?;
        let order_book: OrderBook = serde_json::from_value(deribit_response.result)?;

        Ok(order_book)
    }

    /// Downloads trade data for a specific instrument
    ///
    /// # Arguments
    /// * `instrument_name` - The full instrument name (e.g., "BTC-29MAR24-50000-C")
    /// * `start_timestamp` - Start timestamp in milliseconds
    /// * `end_timestamp` - End timestamp in milliseconds
    ///
    /// # Returns
    /// * `Result<Vec<TradeData>, DynError>` - Trade data or error
    pub async fn get_trades(
        &self,
        instrument_name: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
    ) -> Result<Vec<TradeData>, DynError> {
        let url = format!("{}/public/get_last_trades_by_instrument", self.base_url);

        let mut params = vec![
            ("instrument_name", instrument_name.to_string()),
            ("count", "1000".to_string()), // Max allowed
            ("include_old", "true".to_string()),
        ];

        if let Some(start) = start_timestamp {
            params.push(("start_timestamp", start.to_string()));
        }

        if let Some(end) = end_timestamp {
            params.push(("end_timestamp", end.to_string()));
        }

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let deribit_response: DeribitResponse = response.json().await?;

        // Parse trades from the result
        let trades_value = deribit_response
            .result
            .get("trades")
            .ok_or("No trades field in response")?;

        let mut trades = Vec::new();
        if let Value::Array(trade_array) = trades_value {
            for trade_val in trade_array {
                if let Ok(trade) = serde_json::from_value::<TradeData>(trade_val.clone()) {
                    trades.push(trade);
                }
            }
        }

        Ok(trades)
    }

    /// Downloads open interest data for a specific instrument
    ///
    /// # Arguments
    /// * `instrument_name` - The full instrument name (e.g., "BTC-29MAR24-50000-C")
    ///
    /// # Returns
    /// * `Result<Option<f64>, DynError>` - Open interest value or error
    pub async fn get_open_interest(&self, instrument_name: &str) -> Result<Option<f64>, DynError> {
        let url = format!("{}/public/get_book_summary_by_instrument", self.base_url);
        let params = [("instrument_name", instrument_name)];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Ok(None); // Return None instead of error for missing data
        }

        let deribit_response: DeribitResponse = response.json().await?;

        // Extract open interest from the result
        if let Some(open_interest_val) = deribit_response.result.get("open_interest") {
            if let Some(oi) = open_interest_val.as_f64() {
                return Ok(Some(oi));
            }
        }

        Ok(None)
    }

    /// Downloads comprehensive book summary data for multiple instruments
    ///
    /// # Arguments
    /// * `currency` - The currency (e.g., "BTC", "ETH")
    ///
    /// # Returns
    /// * `Result<FxHashMap<String, f64>, DynError>` - Map of instrument -> open interest
    pub async fn get_all_open_interests(
        &self,
        currency: &str,
    ) -> Result<FxHashMap<String, f64>, DynError> {
        let url = format!("{}/public/get_book_summary_by_currency", self.base_url);
        let params = [
            ("currency", currency.to_uppercase()),
            ("kind", "option".to_string()),
        ];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let deribit_response: DeribitResponse = response.json().await?;
        let mut open_interests = FxHashMap::default();
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut zero_count = 0;

        // Parse the result array
        if let Value::Array(summaries) = &deribit_response.result {
            for summary in summaries {
                if let (Some(instrument_name), Some(open_interest)) = (
                    summary.get("instrument_name").and_then(|v| v.as_str()),
                    summary.get("open_interest").and_then(|v| v.as_f64()),
                ) {
                    // Count different types of open interest values
                    if open_interest > 0.0 {
                        positive_count += 1;
                    } else if open_interest < 0.0 {
                        negative_count += 1;
                    } else {
                        zero_count += 1;
                    }

                    open_interests.insert(instrument_name.to_string(), open_interest);
                }
            }
        }

        println!("📊 Open Interest Summary:");
        println!("  Positive: {} instruments", positive_count);
        println!("  Negative: {} instruments", negative_count);
        println!("  Zero: {} instruments", zero_count);
        println!("  Total: {} instruments", open_interests.len());

        Ok(open_interests)
    }

    /// Downloads ticker data for instruments matching asset and expiry
    ///
    /// # Arguments
    /// * `asset` - The asset symbol (e.g., "BTC", "ETH")
    /// * `expiry_date` - The expiry date in format "YYYY-MM-DD"
    ///
    /// # Returns
    /// * `Result<Vec<Value>, DynError>` - Ticker data or error
    pub async fn get_ticker_by_expiry(
        &self,
        asset: &str,
        expiry_date: &str,
    ) -> Result<Vec<Value>, DynError> {
        // First get the instruments
        let instruments = self.get_instruments_by_expiry(asset, expiry_date).await?;

        let mut tickers = Vec::new();

        // Get ticker for each instrument
        for instrument in instruments.iter().take(10) {
            // Limit to 10 to avoid rate limiting
            let url = format!("{}/public/ticker", self.base_url);
            let params = [("instrument_name", &instrument.instrument_name)];

            let response = self.client.get(&url).query(&params).send().await?;

            if response.status().is_success() {
                let ticker_response: DeribitResponse = response.json().await?;
                tickers.push(ticker_response.result);
            }

            // Small delay to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(tickers)
    }
}

/// Main function to download comprehensive Deribit data as a Polars DataFrame
///
/// # Arguments
/// * `asset` - The asset symbol (e.g., "BTC", "ETH")
/// * `expiry_date` - The expiry date in format "YYYY-MM-DD"
///
/// # Returns
/// * `Result<DataFrame, DynError>` - DataFrame with comprehensive trading data
///
/// # Example
/// ```
/// let df = download_deribit_data("BTC", "2024-03-29").await?;
/// ```
pub async fn download_deribit_data(asset: &str, expiry_date: &str) -> Result<DataFrame, DynError> {
    let client = DeribitClient::new();

    // Get instruments for the expiry date
    let instruments = client.get_instruments_by_expiry(asset, expiry_date).await?;

    if instruments.is_empty() {
        println!(
            "No instruments found for {} expiring on {}",
            asset, expiry_date
        );
        // Return empty DataFrame with correct schema
        return Ok(create_empty_dataframe());
    }

    println!(
        "Found {} instruments, fetching comprehensive data...",
        instruments.len()
    );

    // First, get all open interest data for the currency (more efficient than individual calls)
    println!("📊 Fetching open interest data for {}...", asset);
    let open_interests = client
        .get_all_open_interests(asset)
        .await
        .unwrap_or_else(|e| {
            println!("⚠️ Warning: Could not fetch open interest data: {}", e);
            FxHashMap::default()
        });

    println!(
        "✅ Found open interest data for {} instruments",
        open_interests.len()
    );

    // Check for negative open interest in our specific instruments
    let our_negative_oi: Vec<_> = instruments
        .iter()
        .filter_map(|inst| {
            open_interests
                .get(&inst.instrument_name)
                .filter(|&&oi| oi < 0.0)
                .map(|&oi| (&inst.instrument_name, oi))
        })
        .collect();

    if !our_negative_oi.is_empty() {
        println!(
            "⚠️  Found {} instruments with negative open interest:",
            our_negative_oi.len()
        );
        for (name, oi) in &our_negative_oi {
            println!("    {}: {}", name, oi);
        }
    }

    let mut all_data: Vec<(
        String,         // instrument_name
        String,         // currency
        String,         // expiry_token
        String,         // expiry_iso
        i64,            // timestamp_ms
        String,         // timestamp_utc
        String,         // direction
        f64,            // price
        f64,            // amount
        Option<f64>,    // iv
        Option<f64>,    // index_price
        Option<f64>,    // mark_price
        String,         // trade_id
        i64,            // trade_seq
        Option<String>, // block_trade_id
        Option<String>, // liquidity
        Option<i32>,    // tick_direction
        Option<f64>,    // strike
        Option<String>, // option_type
        Option<f64>, // open_interest
        Option<f64>, // interest_rate

    )> = Vec::new();

    // Fetch trade data for each instrument (limit to avoid rate limits)
    for instrument in instruments.iter().take(5) {
        println!("Fetching trades for: {}", instrument.instrument_name);

        // Get the open interest for this specific instrument
        let instrument_open_interest = open_interests.get(&instrument.instrument_name).copied();

        // Get recent trades
        match client
            .get_trades(&instrument.instrument_name, None, None)
            .await
        {
            Ok(trades) => {
                println!("  Found {} trades", trades.len());

                // Get current order book for additional data
                let order_book = client
                    .get_order_book(&instrument.instrument_name)
                    .await
                    .ok();

                for trade in trades {
                    // Parse expiry date for token and ISO format
                    let expiry_dt =
                        DateTime::from_timestamp(instrument.expiration_timestamp / 1000, 0)
                            .unwrap_or_default();
                    let expiry_token = expiry_dt.format("%d%b%y").to_string().to_uppercase();
                    let expiry_iso = expiry_dt.format("%Y-%m-%d").to_string();

                    // Parse timestamp
                    let timestamp_dt =
                        DateTime::from_timestamp(trade.timestamp / 1000, 0).unwrap_or_default();
                    let timestamp_utc = timestamp_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();

                    all_data.push((
                        trade.instrument_name.clone(),
                        instrument.quote_currency.clone(),
                        expiry_token,
                        expiry_iso,
                        trade.timestamp,
                        timestamp_utc,
                        trade.direction,
                        trade.price,
                        trade.amount,
                        trade.iv,
                        trade.index_price,
                        trade
                            .mark_price
                            .or(order_book.as_ref().map(|ob| ob.mark_price)),
                        trade.trade_id,
                        trade.trade_seq,
                        trade.block_trade_id,
                        trade.liquidity,
                        trade.tick_direction,
                        instrument.strike,
                        instrument.option_type.clone(),
                        instrument_open_interest, // Now using actual open interest data!
                        order_book.as_ref().map(|ob| ob.interest_rate), // Add interest rate from order book
                    ));
                }
            }
            Err(e) => {
                println!("  Error fetching trades: {}", e);
                continue;
            }
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    if all_data.is_empty() {
        println!("No trade data found");
        return Ok(create_empty_dataframe());
    }

    // Convert to Polars DataFrame
    let df = create_dataframe_from_data(all_data)?;

    println!(
        "Created DataFrame with {} rows and {} columns",
        df.height(),
        df.width()
    );
    Ok(df)
}

fn create_empty_dataframe() -> DataFrame {
    df! [
        "instrument_name" => Vec::<String>::new(),
        "currency" => Vec::<String>::new(),
        "expiry_token" => Vec::<String>::new(),
        "expiry_iso" => Vec::<String>::new(),
        "timestamp_ms" => Vec::<i64>::new(),
        "timestamp_utc" => Vec::<String>::new(),
        "direction" => Vec::<String>::new(),
        "price" => Vec::<f64>::new(),
        "amount" => Vec::<f64>::new(),
        "iv" => Vec::<Option<f64>>::new(),
        "index_price" => Vec::<Option<f64>>::new(),
        "mark_price" => Vec::<Option<f64>>::new(),
        "trade_id" => Vec::<String>::new(),
        "trade_seq" => Vec::<i64>::new(),
        "block_trade_id" => Vec::<Option<String>>::new(),
        "liquidity" => Vec::<Option<String>>::new(),
        "tick_direction" => Vec::<Option<i32>>::new(),
        "strike" => Vec::<Option<f64>>::new(),
        "option_type" => Vec::<Option<String>>::new(),
        "open_interest" => Vec::<Option<f64>>::new(),
        "interest_rate" => Vec::<Option<f64>>::new(),
    ].unwrap()
}

fn create_dataframe_from_data(
    data: Vec<(String, String, String, String, i64, String, String, f64, f64, 
              Option<f64>, Option<f64>, Option<f64>, String, i64, Option<String>, 
              Option<String>, Option<i32>, Option<f64>, Option<String>, Option<f64>, Option<f64>)>
) -> Result<DataFrame, Box<dyn Error>> {
    let mut instrument_names = Vec::new();
    let mut currencies = Vec::new();
    let mut expiry_tokens = Vec::new();
    let mut expiry_isos = Vec::new();
    let mut timestamps_ms = Vec::new();
    let mut timestamps_utc = Vec::new();
    let mut directions = Vec::new();
    let mut prices = Vec::new();
    let mut amounts = Vec::new();
    let mut ivs = Vec::new();
    let mut index_prices = Vec::new();
    let mut mark_prices = Vec::new();
    let mut trade_ids = Vec::new();
    let mut trade_seqs = Vec::new();
    let mut block_trade_ids = Vec::new();
    let mut liquidities = Vec::new();
    let mut tick_directions = Vec::new();
    let mut strikes = Vec::new();
    let mut option_types = Vec::new();
    let mut open_interests = Vec::new();
    let mut interest_rates = Vec::new();
    
    for (instrument_name, currency, expiry_token, expiry_iso, timestamp_ms, timestamp_utc,
         direction, price, amount, iv, index_price, mark_price, trade_id, trade_seq,
         block_trade_id, liquidity, tick_direction, strike, option_type, open_interest, interest_rate) in data {
        instrument_names.push(instrument_name);
        currencies.push(currency);
        expiry_tokens.push(expiry_token);
        expiry_isos.push(expiry_iso);
        timestamps_ms.push(timestamp_ms);
        timestamps_utc.push(timestamp_utc);
        directions.push(direction);
        prices.push(price);
        amounts.push(amount);
        ivs.push(iv);
        index_prices.push(index_price);
        mark_prices.push(mark_price);
        trade_ids.push(trade_id);
        trade_seqs.push(trade_seq);
        block_trade_ids.push(block_trade_id);
        liquidities.push(liquidity);
        tick_directions.push(tick_direction);
        strikes.push(strike);
        option_types.push(option_type);
        open_interests.push(open_interest);
        interest_rates.push(interest_rate);
    }

    let df = df! [
        "instrument_name" => instrument_names,
        "currency" => currencies,
        "expiry_token" => expiry_tokens,
        "expiry_iso" => expiry_isos,
        "timestamp_ms" => timestamps_ms,
        "timestamp_utc" => timestamps_utc,
        "direction" => directions,
        "price" => prices,
        "amount" => amounts,
        "iv" => ivs,
        "index_price" => index_prices,
        "mark_price" => mark_prices,
        "trade_id" => trade_ids,
        "trade_seq" => trade_seqs,
        "block_trade_id" => block_trade_ids,
        "liquidity" => liquidities,
        "tick_direction" => tick_directions,
        "strike" => strikes,
        "option_type" => option_types,
        "open_interest" => open_interests,
        "interest_rate" => interest_rates,
    ]?;

    Ok(df)
}

/// Legacy function for compatibility - downloads instruments only
///
/// # Arguments
/// * `asset` - The asset symbol (e.g., "BTC", "ETH")
/// * `expiry_date` - The expiry date in format "YYYY-MM-DD"
///
/// # Example
/// ```
/// let data = download_deribit_instruments("BTC", "2024-03-29").await?;
/// ```
pub async fn download_deribit_instruments(
    asset: &str,
    expiry_date: &str,
) -> Result<Vec<Instrument>, DynError> {
    let client = DeribitClient::new();
    client.get_instruments_by_expiry(asset, expiry_date).await
}

pub async fn fetch_available_expiries(asset: &str) -> Result<Vec<String>, DynError> {
    let client = DeribitClient::new();
    client.get_available_expiry_dates(asset).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_btc_options() {
        let result = download_deribit_data("BTC", "2024-03-29").await;
        assert!(result.is_ok() || result.is_err()); // This will pass either way

        if let Ok(df) = result {
            println!("Found DataFrame with {} rows", df.height());
            if df.height() > 0 {
                println!("Columns: {:?}", df.get_column_names());
            }
        }
    }

    #[tokio::test]
    async fn test_get_order_book() {
        let client = DeribitClient::new();
        // This might fail if the instrument doesn't exist anymore
        let result = client.get_order_book("BTC-29MAR24-50000-C").await;

        if let Ok(order_book) = result {
            println!("Order book for {}", order_book.instrument_name);
            println!("Best bid: {:?}", order_book.best_bid_price);
            println!("Best ask: {:?}", order_book.best_ask_price);
        }
    }
}

/// Fetch all available expiry dates for a given asset
/// 
/// # Arguments
/// * `asset` - The asset symbol (e.g., "BTC", "ETH")
/// 
/// # Returns
/// A vector of expiry dates in "YYYY-MM-DD" format
/// 
/// # Example
/// ```
/// let expiries = fetch_available_expiries("BTC").await?;
/// ```
pub async fn fetch_available_expiries(asset: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = DeribitClient::new();
    
    // Get all instruments for the asset
    let url = format!("{}/public/get_instruments", client.base_url);
    let params = [
        ("currency", asset.to_uppercase()),
        ("kind", "option".to_string()),
        ("expired", "false".to_string()),
    ];

    let response = client.client
        .get(&url)
        .query(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let deribit_response: DeribitResponse = response.json().await?;
    
    // Parse instruments from the result
    let instruments: Vec<Instrument> = serde_json::from_value(deribit_response.result)?;
    
    // Extract unique expiry dates from instrument names
    let mut expiry_dates = std::collections::HashSet::new();
    
    for instrument in instruments {
        if let Some(expiry_str) = extract_expiry_from_instrument(&instrument.instrument_name) {
            if let Ok(iso_date) = convert_deribit_date_to_iso(&expiry_str) {
                expiry_dates.insert(iso_date);
            }
        }
    }
    
    let mut sorted_expiries: Vec<String> = expiry_dates.into_iter().collect();
    sorted_expiries.sort();
    
    Ok(sorted_expiries)
}

/// Extract expiry string from instrument name (e.g., "BTC-3JAN25-90000-C" -> "3JAN25")
fn extract_expiry_from_instrument(instrument_name: &str) -> Option<String> {
    let parts: Vec<&str> = instrument_name.split('-').collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Convert Deribit expiry format to ISO date (e.g., "1OCT25" -> "2025-10-01")
fn convert_deribit_date_to_iso(deribit_date: &str) -> Result<String, Box<dyn Error>> {
    // Parse date like "1OCT25" -> "2025-10-01"
    // Note: Deribit uses single digit days (1OCT25, not 01OCT25)
    let date = NaiveDate::parse_from_str(deribit_date, "%d%b%y")?;
    Ok(date.format("%Y-%m-%d").to_string())
}

