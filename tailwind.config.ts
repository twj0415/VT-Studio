import type { Config } from 'tailwindcss';

export default {
  content: ['./src/renderer/index.html', './src/renderer/**/*.{vue,ts}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Microsoft YaHei"', '"PingFang SC"', 'system-ui', 'sans-serif'],
      },
      colors: {
        surface: {
          app: 'var(--vt-surface-app)',
          panel: 'var(--vt-surface-panel)',
          raised: 'var(--vt-surface-raised)',
        },
        line: {
          soft: 'var(--vt-line-soft)',
          strong: 'var(--vt-line-strong)',
        },
        text: {
          primary: 'var(--vt-text-primary)',
          secondary: 'var(--vt-text-secondary)',
          muted: 'var(--vt-text-muted)',
        },
        brand: {
          DEFAULT: 'var(--vt-brand)',
          strong: 'var(--vt-brand-strong)',
        },
        state: {
          success: 'var(--vt-success)',
          warning: 'var(--vt-warning)',
          danger: 'var(--vt-danger)',
        },
      },
      boxShadow: {
        panel: 'var(--vt-shadow-panel)',
      },
    },
  },
  plugins: [],
} satisfies Config;
