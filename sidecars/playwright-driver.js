#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require('node:fs/promises')
const path = require('node:path')
const { pathToFileURL } = require('node:url')

const REQUEST_PATH = process.argv[2]

main().catch((error) => {
  console.error(safeMessage(error))
  process.exit(1)
})

async function main() {
  if (!REQUEST_PATH) {
    throw new Error('template.render_failed: request path is required.')
  }

  const request = JSON.parse(await fs.readFile(REQUEST_PATH, 'utf8'))
  validateRequest(request)

  let playwright
  try {
    playwright = require('playwright-core')
  } catch (_error) {
    throw new Error('template.sidecar_missing: sidecars/playwright-driver.js requires bundled playwright-core.')
  }

  await renderWithRetry(playwright, request)
}

async function renderWithRetry(playwright, request) {
  let lastError = null
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      await renderOnce(playwright, request)
      return
    } catch (error) {
      lastError = error
      if (!isBrowserCrash(error) || attempt > 0) {
        break
      }
    }
  }

  if (isBrowserCrash(lastError)) {
    throw new Error(`browser_crashed: ${safeMessage(lastError)}`)
  }
  throw lastError
}

async function renderOnce(playwright, request) {
  const browser = await playwright.chromium.launch({
    executablePath: request.chromiumPath,
    headless: true,
    chromiumSandbox: true,
    args: [
      '--disable-background-networking',
      '--disable-sync',
      '--disable-extensions',
      '--disable-default-apps',
      '--disable-popup-blocking',
      '--disable-notifications',
      '--no-first-run',
    ],
  })

  try {
    const context = await browser.newContext({
      viewport: request.viewport,
      acceptDownloads: false,
      javaScriptEnabled: true,
      bypassCSP: false,
      storageState: undefined,
      permissions: [],
    })
    await context.addInitScript(buildGuardScript(request))
    await context.addInitScript(request.injectionScript)
    await context.route('**/*', async (route) => {
      const url = route.request().url()
      if (!isAllowedUrl(url, request)) {
        await route.abort('blockedbyclient')
        return
      }
      await route.continue()
    })

    const page = await context.newPage()
    page.on('popup', async (popup) => {
      await popup.close().catch(() => {})
    })
    page.on('download', async (download) => {
      await download.cancel().catch(() => {})
    })

    const entryUrl = pathToFileURL(resolveInsideWorkspace(request.workspaceRoot, request.entryPath)).toString()
    await page.goto(entryUrl, { waitUntil: 'networkidle', timeout: 30000 })
    await page.screenshot({
      path: resolveInsideWorkspace(request.workspaceRoot, request.outputPath),
      type: 'png',
      fullPage: false,
      timeout: 30000,
    })
    await context.close()
  } finally {
    await browser.close().catch(() => {})
  }
}

function buildGuardScript(request) {
  return `
    (() => {
      const deny = () => { throw new Error('template API is disabled in sandbox') }
      Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: false })
      window.open = () => null
      if (window.showOpenFilePicker) window.showOpenFilePicker = deny
      if (window.showSaveFilePicker) window.showSaveFilePicker = deny
      if (window.showDirectoryPicker) window.showDirectoryPicker = deny
      Object.freeze(window.__VT_TEMPLATE_PAYLOAD__ = ${JSON.stringify(request.payload)})
      document.dispatchEvent(new CustomEvent('vt-template-data'))
    })();
  `
}

function validateRequest(request) {
  if (!request || typeof request !== 'object') throw new Error('template.render_failed: invalid request.')
  for (const key of ['workspaceRoot', 'entryPath', 'outputPath', 'chromiumPath']) {
    if (typeof request[key] !== 'string' || request[key].length === 0) {
      throw new Error(`template.render_failed: ${key} is required.`)
    }
  }
  if (!request.viewport || typeof request.viewport.width !== 'number' || typeof request.viewport.height !== 'number') {
    throw new Error('template.render_failed: viewport is required.')
  }
  if (!Array.isArray(request.allowedResourceRoots)) {
    throw new Error('template.render_failed: allowedResourceRoots must be an array.')
  }
  resolveInsideWorkspace(request.workspaceRoot, request.entryPath)
  resolveInsideWorkspace(request.workspaceRoot, request.outputPath)
}

function isAllowedUrl(url, request) {
  const parsed = new URL(url)
  if (parsed.protocol !== 'file:') return false
  const pathname = decodeURIComponent(parsed.pathname)
  const normalized = process.platform === 'win32' && pathname.startsWith('/')
    ? pathname.slice(1)
    : pathname
  const filePath = path.resolve(normalized)
  const workspaceRoot = path.resolve(request.workspaceRoot)
  if (!filePath.startsWith(workspaceRoot + path.sep) && filePath !== workspaceRoot) return false
  const relative = path.relative(workspaceRoot, filePath).replaceAll(path.sep, '/')
  return request.allowedResourceRoots.some((root) => relative.startsWith(root))
}

function resolveInsideWorkspace(workspaceRoot, relativePath) {
  if (relativePath.includes('\\') || relativePath.includes('\0') || relativePath.includes('..') || path.isAbsolute(relativePath)) {
    throw new Error('template.resource_denied: unsafe relative path.')
  }
  const resolved = path.resolve(workspaceRoot, relativePath)
  const root = path.resolve(workspaceRoot)
  if (!resolved.startsWith(root + path.sep) && resolved !== root) {
    throw new Error('template.resource_denied: path escaped workspace.')
  }
  return resolved
}

function isBrowserCrash(error) {
  const message = safeMessage(error)
  return /browser.*(closed|crash|disconnect)|target.*closed|browser_crashed/i.test(message)
}

function safeMessage(error) {
  const raw = error && error.stack ? String(error.stack) : String(error && error.message ? error.message : error)
  return raw.replace(/[A-Za-z]:[\\/][^\s"'<>]+/g, '<path>').replace(/(Bearer\s+)[A-Za-z0-9._+=-]+/gi, '$1***REDACTED***')
}
