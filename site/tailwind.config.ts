import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{astro,html,js,ts,jsx,tsx,md,mdx}"],
  theme: {
    extend: {
      colors: {
        bg:      "#0D1117",
        surface: { 1: "#161B22", 2: "#1C2128" },
        border:  "#21262D",
        accent:  "#10B981",
        text: {
          1: "#E6EDF3",
          2: "#8B949E",
          3: "#484F58",
        },
      },
      fontFamily: {
        sans:    ["DM Sans", "system-ui", "sans-serif"],
        mono:    ['"JetBrains Mono"', "monospace"],
        display: ["Outfit", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [require("@tailwindcss/typography")],
};

export default config;
