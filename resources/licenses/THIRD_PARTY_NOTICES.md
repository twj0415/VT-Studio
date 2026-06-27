# Third Party Notices

This file is bundled with the Windows desktop package. It records license obligations for runtime components used by VT AI Short Video Maker.

## Application Dependencies

| Component | Scope | License status |
|---|---|---|
| Tauri / tauri-build / tauri CLI | Desktop runtime and build tooling | Rust/Node package license files are provided by upstream packages in dependency folders. |
| Vue / Vue Router / Pinia / Vue I18n | Frontend runtime | Upstream package license files are present under `src/node_modules` during development and must be included in release notices. |
| Naive UI | Frontend UI runtime | Upstream package license file is present under `src/node_modules` during development and must be included in release notices. |
| Vite / TypeScript / Tailwind CSS / Sass / vue-tsc | Build tooling | Development/build-time dependencies; include notices where distributed in the final package. |
| rusqlite bundled SQLite | Local database | SQLite is public domain; rusqlite package license is provided by upstream crate metadata. |
| keyring | System keyring integration | Upstream crate license metadata must remain reviewed before release. |

## Runtime Sidecars

| Component | Required before release | Current status |
|---|---|---|
| FFmpeg / FFprobe | Record exact build source, version, enabled codecs, and LGPL/GPL boundary. Include license text with the installer. | Pending. `resources/bin/ffmpeg.exe` and `ffprobe.exe` are not currently present. |
| Node.js | Record exact distribution source and include Node.js license notices. | Pending. `resources/bin/node.exe` is not currently present. |
| Chromium | Record exact distribution source and include Chromium license / third-party notices. | Pending. `resources/bin/chromium/` is not currently present. |
| Playwright core / driver | Include Playwright `LICENSE`, `NOTICE`, and `ThirdPartyNotices.txt`. | Pending for packaged `resources/bin/node_modules/playwright-core/`. |

## Project-Owned Assets

The built-in HTML templates under `templates/builtin/` are project-owned source assets unless a later template explicitly declares another source. New templates, fonts, images, audio, or sample media must declare source and license before being bundled.

## Reference Projects

Reference projects are used only for product and architecture analysis. Do not copy code, assets, templates, prompts, or binary files from reference repositories unless their license has been reviewed and explicitly recorded here.
