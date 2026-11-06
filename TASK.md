# Task: 3D-print fit & tolerance calculator (Rust → WebAssembly, static web app)

Original open-source tool. Write all code yourself; do not copy anyone's code or assets.

## Product
A single-page web app (GitHub Pages, sub-path `/print-fit/`) that helps makers design parts that fit
after 3D printing (FDM and resin/MSLA).

1. **Fit calculator** — inputs: nominal diameter/width (mm), feature kind (round hole + pin/shaft,
   or square slot + tab), fit class (press / snug / slide / loose / free-spin), process preset
   (FDM 0.4 mm nozzle, FDM 0.6 mm, resin MSLA) with editable per-printer compensation:
   XY hole shrink offset (mm), material shrink (%), elephant-foot (mm, first layers only, note),
   horizontal-hole droop note. Output: recommended CAD hole size and CAD shaft size, resulting
   radial/diametral clearance, and a short Korean/English explanation. Show a small SVG/canvas
   cross-section diagram of hole vs shaft with the gap exaggerated and labelled.
   Polygon note: optional "faceted hole" correction — a hole exported as an N-sided polygon prints
   smaller; compute circumscribed diameter for N segments (d / cos(pi/N)).
2. **Calibration from a test print** — user enters, for several printed test holes/pins, the CAD size
   and the measured size (calipers). App fits a linear model measured = a*cad + b separately for
   holes and pins (least squares), shows residuals and derived XY compensation, and lets the user
   "Apply" it to the calculator's printer profile.
3. **Tolerance test coupon generator** — generates a binary STL (in Rust, client-side) of a flat
   plate with a row of holes (or pegs) stepping clearance from e.g. -0.1 to +0.5 mm in user-chosen
   steps, with each step marked by notches/dots count (no text rendering needed). Download via
   Blob/object URL. Also show a simple 2D preview (top view) of the coupon. Keep triangle count
   reasonable (configurable segments per circle, default 48).
4. **Thread/screw cheat helper** (small): table of common metric screw sizes (M2–M8): recommended
   printed hole for self-tapping into plastic, clearance hole, heat-set insert hole (typical
   values, clearly labelled as starting points to test), nut trap across-flats + clearance.
5. Profiles: save multiple printer profiles to localStorage, import/export JSON file, share
   settings via URL fragment (#p=<base64>), fragment never sent to a server.
6. Korean + English UI toggle (default follows browser language). Mobile friendly. Dark/light
   follows system. Clearly state values are starting points; test on your own printer.

## Core crate (pure Rust, no web deps, unit tested)
- `crates/fit_core`: fit classes table (documented defaults, e.g. press -0.05..0, snug 0.1,
  slide 0.2, loose 0.3, free-spin 0.4 mm diametral for FDM; smaller for resin), compensation math,
  polygon correction, least-squares calibration, STL writer (binary, correct normals, watertight
  plate with cylindrical holes via simple triangulation is acceptable; validate triangle count and
  that the byte length = 84 + 50*n), profile JSON (serde) + URL share encode/decode.
- `crates/fit_web`: UI. Prefer plain web-sys or a small framework (Leptos CSR / Yew) — small bundle.

## Repo hygiene
- README (Korean first, then English): what it is, how to use each panel, the math & limitations,
  local build (`cargo test -p fit_core`, `trunk build --release`), MIT license.
- LICENSE: MIT, copyright "print-fit contributors". `.gitignore` for target/ dist/.
- `index.html` with a CSP meta that blocks external requests, e.g.
  `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'self'`
  (make sure trunk's injected scripts still work; add 'unsafe-inline' to script-src only if strictly needed).
- `Trunk.toml`: `public_url = "/print-fit/"`.
- `.github/workflows/pages.yml`: checkout, rust stable + wasm32 target, install trunk 0.22.1 binary,
  `sh tools/privacy_check.sh`, `cargo test -p fit_core --release`, `trunk build --release`,
  configure-pages@v5, upload-pages-artifact@v3 (dist), deploy job with actions/deploy-pages@v4.
- `tools/privacy_check.sh` already exists; keep it and keep it clean.

## Done when
- `cargo test -p fit_core` green (tests: fit table lookup, hole/shaft compensation, polygon
  correction, calibration fit recovers known a/b, STL byte length & normal sanity, profile JSON and
  URL share round-trip).
- `trunk build --release` succeeds; `sh tools/privacy_check.sh` clean.
- Commit everything with clear commits. Do NOT create new GitHub repos or change repo settings.
