use std::env;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::BotResult;
use crate::types::StrategyConfig;

// ============================================================
// CONFIGURATION STRUCTS
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub jwt_secret: String, // Will be overridden by env
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_hours: u64,
}

fn default_jwt_expiry() -> u64 {
    24
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
    #[serde(default = "default_backup_interval")]
    pub backup_interval_hours: u64,
}

fn default_db_path() -> String {
    "data/trading.db".to_string()
}

fn default_backup_interval() -> u64 {
    6
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    #[serde(default = "default_exchange_url")]
    pub base_url: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub api_key: String, // Will be overridden by env
    #[serde(default)]
    pub secret_key: String, // Will be overridden by env
}

fn default_exchange_url() -> String {
    "https://indodax.com".to_string()
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub fcm_server_key: String, // Will be overridden by env
    #[serde(default = "default_notif_enabled")]
    pub enabled: bool,
}

fn default_notif_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_file")]
    pub file_path: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_log_file() -> String {
    "logs/bot.log".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub exchange: ExchangeConfig,
    pub strategy: StrategyConfig,
    pub notification: NotificationConfig,
    pub logging: LoggingConfig,
}

// ============================================================
// IMPLEMENTATION: LOAD & VALIDATE
// ============================================================

impl Config {
    /// Load config dari `config.yaml` dan override dari environment variables.
    pub fn load() -> BotResult<Self> {
        // 1. Load .env file (jika ada)
        let _ = dotenvy::dotenv();

        // 2. Baca file config.yaml
        let yaml_content = fs::read_to_string("config.yaml").map_err(|e| {
            crate::error::BotError::Config(format!("Failed to read config.yaml: {}", e))
        })?;

        let mut config: Config = serde_yaml::from_str(&yaml_content).map_err(|e| {
            crate::error::BotError::Config(format!("Failed to parse config.yaml: {}", e))
        })?;

        // 3. Override dari environment variables
        if let Ok(val) = env::var("JWT_SECRET") {
            config.server.jwt_secret = val;
        }
        if let Ok(val) = env::var("INDODAX_API_KEY") {
            config.exchange.api_key = val;
        }
        if let Ok(val) = env::var("INDODAX_SECRET_KEY") {
            config.exchange.secret_key = val;
        }
        if let Ok(val) = env::var("FCM_SERVER_KEY") {
            config.notification.fcm_server_key = val;
        }
        if let Ok(val) = env::var("DATABASE_PATH") {
            config.database.path = val;
        }
        if let Ok(val) = env::var("LOG_LEVEL") {
            config.logging.level = val;
        }

        // 4. Validasi konfigurasi
        config.validate()?;

        Ok(config)
    }

    /// Validasi semua aturan bisnis yang tertera di SRS.
    pub fn validate(&self) -> BotResult<()> {
        use crate::error::BotError::InvalidParameter;

        // --- Server ---
        if self.server.jwt_secret.is_empty() {
            return Err(InvalidParameter {
                field: "server.jwt_secret".to_string(),
                value: "empty".to_string(),
            });
        }
        if self.server.port == 0 {
            return Err(InvalidParameter {
                field: "server.port".to_string(),
                value: self.server.port.to_string(),
            });
        }

        // --- Exchange ---
        if self.exchange.api_key.is_empty() {
            // Warning saja, karena mungkin untuk paper trading
            tracing::warn!("INDODAX_API_KEY is empty. Exchange trading will fail.");
        }
        if self.exchange.secret_key.is_empty() {
            tracing::warn!("INDODAX_SECRET_KEY is empty. Exchange trading will fail.");
        }

        // --- Strategy -> Entry ---
        let entry = &self.strategy.entry;
        if !(3..=8).contains(&entry.min_score) {
            return Err(InvalidParameter {
                field: "strategy.entry.min_score".to_string(),
                value: entry.min_score.to_string(),
            });
        }
        let ema = &entry.ema_periods;
        if ema.fast == 0 || ema.medium == 0 || ema.slow == 0 {
            return Err(InvalidParameter {
                field: "strategy.entry.ema_periods".to_string(),
                value: format!(
                    "fast={}, medium={}, slow={}",
                    ema.fast, ema.medium, ema.slow
                ),
            });
        }
        if !(ema.fast < ema.medium && ema.medium < ema.slow) {
            return Err(InvalidParameter {
                field: "strategy.entry.ema_periods".to_string(),
                value: format!(
                    "fast={}, medium={}, slow={} (must be fast < medium < slow)",
                    ema.fast, ema.medium, ema.slow
                ),
            });
        }
        let rsi = &entry.rsi;
        if !(5..=30).contains(&rsi.period) {
            return Err(InvalidParameter {
                field: "strategy.entry.rsi.period".to_string(),
                value: rsi.period.to_string(),
            });
        }
        if !(20.0..40.0).contains(&rsi.oversold) {
            return Err(InvalidParameter {
                field: "strategy.entry.rsi.oversold".to_string(),
                value: rsi.oversold.to_string(),
            });
        }
        if !(60.0..80.0).contains(&rsi.overbought) {
            return Err(InvalidParameter {
                field: "strategy.entry.rsi.overbought".to_string(),
                value: rsi.overbought.to_string(),
            });
        }
        if rsi.oversold >= rsi.overbought {
            return Err(InvalidParameter {
                field: "strategy.entry.rsi".to_string(),
                value: format!("oversold={} >= overbought={}", rsi.oversold, rsi.overbought),
            });
        }
        if !(1.0..=5.0).contains(&entry.volume_threshold) {
            return Err(InvalidParameter {
                field: "strategy.entry.volume_threshold".to_string(),
                value: entry.volume_threshold.to_string(),
            });
        }

        // --- Strategy -> Exit ---
        let exit = &self.strategy.exit;
        for (i, &level) in exit.take_profit_levels.iter().enumerate() {
            if level <= 0.0 {
                return Err(InvalidParameter {
                    field: format!("strategy.exit.take_profit_levels[{}]", i),
                    value: level.to_string(),
                });
            }
        }
        if exit.take_profit_levels[0] >= exit.take_profit_levels[1]
            || exit.take_profit_levels[1] >= exit.take_profit_levels[2]
        {
            return Err(InvalidParameter {
                field: "strategy.exit.take_profit_levels".to_string(),
                value: format!("{:?} (must be ascending)", exit.take_profit_levels),
            });
        }
        let sum_close: f64 = exit.close_percentages.iter().sum();
        if (sum_close - 100.0).abs() > f64::EPSILON {
            return Err(InvalidParameter {
                field: "strategy.exit.close_percentages".to_string(),
                value: format!("sum = {} (must be exactly 100)", sum_close),
            });
        }
        for (i, &pct) in exit.close_percentages.iter().enumerate() {
            if pct <= 0.0 {
                return Err(InvalidParameter {
                    field: format!("strategy.exit.close_percentages[{}]", i),
                    value: pct.to_string(),
                });
            }
        }
        if !(0.2..=2.0).contains(&exit.trailing_stop_activation) {
            return Err(InvalidParameter {
                field: "strategy.exit.trailing_stop_activation".to_string(),
                value: exit.trailing_stop_activation.to_string(),
            });
        }
        if !(0.1..=1.0).contains(&exit.trailing_stop_distance) {
            return Err(InvalidParameter {
                field: "strategy.exit.trailing_stop_distance".to_string(),
                value: exit.trailing_stop_distance.to_string(),
            });
        }
        if !(1..=24).contains(&exit.max_hold_hours) {
            return Err(InvalidParameter {
                field: "strategy.exit.max_hold_hours".to_string(),
                value: exit.max_hold_hours.to_string(),
            });
        }

        // --- Strategy -> Risk ---
        let risk = &self.strategy.risk;
        if !(1.0..=15.0).contains(&risk.max_position_pct) {
            return Err(InvalidParameter {
                field: "strategy.risk.max_position_pct".to_string(),
                value: risk.max_position_pct.to_string(),
            });
        }
        if !(0.5..=5.0).contains(&risk.daily_target_pct) {
            return Err(InvalidParameter {
                field: "strategy.risk.daily_target_pct".to_string(),
                value: risk.daily_target_pct.to_string(),
            });
        }
        if !(1.0..=10.0).contains(&risk.daily_loss_limit_pct) {
            return Err(InvalidParameter {
                field: "strategy.risk.daily_loss_limit_pct".to_string(),
                value: risk.daily_loss_limit_pct.to_string(),
            });
        }
        if !(1..=10).contains(&risk.max_consecutive_losses) {
            return Err(InvalidParameter {
                field: "strategy.risk.max_consecutive_losses".to_string(),
                value: risk.max_consecutive_losses.to_string(),
            });
        }
        if !(5..=100).contains(&risk.max_daily_trades) {
            return Err(InvalidParameter {
                field: "strategy.risk.max_daily_trades".to_string(),
                value: risk.max_daily_trades.to_string(),
            });
        }
        if !(0.5..=10.0).contains(&risk.price_spike_threshold_pct) {
            return Err(InvalidParameter {
                field: "strategy.risk.price_spike_threshold_pct".to_string(),
                value: risk.price_spike_threshold_pct.to_string(),
            });
        }
        if risk.min_order_idr <= 0.0 {
            return Err(InvalidParameter {
                field: "strategy.risk.min_order_idr".to_string(),
                value: risk.min_order_idr.to_string(),
            });
        }

        // --- Strategy -> Sessions ---
        let sessions = &self.strategy.sessions;
        Self::validate_session(&sessions.asia)?;
        Self::validate_session(&sessions.low)?;
        Self::validate_session(&sessions.us_eu)?;
        Self::validate_session(&sessions.off)?;

        Ok(())
    }

    fn validate_session(session: &crate::types::SessionConfig) -> BotResult<()> {
        // Cek format "HH:MM"
        let parse_time =
            |s: &str| -> BotResult<()> {
                if s.len() != 5 || &s[2..3] != ":" {
                    return Err(crate::error::BotError::InvalidParameter {
                        field: "session time".to_string(),
                        value: s.to_string(),
                    });
                }
                let hour = s[0..2].parse::<u8>().map_err(|_| {
                    crate::error::BotError::InvalidParameter {
                        field: "session hour".to_string(),
                        value: s.to_string(),
                    }
                })?;
                let min = s[3..5].parse::<u8>().map_err(|_| {
                    crate::error::BotError::InvalidParameter {
                        field: "session minute".to_string(),
                        value: s.to_string(),
                    }
                })?;
                if hour > 23 || min > 59 {
                    return Err(crate::error::BotError::InvalidParameter {
                        field: "session time".to_string(),
                        value: s.to_string(),
                    });
                }
                Ok(())
            };
        parse_time(&session.start)?;
        parse_time(&session.end)?;
        Ok(())
    }
}

// ============================================================
// UNIT TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper: membuat file config.yaml sementara di direktori temp
    fn create_test_config(dir: &std::path::Path, content: &str) {
        let path = dir.join("config.yaml");
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    // Helper: pindah ke direktori temp untuk testing
    fn with_temp_dir<F>(test_fn: F)
    where
        F: FnOnce(&std::path::Path),
    {
        let dir = tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
        test_fn(dir.path());
        env::set_current_dir(original_dir).unwrap();
    }

    const VALID_YAML: &str = r#"
server:
  host: "0.0.0.0"
  port: 3000
  jwt_secret: "change_me"
  jwt_expiry_hours: 24

database:
  path: "data/trading.db"
  backup_interval_hours: 6

exchange:
  base_url: "https://indodax.com"
  timeout_seconds: 30
  api_key: ""
  secret_key: ""

strategy:
  enabled: true
  entry:
    min_score: 5
    ema_periods:
      fast: 9
      medium: 21
      slow: 50
    rsi:
      period: 14
      oversold: 30
      overbought: 70
    volume_threshold: 1.5
  exit:
    take_profit_levels: [1.0, 1.5, 2.0]
    close_percentages: [40.0, 30.0, 30.0]
    trailing_stop_activation: 0.5
    trailing_stop_distance: 0.3
    max_hold_hours: 4
  risk:
    max_position_pct: 10.0
    daily_target_pct: 1.0
    daily_loss_limit_pct: 5.0
    max_consecutive_losses: 3
    max_daily_trades: 20
    price_spike_threshold_pct: 3.0
    min_order_idr: 50000.0
  sessions:
    asia:
      start: "07:00"
      end: "15:00"
      enabled: true
    low:
      start: "15:00"
      end: "19:00"
      enabled: false
    us_eu:
      start: "19:00"
      end: "03:00"
      enabled: true
    off:
      start: "03:00"
      end: "07:00"
      enabled: false

notification:
  fcm_server_key: ""
  enabled: true

logging:
  level: "info"
  format: "json"
  file_path: "logs/bot.log"
"#;

    #[test]
    fn test_load_valid_config() {
        with_temp_dir(|dir| {
            create_test_config(dir, VALID_YAML);

            // Set environment untuk override (kosong, biar default dari yaml)
            env::remove_var("JWT_SECRET");
            env::remove_var("INDODAX_API_KEY");

            let config = Config::load().unwrap();

            assert_eq!(config.server.port, 3000);
            assert_eq!(config.strategy.entry.min_score, 5);
            assert_eq!(config.strategy.risk.max_position_pct, 10.0);
            assert_eq!(config.strategy.sessions.asia.start, "07:00");
            assert_eq!(config.strategy.sessions.asia.enabled, true);
            assert_eq!(config.strategy.sessions.low.enabled, false);
        });
    }

    #[test]
    fn test_env_override() {
        with_temp_dir(|dir| {
            create_test_config(dir, VALID_YAML);

            // Set environment variables
            env::set_var("JWT_SECRET", "override_jwt_secret");
            env::set_var("INDODAX_API_KEY", "override_api_key");
            env::set_var("INDODAX_SECRET_KEY", "override_secret");
            env::set_var("DATABASE_PATH", "override/path.db");
            env::set_var("LOG_LEVEL", "debug");

            let config = Config::load().unwrap();

            assert_eq!(config.server.jwt_secret, "override_jwt_secret");
            assert_eq!(config.exchange.api_key, "override_api_key");
            assert_eq!(config.exchange.secret_key, "override_secret");
            assert_eq!(config.database.path, "override/path.db");
            assert_eq!(config.logging.level, "debug");
        });
    }

    #[test]
    fn test_invalid_config_min_score() {
        let invalid_yaml = VALID_YAML.replace("min_score: 5", "min_score: 9");
        with_temp_dir(|dir| {
            create_test_config(dir, &invalid_yaml);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("min_score"));
        });
    }

    #[test]
    fn test_invalid_config_ema_order() {
        let invalid_yaml = VALID_YAML.replace("fast: 9", "fast: 25");
        with_temp_dir(|dir| {
            create_test_config(dir, &invalid_yaml);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("ema_periods"));
        });
    }

    #[test]
    fn test_invalid_config_close_percentages_sum() {
        let invalid_yaml = VALID_YAML.replace(
            "close_percentages: [40.0, 30.0, 30.0]",
            "close_percentages: [50.0, 30.0, 30.0]",
        );
        with_temp_dir(|dir| {
            create_test_config(dir, &invalid_yaml);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("sum = 110"));
        });
    }

    #[test]
    fn test_invalid_config_rsi_oversold_gt_overbought() {
        let invalid_yaml = VALID_YAML.replace("oversold: 30", "oversold: 80");
        with_temp_dir(|dir| {
            create_test_config(dir, &invalid_yaml);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("oversold="));
        });
    }

    #[test]
    fn test_invalid_session_format() {
        let invalid_yaml = VALID_YAML.replace("\"07:00\"", "\"7:00\"");
        with_temp_dir(|dir| {
            create_test_config(dir, &invalid_yaml);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("session time"));
        });
    }

    #[test]
    fn test_missing_jwt_secret() {
        let yaml_no_jwt = VALID_YAML.replace("jwt_secret: \"change_me\"", "jwt_secret: \"\"");
        with_temp_dir(|dir| {
            create_test_config(dir, &yaml_no_jwt);
            let result = Config::load();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("jwt_secret"));
        });
    }
}
