use axum::{
    extract::{Query, State},
    response::{Html, Json},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use serde::{Serialize, Deserialize};
use polars::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono;

use optionvit::download_deribit_data;

// App state to hold shared data
#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<DataFrame>>,
}

// Global storage for the DataFrame
static mut GLOBAL_DF: Option<Arc<Mutex<DataFrame>>> = None;

#[derive(Deserialize)]
struct FilterParams {
    // Numeric filters with comparison operators
    strike_min: Option<f64>,
    strike_max: Option<f64>,
    price_min: Option<f64>,
    price_max: Option<f64>,
    amount_min: Option<f64>,
    amount_max: Option<f64>,
    iv_min: Option<f64>,
    iv_max: Option<f64>,
    index_price_min: Option<f64>,
    index_price_max: Option<f64>,
    mark_price_min: Option<f64>,
    mark_price_max: Option<f64>,
    trade_seq_min: Option<i64>,
    trade_seq_max: Option<i64>,
    tick_direction_min: Option<i32>,
    tick_direction_max: Option<i32>,
    open_interest_min: Option<f64>,
    open_interest_max: Option<f64>,
    
    // Date filters
    timestamp_from: Option<String>, // ISO 8601 format
    timestamp_to: Option<String>,   // ISO 8601 format
    
    // Categorical filters (comma-separated values)
    instrument_names: Option<String>,
    currencies: Option<String>,
    expiry_tokens: Option<String>,
    expiry_dates: Option<String>,
    directions: Option<String>,
    option_types: Option<String>,
    liquidities: Option<String>,
    instrument_categories: Option<String>,  // Options, Futures, Future Spreads
    
    // Legacy filters for backward compatibility
    strike: Option<f64>,
    expiry_date: Option<String>,
    option_type: Option<String>,
}

#[derive(Serialize)]
struct ColumnMetadata {
    name: String,
    data_type: String,
    unique_values: Option<Vec<String>>, // For categorical columns
    min_value: Option<f64>,             // For numeric columns
    max_value: Option<f64>,             // For numeric columns
    min_date: Option<String>,           // For date columns
    max_date: Option<String>,           // For date columns
}

// Helper function to determine instrument category
fn get_instrument_category(instrument_name: &str) -> String {
    if instrument_name.contains("-FS-") {
        "Future Spreads".to_string()
    } else if instrument_name.ends_with("-C") || instrument_name.ends_with("-P") {
        "Options".to_string()
    } else {
        "Futures".to_string()
    }
}

#[derive(Serialize)]
struct MetadataResponse {
    columns: Vec<ColumnMetadata>,
    available_expiries: Vec<String>,
}

#[derive(Serialize)]
struct DataRow {
    instrument_name: String,
    currency: String,
    expiry_token: String,
    expiry_iso: String,
    instrument_category: String,  // Options, Futures, or Future Spreads
    timestamp: i64,        // Timestamp in seconds for JavaScript
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
    interest_rate: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, fetch available expiry dates
    println!("🔍 Fetching available BTC option expiry dates...");
    
    use optionvit::fetch_available_expiries;
    let available_expiries = fetch_available_expiries("BTC").await?;
    
    if available_expiries.is_empty() {
        println!("❌ No BTC option expiry dates found!");
        return Ok(());
    }
    
    println!("📅 Found {} available expiry dates: {:?}", available_expiries.len(), available_expiries);
    
    // Download data for ALL available expiry dates
    println!("📊 Downloading comprehensive BTC options data for all {} expiry dates...", available_expiries.len());
    
    let mut all_dataframes = Vec::new();
    
    for (i, expiry_date) in available_expiries.iter().enumerate() {
        println!("📊 [{}/{}] Downloading data for {}...", i + 1, available_expiries.len(), expiry_date);
        match download_deribit_data("BTC", expiry_date).await {
            Ok(df) => {
                println!("✅ Downloaded {} rows for {}", df.height(), expiry_date);
                if df.height() > 0 {
                    all_dataframes.push(df);
                }
            }
            Err(e) => {
                println!("⚠️ Failed to download data for {}: {}", expiry_date, e);
            }
        }
    }
    
    if all_dataframes.is_empty() {
        println!("❌ No data found for any expiry dates!");
        return Ok(());
    }
    
    // Combine all dataframes
    println!("📊 Combining data from {} expiry dates...", all_dataframes.len());
    let mut combined_df = all_dataframes[0].clone();
    
    for df in all_dataframes.iter().skip(1) {
        combined_df = combined_df.vstack(df)?;
    }
    
    println!("✅ Downloaded data successfully!");
    println!("📈 Combined DataFrame shape: {} rows × {} columns", combined_df.height(), combined_df.width());
    
    // Store the DataFrame globally for web access
    unsafe {
        GLOBAL_DF = Some(Arc::new(Mutex::new(combined_df.clone())));
    }
    
    // Create app state
    let app_state = AppState {
        data: Arc::new(Mutex::new(combined_df)),
    };

    // Start web server
    println!("🚀 Starting web server...");
    
    // Define routes
    let app = Router::new()
        .route("/", get(data_table_page))
        .route("/data", get(data_table_page))
        .route("/api/data", get(data_api_filtered))
        .route("/api/metadata", get(get_column_metadata))
        .with_state(app_state);

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
    <title>BTC Options Data - Advanced Filtering</title>
    <style>
        body { 
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
            margin: 0;
            padding: 20px;
            background-color: #f8f9fa;
        }
        .container {
            max-width: 98%;
            margin: 0 auto;
            background-color: white;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.08);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 25px;
            text-align: center;
        }
        h1 { 
            margin: 0;
            font-size: 28px;
            font-weight: 300;
        }
        
        /* Filter Panel Styles */
        .filter-panel {
            background-color: #f8f9fa;
            border-bottom: 2px solid #e9ecef;
            padding: 20px;
        }
        .filter-title {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 15px;
            color: #495057;
        }
        .filter-sections {
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            gap: 25px;
        }
        .filter-section {
            background: white;
            padding: 15px;
            border-radius: 8px;
            border: 1px solid #dee2e6;
        }
        .filter-section h3 {
            margin: 0 0 10px 0;
            font-size: 14px;
            font-weight: 600;
            color: #6c757d;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .filter-row {
            display: flex;
            align-items: center;
            margin-bottom: 8px;
            gap: 8px;
        }
        .filter-row label {
            min-width: 80px;
            font-size: 12px;
            color: #6c757d;
        }
        .filter-input {
            flex: 1;
            padding: 6px 10px;
            border: 1px solid #ced4da;
            border-radius: 4px;
            font-size: 12px;
        }
        .filter-buttons {
            margin-top: 15px;
            text-align: center;
        }
        .btn {
            padding: 10px 20px;
            margin: 0 5px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.2s;
        }
        .btn-primary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .btn-secondary {
            background-color: #6c757d;
            color: white;
        }
        .btn:hover {
            transform: translateY(-1px);
            box-shadow: 0 2px 8px rgba(0,0,0,0.15);
        }
        
        /* Stats Panel */
        .stats-container {
            display: flex;
            justify-content: space-around;
            padding: 20px;
            background-color: #ffffff;
            border-bottom: 1px solid #dee2e6;
        }
        .stat-box {
            text-align: center;
            padding: 10px;
        }
        .stat-number {
            font-size: 24px;
            font-weight: bold;
            color: #495057;
        }
        .stat-label {
            font-size: 12px;
            color: #6c757d;
            margin-top: 5px;
        }
        
        /* Table Styles */
        .table-container {
            padding: 20px;
            overflow-x: auto;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            font-size: 11px;
        }
        th, td {
            padding: 8px 6px;
            text-align: left;
            border-bottom: 1px solid #dee2e6;
            white-space: nowrap;
        }
        th {
            background-color: #f8f9fa;
            font-weight: 600;
            color: #495057;
            position: sticky;
            top: 0;
            z-index: 10;
        }
        tr:hover {
            background-color: #f8f9fa;
        }
        .loading {
            text-align: center;
            padding: 40px;
            color: #6c757d;
        }
        
        /* Responsive design */
        @media (max-width: 1200px) {
            .filter-sections {
                grid-template-columns: 1fr 1fr;
            }
        }
        @media (max-width: 768px) {
            .filter-sections {
                grid-template-columns: 1fr;
            }
            .stats-container {
                flex-wrap: wrap;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🪙 BTC Options Trading Data</h1>
            <p>Advanced filtering and real-time data analysis</p>
        </div>
        
        <!-- Advanced Filter Panel -->
        <div class="filter-panel">
            <div class="filter-title">🔍 Advanced Filters</div>
            <div class="filter-sections">
                <!-- Numeric Filters -->
                <div class="filter-section">
                    <h3>Numeric Ranges</h3>
                    <div class="filter-row">
                        <label>Price:</label>
                        <input type="number" class="filter-input" id="price_min" placeholder="Min" step="0.0001">
                        <input type="number" class="filter-input" id="price_max" placeholder="Max" step="0.0001">
                    </div>
                    <div class="filter-row">
                        <label>Strike:</label>
                        <input type="number" class="filter-input" id="strike_min" placeholder="Min">
                        <input type="number" class="filter-input" id="strike_max" placeholder="Max">
                    </div>
                    <div class="filter-row">
                        <label>Amount:</label>
                        <input type="number" class="filter-input" id="amount_min" placeholder="Min">
                        <input type="number" class="filter-input" id="amount_max" placeholder="Max">
                    </div>
                    <div class="filter-row">
                        <label>IV:</label>
                        <input type="number" class="filter-input" id="iv_min" placeholder="Min %" step="0.01">
                        <input type="number" class="filter-input" id="iv_max" placeholder="Max %" step="0.01">
                    </div>
                    <div class="filter-row">
                        <label>Open Interest:</label>
                        <input type="number" class="filter-input" id="open_interest_min" placeholder="Min">
                        <input type="number" class="filter-input" id="open_interest_max" placeholder="Max">
                    </div>
                </div>
                
                <!-- Categorical Filters -->
                <div class="filter-section">
                    <h3>Categories</h3>
                    <div class="filter-row">
                        <label>Instrument:</label>
                        <select class="filter-input" id="instrument_category">
                            <option value="">All</option>
                            <option value="Options">Options</option>
                            <option value="Futures">Futures</option>
                            <option value="Future Spreads">Future Spreads</option>
                        </select>
                    </div>
                    <div class="filter-row">
                        <label>Option Type:</label>
                        <select class="filter-input" id="option_type">
                            <option value="">All</option>
                            <option value="call">Call</option>
                            <option value="put">Put</option>
                        </select>
                    </div>
                    <div class="filter-row">
                        <label>Direction:</label>
                        <select class="filter-input" id="direction">
                            <option value="">All</option>
                            <option value="buy">Buy</option>
                            <option value="sell">Sell</option>
                        </select>
                    </div>
                    <div class="filter-row">
                        <label>Liquidity:</label>
                        <select class="filter-input" id="liquidity">
                            <option value="">All</option>
                            <option value="M">Maker</option>
                            <option value="T">Taker</option>
                        </select>
                    </div>
                </div>
                
                <!-- Date & Time Filters -->
                <div class="filter-section">
                    <h3>Date & Time</h3>
                    <div class="filter-row">
                        <label>From:</label>
                        <input type="datetime-local" class="filter-input" id="timestamp_from">
                    </div>
                    <div class="filter-row">
                        <label>To:</label>
                        <input type="datetime-local" class="filter-input" id="timestamp_to">
                    </div>
                    <div class="filter-row">
                        <label>Expiry:</label>
                        <select class="filter-input" id="expiry_date">
                            <option value="">All Expiries</option>
                        </select>
                    </div>
                </div>
            </div>
            
            <div class="filter-buttons">
                <button class="btn btn-primary" onclick="applyFilters()">🔍 Apply Filters</button>
                <button class="btn btn-secondary" onclick="clearFilters()">🔄 Clear All</button>
                <button class="btn btn-secondary" onclick="refreshData()">📊 Refresh Data</button>
            </div>
        </div>
        
        <!-- Stats Summary -->
        <div class="stats-container" id="statsContainer">
            <div class="stat-box">
                <div class="stat-number">-</div>
                <div class="stat-label">Total Trades</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">-</div>
                <div class="stat-label">Instruments</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">-</div>
                <div class="stat-label">Total Volume</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">-</div>
                <div class="stat-label">Avg Price</div>
            </div>
        </div>
        
        <!-- Data Table -->
        <div class="table-container">
            <div id="loading" class="loading">Loading BTC options data...</div>
            <table id="dataTable" style="display: none;">
                <thead>
                    <tr>
                        <th>Timestamp</th>
                        <th>Instrument</th>
                        <th>Category</th>
                        <th>Price</th>
                        <th>Amount</th>
                        <th>Direction</th>
                        <th>Strike</th>
                        <th>Type</th>
                        <th>Expiry</th>
                        <th>IV (%)</th>
                        <th>Delta</th>
                        <th>Gamma</th>
                        <th>Theta</th>
                        <th>Vega</th>
                        <th>Index Price</th>
                        <th>Mark Price</th>
                        <th>Trade ID</th>
                        <th>Liquidity</th>
                        <th>Open Interest</th>
                        <th>Interest Rate (%)</th>
                    </tr>
                </thead>
                <tbody id="tableBody">
                </tbody>
            </table>
        </div>
    </div>

    <script>
        let allData = [];
        let filteredData = [];
        
        // Load initial data
        async function loadData() {
            try {
                console.log('Loading data from /api/data...');
                const response = await fetch('/api/data');
                if (!response.ok) {
                    throw new Error('Network response was not ok: ' + response.status);
                }
                allData = await response.json();
                console.log('Loaded data:', allData.length, 'rows');
                filteredData = [...allData];
                renderTable(filteredData);
                updateStats(filteredData);
                document.getElementById('loading').style.display = 'none';
                document.getElementById('dataTable').style.display = 'table';
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('loading').innerHTML = 'Error loading data: ' + error.message + '. Please try refreshing.';
            }
        }
        
        // Apply filters
        async function applyFilters() {
            console.log('Applying filters...');
            const params = new URLSearchParams();
            
            // Numeric filters
            const numericFilters = ['price', 'strike', 'amount', 'iv', 'open_interest'];
            numericFilters.forEach(filter => {
                const min = document.getElementById(filter + '_min').value;
                const max = document.getElementById(filter + '_max').value;
                if (min) params.append(filter + '_min', min);
                if (max) params.append(filter + '_max', max);
            });
            
            // Categorical filters (need to map frontend field names to backend parameter names)
            const categoricalFilters = [
                { frontend: 'instrument_category', backend: 'instrument_categories' },
                { frontend: 'option_type', backend: 'option_types' },
                { frontend: 'direction', backend: 'directions' },
                { frontend: 'liquidity', backend: 'liquidities' },
                { frontend: 'expiry_date', backend: 'expiry_dates' }
            ];
            categoricalFilters.forEach(filter => {
                const value = document.getElementById(filter.frontend).value;
                if (value) params.append(filter.backend, value);
            });
            
            // Date filters
            const timestampFrom = document.getElementById('timestamp_from').value;
            const timestampTo = document.getElementById('timestamp_to').value;
            if (timestampFrom) params.append('timestamp_from', new Date(timestampFrom).toISOString());
            if (timestampTo) params.append('timestamp_to', new Date(timestampTo).toISOString());
            
            try {
                const url = '/api/data?' + params.toString();
                console.log('Fetching filtered data from:', url);
                const response = await fetch(url);
                if (!response.ok) {
                    throw new Error('Network response was not ok: ' + response.status);
                }
                filteredData = await response.json();
                console.log('Filtered data:', filteredData.length, 'rows');
                renderTable(filteredData);
                updateStats(filteredData);
            } catch (error) {
                console.error('Error applying filters:', error);
                alert('Error applying filters: ' + error.message);
            }
        }
        
        // Clear all filters
        function clearFilters() {
            console.log('Clearing filters...');
            document.querySelectorAll('.filter-input').forEach(input => {
                input.value = '';
            });
            filteredData = [...allData];
            renderTable(filteredData);
            updateStats(filteredData);
        }
        
        // Refresh data from server
        function refreshData() {
            console.log('Refreshing data...');
            document.getElementById('loading').style.display = 'block';
            document.getElementById('dataTable').style.display = 'none';
            loadData();
        }
        
        // Render table
        function renderTable(data) {
            console.log('Rendering table with', data.length, 'rows');
            const tbody = document.getElementById('tableBody');
            tbody.innerHTML = '';
            
            data.forEach((row, index) => {
                const tr = document.createElement('tr');
                tr.innerHTML = 
                    '<td>' + (row.timestamp ? new Date(row.timestamp * 1000).toLocaleString() : '-') + '</td>' +
                    '<td>' + (row.instrument_name || '-') + '</td>' +
                    '<td>' + (row.instrument_category || '-') + '</td>' +
                    '<td>$' + ((row.price || 0).toFixed(6)) + '</td>' +
                    '<td>' + (row.amount || '-') + '</td>' +
                    '<td>' + (row.direction || '-') + '</td>' +
                    '<td>' + (row.strike || '-') + '</td>' +
                    '<td>' + (row.option_type || '-') + '</td>' +
                    '<td>' + (row.expiry_iso || '-') + '</td>' +
                    '<td>' + (row.iv ? (row.iv * 100).toFixed(2) : '-') + '</td>' +
                    '<td>' + (row.delta ? row.delta.toFixed(4) : '-') + '</td>' +
                    '<td>' + (row.gamma ? row.gamma.toFixed(6) : '-') + '</td>' +
                    '<td>' + (row.theta ? row.theta.toFixed(6) : '-') + '</td>' +
                    '<td>' + (row.vega ? row.vega.toFixed(6) : '-') + '</td>' +
                    '<td>$' + (row.index_price ? row.index_price.toFixed(2) : '-') + '</td>' +
                    '<td>$' + (row.mark_price ? row.mark_price.toFixed(6) : '-') + '</td>' +
                    '<td>' + (row.trade_id || '-') + '</td>' +
                    '<td>' + (row.liquidity || '-') + '</td>' +
                    '<td>' + (row.open_interest || '-') + '</td>' +
                    '<td>' + (row.interest_rate ? (row.interest_rate * 100).toFixed(4) + '%' : '-') + '</td>';
                tbody.appendChild(tr);
            });
        }
        
        // Update statistics
        function updateStats(data) {
            console.log('Updating stats for', data.length, 'rows');
            const statsContainer = document.getElementById('statsContainer');
            const totalRows = data.length;
            const uniqueInstruments = new Set(data.map(row => row.instrument_name)).size;
            const totalVolume = data.reduce((sum, row) => sum + (row.amount || 0), 0);
            const avgPrice = totalRows > 0 ? data.reduce((sum, row) => sum + (row.price || 0), 0) / totalRows : 0;
            
            statsContainer.innerHTML = 
                '<div class="stat-box">' +
                    '<div class="stat-number">' + totalRows + '</div>' +
                    '<div class="stat-label">Total Trades</div>' +
                '</div>' +
                '<div class="stat-box">' +
                    '<div class="stat-number">' + uniqueInstruments + '</div>' +
                    '<div class="stat-label">Instruments</div>' +
                '</div>' +
                '<div class="stat-box">' +
                    '<div class="stat-number">' + totalVolume.toFixed(1) + '</div>' +
                    '<div class="stat-label">Total Volume</div>' +
                '</div>' +
                '<div class="stat-box">' +
                    '<div class="stat-number">$' + avgPrice.toFixed(6) + '</div>' +
                    '<div class="stat-label">Avg Price</div>' +
                '</div>';
        }
        
        // Load metadata (expiry dates and other info)
        async function loadMetadata() {
            try {
                console.log('Loading metadata...');
                const response = await fetch('/api/metadata');
                if (!response.ok) {
                    throw new Error('Network response was not ok: ' + response.status);
                }
                const metadata = await response.json();
                console.log('Loaded metadata:', metadata);
                
                // Populate expiry dates dropdown
                const expirySelect = document.getElementById('expiry_date');
                if (metadata.available_expiries && metadata.available_expiries.length > 0) {
                    metadata.available_expiries.forEach(expiry => {
                        const option = document.createElement('option');
                        option.value = expiry;
                        option.textContent = expiry;
                        expirySelect.appendChild(option);
                    });
                }
                
                // You can also populate other metadata-driven elements here
                console.log('Metadata loaded successfully');
            } catch (error) {
                console.error('Error loading metadata:', error);
                // Don't fail completely if metadata fails to load
            }
        }
        
        // Load data when page loads
        document.addEventListener('DOMContentLoaded', function() {
            console.log('Page loaded, starting data fetch...');
            loadMetadata(); // Load expiry dates first
            loadData();     // Then load the actual data
        });
    </script>
</body>
</html>
    "#)
}

async fn data_api_filtered(Query(params): Query<FilterParams>, State(app_state): State<AppState>) -> Json<Vec<DataRow>> {
    let df = app_state.data.lock().await.clone();
    
    if df.is_empty() {
        return Json(vec![]);
    }
    
    // Start with the original dataframe
    let mut lazy_df = df.clone().lazy();
    
    // Apply numeric range filters
    if let Some(min_val) = params.strike_min {
        lazy_df = lazy_df.filter(col("strike").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.strike_max {
        lazy_df = lazy_df.filter(col("strike").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.price_min {
        lazy_df = lazy_df.filter(col("price").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.price_max {
        lazy_df = lazy_df.filter(col("price").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.amount_min {
        lazy_df = lazy_df.filter(col("amount").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.amount_max {
        lazy_df = lazy_df.filter(col("amount").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.iv_min {
        lazy_df = lazy_df.filter(col("iv").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.iv_max {
        lazy_df = lazy_df.filter(col("iv").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.index_price_min {
        lazy_df = lazy_df.filter(col("index_price").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.index_price_max {
        lazy_df = lazy_df.filter(col("index_price").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.mark_price_min {
        lazy_df = lazy_df.filter(col("mark_price").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.mark_price_max {
        lazy_df = lazy_df.filter(col("mark_price").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.trade_seq_min {
        lazy_df = lazy_df.filter(col("trade_seq").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.trade_seq_max {
        lazy_df = lazy_df.filter(col("trade_seq").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.tick_direction_min {
        lazy_df = lazy_df.filter(col("tick_direction").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.tick_direction_max {
        lazy_df = lazy_df.filter(col("tick_direction").lt_eq(lit(max_val)));
    }
    
    if let Some(min_val) = params.open_interest_min {
        lazy_df = lazy_df.filter(col("open_interest").gt_eq(lit(min_val)));
    }
    if let Some(max_val) = params.open_interest_max {
        lazy_df = lazy_df.filter(col("open_interest").lt_eq(lit(max_val)));
    }
    
    // Apply timestamp range filters
    if let Some(timestamp_from) = &params.timestamp_from {
        if let Ok(timestamp_ms) = chrono::DateTime::parse_from_rfc3339(timestamp_from) {
            let timestamp_ms = timestamp_ms.timestamp_millis();
            lazy_df = lazy_df.filter(col("timestamp_ms").gt_eq(lit(timestamp_ms)));
        }
    }
    if let Some(timestamp_to) = &params.timestamp_to {
        if let Ok(timestamp_ms) = chrono::DateTime::parse_from_rfc3339(timestamp_to) {
            let timestamp_ms = timestamp_ms.timestamp_millis();
            lazy_df = lazy_df.filter(col("timestamp_ms").lt_eq(lit(timestamp_ms)));
        }
    }
    
    // Apply categorical filters (comma-separated values)
    if let Some(instrument_names) = &params.instrument_names {
        let names: Vec<&str> = instrument_names.split(',').map(|s| s.trim()).collect();
        if !names.is_empty() {
            let expr = names.iter().fold(lit(false), |acc, &name| {
                acc.or(col("instrument_name").eq(lit(name)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(currencies) = &params.currencies {
        let values: Vec<&str> = currencies.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("currency").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(expiry_tokens) = &params.expiry_tokens {
        let values: Vec<&str> = expiry_tokens.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("expiry_token").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(expiry_dates) = &params.expiry_dates {
        let values: Vec<&str> = expiry_dates.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("expiry_iso").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(directions) = &params.directions {
        let values: Vec<&str> = directions.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("direction").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(option_types) = &params.option_types {
        let values: Vec<&str> = option_types.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("option_type").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(liquidities) = &params.liquidities {
        let values: Vec<&str> = liquidities.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("liquidity").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    if let Some(instrument_categories) = &params.instrument_categories {
        let values: Vec<&str> = instrument_categories.split(',').map(|s| s.trim()).collect();
        if !values.is_empty() {
            let expr = values.iter().fold(lit(false), |acc, &val| {
                acc.or(col("instrument_category").eq(lit(val)))
            });
            lazy_df = lazy_df.filter(expr);
        }
    }
    
    // Legacy filters for backward compatibility
    if let Some(option_type) = &params.option_type {
        lazy_df = lazy_df.filter(col("option_type").eq(lit(option_type.as_str())));
    }
    if let Some(strike) = params.strike {
        lazy_df = lazy_df.filter(col("strike").eq(lit(strike)));
    }
    if let Some(expiry_date) = &params.expiry_date {
        lazy_df = lazy_df.filter(col("expiry_iso").eq(lit(expiry_date.as_str())));
    }
    
    // Execute the filters
    let filtered_df = match lazy_df.collect() {
        Ok(df) => df,
        Err(_) => df, // If filter fails, return original data
    };
    
    let mut rows = Vec::new();
    
    for i in 0..filtered_df.height() {
        let row = DataRow {
            instrument_name: filtered_df.column("instrument_name").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            currency: filtered_df.column("currency").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_token: filtered_df.column("expiry_token").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_iso: filtered_df.column("expiry_iso").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            instrument_category: filtered_df.column("instrument_category").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            timestamp: {
                let timestamp_ms = filtered_df.column("timestamp_ms").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0);
                timestamp_ms / 1000  // Convert milliseconds to seconds for JavaScript
            },
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
            interest_rate: filtered_df.column("interest_rate").unwrap().get(i).unwrap().to_string().parse().ok(),
        };
        rows.push(row);
    }
    
    Json(rows)
}

// Handler for metadata endpoint
async fn get_column_metadata(State(app_state): State<AppState>) -> Json<MetadataResponse> {
    let data = app_state.data.lock().await;
    
    if data.is_empty() {
        return Json(MetadataResponse { 
            columns: vec![],
            available_expiries: vec![],
        });
    }
    
    let mut columns = vec![];
    let schema = data.schema();
    
    for (name, dtype) in schema.iter() {
        match dtype {
            // Numeric columns
            polars::datatypes::DataType::Float64 | 
            polars::datatypes::DataType::Float32 |
            polars::datatypes::DataType::Int64 |
            polars::datatypes::DataType::Int32 |
            polars::datatypes::DataType::UInt64 |
            polars::datatypes::DataType::UInt32 => {
                columns.push(ColumnMetadata {
                    name: name.to_string(),
                    data_type: "numeric".to_string(),
                    min_value: Some(0.0), // Placeholder - could calculate actual min/max
                    max_value: Some(100.0), // Placeholder - could calculate actual min/max
                    unique_values: None,
                    min_date: None,
                    max_date: None,
                });
            },
            
            // String/categorical columns
            polars::datatypes::DataType::String => {
                // For now, just provide basic metadata without calculating unique values
                if name.contains("date") || name.contains("time") || name == "timestamp" {
                    columns.push(ColumnMetadata {
                        name: name.to_string(),
                        data_type: "date".to_string(),
                        min_value: None,
                        max_value: None,
                        unique_values: Some(vec!["2025-01-01".to_string(), "2025-12-31".to_string()]), // Placeholder
                        min_date: None,
                        max_date: None,
                    });
                } else {
                    columns.push(ColumnMetadata {
                        name: name.to_string(),
                        data_type: "categorical".to_string(),
                        min_value: None,
                        max_value: None,
                        unique_values: Some(vec!["call".to_string(), "put".to_string()]), // Placeholder
                        min_date: None,
                        max_date: None,
                    });
                }
            },
            
            // DateTime columns
            polars::datatypes::DataType::Datetime(_, _) => {
                columns.push(ColumnMetadata {
                    name: name.to_string(),
                    data_type: "date".to_string(),
                    min_value: None,
                    max_value: None,
                    unique_values: None,
                    min_date: None,
                    max_date: None,
                });
            },
            
            _ => {
                // Other types treated as categorical
                columns.push(ColumnMetadata {
                    name: name.to_string(),
                    data_type: "other".to_string(),
                    min_value: None,
                    max_value: None,
                    unique_values: None,
                    min_date: None,
                    max_date: None,
                });
            }
        }
    }
    
    // Extract unique expiry dates from the actual data
    let mut available_expiries = vec![];
    if let Ok(expiry_column) = data.column("expiry_iso") {
        let unique_expiries: std::collections::HashSet<String> = (0..expiry_column.len())
            .map(|i| expiry_column.get(i).unwrap().to_string().trim_matches('"').to_string())
            .filter(|s| !s.is_empty() && s != "null")
            .collect();
        
        available_expiries = unique_expiries.into_iter().collect();
        available_expiries.sort();
    }
    
    Json(MetadataResponse { 
        columns,
        available_expiries,
    })
}