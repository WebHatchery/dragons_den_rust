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
