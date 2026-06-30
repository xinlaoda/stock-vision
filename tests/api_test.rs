use stock_vision_data_source::DataSource;
use stock_vision_data_model::Exchange;

#[tokio::test]
async fn test_tencent_kline() {
    let source = stock_vision_data_source::TencentSource::new();
    let bars = source
        .get_daily_bars("000001", Exchange::SZ, None, None, None)
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

#[tokio::test]
async fn test_financial_reports() {
    let source = stock_vision_data_source::EastMoneySource::new();

    // Test a non-financial (industrial) stock
    for (code, exchange, name) in [
        ("000001", Exchange::SZ, "平安银行"),
        ("600519", Exchange::SH, "贵州茅台"),
        ("002415", Exchange::SZ, "海康威视"),
    ] {
        let reports = source
            .get_financial_reports(code, exchange, None)
            .await
            .unwrap();
        println!("=== {} ({}) ===", name, code);
        println!("Got {} financial reports", reports.len());
        for r in reports.iter().take(4) {
            println!(
                "  {} | EPS:{} ROE:{}% 营收:{} 净利:{} 毛利率:{}%",
                r.report_date,
                r.eps.map(|v| format!("{:.2}", v)).unwrap_or("-".into()),
                r.roe.map(|v| format!("{:.1}", v)).unwrap_or("-".into()),
                r.revenue.map(|v| format!("{:.0}", v)).unwrap_or("-".into()),
                r.net_profit.map(|v| format!("{:.0}", v)).unwrap_or("-".into()),
                r.gross_margin.map(|v| format!("{:.1}", v)).unwrap_or("-".into()),
            );
        }
        assert!(!reports.is_empty(), "Should have financial reports");
    }
}
