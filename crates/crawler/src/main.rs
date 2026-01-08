use archive_crawler::Crawler;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("ArchiveStream Crawler v0.1.0 starting...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:password@localhost/archivestream".into());
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let crawler = Crawler::new(pool);
    
    // Seed for demo
    let _ = crawler.add_url("https://example.com").await;

    crawler.run().await?;

    Ok(())
}
