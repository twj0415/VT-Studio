const fs = require('fs')
const path = require('path')

const sidecars = path.resolve(__dirname, '..', 'sidecars')
const roots = fs.readdirSync(sidecars, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && entry.name.startsWith('ms-playwright'))
  .map((entry) => path.join(sidecars, entry.name))

const candidates = []
for (const root of roots) {
  collect(root, 0)
}

function collect(dir, depth) {
  if (depth > 6) return
  let entries
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true })
  } catch {
    return
  }
  for (const entry of entries) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) collect(full, depth + 1)
    else if (entry.name.toLowerCase() === 'chrome.exe') candidates.push(full)
  }
}

const chrome = candidates.find((candidate) => candidate.includes(`chrome-win64${path.sep}chrome.exe`))
  || candidates.find((candidate) => candidate.endsWith(`${path.sep}chrome.exe`))

if (!chrome) {
  console.error('No downloaded chrome.exe found under sidecars/ms-playwright*')
  process.exit(1)
}

const sourceDir = path.dirname(chrome)
const targetDir = path.join(sidecars, 'chromium')
fs.rmSync(targetDir, { recursive: true, force: true })
fs.cpSync(sourceDir, targetDir, { recursive: true })
fs.copyFileSync(chrome, path.join(sidecars, 'chromium.exe'))
console.log(`Copied ${sourceDir} -> ${targetDir}`)
console.log(`Copied launcher ${chrome} -> ${path.join(sidecars, 'chromium.exe')}`)
