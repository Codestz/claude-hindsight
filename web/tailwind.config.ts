import type { Config } from "tailwindcss";

// Every color points to a CSS custom property.
// The hex lives in globals.css — Tailwind utilities are aliases only.

const config: Config = {
  darkMode: "class",
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Backgrounds
        canvas:  "var(--bg-0)",
        surface: "var(--bg-1)",
        card:    "var(--bg-2)",
        overlay: "var(--bg-3)",

        // Borders
        "border-subtle":   "var(--border-1)",
        "border-default":  "var(--border-2)",
        "border-emphasis": "var(--border-3)",

        // Text
        "text-primary":   "var(--text-1)",
        "text-secondary": "var(--text-2)",
        "text-muted":     "var(--text-3)",

        // Accents
        green:  "var(--green)",
        amber:  "var(--amber)",
        cyan:   "var(--cyan)",
        purple: "var(--purple)",
        danger: "var(--red)",

        // Semantic
        accent: "var(--accent)",
        warn:   "var(--warn)",
        info:   "var(--info)",
        error:  "var(--error)",
      },

      fontFamily: {
        sans: ["var(--font-sans)", "Inter", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", '"JetBrains Mono"', "monospace"],
      },

      fontSize: {
        "2xs": ["0.625rem",  { lineHeight: "1rem" }],    // 10px — badges
        xs:    ["0.75rem",   { lineHeight: "1rem" }],    // 12px — captions
        sm:    ["0.8125rem", { lineHeight: "1.25rem" }], // 13px — table rows
        base:  ["0.875rem",  { lineHeight: "1.5rem" }],  // 14px — body
        md:    ["1rem",      { lineHeight: "1.5rem" }],  // 16px — subheadings
        lg:    ["1.125rem",  { lineHeight: "1.5rem" }],  // 18px
        xl:    ["1.25rem",   { lineHeight: "1.75rem" }], // 20px — section titles
        "2xl": ["1.5rem",    { lineHeight: "2rem" }],    // 24px
        "3xl": ["1.875rem",  { lineHeight: "2.25rem" }], // 30px — stat numbers
      },

      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
      },
    },
  },
  plugins: [],
};

export default config;
