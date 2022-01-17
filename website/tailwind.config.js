const colors = require('tailwindcss/colors');

module.exports = {
  jit: true,
  purge: {
    content: [
      "./src/**/*.rs"
    ],
  },
  darkMode: "class", // or 'media' or 'class'
  theme: {
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
