//! Triangle mesh and binary STL writer.
//!
//! Binary STL layout: 80-byte header, `u32` triangle count, then per triangle 12 little-endian
//! `f32` (normal, v0, v1, v2) and a `u16` attribute — 50 bytes. Total `84 + 50 * n` bytes.

pub type V3 = [f64; 3];

/// A triangle soup. Triangles are counter-clockwise seen from outside (right-hand normal).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub triangles: Vec<[V3; 3]>,
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Unnormalized normal `(v1 - v0) x (v2 - v0)` (length = 2 × area).
pub fn raw_normal(t: &[V3; 3]) -> V3 {
    cross(sub(t[1], t[0]), sub(t[2], t[0]))
}

/// Unit normal, or zero for a degenerate triangle.
pub fn unit_normal(t: &[V3; 3]) -> V3 {
    let n = raw_normal(t);
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0; 3]
    }
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh::default()
    }

    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub fn tri(&mut self, a: V3, b: V3, c: V3) {
        self.triangles.push([a, b, c]);
    }

    /// Quad `a b c d` (counter-clockwise) as two triangles.
    pub fn quad(&mut self, a: V3, b: V3, c: V3, d: V3) {
        self.tri(a, b, c);
        self.tri(a, c, d);
    }

    /// Closed axis-aligned box.
    pub fn add_box(&mut self, min: V3, max: V3) {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let p = |x, y, z| [x, y, z];
        // bottom (-z) and top (+z)
        self.quad(p(x0, y0, z0), p(x0, y1, z0), p(x1, y1, z0), p(x1, y0, z0));
        self.quad(p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1));
        // -y, +y
        self.quad(p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1));
        self.quad(p(x1, y1, z0), p(x0, y1, z0), p(x0, y1, z1), p(x1, y1, z1));
        // -x, +x
        self.quad(p(x0, y1, z0), p(x0, y0, z0), p(x0, y0, z1), p(x0, y1, z1));
        self.quad(p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1));
    }

    /// Axis-aligned bounding box `(min, max)`; `None` if empty.
    pub fn bounds(&self) -> Option<(V3, V3)> {
        let mut it = self.triangles.iter().flatten();
        let first = *it.next()?;
        Some(it.fold((first, first), |(mut lo, mut hi), v| {
            for i in 0..3 {
                lo[i] = lo[i].min(v[i]);
                hi[i] = hi[i].max(v[i]);
            }
            (lo, hi)
        }))
    }

    /// Signed volume (positive for a closed, outward-oriented mesh).
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let c = cross(t[1], t[2]);
                t[0][0] * c[0] + t[0][1] * c[1] + t[0][2] * c[2]
            })
            .sum::<f64>()
            / 6.0
    }

    /// Serialize to binary STL. The header never starts with `solid` (that would make some
    /// readers mistake it for ASCII STL).
    pub fn to_binary_stl(&self, header: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(84 + 50 * self.triangles.len());
        let mut h = [b' '; 80];
        let text = if header
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("solid")
        {
            format!("binary {header}")
        } else {
            header.to_string()
        };
        for (dst, src) in h.iter_mut().zip(text.bytes().filter(u8::is_ascii)) {
            *dst = src;
        }
        out.extend_from_slice(&h);
        out.extend_from_slice(&(self.triangles.len() as u32).to_le_bytes());
        for t in &self.triangles {
            let n = unit_normal(t);
            for v in std::iter::once(&n).chain(t.iter()) {
                for c in v {
                    out.extend_from_slice(&(*c as f32).to_le_bytes());
                }
            }
            out.extend_from_slice(&0u16.to_le_bytes());
        }
        out
    }
}

/// Parse a binary STL back into `(normal, [v0, v1, v2])` per triangle (used for testing and
/// sanity checks).
pub fn parse_binary_stl(bytes: &[u8]) -> Option<Vec<([f32; 3], [[f32; 3]; 3])>> {
    if bytes.len() < 84 {
        return None;
    }
    let n = u32::from_le_bytes(bytes[80..84].try_into().ok()?) as usize;
    if bytes.len() != 84 + 50 * n {
        return None;
    }
    let f = |off: usize| f32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
    let v = |off: usize| [f(off), f(off + 4), f(off + 8)];
    Some(
        (0..n)
            .map(|i| {
                let o = 84 + 50 * i;
                (v(o), [v(o + 12), v(o + 24), v(o + 36)])
            })
            .collect(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Every directed edge must have exactly one reversed twin: closed, consistently oriented,
    /// no T-junctions. Vertices are compared exactly (bit patterns).
    pub fn assert_watertight(mesh: &Mesh) {
        let key = |v: &V3| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()];
        let mut edges: HashMap<([u64; 3], [u64; 3]), i32> = HashMap::new();
        for t in &mesh.triangles {
            for i in 0..3 {
                let a = key(&t[i]);
                let b = key(&t[(i + 1) % 3]);
                assert_ne!(a, b, "degenerate edge");
                *edges.entry((a, b)).or_default() += 1;
            }
        }
        for (&(a, b), &count) in &edges {
            assert_eq!(count, 1, "duplicated directed edge");
            assert_eq!(edges.get(&(b, a)), Some(&1), "open or flipped edge");
        }
    }

    pub fn assert_no_degenerate(mesh: &Mesh) {
        for t in &mesh.triangles {
            let n = raw_normal(t);
            let area2 = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(area2 > 1e-9, "zero-area triangle {t:?}");
        }
    }

    #[test]
    fn box_is_closed_with_outward_normals() {
        let mut m = Mesh::new();
        m.add_box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        assert_eq!(m.len(), 12);
        assert_watertight(&m);
        assert_no_degenerate(&m);
        assert!((m.volume() - 24.0).abs() < 1e-9);
        // Each normal points away from the centre.
        let c = [1.0, 1.5, 2.0];
        for t in &m.triangles {
            let n = unit_normal(t);
            let d = sub(t[0], c);
            assert!(n[0] * d[0] + n[1] * d[1] + n[2] * d[2] > 0.0);
        }
    }

    #[test]
    fn binary_layout() {
        let mut m = Mesh::new();
        m.add_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let bytes = m.to_binary_stl("solid-looking header");
        assert_eq!(bytes.len(), 84 + 50 * m.len());
        assert!(!bytes.starts_with(b"solid"));
        assert_eq!(u32::from_le_bytes(bytes[80..84].try_into().unwrap()), 12);
        let parsed = parse_binary_stl(&bytes).unwrap();
        for ((n, v), t) in parsed.iter().zip(&m.triangles) {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
            for (pv, tv) in v.iter().zip(t) {
                for i in 0..3 {
                    assert_eq!(pv[i], tv[i] as f32);
                }
            }
        }
        assert!(parse_binary_stl(&bytes[..bytes.len() - 1]).is_none());
        assert_eq!(Mesh::new().to_binary_stl("x").len(), 84);
    }
}
