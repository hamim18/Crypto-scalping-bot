use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================
// MARKET DATA TYPES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticker {
    pub timestamp: i64,
    pub last_price: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_btc: f64,
    pub volume_idr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeTrade {
    pub trade_id: String,
    pub timestamp: i64,
    pub price: f64,
    pub amount: f64,
    pub side: TradeSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Timeframe {
    Minutes5,
    Minutes15,
    Hours1,
}

impl Timeframe {
    pub fn duration_seconds(&self) -> i64 {
        match self {
            Timeframe::Minutes5 => 300,
            Timeframe::Minutes15 => 900,
            Timeframe::Hours1 => 3600,
        }
    }
}

// ============================================================
// STRATEGY & SIGNAL TYPES
// ============================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketRegime {
    Trending,
    Ranging,
    Volatile,
}

impl Default for MarketRegime {
    fn default() -> Self {
        MarketRegime::Ranging
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub action: SignalAction,
    pub score: u8,
    pub max_score: u8,
    pub confidence: f64,
    pub regime: MarketRegime,
    pub reasons: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalAction {
    Buy,
    Hold,
}

// ============================================================
// CONFIGURATION SUB-STRUCTS (LENGKAP)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryConfig {
    pub min_score: u8,
    pub ema_periods: EmaPeriods,
    pub rsi: RsiConfig,
    pub volume_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaPeriods {
    pub fast: usize,
    pub medium: usize,
    pub slow: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiConfig {
    pub period: usize,
    pub oversold: f64,
    pub overbought: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitConfig {
    pub take_profit_levels: [f64; 3],
    pub close_percentages: [f64; 3],
    pub trailing_stop_activation: f64,
    pub trailing_stop_distance: f64,
    pub max_hold_hours: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_pct: f64,
    pub daily_target_pct: f64,
    pub daily_loss_limit_pct: f64,
    pub max_consecutive_losses: u8,
    pub max_daily_trades: u16,
    // 👇 Field yang diperlukan oleh config.rs
    pub price_spike_threshold_pct: f64,
    pub min_order_idr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub start: String, // "HH:MM"
    pub end: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGroup {
    pub asia: SessionConfig,
    pub low: SessionConfig,
    pub us_eu: SessionConfig,
    pub off: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub entry: EntryConfig,
    pub exit: ExitConfig,
    pub risk: RiskConfig,
    // 👇 Field sessions yang dibutuhkan
    pub sessions: SessionGroup,
}

// ============================================================
// ORDER & POSITION TYPES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Option<i64>,
    pub exchange_order_id: Option<String>,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub amount_btc: f64,
    pub amount_idr: f64,
    pub status: OrderStatus,
    pub filled_amount: f64,
    pub avg_price: Option<f64>,
    pub fee: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Option<i64>,
    pub buy_order_id: i64,
    pub pair: String,
    pub entry_price: f64,
    pub amount_btc: f64,
    pub total_idr: f64,
    pub stop_loss: f64,
    pub take_profit_1: f64,
    pub take_profit_2: f64,
    pub take_profit_3: f64,
    pub tp1_hit: bool,
    pub tp2_hit: bool,
    pub tp3_hit: bool,
    pub trailing_sl: Option<f64>,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub status: PositionStatus,
    pub strategy_name: Option<String>,
    pub signal_score: Option<u8>,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_reason: Option<String>,
    pub pnl_idr: Option<f64>,
    pub pnl_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionStatus {
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTrade {
    pub id: Option<i64>,
    pub position_id: i64,
    pub buy_order_id: i64,
    pub sell_order_id: i64,
    pub pair: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub amount_btc: f64,
    pub total_buy_idr: f64,
    pub total_sell_idr: f64,
    pub fee_buy: f64,
    pub fee_sell: f64,
    pub total_fee: f64,
    pub pnl_gross_idr: f64,
    pub pnl_net_idr: f64,
    pub pnl_pct: f64,
    pub strategy_name: Option<String>,
    pub signal_score: Option<u8>,
    pub exit_reason: String,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub holding_duration_seconds: i64,
}

// ============================================================
// CAPITAL & PERFORMANCE TYPES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalStatus {
    pub working_capital: f64,
    pub total_equity: f64,
    pub available_idr: f64,
    pub available_btc: f64,
    pub realized_profit: f64,
    pub unrealized_profit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPerformance {
    pub date: String,
    pub working_capital: f64,
    pub ending_capital: f64,
    pub pnl_idr: f64,
    pub pnl_pct: f64,
    pub trades_count: u32,
    pub win_count: u32,
    pub loss_count: u32,
    pub win_rate: f64,
}

// ============================================================
// UNIT TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_serialization() {
        let ticker = Ticker {
            timestamp: 1700000000,
            last_price: 520_000_000.0,
            high_24h: 525_000_000.0,
            low_24h: 518_000_000.0,
            volume_btc: 10.5,
            volume_idr: 5_400_000_000.0,
        };

        let json = serde_json::to_string(&ticker).unwrap();
        let deserialized: Ticker = serde_json::from_str(&json).unwrap();

        assert_eq!(ticker.timestamp, deserialized.timestamp);
        assert_eq!(ticker.last_price, deserialized.last_price);
    }

    #[test]
    fn test_signal_serialization() {
        let signal = Signal {
            action: SignalAction::Buy,
            score: 7,
            max_score: 8,
            confidence: 0.875,
            regime: MarketRegime::Trending,
            reasons: vec![
                "EMA alignment +2".to_string(),
                "Volume strong +2".to_string(),
            ],
            timestamp: 1700000000,
        };

        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&json).unwrap();

        assert_eq!(signal.action, deserialized.action);
        assert_eq!(signal.reasons.len(), deserialized.reasons.len());
    }

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(Timeframe::Minutes5.duration_seconds(), 300);
        assert_eq!(Timeframe::Minutes15.duration_seconds(), 900);
        assert_eq!(Timeframe::Hours1.duration_seconds(), 3600);
    }

    #[test]
    fn test_market_regime_default() {
        let regime = MarketRegime::default();
        assert_eq!(regime, MarketRegime::Ranging);
    }
}
