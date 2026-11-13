// wasm_check.mjs — verifies the WASM adapter against the native CLI.
//
//   node tools/wasm_check.mjs [--wasm <path>] [--bin <path>]
//
// Defaults: the debug .wasm and the debug native binary. The Makefile
// `wasm` target builds both first. What it checks, for seeds
// 0, 7, 42, 106, 199:
//   - fp_version() is 1 and the module imports nothing.
//   - fp_house_json == the native `generate --json` text, exactly,
//     for all six styles.
//   - fp_house_mesh parses (magic, counts, table) and its vert/face
//     counts match the native `generate --obj` counts; its material
//     names match the native .mtl names, in order.
//   - fp_neighborhood_json == the native `neighborhood --json` text,
//     exactly, for 2x2 and 3x3 blocks.
//   - fp_neighborhood_mesh parses and validates (indices and
//     materials in range) and its counts equal the native per-house
//     OBJ counts plus one ground box per street and lot (2x2 only;
//     3x3 is checked for internal consistency and determinism).
//   - Every payload is byte-identical across two calls, and an
//     invalid style index returns a null pointer with length 0.
//
// Exit code 0 when every check passes, 1 otherwise.

import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const SEEDS = [0, 7, 42, 106, 199];
const STYLES = ['ranch', 'colonial', 'craftsman', 'modern', 'cottage', 'townhouse'];
const BLOCKS = ['2x2', '3x3'];

const args = process.argv.slice(2);
const opt = (name, def) => {
  const i = args.indexOf(name);
  return i >= 0 && i + 1 < args.length ? args[i + 1] : def;
};
const WASM_PATH = opt('--wasm', 'target/wasm32-unknown-unknown/debug/floorplan.wasm');
const BIN_PATH = opt('--bin', 'target/debug/floorplan');

let failures = 0;
let passes = 0;
function check(name, cond, detail = '') {
  if (cond) {
    passes++;
  } else {
    failures++;
    console.log(`FAIL: ${name}${detail ? ` — ${detail}` : ''}`);
  }
}

const bytes = readFileSync(WASM_PATH);
const mod = new WebAssembly.Module(bytes);
check('module imports nothing', WebAssembly.Module.imports(mod).length === 0);
const instance = await WebAssembly.instantiate(mod, {});
const ex = instance.exports;
const mem = () => new Uint8Array(ex.memory.buffer);
const view = () => new DataView(ex.memory.buffer);

check('fp_version is 1', ex.fp_version() === 1);

function payload(ptr) {
  const len = ex.fp_last_len();
  if (ptr === 0) return null;
  const out = mem().slice(ptr, ptr + len);
  ex.fp_free(ptr, len);
  return out;
}

function readJson(ptr) {
  const p = payload(ptr);
  return p === null ? null : new TextDecoder().decode(p);
}

/// Parses the mesh binary format (ADR 0004): returns the header
/// counts, the material names, and the raw index/material arrays.
function parseMesh(p) {
  const dv = new DataView(p.buffer, p.byteOffset, p.byteLength);
  const magic = new TextDecoder().decode(p.slice(0, 4));
  const verts = dv.getUint32(4, true);
  const faces = dv.getUint32(8, true);
  const materials = dv.getUint32(12, true);
  let off = 16 + verts * 12 + faces * 12;
  const indices = new Uint32Array(p.buffer, p.byteOffset + 16 + verts * 12, faces * 3);
  const matIds = new Uint32Array(p.buffer, p.byteOffset + off, faces);
  off += faces * 4;
  const names = [];
  for (let i = 0; i < materials; i++) {
    const len = p[off];
    off += 1;
    names.push(new TextDecoder().decode(p.slice(off, off + len)));
    off += len + 12;
  }
  return { magic, verts, faces, materials, indices, matIds, names, end: off, total: p.length };
}

const tmp = mkdtempSync(join(tmpdir(), 'fp-wasm-'));
function native(args, outFile) {
  execFileSync(BIN_PATH, args, { stdio: 'pipe' });
  return outFile ? readFileSync(outFile, 'utf8') : null;
}
function countLines(text, prefix) {
  return text.split('\n').filter((l) => l.startsWith(prefix)).length;
}

const objCounts = new Map(); // "seed:style" -> { verts, faces, mtlNames }
function nativeHouseObj(seed, style) {
  const key = `${seed}:${style}`;
  if (objCounts.has(key)) return objCounts.get(key);
  const obj = join(tmp, `h-${seed}-${style}.obj`);
  const mtl = join(tmp, `h-${seed}-${style}.mtl`);
  native(['generate', '--seed', String(seed), '--style', style, '--obj', obj]);
  const objText = readFileSync(obj, 'utf8');
  const mtlText = readFileSync(mtl, 'utf8');
  const counts = {
    verts: countLines(objText, 'v '),
    faces: countLines(objText, 'f '),
    mtlNames: mtlText.split('\n').filter((l) => l.startsWith('newmtl ')).map((l) => l.slice(7)),
  };
  objCounts.set(key, counts);
  return counts;
}

for (const seed of SEEDS) {
  for (let s = 0; s < 6; s++) {
    const style = STYLES[s];
    // House JSON: byte-identical to the native CLI text.
    const jsonPath = join(tmp, `h-${seed}-${style}.json`);
    const nativeJson = native(['generate', '--seed', String(seed), '--style', style, '--json', jsonPath], jsonPath);
    const wasmJson = readJson(ex.fp_house_json(seed, s));
    check(`house json seed ${seed} ${style}`, wasmJson === nativeJson);
    // House mesh: counts and material names match the native OBJ.
    const nativeObj = nativeHouseObj(seed, style);
    const p = payload(ex.fp_house_mesh(seed, s));
    const m = parseMesh(p);
    check(`house mesh magic seed ${seed} ${style}`, m.magic === 'FPM1');
    check(`house mesh verts seed ${seed} ${style}`, m.verts === nativeObj.verts, `${m.verts} vs ${nativeObj.verts}`);
    check(`house mesh faces seed ${seed} ${style}`, m.faces === nativeObj.faces, `${m.faces} vs ${nativeObj.faces}`);
    check(`house mesh materials seed ${seed} ${style}`, JSON.stringify(m.names) === JSON.stringify(nativeObj.mtlNames));
    check(`house mesh exact length seed ${seed} ${style}`, m.end === m.total);
    check(`house mesh indices in range seed ${seed} ${style}`, [...m.indices].every((i) => i < m.verts));
    check(`house mesh mat ids in range seed ${seed} ${style}`, [...m.matIds].every((i) => i < m.materials));
    // Determinism: the same call twice gives the same bytes.
    const p2 = payload(ex.fp_house_mesh(seed, s));
    check(`house mesh deterministic seed ${seed} ${style}`, Buffer.compare(Buffer.from(p), Buffer.from(p2)) === 0);
  }

  for (const blocks of BLOCKS) {
    const [w, h] = blocks.split('x').map(Number);
    const jsonPath = join(tmp, `n-${seed}-${blocks}.json`);
    const nativeJson = native(
      ['neighborhood', '--seed', String(seed), '--blocks', blocks, '--json', jsonPath],
      jsonPath,
    );
    const wasmJson = readJson(ex.fp_neighborhood_json(seed, w, h));
    check(`neighborhood json seed ${seed} ${blocks}`, wasmJson === nativeJson);
    const nb = JSON.parse(wasmJson);
    const p = payload(ex.fp_neighborhood_mesh(seed, w, h));
    const m = parseMesh(p);
    check(`neighborhood mesh magic seed ${seed} ${blocks}`, m.magic === 'FPM1');
    check(`neighborhood mesh materials seed ${seed} ${blocks}`, m.materials === 31, `${m.materials} vs 31`);
    check(`neighborhood mesh ground names seed ${seed} ${blocks}`, m.names[29] === 'street' && m.names[30] === 'lot');
    check(`neighborhood mesh exact length seed ${seed} ${blocks}`, m.end === m.total);
    check(`neighborhood mesh indices in range seed ${seed} ${blocks}`, [...m.indices].every((i) => i < m.verts));
    check(`neighborhood mesh mat ids in range seed ${seed} ${blocks}`, [...m.matIds].every((i) => i < m.materials));
    const p2 = payload(ex.fp_neighborhood_mesh(seed, w, h));
    check(`neighborhood mesh deterministic seed ${seed} ${blocks}`, Buffer.compare(Buffer.from(p), Buffer.from(p2)) === 0);
    if (blocks === '2x2') {
      // Counts equal the native per-house OBJ counts plus one
      // ground box (8 verts, 12 faces) per street and lot.
      let verts = 8 * (nb.streets.length + nb.lots.length);
      let faces = 12 * (nb.streets.length + nb.lots.length);
      for (const house of nb.houses) {
        const c = nativeHouseObj(house.seed, house.style);
        verts += c.verts;
        faces += c.faces;
      }
      check(`neighborhood mesh verts seed ${seed} ${blocks}`, m.verts === verts, `${m.verts} vs ${verts}`);
      check(`neighborhood mesh faces seed ${seed} ${blocks}`, m.faces === faces, `${m.faces} vs ${faces}`);
    }
  }
}

// An invalid style index is refused, not a trap.
check('invalid style returns null', ex.fp_house_json(42, 6) === 0 && ex.fp_last_len() === 0);
check('invalid mesh style returns null', ex.fp_house_mesh(42, 99) === 0 && ex.fp_last_len() === 0);

rmSync(tmp, { recursive: true, force: true });
console.log(`wasm_check: ${passes} passed, ${failures} failed`);
process.exit(failures === 0 ? 0 : 1);
