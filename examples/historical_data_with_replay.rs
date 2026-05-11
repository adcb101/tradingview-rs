use colored::*;
use std::sync::Once;
use tradingview::{DataServer, Interval, history};

fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init();
    dotenv::dotenv().ok();

    // Print a colored header
    println!(
        "{}",
        "📈 TradingView Historical Data Fetcher 📉"
            .bright_green()
            .bold()
    );

    let auth_token = std::env::var("TV_AUTH_TOKEN").ok();
    let session = std::env::var("TV_SESSION").ok();
    let signature = std::env::var("TV_SIGNATURE").ok();

    if auth_token.is_none() && (session.is_none() || signature.is_none()) {
        anyhow::bail!("TV_AUTH_TOKEN or TV_SESSION+TV_SIGNATURE must be set");
    }

    let symbol = "XAUUSD";
    let exchange = "OANDA";
    let interval = Interval::OneHour;

    println!(
        "{} Fetching data for {} {} ({})",
        "→".bright_blue().bold(),
        symbol.yellow().bold(),
        exchange.cyan(),
        format!("{interval:?}").magenta(),
    );

    let (_info, data) = history::single::retrieve()
        .maybe_auth_token(auth_token.as_deref())
        .symbol(symbol)
        .exchange(exchange)
        .interval(interval)
        .with_replay(false)
        .server(DataServer::Data)
        .call()
        .await?;

    println!("{}", "✅ Data retrieved successfully!".green());
    println!("{}", "----------------------------------------".dimmed());
    println!(
        "{} Total data points: {}",
        "📊".bright_yellow(),
        data.len().to_string().bright_blue()
    );

    // // Print each data point with different colors
    // for (i, bar) in data.iter().rev().enumerate() {
    // println!(
    // "{} {} | Open: {} | High: {} | Low: {} | Close: {} | Volume: {}",
    // format!("[{}]", i).blue(),
    // format!("{}", bar.datetime()).bright_yellow().bold(),
    // format!("{:.2}", bar.open()).green(),
    // format!("{:.2}", bar.high()).bright_green(),
    // format!("{:.2}", bar.low()).red(),
    // format!("{:.2}", bar.close()).bright_cyan().bold(),
    // format!("{}", bar.volume()).magenta()
    // )                                                                 ;
    // }

    println!("{}", "----------------------------------------".dimmed());
    println!("{}", "Done!".bright_green().bold());

    Ok(())
}
