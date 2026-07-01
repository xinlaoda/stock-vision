/// Indicator service — manages which indicators are active and computes their values.

use stock_vision_data_model::DailyBar;
use stock_vision_indicator_core::*;

/// Types of indicators available for display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicatorType {
    /// Overlays on the K-line chart
    BollingerBands,
    /// Sub-panel indicators (rendered below main chart)
    KDJ,
    RSI,
    MACD,
}

impl IndicatorType {
    pub fn label(&self) -> &str {
        match self {
            IndicatorType::BollingerBands => "BOLL(20,2)",
            IndicatorType::KDJ => "KDJ(9,3,3)",
            IndicatorType::RSI => "RSI(14)",
            IndicatorType::MACD => "MACD(12,26,9)",
        }
    }

    /// Whether this indicator is rendered as an overlay on the main K-line chart
    pub fn is_chart_overlay(&self) -> bool {
        matches!(self, IndicatorType::BollingerBands)
    }

    /// Whether this indicator needs its own sub-panel
    pub fn is_sub_panel(&self) -> bool {
        !self.is_chart_overlay()
    }
}

/// Computed indicator data ready for rendering
#[derive(Debug, Clone)]
pub struct ComputedIndicator {
    pub indicator_type: IndicatorType,
    /// Main line values
    pub line1: Vec<Option<f64>>,
    pub line1_label: String,
    pub line1_color: (f32, f32, f32),
    /// Second line (optional, e.g. lower BOLL, D in KDJ)
    pub line2: Vec<Option<f64>>,
    pub line2_label: String,
    pub line2_color: (f32, f32, f32),
    /// Third line (optional, e.g. J in KDJ, signal in MACD)
    pub line3: Vec<Option<f64>>,
    pub line3_label: String,
    pub line3_color: (f32, f32, f32),
    /// Histogram values (optional, for MACD histogram)
    pub histogram: Vec<Option<f64>>,
}

/// Compute indicator data for a set of bars
pub fn compute_indicator(indicator_type: IndicatorType, bars: &[DailyBar], params: &crate::state::IndicatorParams) -> Option<ComputedIndicator> {
    if bars.is_empty() {
        return None;
    }

    match indicator_type {
        IndicatorType::BollingerBands => {
            let bb = BollingerBands { period: params.boll_period, std_dev: params.boll_std };
            let upper = bb.calculate(bars);
            // Compute middle (SMA) and lower band
            let sma = SMA { period: params.boll_period };
            let middle = sma.calculate(bars);
            // Lower = middle - 2*std (since upper = middle + 2*std, we can derive)
            let lower: Vec<Option<f64>> = upper.values.iter().zip(middle.values.iter()).map(|(u, m)| {
                if u.value.is_nan() || m.value.is_nan() { None }
                else { Some(m.value - (u.value - m.value)) }
            }).collect();
            let mid_v: Vec<Option<f64>> = middle.values.iter().map(|v| if v.value.is_nan() { None } else { Some(v.value) }).collect();
            let up_v: Vec<Option<f64>> = upper.values.iter().map(|v| if v.value.is_nan() { None } else { Some(v.value) }).collect();

            Some(ComputedIndicator {
                indicator_type,
                line1: up_v,
                line1_label: "BOLL上轨".to_string(),
                line1_color: (0.9, 0.6, 0.2),
                line2: mid_v,
                line2_label: "BOLL中轨".to_string(),
                line2_color: (1.0, 0.85, 0.2),
                line3: lower,
                line3_label: "BOLL下轨".to_string(),
                line3_color: (0.9, 0.6, 0.2),
                histogram: vec![],
            })
        }
        IndicatorType::KDJ => {
            let kdj = KDJ { n: params.kdj_n, m1: params.kdj_m1, m2: params.kdj_m2 };
            let points = kdj.calculate_full(bars);
            let k: Vec<Option<f64>> = points.iter().map(|p| if p.k.is_nan() { None } else { Some(p.k) }).collect();
            let d: Vec<Option<f64>> = points.iter().map(|p| if p.d.is_nan() { None } else { Some(p.d) }).collect();
            let j: Vec<Option<f64>> = points.iter().map(|p| if p.j.is_nan() { None } else { Some(p.j) }).collect();

            Some(ComputedIndicator {
                indicator_type,
                line1: k,
                line1_label: "K".to_string(),
                line1_color: (1.0, 0.8, 0.0),  // Yellow
                line2: d,
                line2_label: "D".to_string(),
                line2_color: (0.3, 0.7, 1.0),  // Blue
                line3: j,
                line3_label: "J".to_string(),
                line3_color: (1.0, 0.4, 0.7),  // Pink
                histogram: vec![],
            })
        }
        IndicatorType::RSI => {
            let rsi = RSI { period: params.rsi_period };
            let result = rsi.calculate(bars);
            let vals: Vec<Option<f64>> = result.values.iter().map(|v| if v.value.is_nan() { None } else { Some(v.value) }).collect();

            Some(ComputedIndicator {
                indicator_type,
                line1: vals,
                line1_label: "RSI".to_string(),
                line1_color: (0.6, 0.3, 1.0),  // Purple
                line2: vec![],
                line2_label: String::new(),
                line2_color: (0.0, 0.0, 0.0),
                line3: vec![],
                line3_label: String::new(),
                line3_color: (0.0, 0.0, 0.0),
                histogram: vec![],
            })
        }
        IndicatorType::MACD => {
            let macd = stock_vision_indicator_core::MACD { fast: params.macd_fast, slow: params.macd_slow, signal: params.macd_signal };
            let result = macd.calculate(bars);
            let vals: Vec<Option<f64>> = result.values.iter().map(|v| if v.value.is_nan() { None } else { Some(v.value) }).collect();

            // Recompute DIF and DEA for display
            let fast_ema = EMA { period: params.macd_fast }.calculate(bars);
            let slow_ema = EMA { period: params.macd_slow }.calculate(bars);
            let dif: Vec<Option<f64>> = fast_ema.values.iter().zip(slow_ema.values.iter()).map(|(f, s)| {
                if f.value.is_nan() || s.value.is_nan() { None } else { Some(f.value - s.value) }
            }).collect();

            let signal_k = 2.0 / (params.macd_signal as f64 + 1.0);
            let mut dea: Vec<Option<f64>> = Vec::new();
            let mut prev_dea = 0.0;
            for (i, d) in dif.iter().enumerate() {
                match d {
                    Some(v) => {
                        if i == 0 { prev_dea = *v; }
                        else { prev_dea = v * signal_k + prev_dea * (1.0 - signal_k); }
                        dea.push(Some(prev_dea));
                    }
                    None => dea.push(None),
                }
            }

            Some(ComputedIndicator {
                indicator_type,
                line1: dif,
                line1_label: "DIF".to_string(),
                line1_color: (1.0, 1.0, 1.0),    // White
                line2: dea,
                line2_label: "DEA".to_string(),
                line2_color: (1.0, 0.8, 0.0),    // Yellow
                line3: vals.clone(),
                line3_label: "MACD".to_string(),
                line3_color: (0.3, 0.7, 1.0),    // Blue
                histogram: vals,
            })
        }
    }
}
