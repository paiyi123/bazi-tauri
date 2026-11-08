// Builds and runs the app's Rust charting core without Tauri.
// Copies src-tauri/src into .build/, strips the #[tauri::command] attributes,
// and compiles a small CLI (bazi_core) that reads a request from stdin.
// Rebuilds only when the Rust sources change.
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const TOOL_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO = resolve(TOOL_DIR, "..", "..");
const BUILD = join(TOOL_DIR, ".build");
const RUST_SRC = join(REPO, "src-tauri", "src");
const SKIP = new Set(["main.rs", "lib.rs", "print_template.rs"]);
const EXE = process.platform === "win32" ? "bazi_core.exe" : "bazi_core";

const BIN = `use bazi_lib::bazi::*;
use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("read stdin");
    let value: serde_json::Value = serde_json::from_str(&input).expect("request JSON");
    let mode = value.get("mode").and_then(|m| m.as_str()).unwrap_or("calculate").to_string();
    let request = value.get("request").cloned().expect("request field");
    let result = if mode == "pillars" {
        analyze_pillars(serde_json::from_value(request).expect("PillarAnalyzeRequest"))
    } else {
        calculate_bazi(serde_json::from_value(request).expect("BaziRequest"))
    };
    match result {
        Ok(response) => println!("{}", serde_json::to_string(&response).unwrap()),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(2);
        }
    }
}
`;

function listRust(dir, base = dir, out = []) {
  for (const name of readdirSync(dir).sort()) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) listRust(full, base, out);
    else if (name.endsWith(".rs")) out.push(full.slice(base.length + 1));
  }
  return out;
}

function cargoManifest() {
  const toml = readFileSync(join(REPO, "src-tauri", "Cargo.toml"), "utf8").replace(/\r\n/g, "\n");
  const section = toml.split(/^\[dependencies\]\s*$/m)[1]?.split(/^\[/m)[0] ?? "";
  const deps = section
    .split("\n")
    .filter((line) => line.trim() && !/^\s*tauri/.test(line))
    .join("\n");
  return `[package]\nname = "bazi_lib"\nversion = "0.0.0"\nedition = "2021"\npublish = false\n\n[workspace]\n\n[dependencies]\n${deps}\n`;
}

function sourceHash(files, manifest) {
  const hash = createHash("sha256").update(manifest);
  for (const file of files) hash.update(file).update(readFileSync(join(RUST_SRC, file)));
  return hash.digest("hex");
}

/** Copies the Rust core into .build and compiles it when the sources changed. Returns the binary path. */
export function ensureCore({ quiet = true } = {}) {
  const files = listRust(RUST_SRC).filter((file) => !SKIP.has(file));
  const manifest = cargoManifest();
  const hash = sourceHash(files, manifest + BIN);
  const exe = join(BUILD, "target", "release", EXE);
  const stamp = join(BUILD, "source.hash");
  if (existsSync(exe) && existsSync(stamp) && readFileSync(stamp, "utf8") === hash) return exe;

  rmSync(join(BUILD, "src"), { recursive: true, force: true });
  mkdirSync(join(BUILD, "src", "bin"), { recursive: true });
  const modules = [];
  for (const file of files) {
    const target = join(BUILD, "src", file);
    mkdirSync(dirname(target), { recursive: true });
    const text = readFileSync(join(RUST_SRC, file), "utf8").replace(/^[ \t]*#\[tauri::command\][ \t]*\r?\n/gm, "");
    writeFileSync(target, text);
    if (!file.includes("/") && !file.includes("\\")) modules.push(file.replace(/\.rs$/, ""));
  }
  writeFileSync(join(BUILD, "src", "lib.rs"), modules.map((name) => `pub mod ${name};`).join("\n") + "\n");
  writeFileSync(join(BUILD, "src", "bin", "bazi_core.rs"), BIN);
  writeFileSync(join(BUILD, "Cargo.toml"), manifest);
  const lock = join(REPO, "src-tauri", "Cargo.lock");
  if (existsSync(lock) && !existsSync(join(BUILD, "Cargo.lock"))) cpSync(lock, join(BUILD, "Cargo.lock"));

  const build = spawnSync("cargo", ["build", "--release", "--bin", "bazi_core", "--quiet"], {
    cwd: BUILD,
    encoding: "utf8",
    env: { ...process.env, RUSTFLAGS: `${process.env.RUSTFLAGS ?? ""} -A warnings`.trim() },
    stdio: quiet ? ["ignore", "pipe", "pipe"] : "inherit",
    shell: process.platform === "win32",
  });
  if (build.status !== 0) {
    throw new Error(`cargo build failed:\n${build.stderr ?? ""}`);
  }
  writeFileSync(stamp, hash);
  return exe;
}

/** Runs calculate_bazi (mode "calculate") or analyze_pillars (mode "pillars"). */
export function runCore(mode, request) {
  const exe = ensureCore();
  const output = execFileSync(exe, [], {
    input: JSON.stringify({ mode, request }),
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return JSON.parse(output);
}
