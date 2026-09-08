use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsage {
    pub five_hour_remaining: u8,
    pub weekly_remaining: u8,
    pub five_hour_reset_at: Option<i64>,
    pub weekly_reset_at: Option<i64>,
    pub updated_at: i64,
    pub source: String,
    pub stale: bool,
}

pub fn remaining_percent(used: f64) -> u8 {
    if !used.is_finite() {
        return 0;
    }
    (100.0 - used).round().clamp(0.0, 100.0) as u8
}

#[allow(dead_code)]
pub fn human_duration(seconds: i64) -> String {
    let total_minutes = seconds.max(0) / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let minutes = total_minutes % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else {
        format!("{hours}h {minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_percentage_is_rounded_and_clamped() {
        assert_eq!(remaining_percent(27.0), 73);
        assert_eq!(remaining_percent(27.6), 72);
        assert_eq!(remaining_percent(-4.0), 100);
        assert_eq!(remaining_percent(120.0), 0);
        assert_eq!(remaining_percent(f64::NAN), 0);
    }

    #[test]
    fn duration_uses_compact_day_hour_minute_format() {
        assert_eq!(human_duration(9_060), "2h 31m");
        assert_eq!(human_duration(309_600), "3d 14h 0m");
        assert_eq!(human_duration(-1), "0h 0m");
    }
}
