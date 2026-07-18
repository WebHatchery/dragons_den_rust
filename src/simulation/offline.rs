//! Offline / idle progress (GDD §5.2): the same passive-income rate the live
//! tick uses, applied to wall-clock time elapsed since the last save.
//!
//! Uncapped by design (GDD §12 open question 3 — "idle time is real time");
//! if a cap is added later it belongs here, not at the call site.

/// Gold earned while away. Negative elapsed time (clock skew, corrupted save
/// timestamps) earns nothing rather than debiting the hoard.
pub fn offline_gold(gold_per_second: f64, seconds_elapsed: f64) -> f64 {
    gold_per_second * seconds_elapsed.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accrues_at_live_rate() {
        assert!((offline_gold(2.5, 100.0) - 250.0).abs() < 1e-9);
    }

    #[test]
    fn negative_elapsed_earns_nothing() {
        assert!((offline_gold(2.5, -60.0)).abs() < 1e-9);
    }
}
