/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{js,jsx,ts,tsx}'],
  darkMode: 'class',
  corePlugins: {
    preflight: false,
  },
  theme: {
    extend: {
      colors: {
        'z-bg': '#0a0a0a',
        'z-surface': '#111111',
        'z-border': '#1e1e1e',
        'z-text': '#ededed',
        'z-muted': '#888888',
        'z-accent': '#f97316',
        'z-accent-hover': '#fb923c',
      },
    },
  },
  plugins: [],
};
