# 🌐 Aevrix

> A deterministic, privacy-first first-paint engine in Rust with an Avalonia desktop host.

![Rust](https://img.shields.io/badge/Rust-1.82%2B-000000?logo=rust)
![.NET](https://img.shields.io/badge/.NET-8.0-512BD4?logo=dotnet)
![UI](https://img.shields.io/badge/UI-Avalonia%2011-005C8A)
![Rendering](https://img.shields.io/badge/Rendering-Deterministic%20First%20Paint-1E7F4F)
![Mode](https://img.shields.io/badge/Network-Offline%20by%20Default-8A2BE2)

---

## 📌 What Is Aevrix?

**Aevrix** is an early-stage browser/runtime prototype focused on **deterministic rendering** and **privacy-first defaults**.
Current scope is intentionally narrow: parse HTML + CSS subset, compute layout, and raster a stable first paint.

It is split into:
- A modular Rust engine (`engine/crates/*`)
- A C ABI bridge (`hb_ffi`)
- A desktop host app in Avalonia (`src/Aevrix.App`)

---

## ✨ Current Highlights

- 🧱 **Deterministic pipeline**: HTML tokenization/tree building, CSS cascade, layout, and paint planning with stable traversal and ordering.
- 🎨 **CPU raster path**: RGBA surface output via `hb_graphics` for predictable first paint.
- ⚡ **GPU compositor (bootstrap)**: `hb_compositor` provides an offscreen `wgpu` path.
- 🔌 **FFI bridge**: `hb_ffi` exposes `hb_init`, `hb_render_html`, `hb_surface_release`, and version calls.
- 🖥️ **Desktop shell**: Avalonia app with tabs, address bar, refresh, and viewport-driven re-render.
- 🔒 **Offline-first behavior**: URL inputs currently resolve to offline placeholder content by design.

---

## 🏗️ Architecture

```text
HTML input
  -> hb_html      (tokenizer + DOM tree builder)
  -> hb_css       (parse + selectors + cascade)
  -> hb_layout    (block/inline layout + paint list)
  -> hb_graphics  (deterministic CPU raster, RGBA8)
  -> hb_ffi       (C ABI surface for hosts)
  -> Aevrix.App   (Avalonia desktop integration)
```

Also available:
- `hb_cli`: headless harness (`HTML -> RGBA/PNG`)
- `hb_compositor`: GPU-first compositor experiments via `wgpu`
- `hb_net`: capability-gated/offline-first network surface (currently stub-focused)

---

## 📂 Repository Layout

```text
Aevrix_Backup/
├─ engine/
│  ├─ Cargo.toml
│  ├─ crates/
│  │  ├─ hb_html/
│  │  ├─ hb_css/
│  │  ├─ hb_layout/
│  │  ├─ hb_graphics/
│  │  ├─ hb_compositor/
│  │  ├─ hb_ffi/
│  │  ├─ hb_cli/
│  │  ├─ hb_core/
│  │  └─ hb_net/
├─ src/
│  └─ Aevrix.App/         (Avalonia desktop host)
├─ tests/
│  ├─ dotnet/
│  └─ rust/
└─ Aevrix.sln
```

---

## 🛠️ Tech Stack

- **Engine**: Rust (workspace crates)
- **Interop**: C ABI (`cdylib`) + P/Invoke
- **Desktop UI**: .NET 8 + Avalonia 11
- **Graphics**:
  - CPU raster: custom deterministic painter
  - GPU path: `wgpu` compositor (offscreen flow)
- **Tests**: Rust unit tests + xUnit (.NET)

---

## 🚀 Quick Start

### 1) Prerequisites

- Rust toolchain `1.82.0` (see `engine/rust-toolchain.toml`)
- .NET SDK `8.0+`
- Windows currently has the smoothest path in this workspace layout

### 2) Build Rust Engine Components

```powershell
cd engine
cargo build --release -p hb_ffi
cargo build --release -p hb_cli
```

### 3) Run Headless Renderer (CLI)

Render from a file:

```powershell
cd engine
@'
<style>body{background:#0b0b0f;color:#e6e6e6;font-size:22px}</style>
<div style="background:#181a1f;padding:10px">Hello from Aevrix CLI</div>
'@ | Set-Content .\demo.html

cargo run -p hb_cli -- --in .\demo.html --out ..\first-paint.png --width 800 --height 600
```

Render from stdin (raw RGBA to stdout if `--out` is omitted):

```powershell
cd engine
"<div style='font-size:22px'>Hello Aevrix</div>" | cargo run -p hb_cli -- --width 800 --height 600 --out ..\first-paint.png
```

### 4) Run Desktop Host

```powershell
dotnet run --project src\Aevrix.App\Aevrix.App.csproj -c Debug
```

---

## 🧪 Validation Commands

Rust workspace tests:

```powershell
cd engine
cargo test --workspace
```

.NET tests:

```powershell
dotnet test tests\dotnet\Hummingbird.App.Tests\Hummingbird.App.Tests.csproj -c Debug
```

---

## ⚠️ Current Status / Known Issues

This repository is clearly in **active bootstrap/refactor state**. As of **March 4, 2026**:

- `cargo test --workspace` mostly runs, but currently has a failing unit test in `hb_html` (`attributes_are_sorted`).
- `dotnet build`/`dotnet test` currently fail early due Avalonia XAML parsing error in:
  - `src/Aevrix.App/Styles/Colors.axaml` (`x` namespace prefix undeclared).
- Naming migration is in progress (`Hummingbird.*` namespaces still exist in parts of the .NET side).
- Rust integration test manifest at `tests/rust/hb_layout_tests` is incomplete (no valid Cargo target entrypoint yet).

---

## 🧭 Roadmap Direction

- ✅ Keep first-paint deterministic with reproducible paint output
- ⏭️ Expand CSS/layout coverage (margins, padding, richer inline flow)
- ⏭️ Tighten host/engine integration and namespace consistency
- ⏭️ Stabilize test matrix across Rust + .NET
- ⏭️ Introduce capability-gated networking beyond offline stubs

---

## 🤝 Contributing

Contributions are welcome, especially around:
- deterministic rendering tests
- parser/layout correctness
- Avalonia host cleanup and integration hardening
- documentation and reproducible setup scripts

---

## 📜 License

Engine crates currently declare `MIT OR Apache-2.0` in their manifests.
If you plan to publish this repo as-is, add a root `LICENSE` file to make licensing explicit for the full project.
