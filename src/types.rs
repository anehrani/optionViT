use serde::{Deserialize, Serialize};

/// Request parameters for fetching options data (simplified two-part filtering)
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsRequest {
    pub currency: String,
    pub creation_from: Option<String>,
    pub creation_to: Option<String>,
    pub include_expired: Option<bool>,
}

/// Legacy request structure (kept for backwards compatibility)
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsRequestOld {
    pub currency: String,
    pub creation_from: Option<String>,
    pub creation_to: Option<String>,
    pub expiry_from: Option<String>,
    pub expiry_to: Option<String>,
    pub oi_min: Option<f64>,
    pub oi_max: Option<f64>,
    pub option_type: Option<String>,
    pub include_expired: Option<bool>,
}
