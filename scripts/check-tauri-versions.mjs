import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

export function checkTauriVersions(root) {
  const lock = JSON.parse(readFileSync(resolve(root, 'package-lock.json'), 'utf8'));
  const cargo = readFileSync(resolve(root, 'src-tauri/Cargo.lock'), 'utf8');
  const crates = new Map([...cargo.matchAll(/\[\[package\]\]\r?\nname = "([^"]+)"\r?\nversion = "([^"]+)"/g)]
    .map((match) => [match[1], match[2]]));
  const mismatches = [];
  for (const [crate, npm] of [['tauri', '@tauri-apps/api'],
    ['tauri-plugin-dialog', '@tauri-apps/plugin-dialog'],
    ['tauri-plugin-opener', '@tauri-apps/plugin-opener']]) {
    const rust = crates.get(crate);
    const js = lock.packages[`node_modules/${npm}`]?.version;
    if (!rust || !js || rust.split('.').slice(0, 2).join('.') !== js.split('.').slice(0, 2).join('.')) {
      mismatches.push(`${crate} ${rust ?? 'missing'} / ${npm} ${js ?? 'missing'}`);
    }
  }
  if (mismatches.length) throw new Error(`Incompatible Tauri versions:\n${mismatches.join('\n')}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  checkTauriVersions(fileURLToPath(new URL('../', import.meta.url)));
  console.log('Tauri JS and Rust major/minor versions match.');
}
