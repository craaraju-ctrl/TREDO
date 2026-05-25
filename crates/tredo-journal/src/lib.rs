use rusqlite::{params, Connection, Result as SqlResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// A trade record — opened when a position is entered, closed when exited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub symbol: String,
    pub side: String,          // "BUY" or "SELL"
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub pnl: Option<f64>,       // Realized P&L when closed
    pub pnl_pct: Option<f64>,   // Percentage P&L
    pub conviction_at_entry: f64,
    pub entry_reasoning: String,
    pub exit_reasoning: Option<String>,
    pub market_regime: String,  // "trending_bullish", "trending_bearish", "ranging", "volatile"
    pub strategies_used: String, // Comma-separated strategy IDs
    pub open_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub is_open: bool,
}

/// A decision record — every analysis cycle creates one, even if no trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub overall_conviction: f64,
    pub overall_direction: String,
    pub market_regime: String,
    pub action_taken: String,   // "BUY", "SELL", "HOLD", "SKIP"
    pub reason: String,
    pub bullish_signals: u32,
    pub bearish_signals: u32,
    pub neutral_signals: u32,
}

/// Aggregated performance stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
}

/// The trade journal — thread-safe SQLite-backed store
pub struct TradeJournal {
    conn: Mutex<Connection>,
}

impl TradeJournal {
    /// Open or create the journal database at the given path
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let journal = Self {
            conn: Mutex::new(conn),
        };
        journal.initialize_tables()?;
        println!("[TradeJournal] Opened database at {}", db_path);
        Ok(journal)
    }

    /// Create tables if they don't exist
    fn initialize_tables(&self) -> SqlResult<()> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL,
                quantity REAL NOT NULL,
                pnl REAL,
                pnl_pct REAL,
                conviction_at_entry REAL NOT NULL,
                entry_reasoning TEXT NOT NULL DEFAULT '',
                exit_reasoning TEXT,
                market_regime TEXT NOT NULL DEFAULT '',
                strategies_used TEXT NOT NULL DEFAULT '',
                open_time TEXT NOT NULL,
                close_time TEXT,
                is_open INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                overall_conviction REAL NOT NULL,
                overall_direction TEXT NOT NULL,
                market_regime TEXT NOT NULL,
                action_taken TEXT NOT NULL,
                reason TEXT NOT NULL,
                bullish_signals INTEGER NOT NULL DEFAULT 0,
                bearish_signals INTEGER NOT NULL DEFAULT 0,
                neutral_signals INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
            CREATE INDEX IF NOT EXISTS idx_trades_open ON trades(is_open);
            CREATE INDEX IF NOT EXISTS idx_decisions_symbol ON decisions(symbol);
            CREATE INDEX IF NOT EXISTS idx_decisions_timestamp ON decisions(timestamp);
            ",
        )?;
        Ok(())
    }

    /// Record a new trade opening
    pub fn open_trade(&self, trade: &TradeRecord) -> SqlResult<()> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        conn.execute(
            "INSERT INTO trades (id, symbol, side, entry_price, quantity, conviction_at_entry, entry_reasoning, market_regime, strategies_used, open_time, is_open)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1)",
            params![
                trade.id,
                trade.symbol,
                trade.side,
                trade.entry_price,
                trade.quantity,
                trade.conviction_at_entry,
                trade.entry_reasoning,
                trade.market_regime,
                trade.strategies_used,
                trade.open_time.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Close an existing trade with exit data
    pub fn close_trade(&self, trade_id: &str, exit_price: f64, exit_reasoning: &str) -> SqlResult<Option<TradeRecord>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");

        // Get the open trade
        let trade = conn.query_row(
            "SELECT id, symbol, side, entry_price, quantity, conviction_at_entry, entry_reasoning, market_regime, strategies_used, open_time, is_open
             FROM trades WHERE id = ?1 AND is_open = 1",
            params![trade_id],
            |row| {
                Ok(TradeRecord {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    side: row.get(2)?,
                    entry_price: row.get(3)?,
                    exit_price: None,
                    quantity: row.get(4)?,
                    pnl: None,
                    pnl_pct: None,
                    conviction_at_entry: row.get(5)?,
                    entry_reasoning: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    exit_reasoning: None,
                    market_regime: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    strategies_used: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    open_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    close_time: None,
                    is_open: row.get::<_, i32>(10)? != 0,
                })
            },
        );

        match trade {
            Ok(mut trade) => {
                let close_time = Utc::now();
                let pnl = if trade.side == "BUY" {
                    (exit_price - trade.entry_price) * trade.quantity
                } else {
                    (trade.entry_price - exit_price) * trade.quantity
                };
                let pnl_pct = if trade.entry_price > 0.0 {
                    (pnl / (trade.entry_price * trade.quantity)) * 100.0
                } else {
                    0.0
                };

                conn.execute(
                    "UPDATE trades SET exit_price = ?1, pnl = ?2, pnl_pct = ?3, exit_reasoning = ?4, close_time = ?5, is_open = 0
                     WHERE id = ?6",
                    params![exit_price, pnl, pnl_pct, exit_reasoning, close_time.to_rfc3339(), trade_id],
                )?;

                trade.exit_price = Some(exit_price);
                trade.pnl = Some(pnl);
                trade.pnl_pct = Some(pnl_pct);
                trade.exit_reasoning = Some(exit_reasoning.to_string());
                trade.close_time = Some(close_time);
                trade.is_open = false;

                Ok(Some(trade))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Record a decision (every analysis cycle)
    pub fn record_decision(&self, decision: &DecisionRecord) -> SqlResult<()> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        conn.execute(
            "INSERT INTO decisions (id, symbol, timestamp, overall_conviction, overall_direction, market_regime, action_taken, reason, bullish_signals, bearish_signals, neutral_signals)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                decision.id,
                decision.symbol,
                decision.timestamp.to_rfc3339(),
                decision.overall_conviction,
                decision.overall_direction,
                decision.market_regime,
                decision.action_taken,
                decision.reason,
                decision.bullish_signals,
                decision.bearish_signals,
                decision.neutral_signals,
            ],
        )?;
        Ok(())
    }

    /// Get all open trades
    pub fn get_open_trades(&self) -> SqlResult<Vec<TradeRecord>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, symbol, side, entry_price, exit_price, quantity, pnl, pnl_pct, conviction_at_entry, entry_reasoning, exit_reasoning, market_regime, strategies_used, open_time, close_time, is_open
             FROM trades WHERE is_open = 1 ORDER BY open_time DESC",
        )?;

        let trades = stmt.query_map([], Self::map_trade_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(trades)
    }

    /// Get trade history with pagination
    pub fn get_trade_history(&self, limit: u64, offset: u64) -> SqlResult<Vec<TradeRecord>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, symbol, side, entry_price, exit_price, quantity, pnl, pnl_pct, conviction_at_entry, entry_reasoning, exit_reasoning, market_regime, strategies_used, open_time, close_time, is_open
             FROM trades ORDER BY open_time DESC LIMIT ?1 OFFSET ?2",
        )?;

        let trades = stmt.query_map(params![limit, offset], Self::map_trade_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(trades)
    }

    /// Get recent decisions
    pub fn get_recent_decisions(&self, limit: u64) -> SqlResult<Vec<DecisionRecord>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, symbol, timestamp, overall_conviction, overall_direction, market_regime, action_taken, reason, bullish_signals, bearish_signals, neutral_signals
             FROM decisions ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let decisions = stmt.query_map(params![limit], |row| {
            Ok(DecisionRecord {
                id: row.get(0)?,
                symbol: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                overall_conviction: row.get(3)?,
                overall_direction: row.get(4)?,
                market_regime: row.get(5)?,
                action_taken: row.get(6)?,
                reason: row.get(7)?,
                bullish_signals: row.get(8)?,
                bearish_signals: row.get(9)?,
                neutral_signals: row.get(10)?,
            })
        })?;

        let results = decisions.filter_map(|r| r.ok()).collect();
        Ok(results)
    }

    /// Compute aggregate performance statistics
    pub fn get_performance_stats(&self) -> SqlResult<PerformanceStats> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");

        let (total_trades, winning_trades, losing_trades): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END)
             FROM trades WHERE is_open = 0 AND pnl IS NOT NULL",
            [],
            |row| Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0), row.get::<_, Option<i64>>(2)?.unwrap_or(0))),
        )?;

        let (total_pnl, avg_win, avg_loss): (f64, f64, f64) = conn.query_row(
            "SELECT COALESCE(SUM(pnl), 0), AVG(CASE WHEN pnl > 0 THEN pnl END), AVG(CASE WHEN pnl < 0 THEN pnl END)
             FROM trades WHERE is_open = 0 AND pnl IS NOT NULL",
            [],
            |row| Ok((
                row.get(0)?,
                row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(2)?.unwrap_or(0.0)
            )),
        )?;

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let profit_factor = if avg_loss.abs() > 0.0 {
            (winning_trades as f64 * avg_win.abs()) / (losing_trades as f64 * avg_loss.abs() + 1.0)
        } else if winning_trades > 0 {
            999.0 // Infinite profit factor
        } else {
            0.0
        };

        // Max drawdown: worst peak-to-trough of cumulative P&L
        let max_drawdown = Self::calculate_max_drawdown(&conn)?;

        // Sharpe ratio: avg return / std dev of returns (simplified)
        let sharpe_ratio = Self::calculate_sharpe_ratio(&conn)?;

        Ok(PerformanceStats {
            total_trades: total_trades as u64,
            winning_trades: winning_trades as u64,
            losing_trades: losing_trades as u64,
            win_rate,
            total_pnl,
            avg_win,
            avg_loss,
            profit_factor,
            max_drawdown,
            sharpe_ratio,
        })
    }

    /// Get win rate broken down by strategy
    pub fn get_strategy_win_rates(&self) -> SqlResult<Vec<(String, f64, i64)>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT strategies_used, CAST(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END) AS REAL) * 100.0 / COUNT(*) as win_rate, COUNT(*) as total
             FROM trades WHERE is_open = 0 AND pnl IS NOT NULL AND strategies_used != ''
             GROUP BY strategies_used ORDER BY win_rate DESC",
        )?;

        let results = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?))
        })?;

        Ok(results.filter_map(|r| r.ok()).collect())
    }

    /// Get win rate breakdown by market regime
    pub fn get_regime_win_rates(&self) -> SqlResult<Vec<(String, f64, i64)>> {
        let conn = self.conn.lock().expect("Journal Mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT market_regime, CAST(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END) AS REAL) * 100.0 / COUNT(*) as win_rate, COUNT(*) as total
             FROM trades WHERE is_open = 0 AND pnl IS NOT NULL AND market_regime != ''
             GROUP BY market_regime ORDER BY win_rate DESC",
        )?;

        let results = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, i64>(2)?))
        })?;

        Ok(results.filter_map(|r| r.ok()).collect())
    }

    fn map_trade_row(row: &rusqlite::Row) -> rusqlite::Result<TradeRecord> {
        Ok(TradeRecord {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            entry_price: row.get(3)?,
            exit_price: row.get(4)?,
            quantity: row.get(5)?,
            pnl: row.get(6)?,
            pnl_pct: row.get(7)?,
            conviction_at_entry: row.get(8)?,
            entry_reasoning: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
            exit_reasoning: row.get(10)?,
            market_regime: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
            strategies_used: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
            open_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            close_time: row.get::<_, Option<String>>(14)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            is_open: row.get::<_, i32>(15)? != 0,
        })
    }

    fn calculate_max_drawdown(conn: &Connection) -> SqlResult<f64> {
        let mut stmt = conn.prepare(
            "SELECT pnl FROM trades WHERE is_open = 0 AND pnl IS NOT NULL ORDER BY close_time ASC",
        )?;

        let pnls: Vec<f64> = stmt.query_map([], |row| row.get::<_, f64>(0))?
            .filter_map(|r| r.ok())
            .collect();

        if pnls.is_empty() {
            return Ok(0.0);
        }

        let mut peak = 0.0;
        let mut max_dd = 0.0;
        let mut cumulative = 0.0;

        for pnl in &pnls {
            cumulative += pnl;
            if cumulative > peak {
                peak = cumulative;
            }
            let dd = (cumulative - peak) / peak.max(1.0) * 100.0;
            if dd < max_dd {
                max_dd = dd;
            }
        }

        Ok(max_dd.abs())
    }

    fn calculate_sharpe_ratio(conn: &Connection) -> SqlResult<f64> {
        let mut stmt = conn.prepare(
            "SELECT pnl FROM trades WHERE is_open = 0 AND pnl IS NOT NULL ORDER BY close_time ASC",
        )?;

        let pnls: Vec<f64> = stmt.query_map([], |row| row.get::<_, f64>(0))?
            .filter_map(|r| r.ok())
            .collect();

        if pnls.len() < 2 {
            return Ok(0.0);
        }

        let mean = pnls.iter().sum::<f64>() / pnls.len() as f64;
        let variance = pnls.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (pnls.len() - 1) as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return Ok(0.0);
        }

        // Annualized Sharpe ratio from trade returns (simplified)
        Ok(mean / std_dev * (365.0_f64).sqrt())
    }
}
