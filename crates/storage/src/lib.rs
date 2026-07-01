/// Local SQLite storage for stock data cache and user preferences.

use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::Connection;
use stock_vision_data_model::*;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        let storage = Self { conn: Mutex::new(conn) };
        storage.initialize_tables()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn: Mutex::new(conn) };
        storage.initialize_tables()?;
        Ok(storage)
    }

    fn initialize_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS daily_bars (
                code TEXT NOT NULL,
                date TEXT NOT NULL,
                open REAL,
                high REAL,
                low REAL,
                close REAL,
                volume REAL,
                amount REAL,
                PRIMARY KEY (code, date)
            );

            CREATE TABLE IF NOT EXISTS watchlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                stocks TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS watch_stocks (
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                exchange TEXT NOT NULL,
                added_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (code, exchange)
            );

            CREATE TABLE IF NOT EXISTS cached_reports (
                code TEXT NOT NULL,
                report_date TEXT NOT NULL,
                data TEXT NOT NULL,
                PRIMARY KEY (code, report_date)
            );

            CREATE TABLE IF NOT EXISTS intraday_bars (
                code TEXT NOT NULL,
                datetime TEXT NOT NULL,
                open REAL,
                high REAL,
                low REAL,
                close REAL,
                volume REAL,
                amount REAL,
                PRIMARY KEY (code, datetime)
            );

            CREATE TABLE IF NOT EXISTS browse_history (
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                exchange TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 1,
                last_access TEXT NOT NULL,
                PRIMARY KEY (code, exchange)
            );

            CREATE INDEX IF NOT EXISTS idx_daily_bars_code_date 
            ON daily_bars(code, date);
            ",
        )?;
        Ok(())
    }

    // ── Daily bars cache ──

    pub fn save_daily_bars(&self, bars: &[DailyBar]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO daily_bars (code, date, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for bar in bars {
            stmt.execute(rusqlite::params![
                bar.code, bar.date.format("%Y-%m-%d").to_string(),
                bar.open, bar.high, bar.low, bar.close, bar.volume, bar.amount,
            ])?;
        }
        Ok(())
    }

    pub fn get_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<DailyBar>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT code, date, open, high, low, close, volume, amount
             FROM daily_bars WHERE code = ?1 ORDER BY date DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, limit as i64], |row| {
            Ok(DailyBar {
                code: row.get(0)?,
                date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                open: row.get(2)?, high: row.get(3)?, low: row.get(4)?,
                close: row.get(5)?, volume: row.get(6)?, amount: row.get(7)?,
                change_pct: None,
            })
        })?;
        let mut bars: Vec<DailyBar> = rows.filter_map(|r| r.ok()).collect();
        bars.reverse();
        Ok(bars)
    }

    // ── Intraday bars cache ──

    pub fn save_intraday_bars(&self, bars: &[IntradayBar]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO intraday_bars (code, datetime, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for bar in bars {
            stmt.execute(rusqlite::params![
                bar.code, bar.datetime,
                bar.open, bar.high, bar.low, bar.close, bar.volume, bar.amount,
            ])?;
        }
        Ok(())
    }

    pub fn get_intraday_bars(&self, code: &str, limit: usize) -> Result<Vec<IntradayBar>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT code, datetime, open, high, low, close, volume, amount
             FROM intraday_bars WHERE code = ?1 ORDER BY datetime DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, limit as i64], |row| {
            Ok(IntradayBar {
                code: row.get(0)?,
                datetime: row.get(1)?,
                open: row.get(2)?, high: row.get(3)?, low: row.get(4)?,
                close: row.get(5)?, volume: row.get(6)?, amount: row.get(7)?,
            })
        })?;
        let mut bars: Vec<IntradayBar> = rows.filter_map(|r| r.ok()).collect();
        bars.reverse();
        Ok(bars)
    }


    // ── Financial reports cache ──

    pub fn save_financial_reports(&self, reports: &[FinancialReport]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO cached_reports (code, report_date, data) VALUES (?1, ?2, ?3)",
        )?;
        for report in reports {
            let data = serde_json::to_string(report)?;
            stmt.execute(rusqlite::params![
                report.code,
                report.report_date.format("%Y-%m-%d").to_string(),
                data,
            ])?;
        }
        Ok(())
    }

    pub fn get_financial_reports(&self, code: &str) -> Result<Vec<FinancialReport>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT data FROM cached_reports WHERE code = ?1 ORDER BY report_date DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![code], |row| {
            Ok(serde_json::from_str::<FinancialReport>(&row.get::<_, String>(0)?).unwrap())
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Browse history ──

    pub fn save_browse_entry(&self, stock: &Stock) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO browse_history (code, name, exchange, count, last_access) VALUES (?1, ?2, ?3, 1, ?4)
             ON CONFLICT(code, exchange) DO UPDATE SET count = count + 1, last_access = ?5, name = ?2",
            rusqlite::params![stock.code, stock.name, format!("{:?}", stock.exchange), now, now],
        )?;
        Ok(())
    }

    pub fn load_browse_history(&self, limit: usize) -> Result<Vec<Stock>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT code, name, exchange FROM browse_history ORDER BY count DESC, last_access DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            let exchange_str: String = row.get(2)?;
            let exchange = match exchange_str.as_str() {
                "SH" => Exchange::SH,
                "SZ" => Exchange::SZ,
                _ => Exchange::SZ,
            };
            Ok(Stock {
                code: row.get(0)?, name: row.get(1)?, exchange,
                sector: None, industry: None, list_date: None,
                total_shares: None, float_shares: None,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Watchlists ──

    pub fn save_watchlist(&self, watchlist: &Watchlist) -> Result<()> {
        let stocks_json = serde_json::to_string(&watchlist.stocks)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO watchlists (id, name, stocks) VALUES (?1, ?2, ?3)",
            rusqlite::params![watchlist.id, watchlist.name, stocks_json],
        )?;
        Ok(())
    }

    pub fn get_watchlists(&self) -> Result<Vec<Watchlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, stocks FROM watchlists")?;
        let rows = stmt.query_map([], |row| {
            Ok(Watchlist {
                id: row.get(0)?,
                name: row.get(1)?,
                stocks: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ── Watch stocks (single flat list, used by AppState.watchlist) ──

    pub fn save_watch_stocks(&self, stocks: &[Stock]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM watch_stocks", [])?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO watch_stocks (code, name, exchange) VALUES (?1, ?2, ?3)",
        )?;
        for stock in stocks {
            stmt.execute(rusqlite::params![
                stock.code,
                stock.name,
                format!("{:?}", stock.exchange),
            ])?;
        }
        Ok(())
    }

    pub fn load_watch_stocks(&self) -> Result<Vec<Stock>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT code, name, exchange FROM watch_stocks ORDER BY added_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let exchange_str: String = row.get(2)?;
            let exchange = match exchange_str.as_str() {
                "SH" => Exchange::SH,
                "SZ" => Exchange::SZ,
                _ => Exchange::SZ,
            };
            Ok(Stock {
                code: row.get(0)?,
                name: row.get(1)?,
                exchange,
                sector: None, industry: None, list_date: None,
                total_shares: None, float_shares: None,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

unsafe impl Sync for Storage {}
