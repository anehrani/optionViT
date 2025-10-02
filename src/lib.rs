// Module declarations
pub mod api_client;
pub mod filters;
pub mod greeks;
pub mod handlers;
pub mod types;
pub mod ui;

// Re-exports for convenience
pub use api_client::DeribitClient;
pub use filters::{filter_by_creation_date, print_sample_instruments};
pub use greeks::{calculate_collective_greeks, calculate_price_projection, CollectiveGreeks};
pub use handlers::get_options;
pub use types::{OptionsRequest, OptionsRequestOld};
pub use ui::index;