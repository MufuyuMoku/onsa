//! Equalizer APO / AutoEQ presets (`ParametricEQ.txt`, SPEC §4.2).
//!
//! ```text
//! Preamp: -6.2 dB
//! Filter 1: ON PK Fc 105 Hz Gain -2.1 dB Q 0.70
//! Filter 2: ON LSC Fc 105 Hz Gain 5.5 dB Q 0.70
//! ```
//!
//! Lines that cannot be understood are skipped with a warning; one bad line
//! never fails the whole import.

use super::biquad::{Band, FilterKind};
use super::eq::MAX_BANDS;

/// A parsed preset.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preset {
    /// The `Preamp:` line, when present.
    pub preamp_db: Option<f64>,
    /// The enabled filters, in file order.
    pub bands: Vec<Band>,
}

/// A preset and everything that had to be skipped to read it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Import {
    /// What could be read.
    pub preset: Preset,
    /// One line per skipped line or setting.
    pub warnings: Vec<String>,
}

/// Reads a preset.
pub fn parse(text: &str) -> Import {
    let mut import = Import::default();
    for (index, raw) in text.lines().enumerate() {
        let number = index + 1;
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("preamp:") {
            match first_number(rest) {
                Some(db) => import.preset.preamp_db = Some(db),
                None => import
                    .warnings
                    .push(format!("line {number}: preamp without a value, ignored")),
            }
            continue;
        }
        if lower.starts_with("filter") {
            match parse_filter(line) {
                Ok(Some(band)) => {
                    if import.preset.bands.len() < MAX_BANDS {
                        import.preset.bands.push(band);
                    } else {
                        import.warnings.push(format!(
                            "line {number}: more than {MAX_BANDS} filters, ignored"
                        ));
                    }
                }
                Ok(None) => {} // an OFF filter
                Err(reason) => import
                    .warnings
                    .push(format!("line {number}: {reason}, ignored")),
            }
            continue;
        }
        import
            .warnings
            .push(format!("line {number}: not understood, ignored"));
    }
    import
}

/// Writes a preset in the same format.
pub fn export(preamp_db: f64, bands: &[Band]) -> String {
    let mut out = format!("Preamp: {} dB\n", tidy(preamp_db, 1));
    for (index, band) in bands.iter().enumerate() {
        let state = if band.enabled { "ON" } else { "OFF" };
        let kind = match band.kind {
            FilterKind::Peaking => "PK",
            FilterKind::LowShelf => "LSC",
            FilterKind::HighShelf => "HSC",
            FilterKind::LowPass => "LP",
            FilterKind::HighPass => "HP",
            FilterKind::Notch => "NO",
        };
        out.push_str(&format!(
            "Filter {}: {state} {kind} Fc {} Hz",
            index + 1,
            tidy(band.freq, 1)
        ));
        if band.kind.uses_gain() {
            out.push_str(&format!(" Gain {} dB", tidy(band.gain_db, 1)));
        }
        out.push_str(&format!(" Q {}\n", tidy(band.q, 2)));
    }
    out
}

/// A number with at most `places` decimals and no trailing zeros.
fn tidy(value: f64, places: usize) -> String {
    let text = format!("{value:.places$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    };
    if text == "-0" {
        "0".to_string()
    } else {
        text
    }
}

fn number(token: &str) -> Option<f64> {
    token
        .trim_end_matches(|c: char| c.is_ascii_alphabetic())
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn first_number(text: &str) -> Option<f64> {
    text.split_whitespace().find_map(number)
}

/// `Ok(None)` for a filter that is switched off.
fn parse_filter(line: &str) -> Result<Option<Band>, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let state = tokens
        .iter()
        .position(|t| t.eq_ignore_ascii_case("on") || t.eq_ignore_ascii_case("off"))
        .ok_or("filter without ON or OFF")?;
    if tokens[state].eq_ignore_ascii_case("off") {
        return Ok(None);
    }
    let kind_token = tokens.get(state + 1).ok_or("filter without a type")?;
    let kind = match kind_token.to_ascii_uppercase().as_str() {
        "PK" | "PEQ" | "MODAL" => FilterKind::Peaking,
        "LSC" | "LS" | "LSQ" => FilterKind::LowShelf,
        "HSC" | "HS" | "HSQ" => FilterKind::HighShelf,
        "LP" | "LPQ" => FilterKind::LowPass,
        "HP" | "HPQ" => FilterKind::HighPass,
        "NO" => FilterKind::Notch,
        other => return Err(format!("unsupported filter type {other}")),
    };
    let value_after = |key: &str| {
        tokens
            .iter()
            .position(|t| t.eq_ignore_ascii_case(key))
            .and_then(|i| tokens.get(i + 1))
            .and_then(|t| number(t))
    };
    let freq = value_after("fc").ok_or("filter without a frequency")?;
    if freq <= 0.0 {
        return Err("filter frequency must be positive".to_string());
    }
    let gain_db = value_after("gain").unwrap_or(0.0);
    if kind.uses_gain() && value_after("gain").is_none() {
        return Err("filter without a gain".to_string());
    }
    if tokens.iter().any(|t| t.eq_ignore_ascii_case("bw")) {
        return Err("bandwidth in octaves is not supported, use Q".to_string());
    }
    let q = value_after("q").unwrap_or(std::f64::consts::FRAC_1_SQRT_2);
    if q <= 0.0 {
        return Err("filter Q must be positive".to_string());
    }
    Ok(Some(Band {
        kind,
        freq,
        gain_db,
        q,
        enabled: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTOEQ: &str = "Preamp: -6.2 dB
Filter 1: ON LSC Fc 105 Hz Gain 5.5 dB Q 0.70
Filter 2: ON PK Fc 43 Hz Gain -1.9 dB Q 0.40
Filter 3: ON PK Fc 1953 Hz Gain -2.4 dB Q 1.62
Filter 4: ON HSC Fc 10000 Hz Gain -3.1 dB Q 0.70
Filter 5: ON LP Fc 18000 Hz Q 0.71
";

    #[test]
    fn reads_an_autoeq_file() {
        let import = parse(AUTOEQ);
        assert!(import.warnings.is_empty(), "{:?}", import.warnings);
        assert_eq!(import.preset.preamp_db, Some(-6.2));
        let bands = &import.preset.bands;
        assert_eq!(bands.len(), 5);
        assert_eq!(bands[0].kind, FilterKind::LowShelf);
        assert_eq!(
            (bands[2].freq, bands[2].gain_db, bands[2].q),
            (1953.0, -2.4, 1.62)
        );
        assert_eq!(bands[4].kind, FilterKind::LowPass);
    }

    #[test]
    fn skips_broken_lines_and_keeps_the_rest() {
        let text = "\u{feff}Preamp: loud
# a comment
Filter 1: ON PK Fc 100 Hz Gain -3 dB Q 1
Filter 2: ON XX Fc 100 Hz Gain 1 dB Q 1
Filter 3: ON PK Gain 2 dB Q 1
Filter 4: OFF PK Fc 500 Hz Gain 9 dB Q 1
Filter 5: ON PK Fc 800 Hz Gain 2 dB BW Oct 1
Filter 6: ON PK Fc 2000 Hz Q 1
Channel: L
Filter 7: ON HSC Fc 8000 Hz Gain 2,5 dB Q 0.7
";
        let import = parse(text);
        let freqs: Vec<f64> = import.preset.bands.iter().map(|b| b.freq).collect();
        assert_eq!(freqs, vec![100.0, 8000.0]);
        assert_eq!(import.preset.bands[1].gain_db, 2.5);
        assert_eq!(import.preset.preamp_db, None);
        // Preamp, XX, missing Fc, BW, missing gain and "Channel:" are reported.
        assert_eq!(import.warnings.len(), 6, "{:#?}", import.warnings);
    }

    #[test]
    fn keeps_at_most_sixteen_filters() {
        let text: String = (1..=20)
            .map(|i| format!("Filter {i}: ON PK Fc {} Hz Gain 1 dB Q 1\n", i * 100))
            .collect();
        let import = parse(&text);
        assert_eq!(import.preset.bands.len(), MAX_BANDS);
        assert_eq!(import.warnings.len(), 4);
    }

    #[test]
    fn export_reads_back_the_same_preset() {
        let import = parse(AUTOEQ);
        let text = export(-6.2, &import.preset.bands);
        assert!(text.starts_with("Preamp: -6.2 dB\nFilter 1: ON LSC Fc 105 Hz Gain 5.5 dB Q 0.7\n"));
        assert_eq!(parse(&text), import);
    }
}
