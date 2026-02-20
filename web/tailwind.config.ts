import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: "#050505",
          2:       "#090909",
          3:       "#0E0E0E",
          base:    "#050505",
          card:    "#090909",
          "card-hover": "#0E0E0E",
        },
        border: {
          DEFAULT: "#1C1C1C",
          2:       "#282828",
          3:       "#383838",
        },
        surface: { 1: "#090909", 2: "#0E0E0E" },
        text: {
          primary: "#E8E8E8",
          muted:   "#909090",
          dim:     "#606060",
          1:       "#E8E8E8",
          2:       "#909090",
          3:       "#606060",
        },
        accent: {
          DEFAULT: "#00FF88",
          cyan:    "#00C8FF",
          green:   "#00FF88",
          yellow:  "#FFB547",
          amber:   "#FFB547",
          red:     "#FF4545",
          purple:  "#A78BFA",
          magenta: "#A78BFA",
        },
      },
      fontFamily: {
        sans: ["var(--font-sans)", "Inter", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "monospace"],
      },
      fontSize: {
        "2xs": ["0.625rem", { lineHeight: "1rem" }],
      },
    },
  },
  plugins: [],
};

export default config;
