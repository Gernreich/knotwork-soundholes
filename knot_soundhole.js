#!/usr/bin/env node
// =====================================================================
// coprime lead-and-bight knot sound hole -> cut-ready SVG
// =====================================================================
//
// Replaces an earlier two-lead-only generator, generalising it to any number of
// leads. A knot of this family is described as L leads by B bights: the strand
// travels L times around before closing, and shows B scallops at the rim.
//
//     r(theta) = R_MID + AMP*cos(B*theta/L),   theta in [0, 2*pi*L)
//
// It closes into ONE strand iff gcd(L,B) = 1. That is the whole constraint,
// and it is the same rule a knot-tyer uses. L=2 reproduces the sibling exactly:
// B odd is precisely gcd(2,B)=1, and cos(B*theta/2) over [0,4pi) is its curve.
//
// INVARIANTS -- these pin the topology and are checked in the report
//   crossings      == B*(L-1)
//   cut regions    == L*B + 1     (one centre + B*(L-1) lenses + B rim gaps)
//   rim anchors    == B
//   engrave lines  == 2*B*(L-1)   (two edges of the over pass per crossing)
//
// Both existing generators are the L=2 row of that table: the knot at
// (2,B odd) gives 2B+1 regions, and the plait at (2,2N) gives 4N+1.
//
// OVER/UNDER IS COMPUTED, NOT ASSUMED
//   The sibling could use a closed form ("over if m is even") because with two
//   leads the other pass sits exactly half the sample array away. With L leads
//   a point has L-1 other passes, at offsets k*M/L, so that shortcut is gone.
//   Crossing events are found numerically, ordered along the strand, and
//   labelled alternately. The pairing is then VERIFIED: each crossing must
//   receive one over and one under. If it does not, the engrave layer is
//   dropped and the report says so, rather than shipping a weave that lies.
//
// THE STRUCTURAL CONSTRAINT (same as both siblings)
//   This is a cut-out: removed material is the open area, the ribbon stays.
//   The ribbon peaks overrun the rim by BITE mm, fusing the rosette into the
//   soundboard at B anchor points, so there is no continuous rim circle in the
//   cut layer -- the outer boundary is B arcs. Correct, not a bug.
//
// USAGE   (nothing is written unless OUT is set)
//   node knot_soundhole.js                    report only
//   LEADS=3 BIGHTS=5 OUT=3-lead_5-bight_knot_radius30mm.svg node knot_soundhole.js
//   LEADS=2 BIGHTS=3 node knot_soundhole.js    2 leads: the classic trefoil
//   SELFTEST=1 node knot_soundhole.js    check the spatial hash, exit
//   DIAG=1 node knot_soundhole.js        per-region dump
//
//   LEADS BIGHTS R_HOLE AMP HW BITE NG MIN_FEATURE OUT DIAG SELFTEST
//
//   AMP and HW do NOT auto-scale with R_HOLE; roughly AMP ~ 0.25*R_HOLE and
//   HW ~ 0.067*R_HOLE. Higher L packs more ribbon into the same annulus, so
//   expect to reduce AMP and HW as leads go up.
//
// TESTED ENVELOPE -- and where it stops
//   Verified against every invariant at (L,B) = (2,3) (2,5) (2,9) (3,2) (3,4)
//   (3,5) and (4,3): regions, crossings, engrave lines and anchors all as
//   predicted, weave alternating, no loose islands. (2,3) reproduces
//   the retired two-lead generator exactly -- identical validation block,
//   areas and sliver count -- the check that the generalisation is faithful.
//
//   It degrades above that, and the limit is manufacturability rather than
//   topology. As L rises the lens regions between passes shrink below
//   MIN_FEATURE and are welded shut, so they never become separate cut
//   regions. At (4,5) the region count falls short of L*B+1 by exactly the
//   welded lenses; by (5,3) the crossing detector loses them too. Reducing
//   AMP and HW does not rescue it -- the passes are close because there are
//   five of them in one annulus, not because the ribbon is fat.
//
//   JUDGE BY THE REGION COUNT, NOT THE SLIVER COUNT. Slivers is an artifact
//   of where the sampling grid falls: the trefoil reports 8 of them at NG=700
//   and 4 at NG=1400 with identical geometry, contours and area. The region
//   count is stable across resolutions and tells the truth -- (5,3) reports
//   13 against a predicted 16 at NG 1400, 2400 and 3600 alike, because those
//   three lenses really are below the cutting floor. So: trust an (L,B) whose
//   region count matches L*B+1 and whose crossing count matches B*(L-1).
//   Both are printed and checked.
//
// OUTPUT LAYERS   cut / engrave
//   Only the black #000000 cut layer goes through the material. The blue
//   #0000ff engrave lines cross the ribbon; cutting them severs it.
//   Black is the last cut across these repositories, and this file has only
//   one; blue means engrave everywhere and never cuts.
//
const fs = require('fs');

// ---------------------------------------------------------------- params
// Every knob, in one place: --help prints this list and the defaults below are
// read from it, so the two cannot drift apart. Add a parameter here and it is
// documented by construction.
const PARAMS = [
  ['LEADS',       '3',    'times round before the strand closes'],
  ['BIGHTS',      '5',    'scallops at the rim; must be coprime to LEADS'],
  ['R_HOLE',      '30',   'sound hole radius, mm'],
  ['AMP',         '7.5',  'radial swing of the weave'],
  ['HW',          '2.0',  'ribbon half-width -> ribbon is 2*HW mm wide'],
  ['BITE',        '1.5',  'rim overrun; this is what anchors the rosette'],
  ['NG',          '1400', 'sampling grid resolution'],
  ['MIN_FEATURE', '0.25', 'gaps narrower than this are welded to solid material'],
];
const FLAGS = [
  ['OUT',      'output path -- NOTHING IS WRITTEN unless this is set'],
  ['SELFTEST', 'verify the spatial hash against brute force, then exit'],
  ['DIAG',     'verbose per-region report'],
];

if (process.argv.slice(2).some(a => a === '--help' || a === '-h')) {
  console.log(`
knot_soundhole.js -- one continuous strand, any coprime leads x bights

  Settings are environment variables, not flags:

    LEADS=3 BIGHTS=5 OUT=3-lead_5-bight_knot_radius30mm.svg node knot_soundhole.js

  Numbers (default):
`);
  PARAMS.forEach(p => console.log(`    ${p[0].padEnd(13)} ${String(p[1]).padEnd(6)} ${p[2]}`));
  console.log('\n  Switches (unset by default), any non-empty value turns them on:\n');
  FLAGS.forEach(f => console.log(`    ${f[0].padEnd(13)}        ${f[1]}`));
  console.log(`
  LEADS and BIGHTS must be coprime -- gcd(L,B) is the number of separate strands,
  so anything but 1 gives several loops rather than one knot.

  AMP and HW do not scale with R_HOLE. Roughly AMP ~ 0.25*R_HOLE and
  HW ~ 0.067*R_HOLE at two or three leads; above that the ribbon must thin and
  AMP/HW has to grow with LEADS -- about 4 at four leads, 6 at five, 16 at nine.
  AMP = LEADS*HW is not enough on its own. The panel has to grow too, or the
  rim gaps weld shut.

  Full documentation: knot_soundhole.md
`);
  process.exit(0);
}

const num = (k, d) => process.env[k] ? Number(process.env[k]) : d;
const DEF = {};
PARAMS.forEach(p => { DEF[p[0]] = Number(p[1]); });
const R_HOLE = num('R_HOLE', DEF.R_HOLE);   // sound-hole radius (mm)
const L      = num('LEADS', DEF.LEADS);     // leads: times round before closing
const B      = num('BIGHTS', DEF.BIGHTS);   // bights: scallops at the rim
const AMP    = num('AMP', DEF.AMP);         // radial swing of the weave
const HW     = num('HW', DEF.HW);           // ribbon half-width
const BITE   = num('BITE', DEF.BITE);       // how far ribbon peak overruns the rim
const NG     = num('NG', DEF.NG);
const MIN_FEATURE = num('MIN_FEATURE', DEF.MIN_FEATURE);

const gcd = (a, b) => b ? gcd(b, a % b) : a;

// Q was the retired two-lead generator's bight count. Silently ignoring it would
// hand back the defaults and call them the answer -- Q=5 would give 3x5, not the
// cinquefoil asked for. Refuse and say what to type instead.
if (process.env.Q !== undefined) {
  console.error(
    `Q is not a setting here (got Q=${process.env.Q}).\n` +
    `  It was the bight count of the two-lead generator this replaced.\n` +
    `  That one was always 2 leads, so the same shape is:\n` +
    `    LEADS=2 BIGHTS=${process.env.Q} node knot_soundhole.js`);
  process.exit(1);
}

if (!Number.isInteger(L) || L < 2) {
  console.error(`LEADS must be an integer >= 2 (got ${L}).`);
  process.exit(1);
}
if (!Number.isInteger(B) || B < 2) {
  console.error(`BIGHTS must be an integer >= 2 (got ${B}).`);
  process.exit(1);
}
if (gcd(L, B) !== 1) {
  const g = gcd(L, B);
  console.error(
    `LEADS and BIGHTS must be coprime (got ${L} and ${B}, gcd = ${g}).\n` +
    `  gcd(L,B) is the number of separate strands: with gcd = ${g} this closes\n` +
    `  into ${g} pieces, not one, so there is no single ribbon to trace and no\n` +
    `  interlace to alternate over. Nearby coprime bight counts for ${L} leads:\n` +
    `  ${[...Array(16).keys()].map(i => i + 2).filter(b => gcd(L, b) === 1 && b !== L).join(' ')}\n` +
    `  For an even count of crossings on two leads use plait_soundhole.js.`);
  process.exit(1);
}

// anchor the ribbon INTO the soundboard: peak outer edge = R_HOLE + BITE
const R_MID    = R_HOLE + BITE - AMP - HW;
const R_CENTRE = R_MID - AMP - HW;
if (R_CENTRE < 1) {
  console.error(
    `Geometry does not close: centre opening would be ${R_CENTRE.toFixed(2)}mm.\n` +
    `  R_HOLE=${R_HOLE} BITE=${BITE} AMP=${AMP} HW=${HW} -> R_MID=${R_MID.toFixed(2)}\n` +
    `  Try AMP=${(R_HOLE * 0.25).toFixed(1)} HW=${(R_HOLE * 0.067).toFixed(1)}.`);
  process.exit(1);
}

// ------------------------------------------- centreline: ONE strand, 2 turns
// M must be EVEN so that i + M/2 is exactly the same theta one turn later,
// which is how the "other pass" at a crossing is located. Distance is measured
// to these SAMPLES, not to the true curve, so it overestimates by up to half
// the sample spacing. M points now cover L turns rather than 2, so spacing
// grows with L as well as with B and the AMP*B/L radial derivative. It stays
// well under the 0.25mm manufacturability floor over the tested range, but
// the margin narrows as either number rises -- and long before sampling
// becomes the limit, MIN_FEATURE does: see the envelope note in the header.
const M = 20000;
const cx = new Float64Array(M), cy = new Float64Array(M);
for (let i = 0; i < M; i++) {
  const t = 2 * Math.PI * L * i / M;
  const r = R_MID + AMP * Math.cos(B * t / L);
  cx[i] = r * Math.cos(t);
  cy[i] = r * Math.sin(t);
}

// --------------------------------------------------- spatial hash for dist
// Two radii per angle here, so no angular-window shortcut. Bucket the
// centreline samples into a uniform grid and do an expanding-ring search.
const EXT  = R_HOLE + 2;
// CELL trades bucket count against points-per-bucket. Correctness does not
// depend on it -- the ring search is exact for any positive CELL -- only speed
// does. ~1.5mm keeps buckets to a handful of samples while the typical query
// still terminates within a ring or two.
const CELL = 1.5;
const NCX  = Math.ceil((2 * EXT) / CELL);
const cellOf = v => Math.max(0, Math.min(NCX - 1, Math.floor((v + EXT) / CELL)));
const buckets = new Array(NCX * NCX);
for (let i = 0; i < M; i++) {
  const k = cellOf(cy[i]) * NCX + cellOf(cx[i]);
  (buckets[k] || (buckets[k] = [])).push(i);
}

function distCurve(x, y) {
  const ci = cellOf(x), cj = cellOf(y);
  let best = Infinity;                              // squared
  for (let ring = 0; ring < NCX; ring++) {
    for (let dj = -ring; dj <= ring; dj++) {
      const b = cj + dj;
      if (b < 0 || b >= NCX) continue;
      const edge = (Math.abs(dj) === ring);
      for (let di = -ring; di <= ring; di++) {
        // only the shell at Chebyshev distance == ring
        if (!edge && Math.abs(di) !== ring) continue;
        const a = ci + di;
        if (a < 0 || a >= NCX) continue;
        const arr = buckets[b * NCX + a];
        if (!arr) continue;
        for (let n = 0; n < arr.length; n++) {
          const i = arr[n], dx = x - cx[i], dy = y - cy[i];
          const s = dx * dx + dy * dy;
          if (s < best) best = s;
        }
      }
    }
    // A point in an unvisited cell (Chebyshev ring >= ring+1) is at least
    // ring*CELL away, so once best beats that no further ring can improve it.
    const bound = ring * CELL;
    if (best <= bound * bound) break;
  }
  return Math.sqrt(best);
}

// SELFTEST=1 checks the hash against brute force. The expanding-ring stopping
// bound is the one thing here that fails SILENTLY if it is wrong -- it would
// just return distances that are too large, i.e. phantom extra "cut" area.
if (process.env.SELFTEST) {
  const brute = (x, y) => {
    let b = Infinity;
    for (let i = 0; i < M; i++) {
      const dx = x - cx[i], dy = y - cy[i];
      const s = dx * dx + dy * dy;
      if (s < b) b = s;
    }
    return Math.sqrt(b);
  };
  let worst = 0, n = 0;
  for (let k = 0; k < 4000; k++) {
    const x = (Math.random() * 2 - 1) * EXT, y = (Math.random() * 2 - 1) * EXT;
    const e = Math.abs(distCurve(x, y) - brute(x, y));
    if (e > worst) worst = e;
    n++;
  }
  console.log(`SELFTEST spatial hash vs brute force: ${n} points, ` +
              `max error ${worst.toExponential(3)}mm ` +
              `${worst < 1e-9 ? 'OK' : '*** HASH IS WRONG ***'}`);
  process.exit(worst < 1e-9 ? 0 : 1);
}

// signed field: >0 = cut away (air), <0 = material
function field(x, y) {
  return Math.min(R_HOLE - Math.hypot(x, y), distCurve(x, y) - HW);
}

// ------------------------------------------------------------- sample
const step = (2 * EXT) / NG;
const gx = i => -EXT + i * step;
const F = new Float64Array((NG + 1) * (NG + 1));
const at = (i, j) => F[j * (NG + 1) + i];
for (let j = 0; j <= NG; j++)
  for (let i = 0; i <= NG; i++)
    F[j * (NG + 1) + i] = field(gx(i), gx(j));

// ------------------------------ region analysis + manufacturability filter
// Flood fill FIRST so sub-cutter-width slivers can be welded shut before
// contouring; otherwise they emit unmanufacturable micro-loops.
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
  if (r.inr >= MIN_FEATURE) continue;
  for (const id of r.cells) F[id] = -1e-9;
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
// INVARIANT: every segment is emitted [from, to] oriented so the INSIDE
// (F > 0, air) lies on its LEFT. Chaining matches one segment's end to the
// next one's start, so a reversed entry dead-ends the walk; and consistent
// orientation is what makes every cut contour counter-clockwise, which is how
// a clockwise contour is detected as a loose island that would fall out.
const TABLE = {
  1: [[0, 3]], 2: [[1, 0]], 4: [[2, 1]], 8: [[3, 2]],
  3: [[1, 3]], 6: [[2, 0]], 12: [[3, 1]], 9: [[0, 2]],
  7: [[2, 3]], 14: [[3, 0]], 13: [[0, 1]], 11: [[1, 2]],
};
// Every contour vertex lies on a grid edge. Identify it by an integer edge id
// and compute its point ONCE from the canonical corner order, so both cells
// sharing an edge get a bit-identical vertex and chaining is exact. (Keying
// on rounded float coordinates silently fragments loops.)
const NH = (NG + 1) * (NG + 1);
const hId = (i, j) => j * (NG + 1) + i;
const vId = (i, j) => NH + j * (NG + 1) + i;
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
// RDP on a CLOSED loop must not use pts[0]..pts[last] as its baseline: they
// are the same point, every perpendicular distance is 0, and the loop
// collapses to two points. Split at the antipode and simplify each half.
function rdpClosed(pts, eps) {
  const n = pts.length - 1;
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
// With L leads a point has L-1 other passes, at parameter offsets k*M/L, so
// the sibling's closed form ("over if m is even", valid when the only other
// pass sits half the array away) does not survive the generalisation.
// Instead: find the crossing events numerically, order them along the strand,
// label them alternately, then VERIFY that the two visits to each crossing got
// opposite labels. An alternating weave exists only if that verification
// passes; if it fails the engrave layer is dropped rather than drawn wrong.
const STEP = M / L;                                 // one lead, in samples
const WIN  = Math.ceil(M / (2 * B * (L - 1)));      // half a crossing spacing

// distance from (x,y) to the nearest point on any OTHER pass of the strand
function distOtherPass(x, y, i) {
  let best = Infinity;
  for (let k = 1; k < L; k++) {
    const base = i + k * STEP;
    for (let d = -WIN; d <= WIN; d++) {
      let j = Math.round(base + d) % M; if (j < 0) j += M;
      const dx = x - cx[j], dy = y - cy[j];
      const q = dx * dx + dy * dy;
      if (q < best) best = q;
    }
  }
  return Math.sqrt(best);
}

// how close the centreline comes to another pass, per sample
const dOther = new Float64Array(M);
for (let i = 0; i < M; i++) dOther[i] = distOtherPass(cx[i], cy[i], i);

// A crossing shows as a run of small dOther; its minimum is the event. Start
// the scan at the sample FURTHEST from any other pass, so no run straddles
// index 0 and gets split in two -- the same class of trap as the sibling's
// engrave seam, avoided by construction rather than by an offset.
let scan0 = 0;
for (let i = 1; i < M; i++) if (dOther[i] > dOther[scan0]) scan0 = i;
const events = [];
{
  const THRESH = 2 * HW;
  let run = null;
  for (let s = 0; s < M; s++) {
    const i = (scan0 + s) % M;
    if (dOther[i] < THRESH) {
      if (!run) run = { at: i, d: dOther[i] };
      else if (dOther[i] < run.d) { run.at = i; run.d = dOther[i]; }
    } else if (run) { events.push(run.at); run = null; }
  }
  if (run) events.push(run.at);
}
// The scan already visited events in strand order starting from scan0, so the
// list is in the order the strand meets them; alternate along it.
const walk = events.slice();
const label = new Map();              // sample index -> true = over
walk.forEach((i, j) => label.set(i, j % 2 === 0));

// VERIFY: pair events by position; the two visits to one crossing must differ
let crossings = 0, weaveOK = true, unpaired = 0;
{
  const used = new Array(walk.length).fill(false);
  for (let j = 0; j < walk.length; j++) {
    if (used[j]) continue;
    let m = -1, md = Infinity;
    for (let k = 0; k < walk.length; k++) {
      if (k === j || used[k]) continue;
      const d = Math.hypot(cx[walk[j]] - cx[walk[k]], cy[walk[j]] - cy[walk[k]]);
      if (d < md) { md = d; m = k; }
    }
    if (m >= 0 && md < HW) {
      used[j] = used[m] = true; crossings++;
      if (label.get(walk[j]) === label.get(walk[m])) weaveOK = false;
    } else { used[j] = true; unpaired++; }
  }
}

// nearest event to a sample, for deciding which pass we are on
function overAt(i) {
  let best = walk[0], bd = Infinity;
  for (const e of walk) {
    let d = Math.abs(e - i); d = Math.min(d, M - d);
    if (d < bd) { bd = d; best = e; }
  }
  return label.get(best);
}

const engrave = [];
if (weaveOK) {
  for (const sign of [1, -1]) {
    let run = [];
    for (let s0 = 0; s0 < M; s0++) {
      const i = (s0 + scan0) % M;
      const a = (i - 1 + M) % M, b2 = (i + 1) % M;
      let tx = cx[b2] - cx[a], ty = cy[b2] - cy[a];
      const seg = Math.hypot(tx, ty); tx /= seg; ty /= seg;
      const px = cx[i] + sign * HW * -ty, py = cy[i] + sign * HW * tx;
      const keep = overAt(i) && distOtherPass(px, py, i) < HW &&
                   Math.hypot(px, py) < R_HOLE - 0.05;
      if (keep) run.push([px, py]);
      else { if (run.length > 4) engrave.push(rdp(run, 0.01)); run = []; }
    }
    if (run.length > 4) engrave.push(rdp(run, 0.01));
  }
}

// ------------------------------------------- rim continuations (engrave)
// Where the ribbon overruns the rim it fuses into the soundboard, and the cut
// layer stops dead at R_HOLE — which leaves the anchors reading as flat pads and
// the whole rosette looking chopped at the edge. The ribbon's own edges do carry
// on out there, as far as R_HOLE + BITE; the cut simply cannot show them, because
// beyond the rim there is nothing to cut. Engraving them puts the ribbon's outline
// back across the anchor, so the eye reads it as passing into the board rather than
// being sliced off by a circle.
//
// Purely decorative, and deliberately kept apart from the interlace lines above:
// those are derived from crossings and their count is a topological check. These
// are counted and reported separately so the two are never confused.
const rimLines = [];
for (const sign of [1, -1]) {
  let run = [];
  for (let s0 = 0; s0 < M; s0++) {
    const i = (s0 + scan0) % M;
    const a = (i - 1 + M) % M, b2 = (i + 1) % M;
    let tx = cx[b2] - cx[a], ty = cy[b2] - cy[a];
    const seg = Math.hypot(tx, ty); tx /= seg; ty /= seg;
    const px = cx[i] + sign * HW * -ty, py = cy[i] + sign * HW * tx;
    // outside the rim, and outside only because of the overrun — never the bore
    const r = Math.hypot(px, py);
    const keep = r >= R_HOLE - 0.05 && r <= R_HOLE + BITE + HW;
    if (keep) run.push([px, py]);
    else { if (run.length > 2) rimLines.push(rdp(run, 0.01)); run = []; }
  }
  if (run.length > 2) rimLines.push(rdp(run, 0.01));
}

const f2 = n => (Math.round(n * 1000) / 1000).toString();
const dOf = p => 'M' + p.map(q => f2(q[0]) + ' ' + f2(q[1])).join('L') + 'Z';
const dOpen = p => 'M' + p.map(q => f2(q[0]) + ' ' + f2(q[1])).join('L');
const cutD = simp.map(dOf).join('');
const engD = engrave.concat(rimLines).map(dOpen).join('');

const NAME = `${L}-lead ${B}-bight knot`;
const OUT  = process.env.OUT;
// The canvas has to hold the rim continuations, which follow the ribbon out to
// R_HOLE + BITE -- past the rim, where the cut layer stops. Sizing this to the cut
// alone clipped 1mm off every file. Half a millimetre on top covers the 0.1mm stroke.
const PAD  = BITE + 0.5;
const SIZE = f2(2 * (R_HOLE + PAD));
const ORG  = f2(-(R_HOLE + PAD));
const R    = f2(R_HOLE);
const svg = `<svg xmlns="http://www.w3.org/2000/svg"
     width="${SIZE}mm" height="${SIZE}mm" viewBox="${ORG} ${ORG} ${SIZE} ${SIZE}">
  <title>${NAME} sound hole - ${R_HOLE}mm radius</title>
  <!-- 1 user unit = 1mm, so this prints/cuts at true size.
       Sound hole radius ${R_HOLE}mm (${2 * R_HOLE}mm diameter).
       ${NAME}: ONE self-crossing ribbon ${2 * HW}mm wide, ${B * (L - 1)} crossings.
       Single strand r = ${R_MID.toFixed(2)} + ${AMP}*cos(${B}*theta/${L}), theta in [0,${2 * L}pi),
       winding twice before it closes.
       The ribbon overruns the rim by ${BITE}mm at ${B} points, fusing it
       into the soundboard, so the rosette cannot drop out when cut.
       Open area ${openArea.toFixed(0)}mm2 = ${(100 * openArea / (Math.PI * R_HOLE * R_HOLE)).toFixed(0)}% of a plain ${2 * R_HOLE}mm hole
       (acoustically equivalent to a plain round hole of
       ${(2 * Math.sqrt(openArea / Math.PI)).toFixed(1)}mm diameter).
       Narrowest cut region: ${(2 * Math.min(...inrad)).toFixed(2)}mm across. -->


  <!-- CUT: every closed path here is waste that drops out -->
  <g id="cut" fill="none" stroke="#000000" stroke-width="0.1">
    <path d="${cutD}"/>
  </g>

  <!-- ENGRAVE (optional): over/under interlace hints -->
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
console.log('knot                :', NAME);
console.log('leads x bights      :', L, 'x', B, ' gcd =', gcd(L, B), '(coprime, so one strand)');
console.log('AMP,HW,R_MID        :', AMP, HW, R_MID.toFixed(2));
console.log('centre opening (mm) :', R_CENTRE.toFixed(2));
console.log('tightest region inradius (mm):', Math.min(...inrad).toFixed(3));
console.log('largest  region inradius (mm):', Math.max(...inrad).toFixed(3));
console.log('open area (mm^2)    :', openArea.toFixed(2));
console.log('plain hole area     :', discArea.toFixed(2));
console.log('open fraction       :', (openArea / discArea * 100).toFixed(1) + '%');
console.log('equiv. round hole dia (mm):', (2 * Math.sqrt(openArea / Math.PI)).toFixed(2));
console.log('crossings           :', crossings, ' expected B*(L-1) =', B * (L - 1),
            crossings === B * (L - 1) ? 'OK' : 'MISMATCH');
console.log('weave alternates    :', weaveOK ? 'OK - every crossing has one over, one under'
                                             : 'NO - engrave layer suppressed');
if (unpaired) console.log('unpaired events     :', unpaired, '<- crossing detection is off');
console.log('engrave polylines   :', engrave.length, ' expected 2*B*(L-1) =', 2 * B * (L - 1),
            engrave.length === 2 * B * (L - 1) ? 'OK' : 'CHECK');
console.log('rim continuations   :', rimLines.length, ' (decorative; the ribbon crossing its anchors)');
console.log('ribbon width (mm)   :', 2 * HW);
console.log('rim anchors         :', B);
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
}
