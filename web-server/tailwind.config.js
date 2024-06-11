/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["./src/templates/**/*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        rust: "#B7410E",
        rustLight: "#e37444",
        rustDark: "#963409",
      },
    },
    fontFamily: {
      jetBrains: ["JetBrains Mono", "sans-serif"],
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
