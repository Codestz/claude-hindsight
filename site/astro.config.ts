import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import mdx from "@astrojs/mdx";

export default defineConfig({
  site: "https://Codestz.github.io",
  base: "/claude-hindsight",
  integrations: [tailwind(), mdx()],
  output: "static",
});
