//! Metric screw cheat sheet (M2–M8).
//!
//! Typical values gathered from common practice and ISO nominal dimensions. Self-tapping and
//! heat-set insert holes in particular vary by material and insert brand — treat every value
//! as a starting point and print a test.

use crate::profile::{Process, Profile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrewSize {
    pub name: &'static str,
    /// Nominal major diameter.
    pub d: f64,
    pub pitch: f64,
    /// Printed hole for self-tapping a machine/plastite screw into plastic (CAD size).
    pub self_tap_hole: f64,
    /// Clearance hole, ISO 273 "medium" series.
    pub clearance_hole: f64,
    /// Typical hole for a heat-set threaded insert (check the insert's datasheet).
    pub heat_set_hole: f64,
    /// Hex nut width across flats (ISO 4032).
    pub nut_af: f64,
    /// Hex nut thickness (ISO 4032).
    pub nut_thickness: f64,
}

pub const SCREWS: [ScrewSize; 7] = [
    ScrewSize {
        name: "M2",
        d: 2.0,
        pitch: 0.4,
        self_tap_hole: 1.6,
        clearance_hole: 2.4,
        heat_set_hole: 3.2,
        nut_af: 4.0,
        nut_thickness: 1.6,
    },
    ScrewSize {
        name: "M2.5",
        d: 2.5,
        pitch: 0.45,
        self_tap_hole: 2.0,
        clearance_hole: 2.9,
        heat_set_hole: 3.6,
        nut_af: 5.0,
        nut_thickness: 2.0,
    },
    ScrewSize {
        name: "M3",
        d: 3.0,
        pitch: 0.5,
        self_tap_hole: 2.5,
        clearance_hole: 3.4,
        heat_set_hole: 4.0,
        nut_af: 5.5,
        nut_thickness: 2.4,
    },
    ScrewSize {
        name: "M4",
        d: 4.0,
        pitch: 0.7,
        self_tap_hole: 3.3,
        clearance_hole: 4.5,
        heat_set_hole: 5.6,
        nut_af: 7.0,
        nut_thickness: 3.2,
    },
    ScrewSize {
        name: "M5",
        d: 5.0,
        pitch: 0.8,
        self_tap_hole: 4.2,
        clearance_hole: 5.5,
        heat_set_hole: 6.4,
        nut_af: 8.0,
        nut_thickness: 4.7,
    },
    ScrewSize {
        name: "M6",
        d: 6.0,
        pitch: 1.0,
        self_tap_hole: 5.0,
        clearance_hole: 6.6,
        heat_set_hole: 8.0,
        nut_af: 10.0,
        nut_thickness: 5.2,
    },
    ScrewSize {
        name: "M8",
        d: 8.0,
        pitch: 1.25,
        self_tap_hole: 6.8,
        clearance_hole: 9.0,
        heat_set_hole: 10.0,
        nut_af: 13.0,
        nut_thickness: 6.8,
    },
];

/// Default nut-trap clearance (diametral, across flats) as printed.
pub fn nut_trap_clearance(process: Process) -> f64 {
    if process.is_resin() {
        0.1
    } else {
        0.2
    }
}

/// Nut trap CAD size: `(across_flats, across_corners)`, compensated for hole shrink and
/// material shrink. Across corners is `af / cos(30°)`.
pub fn nut_trap(profile: &Profile, screw: &ScrewSize) -> (f64, f64) {
    let target = screw.nut_af + nut_trap_clearance(profile.process);
    let af = crate::fit::cad_hole_for(profile, target);
    (af, af * 2.0 / 3f64.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_ordered_and_consistent() {
        for w in SCREWS.windows(2) {
            assert!(w[0].d < w[1].d);
        }
        for s in &SCREWS {
            assert!(s.self_tap_hole < s.d, "{}", s.name);
            assert!(s.clearance_hole > s.d, "{}", s.name);
            assert!(s.heat_set_hole > s.d, "{}", s.name);
            assert!(s.nut_af > s.clearance_hole, "{}", s.name);
            assert!((s.d - s.pitch * 1.0825 - s.self_tap_hole).abs() < 0.5);
        }
    }

    #[test]
    fn nut_trap_math() {
        let mut p = Profile::preset(Process::Fdm04);
        p.hole_shrink_mm = 0.1;
        p.material_shrink_pct = 0.0;
        let m3 = SCREWS[2];
        let (af, ac) = nut_trap(&p, &m3);
        assert!((af - (5.5 + 0.2 + 0.1)).abs() < 1e-9);
        assert!((ac - af / (std::f64::consts::PI / 6.0).cos()).abs() < 1e-9);
    }
}
