// wasm_check.mjs — verifies the WASM adapter against the native CLI.
//
//   node tools/wasm_check.mjs [--wasm <path>] [--bin <path>]
//
// Checks v1 house/neighborhood JSON and FPM1 meshes, v2 streaming chunks,
// M3 lights/daylight, M5 home-fixing tasks and dread steps, and M1/M6
// interactive door/lamp mesh updates. Exit code 0 when all pass.

import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { parseMesh, makeNativeRunner } from './wasm_parse.mjs';

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

check('fp_version is 4', ex.fp_version() === 4);

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

const tmp = mkdtempSync(join(tmpdir(), 'fp-wasm-'));
const { native, nativeHouseObj } = makeNativeRunner(BIN_PATH, tmp);

for (const seed of SEEDS) {
  for (let s = 0; s < 6; s++) {
    const style = STYLES[s];
    const jsonPath = join(tmp, `h-${seed}-${style}.json`);
    const nativeJson = native(['generate', '--seed', String(seed), '--style', style, '--json', jsonPath], jsonPath);
    const wasmJson = readJson(ex.fp_house_json(seed, s));
    check(`house json seed ${seed} ${style}`, wasmJson === nativeJson);
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
    const p2 = payload(ex.fp_house_mesh(seed, s));
    check(`house mesh deterministic seed ${seed} ${style}`, Buffer.compare(Buffer.from(p), Buffer.from(p2)) === 0);
  }

  for (const blocks of BLOCKS) {
    const [w, h] = blocks.split('x').map(Number);
    const jsonPath = join(tmp, `n-${seed}-${blocks}.json`);
    const nativeJson = native(['neighborhood', '--seed', String(seed), '--blocks', blocks, '--json', jsonPath], jsonPath);
    const wasmJson = readJson(ex.fp_neighborhood_json(seed, w, h));
    check(`neighborhood json seed ${seed} ${blocks}`, wasmJson === nativeJson);
    const nb = JSON.parse(wasmJson);
    const p = payload(ex.fp_neighborhood_mesh(seed, w, h));
    const m = parseMesh(p);
    check(`neighborhood mesh magic seed ${seed} ${blocks}`, m.magic === 'FPM1');
    check(
      `neighborhood mesh ground and furniture names seed ${seed} ${blocks}`,
      m.names.slice(-7).join(',') === 'street,lot,sidewalk,lamp_post,lamp_glass,sign_post,sign_face',
    );
    check(`neighborhood json has lamps seed ${seed} ${blocks}`, nb.lamps.length > 0);
    check(`neighborhood json has signs seed ${seed} ${blocks}`, nb.signs.length > 0);
    check(
      `neighborhood lamps carry lamp lights seed ${seed} ${blocks}`,
      nb.lamps.every((l) => l.light && l.light.kind === 'lamp'),
    );
    check(
      `neighborhood signs carry text seed ${seed} ${blocks}`,
      nb.signs.every((g) => typeof g.text === 'string' && g.text.length > 0),
    );
    check(`neighborhood mesh exact length seed ${seed} ${blocks}`, m.end === m.total);
    check(`neighborhood mesh indices in range seed ${seed} ${blocks}`, [...m.indices].every((i) => i < m.verts));
    check(`neighborhood mesh mat ids in range seed ${seed} ${blocks}`, [...m.matIds].every((i) => i < m.materials));
    const p2 = payload(ex.fp_neighborhood_mesh(seed, w, h));
    check(`neighborhood mesh deterministic seed ${seed} ${blocks}`, Buffer.compare(Buffer.from(p), Buffer.from(p2)) === 0);
    if (blocks === '2x2') {
      let verts = 8 * (nb.streets.length + nb.lots.length);
      let houseExpected = 0;
      for (const house of nb.houses) {
        const c = nativeHouseObj(house.seed, house.style);
        verts += c.verts;
        houseExpected += c.faces;
      }
      check(`neighborhood mesh verts seed ${seed} ${blocks}`, m.verts >= verts, `${m.verts} vs ${verts}`);
      const groundNames = new Set([
        'street', 'lot', 'driveway', 'sidewalk', 'trunk', 'leaf',
        'lamp_post', 'lamp_glass', 'sign_post', 'sign_face',
      ]);
      const houseFaces = [...m.matIds].filter((id) => !groundNames.has(m.names[id])).length;
      check(`neighborhood house faces seed ${seed} ${blocks}`, houseFaces === houseExpected, `${houseFaces} vs ${houseExpected}`);
    }
  }

  const CHUNKS = [[0, 0], [1, 0], [0, 1], [-1, 2], [3, -1]];
  const chunkPlans = [];
  for (const [cx, cz] of CHUNKS) {
    const cj = JSON.parse(readJson(ex.fp_chunk_json(seed, cx, cz)));
    check(`chunk json coords seed ${seed} (${cx},${cz})`, cj.chunk[0] === cx && cj.chunk[1] === cz);
    check(`chunk has streets seed ${seed} (${cx},${cz})`, cj.neighborhood.streets.length > 0);
    check(`chunk has lots seed ${seed} (${cx},${cz})`, cj.neighborhood.lots.length > 0);
    check(`chunk has lamps seed ${seed} (${cx},${cz})`, cj.neighborhood.lamps.length > 0);
    check(`chunk has signs seed ${seed} (${cx},${cz})`, cj.neighborhood.signs.length > 0);
    check(`chunk houses match lots seed ${seed} (${cx},${cz})`, cj.neighborhood.houses.length === cj.neighborhood.lots.length);
    chunkPlans.push(JSON.stringify(cj.neighborhood));
    const cp = payload(ex.fp_chunk_mesh(seed, cx, cz));
    const cm = parseMesh(cp);
    check(`chunk mesh magic seed ${seed} (${cx},${cz})`, cm.magic === 'FPM1');
    check(`chunk mesh exact length seed ${seed} (${cx},${cz})`, cm.end === cm.total);
    check(`chunk mesh indices in range seed ${seed} (${cx},${cz})`, [...cm.indices].every((i) => i < cm.verts));
    check(`chunk mesh mat ids in range seed ${seed} (${cx},${cz})`, [...cm.matIds].every((i) => i < cm.materials));
    const cp2 = payload(ex.fp_chunk_mesh(seed, cx, cz));
    check(`chunk mesh deterministic seed ${seed} (${cx},${cz})`, Buffer.compare(Buffer.from(cp), Buffer.from(cp2)) === 0);
  }
  check(`chunks are distinct seed ${seed}`, new Set(chunkPlans).size === CHUNKS.length);

  const lights = JSON.parse(readJson(ex.fp_house_lights(seed, 3)));
  check(`house lights have a flashlight seed ${seed}`, lights.some((l) => l.kind === 'flashlight'));
  check(`house lights have a ceiling seed ${seed}`, lights.some((l) => l.kind === 'ceiling'));
  const ceiling = lights.find((l) => l.kind === 'ceiling');
  const toggled = JSON.parse(readJson(ex.fp_house_lights_toggled(seed, 3, ceiling.id)));
  const after = toggled.find((l) => l.id === ceiling.id);
  check(`toggled light changes state seed ${seed}`, after.state !== ceiling.state);
  check(`daylight noon is day seed ${seed}`, JSON.parse(readJson(ex.fp_daylight(720, 1))).phase === 'day');
  check(`daylight midnight is night seed ${seed}`, JSON.parse(readJson(ex.fp_daylight(0, 0))).phase === 'night');
  check(`light toggle codes seed ${seed}`, ex.fp_light_toggle_state(0) === 1 && ex.fp_light_toggle_state(9) === -1);

  const tasks = JSON.parse(readJson(ex.fp_house_tasks(seed, 3)));
  check(`house tasks generated seed ${seed}`, tasks.length > 0 && tasks.every((t) => t.done === false));
  const calm = JSON.parse(readJson(ex.fp_haunt_step(seed, 3, 0, 0, ceiling.room, 0, 0, 1, 100, 0)));
  check(`haunt dial 0 is calm seed ${seed}`, calm.level === 0 && calm.events.length === 0);
  const scare = JSON.parse(readJson(ex.fp_haunt_step(seed, 3, 100, 30, ceiling.room, 0, 0, 1, 100, 0)));
  check(`haunt dial 100 raises dread seed ${seed}`, scare.level > 30 && scare.events.length > 0);
  const doorTask = tasks.find((t) => t.kind === 'close_doors');
  if (doorTask) {
    const baseHouse = payload(ex.fp_house_mesh(seed, 3));
    const openedHouse = payload(ex.fp_house_interact_mesh(seed, 3, 0, doorTask.target, 0, 0));
    check(`interact open door updates mesh seed ${seed}`, openedHouse !== null && Buffer.compare(Buffer.from(baseHouse), Buffer.from(openedHouse)) !== 0);
  }
}

check('invalid style returns null', ex.fp_house_json(42, 6) === 0 && ex.fp_last_len() === 0);
check('invalid mesh style returns null', ex.fp_house_mesh(42, 99) === 0 && ex.fp_last_len() === 0);
check('invalid light returns null', ex.fp_house_lights_toggled(42, 3, 99999) === 0 && ex.fp_last_len() === 0);
check('invalid interact returns null', ex.fp_house_interact_mesh(42, 3, 0, 99999, 0, 0) === 0 && ex.fp_last_len() === 0);

rmSync(tmp, { recursive: true, force: true });
console.log(`wasm_check: ${passes} passed, ${failures} failed`);
process.exit(failures === 0 ? 0 : 1);
