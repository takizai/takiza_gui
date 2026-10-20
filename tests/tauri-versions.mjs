import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkTauriVersions } from '../scripts/check-tauri-versions.mjs';

test('Tauri patch differences are allowed, incompatible core/plugins fail before packaging', () => {
  const dir = mkdtempSync(join(tmpdir(), 'takiza-tauri-'));
  try {
    mkdirSync(join(dir, 'src-tauri'));
    writeFileSync(join(dir, 'src-tauri/Cargo.lock'), [
      ['tauri', '2.11.6'], ['tauri-plugin-dialog', '2.7.3'], ['tauri-plugin-opener', '2.5.5'],
    ].map(([name, version]) => `[[package]]\nname = "${name}"\nversion = "${version}"\n`).join('\n'));
    const packages = Object.fromEntries([
      ['api', '2.11.1'], ['plugin-dialog', '2.7.3'], ['plugin-opener', '2.5.5'],
    ].map(([name, version]) => [`node_modules/@tauri-apps/${name}`, { version }]));
    const save = () => writeFileSync(join(dir, 'package-lock.json'), JSON.stringify({ packages }));
    save();
    assert.doesNotThrow(() => checkTauriVersions(dir));
    packages['node_modules/@tauri-apps/api'].version = '2.12.4';
    save();
    assert.throws(() => checkTauriVersions(dir), /tauri 2\.11\.6/);
    packages['node_modules/@tauri-apps/api'].version = '2.11.1';
    packages['node_modules/@tauri-apps/plugin-dialog'].version = '2.8.3';
    save();
    assert.throws(() => checkTauriVersions(dir), /tauri-plugin-dialog 2\.7\.3/);
    delete packages['node_modules/@tauri-apps/plugin-dialog'];
    save();
    assert.throws(() => checkTauriVersions(dir), /missing/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
