Template and media sidecar packaging source.

Expected runtime files are prepared by `scripts/prepare-template-sidecar.ps1` and copied into this directory before packaging:

- `node.exe`
- `chromium.exe`
- `chromium/` copied from the Chrome for Testing `chrome-win64` directory
- `playwright-driver.js`
- `node_modules/playwright-core/`

Do not commit local secrets or provider credentials here.
