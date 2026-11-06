//! `fit_core` — pure-Rust math behind the print-fit web app.
//!
//! * [`fit`]: fit-class clearance table, hole/shaft compensation, polygon (faceted hole) correction.
//! * [`calib`]: least-squares calibration from measured test prints.
//! * [`coupon`]: tolerance test coupon mesh generator.
//! * [`stl`]: triangle mesh + binary STL writer.
//! * [`profile`]: printer profiles (serde JSON) and `#p=` URL-fragment sharing.
//! * [`threads`]: metric screw cheat sheet (M2–M8).
//!
//! All numbers are millimetres unless noted. Every default in this crate is a *starting point*
//! for your own tests, not a guarantee for any particular printer or material.

pub mod base64;
pub mod calib;
pub mod coupon;
pub mod fit;
pub mod profile;
pub mod stl;
pub mod threads;

pub use calib::{derive_compensation, least_squares, Calibration, LinearFit, Sample};
pub use coupon::{CouponKind, CouponParams};
pub use fit::{
    circumscribed_diameter, compute_fit, fit_clearance, Clearance, Datum, FeatureKind, FitClass,
    FitError, FitInput, FitResult,
};
pub use profile::{Process, Profile, ProfileSet};
pub use stl::Mesh;
