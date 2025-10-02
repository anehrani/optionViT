use actix_web::{HttpResponse, Result};

/// Serve the main HTML page with embedded CSS and JavaScript
pub async fn index() -> Result<HttpResponse> {
    let html = get_html_content();
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// Returns the complete HTML content for the application
/// This includes all CSS styles and JavaScript for the two-part filtering system
fn get_html_content() -> &'static str {
    r#"<!DOCTYPE html>
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
        
        <!-- PART 1: DATA FETCHING SECTION -->
        <div class="fetch-section">
            <div class="filters-title">📊 Step 1: Fetch Data from API</div>
            <p style="color: #64748b; margin-bottom: 20px; font-size: 14px;">Fetch options data from Deribit. Use creation date to limit the data volume.</p>
            
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
                    <label>📅 Creation Date Range <span style="opacity: 0.6; font-weight: 400;">(Optional)</span></label>
                    <div class="range-inputs">
                        <input type="date" id="creationFrom" placeholder="From" title="Leave empty to include all" autocomplete="off">
                        <input type="date" id="creationTo" placeholder="To" title="Leave empty to include all" autocomplete="off">
                    </div>
                    <small style="color: #64748b; font-size: 11px; margin-top: 4px; display: block;">💡 Filters options by when they were created</small>
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
                <button class="btn-reset" onclick="resetFilters()">🔄 Reset All</button>
            </div>
        </div>
        
        <!-- PART 2: CLIENT-SIDE FILTERING SECTION -->
        <div class="fetch-section" id="filterSection" style="display: none;">
            <div class="filters-title">🔍 Step 2: Filter Fetched Data</div>
            <p style="color: #64748b; margin-bottom: 20px; font-size: 14px;">Apply filters to the fetched data instantly (no additional API calls).</p>
            
            <div class="form-grid">
                <div class="form-group">
                    <label for="optionType">📈 Option Type</label>
                    <select id="optionType" onchange="applyClientFilters()">
                        <option value="all">All (Calls & Puts)</option>
                        <option value="call">Calls Only</option>
                        <option value="put">Puts Only</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label>⏰ Expiration Date Range</label>
                    <div class="range-inputs">
                        <input type="date" id="expiryFrom" placeholder="From" onchange="applyClientFilters()">
                        <input type="date" id="expiryTo" placeholder="To" onchange="applyClientFilters()">
                    </div>
                </div>
                
                <div class="form-group">
                    <label>📊 Open Interest Range</label>
                    <div class="range-inputs">
                        <input type="number" id="oiMin" placeholder="Min" step="0.01" onchange="applyClientFilters()">
                        <input type="number" id="oiMax" placeholder="Max" step="0.01" onchange="applyClientFilters()">
                    </div>
                </div>
                
                <div class="form-group">
                    <label>📈 24h Volume Range</label>
                    <div class="range-inputs">
                        <input type="number" id="volumeMin" placeholder="Min" step="0.01" onchange="applyClientFilters()">
                        <input type="number" id="volumeMax" placeholder="Max" step="0.01" onchange="applyClientFilters()">
                    </div>
                </div>
                
                <div class="form-group">
                    <label>💹 IV % Range</label>
                    <div class="range-inputs">
                        <input type="number" id="ivMin" placeholder="Min" step="0.1" onchange="applyClientFilters()">
                        <input type="number" id="ivMax" placeholder="Max" step="0.1" onchange="applyClientFilters()">
                    </div>
                </div>
            </div>
            
            <div class="button-group">
                <button class="btn-fetch" onclick="applyClientFilters()" style="background: linear-gradient(135deg, #059669 0%, #047857 100%);">✓ Apply Filters</button>
                <button class="btn-reset" onclick="clearClientFilters()">🔄 Clear Filters</button>
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
        let allFetchedData = [];
        let filteredData = [];
        
        function resetFilters() {
            document.getElementById('currency').value = 'BTC';
            document.getElementById('creationFrom').value = '';
            document.getElementById('creationTo').value = '';
            document.getElementById('includeExpired').checked = false;
            clearClientFilters();
            document.getElementById('filterSection').style.display = 'none';
            allFetchedData = [];
            filteredData = [];
            document.getElementById('tableContainer').innerHTML = '';
            document.getElementById('stats').style.display = 'none';
        }
        
        function clearClientFilters() {
            document.getElementById('optionType').value = 'all';
            document.getElementById('expiryFrom').value = '';
            document.getElementById('expiryTo').value = '';
            document.getElementById('oiMin').value = '';
            document.getElementById('oiMax').value = '';
            document.getElementById('volumeMin').value = '';
            document.getElementById('volumeMax').value = '';
            document.getElementById('ivMin').value = '';
            document.getElementById('ivMax').value = '';
            if (allFetchedData.length > 0) {
                applyClientFilters();
            }
        }
        
        async function fetchData() {
            const currency = document.getElementById('currency').value;
            const creationFrom = document.getElementById('creationFrom').value;
            const creationTo = document.getElementById('creationTo').value;
            const includeExpired = document.getElementById('includeExpired').checked;
            
            const loading = document.getElementById('loading');
            const tableContainer = document.getElementById('tableContainer');
            const errorDiv = document.getElementById('error');
            const statsDiv = document.getElementById('stats');
            const fetchBtn = document.getElementById('fetchBtn');
            const filterSection = document.getElementById('filterSection');
            
            loading.style.display = 'block';
            tableContainer.innerHTML = '';
            errorDiv.style.display = 'none';
            statsDiv.style.display = 'none';
            filterSection.style.display = 'none';
            fetchBtn.disabled = true;
            
            try {
                const params = new URLSearchParams({
                    currency: currency,
                    include_expired: includeExpired
                });
                
                if (creationFrom) params.append('creation_from', creationFrom);
                if (creationTo) params.append('creation_to', creationTo);
                
                console.log('Fetching data from API with params:', params.toString());
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
                    allFetchedData = data.result;
                    console.log('✅ Fetched', allFetchedData.length, 'instruments from API');
                    filterSection.style.display = 'block';
                    applyClientFilters();
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
        
        function applyClientFilters() {
            if (allFetchedData.length === 0) {
                console.warn('No data to filter');
                return;
            }
            
            console.log('🔍 Applying client-side filters to', allFetchedData.length, 'instruments');
            
            const optionType = document.getElementById('optionType').value;
            const expiryFrom = document.getElementById('expiryFrom').value;
            const expiryTo = document.getElementById('expiryTo').value;
            const oiMin = document.getElementById('oiMin').value;
            const oiMax = document.getElementById('oiMax').value;
            const volumeMin = document.getElementById('volumeMin').value;
            const volumeMax = document.getElementById('volumeMax').value;
            const ivMin = document.getElementById('ivMin').value;
            const ivMax = document.getElementById('ivMax').value;
            
            filteredData = allFetchedData.filter(item => {
                if (optionType && optionType !== 'all') {
                    const instrumentName = item.instrument_name || '';
                    const isCall = instrumentName.endsWith('-C');
                    const isPut = instrumentName.endsWith('-P');
                    if (optionType === 'call' && !isCall) return false;
                    if (optionType === 'put' && !isPut) return false;
                }
                
                if (expiryFrom || expiryTo) {
                    const expTs = item.expiration_timestamp;
                    if (!expTs) return false;
                    
                    if (expiryFrom) {
                        const fromDate = new Date(expiryFrom);
                        fromDate.setHours(0, 0, 0, 0);
                        if (expTs < fromDate.getTime()) return false;
                    }
                    
                    if (expiryTo) {
                        const toDate = new Date(expiryTo);
                        toDate.setHours(23, 59, 59, 999);
                        if (expTs > toDate.getTime()) return false;
                    }
                }
                
                if (oiMin || oiMax) {
                    const oi = parseFloat(item.open_interest);
                    if (isNaN(oi)) return false;
                    if (oiMin && oi < parseFloat(oiMin)) return false;
                    if (oiMax && oi > parseFloat(oiMax)) return false;
                }
                
                if (volumeMin || volumeMax) {
                    const volume = parseFloat(item.volume_24h);
                    if (isNaN(volume)) return false;
                    if (volumeMin && volume < parseFloat(volumeMin)) return false;
                    if (volumeMax && volume > parseFloat(volumeMax)) return false;
                }
                
                if (ivMin || ivMax) {
                    const iv = parseFloat(item.mark_iv);
                    if (isNaN(iv)) return false;
                    const ivPercent = iv * 100;
                    if (ivMin && ivPercent < parseFloat(ivMin)) return false;
                    if (ivMax && ivPercent > parseFloat(ivMax)) return false;
                }
                
                return true;
            });
            
            console.log('✅ After filtering:', filteredData.length, 'instruments match criteria');
            displayStats(filteredData);
            displayTable(filteredData);
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
            filteredData.sort((a, b) => {
                const aVal = a[key];
                const bVal = b[key];
                
                if (aVal === null || aVal === undefined) return 1;
                if (bVal === null || bVal === undefined) return -1;
                
                if (typeof aVal === 'number' && typeof bVal === 'number') {
                    return bVal - aVal;
                }
                
                return String(aVal).localeCompare(String(bVal));
            });
            
            displayTable(filteredData);
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
            document.getElementById('creationFrom').value = '';
            document.getElementById('creationTo').value = '';
            clearClientFilters();
            console.log('👋 Ready! Click "Fetch Options Data" to begin.');
        });
    </script>
</body>
</html>"#
}
