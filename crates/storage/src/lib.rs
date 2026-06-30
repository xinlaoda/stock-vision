/// Local SQLite storage for stock data cache and user preferences.

use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::Connection;
use stock_vision_data_model::*;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.initialize_tables()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.initialize_tables()?;
        Ok(storage)
    }

    fn initialize_tables(&self) -> Result<()> {
        self.conn.execute_batch(
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

            CREATE TABLE IF NOT EXISTS cached_reports (
                code TEXT NOT NULL,
                report_date TEXT NOT NULL,
                data TEXT NOT NULL,
                PRIMARY KEY (code, report_date)
            );

            CREATE INDEX IF NOT EXISTS idx_daily_bars_code_date 
            ON daily_bars(code, date);
            ",
        )?;
        Ok(())
    }

    pub fn save_daily_bars(&self, bars: &[DailyBar]) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO daily_bars (code, date, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for bar in bars {
            stmt.execute(rusqlite::params![
                bar.code,
                bar.date.format("%Y-%m-%d").to_string(),
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume,
                bar.amount,
            ])?;
        }
        Ok(())
    }

    pub fn get_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<DailyBar>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT code, date, open, high, low, close, volume, amount
             FROM daily_bars
             WHERE code = ?1
             ORDER BY date DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, limit as i64], |row| {
            Ok(DailyBar {
                code: row.get(0)?,
                date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
                amount: row.get(7)?,
                change_pct: None,
            })
        })?;
        let mut bars: Vec<DailyBar> = rows.filter_map(|r| r.ok()).collect();
        bars.reverse();
        Ok(bars)
    }

    pub fn save_watchlist(&self, watchlist: &Watchlist) -> Result<()> {
        let stocks_json = serde_json::to_string(&watchlist.stocks)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO watchlists (id, name, stocks) VALUES (?1, ?2, ?3)",
            rusqlite::params![watchlist.id, watchlist.name, stocks_json],
        )?;
        Ok(())
    }

    pub fn get_watchlists(&self) -> Result<Vec<Watchlist>> {
        let mut stmt = self.conn.prepare("SELECT id, name, stocks FROM watchlists")?;
        let rows = stmt.query_map([], |row| {
            Ok(Watchlist {
                id: row.get(0)?,
                name: row.get(1)?,
                stocks: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
