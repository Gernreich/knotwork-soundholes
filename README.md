# Knotwork Sound Holes

Two Node generators that produce cut-ready SVG rosettes for an instrument sound hole.
Output is millimetre-true — `1 user unit = 1mm`, with a physical `width`/`height` — so it
prints and cuts at real size.

<p>
<img src="previews/2-lead_3-bight_knot_radius30mm.svg" alt="A 2-lead 3-bight knot rosette in deep gold: one continuous ribbon crossing itself three times across a 60mm hole and running on into the paler surrounding board at three anchors; the cream shapes are the material that drops out" width="46%">
<img src="previews/10-crossing_plait_radius30mm.svg" alt="A 10-crossing plait rosette in deep gold: two ribbons woven around a 60mm hole and running on into the paler surrounding board; the cream shapes are the material that drops out" width="46%">
</p>

*Left: one strand, 2 leads × 3 bights. Right: two ribbons, 10 crossings. Both at a 60mm
hole. Display renderings — **deep gold is the rosette, pale gold the board it fuses into,
cream is what drops out.** The ribbon crosses between the two golds without a break,
because the rosette is not a ring sitting in a hole: it is board material, and that is what
stops it falling through. The cut files carry no fill, so a browser shows them as thin
lines on a transparency checkerboard. **[See all thirteen](https://gernreich.github.io/knotwork-soundholes/#every-sample-at-a-glance)**.*

**[Read the writeup](https://gernreich.github.io/knotwork-soundholes/)** · plait:
[`plait_soundhole.md`](plait_soundhole.md) · knot:
[`knot_soundhole.md`](knot_soundhole.md)

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and assembly are shown.

**[Download everything as a ZIP](https://github.com/Gernreich/knotwork-soundholes/archive/refs/heads/main.zip)** — both generators, both writeups, every cut file.

## Which generator?

They differ in whether the ribbon is one strand or two, and neither can produce the
other's shapes. Both are described the same way — `L` leads by `B` bights — and
`gcd(L, B)` is the number of separate pieces you end up with. An alternating over/under
interlace has to close up when you return to your start, and that is what decides it.

| | two ribbons | one strand |
|---|---|---|
| Look | two-ribbon plait, braided | one continuous self-crossing strand |
| Sized by | `N` lobes → `2N` crossings | `LEADS` × `BIGHTS`, coprime |
| Named forms | 6, 8, **10**, 12 crossings | **trefoil**, cinquefoil, septafoil, any coprime pair |
| Script | `plait_soundhole.js` | `knot_soundhole.js` |
| Open area at radius 30mm | 52.4% (default `N = 5`) | 63.3% at 2 × 3, 48.2% at 3 × 5 |
| Rim anchors | `2N` | `BIGHTS` |

In the plait each strand meets every crossing once, so the count is even by
construction. A self-crossing strand visits each crossing twice, which is why a
3-crossing trefoil is a valid alternating knot. Asking the knot generator for a
leads/bights pair that is not coprime is refused with an error.

## Quick start

```
node plait_soundhole.js                          # report only, writes nothing
OUT=10-crossing_plait_radius30mm.svg node plait_soundhole.js  # 10-crossing plait, 60mm hole
LEADS=2 BIGHTS=5 OUT=2-lead_5-bight_knot_radius30mm.svg node knot_soundhole.js
```

**Nothing is written unless you set `OUT`.** A bare run prints the validation report and
exits, which is the cheapest way to see what a parameter change would do.

Every run prints a validation report. Read it — it is the point of the tools, not
decoration. `loose islands (CW)` must be `0`, or material falls out when cut.

## Before you cut

These are **cut-outs**: the removed material is the open area, and the ribbon is what
stays. A ribbon ring floating inside a round hole would drop out when the last cut
closes, so the ribbon peaks deliberately overrun the rim and fuse the rosette into the
soundboard. **There is no continuous rim circle in the cut layer** — the outer boundary
is a series of arcs between anchors. That is correct; adding a full circle drops the
rosette on the floor.

The files carry two layers: black `#000000` cuts, blue `#0000ff` engraves. Give the blue
lines a score or engrave operation — cutting them severs the ribbon. That follows the
sequence every LaserMadeMusic repository uses — blue engraves, then
green → orange → cyan → black — where black is always the last cut. These rosettes have a
single cut stage, so it is black. Elsewhere that last cut frees the part; **here it must
not.** What black removes is the waste, and the rosette stays fused to the soundboard by
its rim anchors. A rosette that comes free has been cut wrong. The designs are
very open (38% to 63% of the disc removed across the documented variants). Cut in
**3mm Baltic birch plywood** — the void-free core matters when the ribbon narrows to a
couple of millimetres.

## Files

| | |
|---|---|
| `plait_soundhole.js` · `.md` | even-crossing plait generator and its writeup |
| `knot_soundhole.js` · `.md` | single-strand knot generator, any coprime leads × bights |
| `*-bight_knot_radius*.svg` | cut-ready knot samples, named leads × bights × radius |
| `*-crossing_plait_radius*.svg` | cut-ready plait samples |
| `previews/` | display renderings of every sample — **not** cut files |
| `index.md` · `*.html` | the published pages; the markdown is the source |

Requires Node. No dependencies. Released under [CC0 1.0](LICENSE).
