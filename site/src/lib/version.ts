import { readFileSync } from "fs";
import { resolve } from "path";

const cargo = readFileSync(resolve(process.cwd(), "../Cargo.toml"), "utf-8");
const match = cargo.match(/^version\s*=\s*"([^"]+)"/m);

export const VERSION = match?.[1] ?? "0.0.0";
