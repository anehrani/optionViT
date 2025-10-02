use reqwest::Client;
use serde_json::Value;
use futures::future::join_all;

/// Deribit API client for fetching options data
pub struct DeribitClient {
    client: Client,
}

impl DeribitClient {
    /// Create a new Deribit API client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch instruments from Deribit API
    pub async fn fetch_instruments(
        &self,
        currency: &str,
        kind: &str,
        expired: bool,
    ) -> Result<Vec<Value>, String> {
        let url = format!(
            "https://www.deribit.com/api/v2/public/get_instruments?currency={}&kind={}&expired={}",
            currency, kind, expired
        );

        println!("Fetching instruments for {} (expired: {})", currency, expired);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        let data: Value = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        if let Some(result) = data.get("result") {
            if let Some(instruments) = result.as_array() {
                return Ok(instruments.clone());
            }
        }

        Ok(vec![])
    }

    /// Fetch ticker data for a single instrument
    pub async fn fetch_ticker(&self, instrument_name: &str) -> Option<Value> {
        let ticker_url = format!(
            "https://www.deribit.com/api/v2/public/ticker?instrument_name={}",
            instrument_name
        );

        if let Ok(ticker_resp) = self.client.get(&ticker_url).send().await {
            if let Ok(ticker_data) = ticker_resp.json::<Value>().await {
                return ticker_data.get("result").cloned();
            }
        }

        None
    }

    /// Enrich instruments with ticker data (OI, Volume, IV, Prices)
    pub async fn enrich_instruments(&self, instruments: Vec<Value>) -> Vec<Value> {
        let mut futures = Vec::new();

        for instrument in instruments.iter() {
            if let Some(instrument_name) = instrument.get("instrument_name").and_then(|v| v.as_str())
            {
                let client_clone = self.client.clone();
                let instrument_name_owned = instrument_name.to_string();
                let instrument_clone = instrument.clone();

                let future = async move {
                    let ticker_url = format!(
                        "https://www.deribit.com/api/v2/public/ticker?instrument_name={}",
                        instrument_name_owned
                    );

                    let mut combined = instrument_clone;

                    if let Ok(ticker_resp) = client_clone.get(&ticker_url).send().await {
                        if let Ok(ticker_data) = ticker_resp.json::<Value>().await {
                            if let Some(ticker_result) = ticker_data.get("result") {
                                if let Some(obj) = combined.as_object_mut() {
                                    // Add open interest
                                    if let Some(oi) = ticker_result.get("open_interest") {
                                        obj.insert("open_interest".to_string(), oi.clone());
                                    }
                                    // Add 24h volume
                                    if let Some(stats) = ticker_result.get("stats") {
                                        if let Some(volume) = stats.get("volume") {
                                            obj.insert("volume_24h".to_string(), volume.clone());
                                        }
                                    }
                                    // Add mark IV
                                    if let Some(iv) = ticker_result.get("mark_iv") {
                                        obj.insert("mark_iv".to_string(), iv.clone());
                                    }
                                    // Add prices
                                    if let Some(bid) = ticker_result.get("best_bid_price") {
                                        obj.insert("bid_price".to_string(), bid.clone());
                                    }
                                    if let Some(ask) = ticker_result.get("best_ask_price") {
                                        obj.insert("ask_price".to_string(), ask.clone());
                                    }
                                    if let Some(last) = ticker_result.get("last_price") {
                                        obj.insert("last_price".to_string(), last.clone());
                                    }
                                    if let Some(mark_price) = ticker_result.get("mark_price") {
                                        obj.insert("mark_price".to_string(), mark_price.clone());
                                    }
                                    // Add underlying price (spot price)
                                    if let Some(underlying) = ticker_result.get("underlying_price") {
                                        obj.insert("underlying_price".to_string(), underlying.clone());
                                    }
                                }
                            }
                        }
                    }

                    combined
                };

                futures.push(future);
            }
        }

        // Execute all futures concurrently
        join_all(futures).await
    }
}

impl Default for DeribitClient {
    fn default() -> Self {
        Self::new()
    }
}
