const colors = require('tailwindcss/colors');

module.exports = {
  content: [
    "./src/**/*.rs"
  ],
  darkMode: "class", // or 'media' or 'class'
  theme: {
    fontFamily: {
      body: ['Inter', 'system-ui', 'sans-serif'],
      mono: ['IBM Plex Mono', 'Menlo', 'monospace'],
      code: ['ui-monospace', 'monospace'],
    },
    extend: {
      zIndex: {
        'neg': -1
      }
    },
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
      black: colors.black,
      red: colors.red,
      gray: colors.gray,
      orange: colors.orange,
      amber: colors.amber,
      yellow: colors.yellow,
      white: colors.white,
    }
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
