pub fn parse_signature(sig: &str) -> Result<(u8, u8), String> {
    let parts: Vec<&str> = sig.split('/').collect();
    if parts.len() != 2 {
        return Err("signature must be in the form M/N".to_string());
    }
    let m: u8 = parts[0]
        .parse()
        .map_err(|_| "invalid signature numerator".to_string())?;
    let n: u8 = parts[1]
        .parse()
        .map_err(|_| "invalid signature denominator".to_string())?;
    if m == 0 {
        return Err("numerator must be >= 1".to_string());
    }
    match n {
        1 | 2 | 4 | 8 | 16 => Ok((m, n)),
        _ => Err("denominator must be one of 1,2,4,8,16".to_string()),
    }
}

pub fn parse_duration_ms(src: &str) -> Result<u64, String> {
    if let Some(s) = src.strip_suffix("ms") {
        return s.parse::<u64>().map_err(|_| "invalid ms".to_string());
    }
    if let Some(s) = src.strip_suffix('s') {
        return s
            .parse::<f64>()
            .map(|v| (v * 1000.0) as u64)
            .map_err(|_| "invalid seconds".to_string());
    }
    if let Some(s) = src.strip_suffix('m') {
        return s
            .parse::<f64>()
            .map(|v| (v * 60_000.0) as u64)
            .map_err(|_| "invalid minutes".to_string());
    }
    src.parse::<u64>()
        .map_err(|_| "invalid duration".to_string())
}

pub struct RampCfg {
    pub from_bpm: u16,
    pub to_bpm: u16,
    pub duration_ms: u64,
}

pub fn parse_ramp_pattern(
    p: &str,
    parse_duration_ms_fn: fn(&str) -> Result<u64, String>,
) -> Option<RampCfg> {
    let (range, dur) = p.split_once('@')?;
    let (from, to) = range.split_once("..")?;
    let from_bpm: u16 = from.trim().parse().ok()?;
    let to_bpm: u16 = to.trim().parse().ok()?;
    let duration_ms = parse_duration_ms_fn(dur.trim()).ok()?;
    Some(RampCfg {
        from_bpm,
        to_bpm,
        duration_ms,
    })
}
