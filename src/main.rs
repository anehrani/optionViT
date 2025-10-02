use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use optionvit::{get_options, index};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting Deribit Options Data Fetcher");
    println!("📡 Server running at http://127.0.0.1:8080");
    println!("📖 Two-part filtering system: Fetch once, filter instantly!");
    println!();

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
