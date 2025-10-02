use actix_web::{web, HttpResponse, Result};

use crate::api_client::DeribitClient;
use crate::filters::{filter_by_creation_date, print_sample_instruments};
use crate::types::OptionsRequest;

/// Handler for GET /api/options
/// Fetches options data from Deribit with server-side creation date filtering only
pub async fn get_options(query: web::Query<OptionsRequest>) -> Result<HttpResponse> {
    let currency = &query.currency;
    let kind = "option";
    let expired = query.include_expired.unwrap_or(false);

    // Log query parameters
    println!("=== Query Parameters ===");
    println!("currency: {}", currency);
    println!("creation_from: {:?}", query.creation_from);
    println!("creation_to: {:?}", query.creation_to);
    println!("include_expired: {}", expired);
    println!("========================");

    // Create API client
    let client = DeribitClient::new();

    // Fetch instruments from Deribit
    let instruments = match client.fetch_instruments(currency, kind, expired).await {
        Ok(insts) => insts,
        Err(e) => {
            println!("Error fetching instruments: {}", e);
            return Ok(HttpResponse::InternalServerError().body(e));
        }
    };

    println!("Found {} instruments, applying filters...", instruments.len());

    // Show sample instruments for debugging
    print_sample_instruments(&instruments, 5);

    // Apply server-side creation date filter only
    let filtered_instruments = filter_by_creation_date(
        instruments,
        query.creation_from.as_ref(),
        query.creation_to.as_ref(),
    );

    println!(
        "After server-side filtering: {} instruments (client-side filtering will be applied in browser)",
        filtered_instruments.len()
    );

    // Limit to 100 instruments to avoid too many API calls
    let limited_instruments: Vec<_> = filtered_instruments.into_iter().take(100).collect();

    // Enrich with ticker data (OI, Volume, IV, Prices)
    let enriched_data = client.enrich_instruments(limited_instruments).await;

    println!(
        "Successfully fetched data for {} instruments (all filtering except creation date will be done client-side)",
        enriched_data.len()
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "result": enriched_data
    })))
}
