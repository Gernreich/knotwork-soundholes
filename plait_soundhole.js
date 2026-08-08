'use strict';
//
// knot sound hole -> cut-ready SVG
// =======================================
//
// WHAT IT MAKES
//   A circular plait sized for an instrument sound hole. Two identical
//   sinusoidal ribbons (B is A rotated by pi/N) weave around a ring and cross
//   2N times. Output is an SVG in millimetres, 1 user unit = 1mm, so it
//   prints and cuts at true size.
//
//   CROSSING COUNT IS ALWAYS EVEN HERE, and no parameter changes that. Both
//   ribbons are polar graphs r(theta) and a plane point has a unique (r,theta),
//   so they can only meet where rA = rB; that difference is 2*AMP*sin(N*theta),
//   which has exactly 2N zeros per turn. For an ODD number of crossings you
//   need a single self-crossing strand instead -- see the companion generator
//   knot_soundhole.js -- any coprime LEADS x BIGHTS.
//
// THE CONSTRAINT THAT DRIVES THE DESIGN
//   This is a CUT-OUT, so the removed material is the open area and the knot
//   ribbon is what stays. A ribbon ring floating inside a round hole would
//   simply drop out when the last cut closes. So the ribbon peaks are made to
//   overrun the rim by BITE mm (R_MID is derived, not chosen, to guarantee
//   this), fusing the rosette into the soundboard at 2N anchor points. There
//   is therefore NO continuous rim circle in the cut layer -- the outer
//   boundary is 2N separate arcs between anchors. That is correct, not a bug.
//
//   Because it is a single sheet cut through, the over/under of a real knot
//   cannot exist in the material: at a crossing both ribbons are material and
//   simply merge. The interlace is suggested instead by the ENGRAVE layer.
//
// HOW IT WORKS
//   signed distance field  ->  sample on a grid  ->  flood fill regions  ->
//   weld unmanufacturable slivers shut  ->  marching squares  ->  chain
//   segments into loops  ->  RDP simplify  ->  emit SVG
//
//   field(x,y) > 0 means "cut away (air)", < 0 means "material".
//
// USAGE   (all knobs are env vars; nothing is written unless OUT is set)
//   node plait_soundhole.js                       report only, no file
//   OUT=10-crossing_plait_radius30mm.svg node plait_soundhole.js
//   N=4 OUT=8-crossing_plait_radius30mm.svg node plait_soundhole.js
//   R_HOLE=50 AMP=10.8 HW=3.3 BITE=2.5 OUT=10-crossing_plait_radius50mm.svg
//   DIAG=1 node plait_soundhole.js                per-region dump
//
//   R_HOLE  sound hole radius mm      AMP   radial swing of the weave
//   HW      ribbon half-width         N     lobes per ribbon -> 2N crossings
//   BITE    rim overrun (anchoring)   NG    sampling grid resolution
//   MIN_FEATURE  narrowest cuttable gap; anything thinner is welded solid
//   OUT     output path               DIAG  verbose per-region report
//
//   AMP and HW do NOT auto-scale with R_HOLE. Roughly AMP ~ 0.217*R_HOLE and
//   HW ~ 0.067*R_HOLE reproduces the shipped proportions; the guard below
//   refuses geometry that cannot close and suggests values.
//
// OUTPUT LAYERS
//   cut       every closed path is waste that drops out.
//   engrave   over/under interlace hints and rim continuations. NOT cuts.
//
// THE VALIDATION REPORT -- read it, it is the point
//   closed contours vs flood regions   must match, else loops were lost
//   unclosed chains                    must be 0, else a contour leaked
//   loose islands (CW)                 must be 0. A clockwise contour means a
//                                      piece of material is fully surrounded
//                                      by air and will FALL OUT when cut.
//   contour area vs flood area         two independent area measurements;
//                                      a delta above ~0.1% means the polygons
//                                      do not describe the same shape as the
//                                      field they came from.
//
// Two bugs this harness caught, both silent, both worth not reintroducing:
// chaining loops on rounded float coordinates (shared cell edges disagree in
// the last bits), and running plain RDP on a closed loop (see rdpClosed).
//
const fs = require('fs');

// ---------------------------------------------------------------- params
// Every knob, in one place: --help prints this list and the defaults below are
// read from it, so the two cannot drift apart. NG and MIN_FEATURE are declared
// further down, where the sampling and welding code needs them.
const PARAMS = [
  ['N',           '5',    'lobes per ribbon -> 2N crossings and 2N rim anchors'],
  ['R_HOLE',      '30',   'sound hole radius, mm'],
  ['AMP',         '6.5',  'radial swing of the weave'],
  ['HW',          '2.0',  'ribbon half-width -> ribbon is 2*HW mm wide'],
  ['BITE',        '1.5',  'rim overrun; this is what anchors the rosette'],
  ['NG',          '1400', 'sampling grid resolution'],
  ['MIN_FEATURE', '0.25', 'gaps narrower than this are welded to solid material'],
];
const FLAGS = [
  ['OUT',  'output path -- NOTHING IS WRITTEN unless this is set'],
  ['DIAG', 'verbose per-region report'],
];

if (process.argv.slice(2).some(a => a === '--help' || a === '-h')) {
  console.log(`
plait_soundhole.js -- two woven ribbons, always an even crossing count

  Settings are environment variables, not flags:

    N=4 OUT=8-crossing_plait_radius30mm.svg node plait_soundhole.js

  Numbers (default):
`);
  PARAMS.forEach(p => console.log(`    ${p[0].padEnd(13)} ${String(p[1]).padEnd(6)} ${p[2]}`));
  console.log('\n  Switches (unset by default), any non-empty value turns them on:\n');
  FLAGS.forEach(f => console.log(`    ${f[0].padEnd(13)}        ${f[1]}`));
  console.log(`
  The crossing count is 2N and is always even -- no parameter changes that. For an
  odd count, or any coprime leads x bights, use knot_soundhole.js.

  AMP and HW do not scale with R_HOLE. Roughly AMP ~ 0.217*R_HOLE and
  HW ~ 0.067*R_HOLE reproduces the shipped proportions; a guard refuses geometry
  that cannot close and suggests values.

  Full documentation: plait_soundhole.md
`);
  process.exit(0);
}

const num = (k, d) => process.env[k] ? Number(process.env[k]) : d;
const DEF = {};
PARAMS.forEach(p => { DEF[p[0]] = Number(p[1]); });
const R_HOLE = num('R_HOLE', DEF.R_HOLE);   // sound-hole radius (mm)
const AMP    = num('AMP', DEF.AMP);         // radial swing of the weave
const HW     = num('HW', DEF.HW);           // ribbon half-width -> 4mm ribbon
const N      = num('N', DEF.N);             // lobes per ribbon -> 2N crossings
const BITE   = num('BITE', DEF.BITE);    // how far ribbon peak overruns the rim
// anchor the ribbon INTO the soundboard: peak outer edge = R_HOLE + BITE
const R_MID  = R_HOLE + BITE - AMP - HW;
const PHI    = Math.PI / N;         // rotation mapping ribbon A onto ribbon B
const NG     = num('NG', DEF.NG);

// The weave has to fit inside the hole: the ribbon troughs dip to
// R_MID - AMP - HW, and that has to leave a real opening in the middle.
const R_CENTRE = R_MID - AMP - HW;
if (R_CENTRE < 1) {
  console.error(
    `Geometry does not close: centre opening would be ${R_CENTRE.toFixed(2)}mm.\n` +
    `  R_HOLE=${R_HOLE} BITE=${BITE} AMP=${AMP} HW=${HW} -> R_MID=${R_MID.toFixed(2)}\n` +
    `  The weave needs AMP + HW well under R_HOLE + BITE. For a hole this\n` +
    `  size try AMP=${(R_HOLE * 0.217).toFixed(1)} HW=${(R_HOLE * 0.067).toFixed(1)} ` +
    `(the shipped 30mm proportions scaled).`);
  process.exit(1);
}
// smallest hole any cutter can actually make; anything narrower
// must become solid material, not a micron-wide sliver
const MIN_FEATURE = num('MIN_FEATURE', DEF.MIN_FEATURE);

// Centreline curvature at the inner trough. If this drops near HW the ribbon
// self-fills its own notch and leaves unmanufacturable slivers.
const rT = R_MID - AMP, rppT = AMP * N * N;
const RHO_TROUGH = rT * rT / Math.abs(rT - rppT);

// ------------------------------------------------- ribbon A centreline
const M = 4096;
const cx = new Float64Array(M), cy = new Float64Array(M);
for (let i = 0; i < M; i++) {
  const t = 2 * Math.PI * i / M;
  const r = R_MID + AMP * Math.sin(N * t);
  cx[i] = r * Math.cos(t);
  cy[i] = r * Math.sin(t);
}
// distA searches only a window of centreline samples around the query point's
// own angle, which is what makes the field cheap. The window must be a full
// half-period: at the tight inner troughs the nearest centreline point can sit
// a long way off in theta, and too small a window silently overestimates the
// distance (reported as extra "cut"), which shows up as phantom slivers.
const WIN = Math.ceil(M / (2 * N)) + 60;

function distA(x, y) {
  let th = Math.atan2(y, x);
  if (th < 0) th += 2 * Math.PI;
  const i0 = Math.round(th / (2 * Math.PI) * M);
  let best = Infinity;
  for (let d = -WIN; d <= WIN; d++) {
    let i = (i0 + d) % M; if (i < 0) i += M;
    const dx = x - cx[i], dy = y - cy[i];
    const s = dx * dx + dy * dy;
    if (s < best) best = s;
  }
  return Math.sqrt(best);
}
const CP = Math.cos(PHI), SP = Math.sin(PHI);
const distB = (x, y) => distA(x * CP + y * SP, -x * SP + y * CP);

// signed field: >0 = cut away (air), <0 = material
function field(x, y) {
  const d = Math.hypot(x, y);
  return Math.min(R_HOLE - d, Math.min(distA(x, y), distB(x, y)) - HW);
}

// ------------------------------------------------------------- sample
// Domain just has to extend past the rim so the border is all material;
// nothing is ever cut beyond r = R_HOLE.
const EXT = R_HOLE + 2;
const step = (2 * EXT) / NG;
const gx = i => -EXT + i * step;
const F = new Float64Array((NG + 1) * (NG + 1));
const at = (i, j) => F[j * (NG + 1) + i];
for (let j = 0; j <= NG; j++)
  for (let i = 0; i <= NG; i++)
    F[j * (NG + 1) + i] = field(gx(i), gx(j));

// ------------------------------ region analysis + manufacturability filter
// Flood fill the cut regions FIRST so sub-cutter-width slivers can be welded
// shut before contouring; otherwise they emit unmanufacturable micro-loops.
function floodRegions() {
  const lab = new Int32Array((NG + 1) * (NG + 1)).fill(-1);
  const regs = [];
  for (let j0 = 0; j0 <= NG; j0++) for (let i0 = 0; i0 <= NG; i0++) {
    const id0 = j0 * (NG + 1) + i0;
    if (F[id0] <= 0 || lab[id0] !== -1) continue;
    const rid = regs.length, st = [id0], cells = [];
    lab[id0] = rid;
    let mx = 0;
    while (st.length) {
      const id = st.pop(), i = id % (NG + 1), j = (id - i) / (NG + 1);
      cells.push(id);
      if (F[id] > mx) mx = F[id];
      const nb = [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]];
      for (const [a, b] of nb) {
        if (a < 0 || b < 0 || a > NG || b > NG) continue;
        const q = b * (NG + 1) + a;
        if (F[q] > 0 && lab[q] === -1) { lab[q] = rid; st.push(q); }
      }
    }
    regs.push({ inr: mx, cells });
  }
  return { lab, regs };
}

let { regs } = floodRegions();
let welded = 0;
for (const r of regs) {
  if (r.inr >= MIN_FEATURE) continue;       // keep: physically cuttable
  for (const id of r.cells) F[id] = -1e-9;  // weld shut: becomes material
  welded++;
}
const after = floodRegions();
const lab = after.lab;
const nreg = after.regs.length;
const inrad = after.regs.map(r => r.inr);
let cutCells = 0;
for (let k = 0; k < F.length; k++) if (F[k] > 0) cutCells++;
const floodArea = cutCells * step * step;

// --------------------------------------------------- marching squares
// corners bl,br,tr,tl = bits 1,2,4,8 ; edges 0=bottom 1=right 2=top 3=left
//
// INVARIANT: every segment is emitted [from, to] oriented so that the INSIDE
// (F > 0, air) lies on its LEFT. Two things depend on this and break silently
// if an entry is reversed:
//   - chaining works by matching one segment's end to the next one's start,
//     so a reversed entry dead-ends the walk (shows up as "unclosed chains");
//   - consistent orientation makes every cut contour come out counter-
//     clockwise, which is what lets a clockwise contour be detected as a
//     loose island of material that would fall out.
// Cases 5 and 10 are the ambiguous saddles and are resolved by the cell-centre
// value; the center-inside branch of case 5 is the one that was wrong first
// time round, so re-derive rather than trust it if you touch this.
const TABLE = {
  1: [[0, 3]], 2: [[1, 0]], 4: [[2, 1]], 8: [[3, 2]],
  3: [[1, 3]], 6: [[2, 0]], 12: [[3, 1]], 9: [[0, 2]],
  7: [[2, 3]], 14: [[3, 0]], 13: [[0, 1]], 11: [[1, 2]],
};
// Every contour vertex lies on a grid edge. Identify it by an integer edge
// id and compute its point ONCE from the canonical corner order, so both
// cells sharing an edge get a bit-identical vertex and chaining is exact.
const NH = (NG + 1) * (NG + 1);
const hId = (i, j) => j * (NG + 1) + i;            // horizontal edge (i,j)-(i+1,j)
const vId = (i, j) => NH + j * (NG + 1) + i;       // vertical   edge (i,j)-(i,j+1)
const ptCache = new Map();
function edgePoint(id) {
  let p = ptCache.get(id);
  if (p) return p;
  if (id < NH) {
    const i = id % (NG + 1), j = (id - i) / (NG + 1);
    const a = at(i, j), b = at(i + 1, j);
    p = [gx(i) + step * (a / (a - b)), gx(j)];
  } else {
    const r = id - NH, i = r % (NG + 1), j = (r - i) / (NG + 1);
    const a = at(i, j), b = at(i, j + 1);
    p = [gx(i), gx(j) + step * (a / (a - b))];
  }
  ptCache.set(id, p);
  return p;
}
// cell-local edge index -> global edge id
const gid = (e, i, j) => e === 0 ? hId(i, j) : e === 1 ? vId(i + 1, j)
                       : e === 2 ? hId(i, j + 1) : vId(i, j);

const segs = [];
for (let j = 0; j < NG; j++) for (let i = 0; i < NG; i++) {
  const f = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
  let c = 0;
  if (f[0] > 0) c |= 1; if (f[1] > 0) c |= 2;
  if (f[2] > 0) c |= 4; if (f[3] > 0) c |= 8;
  if (c === 0 || c === 15) continue;
  let list;
  if (c === 5 || c === 10) {
    const ctr = (f[0] + f[1] + f[2] + f[3]) / 4 > 0;
    if (c === 5)  list = ctr ? [[0, 1], [2, 3]] : [[0, 3], [2, 1]];
    else          list = ctr ? [[3, 0], [1, 2]] : [[1, 0], [3, 2]];
  } else list = TABLE[c];
  for (const [a, b] of list) segs.push([gid(a, i, j), gid(b, i, j)]);
}

// ------------------------------------------------------ chain segments
const byStart = new Map();
segs.forEach((s, k) => {
  if (!byStart.has(s[0])) byStart.set(s[0], []);
  byStart.get(s[0]).push(k);
});
const used = new Uint8Array(segs.length);
const loops = [];
let openChains = 0;
for (let s = 0; s < segs.length; s++) {
  if (used[s]) continue;
  const start = segs[s][0];
  const ids = [start];
  let cur = s, closed = false;
  for (;;) {
    used[cur] = 1;
    const end = segs[cur][1];
    if (end === start) { closed = true; break; }
    ids.push(end);
    const cand = byStart.get(end);
    if (!cand) break;
    const nxt = cand.find(k => !used[k]);
    if (nxt === undefined) break;
    cur = nxt;
  }
  if (!closed) { openChains++; continue; }
  if (ids.length >= 3) loops.push(ids.map(edgePoint).concat([edgePoint(start)]));
}

// ------------------------------------------------------------- RDP
function rdp(pts, eps) {
  if (pts.length < 3) return pts;
  let idx = 0, dmax = 0;
  const [ax, ay] = pts[0], [bx, by] = pts[pts.length - 1];
  const L = Math.hypot(bx - ax, by - ay) || 1;
  for (let i = 1; i < pts.length - 1; i++) {
    const d = Math.abs((bx - ax) * (ay - pts[i][1]) - (ax - pts[i][0]) * (by - ay)) / L;
    if (d > dmax) { dmax = d; idx = i; }
  }
  if (dmax > eps) {
    const l = rdp(pts.slice(0, idx + 1), eps), r = rdp(pts.slice(idx), eps);
    return l.slice(0, -1).concat(r);
  }
  return [pts[0], pts[pts.length - 1]];
}
// RDP on a CLOSED loop must not use pts[0]..pts[last] as the baseline: they
// are the same point, every perpendicular distance is 0, and the whole loop
// collapses to two points. Split at the antipode and simplify each half.
function rdpClosed(pts, eps) {
  const n = pts.length - 1;              // pts[n] === pts[0]
  if (n < 4) return pts;
  const k = n >> 1;
  const a = rdp(pts.slice(0, k + 1), eps);
  const b = rdp(pts.slice(k), eps);
  return a.slice(0, -1).concat(b);
}
const simp = loops.map(l => rdpClosed(l, 0.015));

// ------------------------------------------------------------- areas
const shoe = p => {
  let a = 0;
  for (let i = 0; i < p.length - 1; i++)
    a += p[i][0] * p[i + 1][1] - p[i + 1][0] * p[i][1];
  return a / 2;
};
const openArea = simp.reduce((s, p) => s + Math.abs(shoe(p)), 0);

// ------------------------------------------- interlace engrave lines
// Start the scan a quarter-period away from a crossing so no engrave run
// straddles the array seam and gets split into two polylines.
const SEAM = Math.round(M / (4 * N));
function offsetCurve(sign) {
  const out = [];
  for (let s0 = 0; s0 < M; s0++) {
    const i = (s0 + SEAM) % M;
    const a = (i - 1 + M) % M, b = (i + 1) % M;
    let tx = cx[b] - cx[a], ty = cy[b] - cy[a];
    const L = Math.hypot(tx, ty); tx /= L; ty /= L;
    out.push([cx[i] + sign * HW * -ty, cy[i] + sign * HW * tx]);
  }
  return out;
}
const rot = (p, s) => [p[0] * Math.cos(s * PHI) - p[1] * Math.sin(s * PHI),
                       p[0] * Math.sin(s * PHI) + p[1] * Math.cos(s * PHI)];
const crossIdx = p => {
  let th = Math.atan2(p[1], p[0]); if (th < 0) th += 2 * Math.PI;
  return Math.round(th / (Math.PI / N)) % (2 * N);
};
// Interlace convention: at a crossing you engrave the OVER strand's two edges
// continuing across the under strand's body, which reads as the under strand
// passing beneath. Crossing k sits at theta = k*pi/N; strand A is over at even
// k, B at odd k. Alternating like this is only consistent because a strand
// meets an even number of crossings (2N) on its way round, so it always
// arrives back where it started with the same over/under parity.
const engrave = [];
for (const sign of [1, -1]) {
  const base = offsetCurve(sign);
  // ribbon A edges, engraved where they cross ribbon B and A is "over"
  for (const [curve, other, wantEven] of
       [[base, distB, true], [base.map(p => rot(p, 1)), distA, false]]) {
    let run = [];
    for (const p of curve) {
      const inOther = other(p[0], p[1]) < HW;
      const k = crossIdx(p);
      const over = (k % 2 === 0) === wantEven;
      const keep = inOther && over && Math.hypot(p[0], p[1]) < R_HOLE - 0.05;
      if (keep) run.push(p);
      else { if (run.length > 4) engrave.push(rdp(run, 0.01)); run = []; }
    }
    if (run.length > 4) engrave.push(rdp(run, 0.01));
  }
}

// ------------------------------------------- rim continuations (engrave)
// Where a ribbon overruns the rim it fuses into the soundboard, and the cut layer
// stops dead at R_HOLE — past the rim there is nothing to cut. That leaves every
// anchor reading as a flat pad and the rosette looking sliced off by a circle.
// The ribbon edges do carry on out there, as far as R_HOLE + BITE, so engrave them:
// the outline continues across the anchor and the eye reads the ribbon as passing
// into the board. Both ribbons get it, A directly and B by the same rotation the
// weave uses.
//
// Purely decorative, and kept apart from the interlace lines above, whose count is
// tied to the crossings. Reported separately so the two are never confused.
const rimLines = [];
for (const sign of [1, -1]) {
  const base = offsetCurve(sign);
  for (const curve of [base, base.map(p => rot(p, 1))]) {
    let run = [];
    for (const p of curve) {
      const r = Math.hypot(p[0], p[1]);
      const keep = r >= R_HOLE - 0.05 && r <= R_HOLE + BITE + HW;
      if (keep) run.push(p);
      else { if (run.length > 2) rimLines.push(rdp(run, 0.01)); run = []; }
    }
    if (run.length > 2) rimLines.push(rdp(run, 0.01));
  }
}

// ------------------------------------------------------------- output
const f2 = n => (Math.round(n * 1000) / 1000).toString();
const dOf = p => 'M' + p.map(q => f2(q[0]) + ' ' + f2(q[1])).join('L') + 'Z';
const dOpen = p => 'M' + p.map(q => f2(q[0]) + ' ' + f2(q[1])).join('L');
const cutD = simp.map(dOf).join('');
const engD = engrave.concat(rimLines).map(dOpen).join('');

const OUT = process.env.OUT;
// The canvas has to hold the rim continuations, which follow the ribbon out to
// R_HOLE + BITE -- past the rim, where the cut layer stops. Sizing this to the cut
// alone clipped 1mm off every file. Half a millimetre on top covers the 0.1mm stroke.
const PAD  = BITE + 0.5;
const SIZE = f2(2 * (R_HOLE + PAD));
const ORG  = f2(-(R_HOLE + PAD));
const R    = f2(R_HOLE);
const svg = `<svg xmlns="http://www.w3.org/2000/svg"
     width="${SIZE}mm" height="${SIZE}mm" viewBox="${ORG} ${ORG} ${SIZE} ${SIZE}">
  <title>${2 * N}-crossing plait sound hole - ${R_HOLE}mm radius</title>
  <!-- 1 user unit = 1mm, so this prints/cuts at true size.
       Sound hole radius ${R_HOLE}mm (${2 * R_HOLE}mm diameter).
       Two interwoven ribbons ${2 * HW}mm wide, ${2 * N} crossings.
       The ribbon overruns the rim by ${BITE}mm at ${2 * N} points, fusing it
       into the soundboard, so the rosette cannot drop out when cut.
       Open area ${openArea.toFixed(0)}mm2 = ${(100 * openArea / (Math.PI * R_HOLE * R_HOLE)).toFixed(0)}% of a plain ${2 * R_HOLE}mm hole
       (acoustically equivalent to a plain round hole of
       ${(2 * Math.sqrt(openArea / Math.PI)).toFixed(1)}mm diameter).
       Narrowest cut region: ${(2 * Math.min(...inrad)).toFixed(2)}mm across. -->


  <!-- CUT: every closed path here is waste that drops out -->
  <g id="cut" fill="none" stroke="#000000" stroke-width="0.1">
    <path d="${cutD}"/>
  </g>

  <!-- ENGRAVE (optional): over/under interlace breaks -->
  <g id="engrave" fill="none" stroke="#0000ff" stroke-width="0.1"
     stroke-linecap="round">
    <path d="${engD}"/>
  </g>
</svg>
`;
if (OUT) fs.writeFileSync(OUT, svg);

// ------------------------------------------------------------- report
const discArea = Math.PI * R_HOLE * R_HOLE;
const signed = simp.map(shoe);
console.log('-- validation --');
console.log('closed contours     :', simp.length, ' flood regions:', nreg,
            simp.length === nreg ? 'OK' : '*** MISMATCH ***');
console.log('unclosed chains     :', openChains, openChains === 0 ? 'OK' : '*** LEAK ***');
console.log('slivers welded shut :', welded,
            '(< ' + MIN_FEATURE + 'mm, unmanufacturable)');
console.log('loose islands (CW)  :', signed.filter(a => a < 0).length,
            signed.every(a => a > 0) ? 'OK - nothing falls out' : '*** ISLAND ***');
console.log('contour area        :', openArea.toFixed(2),
            ' flood area:', floodArea.toFixed(2),
            ' delta:', (100 * Math.abs(openArea - floodArea) / floodArea).toFixed(2) + '%');
console.log('-- geometry --');
console.log('N,AMP,HW,R_MID     :', N, AMP, HW, R_MID.toFixed(2));
console.log('trough curv. radius:', RHO_TROUGH.toFixed(3), 'mm  (HW =', HW, ') ratio',
            (RHO_TROUGH / HW).toFixed(2));
console.log('tightest region inradius (mm):', Math.min(...inrad).toFixed(3));
console.log('largest  region inradius (mm):', Math.max(...inrad).toFixed(3));
console.log('open area (mm^2)    :', openArea.toFixed(2));
console.log('plain hole area     :', discArea.toFixed(2));
console.log('open fraction       :', (openArea / discArea * 100).toFixed(1) + '%');
console.log('equiv. round hole dia (mm):', (2 * Math.sqrt(openArea / Math.PI)).toFixed(2));
console.log('engrave polylines   :', engrave.length);
console.log('rim continuations   :', rimLines.length, ' (decorative; the ribbons crossing their anchors)');
console.log('ribbon width (mm)   :', 2 * HW);
console.log('rim anchors         :', 2 * N);
// ------------------------------------------- does it all fit on the page?
// Every other line in this report describes geometry the generator computed; none
// of them look at the document it gets written into. On 2026-08-06 the rim
// continuations reached 1mm past a canvas sized for the cut layer alone, and every
// invariant kept passing while the files were clipped. Measured from the point
// arrays rather than by re-parsing the emitted path strings, which is where the
// arithmetic is actually known.
const halfCanvas = R_HOLE + PAD;
let maxAbs = 0;
for (const group of [simp, engrave, rimLines]) {
  for (const poly of group) {
    for (const p of poly) {
      const m = Math.max(Math.abs(p[0]), Math.abs(p[1]));
      if (m > maxAbs) maxAbs = m;
    }
  }
}
const fits = maxAbs <= halfCanvas;
console.log('content vs canvas   :', maxAbs.toFixed(2) + 'mm of', halfCanvas.toFixed(2) + 'mm half-extent',
            fits ? `OK (${(halfCanvas - maxAbs).toFixed(2)}mm margin)`
                 : `CLIPPED by ${(maxAbs - halfCanvas).toFixed(2)}mm -- raise PAD`);
console.log('cut path bytes      :', cutD.length);

if (process.env.DIAG) {
  const info = [];
  for (let j = 0; j <= NG; j++) for (let i = 0; i <= NG; i++) {
    const id = j * (NG + 1) + i, L = lab[id];
    if (L < 0) continue;
    if (!info[L]) info[L] = { n: 0, mx: 0, sx: 0, sy: 0 };
    const o = info[L]; o.n++; o.sx += gx(i); o.sy += gx(j);
    if (F[id] > o.mx) o.mx = F[id];
  }
  console.log('\n-- regions (sorted by inradius) --');
  info.map((o, k) => ({
    k, area: o.n * step * step, inr: o.mx,
    r: Math.hypot(o.sx / o.n, o.sy / o.n),
    th: Math.atan2(o.sy / o.n, o.sx / o.n) * 180 / Math.PI,
  })).sort((a, b) => a.inr - b.inr).forEach(o => console.log(
    `reg${String(o.k).padStart(2)} inr=${o.inr.toFixed(4)}mm area=${o.area.toFixed(3)}mm2 ` +
    `centroid r=${o.r.toFixed(2)} th=${o.th.toFixed(1)}`));
  console.log('\ncontour areas:', simp.map(p => Math.abs(shoe(p)).toFixed(3))
    .sort((a, b) => a - b).join(' '));
}
