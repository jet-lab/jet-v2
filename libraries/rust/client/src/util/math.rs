pub fn f64_to_bps(f: impl Into<f64>) -> u64 {
    let bps = f.into() * 10_000.0;
    assert!(bps <= u64::MAX as f64);
    assert!(bps >= 0.0);
    bps.round() as u64
}

pub fn bps_to_f64(bps: impl Into<u64>) -> f64 {
    bps.into() as f64 / 10_000.0
}
