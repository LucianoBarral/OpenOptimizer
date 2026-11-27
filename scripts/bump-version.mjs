// uso: node scripts/bump-version.mjs 0.1.1
// Fonte única da versão: app/tauri.conf.json; sincroniza ui/package.json e Cargo.toml.
import fs from 'node:fs';

const v = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(v ?? '')) {
  console.error('uso: node scripts/bump-version.mjs X.Y.Z');
  process.exit(1);
}
const json = (p, f) => {
  const o = JSON.parse(fs.readFileSync(p, 'utf8'));
  f(o);
  fs.writeFileSync(p, JSON.stringify(o, null, 2) + '\n');
};
json('app/tauri.conf.json', (o) => (o.version = v));
json('ui/package.json', (o) => (o.version = v));
const cargo = fs.readFileSync('Cargo.toml', 'utf8');
const next = cargo.replace(/(\[workspace\.package\]\s*\r?\nversion = ")[^"]+/, `$1${v}`);
if (next === cargo && !cargo.includes(`version = "${v}"`)) {
  console.error('versão do workspace não encontrada no Cargo.toml');
  process.exit(1);
}
fs.writeFileSync('Cargo.toml', next);
console.log(`versão → ${v}`);
