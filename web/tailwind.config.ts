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
          base: "#0d0d0f",
          card: "rgba(255,255,255,0.04)",
          "card-hover": "rgba(255,255,255,0.08)",
        },
        border: "rgba(255,255,255,0.08)",
        accent: {
          cyan: "#22d3ee",
          green: "#4ade80",
          yellow: "#facc15",
          magenta: "#e879f9",
          red: "#f87171",
        },
        text: {
          primary: "#e2e8f0",
          muted: "#64748b",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
