/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["./src/templates/**/*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {},
    fontFamily: {
      jetBrains: ["JetBrains Mono", "sans-serif"],
    },
  },
};
