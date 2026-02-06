import type { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";

const config: Config = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "#f0f9ff",
          500: "#0ea5e9",
          700: "#0369a1",
        },
      },
    },
  },
  plugins: [tailwindcssAnimate],
};

export default config;
