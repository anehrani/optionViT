use axum::{
    extract::Query,
    response::{Html, Json},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use serde::{Serialize, Deserialize};
use polars::prelude::*;
use std::sync::{Arc, Mutex};

use optionvit::download_deribit_data;

// Global storage for the DataFrame
static mut GLOBAL_DF: Option<Arc<Mutex<DataFrame>>> = None;

#[derive(Deserialize)]
struct FilterParams {
    strike: Option<f64>,
    expiry_date: Option<String>,
    option_type: Option<String>,
}

#[derive(Serialize)]
struct DataRow {
    instrument_name: String,
    currency: String,
    expiry_token: String,
    expiry_iso: String,
    timestamp_ms: i64,
    timestamp_utc: String,
    direction: String,
    price: f64,
    amount: f64,
    iv: Option<f64>,
    index_price: Option<f64>,
    mark_price: Option<f64>,
    trade_id: String,
    trade_seq: i64,
    block_trade_id: Option<String>,
    liquidity: Option<String>,
    tick_direction: Option<i32>,
    strike: Option<f64>,
    option_type: Option<String>,
    open_interest: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data for default expiry date
    let expiry_date = "2025-09-30";
    
    println!("📊 Downloading comprehensive BTC options data for {}...", expiry_date);
    let df = download_deribit_data("BTC", expiry_date).await?;
    
    println!("✅ Downloaded data successfully!");
    println!("📈 DataFrame shape: {} rows × {} columns", df.height(), df.width());
    
    // Store the DataFrame globally for web access
    unsafe {
        GLOBAL_DF = Some(Arc::new(Mutex::new(df.clone())));
    }
    
    // Start web server
    println!("🚀 Starting web server...");
    
    // Define routes
    let app = Router::new()
        .route("/", get(data_table_page))
        .route("/data", get(data_table_page))
        .route("/api/data", get(data_api_filtered));

    // Specify where to listen (localhost:3000)
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🌐 Server running on http://{}", listener.local_addr().unwrap());
    println!("📊 View data at: http://127.0.0.1:3000/data");

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}

async fn data_table_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>BTC Options Data - Excel View</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 95%;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { 
            color: #333; 
            text-align: center;
            margin-bottom: 30px;
        }
        .info {
            background-color: #e8f4fd;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }
        #dataTable {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            font-size: 12px;
        }
        #dataTable th, #dataTable td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        #dataTable th {
            background-color: #4CAF50;
            color: white;
            font-weight: bold;
            position: sticky;
            top: 0;
            z-index: 100;
        }
        #dataTable tr:nth-child(even) {
            background-color: #f2f2f2;
        }
        #dataTable tr:hover {
            background-color: #e8f4fd;
        }
        .table-container {
            max-height: 600px;
            overflow-y: auto;
            border: 1px solid #ddd;
        }
        .loading {
            text-align: center;
            padding: 50px;
            font-size: 18px;
            color: #666;
        }
        .stats {
            display: flex;
            justify-content: space-around;
            margin: 20px 0;
        }
        .stat-box {
            background-color: #f9f9f9;
            padding: 15px;
            border-radius: 5px;
            text-align: center;
            border: 1px solid #ddd;
        }
        .stat-number {
            font-size: 24px;
            font-weight: bold;
            color: #4CAF50;
        }
        .stat-label {
            font-size: 14px;
            color: #666;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 BTC Options Trading Data</h1>
        
        <div class="info">
            <p><strong>Real-time BTC Options Data:</strong></p>
            <ul>
                <li>Live data from Deribit API</li>
                <li>Includes trade history, open interest, and market data</li>
                <li>Sortable and scrollable table view</li>
            </ul>
        </div>

        <div class="stats" id="statsContainer">
            <!-- Stats will be populated by JavaScript -->
        </div>
        
        <div class="table-container">
            <div id="loading" class="loading">📡 Loading data...</div>
            <table id="dataTable" style="display:none;">
                <thead>
                    <tr id="tableHeader">
                        <!-- Headers will be populated by JavaScript -->
                    </tr>
                </thead>
                <tbody id="tableBody">
                    <!-- Data will be populated by JavaScript -->
                </tbody>
            </table>
        </div>
    </div>

    <script>
        let allData = [];

        async function loadData() {
            try {
                const response = await fetch('/api/data');
                const data = await response.json();
                
                if (data.length === 0) {
                    document.getElementById('loading').innerHTML = '📊 No data available.';
                    return;
                }
                
                allData = data;
                
                // Create table headers
                const headers = Object.keys(data[0]);
                const headerRow = document.getElementById('tableHeader');
                headerRow.innerHTML = '';
                headers.forEach(header => {
                    const th = document.createElement('th');
                    th.textContent = header.replace(/_/g, ' ').toUpperCase();
                    headerRow.appendChild(th);
                });
                
                // Populate table body
                populateTable(data);
                
                // Show stats
                showStats(data);
                
                // Hide loading and show table
                document.getElementById('loading').style.display = 'none';
                document.getElementById('dataTable').style.display = 'table';
                
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('loading').innerHTML = '❌ Error loading data: ' + error.message;
            }
        }
        
        function populateTable(data) {
            const tbody = document.getElementById('tableBody');
            tbody.innerHTML = '';
            
            data.forEach((row, index) => {
                const tr = document.createElement('tr');
                
                Object.values(row).forEach(value => {
                    const td = document.createElement('td');
                    if (typeof value === 'number') {
                        td.textContent = value.toLocaleString();
                        td.style.textAlign = 'right';
                    } else {
                        td.textContent = value || 'N/A';
                    }
                    tr.appendChild(td);
                });
                
                tbody.appendChild(tr);
            });
        }
        
        function showStats(data) {
            const statsContainer = document.getElementById('statsContainer');
            
            // Calculate basic stats
            const totalRows = data.length;
            const uniqueInstruments = new Set(data.map(row => row.instrument_name)).size;
            const totalVolume = data.reduce((sum, row) => sum + (row.amount || 0), 0);
            const avgPrice = data.reduce((sum, row) => sum + (row.price || 0), 0) / totalRows;
            
            statsContainer.innerHTML = `
                <div class="stat-box">
                    <div class="stat-number">${totalRows}</div>
                    <div class="stat-label">Total Trades</div>
                </div>
                <div class="stat-box">
                    <div class="stat-number">${uniqueInstruments}</div>
                    <div class="stat-label">Instruments</div>
                </div>
                <div class="stat-box">
                    <div class="stat-number">${totalVolume.toFixed(1)}</div>
                    <div class="stat-label">Total Volume</div>
                </div>
                <div class="stat-box">
                    <div class="stat-number">$${avgPrice.toFixed(6)}</div>
                    <div class="stat-label">Avg Price</div>
                </div>
            `;
        }
        
        // Load data when page loads
        loadData();
    </script>
</body>
</html>
    "#)
}

async fn data_api_filtered(Query(params): Query<FilterParams>) -> Json<Vec<DataRow>> {
    let df = unsafe {
        match &GLOBAL_DF {
            Some(df_arc) => df_arc.lock().unwrap().clone(),
            None => return Json(vec![]),
        }
    };
    
    // Apply option_type filter if provided
    let filtered_df = if let Some(option_type) = &params.option_type {
        match df
            .clone()
            .lazy()
            .filter(col("option_type").eq(lit(option_type.as_str())))
            .collect() {
            Ok(filtered) => filtered,
            Err(_) => df, // If filter fails, return original data
        }
    } else {
        df
    };
    
    let mut rows = Vec::new();
    
    for i in 0..filtered_df.height() {
        let row = DataRow {
            instrument_name: filtered_df.column("instrument_name").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            currency: filtered_df.column("currency").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_token: filtered_df.column("expiry_token").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_iso: filtered_df.column("expiry_iso").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            timestamp_ms: filtered_df.column("timestamp_ms").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
            timestamp_utc: filtered_df.column("timestamp_utc").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            direction: filtered_df.column("direction").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            price: filtered_df.column("price").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
            amount: filtered_df.column("amount").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
            iv: filtered_df.column("iv").unwrap().get(i).unwrap().to_string().parse().ok(),
            index_price: filtered_df.column("index_price").unwrap().get(i).unwrap().to_string().parse().ok(),
            mark_price: filtered_df.column("mark_price").unwrap().get(i).unwrap().to_string().parse().ok(),
            trade_id: filtered_df.column("trade_id").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            trade_seq: filtered_df.column("trade_seq").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
            block_trade_id: {
                let val = filtered_df.column("block_trade_id").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            liquidity: {
                let val = filtered_df.column("liquidity").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            tick_direction: filtered_df.column("tick_direction").unwrap().get(i).unwrap().to_string().parse().ok(),
            strike: filtered_df.column("strike").unwrap().get(i).unwrap().to_string().parse().ok(),
            option_type: {
                let val = filtered_df.column("option_type").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            open_interest: filtered_df.column("open_interest").unwrap().get(i).unwrap().to_string().parse().ok(),
        };
        rows.push(row);
    }
    
    Json(rows)
}