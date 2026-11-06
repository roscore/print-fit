//! print-fit browser UI: plain `web-sys`, static markup in `index.html`, Rust wires events and
//! renders results. Everything runs locally; the only persistence is optional localStorage.

mod i18n;
mod render;

use std::cell::RefCell;

use fit_core::calib::{derive_compensation, parse_samples, Calibration};
use fit_core::coupon::{CouponKind, CouponParams};
use fit_core::fit::{compute_fit, Datum, FeatureKind, FitClass, FitInput};
use fit_core::{Process, Profile, ProfileSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, BlobPropertyBag, Document, Element, Event, HtmlAnchorElement, HtmlElement,
    HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Storage, Url,
};

use i18n::{tr, trf, Lang};

const KEY_PROFILES: &str = "print-fit.profiles";
const KEY_LANG: &str = "print-fit.lang";

struct App {
    lang: Lang,
    set: ProfileSet,
    calib: Option<Calibration>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|cell| cell.try_borrow_mut().ok()?.as_mut().map(f))
}

// ---------- DOM helpers ----------

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

fn doc() -> Document {
    window().document().expect("document")
}

fn el(id: &str) -> Element {
    doc()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("missing element #{id}"))
}

fn input(id: &str) -> HtmlInputElement {
    el(id).dyn_into().expect("input element")
}

fn select(id: &str) -> HtmlSelectElement {
    el(id).dyn_into().expect("select element")
}

fn textarea(id: &str) -> HtmlTextAreaElement {
    el(id).dyn_into().expect("textarea element")
}

fn html_el(id: &str) -> HtmlElement {
    el(id).dyn_into().expect("html element")
}

fn num(id: &str) -> Option<f64> {
    let v = input(id).value();
    let v = v.trim().replace(',', ".");
    v.parse::<f64>().ok().filter(|x| x.is_finite())
}

fn set_html(id: &str, html: &str) {
    el(id).set_inner_html(html);
}

fn set_msg(id: &str, text: &str, ok: bool) {
    let e = el(id);
    e.set_text_content(Some(text));
    e.set_class_name(if ok { "msg ok" } else { "msg err" });
}

fn on(target: &web_sys::EventTarget, event: &str, f: impl FnMut(Event) + 'static) {
    let cb = Closure::<dyn FnMut(Event)>::new(f);
    target
        .add_event_listener_with_callback(event, cb.as_ref().unchecked_ref())
        .expect("add listener");
    cb.forget();
}

fn on_id(id: &str, event: &str, f: impl FnMut(Event) + 'static) {
    on(&el(id), event, f);
}

fn storage() -> Option<Storage> {
    window().local_storage().ok().flatten()
}

fn store(key: &str, value: &str) -> bool {
    storage().is_some_and(|s| s.set_item(key, value).is_ok())
}

fn load(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

fn download(bytes: &[u8], name: &str, mime: &str) -> Result<(), JsValue> {
    let arr = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&arr);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let a: HtmlAnchorElement = doc().create_element("a")?.dyn_into()?;
    a.set_href(&url);
    a.set_download(name);
    a.set_hidden(true);
    let body = doc().body().ok_or("no body")?;
    body.append_child(&a)?;
    a.click();
    a.remove();
    // Revoke after the browser had a chance to start the download.
    let revoke = Closure::once_into_js(move || {
        let _ = Url::revoke_object_url(&url);
    });
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 10_000)?;
    Ok(())
}

/// Best-effort clipboard write (ignored if unsupported or denied).
fn copy_to_clipboard(text: &str) {
    let nav = window().navigator();
    let Ok(clip) = js_sys::Reflect::get(&nav, &JsValue::from_str("clipboard")) else {
        return;
    };
    if clip.is_undefined() || clip.is_null() {
        return;
    }
    if let Ok(f) = js_sys::Reflect::get(&clip, &JsValue::from_str("writeText")) {
        if let Some(f) = f.dyn_ref::<js_sys::Function>() {
            if let Ok(p) = f.call1(&clip, &JsValue::from_str(text)) {
                if let Ok(p) = p.dyn_into::<js_sys::Promise>() {
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = wasm_bindgen_futures::JsFuture::from(p).await;
                    });
                }
            }
        }
    }
}

// ---------- profile form ----------

fn save_profiles(app: &App) {
    if !store(
        KEY_PROFILES,
        &serde_json::to_string(&app.set).unwrap_or_default(),
    ) {
        set_msg("pf-msg", tr(app.lang, "msg_no_storage"), false);
    }
}

fn fill_profile_select(app: &App) {
    let mut html = String::new();
    for (i, p) in app.set.profiles.iter().enumerate() {
        let sel = if i == app.set.active { " selected" } else { "" };
        html += &format!(
            "<option value=\"{i}\"{sel}>{}</option>",
            render::esc(&p.name)
        );
    }
    set_html("profile-select", &html);
    el("profile-summary").set_text_content(Some(&app.set.active_profile().name));
}

fn load_profile_form(app: &App) {
    let p = app.set.active_profile();
    input("pf-name").set_value(&p.name);
    select("pf-process").set_value(p.process.id());
    input("pf-hole").set_value(&format!("{}", p.hole_shrink_mm));
    input("pf-pin").set_value(&format!("{}", p.pin_grow_mm));
    input("pf-shrink").set_value(&format!("{}", p.material_shrink_pct));
    input("pf-ef").set_value(&format!("{}", p.elephant_foot_mm));
    input("pf-droop").set_value(&format!("{}", p.droop_mm));
}

fn read_profile_form(current: &Profile) -> Result<Profile, String> {
    let field = |id: &str, label: &str| num(id).ok_or_else(|| format!("{label}?"));
    let p = Profile {
        name: {
            let n = input("pf-name").value();
            let n = n.trim();
            if n.is_empty() {
                current.name.clone()
            } else {
                n.chars().take(60).collect()
            }
        },
        process: Process::from_id(&select("pf-process").value()).unwrap_or(current.process),
        hole_shrink_mm: field("pf-hole", "hole")?,
        pin_grow_mm: field("pf-pin", "pin")?,
        material_shrink_pct: field("pf-shrink", "shrink")?,
        elephant_foot_mm: field("pf-ef", "elephant foot")?,
        droop_mm: field("pf-droop", "droop")?,
    };
    p.validate()?;
    Ok(p)
}

fn active_mut(app: &mut App) -> &mut Profile {
    let i = app.set.active.min(app.set.profiles.len() - 1);
    &mut app.set.profiles[i]
}

/// Import a `#p=` fragment (if present), then strip it from the address bar.
fn handle_fragment(app: &mut App) {
    let loc = window().location();
    let hash = loc.hash().unwrap_or_default();
    if !hash.contains("p=") {
        return;
    }
    match Profile::from_fragment(&hash) {
        Ok(mut p) => {
            if let Some(i) = app.set.profiles.iter().position(|x| *x == p) {
                app.set.active = i;
            } else {
                p.name = format!("{} {}", p.name, tr(app.lang, "shared_suffix"));
                app.set.profiles.push(p);
                app.set.active = app.set.profiles.len() - 1;
            }
            save_profiles(app);
            let name = app.set.active_profile().name.clone();
            set_msg(
                "pf-msg",
                &trf(app.lang, "msg_shared", &[("name", name)]),
                true,
            );
        }
        Err(e) => set_msg(
            "pf-msg",
            &trf(app.lang, "msg_share_bad", &[("err", e)]),
            false,
        ),
    }
    if let Ok(history) = window().history() {
        let clean = format!(
            "{}{}",
            loc.pathname().unwrap_or_default(),
            loc.search().unwrap_or_default()
        );
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&clean));
    }
}

// ---------- rendering ----------

fn apply_lang(app: &App) {
    let lang = app.lang;
    if let Some(root) = doc().document_element() {
        let _ = root.set_attribute("lang", lang.code());
    }
    if let Ok(nodes) = doc().query_selector_all("[data-i18n]") {
        for i in 0..nodes.length() {
            if let Some(e) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                if let Some(key) = e.get_attribute("data-i18n") {
                    e.set_text_content(Some(tr(lang, &key)));
                }
            }
        }
    }
    el("lang-toggle").set_text_content(Some(tr(lang.other(), "lang_button")));
}

fn render_fit(app: &App) {
    let lang = app.lang;
    let profile = app.set.active_profile();
    let feature = if select("fit-feature").value() == "slot" {
        FeatureKind::SquareSlot
    } else {
        FeatureKind::RoundHole
    };
    let class = FitClass::from_id(&select("fit-class").value()).unwrap_or(FitClass::Slide);
    let datum = Datum::from_id(&select("fit-datum").value()).unwrap_or(Datum::Split);
    let facet_box = input("fit-facet");
    facet_box.set_disabled(feature != FeatureKind::RoundHole);
    let facet = facet_box.checked() && feature == FeatureKind::RoundHole;
    input("fit-segments").set_disabled(!facet);
    let segments = if facet {
        num("fit-segments").map(|v| v.round().clamp(0.0, 4096.0) as u32)
    } else {
        None
    };
    let nominal = num("fit-nominal").unwrap_or(f64::NAN);
    let fit_in = FitInput {
        nominal_mm: nominal,
        feature,
        class,
        datum,
        polygon_segments: segments,
    };
    match compute_fit(profile, &fit_in) {
        Ok(r) => {
            set_html("fit-out", &render::fit_table(lang, feature, segments, &r));
            set_html("fit-diagram", &render::fit_svg(lang, feature, &r));
            let class_label = tr(lang, &format!("cls_{}", class.id()));
            set_html(
                "fit-explain",
                &render::fit_explanation(lang, profile, feature, class_label, datum, nominal, &r),
            );
            set_html(
                "fit-notes",
                &render::fit_notes(lang, profile, feature, segments, &r),
            );
        }
        Err(e) => {
            set_html(
                "fit-out",
                &format!("<p class=\"msg err\">{}</p>", render::esc(&e.to_string())),
            );
            set_html("fit-diagram", "");
            set_html("fit-explain", "");
            set_html("fit-notes", "");
        }
    }
}

fn render_calib(app: &mut App) {
    let lang = app.lang;
    let (holes, bad_h) = parse_samples(&textarea("cal-holes").value());
    let (pins, bad_p) = parse_samples(&textarea("cal-pins").value());
    let mut html = String::new();
    let bad: Vec<String> = bad_h
        .iter()
        .map(|l| format!("{} {l}", tr(lang, "cal_holes_h")))
        .chain(
            bad_p
                .iter()
                .map(|l| format!("{} {l}", tr(lang, "cal_pins_h"))),
        )
        .collect();
    if !bad.is_empty() {
        html += &format!(
            "<p class=\"warn small\">{}</p>",
            render::esc(&trf(lang, "cal_bad_lines", &[("lines", bad.join(", "))]))
        );
    }
    if holes.is_empty() && pins.is_empty() {
        html += &format!(
            "<p class=\"muted\">{}</p>",
            render::esc(tr(lang, "cal_need_data"))
        );
        app.calib = None;
    } else {
        let c = derive_compensation(&holes, &pins);
        html += &render::calib_html(lang, &holes, &pins, &c);
        app.calib = Some(c);
    }
    set_html("cal-out", &html);
    el("cal-apply")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("button")
        .set_disabled(app.calib.is_none());
}

fn coupon_params() -> Option<CouponParams> {
    Some(CouponParams {
        kind: if select("cp-kind").value() == "pegs" {
            CouponKind::Pegs
        } else {
            CouponKind::Holes
        },
        nominal_mm: num("cp-nominal")?,
        start_mm: num("cp-start")?,
        end_mm: num("cp-end")?,
        step_mm: num("cp-step")?,
        plate_thickness_mm: num("cp-thick")?,
        peg_height_mm: num("cp-peg-h")?,
        wall_mm: num("cp-wall")?,
        segments: num("cp-seg")?.round().clamp(0.0, 4096.0) as u32,
    })
}

fn render_coupon(app: &App) {
    let lang = app.lang;
    let pegs = select("cp-kind").value() == "pegs";
    input("cp-peg-h").set_disabled(!pegs);
    let params = coupon_params().ok_or_else(|| "?".to_string());
    let result = params.and_then(|p| p.validate().map(|_| p));
    let button = el("cp-download")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("button");
    match result {
        Ok(p) => {
            set_html("cp-preview", &render::coupon_svg(&p));
            let l = p.layout();
            let tris = p.mesh().map(|m| m.len()).unwrap_or(0);
            let info = trf(
                lang,
                "cp_info",
                &[
                    ("n", p.clearances().len().to_string()),
                    ("tris", tris.to_string()),
                    ("size", format!("{:.0}", (84 + 50 * tris) as f64 / 1024.0)),
                    ("w", format!("{:.1}", l.width)),
                    ("h", format!("{:.1}", l.height)),
                ],
            );
            set_html(
                "cp-info",
                &format!("{}{}", render::esc(&info), render::coupon_table(lang, &p)),
            );
            button.set_disabled(false);
        }
        Err(e) => {
            set_html("cp-preview", "");
            set_html(
                "cp-info",
                &format!("<span class=\"msg err\">{}</span>", render::esc(&e)),
            );
            button.set_disabled(true);
        }
    }
}

fn render_all(app: &mut App) {
    render_fit(app);
    render_calib(app);
    render_coupon(app);
    set_html(
        "screw-table",
        &render::screws_html(app.lang, app.set.active_profile()),
    );
}

/// Run `f` with the app, then re-render everything.
fn update(f: impl FnOnce(&mut App)) {
    with_app(|app| {
        f(app);
        render_all(app);
    });
}

// ---------- startup ----------

fn wire_events() {
    // Language toggle.
    on_id("lang-toggle", "click", |_| {
        update(|app| {
            app.lang = app.lang.other();
            store(KEY_LANG, app.lang.code());
            apply_lang(app);
        })
    });

    // Tabs.
    if let Ok(tabs) = doc().query_selector_all(".tab") {
        for i in 0..tabs.length() {
            let Some(tab) = tabs.item(i).and_then(|n| n.dyn_into::<Element>().ok()) else {
                continue;
            };
            let name = tab.get_attribute("data-tab").unwrap_or_default();
            on(&tab, "click", move |_| {
                if let Ok(all) = doc().query_selector_all(".tab") {
                    for j in 0..all.length() {
                        if let Some(t) = all.item(j).and_then(|n| n.dyn_into::<Element>().ok()) {
                            let active = t.get_attribute("data-tab").as_deref() == Some(&name);
                            let _ = t.class_list().toggle_with_force("active", active);
                            let _ = t.set_attribute(
                                "aria-selected",
                                if active { "true" } else { "false" },
                            );
                        }
                    }
                }
                for sec in ["fit", "calib", "coupon", "screws"] {
                    html_el(&format!("tab-{sec}")).set_hidden(sec != name);
                }
            });
        }
    }

    // Profile editing.
    for id in [
        "pf-name",
        "pf-hole",
        "pf-pin",
        "pf-shrink",
        "pf-ef",
        "pf-droop",
    ] {
        on_id(id, "input", |_| {
            update(|app| match read_profile_form(app.set.active_profile()) {
                Ok(p) => {
                    *active_mut(app) = p;
                    save_profiles(app);
                    fill_profile_select(app);
                    set_msg("pf-msg", "", true);
                }
                Err(e) => set_msg(
                    "pf-msg",
                    &trf(app.lang, "msg_invalid", &[("err", e)]),
                    false,
                ),
            })
        });
    }
    on_id("pf-process", "change", |_| {
        update(|app| {
            if let Some(proc) = Process::from_id(&select("pf-process").value()) {
                let name = app.set.active_profile().name.clone();
                *active_mut(app) = Profile {
                    name,
                    ..Profile::preset(proc)
                };
                save_profiles(app);
                load_profile_form(app);
            }
        })
    });
    on_id("profile-select", "change", |_| {
        update(|app| {
            if let Ok(i) = select("profile-select").value().parse::<usize>() {
                if i < app.set.profiles.len() {
                    app.set.active = i;
                    save_profiles(app);
                    fill_profile_select(app);
                    load_profile_form(app);
                    set_msg("pf-msg", "", true);
                }
            }
        })
    });
    on_id("pf-new", "click", |_| {
        update(|app| {
            let mut p = app.set.active_profile().clone();
            p.name = format!("{} {}", p.name, tr(app.lang, "copy_suffix"));
            app.set.profiles.push(p);
            app.set.active = app.set.profiles.len() - 1;
            save_profiles(app);
            fill_profile_select(app);
            load_profile_form(app);
            set_msg("pf-msg", tr(app.lang, "msg_new"), true);
        })
    });
    on_id("pf-reset", "click", |_| {
        update(|app| {
            let cur = app.set.active_profile().clone();
            *active_mut(app) = Profile {
                name: cur.name,
                ..Profile::preset(cur.process)
            };
            save_profiles(app);
            load_profile_form(app);
            set_msg("pf-msg", tr(app.lang, "msg_reset"), true);
        })
    });
    on_id("pf-delete", "click", |_| {
        update(|app| {
            if app.set.profiles.len() <= 1 {
                set_msg("pf-msg", tr(app.lang, "msg_last"), false);
                return;
            }
            let i = app.set.active.min(app.set.profiles.len() - 1);
            app.set.profiles.remove(i);
            app.set.active = i.min(app.set.profiles.len() - 1);
            save_profiles(app);
            fill_profile_select(app);
            load_profile_form(app);
            set_msg("pf-msg", tr(app.lang, "msg_deleted"), true);
        })
    });
    on_id("pf-export", "click", |_| {
        with_app(|app| {
            let json = app.set.to_json_pretty();
            if let Err(e) = download(
                json.as_bytes(),
                "print-fit-profiles.json",
                "application/json",
            ) {
                set_msg("pf-msg", &format!("{e:?}"), false);
            }
        });
    });
    on_id("pf-import-file", "change", |_| {
        let inp = input("pf-import-file");
        let Some(file) = inp.files().and_then(|f| f.get(0)) else {
            return;
        };
        let promise = file.text();
        inp.set_value("");
        wasm_bindgen_futures::spawn_local(async move {
            let text = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            update(|app| match ProfileSet::from_json(&text) {
                Ok(imported) => {
                    let n = imported.profiles.len();
                    let first = app.set.profiles.len();
                    app.set.profiles.extend(imported.profiles);
                    app.set.active = first;
                    save_profiles(app);
                    fill_profile_select(app);
                    load_profile_form(app);
                    set_msg(
                        "pf-msg",
                        &trf(app.lang, "msg_imported", &[("n", n.to_string())]),
                        true,
                    );
                }
                Err(e) => set_msg(
                    "pf-msg",
                    &trf(app.lang, "msg_import_err", &[("err", e)]),
                    false,
                ),
            });
        });
    });
    on_id("pf-share", "click", |_| {
        with_app(|app| {
            let loc = window().location();
            let url = format!(
                "{}{}{}#{}",
                loc.origin().unwrap_or_default(),
                loc.pathname().unwrap_or_default(),
                loc.search().unwrap_or_default(),
                app.set.active_profile().to_fragment()
            );
            let field = input("share-url");
            field.set_value(&url);
            field.set_hidden(false);
            field.select();
            copy_to_clipboard(&url);
            set_msg("pf-msg", tr(app.lang, "msg_share_ready"), true);
        });
    });
    on(&window(), "hashchange", |_| {
        update(|app| {
            handle_fragment(app);
            fill_profile_select(app);
            load_profile_form(app);
        })
    });

    // Fit calculator.
    for id in ["fit-nominal", "fit-segments"] {
        on_id(id, "input", |_| update(|_| {}));
    }
    for id in ["fit-feature", "fit-class", "fit-datum", "fit-facet"] {
        on_id(id, "change", |_| update(|_| {}));
    }

    // Calibration.
    for id in ["cal-holes", "cal-pins"] {
        on_id(id, "input", |_| update(|_| {}));
    }
    on_id("cal-example", "click", |_| {
        // Synthetic example: holes print ~0.18 mm small, pins ~0.06 mm large, 0.2 % shrink.
        textarea("cal-holes").set_value("4.0 3.81\n6.0 5.81\n8.0 7.80\n10.0 9.80\n12.0 11.79");
        textarea("cal-pins").set_value("4.0 4.05\n6.0 6.05\n8.0 8.04\n10.0 10.04");
        update(|_| {});
    });
    on_id("cal-apply", "click", |_| {
        update(|app| {
            if let Some(c) = app.calib.clone() {
                c.apply_to(active_mut(app));
                save_profiles(app);
                load_profile_form(app);
                let name = app.set.active_profile().name.clone();
                set_msg(
                    "cal-msg",
                    &trf(app.lang, "cal_applied", &[("name", name)]),
                    true,
                );
            }
        })
    });

    // Coupon.
    for id in [
        "cp-nominal",
        "cp-start",
        "cp-end",
        "cp-step",
        "cp-thick",
        "cp-peg-h",
        "cp-wall",
        "cp-seg",
    ] {
        on_id(id, "input", |_| update(|_| {}));
    }
    on_id("cp-kind", "change", |_| update(|_| {}));
    on_id("cp-download", "click", |_| {
        let Some(p) = coupon_params() else { return };
        match p.stl() {
            Ok(bytes) => {
                let kind = match p.kind {
                    CouponKind::Holes => "holes",
                    CouponKind::Pegs => "pegs",
                };
                let name = format!("print-fit-coupon-{kind}-d{}.stl", p.nominal_mm);
                let _ = download(&bytes, &name, "model/stl");
            }
            Err(e) => set_html(
                "cp-info",
                &format!("<span class=\"msg err\">{}</span>", render::esc(&e)),
            ),
        }
    });
}

fn start() -> Result<(), JsValue> {
    let lang = load(KEY_LANG)
        .map(|c| Lang::from_code(&c))
        .unwrap_or_else(|| Lang::from_code(&window().navigator().language().unwrap_or_default()));
    let set = load(KEY_PROFILES)
        .and_then(|s| ProfileSet::from_json(&s).ok())
        .unwrap_or_default();
    APP.with(|cell| {
        *cell.borrow_mut() = Some(App {
            lang,
            set,
            calib: None,
        })
    });
    wire_events();
    with_app(|app| {
        apply_lang(app);
        handle_fragment(app);
        fill_profile_select(app);
        load_profile_form(app);
        render_all(app);
    });
    html_el("loading").set_hidden(true);
    html_el("app").set_hidden(false);
    Ok(())
}

fn main() {
    if let Err(e) = start() {
        if let Some(l) = doc().get_element_by_id("loading") {
            l.set_text_content(Some(&format!("Error: {e:?}")));
        }
    }
}
