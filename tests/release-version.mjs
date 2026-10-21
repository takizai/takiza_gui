import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, cpSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';

test('old tags cannot downgrade a release; mismatched release tags fail before writes', () => {
  const root = new URL('../', import.meta.url);
  const expectedVersion = JSON.parse(readFileSync(new URL('package.json', root))).version;
  const dir = mkdtempSync(join(tmpdir(), 'takiza-version-'));
  try {
    mkdirSync(join(dir, 'scripts'));
    mkdirSync(join(dir, 'src-tauri'));
    for (const name of ['scripts/sync-version.mjs', 'package.json', 'package-lock.json',
      'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml', 'src-tauri/Cargo.lock']) {
      cpSync(new URL(name, root), join(dir, name));
    }
    const git = (...args) => execFileSync('git', args, { cwd: dir, stdio: 'pipe' });
    git('init');
    git('-c', 'user.name=Test', '-c', 'user.email=test@example.org', 'commit', '--allow-empty', '-m', 'old release');
    git('tag', 'v0.1.1');
    const env = { ...process.env };
    delete env.RELEASE_TAG;
    execFileSync(process.execPath, ['scripts/sync-version.mjs'], { cwd: dir, env });
    const version = JSON.parse(readFileSync(join(dir, 'package.json'))).version;
    assert.equal(version, expectedVersion);
    assert.equal(JSON.parse(readFileSync(join(dir, 'src-tauri/tauri.conf.json'))).version, version);
    const manifest = readFileSync(join(dir, 'src-tauri/Cargo.toml'), 'utf8');
    // A mismatched tag must not silently rewrite any release manifest.
    writeFileSync(join(dir, 'src-tauri/Cargo.toml'), manifest.replace(version, '0.0.1'));
    const result = spawnSync(process.execPath, ['scripts/sync-version.mjs'], {
      cwd: dir, env: { ...env, RELEASE_TAG: 'v9.9.9' }, encoding: 'utf8',
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /does not match/);
    assert.match(readFileSync(join(dir, 'src-tauri/Cargo.toml'), 'utf8'), /0\.0\.1/);
    execFileSync(process.execPath, ['scripts/sync-version.mjs'], { cwd: dir, env: { ...env, RELEASE_TAG: `v${version}` } });
    assert.equal(readFileSync(join(dir, 'src-tauri/Cargo.toml'), 'utf8'), manifest);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
