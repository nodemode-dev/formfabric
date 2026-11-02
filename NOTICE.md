# Third-party notices

formfabric is licensed under the Apache License 2.0 (see `LICENSE`). Copyright 2026 NodeAgentc, Inc.

## Included in this repository

| Work | License | Where |
| --- | --- | --- |
| formvis desktop shell (nodemode-ai) | Apache-2.0 | `src-tauri/src/{reveal,traffic_lights,text_edit,window_state,packaged,main}.rs`, `src-tauri/build.rs`, `capabilities/`, `entitlements.plist`, `about.*`, `frontend/public/splash.html` (geometry and timing), `setup/freeze_backend` (adapted) |
| formvis window shell (nodemode-ai) | Apache-2.0 | `frontend/index.html` (theme boot), `frontend/src/ui/{dom,dialogs,dialog-history,dropdown,number-fields,toast,menu,native-menu,caption-buttons,commands,command-palette,i18n,themes,icons}.ts`, `frontend/src/ui/i18n/` (core tables), `frontend/src/styles/` (tokens and shell rules) (adapted) |
| formsculpt scripts, service, dock and engine pieces (nodemode-ai) | Apache-2.0 | `formfabric`, `formfabric.bat`, `build_app`, `setup/`, `backend/app/` (core, terminal, Claude Code, chat, agents, MCP), `frontend/src/dock/`, `frontend/src/styles/dock.css`, `native/crates/ffb-core/src/{bvh,mesh}.rs`, the registry and history shape in `native/crates/ffb-commands`, `ffb-py`, `ffb-wasm`, `frontend/src/workspace/view3d/{camera,math}.ts`, `.claude/` and `.agents/` kit layout, `setup/agents.py` (adapted) |
| NodeMode design system tokens | MIT | `frontend/src/styles/tokens.css` (values) |

## Data

Default bodies: the female and male base bodies are derived from the MakeHuman community assets distributed with MPFB2 (base mesh, macro targets, default rig and skin weights), dedicated to the public domain under CC0 1.0 Universal. Source: https://github.com/makehumancommunity/mpfb2 (assets, not the GPLv3 code; no MPFB code is included). Licence text: `assets/bodies/mpfb2-female/LICENSE.txt` and `assets/bodies/mpfb2-male/LICENSE.txt`. Converted, scaled and re-skinned by formfabric. The Basic Form body is formfabric's own.

Fabric presets and Fabric Lab formulas are derived from published measurements and textile-testing literature cited in `docs/encyclopedia/16-FABRIC-DATA.md`; the values are formfabric's own and no dataset is redistributed.

The DXF-AAMA reader and writer follow the public ASTM D6673 conventions as described in the sources cited in the build notes; no code from any CAD vendor is included.

## Referenced, not included

`docs/encyclopedia/` describes Marvelous Designer (CLO Virtual Fashion Inc.) from public sources, paraphrased and linked. No code, artwork, avatars, presets or manual text from it are included. Marvelous Designer and CLO are trademarks of CLO Virtual Fashion Inc.

## Distributed with the app

Audited in Phase 18 against the lock files: `native/Cargo.lock` and `src-tauri/Cargo.lock` (Rust, with `cargo tree -e normal` for the engine module `ffb-py` with its default `gpu` feature, the in-page engine `ffb-wasm` for wasm32, and the desktop shell for Linux, macOS and Windows targets), `frontend/package-lock.json` (npm `dependencies` only) and `backend/requirements*.txt` (Python, licences read from the installed distributions). Licences are read from each crate's `Cargo.toml`. Where a crate offers a choice (`OR`), formfabric uses it under the permissive option named in the tables. Result: no GPL, LGPL or AGPL code is linked or distributed; MPL-2.0 appears in five desktop shell crates (file-level, see Notes).

### Licence summary (Rust)

433 distinct crates (463 name and version pairs): 137 in the native engine module, 40 in the wasm engine, and 364 in the desktop shell (312 of them only there). Counts by declared licence, per distinct crate:

| Declared licence | Crates | Where listed |
| --- | --- | --- |
| MIT or Apache-2.0 (either order, or `/`) | 254 | Rust group A |
| MIT | 107 | Rust group B |
| MIT or Apache-2.0 with a further option (Zlib, Unlicense, ISC, BSD-2-Clause, 0BSD, CC0-1.0, MIT-0, LLVM exception, BSD-3-Clause) | 31 | Rust group C |
| Unicode-3.0 | 18 | Rust group D |
| Apache-2.0 only | 6 | Rust group D |
| MPL-2.0 | 5 | Rust group D |
| ISC | 3 | Rust group D |
| BSD-3-Clause | 3 | Rust group D |
| Zlib | 1 | Rust group D |
| Combined with AND (each part permissive) | 5 | Rust group D |

### Rust group A: MIT or Apache-2.0 (254)

allocator-api2, anyhow, arrayvec, ash, async-broadcast, async-channel, async-executor, async-io, async-lock, async-process, async-recursion, async-signal, async-task, async-trait, atomic-waker, base64, bit-set, bit-vec, bitflags, block-buffer, blocking, bumpalo, camino, cargo-platform, cfg-if, chacha20, concurrent-queue, const-oid, cookie, core-foundation, core-foundation-sys, core-graphics, core-graphics-types, core_detect, cpufeatures, crc32fast, crossbeam-channel, crossbeam-deque, crossbeam-epoch, crossbeam-utils, crypto-common, ctor, dbus, deranged, digest, dirs, dirs-sys, displaydoc, document-features, dtoa, dyn-clone, either, embed_plist, enumflags2, enumflags2_derive, equivalent, erased-serde, errno, event-listener, event-listener-strategy, fastrand, fdeflate, field-offset, filetime, flate2, fnv, foreign-types, foreign-types-macros, foreign-types-shared, form_urlencoded, futures-channel, futures-core, futures-executor, futures-io, futures-lite, futures-macro, futures-sink, futures-task, futures-util, getrandom, glam, glob, gltf, gltf-derive, gltf-json, gpu-allocator, half, hashbrown, heck, hex, html5ever, http, httparse, hybrid-array, ident_case, idna, idna_adapter, indexmap, ipnet, itoa, json-patch, jsonptr, keyboard-types, khronos-egl, lazy_static, libc, libdbus-sys, litrs, lock_api, log, markup5ever, mime, muda, multiversion_no_op, naga, naga-types, num-conv, num-traits, once_cell, openssl-probe, ordered-stream, osakit, parking, parking_lot, parking_lot_core, percent-encoding, pin-project-lite, piper, png, polling, pollster, powerfmt, presser, proc-macro-crate, proc-macro-error, proc-macro-error-attr, proc-macro2, profiling, pyo3, pyo3-ffi, pyo3-macros, pyo3-macros-backend, quote, rand, rand_core, range-alloc, raw-window-metal, rayon, rayon-core, regex, regex-automata, regex-syntax, renderdoc-sys, reqwest, robust, rustc-hash, rustls-pki-types, rustls-platform-verifier, scopeguard, security-framework, security-framework-sys, semver, serde, serde-untagged, serde_core, serde_derive, serde_derive_internals, serde_json, serde_repr, serde_spanned, serde_with, serde_with_macros, serialize-to-javascript, serialize-to-javascript-impl, servo_arc, sha1, sha2, signal-hook-registry, simdutf8, siphasher, smallvec, socket2, softbuffer, spade, stable_deref_trait, static_assertions, string_cache, syn, system-configuration, system-configuration-sys, tar, tauri, tauri-codegen, tauri-macros, tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-shell, tauri-plugin-single-instance, tauri-plugin-updater, tauri-runtime, tauri-runtime-wry, tauri-utils, tempfile, tendril, thiserror, thiserror-impl, time, time-core, time-macros, tokio-rustls, toml, toml_datetime, toml_edit, toml_parser, toml_writer, tungstenite, typed-path, typeid, typenum, unicode-segmentation, unicode-width, url, utf8_iter, utf_types, uuid, web-time, web_atoms, wgpu, wgpu-core, wgpu-core-deps-apple, wgpu-core-deps-windows-linux-android, wgpu-hal, wgpu-naga-bridge, wgpu-types, window-vibrancy, windows, windows-collections, windows-core, windows-future, windows-implement, windows-interface, windows-link, windows-numerics, windows-registry, windows-result, windows-strings, windows-sys, windows-targets, windows-threading, windows-version, windows_x86_64_msvc, wry, xattr, zeroize.

### Rust group B: MIT (107)

atk, atk-sys, block2, bytes, cairo-rs, cairo-sys-rs, cargo_metadata, cfb, darling, darling_core, darling_macro, data-encoding, derive_more, derive_more-impl, dlib, dlopen2, dlopen2_derive, dom_query, endi, gdk, gdk-pixbuf, gdk-pixbuf-sys, gdk-sys, gdkwayland-sys, gdkx11, gdkx11-sys, generic-array, gio, gio-sys, glib, glib-macros, glib-sys, gobject-sys, gtk, gtk-sys, gtk3-macros, http-body, http-body-util, hyper, hyper-util, ico, infer, inflections, is-docker, is-wsl, javascriptcore-rs, javascriptcore-rs-sys, libm, memoffset, minisign-verify, mio, new_debug_unreachable, objc2, objc2-encode, objc2-foundation, open, ordered-float, os_pipe, pango, pango-sys, phf, phf_generator, phf_macros, phf_shared, plist, precomputed-hash, quick-xml, rfd, schemars, schemars_derive, shared_child, simd-adler32, slab, soup3, soup3-sys, strsim, synstructure, tokio, tokio-util, tower, tower-http, tower-layer, tower-service, tracing, tracing-attributes, tracing-core, try-lock, urlpattern, want, wayland-sys, webkit2gtk, webkit2gtk-sys, webview2-com, webview2-com-macros, webview2-com-sys, winnow, x11, x11-dl, zbus, zbus_macros, zbus_names, zcheapstr, zip, zmij, zvariant, zvariant_derive, zvariant_utils.

### Rust group C: dual licensed with a further permissive option (31)

| Crate | Declared | Used under |
| --- | --- | --- |
| adler2 | 0BSD, MIT or Apache-2.0 | MIT |
| aho-corasick, byteorder, memchr, same-file, walkdir, winapi-util | Unlicense or MIT | MIT |
| brotli-decompressor | BSD-3-Clause or MIT | MIT |
| bytemuck, bytemuck_derive, dispatch2, objc2-app-kit, objc2-core-foundation, objc2-core-graphics, objc2-exception-helper, objc2-metal, objc2-osa-kit, objc2-quartz-core, objc2-web-kit | Zlib, Apache-2.0 or MIT | MIT |
| dunce | CC0-1.0, MIT-0 or Apache-2.0 | MIT-0 |
| glow, raw-window-handle, miniz_oxide (0.8.9 and 0.9.1) | MIT, Zlib or Apache-2.0 | MIT |
| hyper-rustls, rustls, rustls-native-certs | Apache-2.0, ISC or MIT | MIT |
| linux-raw-sys, rustix | Apache-2.0 with LLVM exception, Apache-2.0 or MIT | MIT |
| ufbx 0.11.5 (with the bundled ufbx C library) | MIT or Unlicense | MIT |
| zerocopy, zerocopy-derive | BSD-2-Clause, Apache-2.0 or MIT | MIT |

### Rust group D: other licences, named explicitly

| Licence | Crates | Surface |
| --- | --- | --- |
| Unicode-3.0 | icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive (all ICU4X) | desktop shell |
| Apache-2.0 only | codespan-reporting 0.13.1, glutin_wgl_sys 0.6.1, spirv 0.4.0, sync_wrapper 1.0.2, tao 0.37.2, zopfli 0.8.4 | codespan-reporting, glutin_wgl_sys and spirv in the GPU solver; zopfli in the engine and wasm; sync_wrapper and tao in the shell |
| MPL-2.0 | cssparser 0.37.0, cssparser-macros 0.7.1, dtoa-short 0.3.5, option-ext 0.2.0, selectors 0.38.0 | desktop shell only (HTML and CSS parsing in Tauri) |
| ISC | libloading 0.8.9 (GPU solver), rustls-webpki 0.103.16, untrusted 0.9.0 (shell, TLS) | engine and shell |
| BSD-3-Clause | alloc-no-stdlib 3.0.0, alloc-stdlib 0.3.0, subtle 2.6.1 | desktop shell |
| Zlib | foldhash 0.2.0 | engine, wasm and shell |
| BSD-3-Clause and MIT | brotli 9.1.1 | desktop shell |
| Apache-2.0 and MIT | dpi 0.1.2 | desktop shell |
| (Apache-2.0 or MIT) and BSD-3-Clause | encoding_rs 0.8.44 | desktop shell |
| Apache-2.0 and ISC | ring 0.17.14 | desktop shell (TLS) |
| (MIT or Apache-2.0) and Unicode-3.0 | unicode-ident 1.0.27 | engine, wasm and shell |

### npm (runtime `dependencies` of `frontend/package.json`)

| Package | License | Notes |
| --- | --- | --- |
| @fontsource/inter 5.3.0, @fontsource/jetbrains-mono 5.3.0 | OFL-1.1 | bundled fonts; the SIL Open Font License text ships with the font files |
| @xterm/xterm 6.0.0, @xterm/addon-fit 0.11.0 | MIT | terminal dock |
| @tauri-apps/api 2.11.1 | MIT or Apache-2.0 | window bridge |
| @tauri-apps/plugin-dialog 2.8.4 | MIT or Apache-2.0 | file dialogs |

These six packages have no further runtime dependencies in `package-lock.json`.

### Python (runtime requirements of `backend/requirements*.txt`)

| Package | License | Notes |
| --- | --- | --- |
| fastapi 0.141.1, annotated-doc 0.0.5, annotated-types 0.8.0, typing_inspection 0.4.4, anyio 4.15.1 | MIT | engine service |
| pydantic 2.14.2, pydantic_core 2.50.2 | MIT | pydantic_core is a compiled Rust wheel; its own Rust crates are MIT, Apache-2.0, Unicode-3.0, Unlicense, BSD-2-Clause, Zlib and LLVM-exception terms, and one is offered as MIT, Apache-2.0 or LGPL-2.1-or-later, used here under MIT |
| starlette 1.10.0, uvicorn 0.53.0, click 8.5.0, idna 3.20, websockets 17.4, python-dotenv 1.2.4 | BSD-3-Clause | engine service |
| h11 0.16.0, httptools 0.9.0, pyyaml 6.0.3, watchfiles 1.3.0 | MIT | uvicorn standard extras |
| uvloop 0.23.0 | MIT or Apache-2.0 | uvicorn standard extra, not on Windows |
| python-multipart 0.0.32 | Apache-2.0 | engine service |
| typing_extensions 4.16.0 | PSF-2.0 | permissive, Python Software Foundation licence |
| ptyprocess 0.7.0 | ISC | terminal, not on Windows |
| pywinpty 3.0.5 | MIT | terminal on Windows; licence as declared upstream, not installed on the audit machine |

The frozen service also carries CPython (PSF-2.0) and the shared libraries that interpreter links. Inspect the frozen folder (`src-tauri/engine`) after each freeze and add anything found.

### Notes

- Copyleft: none found. The only copyleft text met is an alternative licence on a choice (LGPL-2.1-or-later inside pydantic_core's Rust dependencies, and in `src-tauri/about.toml`'s accepted list), never the one used. Nothing GPL is a dependency. Blockers: none.
- MPL-2.0 (cssparser, cssparser-macros, dtoa-short, option-ext, selectors): file-level copyleft, allowed by the repository rule. They are used unmodified from crates.io; their source is available at the crate pages. Keep it that way, or publish any changed files.
- Linux desktop builds link WebKitGTK, GTK and GLib dynamically through the system packages (LGPL libraries, not bundled and not modified); the Rust binding crates are MIT. Windows uses the system WebView2; macOS uses the system WebKit.
- ring carries code from BoringSSL (Apache-2.0 and ISC) and once_cell (MIT or Apache-2.0) with their licence files, in addition to its own ISC terms; those licence texts must ship with the desktop app (`src-tauri/about.hbs` generates the third-party licence page).
- The ufbx C library is compiled into desktop builds and the Python module (not wasm), under MIT or Unlicense (public domain).
- Present in the lock files but not in any shipped tree: slotmap 1.1.1 (Zlib, a wasm-only dependency of glow), zlib-rs 0.6.8 (Zlib, a disabled feature) and, as build scripts only, gl_generator 0.14.0 and khronos_api 3.1.0 (Apache-2.0). Proc-macro and build-script crates in the tables above are compile-time only; they are listed because the dependency trees include them.
- Build tools that are not distributed: Vite (MIT), TypeScript (Apache-2.0), @tauri-apps/cli (MIT or Apache-2.0), maturin (MIT or Apache-2.0), PyInstaller (GPL-2.0 with the bootloader exception, so the frozen program is not under the GPL).

This file grows with each phase: every adapted family file and every distributed dependency is listed when it is added. Regenerate the Rust tables with `cargo tree -e normal --prefix none` per shipped crate and target, and read each licence from the crate's `Cargo.toml`.
