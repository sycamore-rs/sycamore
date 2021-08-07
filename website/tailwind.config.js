const colors = require('tailwindcss/colors')

module.exports = {
  jit: true,
  purge: {
    content: [
      "./src/**/*.rs"
    ],
  },
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
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
}
