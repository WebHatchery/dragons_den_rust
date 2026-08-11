use super::*;

#[test]
fn hex_swatch_parses() {
    let color = parse_hex_color("#FF4500").unwrap();
    assert!((color.r - 1.0).abs() < 1e-6);
    assert!((color.g - 69.0 / 255.0).abs() < 1e-6);
    assert!((color.b - 0.0).abs() < 1e-6);
    assert!(parse_hex_color("nope").is_none());
}
