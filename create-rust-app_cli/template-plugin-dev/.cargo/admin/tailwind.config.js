const defaultTheme = require('tailwindcss/defaultTheme')

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/**/**/*.{js,jsx,ts,tsx}',
    './src/**/**/*.{js,jsx,ts,tsx}',
    './src/**/*.{js,jsx,ts,tsx}',
    './src/*.{js,jsx,ts,tsx}',
    './src/Woosh/*.tsx',
    './src/Woosh/RichTextEditor/*.tsx',
    './public/index.html',
    './src/dynamic-tailwind-classes.txt',
  ],
  theme: {
    screens: {
      mj: '480px',
      ...defaultTheme.screens,
    },
  },
  plugins: [],
}
