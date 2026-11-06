//! Pure HTML/SVG string builders (no DOM access, unit-testable on the host).

use fit_core::calib::{Calibration, LinearFit, Sample};
use fit_core::coupon::{CouponKind, CouponParams};
use fit_core::fit::{Datum, FeatureKind, FitResult};
use fit_core::threads::{nut_trap, SCREWS};
use fit_core::Profile;

use crate::i18n::{tr, trf, Lang};

/// Millimetres with 2–3 decimals (trailing zero of the 3rd decimal trimmed).
pub fn mm(v: f64) -> String {
    let mut s = format!("{v:.3}");
    if s.ends_with('0') {
        s.pop();
    }
    if s == "-0.00" {
        s = "0.00".into();
    }
    s
}

/// Like [`mm`] but with an explicit `+` for positive values.
pub fn mm_signed(v: f64) -> String {
    let s = mm(v);
    if s.starts_with('-') || s == "0.00" {
        s
    } else {
        format!("+{s}")
    }
}

/// Escape text for HTML element content and attribute values.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn row(label: &str, value: &str, big: bool) -> String {
    let cls = if big {
        " class=\"num big\""
    } else {
        " class=\"num\""
    };
    format!("<tr><th>{}</th><td{cls}>{value}</td></tr>", esc(label))
}

/// Result table for the fit calculator.
pub fn fit_table(lang: Lang, feature: FeatureKind, segments: Option<u32>, r: &FitResult) -> String {
    let (hole_key, shaft_key) = match feature {
        FeatureKind::RoundHole => ("r_cad_hole", "r_cad_shaft"),
        FeatureKind::SquareSlot => ("r_cad_slot", "r_cad_tab"),
    };
    let bought = |v: f64, printed: bool| {
        if printed {
            format!("{} mm", mm(v))
        } else {
            format!(
                "{} mm <span class=\"muted small\">({})</span>",
                mm(v),
                esc(tr(lang, "r_bought"))
            )
        }
    };
    let mut html = String::from("<table>");
    html += &row(
        tr(lang, hole_key),
        &bought(r.cad_hole, r.hole_printed),
        true,
    );
    html += &row(
        tr(lang, shaft_key),
        &bought(r.cad_shaft, r.shaft_printed),
        true,
    );
    if let (Some(f), Some(n)) = (r.cad_hole_faceted, segments) {
        html += &row(
            &trf(lang, "r_faceted", &[("n", n.to_string())]),
            &format!("{} mm", mm(f)),
            true,
        );
    }
    let mut clear = format!("{} mm", mm_signed(r.diametral_clearance));
    if r.clearance.min != r.clearance.max {
        clear += &format!(
            " <span class=\"muted small\">({} {} … {})</span>",
            esc(tr(lang, "r_range")),
            mm_signed(r.clearance.min),
            mm_signed(r.clearance.max)
        );
    }
    html += &row(tr(lang, "r_clear_d"), &clear, false);
    html += &row(
        tr(lang, "r_clear_r"),
        &format!("{} mm", mm_signed(r.radial_clearance)),
        false,
    );
    html += &row(
        tr(lang, "r_printed_hole"),
        &format!("{} mm", mm(r.hole_target)),
        false,
    );
    html += &row(
        tr(lang, "r_printed_shaft"),
        &format!("{} mm", mm(r.shaft_target)),
        false,
    );
    html += "</table>";
    html
}

/// Short explanation paragraph.
pub fn fit_explanation(
    lang: Lang,
    profile: &Profile,
    feature: FeatureKind,
    class_label: &str,
    datum: Datum,
    nominal: f64,
    r: &FitResult,
) -> String {
    let feature_s = match feature {
        FeatureKind::RoundHole => tr(lang, "feat_round_s"),
        FeatureKind::SquareSlot => tr(lang, "feat_slot_s"),
    };
    let key = match datum {
        Datum::Split => "exp_split",
        Datum::ShaftFixed => "exp_shaft_fixed",
        Datum::HoleFixed => "exp_hole_fixed",
    };
    let mut s = trf(
        lang,
        key,
        &[
            ("class", class_label.to_lowercase()),
            ("feature", feature_s.to_string()),
            ("d", mm(nominal)),
            ("c", mm_signed(r.diametral_clearance)),
            ("r", mm_signed(r.radial_clearance)),
            ("hole", mm(r.cad_hole)),
            ("shaft", mm(r.cad_shaft)),
            ("hs", mm(profile.hole_shrink_mm)),
            ("pg", mm(profile.pin_grow_mm)),
            ("s", format!("{:.2}", profile.material_shrink_pct)),
        ],
    );
    if r.diametral_clearance < 0.0 {
        s.push(' ');
        s += &trf(
            lang,
            "exp_press",
            &[
                ("min", mm_signed(r.clearance.min)),
                ("max", mm_signed(r.clearance.max)),
            ],
        );
    }
    esc(&s)
}

/// Notes list items (already escaped `<li>` elements).
pub fn fit_notes(
    lang: Lang,
    profile: &Profile,
    feature: FeatureKind,
    segments: Option<u32>,
    r: &FitResult,
) -> String {
    let mut items: Vec<String> = Vec::new();
    if let (Some(fac), Some(n)) = (r.cad_hole_faceted, segments) {
        let flat = fit_core::fit::inscribed_diameter(r.cad_hole, n).unwrap_or(r.cad_hole);
        items.push(trf(
            lang,
            "note_facet",
            &[
                ("n", n.to_string()),
                ("flat", mm(flat)),
                ("fac", mm(fac)),
                ("hole", mm(r.cad_hole)),
            ],
        ));
    }
    if profile.elephant_foot_mm > 0.0 && r.hole_printed {
        items.push(trf(
            lang,
            "note_ef",
            &[
                ("ef2", mm(2.0 * profile.elephant_foot_mm)),
                ("first", mm(r.first_layer_hole)),
                ("ef", mm(profile.elephant_foot_mm)),
            ],
        ));
    }
    if profile.droop_mm > 0.0 && feature == FeatureKind::RoundHole && r.hole_printed {
        items.push(trf(lang, "note_droop", &[("droop", mm(profile.droop_mm))]));
    }
    if feature == FeatureKind::SquareSlot {
        items.push(tr(lang, "note_slot").to_string());
    }
    if profile.process.is_resin() {
        items.push(tr(lang, "note_resin").to_string());
    }
    items.push(tr(lang, "note_test").to_string());
    items
        .iter()
        .map(|s| format!("<li>{}</li>", esc(s)))
        .collect()
}

/// Cross-section of hole vs shaft (or slot vs tab) with the gap exaggerated.
pub fn fit_svg(lang: Lang, feature: FeatureKind, r: &FitResult) -> String {
    let (cx, cy, big) = (120.0, 112.0, 80.0);
    let radial = r.radial_clearance;
    // Exaggerate: 0.1 mm per side ≈ 10 px, clamped so it stays readable.
    let g = (radial.abs() * 100.0).clamp(4.0, 22.0);
    let interference = radial < 0.0;
    let small = if interference { big + g } else { big - g };
    let mut s = String::from(
        "<svg viewBox=\"0 0 240 270\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\">",
    );
    s += "<rect class=\"svg-part\" x=\"14\" y=\"6\" width=\"212\" height=\"212\" rx=\"6\"/>";
    let (hole_name, shaft_name) = match feature {
        FeatureKind::RoundHole => {
            s += &format!("<circle class=\"svg-hole\" cx=\"{cx}\" cy=\"{cy}\" r=\"{big}\"/>");
            s += &format!("<circle class=\"svg-shaft\" cx=\"{cx}\" cy=\"{cy}\" r=\"{small}\"/>");
            if interference {
                s += &format!(
                    "<path class=\"svg-overlap\" fill-rule=\"evenodd\" d=\"M{} {cy}a{small} {small} 0 1 0 {} 0a{small} {small} 0 1 0 -{} 0ZM{} {cy}a{big} {big} 0 1 0 {} 0a{big} {big} 0 1 0 -{} 0Z\"/>",
                    cx - small, 2.0 * small, 2.0 * small, cx - big, 2.0 * big, 2.0 * big
                );
            }
            (tr(lang, "d_hole"), tr(lang, "d_shaft"))
        }
        FeatureKind::SquareSlot => {
            s += &format!(
                "<rect class=\"svg-hole\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                cx - big,
                cy - big,
                2.0 * big,
                2.0 * big
            );
            s += &format!(
                "<rect class=\"svg-shaft\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                cx - small,
                cy - small,
                2.0 * small,
                2.0 * small
            );
            if interference {
                s += &format!(
                    "<path class=\"svg-overlap\" fill-rule=\"evenodd\" d=\"M{} {}h{}v{}h-{}ZM{} {}h{}v{}h-{}Z\"/>",
                    cx - small, cy - small, 2.0 * small, 2.0 * small, 2.0 * small,
                    cx - big, cy - big, 2.0 * big, 2.0 * big, 2.0 * big
                );
            }
            (tr(lang, "d_slot"), tr(lang, "d_tab"))
        }
    };
    // Dimension marker across the gap on the right side.
    let (x0, x1) = (cx + small.min(big), cx + small.max(big));
    s += &format!(
        "<path class=\"svg-dim\" d=\"M{x0} {cy}H{x1}M{x0} {}V{}M{x1} {}V{}\"/>",
        cy - 6.0,
        cy + 6.0,
        cy - 6.0,
        cy + 6.0
    );
    let gap_label = if interference {
        tr(lang, "d_interference")
    } else {
        tr(lang, "d_gap")
    };
    let lines = [
        format!("{} {} mm", hole_name, mm(r.hole_target)),
        format!("{} {} mm", shaft_name, mm(r.shaft_target)),
        format!(
            "{} {} mm ({})",
            gap_label,
            mm(radial.abs()),
            tr(lang, "d_exaggerated")
        ),
    ];
    for (i, l) in lines.iter().enumerate() {
        s += &format!(
            "<text class=\"svg-text\" x=\"120\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            234 + 15 * i,
            esc(l)
        );
    }
    s += "</svg>";
    s
}

/// Top view of the coupon (front edge with the tally marks at the bottom).
pub fn coupon_svg(p: &CouponParams) -> String {
    let l = p.layout();
    let (w, h) = (l.width, l.height);
    let flip = |y: f64| h - y;
    let mut s = format!(
        "<svg viewBox=\"-2 -2 {} {}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\">",
        w + 4.0,
        h + 9.0
    );
    s += &format!("<rect class=\"svg-plate\" x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\"/>");
    let cls = match p.kind {
        CouponKind::Holes => "svg-cphole",
        CouponKind::Pegs => "svg-cppeg",
    };
    for (i, (d, c)) in p.feature_sizes().iter().zip(p.clearances()).enumerate() {
        s += &format!(
            "<circle class=\"{cls}\" cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\"/>",
            l.center_x(i),
            flip(l.cy),
            d / 2.0
        );
        for m in p.markers(&l, i) {
            s += &format!(
                "<rect class=\"svg-mark\" x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\"/>",
                m[0],
                flip(m[3]),
                m[2] - m[0],
                m[3] - m[1]
            );
        }
        s += &format!(
            "<text class=\"svg-text small\" x=\"{:.3}\" y=\"{:.3}\" text-anchor=\"middle\">{}</text>",
            l.center_x(i),
            h + 4.5,
            mm_signed(c)
        );
    }
    s += "</svg>";
    s
}

/// Per-step table for the coupon.
pub fn coupon_table(lang: Lang, p: &CouponParams) -> String {
    let mut s = format!(
        "<div class=\"scroll\"><table><tr><th>{}</th><th class=\"num\">{}</th><th class=\"num\">{}</th></tr>",
        esc(tr(lang, "cp_step_h")),
        esc(tr(lang, "cp_clear_h")),
        esc(tr(lang, "cp_size_h"))
    );
    for (i, (c, d)) in p.clearances().iter().zip(p.feature_sizes()).enumerate() {
        s += &format!(
            "<tr><td>{}</td><td class=\"num\">{} mm</td><td class=\"num\">{} mm</td></tr>",
            i + 1,
            mm_signed(*c),
            mm(d)
        );
    }
    s += "</table></div>";
    s
}

fn fit_block(lang: Lang, title_key: &str, samples: &[Sample], f: &LinearFit) -> String {
    let b = if f.b < 0.0 {
        format!("− {}", mm(-f.b))
    } else {
        format!("+ {}", mm(f.b))
    };
    let mut s = format!("<h3>{}</h3><p>", esc(tr(lang, title_key)));
    s += &esc(&trf(
        lang,
        "cal_model",
        &[
            ("a", format!("{:.4}", f.a)),
            ("b", b),
            ("rms", mm(f.rms)),
            ("n", f.n.to_string()),
        ],
    ));
    if f.slope_fixed {
        s += &format!(
            " <span class=\"muted\">{}</span>",
            esc(tr(lang, "cal_slope_fixed"))
        );
    }
    s += "</p><div class=\"scroll\"><table><tr>";
    for k in ["col_cad", "col_measured", "col_pred", "col_resid"] {
        s += &format!("<th class=\"num\">{}</th>", esc(tr(lang, k)));
    }
    s += "</tr>";
    for (smp, res) in samples.iter().zip(&f.residuals) {
        s += &format!(
            "<tr><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
            mm(smp.cad),
            mm(smp.measured),
            mm(f.predict(smp.cad)),
            mm_signed(*res)
        );
    }
    s += "</table></div>";
    s
}

/// Calibration results.
pub fn calib_html(lang: Lang, holes: &[Sample], pins: &[Sample], c: &Calibration) -> String {
    let mut s = String::new();
    if let Some(f) = &c.holes {
        s += &fit_block(lang, "cal_holes_h", holes, f);
    }
    if let Some(f) = &c.pins {
        s += &fit_block(lang, "cal_pins_h", pins, f);
    }
    s += &format!("<h3>{}</h3><table>", esc(tr(lang, "cal_derived")));
    s += &row(
        tr(lang, "pf_shrink"),
        &format!("{:.3} %", c.material_shrink_pct),
        false,
    );
    if let Some(h) = c.hole_shrink_mm {
        s += &row(tr(lang, "pf_hole"), &format!("{} mm", mm(h)), true);
    }
    if let Some(p) = c.pin_grow_mm {
        s += &row(tr(lang, "pf_pin"), &format!("{} mm", mm(p)), true);
    }
    s += "</table>";
    if c.suspicious {
        s += &format!("<p class=\"warn\">{}</p>", esc(tr(lang, "cal_suspicious")));
    }
    s
}

/// Screw cheat-sheet table.
pub fn screws_html(lang: Lang, profile: &Profile) -> String {
    let cols = [
        "sc_size",
        "sc_tap",
        "sc_clear",
        "sc_insert",
        "sc_nut_af",
        "sc_trap_af",
        "sc_trap_ac",
        "sc_nut_t",
    ];
    let mut s = String::from("<tr>");
    for (i, k) in cols.iter().enumerate() {
        let cls = if i == 0 { "" } else { " class=\"num\"" };
        s += &format!("<th{cls}>{}</th>", esc(tr(lang, k)));
    }
    s += "</tr>";
    for sc in &SCREWS {
        let (af, ac) = nut_trap(profile, sc);
        s += &format!("<tr><th>{}</th>", sc.name);
        for v in [
            sc.self_tap_hole,
            sc.clearance_hole,
            sc.heat_set_hole,
            sc.nut_af,
            af,
            ac,
            sc.nut_thickness,
        ] {
            s += &format!("<td class=\"num\">{}</td>", mm(v));
        }
        s += "</tr>";
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use fit_core::fit::{compute_fit, FitClass, FitInput};

    #[test]
    fn number_format() {
        assert_eq!(mm(8.3), "8.30");
        assert_eq!(mm(-0.025), "-0.025");
        assert_eq!(mm(-0.0001), "0.00");
        assert_eq!(mm_signed(0.1), "+0.10");
        assert_eq!(mm_signed(0.0), "0.00");
        assert_eq!(
            esc("<a href=\"x\">&'"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
        );
    }

    #[test]
    fn renders_without_panicking() {
        let prof = Profile::default();
        for feature in [FeatureKind::RoundHole, FeatureKind::SquareSlot] {
            for class in FitClass::ALL {
                for datum in Datum::ALL {
                    let input = FitInput {
                        nominal_mm: 6.0,
                        feature,
                        class,
                        datum,
                        polygon_segments: Some(24),
                    };
                    let r = compute_fit(&prof, &input).unwrap();
                    for lang in [Lang::En, Lang::Ko] {
                        let svg = fit_svg(lang, feature, &r);
                        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
                        assert!(!fit_table(lang, feature, Some(24), &r).contains("??"));
                        let e = fit_explanation(lang, &prof, feature, "Slide", datum, 6.0, &r);
                        assert!(!e.contains('{'));
                        assert!(!fit_notes(lang, &prof, feature, Some(24), &r).contains('{'));
                    }
                }
            }
        }
        let p = CouponParams::default();
        assert_eq!(coupon_svg(&p).matches("<circle").count(), 7);
        assert!(screws_html(Lang::Ko, &prof).contains("M8"));
    }
}
