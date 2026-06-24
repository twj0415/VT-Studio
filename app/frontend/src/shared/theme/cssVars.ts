export function applyThemeVars(vars: Record<string, string>) {
  const root = document.documentElement

  Object.entries(vars).forEach(([key, value]) => {
    root.style.setProperty(key, value)
  })
}
