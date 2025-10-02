use serde_json::Value;

/// Apply creation date filters to instruments
pub fn filter_by_creation_date(
    instruments: Vec<Value>,
    creation_from: Option<&String>,
    creation_to: Option<&String>,
) -> Vec<Value> {
    let mut filtered = instruments;

    // Filter by creation_from
    if let Some(creation_from) = creation_from {
        if !creation_from.is_empty() {
            if let Ok(from_timestamp) =
                chrono::NaiveDate::parse_from_str(creation_from, "%Y-%m-%d")
            {
                let from_ts = from_timestamp
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .timestamp_millis();
                println!(
                    "Filtering creation_from: {} -> timestamp: {}",
                    creation_from, from_ts
                );
                let count_before = filtered.len();
                filtered.retain(|inst| {
                    if let Some(create_ts) = inst.get("creation_timestamp").and_then(|v| v.as_i64())
                    {
                        create_ts >= from_ts
                    } else {
                        true // Keep instruments without creation_timestamp
                    }
                });
                println!(
                    "After creation_from filter: {} -> {} instruments",
                    count_before,
                    filtered.len()
                );
            }
        }
    }

    // Filter by creation_to
    if let Some(creation_to) = creation_to {
        if !creation_to.is_empty() {
            if let Ok(to_timestamp) = chrono::NaiveDate::parse_from_str(creation_to, "%Y-%m-%d") {
                let to_ts = to_timestamp
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .timestamp_millis();
                println!(
                    "Filtering creation_to: {} -> timestamp: {}",
                    creation_to, to_ts
                );
                let count_before = filtered.len();
                filtered.retain(|inst| {
                    if let Some(create_ts) = inst.get("creation_timestamp").and_then(|v| v.as_i64())
                    {
                        create_ts <= to_ts
                    } else {
                        true // Keep instruments without creation_timestamp
                    }
                });
                println!(
                    "After creation_to filter: {} -> {} instruments",
                    count_before,
                    filtered.len()
                );
            }
        }
    }

    filtered
}

/// Print debug information about sample instruments
pub fn print_sample_instruments(instruments: &[Value], count: usize) {
    println!("Sample instruments:");
    for (i, inst) in instruments.iter().take(count).enumerate() {
        if let Some(name) = inst.get("instrument_name").and_then(|v| v.as_str()) {
            if let Some(exp_ts) = inst.get("expiration_timestamp").and_then(|v| v.as_i64()) {
                let exp_date = chrono::NaiveDateTime::from_timestamp_millis(exp_ts);
                println!(
                    "  [{}] {} expires at {} (ts: {})",
                    i,
                    name,
                    exp_date
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or("N/A".to_string()),
                    exp_ts
                );
            }
        }
    }
}
