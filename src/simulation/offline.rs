//! Offline / idle progress (GDD §5.2): the same passive-income rate the live
//! tick uses, applied to wall-clock time elapsed since the last save.
//!
//! Decision (GDD §12 open question 3): offline earnings accrue at the live rate
//! but are **capped** to a generous window (`config.offline_cap_hours`, default
//! 12h). This keeps a weeks-long absence from trivializing the multi-tier
//! prestige climb while still making a daily return feel rewarding. A designer
//! who wants "idle time is real time" can raise the cap arbitrarily in JSON —
//! the clamp lives here, not at the call site.

/// Gold earned while away, crediting at most `cap_seconds` of elapsed time.
/// Negative elapsed time (clock skew, corrupted save timestamps) earns nothing
/// rather than debiting the hoard.
pub fn offline_gold(gold_per_second: f64, seconds_elapsed: f64, cap_seconds: f64) -> f64 {
    let credited = seconds_elapsed.clamp(0.0, cap_seconds.max(0.0));
    gold_per_second * credited
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: f64 = 24.0 * 3600.0;
    const CAP_12H: f64 = 12.0 * 3600.0;

    #[test]
    fn accrues_at_live_rate_within_cap() {
        assert!((offline_gold(2.5, 100.0, CAP_12H) - 250.0).abs() < 1e-9);
    }

    #[test]
    fn negative_elapsed_earns_nothing() {
        assert!((offline_gold(2.5, -60.0, CAP_12H)).abs() < 1e-9);
    }

    #[test]
    fn elapsed_beyond_cap_is_clamped() {
        // Away a full day, but only 12h is credited.
        let earned = offline_gold(1.0, DAY, CAP_12H);
        assert!((earned - CAP_12H).abs() < 1e-9);
    }
}
