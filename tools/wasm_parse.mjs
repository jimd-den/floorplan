// wasm_parse.mjs — binary mesh parser and native CLI runner for wasm_check.mjs.

import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';

/// Parses the FPM1 binary mesh format (ADR 0004): returns header counts,
/// material names, and raw index/material arrays.
export function parseMesh(p) {
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

export function makeNativeRunner(binPath, tmp) {
  const objCounts = new Map();
  function native(args, outFile) {
    execFileSync(binPath, args, { stdio: 'pipe' });
    return outFile ? readFileSync(outFile, 'utf8') : null;
  }
  function countLines(text, prefix) {
    return text.split('\n').filter((l) => l.startsWith(prefix)).length;
  }
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
  return { native, nativeHouseObj };
}
