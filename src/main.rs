use dotenv::dotenv;
pub mod bot;
pub mod bot_handler;
pub mod supervisor;
pub mod utils;
use bot::TelegramBotService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::logger::initilize()?;
    dotenv().ok();
    let telegram_bot_service = TelegramBotService::new();
    let _ = telegram_bot_service.initialize().await;

    Ok(())
}
