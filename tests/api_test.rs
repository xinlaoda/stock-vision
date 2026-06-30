use stock_vision_data_source::DataSource;
use stock_vision_data_model::Exchange;

#[tokio::test]
async fn test_tencent_kline() {
    let source = stock_vision_data_source::TencentSource::new();
    let bars = source
        .get_daily_bars(
            "000001",
            Exchange::SZ,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    println!("Got {} daily bars for SZ000001", bars.len());
    assert!(!bars.is_empty(), "Should have at least some bars");
    if let Some(bar) = bars.first() {
        println!(
            "First bar: {} O={} H={} L={} C={} V={}",
            bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume
        );
    }
}

#[tokio::test]
async fn test_search() {
    let source = stock_vision_data_source::EastMoneySource::new();
    let stocks = source.search_stocks("平安").await.unwrap();
    println!("Search '平安' found {} stocks", stocks.len());
    for s in &stocks {
        println!("  {}{} {}", s.exchange.prefix(), s.code, s.name);
    }
    assert!(!stocks.is_empty(), "Should find some stocks");
}
