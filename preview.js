#!/usr/bin/env node
// Build the display rendering of a cut file in this repository.
//
//   node preview.js 2-lead_3-bight_knot_radius30mm.svg [OUT.svg]
//
// WHY THIS EXISTS. The thirteen files in previews/ shipped for weeks with no
// generator at all. .repro listed every one of them with a "!" -- the mark that
// says "we know, and the reason is written above" -- and the reason was that
// nobody had written this. An acknowledged exception is the right answer for a
// drawing made by hand; it is the wrong answer for a mechanical transform of a
// file that IS generated, which is what these are. Every one is the cut layer
// of its cut file, filled even-odd, over two flat colours.
//
// It is derived from the cut file and nothing else, so a preview cannot come to
// show a rosette the sheet does not cut. The rim radius comes from the viewBox
// and the bite from the cut file's own comment, which is why neither is a
// parameter here.
//
// NOT A CUT FILE. It carries fill and no stroke; sending it to a laser cuts
// nothing and engraves nothing.
'use strict';
const fs = require('fs'), path = require('path');

const GROUND = '#faf7f0';   // cream: the waste that drops out
const BOARD  = '#e6d9ae';   // pale gold: the soundboard the ribbon fuses into
const ROSETTE = '#c9a227';  // deep gold: the ribbon itself, inside the rim

function build(src, srcName) {
  const vb = src.match(/viewBox="([^"]+)"/);
  if (!vb) throw new Error('no viewBox in ' + srcName);
  const [x, y, w, h] = vb[1].trim().split(/[\s,]+/).map(Number);

  const title = (src.match(/<title>([^<]*)<\/title>/) || [, srcName])[1];

  // The bite is stated in the cut file's own comment, in the sentence that says
  // what anchors the rosette. Parsed rather than passed in: the two cannot then
  // disagree about the drawing they describe.
  const bite = src.match(/overruns the rim by ([\d.]+)mm/);
  if (!bite) throw new Error('no rim overrun stated in ' + srcName);

  // The cut layer only. The engrave layer suggests the over/under weave and is
  // not material; filling it would paint the interlace hints as if they were
  // ribbon.
  const cutGroup = src.match(/<g id="cut"[^>]*>([\s\S]*?)<\/g>/);
  if (!cutGroup) throw new Error('no <g id="cut"> in ' + srcName);
  const paths = [...cutGroup[1].matchAll(/\bd="([^"]+)"/g)].map(m => m[1]);
  if (!paths.length) throw new Error('the cut layer of ' + srcName + ' is empty');

  // An even-odd fill needs something to be outside-of. The frame is that
  // something: every closed contour inside it reverses the parity, so the waste
  // comes out cream and the ribbon comes out gold without either being named.
  const f1 = n => n.toFixed(1);
  const frame = `M${f1(x)} ${f1(y)}H${f1(x + w)}V${f1(y + h)}H${f1(x)}Z`;
  const body = frame + paths.join('');

  // The two golds part AT THE RIM, and the ribbon runs on through it. Clipping
  // the rosette to r = R_HOLE + BITE rather than to R_HOLE is the whole point:
  // bounding it at the hole sliced every lobe tip flat along a circle and drew a
  // rosette that does not exist. The viewBox is padded by exactly half a
  // millimetre past that, which is where this reads it from.
  const rim = w / 2 - 0.5;

  const mm = n => f1(n).replace(/\.0$/, '');
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${mm(w)}mm"` +
    ` height="${mm(h)}mm" viewBox="${vb[1]}">
  <title>${title} — display only</title>
  <!-- DISPLAY ONLY - not a cut file. The cut layer of ${srcName}, filled even-odd.
       Deep gold is the rosette, pale gold the board it fuses into, cream the waste
       that drops out. The ribbon overruns the rim by BITE (${bite[1]}mm here) and becomes
       board, so the two golds part at r=${rim}mm with the ribbon running straight on
       through - bounding the fill at R_HOLE instead sliced every lobe tip flat along a
       circle and showed a rosette that does not exist. The cream is painted, not
       transparent, so the picture reads the same on a light or a dark page.
       Cut files carry no fill; browsing one shows a transparency checkerboard. -->
  <defs><clipPath id="rosette"><circle cx="0" cy="0" r="${rim}"/></clipPath></defs>
  <rect x="${f1(x)}" y="${f1(y)}" width="${f1(w)}" height="${f1(h)}" fill="${GROUND}"/>
  <path fill="${BOARD}" fill-rule="evenodd" d="${body}"/>
  <g clip-path="url(#rosette)"><path fill="${ROSETTE}" fill-rule="evenodd" d="${body}"/></g>
</svg>
`;
}

if (require.main === module) {
  const src = process.argv[2];
  if (!src) {
    console.error('usage: node preview.js CUTFILE.svg [OUT.svg]');
    console.error('  Writes previews/<name> unless an output path is given.');
    process.exit(2);
  }
  const name = path.basename(src);
  const out = process.argv[3] || path.join(path.dirname(src), 'previews', name);
  const svg = build(fs.readFileSync(src, 'utf8'), name);
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(out, svg);
  console.log(`  ${out}  (${(svg.length / 1024).toFixed(1)} kB)`);
}

module.exports = { build };
