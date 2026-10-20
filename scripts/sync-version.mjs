import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (name) => readFileSync(resolve(root, name), 'utf8');
const write = (name, value) => {
  if (read(name) !== value) writeFileSync(resolve(root, name), value);
};

// The manifest is authoritative during development and in source archives.
// A release tag validates it; an older reachable tag must never downgrade it.
const tag = process.env.RELEASE_TAG;
const version = JSON.parse(read('package.json')).version;
if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error(`Invalid version tag: ${tag ?? version}. Expected vX.Y.Z or X.Y.Z.`);
}
if (tag && tag.replace(/^v/, '') !== version) {
  throw new Error(`Release tag ${tag} does not match package.json version ${version}.`);
}

for (const name of ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']) {
  const value = JSON.parse(read(name));
  value.version = version;
  if (name === 'package-lock.json') value.packages[''].version = version;
  write(name, JSON.stringify(value, null, 2) + '\n');
}
write('src-tauri/Cargo.toml', read('src-tauri/Cargo.toml').replace(
  /(\[package\][\s\S]*?^version\s*=\s*)"[^"]*"/m, `$1"${version}"`,
));
write('src-tauri/Cargo.lock', read('src-tauri/Cargo.lock').replace(
  /(\[\[package\]\]\r?\nname = "takiza-gui"\r?\nversion = )"[^"]*"/, `$1"${version}"`,
));
console.log(`Takiza version: ${version}${tag ? ` (${tag})` : ''}`);
