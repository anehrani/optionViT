use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};

use optionvit::{download_deribit_data, fetch_available_expiries};

const DEFAULT_ASSET: &str = "BTC";

#[derive(Clone)]
struct AppState {
    asset: String,
    data: Arc<RwLock<Option<DataFrame>>>,
    current_expiry: Arc<RwLock<Option<String>>>,
    expiries: Arc<RwLock<Vec<String>>>,
}

#[allow(dead_code)]
fn _assert_state_bounds()
where
    AppState: Clone + Send + Sync + 'static,
{
}

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

#[derive(Deserialize)]
struct LoadRequest {
    expiry: String,
}

#[derive(Serialize)]
struct LoadResponse {
    expiry: String,
    rows: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct ExpiryResponse {
    available: Vec<String>,
    current: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    _assert_state_bounds();

    let state = AppState {
        asset: DEFAULT_ASSET.to_string(),
        data: Arc::new(RwLock::new(None)),
        current_expiry: Arc::new(RwLock::new(None)),
        expiries: Arc::new(RwLock::new(Vec::new())),
    };

    println!(
        "🔍 Fetching available expiry dates for {}...",
        DEFAULT_ASSET
    );
    let initial_expiries = match fetch_available_expiries(DEFAULT_ASSET).await {
        Ok(list) if !list.is_empty() => list,
        Ok(_) => {
            println!("⚠️ No upcoming expiries found for {}.", DEFAULT_ASSET);
            Vec::new()
        }
        Err(err) => {
            eprintln!("⚠️ Could not retrieve expiries: {}", err);
            Vec::new()
        }
    };

    {
        let mut expiries_guard = state.expiries.write().await;
        *expiries_guard = initial_expiries.clone();
    }

    if let Some(default_expiry) = initial_expiries.last().cloned() {
        if let Err(err) = load_expiry_into_state(&state, &default_expiry).await {
            eprintln!(
                "⚠️ Failed to load default expiry {}: {}",
                default_expiry, err
            );
        }
    } else {
        println!("⚠️ Start the server and use the refresh control to load data once expiries are available.");
    }

    let app = Router::new()
        .route("/", get(data_table_page))
        .route("/data", get(data_table_page))
        .route("/api/data", get(data_api_filtered))
        .route("/api/expiries", get(expiries_handler))
        .route("/api/load", post(load_expiry_handler))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("🌐 Server running on http://{}", listener.local_addr()?);
    println!("📊 View data at: http://127.0.0.1:3000/data");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn load_expiry_into_state(
    state: &AppState,
    expiry: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let asset = state.asset.as_str();

    println!(
        "📊 Downloading comprehensive {} options data for {}...",
        asset, expiry
    );
    let df = download_deribit_data(asset, expiry).await?;
    let rows = df.height();

    {
        let mut data_guard = state.data.write().await;
        *data_guard = Some(df);
    }

    {
        let mut current_guard = state.current_expiry.write().await;
        *current_guard = Some(expiry.to_string());
    }

    if let Ok(updated_expiries) = fetch_available_expiries(asset).await {
        let mut expiries_guard = state.expiries.write().await;
        *expiries_guard = updated_expiries;
    }

    println!("✅ Loaded dataset for {} ({} rows)", expiry, rows);
    Ok(rows)
}

async fn data_table_page() -> Html<&'static str> {
    Html(
        r#"
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
        .controls {
            display: flex;
            flex-wrap: wrap;
            align-items: center;
            gap: 12px;
            margin-bottom: 20px;
        }
        .controls label {
            font-weight: bold;
        }
        .controls select, .controls button {
            padding: 8px 12px;
            border: 1px solid #ccc;
            border-radius: 4px;
            font-size: 14px;
        }
        .controls button {
            background-color: #4CAF50;
            color: white;
            cursor: pointer;
        }
        .controls button:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        .status {
            font-size: 13px;
            color: #555;
        }
        .status.error {
            color: #b00020;
        }
        .status.success {
            color: #2e7d32;
        }
        .table-container {
            max-height: 600px;
            overflow-y: auto;
            border: 1px solid #ddd;
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
            flex: 1 1 200px;
            margin: 0 10px;
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
        @media (max-width: 768px) {
            .controls {
                flex-direction: column;
                align-items: flex-start;
            }
            .stats {
                flex-direction: column;
                gap: 12px;
            }
            .stat-box {
                margin: 0;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 BTC Options Trading Data</h1>

        <div class="info">
            <p><strong>Real-time BTC Options Data:</strong></p>
            <ul>
                <li>Select an expiry date to load Deribit data</li>
                <li>Includes trade history, open interest, and market data</li>
                <li>Sortable and scrollable table view</li>
            </ul>
        </div>

        <div class="controls">
            <label for="expirySelect">Expiry Date:</label>
            <select id="expirySelect"></select>
            <button id="refreshButton">Refresh</button>
            <span id="statusMessage" class="status"></span>
        </div>

        <div class="stats" id="statsContainer"></div>

        <div class="table-container">
            <div id="loading" class="loading">📡 Loading data...</div>
            <table id="dataTable" style="display:none;">
                <thead>
                    <tr id="tableHeader"></tr>
                </thead>
                <tbody id="tableBody"></tbody>
            </table>
        </div>
    </div>

    <script>
        let allData = [];

        const expirySelect = document.getElementById('expirySelect');
        const refreshButton = document.getElementById('refreshButton');
        const statusMessage = document.getElementById('statusMessage');

        refreshButton.addEventListener('click', async () => {
            const selectedExpiry = expirySelect.value;
            if (!selectedExpiry) {
                setStatus('Please choose an expiry date first.', 'error');
                return;
            }

            refreshButton.disabled = true;
            setStatus(`Loading data for ${selectedExpiry}...`, '');

            try {
                const response = await fetch('/api/load', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ expiry: selectedExpiry })
                });

                const result = await response.json();

                if (!response.ok) {
                    throw new Error(result.error || 'Failed to load data');
                }

                setStatus(`Loaded ${result.rows} rows for ${result.expiry}.`, 'success');
                await loadExpiries();
                await loadData();
            } catch (error) {
                console.error('Error refreshing data:', error);
                setStatus(`Error: ${error.message}`, 'error');
            } finally {
                refreshButton.disabled = false;
            }
        });

        function setStatus(message, type) {
            statusMessage.textContent = message;
            statusMessage.className = `status ${type}`.trim();
        }

        async function loadExpiries() {
            try {
                const response = await fetch('/api/expiries');
                const data = await response.json();
                expirySelect.innerHTML = '';

                data.available.forEach(expiry => {
                    const option = document.createElement('option');
                    option.value = expiry;
                    option.textContent = expiry;
                    expirySelect.appendChild(option);
                });

                if (data.current && data.available.includes(data.current)) {
                    expirySelect.value = data.current;
                }
            } catch (error) {
                console.error('Error loading expiries:', error);
                setStatus('Error fetching expiry list.', 'error');
            }
        }

        async function loadData() {
            try {
                const response = await fetch('/api/data');
                const data = await response.json();

                if (!Array.isArray(data) || data.length === 0) {
                    allData = [];
                    document.getElementById('dataTable').style.display = 'none';
                    document.getElementById('loading').style.display = 'block';
                    document.getElementById('loading').textContent = '📊 No data available for the selected expiry.';
                    document.getElementById('statsContainer').innerHTML = '';
                    return;
                }

                allData = data;
                buildTable(data);
                showStats(data);

                document.getElementById('loading').style.display = 'none';
                document.getElementById('dataTable').style.display = 'table';
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('loading').style.display = 'block';
                document.getElementById('loading').innerHTML = '❌ Error loading data: ' + error.message;
                document.getElementById('dataTable').style.display = 'none';
                setStatus('Error loading dataset.', 'error');
            }
        }

        function buildTable(data) {
            const headerRow = document.getElementById('tableHeader');
            const tbody = document.getElementById('tableBody');

            headerRow.innerHTML = '';
            tbody.innerHTML = '';

            const headers = Object.keys(data[0] || {});
            headers.forEach(header => {
                const th = document.createElement('th');
                th.textContent = header.replace(/_/g, ' ').toUpperCase();
                headerRow.appendChild(th);
            });

            data.forEach(row => {
                const tr = document.createElement('tr');
                headers.forEach(header => {
                    const td = document.createElement('td');
                    const value = row[header];

                    if (typeof value === 'number') {
                        td.textContent = Number.isInteger(value)
                            ? value.toLocaleString()
                            : value.toLocaleString(undefined, { maximumFractionDigits: 6 });
                        td.style.textAlign = 'right';
                    } else {
                        td.textContent = value ?? 'N/A';
                    }

                    tr.appendChild(td);
                });
                tbody.appendChild(tr);
            });
        }

        function showStats(data) {
            const statsContainer = document.getElementById('statsContainer');
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
                    <div class="stat-number">${totalVolume.toFixed(2)}</div>
                    <div class="stat-label">Total Volume</div>
                </div>
                <div class="stat-box">
                    <div class="stat-number">$${avgPrice.toFixed(6)}</div>
                    <div class="stat-label">Avg Price</div>
                </div>
            `;
        }

        (async function initPage() {
            await loadExpiries();
            await loadData();
        })();
    </script>
</body>
</html>
    "#,
    )
}

async fn data_api_filtered(
    State(state): State<AppState>,
    Query(params): Query<FilterParams>,
) -> Json<Vec<DataRow>> {
    let data_handle = state.data.clone();
    let data_guard = data_handle.read().await;
    let df = match data_guard.as_ref() {
        Some(df) => df.clone(),
        None => return Json(vec![]),
    };
    drop(data_guard);

    let mut lazy_df = df.clone().lazy();

    if let Some(option_type) = &params.option_type {
        lazy_df = lazy_df.filter(col("option_type").eq(lit(option_type.as_str())));
    }

    if let Some(expiry_date) = &params.expiry_date {
        lazy_df = lazy_df.filter(col("expiry_iso").eq(lit(expiry_date.as_str())));
    }

    if let Some(strike) = params.strike {
        lazy_df = lazy_df.filter(col("strike").eq(lit(strike)));
    }

    let filtered_df = lazy_df.collect().unwrap_or(df.clone());

    if filtered_df.height() == 0 {
        return Json(vec![]);
    }

    let mut rows = Vec::with_capacity(filtered_df.height());

    for i in 0..filtered_df.height() {
        let row = DataRow {
            instrument_name: filtered_df
                .column("instrument_name")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            currency: filtered_df
                .column("currency")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            expiry_token: filtered_df
                .column("expiry_token")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            expiry_iso: filtered_df
                .column("expiry_iso")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            timestamp_ms: filtered_df
                .column("timestamp_ms")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok())
                .unwrap_or_default(),
            timestamp_utc: filtered_df
                .column("timestamp_utc")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            direction: filtered_df
                .column("direction")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            price: filtered_df
                .column("price")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok())
                .unwrap_or_default(),
            amount: filtered_df
                .column("amount")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok())
                .unwrap_or_default(),
            iv: filtered_df
                .column("iv")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
            index_price: filtered_df
                .column("index_price")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
            mark_price: filtered_df
                .column("mark_price")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
            trade_id: filtered_df
                .column("trade_id")
                .ok()
                .and_then(|c| c.get(i).ok())
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            trade_seq: filtered_df
                .column("trade_seq")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok())
                .unwrap_or_default(),
            block_trade_id: filtered_df
                .column("block_trade_id")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| {
                    let value = v.to_string();
                    if value == "null" {
                        None
                    } else {
                        Some(value.trim_matches('"').to_string())
                    }
                }),
            liquidity: filtered_df
                .column("liquidity")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| {
                    let value = v.to_string();
                    if value == "null" {
                        None
                    } else {
                        Some(value.trim_matches('"').to_string())
                    }
                }),
            tick_direction: filtered_df
                .column("tick_direction")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
            strike: filtered_df
                .column("strike")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
            option_type: filtered_df
                .column("option_type")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| {
                    let value = v.to_string();
                    if value == "null" {
                        None
                    } else {
                        Some(value.trim_matches('"').to_string())
                    }
                }),
            open_interest: filtered_df
                .column("open_interest")
                .ok()
                .and_then(|c| c.get(i).ok())
                .and_then(|v| v.to_string().parse().ok()),
        };
        rows.push(row);
    }

    Json(rows)
}

async fn expiries_handler(State(state): State<AppState>) -> Json<ExpiryResponse> {
    let expiries_handle = state.expiries.clone();
    let available = {
        let guard = expiries_handle.read().await;
        guard.clone()
    };

    let current_handle = state.current_expiry.clone();
    let current = {
        let guard = current_handle.read().await;
        guard.clone()
    };

    Json(ExpiryResponse { available, current })
}

async fn load_expiry_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoadRequest>,
) -> Result<Json<LoadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let expiry = payload.expiry.trim();

    if expiry.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Expiry date is required".to_string(),
            }),
        ));
    }

    match load_expiry_into_state(&state, expiry).await {
        Ok(rows) => Ok(Json(LoadResponse {
            expiry: expiry.to_string(),
            rows,
        })),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to load data: {}", err),
            }),
        )),
    }
}
