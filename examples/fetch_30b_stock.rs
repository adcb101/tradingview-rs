use chrono::FixedOffset;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};
use tradingview::{
    ChartOptions, Interval, StudyOptions,
    chart::DataPoint,
    live::{
        handler::{
            command::CommandRunner,
            message::{Command, TradingViewResponse},
            types::{CommandTx, DataRx},
        },
        models::DataServer,
        websocket::WebSocketClient,
    },
    pine_indicator::ScriptType,
};

/// Custom indicator ID (replace with your private or public indicator)
/// Public indicator format: "PUB;xxx", private indicator format: "USER;xxx"
const INDICATOR_ID: &str = "USER;d57400f28bfa4b04ab5ea606dd5219d2";
const INDICATOR_VERSION: &str = "last";

/// US Magnificent Seven + AI/Semiconductor/Optical/Storage
const TARGETS: &[(&str, &str)] = &[
    ("AAPL", "NASDAQ"),
    ("MSFT", "NASDAQ"),
    ("GOOGL", "NASDAQ"),
    ("AMZN", "NASDAQ"),
    ("NVDA", "NASDAQ"),
    ("META", "NASDAQ"),
    ("TSLA", "NASDAQ"),
    ("AMD", "NASDAQ"),
    ("AVGO", "NASDAQ"),
    ("TSM", "NYSE"),
    ("MRVL", "NASDAQ"),
    ("ARM", "NASDAQ"),
    ("AMAT", "NASDAQ"),
    ("LRCX", "NASDAQ"),
    ("MU", "NASDAQ"),
    ("SNDK", "NASDAQ"),
    ("LITE", "NASDAQ"),
    ("QCOM", "NASDAQ"),
    ("NBIS", "NASDAQ"),
    ("RKLB", "NASDAQ"),
    ("HSBC", "NYSE"),
];

fn timestamp_to_shanghai(timestamp: i64) -> String {
    let offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.with_timezone(&offset))
        .unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let auth_token = std::env::var("TV_AUTH_TOKEN").ok();
    let session = std::env::var("TV_SESSION").ok();
    let signature = std::env::var("TV_SIGNATURE").ok();

    if auth_token.is_none() && (session.is_none() || signature.is_none()) {
        anyhow::bail!("TV_AUTH_TOKEN or TV_SESSION+TV_SIGNATURE must be set");
    }

    // Single connection + single CommandRunner (following indicator.rs pattern)
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();

    // WS auth via JWT (set_auth_token), HTTP auth via session/signature (Cookie)
    let ws_client = WebSocketClient::builder()
        .maybe_auth_token(auth_token.as_deref())
        .maybe_session(session.as_deref())
        .maybe_signature(signature.as_deref())
        .server(DataServer::Data)
        .data_tx(response_tx)
        .build()
        .await?;

    let command_runner = CommandRunner::new(command_rx, Arc::clone(&ws_client));

    let runner_handle = tokio::spawn(async move {
        if let Err(e) = command_runner.run().await {
            error!("Command runner failed: {}", e);
        }
    });

    // Main logic: reuse the same connection, fetch symbols serially
    info!("Fetching indicator data for {} stocks...", TARGETS.len());

    let mut success = 0u32;
    for (symbol, exchange) in TARGETS {
        //info!("Fetching {}:{} ...", exchange, symbol);

        match fetch_one(&command_tx, &mut response_rx, symbol, exchange).await {
            Ok(_) => success += 1,
            Err(e) => warn!("{}:{} failed: {}", exchange, symbol, e),
        }

        // Drain residual messages to avoid polluting the next symbol
        while response_rx.try_recv().is_ok() {}
    }

    info!("Done! {}/{} succeeded", success, TARGETS.len());

    // Graceful shutdown: Command::Delete handles complete cleanup internally
    //info!("Cleaning up connection...");
    let _ = command_tx.send(Command::Delete);
    let _ = tokio::time::timeout(Duration::from_secs(10), runner_handle).await;

    //info!("Exiting");
    Ok(())
}

async fn fetch_one(
    command_tx: &CommandTx,
    response_rx: &mut DataRx,
    symbol: &str,
    exchange: &str,
) -> anyhow::Result<()> {
    let opts = ChartOptions::builder()
        .symbol((*symbol).into())
        .exchange((*exchange).into())
        .interval(Interval::ThirtyMinutes)
        .bar_count(10)
        .study_config(StudyOptions {
            script_id: INDICATOR_ID.into(),
            script_version: INDICATOR_VERSION.into(),
            script_type: ScriptType::IntervalScript,
        })
        .build();

    command_tx.send(Command::set_market(opts))?;

    // Build expected symbol_id for matching
    let expected_id = format!("{exchange}:{symbol}");

    // Wait for StudyData (confirm SymbolInfo match first, then receive StudyData)
    let periods = tokio::time::timeout(Duration::from_secs(15), async {
        let mut resolved = false;
        while let Some(response) = response_rx.recv().await {
            match response {
                TradingViewResponse::StudyData(_, ref study_data) => {
                    if resolved && study_data.studies.len() >= 3 {
                        //info!("Received StudyData: {} data points", study_data.studies.len());
                        return study_data.studies.clone();
                    }
                }
                TradingViewResponse::StudyCompleted(_) => {
                    //info!("{}:{} indicator loaded", exchange, symbol);
                }
                TradingViewResponse::Error(err, _) => {
                    warn!("{}:{} received error: {:?}", exchange, symbol, err);
                    return Vec::new();
                }
                TradingViewResponse::SymbolInfo(ref info) => {
                    if info.id == expected_id {
                        resolved = true;
                        // info!("{}:{} resolved: {}", exchange, symbol, info.description);
                    }
                }
                other => {
                    debug!("{}:{} received other response: {:?}", exchange, symbol, other);
                }
            }
        }
        Vec::new()
    })
    .await
    .unwrap_or_default();

    if periods.is_empty() {
        return Err(anyhow::anyhow!("Timed out waiting for indicator data"));
    }

    process_signals(symbol, exchange, &periods);
    Ok(())
}

fn process_signals(symbol: &str, exchange: &str, studies: &[DataPoint]) {
    // Follow JS logic: skip latest candle (may not be closed), use previous vs pre-previous
    // JS: line=lines[1], preline=lines[2]
    // Data sorted oldest→newest, lines[0]=newest, lines[1]=previous, lines[2]=pre-previous
    if studies.len() < 3 {
        warn!("{}:{} insufficient data points ({})", exchange, symbol, studies.len());
        return;
    }

    let n = studies.len();
    let curline: &DataPoint = &studies[n - 1];
    let line: &DataPoint = &studies[n - 2];
    let preline = &studies[n - 3];

    // $time = 1778167800
    // Close = 289.3
    // High = 290.74
    // LLT_High = 291.43259083057944
    // LLT_Low = 289.9538108019135
    // Low = 289.12
    // Open = 290.21
    // Short_Down_Trend = 1e+100
    // Short_Up_Trend = 286.1950364201943
    // VWAP = 290.3731292659404
    // Print all values of the line object
    //println!("line {{ index: {}, value: {:?} }}", line.index, line.value);

    let time = line.value.first().copied().unwrap_or(0.0) as i64;
    let localtime = timestamp_to_shanghai(time);

    let curtime = curline.value.first().copied().unwrap_or(0.0) as i64;
    let curlocaltime = timestamp_to_shanghai(curtime);
    // Indicator value indices (depends on plot order, adjust accordingly):

    // $time =1778167800
    // Close = 289.3
    // High = 290.74
    // LLT_High = 291.43259083057944
    // LLT_Low = 289.9538108019135
    // Low = 289.12
    // Open = 290.21
    // Short_Down_Trend = 1e+100
    // Short_Up_Trend = 286.1950364201943
    // VWAP =290.3731292659404
    //[1778167800.0, 286.1950364201943, 1e100, 290.3731292659404, 289.9538108019135, 291.43259083057944, 290.74, 289.12, 289.3, 290.21]
    let close = line.value.get(8).copied().unwrap_or(0.0);
    let cur_close = curline.value.get(8).copied().unwrap_or(0.0);

    let mut cur_up_trend = line.value.get(1).copied().unwrap_or(1e100);
    let mut cur_down_trend = line.value.get(2).copied().unwrap_or(1e100);

    let mut short_up_trend = preline.value.get(1).copied().unwrap_or(1e100);
    let mut short_down_trend = preline.value.get(2).copied().unwrap_or(1e100);
    if short_down_trend == 1e100 {
        short_down_trend = 0.0;
    }
    if short_up_trend == 1e100 {
        short_up_trend = 0.0;
    }
    if cur_down_trend == 1e100 {
        cur_down_trend = 0.0;
    }
    if cur_up_trend == 1e100 {
        cur_up_trend = 0.0;
    }
    // info!(
    //     "TIME:{} {} {} {:.2} {:.2}",
    //     localtime, symbol, close, short_up_trend, short_down_trend
    // );

    if close > short_down_trend && short_down_trend > 0.0 {
        info!("UP_ {} {}:{}", localtime, exchange, symbol);
    }
    if close < short_up_trend {
        info!("Down_ {} {}:{}", localtime, exchange, symbol);
    }
    if cur_close > cur_down_trend && cur_down_trend > 0.0 {
        info!("UPing_ {} {}:{}", curlocaltime, exchange, symbol);
    }
    if cur_close < cur_up_trend {
        info!("Downing_ {} {}:{}", curlocaltime, exchange, symbol);
    }
}
