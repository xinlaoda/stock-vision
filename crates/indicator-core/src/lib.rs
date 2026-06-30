/// Technical indicator calculation core.
/// Provides implementations of common technical analysis indicators.

use stock_vision_data_model::{DailyBar, IndicatorResult, IndicatorValue};

/// Trait for all technical indicators
pub trait Indicator {
    fn name(&self) -> &str;
    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult;
}

// ═══════════════════════════════════
// Simple Moving Average
// ═══════════════════════════════════

pub struct SMA {
    pub period: usize,
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        "SMA"
    }

    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult {
        let mut values = Vec::new();
        for i in 0..bars.len() {
            if i + 1 < self.period {
                values.push(IndicatorValue {
                    date: bars[i].date,
                    value: f64::NAN,
                });
                continue;
            }
            let sum: f64 = bars[i + 1 - self.period..=i]
                .iter()
                .map(|b| b.close)
                .sum();
            values.push(IndicatorValue {
                date: bars[i].date,
                value: sum / self.period as f64,
            });
        }
        IndicatorResult {
            name: format!("SMA({})", self.period),
            values,
        }
    }
}

// ═══════════════════════════════════
// Exponential Moving Average
// ═══════════════════════════════════

pub struct EMA {
    pub period: usize,
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        "EMA"
    }

    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult {
        let k = 2.0 / (self.period as f64 + 1.0);
        let mut values = Vec::new();
        let mut ema = 0.0;
        for (i, bar) in bars.iter().enumerate() {
            if i == 0 {
                ema = bar.close;
            } else {
                ema = bar.close * k + ema * (1.0 - k);
            }
            values.push(IndicatorValue {
                date: bar.date,
                value: ema,
            });
        }
        IndicatorResult {
            name: format!("EMA({})", self.period),
            values,
        }
    }
}

// ═══════════════════════════════════
// MACD
// ═══════════════════════════════════

pub struct MACD {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

impl Indicator for MACD {
    fn name(&self) -> &str {
        "MACD"
    }

    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult {
        let fast_ema = EMA { period: self.fast }.calculate(bars);
        let slow_ema = EMA { period: self.slow }.calculate(bars);
        let mut macd_line = Vec::new();
        for i in 0..bars.len() {
            let diff = fast_ema.values[i].value - slow_ema.values[i].value;
            macd_line.push(IndicatorValue {
                date: bars[i].date,
                value: diff,
            });
        }
        // Signal line (9-period EMA of MACD)
        let signal_k = 2.0 / (self.signal as f64 + 1.0);
        let mut signal_line = Vec::new();
        let mut signal_val = 0.0;
        for (i, mv) in macd_line.iter().enumerate() {
            if i == 0 {
                signal_val = mv.value;
            } else {
                signal_val = mv.value * signal_k + signal_val * (1.0 - signal_k);
            }
            signal_line.push(IndicatorValue {
                date: bars[i].date,
                value: signal_val,
            });
        }
        // MACD histogram
        let mut histogram = Vec::new();
        for i in 0..bars.len() {
            histogram.push(IndicatorValue {
                date: bars[i].date,
                value: macd_line[i].value - signal_line[i].value,
            });
        }
        IndicatorResult {
            name: "MACD".to_string(),
            values: histogram,
        }
    }
}

// ═══════════════════════════════════
// Relative Strength Index
// ═══════════════════════════════════

pub struct RSI {
    pub period: usize,
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "RSI"
    }

    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult {
        let mut values = Vec::new();
        for i in 0..bars.len() {
            if i < self.period {
                values.push(IndicatorValue {
                    date: bars[i].date,
                    value: f64::NAN,
                });
                continue;
            }
            let mut gains = 0.0;
            let mut losses = 0.0;
            for j in i + 1 - self.period..=i {
                if j == 0 {
                    continue;
                }
                let change = bars[j].close - bars[j - 1].close;
                if change >= 0.0 {
                    gains += change;
                } else {
                    losses += change.abs();
                }
            }
            if losses == 0.0 {
                values.push(IndicatorValue {
                    date: bars[i].date,
                    value: 100.0,
                });
            } else {
                let rs = gains / losses;
                values.push(IndicatorValue {
                    date: bars[i].date,
                    value: 100.0 - (100.0 / (1.0 + rs)),
                });
            }
        }
        IndicatorResult {
            name: format!("RSI({})", self.period),
            values,
        }
    }
}

// ═══════════════════════════════════
// Bollinger Bands
// ═══════════════════════════════════

pub struct BollingerBands {
    pub period: usize,
    pub std_dev: f64,
}

impl Indicator for BollingerBands {
    fn name(&self) -> &str {
        "BOLL"
    }

    fn calculate(&self, bars: &[DailyBar]) -> IndicatorResult {
        let sma = SMA { period: self.period }.calculate(bars);
        let mut values = Vec::new();
        for i in 0..bars.len() {
            if i + 1 < self.period {
                values.push(IndicatorValue {
                    date: bars[i].date,
                    value: f64::NAN,
                });
                continue;
            }
            let mean = sma.values[i].value;
            let variance: f64 = bars[i + 1 - self.period..=i]
                .iter()
                .map(|b| (b.close - mean).powi(2))
                .sum::<f64>()
                / self.period as f64;
            let std = variance.sqrt();
            // Upper band
            values.push(IndicatorValue {
                date: bars[i].date,
                value: mean + self.std_dev * std,
            });
        }
        IndicatorResult {
            name: format!("BOLL_UP({},{})", self.period, self.std_dev),
            values,
        }
    }
}
