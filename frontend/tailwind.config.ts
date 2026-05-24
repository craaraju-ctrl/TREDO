import type { Config } from 'tailwindcss';

export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        cyber: {
          dark: '#0a0e17',
          panel: '#121826',
          border: '#1e293b',
          glow: '#7b2cbf',
          green: '#00e676',
          purple: '#9d4edd',
        },
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['Outfit', 'monospace'],
      },
    },
  },
  plugins: [],
} satisfies Config;
