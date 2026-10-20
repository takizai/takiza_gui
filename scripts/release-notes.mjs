import { readFileSync } from 'node:fs';
const version = (process.env.RELEASE_TAG ?? '').replace(/^v/, '');
if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error('RELEASE_TAG must be vX.Y.Z');
const heading = `## ${version}\n`;
const changelog = readFileSync(new URL('../CHANGELOG.md', import.meta.url), 'utf8');
if (!changelog.includes(heading)) throw new Error(`Missing changelog entry for ${version}`);
const notes = changelog.split(heading)[1].split('\n## ')[0].trim();
if (!notes) throw new Error('Empty release notes');
console.log(notes);
