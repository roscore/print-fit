# print-fit

3D 프린팅(FDM·레진) 후에도 잘 맞는 부품을 설계하기 위한 끼워맞춤·공차 계산기입니다.
Rust → WebAssembly로 만든 정적 웹앱이며 **100% 브라우저 안에서만** 동작합니다
(추적·분석 없음, 서버 없음, 실행 중 네트워크 요청 없음).

> 모든 값은 **출발점**입니다. 반드시 본인의 프린터·재료로 테스트하세요.

[English below](#english)

## 한국어

### 패널별 사용법

**프린터 프로파일** (상단 카드)
- 공정 프리셋(FDM 0.4 mm, FDM 0.6 mm, 레진 MSLA)을 고르면 기본 보정값이 채워집니다.
- 보정값: XY 구멍 수축(직경 기준 mm), 핀·탭 팽창(mm), 재료 수축(%), 엘리펀트 풋(첫 레이어, 참고용),
  수평 구멍 처짐(참고용).
- 프로파일은 여러 개 저장할 수 있고(localStorage), JSON 파일로 내보내기/가져오기가 됩니다.
- **공유 링크**는 설정을 `#p=<base64>` URL 프래그먼트에 담습니다. 프래그먼트는 브라우저가 서버로 보내지 않습니다.

**끼워맞춤 계산기**
- 공칭 직경/폭, 형상(원형 구멍+축 / 사각 슬롯+탭), 등급(압입·빡빡함·슬라이드·헐거움·자유 회전),
  어느 쪽을 출력하는지(둘 다 / 축이 기성품 / 구멍이 기성품)를 고르면
  추천 CAD 구멍·축 크기, 직경·반경 간극, 설명, 간극을 과장한 단면도를 보여줍니다.
- “다각형 구멍 보정”을 켜면 N각형으로 내보낼 때 필요한 외접 직경 `d / cos(π/N)`을 알려줍니다.

**보정 (테스트 출력)**
- CAD 크기를 아는 구멍·핀을 출력하고 캘리퍼스로 잰 값을 한 줄에 한 쌍씩 입력합니다 (`5.0 4.82`).
- 구멍과 핀 각각 최소제곱 직선 `측정 = a·CAD + b`를 맞추고 잔차와 도출된 보정값을 보여줍니다.
- “프로파일에 적용”을 누르면 현재 프로파일에 반영됩니다.

**테스트 쿠폰 (STL)**
- 간극을 단계적으로 바꾼 구멍(또는 핀) 한 줄이 있는 판을 바이너리 STL로 만들어 내려받습니다.
- 각 단계는 돌기로 표시됩니다: 막대 = 5, 점 = 1 (왼쪽부터 1단계). 크기는 보정하지 않은 CAD 원래 값입니다.
- 원 분할 수(기본 48)로 삼각형 수를 조절합니다.

**나사 치트시트** — M2–M8의 셀프태핑 구멍, 관통 구멍, 열압입 인서트 구멍, 너트 맞변·트랩 크기.
일반적인 값일 뿐이며 인서트 구멍은 제조사 데이터시트를 확인하세요.

### 계산 방법과 한계
- 보정 모델: `출력 구멍 = CAD·k − 구멍수축`, `출력 핀 = CAD·k + 핀팽창`, `k = 1 − 수축률/100`.
  필요한 CAD 크기는 이 식을 거꾸로 풀어 구합니다.
- 기본 간극(직경, 출력 후): FDM 0.4 — 압입 −0.05…0, 빡빡함 0.10, 슬라이드 0.20, 헐거움 0.30, 자유 회전 0.40 mm.
  FDM 0.6은 조금 크게, 레진은 더 작게(0.05/0.10/0.15/0.20) 잡았습니다.
- “둘 다 출력”이면 간극을 구멍과 축에 반씩 나눕니다. 기성품 쪽은 공칭 그대로 두고 보정하지 않습니다.
- 보정: 구멍·핀 각각 일반 최소제곱으로 `a, b`를 구하고, 공통 배율 `k`는 기울기의 표본 수 가중 평균,
  오프셋은 `k`를 고정한 최소제곱 해(평균 `측정 − k·CAD`)입니다. 서로 다른 CAD 크기가 2개 미만이면 기울기를 1로 고정합니다.
- 한계: 선형 XY 모델일 뿐입니다. 레이어 높이, 방향, 벽 순서, 냉각, 수지 노광·후경화, 재료 흡습 등은 반영하지 않습니다.
  엘리펀트 풋과 수평 구멍 처짐은 안내 문구로만 다룹니다.
- 쿠폰 메쉬: 판은 하나의 닫힌(watertight) 매니폴드 셸이고, 표시 돌기는 판에 살짝 겹친 별도의 닫힌 상자입니다
  (슬라이서가 합칩니다).

### 로컬 빌드
```sh
rustup target add wasm32-unknown-unknown
cargo test -p fit_core          # 코어 단위 테스트
trunk build --release           # dist/ 에 정적 사이트 생성 (public_url = /print-fit/)
sh tools/privacy_check.sh       # 개인정보 패턴 검사 ("clean" 이어야 함)
```
개발 중에는 `trunk serve --public-url /` 로 실행할 수 있습니다.

### 라이선스
MIT — [LICENSE](LICENSE) 참고. Copyright (c) print-fit contributors.

---

## English

A fit & tolerance calculator for designing parts that fit after 3D printing (FDM and resin).
It is a static Rust → WebAssembly web app that runs **entirely in your browser**: no analytics,
no server, no network requests at runtime.

> All values are **starting points**. Test on your own printer and material.

### Panels

**Printer profile** (top card)
- Pick a process preset (FDM 0.4 mm, FDM 0.6 mm, resin MSLA) to load default compensation.
- Compensation: XY hole shrink (mm, diametral), pin/tab growth (mm), material shrink (%),
  elephant foot (first layers, informational), horizontal-hole droop (informational).
- Keep several profiles (localStorage), import/export them as a JSON file.
- **Share link** stores the profile in the URL fragment `#p=<base64>`; browsers never send the
  fragment to a server.

**Fit calculator**
- Choose nominal diameter/width, feature (round hole + shaft, or square slot + tab), fit class
  (press / snug / slide / loose / free spin) and which side is printed (both / shaft is a bought
  part / hole is a bought part). You get the recommended CAD hole and shaft sizes, diametral and
  radial clearance, a short explanation and a cross-section with the gap exaggerated.
- “Faceted hole correction” gives the circumscribed size `d / cos(π/N)` for holes exported as N-gons.

**Calibrate**
- Print holes and pins of known CAD size, measure with calipers, enter one pair per line (`5.0 4.82`).
- A least-squares line `measured = a·CAD + b` is fitted separately for holes and pins, with
  residuals and the derived compensation. “Apply to profile” writes it into the active profile.

**Test coupon (STL)**
- Generates a binary STL of a plate with a row of holes (or pegs) stepping the clearance.
- Steps are marked with bumps: a bar counts 5, a dot counts 1 (step 1 on the left). Sizes are raw,
  uncompensated CAD sizes. Segments per circle (default 48) control the triangle count.

**Screws** — M2–M8 self-tap hole, clearance hole, heat-set insert hole, nut across flats and nut
trap sizes. Typical values only; check your insert's datasheet.

### Math & limitations
- Compensation model: `printed_hole = cad·k − hole_shrink`, `printed_pin = cad·k + pin_grow`,
  `k = 1 − shrink/100`; CAD sizes are obtained by inverting it.
- Default diametral clearances (as printed), FDM 0.4: press −0.05…0, snug 0.10, slide 0.20,
  loose 0.30, free spin 0.40 mm. FDM 0.6 is slightly larger; resin is smaller (0.05/0.10/0.15/0.20).
- “Both printed” splits the clearance half/half. A bought part keeps its nominal size and is not compensated.
- Calibration: ordinary least squares per feature; the shared scale `k` is the sample-weighted mean
  slope, and each offset is the least-squares optimum with `k` fixed (`mean(measured − k·cad)`).
  With fewer than two distinct CAD sizes the slope is fixed to 1.
- Limitations: a linear XY model only. Layer height, orientation, wall order, cooling, resin
  exposure/post-cure and moisture are not modelled. Elephant foot and droop are advisory notes.
- Coupon mesh: the plate is one closed, manifold shell; tally bumps are separate closed boxes that
  slightly overlap the plate (slicers merge them).

### Local build
```sh
rustup target add wasm32-unknown-unknown
cargo test -p fit_core          # core unit tests
trunk build --release           # static site in dist/ (public_url = /print-fit/)
sh tools/privacy_check.sh       # personal-data pattern check (must print "clean")
```
For development: `trunk serve --public-url /`.

### Project layout
- `crates/fit_core` — pure Rust: fit table, compensation, polygon correction, calibration,
  STL writer + coupon generator, profile JSON + URL sharing, screw table. Unit tested.
- `crates/fit_web` — UI with plain `web-sys` (small bundle). Markup in `index.html`, styles in `web/`.
- `web/boot.js` — external loader so the Content-Security-Policy needs no `'unsafe-inline'`.

### License
MIT — see [LICENSE](LICENSE). Copyright (c) print-fit contributors.
