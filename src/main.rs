use dotenv::dotenv;
pub mod supervisor;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::logger::initilize()?;
    dotenv().ok();

    //

    Ok(())
}
