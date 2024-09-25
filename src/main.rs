use netmate_api::startup::startup;
use time::{format_description::well_known::Rfc3339, UtcOffset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn main() {

    // UTCのフォーマットは分析ソフトが解釈しやすいよう、RFC3339のYYYY-MM-DDTHH-mm-ssZを採用する
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_target(true)
        .with_file(true)
        .with_timer(OffsetTime::new(
            UtcOffset::from_hms(9, 0, 0).unwrap(),
            Rfc3339
        ))
        .finish();


    tracing::subscriber::set_global_default(subscriber).unwrap();

    /*
    NTPが導入されている&時刻が正確であることを必ず確認する
    条件を満たさなければpanicで強制終了
    正しいUUIDv7はプラットフォームの基盤であり、ここが狂うと正常化が困難になる

    // NTP同期をチェック
    if let Err(err) = check_ntp_sync() {
        panic!("NTP sync check failed: {}", err);
    }
   */

    // initapp();
    startup().await;

  // init app : startup mod
  // start up : startup mod

  /*
  #[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_app();

    let modules = Modules::new().await;
    let _ = startup(Arc::new(modules)).await;

    Ok(())
} */


}

/*
NTP:

use std::net::SocketAddr;
use std::process::Command;

fn check_ntp_sync() -> Result<(), String> {
    // `ntpstat`コマンドを実行
    let output = Command::new("ntpstat")
        .output()
        .map_err(|e| format!("Failed to execute ntpstat: {}", e))?;
    
    // コマンドの出力を文字列として取得
    let output_str = String::from_utf8_lossy(&output.stdout);

    // 出力に "synchronised to NTP server" が含まれているかチェック
    if output_str.contains("synchronised to NTP server") {
        Ok(())
    } else {
        Err("NTP is not synchronised".to_string())
    }
} */