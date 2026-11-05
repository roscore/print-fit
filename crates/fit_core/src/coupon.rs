//! Tolerance test coupon: a flat plate with a row of through-holes (or pegs) stepping the
//! clearance, each step marked with tally bumps (bar = 5, dot = 1; step 1 is the first value).
//!
//! The plate is triangulated cell by cell. Each cell is a rectangle around one circle; rays
//! from the circle centre through every circle vertex hit the rectangle, and the annulus is
//! fanned between the circle polygon and those hit points. The direction set is built with
//! exact quadrant symmetry, so the points on an edge shared by two cells are bit-identical and
//! the plate is a single closed, manifold shell. Tally bumps are separate closed boxes that
//! overlap the plate slightly (slicers merge overlapping shells).

use crate::stl::{Mesh, V3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouponKind {
    /// Through-holes of size `nominal + clearance`; test with a pin of size `nominal`.
    Holes,
    /// Pegs of size `nominal - clearance`; test with a hole of size `nominal`.
    Pegs,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouponParams {
    pub kind: CouponKind,
    pub nominal_mm: f64,
    pub start_mm: f64,
    pub end_mm: f64,
    pub step_mm: f64,
    pub plate_thickness_mm: f64,
    pub peg_height_mm: f64,
    /// Material between features (and around the row).
    pub wall_mm: f64,
    /// Segments per circle; rounded up to a multiple of 4, clamped to 12..=256.
    pub segments: u32,
}

impl Default for CouponParams {
    fn default() -> Self {
        CouponParams {
            kind: CouponKind::Holes,
            nominal_mm: 5.0,
            start_mm: -0.1,
            end_mm: 0.5,
            step_mm: 0.1,
            plate_thickness_mm: 3.0,
            peg_height_mm: 6.0,
            wall_mm: 2.5,
            segments: 48,
        }
    }
}

pub const MAX_STEPS: usize = 20;
const MARK_W: f64 = 0.8;
const MARK_GAP: f64 = 0.7;
const BAR_LEN: f64 = 2.2;
const DOT_LEN: f64 = 0.8;
const MARK_BAND: f64 = 3.6;
const MARK_HEIGHT: f64 = 0.4;
const MARK_EMBED: f64 = 0.2;

/// Plate layout in mm. Plate spans `x: 0..width`, `y: 0..height`; the markers sit in the
/// band `y: 0..band`.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub pitch: f64,
    pub width: f64,
    pub height: f64,
    pub band: f64,
    /// Y of all feature centres.
    pub cy: f64,
    /// Cell boundaries, `count + 1` values.
    pub xs: Vec<f64>,
}

impl Layout {
    pub fn center_x(&self, i: usize) -> f64 {
        (self.xs[i] + self.xs[i + 1]) / 2.0
    }
}

/// Marker rectangle `[x0, y0, x1, y1]` in plate coordinates.
pub type Rect = [f64; 4];

impl CouponParams {
    /// Effective segment count (multiple of 4, 12..=256).
    pub fn effective_segments(&self) -> u32 {
        (self.segments.clamp(12, 256)).div_ceil(4) * 4
    }

    pub fn validate(&self) -> Result<(), String> {
        let checks = [
            ("nominal", self.nominal_mm, 1.0, 50.0),
            ("start", self.start_mm, -2.0, 2.0),
            ("end", self.end_mm, -2.0, 2.0),
            ("step", self.step_mm, 0.005, 2.0),
            ("plate thickness", self.plate_thickness_mm, 1.0, 10.0),
            ("peg height", self.peg_height_mm, 1.0, 30.0),
            ("wall", self.wall_mm, 1.0, 10.0),
        ];
        for (name, v, lo, hi) in checks {
            if !v.is_finite() || v < lo || v > hi {
                return Err(format!("{name} must be in [{lo}, {hi}]"));
            }
        }
        if self.end_mm < self.start_mm {
            return Err("end must be ≥ start".into());
        }
        let n = self.step_count_unchecked();
        if n > MAX_STEPS {
            return Err(format!("too many steps ({n} > {MAX_STEPS})"));
        }
        if self.sizes_unchecked().iter().any(|&s| s < 0.5) {
            return Err("feature size below 0.5 mm".into());
        }
        Ok(())
    }

    fn step_count_unchecked(&self) -> usize {
        ((self.end_mm - self.start_mm) / self.step_mm + 1e-6).floor() as usize + 1
    }

    /// Clearance per step, rounded to 1 µm.
    pub fn clearances(&self) -> Vec<f64> {
        (0..self.step_count_unchecked().min(MAX_STEPS))
            .map(|i| {
                let c = self.start_mm + i as f64 * self.step_mm;
                let r = (c * 1000.0).round() / 1000.0;
                if r == 0.0 {
                    0.0
                } else {
                    r
                }
            })
            .collect()
    }

    fn sizes_unchecked(&self) -> Vec<f64> {
        self.clearances()
            .into_iter()
            .map(|c| match self.kind {
                CouponKind::Holes => self.nominal_mm + c,
                CouponKind::Pegs => self.nominal_mm - c,
            })
            .collect()
    }

    /// CAD feature size (diameter) per step.
    pub fn feature_sizes(&self) -> Vec<f64> {
        self.sizes_unchecked()
    }

    pub fn layout(&self) -> Layout {
        let sizes = self.sizes_unchecked();
        let dmax = sizes.iter().cloned().fold(0.0, f64::max);
        let count = sizes.len();
        let max_marks = marks_width(count);
        let pitch = (dmax + 2.0 * self.wall_mm).max(max_marks + 1.5);
        let band = MARK_BAND;
        let height = band + dmax + 2.0 * self.wall_mm;
        let cy = band + self.wall_mm + dmax / 2.0;
        let xs = (0..=count).map(|i| i as f64 * pitch).collect();
        Layout {
            pitch,
            width: count as f64 * pitch,
            height,
            band,
            cy,
            xs,
        }
    }

    /// Tally marker rectangles for step `i` (0-based; shows `i + 1`).
    pub fn markers(&self, layout: &Layout, i: usize) -> Vec<Rect> {
        let n = i + 1;
        let (bars, dots) = (n / 5, n % 5);
        let total = bars + dots;
        let w = marks_width(n);
        let mut x = layout.center_x(i) - w / 2.0;
        let y0 = (layout.band - BAR_LEN) / 2.0;
        (0..total)
            .map(|k| {
                let len = if k < bars { BAR_LEN } else { DOT_LEN };
                let r = [x, y0, x + MARK_W, y0 + len];
                x += MARK_W + MARK_GAP;
                r
            })
            .collect()
    }

    /// Build the coupon mesh.
    pub fn mesh(&self) -> Result<Mesh, String> {
        self.validate()?;
        let layout = self.layout();
        let sizes = self.sizes_unchecked();
        let dirs = directions(self.effective_segments());
        let t = self.plate_thickness_mm;
        let mut top: Vec<[V3; 3]> = Vec::new();
        let mut mesh = Mesh::new();
        let count = sizes.len();

        for (i, &d) in sizes.iter().enumerate() {
            let (xl, xr) = (layout.xs[i], layout.xs[i + 1]);
            let cx = layout.center_x(i);
            let cy = layout.cy;
            let r = d / 2.0;
            let cell = Cell {
                xl,
                xr,
                hw: layout.pitch / 2.0,
                h: layout.height,
            };
            let circle: Vec<[f64; 2]> = dirs
                .iter()
                .map(|&(c, s)| [cx + r * c, cy + r * s])
                .collect();
            let hits: Vec<([f64; 2], f64)> =
                dirs.iter().map(|&(c, s)| cell.hit(cx, cy, c, s)).collect();
            let n = dirs.len();
            for k in 0..n {
                let k1 = (k + 1) % n;
                // Polygon (CCW): C_k, P_k, corners.., P_k1, C_k1.
                let mut q: Vec<[f64; 2]> = vec![hits[k].0];
                q.extend(cell.corners_between(hits[k].1, hits[k1].1));
                q.push(hits[k1].0);
                for w in q.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if is_outer(a, b, &layout, count) {
                        let (a0, b0, a1, b1) = (at(a, 0.0), at(b, 0.0), at(a, t), at(b, t));
                        mesh.quad(a0, b0, b1, a1);
                    }
                }
                q.push(circle[k1]);
                let ck = at(circle[k], t);
                for w in q.windows(2) {
                    top.push([ck, at(w[0], t), at(w[1], t)]);
                }
            }

            for k in 0..n {
                let (a, b) = (circle[k], circle[(k + 1) % n]);
                match self.kind {
                    CouponKind::Holes => {
                        mesh.quad(at(b, 0.0), at(a, 0.0), at(a, t), at(b, t));
                    }
                    CouponKind::Pegs => {
                        let z2 = t + self.peg_height_mm;
                        mesh.quad(at(a, t), at(b, t), at(b, z2), at(a, z2));
                        mesh.tri(at([cx, cy], z2), at(a, z2), at(b, z2));
                        mesh.tri(at([cx, cy], 0.0), at(b, 0.0), at(a, 0.0));
                    }
                }
            }

            for m in self.markers(&layout, i) {
                mesh.add_box([m[0], m[1], t - MARK_EMBED], [m[2], m[3], t + MARK_HEIGHT]);
            }
        }

        for tri in top {
            let [a, b, c] = tri;
            mesh.tri(a, b, c);
            mesh.tri([a[0], a[1], 0.0], [c[0], c[1], 0.0], [b[0], b[1], 0.0]);
        }
        Ok(mesh)
    }

    /// Binary STL bytes of the coupon.
    pub fn stl(&self) -> Result<Vec<u8>, String> {
        let kind = match self.kind {
            CouponKind::Holes => "holes",
            CouponKind::Pegs => "pegs",
        };
        let header = format!(
            "print-fit coupon {kind} d={} {:+}..{:+} step {}",
            self.nominal_mm, self.start_mm, self.end_mm, self.step_mm
        );
        Ok(self.mesh()?.to_binary_stl(&header))
    }
}

fn marks_width(n: usize) -> f64 {
    let total = (n / 5 + n % 5).max(1) as f64;
    total * MARK_W + (total - 1.0) * MARK_GAP
}

fn at(p: [f64; 2], z: f64) -> V3 {
    [p[0], p[1], z]
}

fn is_outer(a: [f64; 2], b: [f64; 2], layout: &Layout, count: usize) -> bool {
    (a[1] == 0.0 && b[1] == 0.0)
        || (a[1] == layout.height && b[1] == layout.height)
        || (a[0] == layout.xs[0] && b[0] == layout.xs[0])
        || (a[0] == layout.xs[count] && b[0] == layout.xs[count])
}

/// Unit directions at angles `(k + 0.5) * 2π / n`, `n` a multiple of 4, with exact symmetry:
/// the set is closed under `x -> -x` and `y -> -y` bit-for-bit.
fn directions(n: u32) -> Vec<(f64, f64)> {
    let m = (n / 4) as usize;
    let step = std::f64::consts::TAU / f64::from(n);
    let mut q = vec![(0.0, 0.0); m];
    for k in 0..m {
        let j = m - 1 - k;
        if k < j {
            let a = (k as f64 + 0.5) * step;
            q[k] = (a.cos(), a.sin());
        } else if k == j {
            let v = std::f64::consts::FRAC_1_SQRT_2;
            q[k] = (v, v);
        } else {
            q[k] = (q[j].1, q[j].0);
        }
    }
    let mut out = Vec::with_capacity(n as usize);
    out.extend(q.iter().map(|&(c, s)| (c, s)));
    out.extend(q.iter().map(|&(c, s)| (-s, c)));
    out.extend(q.iter().map(|&(c, s)| (-c, -s)));
    out.extend(q.iter().map(|&(c, s)| (s, -c)));
    out
}

struct Cell {
    xl: f64,
    xr: f64,
    /// Half pitch, shared by all cells so that shared-edge points are bit-identical
    /// (`xr - xl` can differ in the last bit between cells).
    hw: f64,
    h: f64,
}

impl Cell {
    fn width(&self) -> f64 {
        self.xr - self.xl
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width() + self.h)
    }

    /// CCW corners starting at bottom-left, with their perimeter parameters.
    fn corners(&self) -> [([f64; 2], f64); 4] {
        let w = self.width();
        [
            ([self.xl, 0.0], 0.0),
            ([self.xr, 0.0], w),
            ([self.xr, self.h], w + self.h),
            ([self.xl, self.h], 2.0 * w + self.h),
        ]
    }

    /// Where the ray from `(cx, cy)` in direction `(c, s)` leaves the cell, and the CCW
    /// perimeter parameter of that point. Hits within 1 µm of a corner snap to it.
    fn hit(&self, cx: f64, cy: f64, c: f64, s: f64) -> ([f64; 2], f64) {
        let hw = self.hw;
        let tv = hw / c.abs();
        let th = if s > 0.0 { (self.h - cy) / s } else { cy / -s };
        let p = if tv <= th {
            let x = if c > 0.0 { self.xr } else { self.xl };
            [x, cy + hw * (s / c.abs())]
        } else {
            let y = if s > 0.0 { self.h } else { 0.0 };
            [cx + c * th, y]
        };
        for (corner, param) in self.corners() {
            if (p[0] - corner[0]).abs() < 1e-6 && (p[1] - corner[1]).abs() < 1e-6 {
                return (corner, param);
            }
        }
        (p, self.param(p))
    }

    fn param(&self, p: [f64; 2]) -> f64 {
        let w = self.width();
        if p[1] == 0.0 {
            p[0] - self.xl
        } else if p[0] == self.xr {
            w + p[1]
        } else if p[1] == self.h {
            w + self.h + (self.xr - p[0])
        } else {
            2.0 * w + self.h + (self.h - p[1])
        }
    }

    /// Corners strictly between parameters `a` and `b` going CCW (with wrap-around).
    fn corners_between(&self, a: f64, b: f64) -> Vec<[f64; 2]> {
        let per = self.perimeter();
        let span = (b - a).rem_euclid(per);
        let mut found: Vec<(f64, [f64; 2])> = self
            .corners()
            .iter()
            .filter_map(|&(p, t)| {
                let d = (t - a).rem_euclid(per);
                (d > 0.0 && d < span).then_some((d, p))
            })
            .collect();
        found.sort_by(|x, y| x.0.total_cmp(&y.0));
        found.into_iter().map(|(_, p)| p).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stl::tests::{assert_no_degenerate, assert_watertight};
    use crate::stl::{parse_binary_stl, raw_normal, unit_normal};

    #[test]
    fn steps_and_sizes() {
        let p = CouponParams::default();
        assert_eq!(p.clearances(), vec![-0.1, 0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        let sizes = p.feature_sizes();
        assert!((sizes[0] - 4.9).abs() < 1e-9 && (sizes[6] - 5.5).abs() < 1e-9);
        let pegs = CouponParams {
            kind: CouponKind::Pegs,
            ..p
        };
        assert!((pegs.feature_sizes()[6] - 4.5).abs() < 1e-9);
        assert_eq!(p.effective_segments(), 48);
        assert_eq!(CouponParams { segments: 30, ..p }.effective_segments(), 32);
        assert_eq!(CouponParams { segments: 3, ..p }.effective_segments(), 12);
    }

    #[test]
    fn direction_symmetry() {
        for n in [12, 16, 20, 48, 64] {
            let d = directions(n);
            assert_eq!(d.len(), n as usize);
            for &(c, s) in &d {
                assert!(d.contains(&(-c, s)), "x-mirror missing for n={n}");
                assert!(d.contains(&(c, -s)), "y-mirror missing for n={n}");
                assert!(((c * c + s * s) - 1.0).abs() < 1e-12);
            }
            // CCW order.
            for k in 0..d.len() {
                let (a, b) = (d[k], d[(k + 1) % d.len()]);
                assert!(a.0 * b.1 - a.1 * b.0 > 0.0);
            }
        }
    }

    fn check_mesh(p: &CouponParams) -> Mesh {
        let m = p.mesh().unwrap();
        assert_watertight(&m);
        assert_no_degenerate(&m);
        assert!(m.volume() > 0.0);
        m
    }

    #[test]
    fn hole_coupon_is_watertight_and_sane() {
        let p = CouponParams::default();
        let m = check_mesh(&p);
        let l = p.layout();
        let (lo, hi) = m.bounds().unwrap();
        assert_eq!(lo, [0.0, 0.0, 0.0]);
        assert!((hi[0] - l.width).abs() < 1e-9 && (hi[1] - l.height).abs() < 1e-9);
        assert!((hi[2] - (p.plate_thickness_mm + MARK_HEIGHT)).abs() < 1e-9);
        // Reasonable triangle count for 7 holes × 48 segments.
        assert!(
            m.len() > 7 * 48 * 4 && m.len() < 7 * 48 * 8 + 2000,
            "{}",
            m.len()
        );
        // Plate volume ≈ box − holes (plus marker boxes).
        let holes: f64 = p
            .feature_sizes()
            .iter()
            .map(|d| {
                let n = p.effective_segments() as f64;
                // Area of the inscribed polygon.
                0.5 * n * (d / 2.0).powi(2) * (std::f64::consts::TAU / n).sin()
            })
            .sum::<f64>()
            * p.plate_thickness_mm;
        let plate = l.width * l.height * p.plate_thickness_mm - holes;
        assert!(m.volume() > plate && m.volume() < plate + 50.0);
    }

    #[test]
    fn top_and_bottom_faces_point_out() {
        let p = CouponParams::default();
        let m = check_mesh(&p);
        let t = p.plate_thickness_mm;
        let mut up = 0;
        for tri in &m.triangles {
            let n = unit_normal(tri);
            if tri.iter().all(|v| v[2] == t) {
                assert!(n[2] > 0.999, "top-face triangle not facing +z");
                up += 1;
            }
            if tri.iter().all(|v| v[2] == 0.0) {
                assert!(n[2] < -0.999, "bottom-face triangle not facing -z");
            }
        }
        assert!(up > 0);
    }

    #[test]
    fn peg_coupon_is_watertight() {
        let p = CouponParams {
            kind: CouponKind::Pegs,
            nominal_mm: 6.0,
            start_mm: 0.0,
            end_mm: 0.4,
            step_mm: 0.05,
            segments: 32,
            ..CouponParams::default()
        };
        let m = check_mesh(&p);
        assert_eq!(p.clearances().len(), 9);
        let (_, hi) = m.bounds().unwrap();
        assert!((hi[2] - (p.plate_thickness_mm + p.peg_height_mm)).abs() < 1e-9);
    }

    #[test]
    fn many_shapes_stay_watertight() {
        for &(nom, wall, seg, kind) in &[
            (1.0, 1.0, 12, CouponKind::Holes),
            (2.0, 1.2, 13, CouponKind::Holes),
            (12.0, 3.0, 64, CouponKind::Holes),
            (25.0, 1.0, 100, CouponKind::Pegs),
            (3.0, 4.0, 20, CouponKind::Pegs),
        ] {
            let p = CouponParams {
                kind,
                nominal_mm: nom,
                wall_mm: wall,
                segments: seg,
                start_mm: 0.0,
                end_mm: 0.3,
                step_mm: 0.1,
                ..CouponParams::default()
            };
            check_mesh(&p);
        }
    }

    #[test]
    fn stl_bytes_and_normals() {
        let p = CouponParams {
            segments: 24,
            ..CouponParams::default()
        };
        let m = p.mesh().unwrap();
        let bytes = p.stl().unwrap();
        assert_eq!(bytes.len(), 84 + 50 * m.len());
        assert_eq!(
            u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize,
            m.len()
        );
        assert!(bytes.starts_with(b"print-fit coupon holes"));
        let parsed = parse_binary_stl(&bytes).unwrap();
        for ((n, _), tri) in parsed.iter().zip(&m.triangles) {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
            let r = raw_normal(tri);
            let rl = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            let dot =
                (f64::from(n[0]) * r[0] + f64::from(n[1]) * r[1] + f64::from(n[2]) * r[2]) / rl;
            assert!(dot > 0.9999, "stored normal not parallel to winding normal");
        }
    }

    #[test]
    fn markers_count_tally() {
        let p = CouponParams {
            start_mm: 0.0,
            end_mm: 1.1,
            step_mm: 0.1,
            ..CouponParams::default()
        };
        let l = p.layout();
        assert_eq!(p.markers(&l, 0).len(), 1); // 1 dot
        assert_eq!(p.markers(&l, 4).len(), 1); // 1 bar
        assert_eq!(p.markers(&l, 6).len(), 3); // bar + 2 dots
        assert_eq!(p.markers(&l, 11).len(), 4); // 2 bars + 2 dots
        for i in 0..12 {
            for r in p.markers(&l, i) {
                assert!(r[0] > l.xs[i] && r[2] < l.xs[i + 1]);
                assert!(r[1] > 0.0 && r[3] < l.band);
            }
        }
    }

    #[test]
    fn invalid_params() {
        let base = CouponParams::default();
        assert!(CouponParams {
            step_mm: 0.0,
            ..base
        }
        .mesh()
        .is_err());
        assert!(CouponParams {
            end_mm: -0.5,
            ..base
        }
        .mesh()
        .is_err());
        assert!(CouponParams {
            step_mm: 0.01,
            ..base
        }
        .mesh()
        .is_err());
        assert!(CouponParams {
            nominal_mm: f64::NAN,
            ..base
        }
        .mesh()
        .is_err());
    }
}
