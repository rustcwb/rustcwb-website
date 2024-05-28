/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["./src/templates/**/*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        'rust': '#B7410E'
      }
    },
    fontFamily: {
      jetBrains: ["JetBrains Mono", "sans-serif"],
    },
  },
};
