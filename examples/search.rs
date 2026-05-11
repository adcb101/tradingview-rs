use anyhow::Result;
use tradingview::{list_symbols, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let symbols: Vec<tradingview::Symbol> = list_symbols().market_type(MarketType::All).call().await?;

    println!("{symbols:?}");
    Ok(())
}
