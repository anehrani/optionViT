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

#[derive(Deserialize)]
struct RefreshParams {
    expiry_date: String,
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
    println!("� Starting BTC Options Analysis Server...");
    println!("📊 Data will be loaded dynamically when you select an expiry date from the web interface.");
    
    // Initialize with empty DataFrame - data will be loaded via web interface
    unsafe {
        GLOBAL_DF = None;
    }
    
    // Start web server to display data
    println!("� Starting web server...");
    
    // Define routes
    let app = Router::new()
        .route("/", get(data_table_page))
        .route("/data", get(data_table_page))
        .route("/api/data", get(data_api_filtered));

    // Specify where to listen (localhost:3000)
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🌐 Server running on http://{}", listener.local_addr().unwrap());
    println!("� View data at: http://127.0.0.1:3000/data");

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("� Starting BTC Options Analysis Server...");
    println!("📊 Data will be loaded dynamically when you select an expiry date from the web interface.");
    
    // Initialize with empty DataFrame - data will be loaded via web interface
    unsafe {
        GLOBAL_DF = None;
    }
    
    // Start web server to display data
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
        .column-filter {
            width: 100%;
            padding: 4px 6px;
            border: 1px solid #ccc;
            border-radius: 3px;
            font-size: 11px;
            margin-top: 5px;
            box-sizing: border-box;
        }
        .column-filter:focus {
            border-color: #4CAF50;
            outline: none;
            box-shadow: 0 0 3px rgba(76, 175, 80, 0.3);
        }
        .filter-header {
            position: relative;
            min-width: 120px;
        }
        .filter-header-content {
            display: flex;
            flex-direction: column;
            align-items: center;
        }
        .header-title {
            cursor: pointer;
            font-weight: bold;
            margin-bottom: 5px;
        }
        .clear-column-filter {
            position: absolute;
            top: 22px;
            right: 2px;
            background: #f44336;
            color: white;
            border: none;
            border-radius: 2px;
            width: 16px;
            height: 16px;
            font-size: 10px;
            cursor: pointer;
            display: none;
        }
        .clear-column-filter:hover {
            background: #da190b;
        }
        .numeric-filter-container {
            display: flex;
            gap: 2px;
            width: 100%;
        }
        .numeric-operator {
            width: 35px;
            padding: 2px;
            border: 1px solid #ccc;
            border-radius: 3px;
            font-size: 10px;
        }
        .numeric-value {
            flex: 1;
            width: auto !important;
        }
        .categorical-filter {
            font-size: 11px;
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
                <li>Excel-like sortable and scrollable table view with column filters</li>
            </ul>
            <div style="margin-top: 15px; display: flex; gap: 15px; align-items: center; flex-wrap: wrap;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <label for="expirySelect" style="font-weight: bold; color: #333;">Expiry Date:</label>
                    <select id="expirySelect" style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; background-color: white; font-size: 14px;">
                        <option value="2025-09-30">30SEP25 (2025-09-30)</option>
                        <option value="2025-12-30">30DEC25 (2025-12-30)</option>
                        <option value="2026-03-31">31MAR26 (2026-03-31)</option>
                        <option value="2026-06-30">30JUN26 (2026-06-30)</option>
                        <option value="2026-09-30">30SEP26 (2026-09-30)</option>
                        <option value="2026-12-31">31DEC26 (2026-12-31)</option>
                    </select>
                </div>
                <button id="refreshBtn" onclick="refreshData()" style="padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: bold; background-color: #4CAF50; color: white;">🔄 Refresh Data</button>
                <button class="btn btn-secondary" onclick="clearFilters()" style="padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: bold; background-color: #f44336; color: white;">🧹 Clear All Filters</button>
            </div>
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
        let filteredData = [];

        // Analyze column types from data
        function analyzeColumnTypes(data) {
            const types = {};
            const sampleSize = Math.min(data.length, 100); // Sample first 100 rows
            
            Object.keys(data[0]).forEach(header => {
                const values = data.slice(0, sampleSize)
                    .map(row => row[header])
                    .filter(val => val !== null && val !== undefined && val !== '');
                
                if (values.length === 0) {
                    types[header] = 'text';
                    return;
                }
                
                // Check if numeric
                const numericValues = values.filter(val => !isNaN(parseFloat(val)) && isFinite(val));
                if (numericValues.length / values.length > 0.8) {
                    types[header] = 'numeric';
                    return;
                }
                
                // Check if categorical (limited unique values)
                const uniqueValues = [...new Set(values)];
                if (uniqueValues.length <= 20 && uniqueValues.length < values.length * 0.5) {
                    types[header] = 'categorical';
                    return;
                }
                
                // Default to text
                types[header] = 'text';
            });
            
            return types;
        }

        // Create appropriate filter element based on column type
        function createFilterElement(header, type, data) {
            if (type === 'numeric') {
                return createNumericFilter(header);
            } else if (type === 'categorical') {
                return createCategoricalFilter(header, data);
            } else {
                return createTextFilter(header);
            }
        }

        function createNumericFilter(header) {
            const container = document.createElement('div');
            container.className = 'numeric-filter-container';
            
            const operatorSelect = document.createElement('select');
            operatorSelect.className = 'numeric-operator';
            operatorSelect.id = `operator_${header}`;
            operatorSelect.innerHTML = `
                <option value="">=</option>
                <option value=">">&gt;</option>
                <option value=">=">&gt;=</option>
                <option value="<">&lt;</option>
                <option value="<=">&lt;=</option>
                <option value="!=">&ne;</option>
            `;
            
            const valueInput = document.createElement('input');
            valueInput.type = 'number';
            valueInput.className = 'column-filter numeric-value';
            valueInput.placeholder = 'Value...';
            valueInput.id = `filter_${header}`;
            valueInput.step = 'any';
            
            operatorSelect.onchange = () => applyColumnFilters();
            valueInput.oninput = () => applyColumnFilters();
            
            container.appendChild(operatorSelect);
            container.appendChild(valueInput);
            return container;
        }

        function createCategoricalFilter(header, data) {
            const select = document.createElement('select');
            select.className = 'column-filter categorical-filter';
            select.id = `filter_${header}`;
            select.onchange = () => applyColumnFilters();
            
            // Get unique values for this column
            const uniqueValues = [...new Set(data.map(row => row[header])
                .filter(val => val !== null && val !== undefined && val !== ''))];
            uniqueValues.sort();
            
            // Add default option
            const defaultOption = document.createElement('option');
            defaultOption.value = '';
            defaultOption.textContent = 'All';
            select.appendChild(defaultOption);
            
            // Add options for each unique value
            uniqueValues.forEach(value => {
                const option = document.createElement('option');
                option.value = value;
                option.textContent = value;
                select.appendChild(option);
            });
            
            return select;
        }

        function createTextFilter(header) {
            const input = document.createElement('input');
            input.type = 'text';
            input.className = 'column-filter';
            input.placeholder = 'Filter...';
            input.id = `filter_${header}`;
            input.oninput = () => applyColumnFilters();
            return input;
        }

        async function loadData() {
            try {
                const response = await fetch('/api/data');
                const data = await response.json();
                
                if (data.length === 0) {
                    document.getElementById('loading').innerHTML = '📊 No data available. Please check the server console for any errors during data loading.';
                    return;
                }
                
                allData = data;
                filteredData = data;
                
                // Analyze column types
                const columnTypes = analyzeColumnTypes(data);
                
                // Create table headers with column filters
                const headers = Object.keys(data[0]);
                const headerRow = document.getElementById('tableHeader');
                headerRow.innerHTML = '';
                headers.forEach(header => {
                    const th = document.createElement('th');
                    th.className = 'filter-header';
                    
                    const headerContent = document.createElement('div');
                    headerContent.className = 'filter-header-content';
                    
                    const headerTitle = document.createElement('div');
                    headerTitle.className = 'header-title';
                    headerTitle.textContent = header.replace(/_/g, ' ').toUpperCase();
                    headerTitle.onclick = () => sortTable(header);
                    
                    // Create appropriate filter based on column type
                    const filterElement = createFilterElement(header, columnTypes[header], data);
                    
                    const clearBtn = document.createElement('button');
                    clearBtn.className = 'clear-column-filter';
                    clearBtn.innerHTML = '×';
                    clearBtn.title = 'Clear filter';
                    clearBtn.onclick = () => clearColumnFilter(header);
                    
                    headerContent.appendChild(headerTitle);
                    headerContent.appendChild(filterElement);
                    th.appendChild(headerContent);
                    th.appendChild(clearBtn);
                    headerRow.appendChild(th);
                });
                
                // Populate table body
                populateTable(filteredData);
                
                // Show stats
                showStats(filteredData);
                
                // Hide loading and show table
                document.getElementById('loading').style.display = 'none';
                document.getElementById('dataTable').style.display = 'table';
                
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('loading').innerHTML = '❌ Error loading data: ' + error.message;
            }
        }

        function applyColumnFilters() {
            const headers = Object.keys(allData[0] || {});
            let currentFilteredData = allData;
            
            // Apply column filters
            headers.forEach(header => {
                const filterInput = document.getElementById(`filter_${header}`);
                const clearBtn = filterInput ? filterInput.closest('.filter-header').querySelector('.clear-column-filter') : null;
                
                if (filterInput) {
                    const isFiltered = applyColumnFilter(header, filterInput, currentFilteredData);
                    
                    if (isFiltered.hasFilter) {
                        currentFilteredData = isFiltered.filteredData;
                        // Show clear button
                        if (clearBtn) clearBtn.style.display = 'block';
                    } else {
                        // Hide clear button
                        if (clearBtn) clearBtn.style.display = 'none';
                    }
                }
            });
            
            filteredData = currentFilteredData;
            
            // Update table and stats
            populateTable(filteredData);
            showStats(filteredData);
            
            // Update info message
            const totalRows = allData.length;
            const filteredRows = filteredData.length;
            console.log(`Filtered ${filteredRows} of ${totalRows} rows`);
        }

        function applyColumnFilter(header, filterElement, data) {
            // Check if it's a numeric filter
            const operatorSelect = document.getElementById(`operator_${header}`);
            if (operatorSelect && filterElement.type === 'number') {
                return applyNumericFilter(header, operatorSelect, filterElement, data);
            }
            
            // Check if it's a categorical filter (select element)
            if (filterElement.tagName === 'SELECT' && filterElement.value.trim()) {
                const filterValue = filterElement.value;
                const filteredData = data.filter(row => {
                    const cellValue = row[header];
                    return cellValue === filterValue;
                });
                return { hasFilter: true, filteredData };
            }
            
            // Text filter
            if (filterElement.value && filterElement.value.trim()) {
                const filterValue = filterElement.value.toLowerCase().trim();
                const filteredData = data.filter(row => {
                    const cellValue = row[header];
                    if (cellValue === null || cellValue === undefined) return false;
                    return cellValue.toString().toLowerCase().includes(filterValue);
                });
                return { hasFilter: true, filteredData };
            }
            
            return { hasFilter: false, filteredData: data };
        }

        function applyNumericFilter(header, operatorSelect, valueInput, data) {
            const operator = operatorSelect.value;
            const filterValue = parseFloat(valueInput.value);
            
            if (isNaN(filterValue)) {
                return { hasFilter: false, filteredData: data };
            }
            
            const filteredData = data.filter(row => {
                const cellValue = parseFloat(row[header]);
                if (isNaN(cellValue)) return false;
                
                switch (operator) {
                    case '>': return cellValue > filterValue;
                    case '>=': return cellValue >= filterValue;
                    case '<': return cellValue < filterValue;
                    case '<=': return cellValue <= filterValue;
                    case '!=': return cellValue !== filterValue;
                    default: return cellValue === filterValue; // equals
                }
            });
            
            return { hasFilter: true, filteredData };
        }

        function clearColumnFilter(header) {
            const filterInput = document.getElementById(`filter_${header}`);
            const operatorSelect = document.getElementById(`operator_${header}`);
            
            if (filterInput) {
                if (filterInput.tagName === 'SELECT') {
                    filterInput.selectedIndex = 0; // Reset to first option (All)
                } else {
                    filterInput.value = '';
                }
            }
            
            if (operatorSelect) {
                operatorSelect.selectedIndex = 0; // Reset to equals
            }
            
            applyColumnFilters();
        }

        async function refreshData() {
            const expiryDate = document.getElementById('expirySelect').value;
            const refreshBtn = document.getElementById('refreshBtn');
            
            if (!expiryDate) {
                alert('Please select an expiry date first');
                return;
            }
            
            // Show informative message about how to change dates
            const currentlyLoaded = allData.length > 0 ? allData[0].expiry_iso || 'Unknown' : 'None';
            
            alert(`📊 Currently showing data for: ${currentlyLoaded}\\n\\nYou selected: ${expiryDate}\\n\\n🔄 To load data for ${expiryDate}:\\n1. Stop the server (Ctrl+C in terminal)\\n2. Edit main.rs line ~49: change expiry_date to "${expiryDate}"\\n3. Restart with: cargo run\\n\\nThis will load fresh ${expiryDate} data from Deribit API!\\n\\n💡 The refresh button will clear current filters and refresh the view.`);
            
            // Clear filters and refresh current data view
            clearAllFiltersQuiet();
            if (allData.length > 0) {
                populateTable(allData);
                showStats(allData);
            }
        }

        function clearAllFiltersQuiet() {
            // Clear all column filters without updating display
            const headers = Object.keys(allData[0] || {});
            headers.forEach(header => {
                const filterInput = document.getElementById(`filter_${header}`);
                const operatorSelect = document.getElementById(`operator_${header}`);
                
                if (filterInput) {
                    if (filterInput.tagName === 'SELECT') {
                        filterInput.selectedIndex = 0;
                    } else {
                        filterInput.value = '';
                    }
                }
                
                if (operatorSelect) {
                    operatorSelect.selectedIndex = 0;
                }
            });
            
            // Hide all clear buttons
            document.querySelectorAll('.clear-column-filter').forEach(btn => {
                btn.style.display = 'none';
            });
        }

        function clearFilters() {
            // Clear all column filters
            const headers = Object.keys(allData[0] || {});
            headers.forEach(header => {
                const filterInput = document.getElementById(`filter_${header}`);
                const operatorSelect = document.getElementById(`operator_${header}`);
                
                if (filterInput) {
                    if (filterInput.tagName === 'SELECT') {
                        filterInput.selectedIndex = 0;
                    } else {
                        filterInput.value = '';
                    }
                }
                
                if (operatorSelect) {
                    operatorSelect.selectedIndex = 0;
                }
            });
            
            filteredData = allData;
            populateTable(filteredData);
            showStats(filteredData);
            
            // Hide all clear buttons
            document.querySelectorAll('.clear-column-filter').forEach(btn => {
                btn.style.display = 'none';
            });
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
        
        let sortDirection = {};
        function sortTable(column) {
            // Toggle sort direction
            sortDirection[column] = sortDirection[column] === 'asc' ? 'desc' : 'asc';
            
            // Note: This is a simple client-side sort
            // For larger datasets, you'd want server-side sorting
            console.log(`Sorting by ${column} (${sortDirection[column]})`);
        }
        
        // Load data when page loads
        loadData();
    </script>
</body>
</html>
    "#)
}

/*
async fn data_api() -> Json<Vec<DataRow>> {
    let df = unsafe {
        match &GLOBAL_DF {
            Some(df_arc) => df_arc.lock().unwrap().clone(),
            None => return Json(vec![]),
        }
    };
    
    let mut rows = Vec::new();
    
    for i in 0..df.height() {
        let row = DataRow {
            instrument_name: df.column("instrument_name").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            currency: df.column("currency").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_token: df.column("expiry_token").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            expiry_iso: df.column("expiry_iso").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            timestamp_ms: df.column("timestamp_ms").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
            timestamp_utc: df.column("timestamp_utc").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            direction: df.column("direction").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            price: df.column("price").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
            amount: df.column("amount").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
            iv: df.column("iv").unwrap().get(i).unwrap().to_string().parse().ok(),
            index_price: df.column("index_price").unwrap().get(i).unwrap().to_string().parse().ok(),
            mark_price: df.column("mark_price").unwrap().get(i).unwrap().to_string().parse().ok(),
            trade_id: df.column("trade_id").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
            trade_seq: df.column("trade_seq").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
            block_trade_id: {
                let val = df.column("block_trade_id").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            liquidity: {
                let val = df.column("liquidity").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            tick_direction: df.column("tick_direction").unwrap().get(i).unwrap().to_string().parse().ok(),
            strike: df.column("strike").unwrap().get(i).unwrap().to_string().parse().ok(),
            option_type: {
                let val = df.column("option_type").unwrap().get(i).unwrap().to_string();
                if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
            },
            open_interest: df.column("open_interest").unwrap().get(i).unwrap().to_string().parse().ok(),
        };
        rows.push(row);
    }
    
    Json(rows)
}
*/

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

/*
async fn download_and_convert_data(expiry_date: &str) -> Vec<DataRow> {
    match download_deribit_data("BTC", expiry_date).await {
        Ok(new_df) => {
            println!("✅ Downloaded {} rows for {}", new_df.height(), expiry_date);
            
            // Update global DataFrame
            unsafe {
                GLOBAL_DF = Some(Arc::new(Mutex::new(new_df.clone())));
            }
            
            // Convert to DataRow format
            let mut rows = Vec::new();
            for i in 0..new_df.height() {
                let row = DataRow {
                    instrument_name: new_df.column("instrument_name").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    currency: new_df.column("currency").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    expiry_token: new_df.column("expiry_token").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    expiry_iso: new_df.column("expiry_iso").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    timestamp_ms: new_df.column("timestamp_ms").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
                    timestamp_utc: new_df.column("timestamp_utc").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    direction: new_df.column("direction").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    price: new_df.column("price").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
                    amount: new_df.column("amount").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0.0),
                    iv: new_df.column("iv").unwrap().get(i).unwrap().to_string().parse().ok(),
                    index_price: new_df.column("index_price").unwrap().get(i).unwrap().to_string().parse().ok(),
                    mark_price: new_df.column("mark_price").unwrap().get(i).unwrap().to_string().parse().ok(),
                    trade_id: new_df.column("trade_id").unwrap().get(i).unwrap().to_string().trim_matches('"').to_string(),
                    trade_seq: new_df.column("trade_seq").unwrap().get(i).unwrap().to_string().parse().unwrap_or(0),
                    block_trade_id: {
                        let val = new_df.column("block_trade_id").unwrap().get(i).unwrap().to_string();
                        if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
                    },
                    liquidity: {
                        let val = new_df.column("liquidity").unwrap().get(i).unwrap().to_string();
                        if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
                    },
                    tick_direction: new_df.column("tick_direction").unwrap().get(i).unwrap().to_string().parse().ok(),
                    strike: new_df.column("strike").unwrap().get(i).unwrap().to_string().parse().ok(),
                    option_type: {
                        let val = new_df.column("option_type").unwrap().get(i).unwrap().to_string();
                        if val == "null" { None } else { Some(val.trim_matches('"').to_string()) }
                    },
                    open_interest: new_df.column("open_interest").unwrap().get(i).unwrap().to_string().parse().ok(),
                };
                rows.push(row);
            }
            
            rows
        }
        Err(e) => {
            println!("❌ Error downloading data for {}: {}", expiry_date, e);
            vec![] // Return empty data on error
        }
    }
}
*/

/*
async fn refresh_endpoint(Query(params): Query<RefreshParams>) -> Json<serde_json::Value> {
    let expiry_date = params.expiry_date.clone();
    println!("🔄 Received refresh request for expiry date: {}", expiry_date);
    
    // Download data directly
    match download_deribit_data("BTC", &expiry_date).await {
        Ok(new_df) => {
            let rows_count = new_df.height();
            println!("✅ Downloaded {} rows for {}", rows_count, expiry_date);
            
            // Update global DataFrame
            unsafe {
                GLOBAL_DF = Some(Arc::new(Mutex::new(new_df)));
            }
            println!("✅ Global DataFrame updated successfully");
            
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Data refreshed successfully for {}", expiry_date),
                "expiry_date": expiry_date,
                "rows_loaded": rows_count
            }))
        }
        Err(_) => {
            println!("❌ Error downloading data for {}", expiry_date);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to refresh data for {}", expiry_date),
                "expiry_date": expiry_date
            }))
        }
    }
}
*/