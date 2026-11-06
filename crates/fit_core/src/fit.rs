//! Fit classes, hole/shaft compensation and polygon (faceted hole) correction.

use std::f64::consts::PI;
use std::fmt;

use crate::profile::{Process, Profile};

/// Shape of the mating features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    /// Round hole + pin/shaft. Sizes are diameters.
    RoundHole,
    /// Square/rectangular slot + tab. Sizes are widths.
    SquareSlot,
}

/// How tight the mating parts should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitClass {
    Press,
    Snug,
    Slide,
    Loose,
    FreeSpin,
}

impl FitClass {
    pub const ALL: [FitClass; 5] = [
        FitClass::Press,
        FitClass::Snug,
        FitClass::Slide,
        FitClass::Loose,
        FitClass::FreeSpin,
    ];

    pub fn id(self) -> &'static str {
        match self {
            FitClass::Press => "press",
            FitClass::Snug => "snug",
            FitClass::Slide => "slide",
            FitClass::Loose => "loose",
            FitClass::FreeSpin => "free_spin",
        }
    }

    pub fn from_id(id: &str) -> Option<FitClass> {
        FitClass::ALL.into_iter().find(|c| c.id() == id)
    }
}

/// Which side keeps the nominal size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datum {
    /// Both parts are printed; clearance is split half on the hole, half on the shaft.
    Split,
    /// The shaft/tab is a bought or fixed part (e.g. steel dowel) at nominal size;
    /// the printed hole takes all the clearance.
    ShaftFixed,
    /// The hole is a bought or fixed part (e.g. bearing bore) at nominal size;
    /// the printed shaft takes all the clearance.
    HoleFixed,
}

impl Datum {
    pub const ALL: [Datum; 3] = [Datum::Split, Datum::ShaftFixed, Datum::HoleFixed];

    pub fn id(self) -> &'static str {
        match self {
            Datum::Split => "split",
            Datum::ShaftFixed => "shaft_fixed",
            Datum::HoleFixed => "hole_fixed",
        }
    }

    pub fn from_id(id: &str) -> Option<Datum> {
        Datum::ALL.into_iter().find(|d| d.id() == id)
    }
}

/// Diametral clearance range (hole size minus shaft size, as printed). Negative = interference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clearance {
    pub min: f64,
    pub max: f64,
}

impl Clearance {
    const fn exact(v: f64) -> Clearance {
        Clearance { min: v, max: v }
    }

    /// Target value: the middle of the range.
    pub fn nominal(&self) -> f64 {
        (self.min + self.max) / 2.0
    }
}

/// Default diametral clearance for a fit class and process.
///
/// | class      | FDM 0.4      | FDM 0.6      | Resin MSLA   |
/// |------------|--------------|--------------|--------------|
/// | press      | -0.05 .. 0   | -0.05 .. 0   | -0.03 .. 0   |
/// | snug       | 0.10         | 0.15         | 0.05         |
/// | slide      | 0.20         | 0.25         | 0.10         |
/// | loose      | 0.30         | 0.35         | 0.15         |
/// | free-spin  | 0.40         | 0.50         | 0.20         |
///
/// These are the clearances *as printed* (after compensation). They are common community
/// starting points; print a test coupon to find your own.
pub fn fit_clearance(process: Process, class: FitClass) -> Clearance {
    use FitClass::*;
    match (process, class) {
        (Process::Fdm04, Press) => Clearance {
            min: -0.05,
            max: 0.0,
        },
        (Process::Fdm04, Snug) => Clearance::exact(0.10),
        (Process::Fdm04, Slide) => Clearance::exact(0.20),
        (Process::Fdm04, Loose) => Clearance::exact(0.30),
        (Process::Fdm04, FreeSpin) => Clearance::exact(0.40),
        (Process::Fdm06, Press) => Clearance {
            min: -0.05,
            max: 0.0,
        },
        (Process::Fdm06, Snug) => Clearance::exact(0.15),
        (Process::Fdm06, Slide) => Clearance::exact(0.25),
        (Process::Fdm06, Loose) => Clearance::exact(0.35),
        (Process::Fdm06, FreeSpin) => Clearance::exact(0.50),
        (Process::ResinMsla, Press) => Clearance {
            min: -0.03,
            max: 0.0,
        },
        (Process::ResinMsla, Snug) => Clearance::exact(0.05),
        (Process::ResinMsla, Slide) => Clearance::exact(0.10),
        (Process::ResinMsla, Loose) => Clearance::exact(0.15),
        (Process::ResinMsla, FreeSpin) => Clearance::exact(0.20),
    }
}

/// Circumscribed diameter of a regular N-gon whose *inscribed* circle has diameter `d`:
/// `d / cos(pi / N)`.
///
/// CAD/mesh exports approximate a round hole with N straight segments through the vertices,
/// so the flat-to-flat size is smaller than the modelled diameter. Model the hole at the
/// returned size so the flats land on `d`. Returns `None` for `n < 3`.
pub fn circumscribed_diameter(d: f64, n: u32) -> Option<f64> {
    if n < 3 {
        return None;
    }
    Some(d / (PI / f64::from(n)).cos())
}

/// Inverse of [`circumscribed_diameter`]: flat-to-flat size of an N-gon with vertex diameter `d`.
pub fn inscribed_diameter(d: f64, n: u32) -> Option<f64> {
    if n < 3 {
        return None;
    }
    Some(d * (PI / f64::from(n)).cos())
}

/// CAD size needed for a hole to print at `target` with this profile.
pub fn cad_hole_for(profile: &Profile, target: f64) -> f64 {
    (target + profile.hole_shrink_mm) / profile.scale()
}

/// CAD size needed for a pin/tab to print at `target` with this profile.
pub fn cad_pin_for(profile: &Profile, target: f64) -> f64 {
    (target - profile.pin_grow_mm) / profile.scale()
}

/// Predicted printed size of a hole modelled at `cad`.
pub fn printed_hole(profile: &Profile, cad: f64) -> f64 {
    cad * profile.scale() - profile.hole_shrink_mm
}

/// Predicted printed size of a pin modelled at `cad`.
pub fn printed_pin(profile: &Profile, cad: f64) -> f64 {
    cad * profile.scale() + profile.pin_grow_mm
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitInput {
    /// Nominal diameter (round) or width (slot), mm.
    pub nominal_mm: f64,
    pub feature: FeatureKind,
    pub class: FitClass,
    pub datum: Datum,
    /// If set, the hole will be exported as an N-sided polygon; report the circumscribed size.
    pub polygon_segments: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitResult {
    /// Diametral clearance range (as printed).
    pub clearance: Clearance,
    /// Target diametral clearance (as printed).
    pub diametral_clearance: f64,
    /// Radial (per-side) clearance = diametral / 2.
    pub radial_clearance: f64,
    /// Size the hole should have after printing.
    pub hole_target: f64,
    /// Size the shaft should have after printing.
    pub shaft_target: f64,
    /// Recommended CAD hole size (compensated) — or nominal if the hole is a fixed part.
    pub cad_hole: f64,
    /// Recommended CAD shaft size (compensated) — or nominal if the shaft is a fixed part.
    pub cad_shaft: f64,
    /// Whether the hole/shaft is printed (and therefore compensated).
    pub hole_printed: bool,
    pub shaft_printed: bool,
    /// Circumscribed CAD hole size if `polygon_segments` was given (round holes only).
    pub cad_hole_faceted: Option<f64>,
    /// Approximate first-layer hole size, reduced by elephant foot on both sides.
    pub first_layer_hole: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FitError {
    InvalidNominal,
    InvalidProfile(String),
    InvalidSegments,
    /// The resulting CAD size would be zero or negative.
    TooSmall,
}

impl fmt::Display for FitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FitError::InvalidNominal => write!(f, "nominal size must be between 0.1 and 1000 mm"),
            FitError::InvalidProfile(e) => write!(f, "invalid profile: {e}"),
            FitError::InvalidSegments => write!(f, "polygon segments must be at least 3"),
            FitError::TooSmall => write!(f, "resulting size is too small"),
        }
    }
}

impl std::error::Error for FitError {}

/// Compute recommended CAD sizes for a fit.
pub fn compute_fit(profile: &Profile, input: &FitInput) -> Result<FitResult, FitError> {
    let d = input.nominal_mm;
    if !d.is_finite() || !(0.1..=1000.0).contains(&d) {
        return Err(FitError::InvalidNominal);
    }
    profile.validate().map_err(FitError::InvalidProfile)?;
    if matches!(input.polygon_segments, Some(n) if n < 3) {
        return Err(FitError::InvalidSegments);
    }

    let clearance = fit_clearance(profile.process, input.class);
    let c = clearance.nominal();
    let (hole_target, shaft_target) = match input.datum {
        Datum::Split => (d + c / 2.0, d - c / 2.0),
        Datum::ShaftFixed => (d + c, d),
        Datum::HoleFixed => (d, d - c),
    };
    let hole_printed = input.datum != Datum::HoleFixed;
    let shaft_printed = input.datum != Datum::ShaftFixed;
    let cad_hole = if hole_printed {
        cad_hole_for(profile, hole_target)
    } else {
        hole_target
    };
    let cad_shaft = if shaft_printed {
        cad_pin_for(profile, shaft_target)
    } else {
        shaft_target
    };
    if cad_hole <= 0.0 || cad_shaft <= 0.0 {
        return Err(FitError::TooSmall);
    }
    let cad_hole_faceted = match (input.feature, input.polygon_segments, hole_printed) {
        (FeatureKind::RoundHole, Some(n), true) => circumscribed_diameter(cad_hole, n),
        _ => None,
    };
    Ok(FitResult {
        clearance,
        diametral_clearance: c,
        radial_clearance: c / 2.0,
        hole_target,
        shaft_target,
        cad_hole,
        cad_shaft,
        hole_printed,
        shaft_printed,
        cad_hole_faceted,
        first_layer_hole: hole_target - 2.0 * profile.elephant_foot_mm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn fit_table_fdm_defaults() {
        let p = Process::Fdm04;
        assert_eq!(
            fit_clearance(p, FitClass::Press),
            Clearance {
                min: -0.05,
                max: 0.0
            }
        );
        assert!(close(fit_clearance(p, FitClass::Press).nominal(), -0.025));
        assert!(close(fit_clearance(p, FitClass::Snug).nominal(), 0.10));
        assert!(close(fit_clearance(p, FitClass::Slide).nominal(), 0.20));
        assert!(close(fit_clearance(p, FitClass::Loose).nominal(), 0.30));
        assert!(close(fit_clearance(p, FitClass::FreeSpin).nominal(), 0.40));
    }

    #[test]
    fn fit_table_is_monotonic_and_resin_is_tighter() {
        for p in Process::ALL {
            let v: Vec<f64> = FitClass::ALL
                .iter()
                .map(|&c| fit_clearance(p, c).nominal())
                .collect();
            assert!(v.windows(2).all(|w| w[0] < w[1]), "{p:?} not monotonic");
        }
        for c in FitClass::ALL {
            let fdm = fit_clearance(Process::Fdm04, c);
            let resin = fit_clearance(Process::ResinMsla, c);
            assert!(resin.nominal().abs() < fdm.nominal().abs(), "{c:?}");
            assert!(fit_clearance(Process::Fdm06, c).max >= fdm.max);
        }
        for c in FitClass::ALL {
            assert_eq!(FitClass::from_id(c.id()), Some(c));
        }
    }

    #[test]
    fn hole_and_shaft_compensation() {
        let mut prof = Profile::preset(Process::Fdm04);
        prof.hole_shrink_mm = 0.2;
        prof.pin_grow_mm = 0.1;
        prof.material_shrink_pct = 0.0;
        let input = FitInput {
            nominal_mm: 8.0,
            feature: FeatureKind::RoundHole,
            class: FitClass::Slide,
            datum: Datum::Split,
            polygon_segments: None,
        };
        let r = compute_fit(&prof, &input).unwrap();
        assert!(close(r.diametral_clearance, 0.2));
        assert!(close(r.radial_clearance, 0.1));
        assert!(close(r.hole_target, 8.1));
        assert!(close(r.shaft_target, 7.9));
        assert!(close(r.cad_hole, 8.3));
        assert!(close(r.cad_shaft, 7.8));
        // Predicted printed sizes give back the target clearance.
        let gap = printed_hole(&prof, r.cad_hole) - printed_pin(&prof, r.cad_shaft);
        assert!(close(gap, 0.2));
        assert!(close(r.first_layer_hole, 8.1 - 0.4));
    }

    #[test]
    fn material_shrink_scales_cad_up() {
        let mut prof = Profile::preset(Process::ResinMsla);
        prof.hole_shrink_mm = 0.0;
        prof.pin_grow_mm = 0.0;
        prof.material_shrink_pct = 1.0;
        let target = 9.9;
        let cad = cad_hole_for(&prof, target);
        assert!(close(cad, 10.0));
        assert!(close(printed_hole(&prof, cad), target));
        assert!(close(printed_pin(&prof, cad_pin_for(&prof, 5.0)), 5.0));
    }

    #[test]
    fn datum_variants() {
        let prof = Profile::preset(Process::Fdm04);
        let mut input = FitInput {
            nominal_mm: 5.0,
            feature: FeatureKind::SquareSlot,
            class: FitClass::Loose,
            datum: Datum::ShaftFixed,
            polygon_segments: Some(32),
        };
        let r = compute_fit(&prof, &input).unwrap();
        assert!(!r.shaft_printed && r.hole_printed);
        assert!(close(r.cad_shaft, 5.0));
        assert!(close(r.hole_target, 5.3));
        assert!(close(r.cad_hole, 5.3 + prof.hole_shrink_mm));
        assert_eq!(r.cad_hole_faceted, None, "no polygon correction for slots");

        input.datum = Datum::HoleFixed;
        input.feature = FeatureKind::RoundHole;
        let r = compute_fit(&prof, &input).unwrap();
        assert!(r.shaft_printed && !r.hole_printed);
        assert!(close(r.cad_hole, 5.0));
        assert!(close(r.cad_shaft, 4.7 - prof.pin_grow_mm));
        assert_eq!(r.cad_hole_faceted, None, "fixed hole is not exported");
    }

    #[test]
    fn press_fit_is_interference() {
        let prof = Profile::preset(Process::Fdm04);
        let input = FitInput {
            nominal_mm: 3.0,
            feature: FeatureKind::RoundHole,
            class: FitClass::Press,
            datum: Datum::Split,
            polygon_segments: None,
        };
        let r = compute_fit(&prof, &input).unwrap();
        assert!(r.diametral_clearance < 0.0);
        assert!(r.hole_target < r.shaft_target);
    }

    #[test]
    fn polygon_correction() {
        let d = 10.0;
        assert!((circumscribed_diameter(d, 4).unwrap() - d * 2f64.sqrt()).abs() < EPS * 10.0);
        assert!((circumscribed_diameter(d, 6).unwrap() - 2.0 * d / 3f64.sqrt()).abs() < 1e-9);
        assert!((circumscribed_diameter(d, 1_000_000).unwrap() - d).abs() < 1e-9);
        assert!(circumscribed_diameter(d, 2).is_none());
        let c = circumscribed_diameter(d, 48).unwrap();
        assert!(c > d);
        assert!((inscribed_diameter(c, 48).unwrap() - d).abs() < 1e-9);

        let prof = Profile::preset(Process::Fdm04);
        let input = FitInput {
            nominal_mm: d,
            feature: FeatureKind::RoundHole,
            class: FitClass::Snug,
            datum: Datum::Split,
            polygon_segments: Some(6),
        };
        let r = compute_fit(&prof, &input).unwrap();
        let faceted = r.cad_hole_faceted.unwrap();
        assert!((faceted - r.cad_hole * 2.0 / 3f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn invalid_inputs() {
        let prof = Profile::default();
        let mut input = FitInput {
            nominal_mm: 0.0,
            feature: FeatureKind::RoundHole,
            class: FitClass::Snug,
            datum: Datum::Split,
            polygon_segments: None,
        };
        assert_eq!(compute_fit(&prof, &input), Err(FitError::InvalidNominal));
        input.nominal_mm = f64::NAN;
        assert_eq!(compute_fit(&prof, &input), Err(FitError::InvalidNominal));
        input.nominal_mm = 5.0;
        input.polygon_segments = Some(2);
        assert_eq!(compute_fit(&prof, &input), Err(FitError::InvalidSegments));
        input.polygon_segments = None;
        input.nominal_mm = 0.1;
        input.class = FitClass::FreeSpin;
        assert_eq!(compute_fit(&prof, &input), Err(FitError::TooSmall));
    }
}
