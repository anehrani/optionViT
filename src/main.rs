use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_cors::Cors;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures::future::join_all;

#[derive(Debug, Serialize, Deserialize)]
struct OptionsRequest {
    currency: String,
    creation_from: Option<String>,
    creation_to: Option<String>,
    expiry_from: Option<String>,
    expiry_to: Option<String>,
    oi_min: Option<f64>,
    oi_max: Option<f64>,
    option_type: Option<String>,
    include_expired: Option<bool>,
}

async fn get_options(query: web::Query<OptionsRequest>) -> Result<HttpResponse> {
    let currency = &query.currency;
    let kind = "option";
    let expired = query.include_expired.unwrap_or(false);
    
    // Debug: Print all query parameters
    println!("=== Query Parameters ===");
    println!("currency: {}", currency);
    println!("creation_from: {:?}", query.creation_from);
    println!("creation_to: {:?}", query.creation_to);
    println!("expiry_from: {:?}", query.expiry_from);
    println!("expiry_to: {:?}", query.expiry_to);
    println!("oi_min: {:?}", query.oi_min);
    println!("oi_max: {:?}", query.oi_max);
    println!("option_type: {:?}", query.option_type);
    println!("include_expired: {}", expired);
    println!("========================");
    
    // Fetch instruments from Deribit
    let url = format!(
        "https://www.deribit.com/api/v2/public/get_instruments?currency={}&kind={}&expired={}",
        currency, kind, expired
    );
    
    let client = reqwest::Client::new();
    
    println!("Fetching instruments for {} (expired: {})", currency, expired);
    
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            println!("Error fetching instruments: {}", e);
            return Ok(HttpResponse::InternalServerError().body(format!("Request error: {}", e)));
        }
    };
    
    let data: Value = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            println!("Error parsing instruments JSON: {}", e);
            return Ok(HttpResponse::InternalServerError().body(format!("Parse error: {}", e)));
        }
    };
    
    if let Some(result) = data.get("result") {
        if let Some(instruments) = result.as_array() {
            println!("Found {} instruments, applying filters...", instruments.len());
            
            // Debug: Show first few expiration dates
            for (i, inst) in instruments.iter().take(5).enumerate() {
                if let Some(name) = inst.get("instrument_name").and_then(|v| v.as_str()) {
                    if let Some(exp_ts) = inst.get("expiration_timestamp").and_then(|v| v.as_i64()) {
                        let exp_date = chrono::NaiveDateTime::from_timestamp_millis(exp_ts);
                        println!("Sample instrument {}: {} expires at {} (ts: {})", i, name, 
                            exp_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or("N/A".to_string()), 
                            exp_ts);
                    }
                }
            }
            
            // Apply filters
            let mut filtered_instruments: Vec<Value> = instruments.clone();
            
            // Filter by option type (call/put)
            if let Some(opt_type) = &query.option_type {
                if !opt_type.is_empty() && opt_type != "all" {
                    filtered_instruments.retain(|inst| {
                        if let Some(instrument_name) = inst.get("instrument_name").and_then(|v| v.as_str()) {
                            let is_call = instrument_name.ends_with("-C");
                            let is_put = instrument_name.ends_with("-P");
                            match opt_type.as_str() {
                                "call" => is_call,
                                "put" => is_put,
                                _ => true,
                            }
                        } else {
                            false
                        }
                    });
                }
            }
            
            // Filter by expiration date
            if let Some(expiry_from) = &query.expiry_from {
                if !expiry_from.is_empty() {
                    if let Ok(from_timestamp) = chrono::NaiveDate::parse_from_str(expiry_from, "%Y-%m-%d") {
                        let from_ts = from_timestamp.and_hms_opt(0, 0, 0).unwrap().timestamp_millis();
                        println!("Filtering expiry_from: {} -> timestamp: {}", expiry_from, from_ts);
                        let count_before = filtered_instruments.len();
                        filtered_instruments.retain(|inst| {
                            if let Some(exp_ts) = inst.get("expiration_timestamp").and_then(|v| v.as_i64()) {
                                exp_ts >= from_ts
                            } else {
                                // Remove instruments without expiration_timestamp (shouldn't happen for options)
                                false
                            }
                        });
                        println!("After expiry_from filter: {} -> {} instruments", count_before, filtered_instruments.len());
                    }
                }
            }
            
            if let Some(expiry_to) = &query.expiry_to {
                if !expiry_to.is_empty() {
                    if let Ok(to_timestamp) = chrono::NaiveDate::parse_from_str(expiry_to, "%Y-%m-%d") {
                        let to_ts = to_timestamp.and_hms_opt(23, 59, 59).unwrap().timestamp_millis();
                        println!("Filtering expiry_to: {} -> timestamp: {}", expiry_to, to_ts);
                        let count_before = filtered_instruments.len();
                        filtered_instruments.retain(|inst| {
                            if let Some(exp_ts) = inst.get("expiration_timestamp").and_then(|v| v.as_i64()) {
                                exp_ts <= to_ts
                            } else {
                                // Remove instruments without expiration_timestamp (shouldn't happen for options)
                                false
                            }
                        });
                        println!("After expiry_to filter: {} -> {} instruments", count_before, filtered_instruments.len());
                    }
                }
            }
            
            // Filter by creation timestamp (optional - only filter if explicitly set)
            if let Some(creation_from) = &query.creation_from {
                if !creation_from.is_empty() {
                    if let Ok(from_timestamp) = chrono::NaiveDate::parse_from_str(creation_from, "%Y-%m-%d") {
                        let from_ts = from_timestamp.and_hms_opt(0, 0, 0).unwrap().timestamp_millis();
                        println!("Filtering creation_from: {} -> timestamp: {}", creation_from, from_ts);
                        let count_before = filtered_instruments.len();
                        filtered_instruments.retain(|inst| {
                            if let Some(create_ts) = inst.get("creation_timestamp").and_then(|v| v.as_i64()) {
                                create_ts >= from_ts
                            } else {
                                // Keep instruments without creation_timestamp when filtering
                                true
                            }
                        });
                        println!("After creation_from filter: {} -> {} instruments", count_before, filtered_instruments.len());
                    }
                }
            }
            
            if let Some(creation_to) = &query.creation_to {
                if !creation_to.is_empty() {
                    if let Ok(to_timestamp) = chrono::NaiveDate::parse_from_str(creation_to, "%Y-%m-%d") {
                        let to_ts = to_timestamp.and_hms_opt(23, 59, 59).unwrap().timestamp_millis();
                        println!("Filtering creation_to: {} -> timestamp: {}", creation_to, to_ts);
                        let count_before = filtered_instruments.len();
                        filtered_instruments.retain(|inst| {
                            if let Some(create_ts) = inst.get("creation_timestamp").and_then(|v| v.as_i64()) {
                                create_ts <= to_ts
                            } else {
                                // Keep instruments without creation_timestamp when filtering
                                true
                            }
                        });
                        println!("After creation_to filter: {} -> {} instruments", count_before, filtered_instruments.len());
                    }
                }
            }
            
            println!("After filtering: {} instruments", filtered_instruments.len());
            
            // Limit to avoid too many API calls
            let limited_instruments: Vec<Value> = filtered_instruments.into_iter().take(100).collect();
            
            // Create futures for fetching ticker data
            let mut futures = Vec::new();
            
            for instrument in limited_instruments.iter() {
                if let Some(instrument_name) = instrument.get("instrument_name").and_then(|v| v.as_str()) {
                    let client_clone = client.clone();
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
                                        if let Some(oi) = ticker_result.get("open_interest") {
                                            obj.insert("open_interest".to_string(), oi.clone());
                                        }
                                        if let Some(stats) = ticker_result.get("stats") {
                                            if let Some(volume) = stats.get("volume") {
                                                obj.insert("volume_24h".to_string(), volume.clone());
                                            }
                                        }
                                        if let Some(iv) = ticker_result.get("mark_iv") {
                                            obj.insert("mark_iv".to_string(), iv.clone());
                                        }
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
            let mut enriched_data = join_all(futures).await;
            
            // Filter by open interest range (optional - only filter if explicitly set)
            if let Some(oi_min) = query.oi_min {
                println!("Filtering OI minimum: {}", oi_min);
                let count_before = enriched_data.len();
                enriched_data.retain(|inst| {
                    if let Some(oi) = inst.get("open_interest").and_then(|v| v.as_f64()) {
                        oi >= oi_min
                    } else {
                        // Keep instruments without OI data when filtering by minimum
                        true
                    }
                });
                println!("After OI min filter: {} -> {} instruments", count_before, enriched_data.len());
            }
            
            if let Some(oi_max) = query.oi_max {
                println!("Filtering OI maximum: {}", oi_max);
                let count_before = enriched_data.len();
                enriched_data.retain(|inst| {
                    if let Some(oi) = inst.get("open_interest").and_then(|v| v.as_f64()) {
                        oi <= oi_max
                    } else {
                        // Keep instruments without OI data when filtering by maximum
                        true
                    }
                });
                println!("After OI max filter: {} -> {} instruments", count_before, enriched_data.len());
            }
            
            println!("Successfully fetched data for {} instruments", enriched_data.len());
            
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "result": enriched_data
            })));
        }
    }
    
    println!("No instruments found in response");
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "result": []
    })))
}

async fn index() -> Result<HttpResponse> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Deribit Options Data Fetcher</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 50%, #7e22ce 100%);
            min-height: 100vh;
            padding: 20px;
            position: relative;
            overflow-x: hidden;
        }
        
        body::before {
            content: '';
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: 
                radial-gradient(circle at 20% 50%, rgba(120, 119, 198, 0.3), transparent 50%),
                radial-gradient(circle at 80% 80%, rgba(138, 43, 226, 0.3), transparent 50%),
                radial-gradient(circle at 40% 20%, rgba(72, 149, 239, 0.2), transparent 50%);
            z-index: 0;
            animation: gradientShift 15s ease infinite;
        }
        
        @keyframes gradientShift {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.8; }
        }
        
        .container {
            max-width: 1800px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.98);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            box-shadow: 0 25px 80px rgba(0, 0, 0, 0.4), 0 0 1px rgba(0, 0, 0, 0.1);
            overflow: hidden;
            position: relative;
            z-index: 1;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }
        
        .header {
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 50%, #7e22ce 100%);
            color: white;
            padding: 40px 30px;
            text-align: center;
            position: relative;
            overflow: hidden;
        }
        
        .header::before {
            content: '';
            position: absolute;
            top: -50%;
            left: -50%;
            width: 200%;
            height: 200%;
            background: radial-gradient(circle, rgba(255, 255, 255, 0.1) 0%, transparent 70%);
            animation: headerPulse 8s ease-in-out infinite;
        }
        
        @keyframes headerPulse {
            0%, 100% { transform: translate(0, 0); }
            50% { transform: translate(10%, 10%); }
        }
        
        .header h1 {
            font-size: 36px;
            margin-bottom: 12px;
            font-weight: 700;
            position: relative;
            z-index: 1;
            text-shadow: 0 2px 20px rgba(0, 0, 0, 0.3);
            letter-spacing: -0.5px;
        }
        
        .header p {
            opacity: 0.95;
            font-size: 15px;
            position: relative;
            z-index: 1;
            font-weight: 400;
            letter-spacing: 0.3px;
        }
        
        .fetch-section {
            padding: 35px;
            background: linear-gradient(135deg, #f8f9ff 0%, #f0f4ff 100%);
            border-bottom: 1px solid rgba(30, 60, 114, 0.1);
            position: relative;
        }
        
        .fetch-section::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
            background: linear-gradient(90deg, #1e3c72, #2a5298, #7e22ce);
        }
        
        .filters-title {
            font-size: 20px;
            font-weight: 700;
            color: #1e3c72;
            margin-bottom: 25px;
            padding-bottom: 12px;
            border-bottom: 3px solid transparent;
            background: linear-gradient(white, white) padding-box,
                        linear-gradient(90deg, #1e3c72, #7e22ce) border-box;
            border-bottom: 3px solid;
            border-image: linear-gradient(90deg, #1e3c72, #7e22ce) 1;
            display: inline-block;
            padding-right: 20px;
        }
        
        .form-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 24px;
            margin-bottom: 25px;
        }
        
        @media (min-width: 1400px) {
            .form-grid {
                grid-template-columns: repeat(3, 1fr);
            }
        }
        
        .form-group {
            display: flex;
            flex-direction: column;
            position: relative;
        }
        
        .form-group label {
            font-weight: 600;
            margin-bottom: 10px;
            color: #2d3748;
            font-size: 13px;
            display: flex;
            align-items: center;
            gap: 6px;
        }
        
        .form-group select,
        .form-group input {
            padding: 13px 16px;
            border: 2px solid #e2e8f0;
            border-radius: 10px;
            font-size: 14px;
            transition: all 0.3s ease;
            background: white;
            font-family: inherit;
            width: 100%;
            box-sizing: border-box;
        }
        
        .form-group select:focus,
        .form-group input:focus {
            outline: none;
            border-color: #2a5298;
            box-shadow: 0 0 0 4px rgba(42, 82, 152, 0.1), 0 2px 8px rgba(0, 0, 0, 0.05);
            transform: translateY(-1px);
        }
        
        .form-group select:hover,
        .form-group input:hover {
            border-color: #cbd5e0;
        }
        
        .checkbox-group {
            display: flex;
            align-items: center;
            margin-top: 12px;
            padding: 12px 16px;
            background: white;
            border-radius: 10px;
            border: 2px solid #e2e8f0;
            transition: all 0.3s ease;
        }
        
        .checkbox-group:hover {
            border-color: #cbd5e0;
        }
        
        .checkbox-group input[type="checkbox"] {
            width: 22px;
            height: 22px;
            margin-right: 12px;
            cursor: pointer;
            accent-color: #2a5298;
        }
        
        .checkbox-group label {
            margin: 0;
            cursor: pointer;
            user-select: none;
            color: #2d3748;
            font-weight: 500;
        }
        
        .button-group {
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 15px;
            margin-top: 25px;
        }
        
        .btn-fetch {
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 50%, #7e22ce 100%);
            color: white;
            border: none;
            padding: 16px 44px;
            border-radius: 12px;
            font-size: 16px;
            font-weight: 700;
            cursor: pointer;
            transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
            box-shadow: 0 8px 24px rgba(30, 60, 114, 0.35), 0 2px 6px rgba(0, 0, 0, 0.1);
            position: relative;
            overflow: hidden;
            letter-spacing: 0.3px;
            white-space: nowrap;
        }
        
        .btn-fetch::before {
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 100%;
            height: 100%;
            background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
            transition: left 0.5s;
        }
        
        .btn-fetch:hover::before {
            left: 100%;
        }
        
        .btn-fetch:hover {
            transform: translateY(-3px);
            box-shadow: 0 12px 32px rgba(30, 60, 114, 0.45), 0 4px 12px rgba(0, 0, 0, 0.15);
        }
        
        .btn-fetch:active {
            transform: translateY(-1px);
            box-shadow: 0 6px 20px rgba(30, 60, 114, 0.35);
        }
        
        .btn-fetch:disabled {
            opacity: 0.6;
            cursor: not-allowed;
            transform: none;
            box-shadow: 0 4px 12px rgba(30, 60, 114, 0.2);
        }
        
        .btn-reset {
            background: linear-gradient(135deg, #64748b 0%, #475569 100%);
            color: white;
            border: none;
            padding: 16px 36px;
            border-radius: 12px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            box-shadow: 0 4px 12px rgba(100, 116, 139, 0.3);
            letter-spacing: 0.2px;
            white-space: nowrap;
        }
        
        .btn-reset:hover {
            background: linear-gradient(135deg, #475569 0%, #334155 100%);
            transform: translateY(-2px);
            box-shadow: 0 6px 18px rgba(100, 116, 139, 0.4);
        }
        
        .btn-reset:active {
            transform: translateY(0);
        }
        
        .loading {
            text-align: center;
            padding: 50px;
            color: #2a5298;
            font-size: 16px;
            background: linear-gradient(135deg, #f8f9ff 0%, #f0f4ff 100%);
            margin: 20px 30px;
            border-radius: 16px;
            box-shadow: inset 0 2px 8px rgba(30, 60, 114, 0.1);
        }
        
        .spinner {
            border: 4px solid rgba(42, 82, 152, 0.1);
            border-top: 4px solid #2a5298;
            border-radius: 50%;
            width: 50px;
            height: 50px;
            animation: spin 0.8s linear infinite;
            margin: 20px auto;
            box-shadow: 0 4px 12px rgba(42, 82, 152, 0.2);
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        .loading p {
            font-weight: 500;
        }
        
        .table-section {
            padding: 35px;
            overflow-x: auto;
            background: linear-gradient(to bottom, #ffffff 0%, #fafbff 100%);
        }
        
        .stats {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 18px;
            margin-bottom: 30px;
        }
        
        @media (min-width: 768px) {
            .stats {
                grid-template-columns: repeat(3, 1fr);
            }
        }
        
        @media (min-width: 1200px) {
            .stats {
                grid-template-columns: repeat(5, 1fr);
            }
        }
        
        .stat-card {
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 50%, #7e22ce 100%);
            color: white;
            padding: 20px 24px;
            border-radius: 14px;
            box-shadow: 0 8px 20px rgba(30, 60, 114, 0.25), 0 2px 6px rgba(0, 0, 0, 0.1);
            transition: all 0.3s ease;
            position: relative;
            overflow: hidden;
        }
        
        .stat-card::before {
            content: '';
            position: absolute;
            top: -50%;
            right: -50%;
            width: 200%;
            height: 200%;
            background: radial-gradient(circle, rgba(255, 255, 255, 0.15) 0%, transparent 70%);
            animation: statCardGlow 4s ease-in-out infinite;
        }
        
        @keyframes statCardGlow {
            0%, 100% { transform: translate(0, 0); }
            50% { transform: translate(-10%, -10%); }
        }
        
        .stat-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 12px 28px rgba(30, 60, 114, 0.35), 0 4px 10px rgba(0, 0, 0, 0.15);
        }
        
        .stat-card .label {
            font-size: 11px;
            opacity: 0.95;
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 1px;
            font-weight: 600;
            position: relative;
            z-index: 1;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        
        .stat-card .value {
            font-size: 26px;
            font-weight: 700;
            position: relative;
            z-index: 1;
            text-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        
        .table-wrapper {
            width: 100%;
            overflow-x: auto;
            border-radius: 12px;
            box-shadow: 0 4px 16px rgba(30, 60, 114, 0.1), 0 1px 3px rgba(0, 0, 0, 0.05);
        }
        
        table {
            width: 100%;
            min-width: 1200px;
            border-collapse: separate;
            border-spacing: 0;
            font-size: 13px;
            background: white;
        }
        
        thead {
            background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
            color: white;
            position: sticky;
            top: 0;
            z-index: 10;
            box-shadow: 0 2px 8px rgba(30, 60, 114, 0.2);
        }
        
        th {
            padding: 16px 18px;
            text-align: left;
            font-weight: 700;
            text-transform: uppercase;
            font-size: 11px;
            letter-spacing: 0.8px;
            cursor: pointer;
            user-select: none;
            white-space: nowrap;
            transition: all 0.2s ease;
            position: relative;
            min-width: 100px;
        }
        
        th::after {
            content: '⇅';
            opacity: 0.5;
            margin-left: 6px;
            font-size: 10px;
        }
        
        th:hover {
            background: rgba(255, 255, 255, 0.15);
        }
        
        th:active {
            background: rgba(255, 255, 255, 0.25);
        }
        
        .highlight-col {
            background: linear-gradient(135deg, #e0f2fe 0%, #dbeafe 100%) !important;
            font-weight: 600;
        }
        
        td {
            padding: 14px 18px;
            border-bottom: 1px solid #f1f5f9;
            min-width: 100px;
        }
        
        tbody tr {
            transition: all 0.2s ease;
            background: white;
        }
        
        tbody tr:nth-child(even) {
            background: #fafbff;
        }
        
        tbody tr:hover {
            background: linear-gradient(135deg, #f0f4ff 0%, #e8edff 100%);
            box-shadow: inset 0 0 0 2px rgba(42, 82, 152, 0.1);
            transform: scale(1.002);
        }
        
        .no-data {
            text-align: center;
            padding: 80px 40px;
            color: #64748b;
            font-size: 17px;
            background: linear-gradient(135deg, #f8f9ff 0%, #f0f4ff 100%);
            border-radius: 16px;
            box-shadow: inset 0 2px 8px rgba(30, 60, 114, 0.08);
            font-weight: 500;
        }
        
        .no-data::before {
            content: '📭';
            display: block;
            font-size: 48px;
            margin-bottom: 16px;
            animation: bounce 2s ease-in-out infinite;
        }
        
        @keyframes bounce {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-10px); }
        }
        
        .error {
            background: linear-gradient(135deg, #fee2e2 0%, #fecaca 100%);
            color: #991b1b;
            padding: 18px 22px;
            border-radius: 12px;
            margin: 20px 30px;
            border: 2px solid #fca5a5;
            box-shadow: 0 4px 12px rgba(153, 27, 27, 0.15);
            font-weight: 500;
        }
        
        .range-inputs {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
            width: 100%;
        }
        
        .range-inputs input {
            min-width: 0;
        }
        
        @media (max-width: 768px) {
            body {
                padding: 10px;
            }
            
            .header h1 {
                font-size: 26px;
            }
            
            .header p {
                font-size: 13px;
            }
            
            .fetch-section,
            .table-section {
                padding: 20px;
            }
            
            .form-grid {
                grid-template-columns: 1fr;
                gap: 18px;
            }
            
            .stats {
                grid-template-columns: 1fr;
            }
            
            .button-group {
                grid-template-columns: 1fr;
            }
            
            table {
                font-size: 11px;
                min-width: 800px;
            }
            
            th, td {
                padding: 10px 12px;
            }
            
            .stat-card .value {
                font-size: 22px;
            }
            
            .btn-fetch,
            .btn-reset {
                width: 100%;
                padding: 16px 24px;
            }
        }
        
        @media (max-width: 480px) {
            body {
                padding: 5px;
            }
            
            .container {
                border-radius: 12px;
            }
            
            .header {
                padding: 30px 20px;
            }
            
            .header h1 {
                font-size: 22px;
                line-height: 1.3;
            }
            
            .header p {
                font-size: 12px;
            }
            
            .fetch-section,
            .table-section {
                padding: 15px;
            }
            
            .range-inputs {
                grid-template-columns: 1fr;
            }
            
            .filters-title {
                font-size: 18px;
            }
            
            .btn-fetch,
            .btn-reset {
                padding: 14px 20px;
                font-size: 15px;
            }
            
            table {
                min-width: 600px;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Deribit Options Data Fetcher</h1>
            <p>Advanced filtering for cryptocurrency options with Open Interest, Volume, and IV</p>
        </div>
        
        <div class="fetch-section">
            <div class="filters-title">📊 Filter Options</div>
            
            <div class="form-grid">
                <div class="form-group">
                    <label for="currency">💰 Currency</label>
                    <select id="currency">
                        <option value="BTC">BTC - Bitcoin</option>
                        <option value="ETH">ETH - Ethereum</option>
                        <option value="SOL">SOL - Solana</option>
                        <option value="USDC">USDC - USD Coin</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="optionType">📈 Option Type</label>
                    <select id="optionType">
                        <option value="all">All (Calls & Puts)</option>
                        <option value="call">Calls Only</option>
                        <option value="put">Puts Only</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label>
                        📅 Creation Date Range <span style="opacity: 0.6; font-weight: 400;">(Optional - </span>
                        <a href="javascript:void(0)" onclick="document.getElementById('creationFrom').value=''; document.getElementById('creationTo').value=''; return false;" style="color: #2a5298; text-decoration: underline; font-weight: 400; font-size: 12px;">Clear</a>
                        <span style="opacity: 0.6; font-weight: 400;">)</span>
                    </label>
                    <div class="range-inputs">
                        <input type="date" id="creationFrom" placeholder="From" title="Leave empty to include all" autocomplete="off">
                        <input type="date" id="creationTo" placeholder="To" title="Leave empty to include all" autocomplete="off">
                    </div>
                    <small style="color: #64748b; font-size: 11px; margin-top: 4px; display: block;">💡 Tip: Leave empty when filtering by expiry date</small>
                </div>
                
                <div class="form-group">
                    <label>⏰ Expiration Date Range <span style="opacity: 0.6; font-weight: 400;">(Optional)</span></label>
                    <div class="range-inputs">
                        <input type="date" id="expiryFrom" placeholder="From" title="Leave empty to include all">
                        <input type="date" id="expiryTo" placeholder="To" title="Leave empty to include all">
                    </div>
                </div>
                
                <div class="form-group">
                    <label>📊 Open Interest Range <span style="opacity: 0.6; font-weight: 400;">(Optional)</span></label>
                    <div class="range-inputs">
                        <input type="number" id="oiMin" placeholder="Min" step="0.01" title="Leave empty to include all">
                        <input type="number" id="oiMax" placeholder="Max" step="0.01" title="Leave empty to include all">
                    </div>
                </div>
                
                <div class="form-group">
                    <div class="checkbox-group">
                        <input type="checkbox" id="includeExpired">
                        <label for="includeExpired">Include Expired Options</label>
                    </div>
                </div>
            </div>
            
            <div class="button-group">
                <button class="btn-fetch" id="fetchBtn" onclick="fetchData()">🚀 Fetch Options Data</button>
                <button class="btn-reset" onclick="resetFilters()">🔄 Reset Filters</button>
            </div>
        </div>
        
        <div id="loading" class="loading" style="display: none;">
            <div class="spinner"></div>
            <p>Fetching comprehensive market data from Deribit...</p>
            <p style="font-size: 12px; margin-top: 10px;">This may take 10-30 seconds depending on filters...</p>
        </div>
        
        <div id="warning" style="display: none; background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%); color: #92400e; padding: 18px 22px; border-radius: 12px; margin: 20px 30px; border: 2px solid #fbbf24; box-shadow: 0 4px 12px rgba(251, 191, 36, 0.2); font-weight: 500;"></div>
        
        <div id="error" style="display: none;"></div>
        
        <div class="table-section">
            <div id="stats" class="stats" style="display: none;"></div>
            <div id="tableContainer"></div>
        </div>
    </div>
    
    <script>
        let allData = [];
        
        function resetFilters() {
            document.getElementById('currency').value = 'BTC';
            document.getElementById('optionType').value = 'all';
            document.getElementById('creationFrom').value = '';
            document.getElementById('creationTo').value = '';
            document.getElementById('expiryFrom').value = '';
            document.getElementById('expiryTo').value = '';
            document.getElementById('oiMin').value = '';
            document.getElementById('oiMax').value = '';
            document.getElementById('includeExpired').checked = false;
        }
        
        async function fetchData() {
            const currency = document.getElementById('currency').value;
            const optionType = document.getElementById('optionType').value;
            const creationFrom = document.getElementById('creationFrom').value;
            const creationTo = document.getElementById('creationTo').value;
            const expiryFrom = document.getElementById('expiryFrom').value;
            const expiryTo = document.getElementById('expiryTo').value;
            const oiMin = document.getElementById('oiMin').value;
            const oiMax = document.getElementById('oiMax').value;
            const includeExpired = document.getElementById('includeExpired').checked;
            
            const loading = document.getElementById('loading');
            const tableContainer = document.getElementById('tableContainer');
            const errorDiv = document.getElementById('error');
            const warningDiv = document.getElementById('warning');
            const statsDiv = document.getElementById('stats');
            const fetchBtn = document.getElementById('fetchBtn');
            
            // Warning if creation dates might filter out expiry results
            if ((creationFrom || creationTo) && (expiryFrom || expiryTo)) {
                console.warn('⚠️ Both creation and expiry dates are set. Creation date filters may exclude options.');
                console.warn('💡 Tip: Leave creation date fields empty when filtering by expiry date.');
                warningDiv.innerHTML = '⚠️ <strong>Warning:</strong> Both creation and expiry date filters are active. This may return no results if options were created outside the creation date range. <a href="javascript:void(0)" onclick="document.getElementById(\'creationFrom\').value=\'\'; document.getElementById(\'creationTo\').value=\'\'; fetchData();" style="color: #92400e; text-decoration: underline; font-weight: 700;">Click here to clear creation dates and retry.</a>';
                warningDiv.style.display = 'block';
            } else {
                warningDiv.style.display = 'none';
            }
            
            loading.style.display = 'block';
            tableContainer.innerHTML = '';
            errorDiv.style.display = 'none';
            statsDiv.style.display = 'none';
            fetchBtn.disabled = true;
            
            try {
                const params = new URLSearchParams({
                    currency: currency,
                    option_type: optionType,
                    include_expired: includeExpired
                });
                
                if (creationFrom) params.append('creation_from', creationFrom);
                if (creationTo) params.append('creation_to', creationTo);
                if (expiryFrom) params.append('expiry_from', expiryFrom);
                if (expiryTo) params.append('expiry_to', expiryTo);
                if (oiMin) params.append('oi_min', oiMin);
                if (oiMax) params.append('oi_max', oiMax);
                
                console.log('Fetching data with params:', params.toString());
                const response = await fetch(`/api/options?${params}`, {
                    method: 'GET',
                    headers: {
                        'Accept': 'application/json',
                    },
                });
                
                console.log('Response status:', response.status);
                
                if (!response.ok) {
                    const errorText = await response.text();
                    throw new Error(`HTTP ${response.status}: ${errorText}`);
                }
                
                const data = await response.json();
                console.log('Received data:', data);
                
                loading.style.display = 'none';
                fetchBtn.disabled = false;
                
                if (data.result && data.result.length > 0) {
                    allData = data.result;
                    console.log('Processing', allData.length, 'instruments');
                    displayStats(data.result);
                    displayTable(data.result);
                } else {
                    tableContainer.innerHTML = '<div class="no-data">No options data found for the selected criteria. Try adjusting your filters.</div>';
                }
            } catch (error) {
                console.error('Error:', error);
                loading.style.display = 'none';
                fetchBtn.disabled = false;
                errorDiv.innerHTML = `<div class="error">❌ Error fetching data: ${error.message}</div>`;
                errorDiv.style.display = 'block';
            }
        }
        
        function displayStats(data) {
            const statsDiv = document.getElementById('stats');
            const totalInstruments = data.length;
            
            let totalOI = 0;
            let totalVolume = 0;
            let instrumentsWithData = 0;
            let callsCount = 0;
            let putsCount = 0;
            
            data.forEach(item => {
                if (item.open_interest !== null && item.open_interest !== undefined) {
                    totalOI += parseFloat(item.open_interest) || 0;
                    instrumentsWithData++;
                }
                if (item.volume_24h !== null && item.volume_24h !== undefined) {
                    totalVolume += parseFloat(item.volume_24h) || 0;
                }
                
                if (item.instrument_name) {
                    if (item.instrument_name.endsWith('-C')) callsCount++;
                    if (item.instrument_name.endsWith('-P')) putsCount++;
                }
            });
            
            statsDiv.innerHTML = `
                <div class="stat-card">
                    <div class="label">Total Instruments</div>
                    <div class="value">${totalInstruments}</div>
                </div>
                <div class="stat-card">
                    <div class="label">Calls / Puts</div>
                    <div class="value">${callsCount} / ${putsCount}</div>
                </div>
                <div class="stat-card">
                    <div class="label">Total Open Interest</div>
                    <div class="value">${totalOI.toLocaleString(undefined, {maximumFractionDigits: 0})}</div>
                </div>
                <div class="stat-card">
                    <div class="label">24h Volume</div>
                    <div class="value">${totalVolume.toLocaleString(undefined, {maximumFractionDigits: 0})}</div>
                </div>
                <div class="stat-card">
                    <div class="label">With Market Data</div>
                    <div class="value">${instrumentsWithData}</div>
                </div>
            `;
            statsDiv.style.display = 'flex';
        }
        
        function displayTable(data) {
            if (data.length === 0) return;
            
            const priorityColumns = [
                'instrument_name',
                'strike',
                'option_type',
                'creation_timestamp',
                'expiration_timestamp',
                'open_interest',
                'volume_24h',
                'mark_iv',
                'last_price',
                'mark_price',
                'bid_price',
                'ask_price'
            ];
            
            const allKeys = new Set();
            data.forEach(item => {
                Object.keys(item).forEach(key => allKeys.add(key));
            });
            
            const keys = [];
            priorityColumns.forEach(col => {
                if (allKeys.has(col)) keys.push(col);
            });
            allKeys.forEach(key => {
                if (!priorityColumns.includes(key)) keys.push(key);
            });
            
            let html = '<table><thead><tr>';
            keys.forEach(key => {
                const isHighlight = ['open_interest', 'volume_24h', 'mark_iv'].includes(key);
                html += `<th class="${isHighlight ? 'highlight-col' : ''}" onclick="sortTable('${key}')">${formatHeader(key)}</th>`;
            });
            html += '</tr></thead><tbody>';
            
            data.forEach(item => {
                html += '<tr>';
                keys.forEach(key => {
                    const value = item[key];
                    const isHighlight = ['open_interest', 'volume_24h', 'mark_iv'].includes(key);
                    html += `<td class="${isHighlight ? 'highlight-col' : ''}">${formatValue(key, value)}</td>`;
                });
                html += '</tr>';
            });
            
            html += '</tbody></table>';
            document.getElementById('tableContainer').innerHTML = '<div class="table-wrapper">' + html + '</div>';
        }
        
        function sortTable(key) {
            allData.sort((a, b) => {
                const aVal = a[key];
                const bVal = b[key];
                
                if (aVal === null || aVal === undefined) return 1;
                if (bVal === null || bVal === undefined) return -1;
                
                if (typeof aVal === 'number' && typeof bVal === 'number') {
                    return bVal - aVal;
                }
                
                return String(aVal).localeCompare(String(bVal));
            });
            
            displayTable(allData);
        }
        
        function formatHeader(key) {
            const headerMap = {
                'open_interest': 'Open Interest',
                'volume_24h': '24h Volume',
                'mark_iv': 'IV (%)',
                'bid_price': 'Bid',
                'ask_price': 'Ask',
                'last_price': 'Last',
                'mark_price': 'Mark',
                'option_type': 'Type',
                'strike': 'Strike',
                'creation_timestamp': 'Created',
                'expiration_timestamp': 'Expires'
            };
            
            if (headerMap[key]) return headerMap[key];
            
            return key.split('_').map(word => 
                word.charAt(0).toUpperCase() + word.slice(1)
            ).join(' ');
        }
        
        function formatValue(key, value) {
            if (value === null || value === undefined) return '-';
            
            if (key === 'mark_iv') {
                return (value * 100).toFixed(2) + '%';
            }
            
            if (key === 'open_interest' || key === 'volume_24h') {
                return parseFloat(value).toLocaleString(undefined, {
                    minimumFractionDigits: 0,
                    maximumFractionDigits: 2
                });
            }
            
            if (key.includes('timestamp')) {
                return new Date(value).toLocaleString();
            }
            
            if (typeof value === 'number') {
                if (key.includes('price') || key.includes('strike')) {
                    return value.toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2
                    });
                }
                return value.toLocaleString();
            }
            
            if (typeof value === 'boolean') {
                return value ? '✓' : '✗';
            }
            
            if (typeof value === 'object') {
                return JSON.stringify(value);
            }
            
            return value;
        }
        
        window.addEventListener('load', () => {
            // Always clear creation date fields on page load to prevent confusion
            document.getElementById('creationFrom').value = '';
            document.getElementById('creationTo').value = '';
            fetchData();
        });
    </script>
</body>
</html>
    "#;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Deribit Options Data Fetcher on http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/api/options", web::get().to(get_options))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}