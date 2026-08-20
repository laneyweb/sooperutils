#!/usr/bin/env node
/**
 * Bumps the app version in src-tauri/tauri.conf.json and src-tauri/Cargo.toml
 * so they stay in sync. The DMG/app bundle name is derived from
 * tauri.conf.json -> version, and the About section reads it at runtime via
 * @tauri-apps/api/app getVersion().
 *
 * Usage: node scripts/bump-version.mjs [patch|minor|major]   (default: patch)
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const confPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const cargoPath = path.join(root, 'src-tauri', 'Cargo.toml');

const part = (process.argv[2] || 'patch').toLowerCase();
if (!['patch', 'minor', 'major'].includes(part)) {
  console.error('Usage: node scripts/bump-version.mjs [patch|minor|major]');
  process.exit(1);
}

const conf = JSON.parse(readFileSync(confPath, 'utf8'));
const [major, minor, patch] = conf.version.split('.').map((n) => parseInt(n, 10) || 0);

let next;
if (part === 'major') next = `${major + 1}.0.0`;
else if (part === 'minor') next = `${major}.${minor + 1}.0`;
else next = `${major}.${minor}.${patch + 1}`;

conf.version = next;
writeFileSync(confPath, JSON.stringify(conf, null, 2) + '\n');

let cargo = readFileSync(cargoPath, 'utf8');
cargo = cargo.replace(/^version = ".*"$/m, `version = "${next}"`);
writeFileSync(cargoPath, cargo);

console.log(`Version bumped ${major}.${minor}.${patch} -> ${next}`);
