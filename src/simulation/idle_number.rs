//! Big-number formatting, ported from the original's `IdleNumber` concept
//! (GDD §10). Values are plain `f64` — ample range for v1 — formatted with
//! idle-genre suffixes and a scientific fallback beyond the suffix table.

const SUFFIXES: [&str; 11] = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No"];

/// Formats a gold-style quantity: integers below 1000, two significant
/// decimals with a suffix up to `No` (1e30), scientific notation beyond.
pub fn format_amount(value: f64) -> String {
    if !value.is_finite() {
        return "∞".to_owned();
    }
    if value < 0.0 {
        return format!("-{}", format_amount(-value));
    }
    if value < 1000.0 {
        return format!("{}", value.floor() as i64);
    }

    let tier = (value.log10() / 3.0).floor() as usize;
    if tier < SUFFIXES.len() {
        let scaled = value / 1000f64.powi(tier as i32);
        format!("{:.2}{}", scaled, SUFFIXES[tier])
    } else {
        format!("{:.2e}", value)
    }
}

/// Formats a per-second rate with one decimal below 1000, suffixed above.
pub fn format_rate(value: f64) -> String {
    if value < 1000.0 {
        format!("{:.1}", value)
    } else {
        format_amount(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_across_tiers() {
        assert_eq!(format_amount(0.0), "0");
        assert_eq!(format_amount(999.0), "999");
        assert_eq!(format_amount(1_500.0), "1.50K");
        assert_eq!(format_amount(2_340_000.0), "2.34M");
        assert_eq!(format_amount(1e9), "1.00B");
        assert_eq!(format_amount(1e12), "1.00T");
        assert_eq!(format_amount(1e30), "1.00No");
    }

    #[test]
    fn falls_back_to_scientific() {
        assert_eq!(format_amount(1e36), "1.00e36");
    }

    #[test]
    fn rates_keep_one_decimal() {
        assert_eq!(format_rate(2.5), "2.5");
        assert_eq!(format_rate(12_500.0), "12.50K");
    }

    #[test]
    fn handles_extreme_scales_without_panicking() {
        // Well past any suffix — plain scientific, no overflow or panic.
        assert_eq!(format_amount(1e100), "1.00e100");
        assert_eq!(format_amount(1e300), "1.00e300");
        // f64::MAX (~1.8e308) is still finite and formats.
        assert!(format_amount(f64::MAX).contains("e308"));
        // Overflow to infinity degrades gracefully rather than printing junk.
        assert_eq!(format_amount(f64::INFINITY), "∞");
        assert_eq!(format_amount(f64::NAN), "∞");
        // Deep multi-tier prestige thresholds (base 1e6 * 8^n) stay in range
        // and finite for any count a player could ever reach.
        let deep = 1e6 * 8f64.powi(100); // ~2e96
        assert!(deep.is_finite());
        assert!(format_amount(deep).contains('e'));
    }
}
