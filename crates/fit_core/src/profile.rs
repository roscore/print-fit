//! Printer profiles: per-printer compensation values, JSON (serde) and URL-fragment sharing.

use serde::{Deserialize, Serialize};

use crate::base64;

/// Printing process preset. Selects the default clearance table and default compensation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Process {
    /// FDM, 0.4 mm nozzle.
    #[default]
    Fdm04,
    /// FDM, 0.6 mm nozzle.
    Fdm06,
    /// Resin (MSLA / LCD).
    ResinMsla,
}

impl Process {
    pub const ALL: [Process; 3] = [Process::Fdm04, Process::Fdm06, Process::ResinMsla];

    /// Stable identifier used in JSON and HTML `<select>` values.
    pub fn id(self) -> &'static str {
        match self {
            Process::Fdm04 => "fdm04",
            Process::Fdm06 => "fdm06",
            Process::ResinMsla => "resin_msla",
        }
    }

    pub fn from_id(id: &str) -> Option<Process> {
        Process::ALL.into_iter().find(|p| p.id() == id)
    }

    pub fn is_resin(self) -> bool {
        matches!(self, Process::ResinMsla)
    }
}

/// One printer/material profile. All lengths in mm.
///
/// Compensation model (applied in [`crate::fit::compute_fit`]):
///
/// ```text
/// printed_hole = cad_hole * (1 - shrink/100) - hole_shrink_mm
/// printed_pin  = cad_pin  * (1 - shrink/100) + pin_grow_mm
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub name: String,
    pub process: Process,
    /// Diametral amount holes/slots print *smaller* than modelled (XY hole shrink offset).
    pub hole_shrink_mm: f64,
    /// Diametral amount pins/tabs print *larger* than modelled.
    pub pin_grow_mm: f64,
    /// Uniform material shrink in percent (positive = part comes out smaller).
    pub material_shrink_pct: f64,
    /// Elephant-foot bulge per side on the first layers (informational; only first layers).
    pub elephant_foot_mm: f64,
    /// Typical sag of the top of a horizontal hole (informational).
    pub droop_mm: f64,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::preset(Process::Fdm04)
    }
}

impl Profile {
    /// Default profile for a process. These are generic starting values, not measurements.
    pub fn preset(process: Process) -> Profile {
        let (name, hole, pin, ef, droop) = match process {
            Process::Fdm04 => ("FDM 0.4 mm", 0.15, 0.05, 0.2, 0.15),
            Process::Fdm06 => ("FDM 0.6 mm", 0.20, 0.08, 0.3, 0.25),
            Process::ResinMsla => ("Resin MSLA", 0.06, 0.03, 0.1, 0.0),
        };
        Profile {
            name: name.to_string(),
            process,
            hole_shrink_mm: hole,
            pin_grow_mm: pin,
            material_shrink_pct: 0.0,
            elephant_foot_mm: ef,
            droop_mm: droop,
        }
    }

    /// Scale factor `1 - shrink/100` applied by the material shrink.
    pub fn scale(&self) -> f64 {
        1.0 - self.material_shrink_pct / 100.0
    }

    /// Check values are finite and within sane bounds.
    pub fn validate(&self) -> Result<(), String> {
        let fields = [
            ("hole_shrink_mm", self.hole_shrink_mm, -2.0, 2.0),
            ("pin_grow_mm", self.pin_grow_mm, -2.0, 2.0),
            ("material_shrink_pct", self.material_shrink_pct, -10.0, 10.0),
            ("elephant_foot_mm", self.elephant_foot_mm, 0.0, 2.0),
            ("droop_mm", self.droop_mm, 0.0, 2.0),
        ];
        for (name, v, lo, hi) in fields {
            if !v.is_finite() || v < lo || v > hi {
                return Err(format!("{name} out of range [{lo}, {hi}]: {v}"));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("profile serializes")
    }

    pub fn from_json(text: &str) -> Result<Profile, String> {
        let p: Profile = serde_json::from_str(text).map_err(|e| e.to_string())?;
        p.validate()?;
        Ok(p)
    }

    /// Encode as a URL fragment `p=<base64url(JSON)>` (without the leading `#`).
    ///
    /// URL fragments are never sent to the server by browsers.
    pub fn to_fragment(&self) -> String {
        format!("p={}", base64::encode_url(self.to_json().as_bytes()))
    }

    /// Decode from a fragment, with or without leading `#`, e.g. `#p=eyJ...`.
    pub fn from_fragment(fragment: &str) -> Result<Profile, String> {
        let frag = fragment.strip_prefix('#').unwrap_or(fragment);
        let payload = frag
            .split('&')
            .find_map(|kv| kv.strip_prefix("p="))
            .ok_or_else(|| "no p= parameter".to_string())?;
        let bytes = base64::decode(payload).ok_or_else(|| "invalid base64".to_string())?;
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        Profile::from_json(&text)
    }
}

/// A collection of profiles, used for localStorage and JSON import/export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileSet {
    pub version: u32,
    pub active: usize,
    pub profiles: Vec<Profile>,
}

impl Default for ProfileSet {
    fn default() -> Self {
        ProfileSet {
            version: 1,
            active: 0,
            profiles: Process::ALL.into_iter().map(Profile::preset).collect(),
        }
    }
}

impl ProfileSet {
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("profile set serializes")
    }

    /// Parse either a `ProfileSet` or a single `Profile` JSON document.
    pub fn from_json(text: &str) -> Result<ProfileSet, String> {
        let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let mut set = if value.get("profiles").is_some() {
            serde_json::from_value::<ProfileSet>(value).map_err(|e| e.to_string())?
        } else {
            let p: Profile = serde_json::from_value(value).map_err(|e| e.to_string())?;
            ProfileSet {
                version: 1,
                active: 0,
                profiles: vec![p],
            }
        };
        for p in &set.profiles {
            p.validate()?;
        }
        if set.profiles.is_empty() {
            return Err("no profiles".into());
        }
        if set.active >= set.profiles.len() {
            set.active = 0;
        }
        Ok(set)
    }

    pub fn active_profile(&self) -> &Profile {
        &self.profiles[self.active.min(self.profiles.len() - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom() -> Profile {
        Profile {
            name: "Test printer – 테스트".into(),
            process: Process::ResinMsla,
            hole_shrink_mm: 0.123,
            pin_grow_mm: -0.04,
            material_shrink_pct: 0.35,
            elephant_foot_mm: 0.15,
            droop_mm: 0.05,
        }
    }

    #[test]
    fn json_round_trip() {
        let p = custom();
        let back = Profile::from_json(&p.to_json()).unwrap();
        assert_eq!(back, p);
        assert!(p.to_json().contains("\"resin_msla\""));
    }

    #[test]
    fn missing_fields_use_defaults() {
        let p = Profile::from_json(r#"{"name":"old","hole_shrink_mm":0.3}"#).unwrap();
        assert_eq!(p.name, "old");
        assert_eq!(p.hole_shrink_mm, 0.3);
        assert_eq!(p.process, Process::Fdm04);
        assert_eq!(p.pin_grow_mm, Profile::preset(Process::Fdm04).pin_grow_mm);
    }

    #[test]
    fn url_share_round_trip() {
        let p = custom();
        let frag = p.to_fragment();
        assert!(frag.starts_with("p="));
        assert!(frag
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"=-_".contains(&c)));
        assert_eq!(Profile::from_fragment(&frag).unwrap(), p);
        assert_eq!(Profile::from_fragment(&format!("#{frag}")).unwrap(), p);
        assert_eq!(Profile::from_fragment(&format!("#x=1&{frag}")).unwrap(), p);
    }

    #[test]
    fn bad_fragments_rejected() {
        assert!(Profile::from_fragment("#nothing").is_err());
        assert!(Profile::from_fragment("#p=!!!").is_err());
        let evil = format!("p={}", base64::encode_url(br#"{"hole_shrink_mm":99}"#));
        assert!(Profile::from_fragment(&evil).is_err());
    }

    #[test]
    fn profile_set_import_export() {
        let mut set = ProfileSet::default();
        set.profiles.push(custom());
        set.active = 3;
        let back = ProfileSet::from_json(&set.to_json_pretty()).unwrap();
        assert_eq!(back, set);
        assert_eq!(back.active_profile(), &custom());
        let single = ProfileSet::from_json(&custom().to_json()).unwrap();
        assert_eq!(single.profiles, vec![custom()]);
        assert!(ProfileSet::from_json(r#"{"profiles":[]}"#).is_err());
    }

    #[test]
    fn process_ids() {
        for p in Process::ALL {
            assert_eq!(Process::from_id(p.id()), Some(p));
            Profile::preset(p).validate().unwrap();
        }
    }
}
