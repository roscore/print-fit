//! Korean / English strings. Keys match `data-i18n` attributes in `index.html`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ko,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
        }
    }

    pub fn from_code(code: &str) -> Lang {
        if code.to_ascii_lowercase().starts_with("ko") {
            Lang::Ko
        } else {
            Lang::En
        }
    }

    pub fn other(self) -> Lang {
        match self {
            Lang::En => Lang::Ko,
            Lang::Ko => Lang::En,
        }
    }
}

/// (key, English, Korean)
const T: &[(&str, &str, &str)] = &[
    ("lang_button", "English", "한국어"),
    ("tagline", "Design parts that fit after 3D printing (FDM & resin).", "3D 프린팅 후에도 잘 맞는 부품 설계를 돕는 도구 (FDM·레진)."),
    ("disclaimer", "All values are starting points. Test on your own printer and material.", "모든 값은 출발점일 뿐입니다. 반드시 본인의 프린터와 재료로 테스트하세요."),
    ("profile_title", "Printer profile", "프린터 프로파일"),
    ("profile_active", "Active profile", "사용 중인 프로파일"),
    ("profile_name", "Name", "이름"),
    ("process", "Process preset", "공정 프리셋"),
    ("proc_fdm04", "FDM, 0.4 mm nozzle", "FDM, 0.4 mm 노즐"),
    ("proc_fdm06", "FDM, 0.6 mm nozzle", "FDM, 0.6 mm 노즐"),
    ("proc_resin", "Resin (MSLA)", "레진 (MSLA)"),
    ("pf_hole", "XY hole shrink (mm, diametral)", "XY 구멍 수축 보정 (mm, 직경 기준)"),
    ("pf_pin", "Pin/tab growth (mm, diametral)", "핀·탭 팽창 (mm, 직경 기준)"),
    ("pf_shrink", "Material shrink (%)", "재료 수축률 (%)"),
    ("pf_ef", "Elephant foot (mm per side, first layers)", "엘리펀트 풋 (한쪽 mm, 첫 레이어)"),
    ("pf_droop", "Horizontal-hole droop (mm)", "수평 구멍 처짐 (mm)"),
    ("pf_new", "Duplicate as new", "복제해서 새로 만들기"),
    ("pf_reset", "Reset to preset", "프리셋 값으로 초기화"),
    ("pf_delete", "Delete", "삭제"),
    ("pf_export", "Export JSON", "JSON 내보내기"),
    ("pf_import", "Import JSON", "JSON 가져오기"),
    ("pf_share", "Share link", "공유 링크"),
    ("pf_privacy", "Profiles are stored only in this browser (localStorage). Share links keep the settings in the URL fragment (#p=…), which browsers never send to a server.", "프로파일은 이 브라우저(localStorage)에만 저장됩니다. 공유 링크는 설정을 URL 프래그먼트(#p=…)에 담으며, 브라우저는 이 부분을 서버로 보내지 않습니다."),
    ("tab_fit", "Fit", "끼워맞춤"),
    ("tab_calib", "Calibrate", "보정"),
    ("tab_coupon", "Test coupon", "테스트 쿠폰"),
    ("tab_screws", "Screws", "나사"),
    ("fit_title", "Fit calculator", "끼워맞춤 계산기"),
    ("fit_nominal", "Nominal diameter / width (mm)", "공칭 직경 / 폭 (mm)"),
    ("fit_feature", "Feature", "형상"),
    ("feat_round", "Round hole + pin/shaft", "원형 구멍 + 핀/축"),
    ("feat_slot", "Square slot + tab", "사각 슬롯 + 탭"),
    ("feat_round_s", "round hole/shaft", "원형 구멍·축"),
    ("feat_slot_s", "slot/tab", "사각 슬롯·탭"),
    ("fit_class", "Fit class", "끼워맞춤 등급"),
    ("cls_press", "Press", "압입 (press)"),
    ("cls_snug", "Snug", "빡빡함 (snug)"),
    ("cls_slide", "Slide", "슬라이드 (slide)"),
    ("cls_loose", "Loose", "헐거움 (loose)"),
    ("cls_free_spin", "Free spin", "자유 회전 (free spin)"),
    ("fit_datum", "Which parts are printed?", "어느 쪽을 출력하나요?"),
    ("datum_split", "Both printed (split clearance)", "둘 다 출력 (간극 반반)"),
    ("datum_shaft_fixed", "Shaft is a bought part (e.g. steel pin)", "축은 기성품 (예: 강철 핀)"),
    ("datum_hole_fixed", "Hole is a bought part (e.g. bearing bore)", "구멍은 기성품 (예: 베어링 내경)"),
    ("fit_facet", "Faceted hole correction (exported as N-sided polygon)", "다각형 구멍 보정 (N각형으로 내보낼 때)"),
    ("fit_segments", "Polygon segments N", "다각형 분할 수 N"),
    ("r_cad_hole", "CAD hole", "CAD 구멍"),
    ("r_cad_shaft", "CAD shaft", "CAD 축"),
    ("r_cad_slot", "CAD slot", "CAD 슬롯"),
    ("r_cad_tab", "CAD tab", "CAD 탭"),
    ("r_faceted", "CAD hole as {n}-gon (circumscribed)", "{n}각형 CAD 구멍 (외접 직경)"),
    ("r_clear_d", "Diametral clearance (printed)", "직경 간극 (출력 후)"),
    ("r_clear_r", "Radial clearance per side", "반경 간극 (한쪽)"),
    ("r_printed_hole", "Expected printed hole", "예상 출력 구멍"),
    ("r_printed_shaft", "Expected printed shaft", "예상 출력 축"),
    ("r_bought", "bought part — not compensated", "기성품 — 보정 안 함"),
    ("r_range", "range", "범위"),
    ("d_hole", "hole", "구멍"),
    ("d_shaft", "shaft", "축"),
    ("d_slot", "slot", "슬롯"),
    ("d_tab", "tab", "탭"),
    ("d_gap", "gap/side", "한쪽 간극"),
    ("d_interference", "interference/side", "한쪽 간섭"),
    ("d_exaggerated", "gap exaggerated", "간극 과장 표시"),
    ("exp_split", "For a {class} fit on a {d} mm {feature}, aim for {c} mm diametral clearance ({r} mm per side) after printing. This profile prints holes {hs} mm small and pins {pg} mm large (material shrink {s} %), so model the hole at {hole} mm and the shaft at {shaft} mm.", "{d} mm {feature}의 {class} 끼워맞춤은 출력 후 직경 간극 {c} mm(한쪽 {r} mm)를 목표로 합니다. 이 프로파일은 구멍이 {hs} mm 작게, 핀이 {pg} mm 크게 출력되므로(재료 수축 {s} %) CAD에서 구멍은 {hole} mm, 축은 {shaft} mm로 모델링하세요."),
    ("exp_shaft_fixed", "For a {class} fit on a {d} mm bought shaft/tab, the printed hole takes all {c} mm of clearance ({r} mm per side). This profile prints holes {hs} mm small (material shrink {s} %), so model the hole at {hole} mm.", "{d} mm 기성품 축·탭에 대한 {class} 끼워맞춤은 출력하는 구멍이 간극 {c} mm(한쪽 {r} mm)를 모두 가집니다. 이 프로파일은 구멍이 {hs} mm 작게 출력되므로(재료 수축 {s} %) CAD 구멍을 {hole} mm로 모델링하세요."),
    ("exp_hole_fixed", "For a {class} fit in a {d} mm bought hole, the printed shaft takes all {c} mm of clearance ({r} mm per side). This profile prints pins {pg} mm large (material shrink {s} %), so model the shaft at {shaft} mm.", "{d} mm 기성품 구멍에 대한 {class} 끼워맞춤은 출력하는 축이 간극 {c} mm(한쪽 {r} mm)를 모두 가집니다. 이 프로파일은 핀이 {pg} mm 크게 출력되므로(재료 수축 {s} %) CAD 축을 {shaft} mm로 모델링하세요."),
    ("exp_press", "Negative clearance means interference: the parts are pressed together (range {min} … {max} mm).", "음수 간극은 간섭을 뜻합니다: 부품을 눌러서 끼웁니다 (범위 {min} … {max} mm)."),
    ("note_ef", "First layers: a hole or slot touching the bed comes out about {ef2} mm smaller (≈ {first} mm) because of elephant foot. Add a {ef} mm chamfer at the bottom edge or use the slicer's elephant-foot compensation.", "첫 레이어: 베드에 닿는 구멍·슬롯은 엘리펀트 풋 때문에 약 {ef2} mm 작게(≈ {first} mm) 나옵니다. 아래 모서리에 {ef} mm 챔퍼를 주거나 슬라이서의 엘리펀트 풋 보정을 쓰세요."),
    ("note_droop", "Horizontal holes (axis parallel to the bed): the top sags about {droop} mm. Use a teardrop or flat-top hole, or enlarge the vertical size by that amount.", "수평 구멍(축이 베드와 평행): 윗부분이 약 {droop} mm 처집니다. 물방울(teardrop)·평평한 윗면 구멍을 쓰거나 세로 방향을 그만큼 키우세요."),
    ("note_facet", "Exported as a {n}-gon the flats would measure only {flat} mm. Model the hole at {fac} mm (circumscribed) so the flats land on {hole} mm.", "{n}각형으로 내보내면 평면 간 거리는 {flat} mm밖에 안 됩니다. 구멍을 {fac} mm(외접)로 모델링해야 평면 간 거리가 {hole} mm가 됩니다."),
    ("note_slot", "Square slots: the nozzle/resin rounds inside corners. Add corner relief (dog-bone) or a small chamfer on the tab edges.", "사각 슬롯: 노즐·레진 특성상 안쪽 모서리가 둥글게 나옵니다. 모서리 릴리프(dog-bone)를 넣거나 탭 모서리에 작은 챔퍼를 주세요."),
    ("note_resin", "Resin: clearances depend strongly on exposure time and post-cure. Re-test after changing resin or exposure.", "레진: 간극은 노광 시간과 후경화에 크게 좌우됩니다. 레진이나 노광을 바꾸면 다시 테스트하세요."),
    ("note_test", "These are starting points. Print the test coupon and adjust the profile.", "이 값은 출발점입니다. 테스트 쿠폰을 출력해서 프로파일을 조정하세요."),
    ("cal_title", "Calibration from a test print", "테스트 출력으로 보정하기"),
    ("cal_help", "Print a few holes and pins of known CAD size, measure them with calipers and enter one pair per line: CAD size, measured size.", "CAD 크기를 아는 구멍과 핀을 몇 개 출력해 캘리퍼스로 재고, 한 줄에 한 쌍씩 입력하세요: CAD 크기, 측정 크기."),
    ("cal_holes", "Holes (CAD measured)", "구멍 (CAD 측정값)"),
    ("cal_pins", "Pins (CAD measured)", "핀 (CAD 측정값)"),
    ("cal_example", "Fill example data", "예시 데이터 채우기"),
    ("cal_apply", "Apply to profile", "프로파일에 적용"),
    ("cal_need_data", "Enter at least one hole or pin measurement.", "구멍 또는 핀 측정값을 하나 이상 입력하세요."),
    ("cal_bad_lines", "Ignored lines (need two positive numbers): {lines}", "무시된 줄 (양수 두 개가 필요): {lines}"),
    ("cal_holes_h", "Holes", "구멍"),
    ("cal_pins_h", "Pins", "핀"),
    ("cal_model", "measured = {a} × CAD {b} mm, RMS residual {rms} mm (n = {n})", "측정값 = {a} × CAD {b} mm, 잔차 RMS {rms} mm (n = {n})"),
    ("cal_slope_fixed", "Slope fixed to 1 (needs ≥ 2 different CAD sizes).", "기울기를 1로 고정 (서로 다른 CAD 크기 2개 이상 필요)."),
    ("col_cad", "CAD", "CAD"),
    ("col_measured", "Measured", "측정"),
    ("col_pred", "Model", "모델"),
    ("col_resid", "Residual", "잔차"),
    ("cal_derived", "Derived compensation", "도출된 보정값"),
    ("cal_suspicious", "Derived shrink is larger than 3 % — check for typos or swapped columns.", "도출된 수축률이 3 %를 넘습니다 — 오타나 열이 뒤바뀌지 않았는지 확인하세요."),
    ("cal_applied", "Applied to profile “{name}”.", "프로파일 “{name}”에 적용했습니다."),
    ("cp_title", "Tolerance test coupon (STL)", "공차 테스트 쿠폰 (STL)"),
    ("cp_help", "A plate with a row of holes (or pegs) stepping the clearance. Each step is marked by bumps: a bar counts 5, a dot counts 1. Sizes are raw CAD sizes, not compensated.", "간극을 단계적으로 바꾼 구멍(또는 핀) 한 줄이 있는 판입니다. 각 단계는 돌기로 표시됩니다: 막대 = 5, 점 = 1. 크기는 보정하지 않은 CAD 원래 크기입니다."),
    ("cp_kind", "Coupon type", "쿠폰 종류"),
    ("cp_holes", "Holes (test with a pin of nominal size)", "구멍 (공칭 크기 핀으로 테스트)"),
    ("cp_pegs", "Pegs (test with a hole of nominal size)", "핀 (공칭 크기 구멍으로 테스트)"),
    ("cp_nominal", "Nominal (mm)", "공칭 (mm)"),
    ("cp_start", "Clearance from (mm)", "간극 시작 (mm)"),
    ("cp_end", "Clearance to (mm)", "간극 끝 (mm)"),
    ("cp_step", "Step (mm)", "간격 (mm)"),
    ("cp_thick", "Plate thickness (mm)", "판 두께 (mm)"),
    ("cp_peg_h", "Peg height (mm)", "핀 높이 (mm)"),
    ("cp_wall", "Wall between features (mm)", "형상 사이 벽 (mm)"),
    ("cp_seg", "Segments per circle", "원 분할 수"),
    ("cp_download", "Download STL", "STL 다운로드"),
    ("cp_info", "{n} steps · {tris} triangles · {size} KB · plate {w} × {h} mm", "{n}단계 · 삼각형 {tris}개 · {size} KB · 판 {w} × {h} mm"),
    ("cp_step_h", "Step", "단계"),
    ("cp_clear_h", "Clearance", "간극"),
    ("cp_size_h", "CAD Ø", "CAD Ø"),
    ("sc_title", "Metric screw cheat sheet", "미터 나사 치트시트"),
    ("sc_warn", "Typical values only — starting points to test. Heat-set insert holes vary by insert brand: check the datasheet.", "일반적인 값일 뿐입니다 — 테스트용 출발점입니다. 열압입 인서트 구멍은 제조사마다 다르니 데이터시트를 확인하세요."),
    ("sc_note", "Self-tap, clearance and insert holes are CAD sizes; add your hole compensation if needed. Nut trap sizes already include the profile's hole compensation plus a small clearance.", "셀프태핑·관통·인서트 구멍은 CAD 크기이며 필요하면 구멍 보정을 더하세요. 너트 트랩 크기에는 프로파일의 구멍 보정과 약간의 간극이 이미 포함되어 있습니다."),
    ("sc_size", "Size", "규격"),
    ("sc_tap", "Self-tap hole", "셀프태핑 구멍"),
    ("sc_clear", "Clearance hole", "관통 구멍"),
    ("sc_insert", "Heat-set insert hole", "열압입 인서트 구멍"),
    ("sc_nut_af", "Nut across flats", "너트 맞변 거리"),
    ("sc_trap_af", "Nut trap AF (CAD)", "너트 트랩 맞변 (CAD)"),
    ("sc_trap_ac", "Nut trap across corners", "너트 트랩 대각"),
    ("sc_nut_t", "Nut thickness", "너트 두께"),
    ("footer", "Open source (MIT). Runs 100% in your browser: no tracking, no server, no network requests.", "오픈 소스 (MIT). 100% 브라우저에서 실행: 추적 없음, 서버 없음, 네트워크 요청 없음."),
    ("msg_saved", "Saved.", "저장했습니다."),
    ("msg_new", "Created a copy.", "복제본을 만들었습니다."),
    ("msg_deleted", "Deleted.", "삭제했습니다."),
    ("msg_last", "Cannot delete the last profile.", "마지막 프로파일은 삭제할 수 없습니다."),
    ("msg_reset", "Reset to preset values.", "프리셋 값으로 초기화했습니다."),
    ("msg_invalid", "Invalid value: {err}", "잘못된 값: {err}"),
    ("msg_imported", "Imported {n} profile(s).", "프로파일 {n}개를 가져왔습니다."),
    ("msg_import_err", "Import failed: {err}", "가져오기 실패: {err}"),
    ("msg_shared", "Loaded shared profile “{name}”.", "공유된 프로파일 “{name}”을(를) 불러왔습니다."),
    ("msg_share_bad", "Could not read the shared link: {err}", "공유 링크를 읽을 수 없습니다: {err}"),
    ("msg_share_ready", "Link ready (copied if your browser allows). The settings live after # and are not sent to any server.", "링크가 준비되었습니다 (브라우저가 허용하면 복사됨). 설정은 # 뒤에 있어 서버로 전송되지 않습니다."),
    ("msg_no_storage", "localStorage unavailable: changes last only for this tab.", "localStorage를 쓸 수 없어 변경 사항은 이 탭에서만 유지됩니다."),
    ("shared_suffix", "(shared)", "(공유됨)"),
    ("copy_suffix", "copy", "복사본"),
];

pub fn tr(lang: Lang, key: &str) -> &'static str {
    match T.iter().find(|(k, _, _)| *k == key) {
        Some((_, en, ko)) => match lang {
            Lang::En => en,
            Lang::Ko => ko,
        },
        None => "??",
    }
}

/// `tr` + replace `{name}` placeholders.
pub fn trf(lang: Lang, key: &str, vars: &[(&str, String)]) -> String {
    let mut s = tr(lang, key).to_string();
    for (k, v) in vars {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_unique_and_both_languages_present() {
        for (i, (k, en, ko)) in T.iter().enumerate() {
            assert!(!en.is_empty() && !ko.is_empty(), "{k}");
            assert!(T[i + 1..].iter().all(|(k2, _, _)| k2 != k), "duplicate {k}");
            // Placeholders must match between languages.
            let ph = |s: &str| {
                let mut v: Vec<String> = s
                    .split('{')
                    .skip(1)
                    .filter_map(|p| p.split('}').next().map(str::to_string))
                    .collect();
                v.sort();
                v
            };
            assert_eq!(ph(en), ph(ko), "placeholder mismatch in {k}");
        }
    }

    #[test]
    fn html_keys_exist() {
        let html = include_str!("../../../index.html");
        for part in html.split("data-i18n=\"").skip(1) {
            let key = part.split('"').next().unwrap();
            assert_ne!(tr(Lang::En, key), "??", "missing key {key}");
        }
    }
}
