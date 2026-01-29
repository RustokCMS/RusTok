/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./assets/**/*.css"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["light", "dark", "corporate", "luxury"],
  },
};
