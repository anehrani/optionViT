# 🔍 Deribit Options Data Fetcher

A high-performance web application for fetching and filtering cryptocurrency options data from Deribit with a unique two-part filtering system.

## 🎯 Key Features

- **Two-Part Filtering System**: Fetch data once from API, filter instantly in browser
- **Greeks Analysis & Portfolio Risk**: Real-time calculation of collective Delta, Gamma, and price projections
- **Real-Time Market Data**: Fetches live options data including OI, Volume, IV, and prices
- **Advanced Filtering**: Filter by expiration date, open interest, volume, IV%, and option type
- **Price Movement Scenarios**: P&L projections for ±1% and ±5% underlying price moves
- **Beautiful Modern UI**: Gradient design with smooth animations and responsive layout
- **Zero Latency Filtering**: Client-side filtering provides instant results without API calls

## 🏗️ Architecture

### Two-Part Filtering System

The application uses a unique two-part approach to optimize performance and user experience:

#### **Part 1: Data Fetching (Server-Side)**
- **Purpose**: Fetch options data once from Deribit API
- **Filters Available**:
  - 💰 Currency (BTC, ETH, SOL, USDC)
  - 📅 Creation Date Range (Optional)
  - ☑️ Include Expired Options
- **Action**: Click "🚀 Fetch Options Data"
- **Duration**: 10-30 seconds (depending on data volume)

#### **Part 2: Client-Side Filtering (Instant)**
- **Purpose**: Filter fetched data instantly in the browser
- **Filters Available**:
  - 📈 Option Type (All / Calls / Puts)
  - ⏰ Expiration Date Range
  - 📊 Open Interest Range
  - 📈 24h Volume Range
  - 💹 IV % Range
- **Action**: Changes apply automatically or click "🔍 Apply Filters"
- **Duration**: Instant (< 1ms)

### Why This Architecture?

**Problem**: Traditional approach requires a new API call for each filter change (30+ seconds per query)

**Solution**: Our two-part system:
1. Fetch comprehensive data once (single 30-second wait)
2. Apply unlimited filters instantly (no additional API calls)

**Benefits**:
- ✅ **99% faster filtering** - Instant results after initial fetch
- ✅ **No API rate limits** - Filter as many times as you want
- ✅ **Better UX** - Responsive, real-time filtering experience
- ✅ **Reduced server load** - Minimize API calls to Deribit

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/anehrani/optionViT.git
cd optionViT

# Build the project
cargo build --release

# Run the server
cargo run --release
```

The application will be available at `http://127.0.0.1:8080`

## 📖 How to Use

### Step-by-Step Guide

#### Step 1: Fetch Data from API

1. **Select Currency** (e.g., BTC)
2. **(Optional)** Set Creation Date Range to limit data volume
   - Leave empty to fetch all options
   - Use specific dates to reduce fetch time
3. **(Optional)** Check "Include Expired Options"
4. **Click "🚀 Fetch Options Data"**
5. **Wait** 10-30 seconds for data to load

The system will fetch and enrich all matching options with:
- Open Interest
- 24-hour Volume
- Mark IV (Implied Volatility)
- Bid/Ask/Last/Mark Prices

#### Step 2: Filter Data Instantly

Once data is fetched, the filter section appears automatically:

1. **Option Type**: Select All, Calls Only, or Puts Only
2. **Expiration Date**: Filter by expiry date range (e.g., 2025-10-03)
3. **Open Interest**: Set minimum/maximum OI thresholds
4. **24h Volume**: Filter by trading volume
5. **IV %**: Filter by implied volatility percentage

All filters apply **instantly** without any API calls!

### Example Workflow

```
Scenario: Find BTC call options expiring Oct 3rd with OI > 10

1. Part 1 - Fetch:
   ├─ Currency: BTC
   ├─ Creation Date: (leave empty)
   └─ Click "Fetch Options Data"
   ✅ Result: 764 instruments fetched in 25 seconds

2. Part 2 - Filter (Instant):
   ├─ Option Type: Calls Only
   ├─ Expiry: 2025-10-03 to 2025-10-03
   └─ OI Min: 10
   ✅ Result: 24 matching options (instant!)

3. Adjust filters:
   ├─ Change OI Min: 50
   ✅ Result: 12 options (instant!)
   ├─ Add Volume Min: 5
   ✅ Result: 8 options (instant!)
```

All filtering in Part 2 happens **instantly** - try different combinations without waiting!

## 🎨 User Interface

### Visual Design

- **Modern Gradient Theme**: Professional blue-purple gradient background
- **Card-Based Layout**: Clean, organized sections with rounded corners
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Animated Elements**: Smooth transitions and hover effects
- **Color-Coded Sections**: 
  - Blue gradient for data fetching
  - Green gradient for client-side filters

### Key UI Elements

1. **Data Fetching Section** (Step 1)
   - Blue header with gradient
   - Minimal, essential controls
   - Large "Fetch Options Data" button

2. **Filtering Section** (Step 2)
   - Appears after data is fetched
   - Green "Apply Filters" button
   - All filters with clear labels

3. **Statistics Cards**
   - Total Instruments
   - Calls / Puts ratio
   - Total Open Interest
   - 24h Volume
   - Instruments with Market Data

4. **Data Table**
   - Sortable columns
   - Highlighted important columns (OI, Volume, IV)
   - Alternating row colors
   - Hover effects for better readability

## 🔧 Technical Details

### Backend (Rust)

**Framework**: Actix-web  
**API Endpoint**: `/api/options`

**Query Parameters**:
- `currency`: String (required) - BTC, ETH, SOL, or USDC
- `creation_from`: String (optional) - ISO date format (YYYY-MM-DD)
- `creation_to`: String (optional) - ISO date format (YYYY-MM-DD)
- `include_expired`: Boolean (optional) - Include expired options

**Response Format**:
```json
{
  "result": [
    {
      "instrument_name": "BTC-3OCT25-100000-C",
      "strike": 100000,
      "expiration_timestamp": 1759478400000,
      "creation_timestamp": 1759392000000,
      "open_interest": 15.5,
      "volume_24h": 2.3,
      "mark_iv": 0.8542,
      "last_price": 0.0125,
      "mark_price": 0.0128,
      "bid_price": 0.0120,
      "ask_price": 0.0135
    }
    // ... more instruments
  ]
}
```

### Frontend (Vanilla JavaScript)

**No Framework Dependencies** - Pure JavaScript for maximum performance

**Key Functions**:
- `fetchData()` - Fetches data from API (Part 1)
- `applyClientFilters()` - Applies filters client-side (Part 2)
- `displayTable()` - Renders the data table
- `displayStats()` - Shows statistics cards

**Client-Side Filtering Logic**:
```javascript
filteredData = allFetchedData.filter(item => {
  // Option Type filter
  // Expiration Date filter
  // Open Interest filter
  // Volume filter
  // IV filter
  return true; // if passes all filters
});
```

## 📊 Data Source

All data is fetched from **Deribit's Public API**:

- **Instruments API**: `GET /api/v2/public/get_instruments`
- **Ticker API**: `GET /api/v2/public/ticker`

No API key required for public endpoints.

## ⚡ Performance Optimization

### Server-Side
- **Async/Await**: Non-blocking I/O for concurrent requests
- **Futures**: Parallel fetching of ticker data for up to 100 instruments
- **Minimal Filtering**: Only creation date filtering server-side

### Client-Side
- **In-Memory Filtering**: All filters applied on cached data
- **Efficient Algorithms**: Filter operations in O(n) time
- **No DOM Manipulation**: Updates only when filters change
- **Debouncing**: Prevents excessive re-renders

## 🛠️ Development

### Project Structure

```
optionViT/
├── src/
│   ├── main.rs          # Main application with embedded HTML/CSS/JS
│   ├── lib.rs           # Library exports
│   └── data_utils.rs    # Data utility functions (if needed)
├── Cargo.toml           # Rust dependencies
├── Cargo.lock           # Locked dependencies
└── README.md            # This file
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with auto-reload (requires cargo-watch)
cargo watch -x run
```

### Dependencies

**Rust Crates**:
- `actix-web` - Web framework
- `actix-cors` - CORS middleware
- `reqwest` - HTTP client
- `serde` & `serde_json` - Serialization
- `futures` - Async runtime
- `chrono` - Date/time handling

## 📝 API Filtering Details

### Creation Date Filter (Server-Side)

The creation date filter helps reduce the initial data volume:

```
Without filter: Fetches ALL available options (~1000+ instruments)
With filter: Fetches only options created in date range (~100-500 instruments)
```

**Example**:
```
creation_from=2025-09-01&creation_to=2025-10-02
→ Only fetches options created between Sept 1 and Oct 2
```

**Tip**: Leave empty to fetch all options, then use client-side filters for maximum flexibility.

### Expiration Date Filter (Client-Side)

Filters by when the option expires (not when it was created):

```javascript
// Example: Options expiring on October 3, 2025
expiryFrom: "2025-10-03"
expiryTo: "2025-10-03"
```

**Timestamp Comparison**:
- Converts user date to milliseconds
- Compares with `expiration_timestamp` from API
- Inclusive range (includes start and end dates)

### Open Interest Filter (Client-Side)

Filters by current open interest in contracts:

```javascript
oiMin: 10    // Minimum 10 contracts open interest
oiMax: 1000  // Maximum 1000 contracts open interest
```

**Note**: Instruments without OI data are excluded when filter is active.

### Volume Filter (Client-Side)

Filters by 24-hour trading volume:

```javascript
volumeMin: 5   // Minimum 5 contracts traded in 24h
volumeMax: 100 // Maximum 100 contracts traded in 24h
```

### IV Filter (Client-Side)

Filters by implied volatility percentage:

```javascript
ivMin: 50   // Minimum 50% IV
ivMax: 150  // Maximum 150% IV
```

**Note**: API returns IV as decimal (0.5 = 50%), automatically converted to percentage.

## 🔍 Troubleshooting

### Issue: No Data Returned

**Possible Causes**:
1. Creation date filter too restrictive
2. Network connectivity issues
3. Deribit API temporarily unavailable

**Solutions**:
1. Clear creation date filters and try again
2. Check console for error messages
3. Verify internet connection

### Issue: Slow Performance

**Possible Causes**:
1. Too many instruments fetched (>500)
2. Browser memory constraints

**Solutions**:
1. Use creation date filter to limit data
2. Refresh page to clear cached data
3. Disable "Include Expired" checkbox

### Issue: Filters Not Working

**Possible Causes**:
1. No data fetched yet
2. All data filtered out by criteria

**Solutions**:
1. Ensure Step 1 (Fetch Data) completed successfully
2. Check filter values are reasonable
3. Use "Clear Filters" button to reset

## 🚦 Best Practices

### For Best Performance

1. **Start with Creation Date Filter**: Limit initial data fetch
2. **Fetch Once, Filter Many**: Use client-side filters extensively
3. **Reasonable Ranges**: Don't set impossible filter combinations
4. **Clear Data**: Reset when switching currencies

### For Accurate Results

1. **Understand Filter Order**: Filters are cumulative (AND logic)
2. **Check Timestamps**: Expiration date is in UTC
3. **OI Can Be Zero**: Some options have no open interest
4. **IV Can Be Null**: Not all options have IV data

## 📈 Future Enhancements

- [ ] Add price range filters
- [ ] Export filtered data to CSV
- [ ] Save filter presets
- [ ] Real-time data updates via WebSocket
- [ ] Advanced charting and visualization
- [ ] Greeks calculation (Delta, Gamma, Theta, Vega)
- [ ] Profit/Loss calculator
- [ ] Multiple currency comparison

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is open source and available under the MIT License.

## 🔗 Links

- **Deribit API Documentation**: https://docs.deribit.com/
- **Actix-web Documentation**: https://actix.rs/
- **Repository**: https://github.com/anehrani/optionViT

## 💬 Support

For issues, questions, or suggestions, please open an issue on GitHub.

---

**Built with ❤️ using Rust and Modern Web Technologies**
