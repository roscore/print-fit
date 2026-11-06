//! Calibration from a test print: least-squares fit `measured = a * cad + b`, separately for
//! holes and pins, and derivation of profile compensation values.

use crate::profile::Profile;

/// One measured feature: modelled (CAD) size and caliper reading, mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub cad: f64,
    pub measured: f64,
}

/// Result of a linear least-squares fit.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearFit {
    /// Slope.
    pub a: f64,
    /// Intercept, mm.
    pub b: f64,
    /// `measured - (a * cad + b)` per sample, in input order.
    pub residuals: Vec<f64>,
    /// Root-mean-square of residuals.
    pub rms: f64,
    pub n: usize,
    /// True when the slope could not be estimated (one sample, or all CAD sizes equal)
    /// and was fixed to 1.
    pub slope_fixed: bool,
}

impl LinearFit {
    pub fn predict(&self, cad: f64) -> f64 {
        self.a * cad + self.b
    }
}

fn finish(samples: &[Sample], a: f64, b: f64, slope_fixed: bool) -> LinearFit {
    let residuals: Vec<f64> = samples
        .iter()
        .map(|s| s.measured - (a * s.cad + b))
        .collect();
    let rms = (residuals.iter().map(|r| r * r).sum::<f64>() / residuals.len() as f64).sqrt();
    LinearFit {
        a,
        b,
        residuals,
        rms,
        n: samples.len(),
        slope_fixed,
    }
}

/// Ordinary least squares fit of `measured = a * cad + b`.
///
/// Returns `None` for no samples or non-finite values. With a single sample, or when all CAD
/// sizes are (nearly) equal, the slope is fixed to 1 and only the offset is estimated.
pub fn least_squares(samples: &[Sample]) -> Option<LinearFit> {
    if samples.is_empty()
        || samples
            .iter()
            .any(|s| !s.cad.is_finite() || !s.measured.is_finite())
    {
        return None;
    }
    let n = samples.len() as f64;
    let mx = samples.iter().map(|s| s.cad).sum::<f64>() / n;
    let my = samples.iter().map(|s| s.measured).sum::<f64>() / n;
    let sxx: f64 = samples.iter().map(|s| (s.cad - mx).powi(2)).sum();
    let sxy: f64 = samples
        .iter()
        .map(|s| (s.cad - mx) * (s.measured - my))
        .sum();
    if samples.len() < 2 || sxx < 1e-9 {
        return Some(finish(samples, 1.0, my - mx, true));
    }
    let a = sxy / sxx;
    Some(finish(samples, a, my - a * mx, false))
}

/// Compensation derived from hole and pin calibration data.
#[derive(Debug, Clone, PartialEq)]
pub struct Calibration {
    pub holes: Option<LinearFit>,
    pub pins: Option<LinearFit>,
    /// Common scale factor `k` (material shrink = `(1 - k) * 100` %).
    pub scale: f64,
    pub material_shrink_pct: f64,
    /// Present if hole data was given.
    pub hole_shrink_mm: Option<f64>,
    /// Present if pin data was given.
    pub pin_grow_mm: Option<f64>,
    /// Raised when the derived shrink is implausible (|shrink| > 3 %): check the data.
    pub suspicious: bool,
}

/// Derive profile compensation from hole and pin samples.
///
/// The profile model is `hole = k*cad - hole_shrink` and `pin = k*cad + pin_grow` with one
/// shared scale `k`. `k` is the sample-count-weighted mean of the fitted slopes (only fits
/// where a slope could be estimated); with no estimable slope `k = 1`. Given `k`, each offset
/// is the least-squares optimum `mean(measured - k*cad)`.
pub fn derive_compensation(holes: &[Sample], pins: &[Sample]) -> Calibration {
    let hf = least_squares(holes);
    let pf = least_squares(pins);
    let (mut wsum, mut ksum) = (0.0, 0.0);
    for f in [&hf, &pf].into_iter().flatten() {
        if !f.slope_fixed {
            wsum += f.n as f64;
            ksum += f.a * f.n as f64;
        }
    }
    let scale = if wsum > 0.0 { ksum / wsum } else { 1.0 };
    let offset =
        |s: &[Sample]| s.iter().map(|x| x.measured - scale * x.cad).sum::<f64>() / s.len() as f64;
    let hole_shrink_mm = hf.as_ref().map(|_| -offset(holes));
    let pin_grow_mm = pf.as_ref().map(|_| offset(pins));
    let material_shrink_pct = (1.0 - scale) * 100.0;
    Calibration {
        holes: hf,
        pins: pf,
        scale,
        material_shrink_pct,
        hole_shrink_mm,
        pin_grow_mm,
        suspicious: material_shrink_pct.abs() > 3.0,
    }
}

impl Calibration {
    /// Write the derived compensation into a profile. Missing data leaves values unchanged;
    /// the material shrink is only replaced when a slope was actually estimated.
    pub fn apply_to(&self, profile: &mut Profile) {
        let slope_known = [&self.holes, &self.pins]
            .into_iter()
            .flatten()
            .any(|f| !f.slope_fixed);
        if slope_known {
            profile.material_shrink_pct = round4(self.material_shrink_pct);
        }
        if let Some(h) = self.hole_shrink_mm {
            profile.hole_shrink_mm = round4(h);
        }
        if let Some(p) = self.pin_grow_mm {
            profile.pin_grow_mm = round4(p);
        }
    }
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

/// Parse lines like `5.0 4.82`, `5,0; 4,82` or `5.0, 4.82` into samples. Lines that are empty
/// or start with `#` are skipped. Returns the samples and the 1-based numbers of bad lines.
pub fn parse_samples(text: &str) -> (Vec<Sample>, Vec<usize>) {
    let mut out = Vec::new();
    let mut bad = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Accept decimal commas only when the separator is unambiguous (whitespace, ';' or tab).
        let parts: Vec<&str> = if line.contains(';') || line.contains('\t') {
            line.split([';', '\t']).collect()
        } else if line.split_whitespace().count() >= 2 {
            line.split_whitespace().collect()
        } else {
            line.split(',').collect()
        };
        let nums: Vec<f64> = parts
            .iter()
            .map(|p| p.trim().trim_end_matches(',').replace(',', "."))
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse::<f64>().ok())
            .collect();
        if nums.len() == 2 && nums.iter().all(|v| v.is_finite() && *v > 0.0) {
            out.push(Sample {
                cad: nums[0],
                measured: nums[1],
            });
        } else {
            bad.push(i + 1);
        }
    }
    (out, bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth(a: f64, b: f64, cads: &[f64]) -> Vec<Sample> {
        cads.iter()
            .map(|&c| Sample {
                cad: c,
                measured: a * c + b,
            })
            .collect()
    }

    #[test]
    fn recovers_known_line() {
        let s = synth(0.995, -0.17, &[3.0, 5.0, 8.0, 10.0, 12.5]);
        let f = least_squares(&s).unwrap();
        assert!((f.a - 0.995).abs() < 1e-9);
        assert!((f.b + 0.17).abs() < 1e-9);
        assert!(f.rms < 1e-9);
        assert!(f.residuals.iter().all(|r| r.abs() < 1e-9));
        assert!(!f.slope_fixed);
        assert!((f.predict(6.0) - (0.995 * 6.0 - 0.17)).abs() < 1e-9);
    }

    #[test]
    fn residuals_with_noise() {
        let mut s = synth(1.0, 0.1, &[2.0, 4.0, 6.0, 8.0]);
        s[1].measured += 0.02;
        s[2].measured -= 0.02;
        let f = least_squares(&s).unwrap();
        // OLS residuals sum to zero.
        assert!(f.residuals.iter().sum::<f64>().abs() < 1e-12);
        assert!(f.rms > 0.0 && f.rms < 0.02);
    }

    #[test]
    fn degenerate_inputs() {
        assert!(least_squares(&[]).is_none());
        let one = least_squares(&[Sample {
            cad: 5.0,
            measured: 4.8,
        }])
        .unwrap();
        assert!(one.slope_fixed && one.a == 1.0 && (one.b + 0.2).abs() < 1e-12);
        let same = least_squares(&synth(1.0, 0.0, &[5.0, 5.0, 5.0])).unwrap();
        assert!(same.slope_fixed);
        assert!(least_squares(&[Sample {
            cad: f64::NAN,
            measured: 1.0
        }])
        .is_none());
    }

    #[test]
    fn derive_and_apply_compensation() {
        let k = 0.996;
        let holes = synth(k, -0.18, &[3.0, 5.0, 8.0, 10.0]);
        let pins = synth(k, 0.07, &[3.0, 5.0, 8.0]);
        let c = derive_compensation(&holes, &pins);
        assert!((c.scale - k).abs() < 1e-9);
        assert!((c.material_shrink_pct - 0.4).abs() < 1e-7);
        assert!((c.hole_shrink_mm.unwrap() - 0.18).abs() < 1e-9);
        assert!((c.pin_grow_mm.unwrap() - 0.07).abs() < 1e-9);
        assert!(!c.suspicious);

        let mut p = Profile::default();
        c.apply_to(&mut p);
        assert_eq!(p.hole_shrink_mm, 0.18);
        assert_eq!(p.pin_grow_mm, 0.07);
        assert_eq!(p.material_shrink_pct, 0.4);
        // Calibrated profile predicts the measurements.
        for s in &holes {
            assert!((crate::fit::printed_hole(&p, s.cad) - s.measured).abs() < 1e-3);
        }
    }

    #[test]
    fn holes_only_keeps_pin_value() {
        let holes = [Sample {
            cad: 5.0,
            measured: 4.75,
        }];
        let c = derive_compensation(&holes, &[]);
        assert_eq!(c.scale, 1.0);
        let mut p = Profile::default();
        p.material_shrink_pct = 0.2;
        let pin_before = p.pin_grow_mm;
        c.apply_to(&mut p);
        assert!((p.hole_shrink_mm - 0.25).abs() < 1e-12);
        assert_eq!(p.pin_grow_mm, pin_before);
        assert_eq!(p.material_shrink_pct, 0.2, "no slope → shrink untouched");
    }

    #[test]
    fn parse_lines() {
        let (s, bad) = parse_samples("# cad measured\n5 4.8\n6.0, 5.85\n7,0;6,9\n\noops\n8\t7.9");
        assert_eq!(s.len(), 4);
        assert_eq!(
            s[2],
            Sample {
                cad: 7.0,
                measured: 6.9
            }
        );
        assert_eq!(
            s[3],
            Sample {
                cad: 8.0,
                measured: 7.9
            }
        );
        assert_eq!(bad, vec![6]);
    }
}
