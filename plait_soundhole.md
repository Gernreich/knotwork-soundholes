# Plait Sound Hole — even crossings (two woven ribbons)

Generator: `plait_soundhole.js`

Companion (one strand, any coprime leads × bights):
[`knot_soundhole.md`](knot_soundhole.md)

Produces a cut-ready SVG of a circular plait sized for an instrument
sound hole. Two identical sinusoidal ribbons weave around a ring; ribbon B is
ribbon A rotated by `pi/N`.

Output is millimetre-true: `1 user unit = 1mm`, with a physical `width`/
`height` in mm, so it prints and cuts at real size.

## The constraint that shapes the design

This is a **cut-out**. The removed material is the open area; the woven ribbon
is what stays. A ribbon ring floating inside a round hole would simply drop
out when the last cut closes.

So the ribbon peaks deliberately **overrun the rim by `BITE` mm**, fusing the
rosette into the soundboard at `2N` anchor points. `R_MID` is *derived*, not
chosen, to guarantee this:

```
R_MID = R_HOLE + BITE - AMP - HW
```

Consequence: **there is no continuous rim circle in the cut layer.** The outer
boundary is `2N` separate arcs between anchors. That is correct — do not
"fix" it by adding a full circle, or the rosette falls on the floor.

Because it is a single sheet cut through, true over/under cannot exist in the
material — at a crossing both ribbons are material and merge. The interlace is
suggested by the **engrave** layer instead.

## Why the crossing count is always even

Both ribbons are polar graphs `r(theta)`, and a point in the plane has a unique
`(r, theta)`, so they can only meet where `rA(theta) = rB(theta)`. That
difference is `2*AMP*sin(N*theta)` — exactly `2N` zeros per turn.

No parameter changes this. There is also a topological reason: each strand
meets every crossing once, and alternating over/under only closes up over an
even number. **For an odd crossing count — or any coprime leads × bights, odd or
even — use the companion generator.**

## Usage

```bash
node plait_soundhole.js --help                               # every setting and its default
node plait_soundhole.js                                      # report only, writes nothing
OUT=10-crossing_plait_radius30mm.svg node plait_soundhole.js # N=5, the default
N=4 OUT=8-crossing_plait_radius30mm.svg node plait_soundhole.js
R_HOLE=50 AMP=10.8 HW=3.3 BITE=2.5 OUT=10-crossing_plait_radius50mm.svg node plait_soundhole.js
DIAG=1 node plait_soundhole.js                               # per-region dump
```

`--help` prints this same table, so you do not need the document to hand.

**`OUT` overwrites without asking** — there is no force flag and no prompt. All three
`OUT=` lines above name shipped sample files, so running them in this directory
rewrites those files; at these parameters the bytes come back identical, but change a
number and reuse the same command line and you have replaced a sample with different
geometry and nothing will have said so. Write somewhere else while experimenting.

| Var | Default | Meaning |
|---|---|---|
| `R_HOLE` | `30` | sound hole radius, mm |
| `N` | `5` | lobes per ribbon → `2N` crossings and `2N` rim anchors |
| `AMP` | `6.5` | radial swing of the weave |
| `HW` | `2.0` | ribbon half-width → ribbon is `2*HW` mm wide |
| `BITE` | `1.5` | rim overrun; this is what anchors the rosette |
| `NG` | `1400` | sampling grid resolution |
| `MIN_FEATURE` | `0.25` | gaps narrower than this are welded to solid material |
| `OUT` | unset | output path — **nothing is written unless set** |
| `DIAG` | unset | verbose per-region report |

**The file that lands is not ready for the laser yet.** Its geometry is, but it
carries two layers and only one of them is a cut. See
[Output layers](#output-layers) before you send it: the blue `engrave` lines must be
given anything other than a cut.

`AMP` and `HW` do **not** auto-scale with `R_HOLE`. Roughly
`AMP ≈ 0.217*R_HOLE` and `HW ≈ 0.067*R_HOLE` reproduces these proportions;
a guard refuses geometry that cannot close and suggests values.

## Variants at R_HOLE=30 (60mm hole)

| `N` | Crossings | Regions | Open area | Equiv. plain hole | Narrowest cut | Slivers welded |
|---|---|---|---|---|---|---|
| 3 | 6 | 13 | 59.0% | 46.10mm | 4.03mm | 4 |
| 4 | 8 | 17 | 55.9% | 44.85mm | 3.54mm | 12 |
| **5** | **10** | **21** | **52.4%** | **43.45mm** | **3.10mm** | **0** |
| 6 | 12 | 25 | 48.9% | 41.96mm | 2.70mm | 0 |

Regions are always `4N + 1` — one centre star, plus a lens and a rim opening for
each of the `2N` crossings. (The count keys on
crossings, not lobes. It is the companion's `L*B + 1` rule seen at two leads: a
plait is 2 leads by `2N` bights, and `2 * 2N + 1` is `4N + 1`.)
Fewer crossings = bolder, more open weave. More crossings = finer and more
lace-like, with less open area.

**Above the table.** `4N + 1` keeps holding — checked at `N` = 7, 8, 10 and 12, all
exact. Unlike the companion, this design does not lose regions as it gets busier,
because two ribbons stay a fixed `2*HW` apart however many lobes they have. What
does move is the narrowest cut: **2.34mm** at `N=7`, 2.03 at 8, 1.49 at 10 and
**1.06mm** at `N=12`. That is the number to watch rather than the region count —
compare it against your kerf, and remember it is the *drawn* width, before the beam
takes its share.

## Output layers

Two groups, told apart by **stroke colour** as well as by `id`. Colour is the one that
survives the trip: many SVG importers flatten groups, and most laser software assigns
operations by colour rather than by group name.

| Group | Stroke | Purpose |
|---|---|---|
| `cut` | **black `#000000`** | every closed path is waste that drops out |
| `engrave` | **blue `#0000ff`** | over/under interlace hints (`4N` lines — two per crossing) **and** rim continuations |

### Rim continuations

The engrave layer carries a second, purely decorative set of lines. Where a ribbon
overruns the rim it fuses into the soundboard, and the cut layer stops dead at
`R_HOLE` — past the rim there is nothing to cut. That leaves every anchor reading as a
flat pad and the rosette looking sliced off by a circle.

The ribbon edges do carry on out there, as far as `R_HOLE + BITE`. These lines put
that outline back across the anchor, so the eye reads the ribbon as passing into the
board. Both ribbons get them, A directly and B through the same `pi/N` rotation the
weave itself uses. At defaults they occupy radii **30.0 to 31.5mm** — exactly the
overrun band — while the interlace lines stay inside the rim.

Reported separately as `rim continuations`, and deliberately not folded into the `4N`
count: that number is tied to the crossings and is a check on the weave. Observed at
`2N + 2` — one per anchor plus two where the scan seam falls — but treat that as
measured rather than guaranteed; it is decoration, not topology.

**Adding them changed no cut geometry.** Verified by regenerating and comparing: the
`cut` group is byte-identical before and after. Raising `BITE` widens the overrun and
strengthens the effect, at the cost of how firmly the rosette is anchored.

**The blue lines must not be cut.** Give them a score or engrave operation, or delete the
layer. They run straight across the ribbon — at defaults they span radii 19.2 to 26.2mm
against a ribbon band of 21 to 25mm — so cutting them severs the ribbon at all `2N`
crossings, through exactly the material that holds the rosette together. It comes apart
as it leaves the machine.

Give every colour you keep an explicit operation. A per-colour job **silently skips any
colour you leave unmapped**: leave `cut` unmapped and you get an engraved picture of a
sound hole and no hole.

## How it works

```
signed distance field -> grid sample -> flood fill regions ->
weld unmanufacturable slivers -> marching squares -> chain segments ->
RDP simplify -> emit SVG
```

`field(x,y) > 0` means "cut away (air)", `< 0` means "material".

## Read the validation report

Printed on every run. This is the point of the tool, not decoration.

| Line | Must be |
|---|---|
| closed contours vs flood regions | equal, else loops were lost |
| unclosed chains | `0`, else a contour leaked |
| loose islands (CW) | `0` — a clockwise contour means material fully surrounded by air, which **falls out when cut** |
| contour area vs flood area | delta ≤ ~0.1%; two independent measurements of the same shape |
| content vs canvas | `OK` — every emitted point inside the viewBox, with the margin shown |

`slivers welded shut` is informational, and its exact value means nothing. A
sliver is a whole air region whose inradius falls under `MIN_FEATURE` — narrower
than the cutter can make — filled back to solid material before the contours are
traced.

**Its value is an artifact of where the sampling grid falls**, not a property of
the design. The companion at 2 leads × 3 bights, geometry untouched, reports 8 slivers at
`NG=700`, 6 at 1000, 4 at 1400 and 8 at 2000, while its contour count, open
fraction and area hold steady throughout. It is not normally `0`, it does not
converge, and chasing it to zero is chasing the grid.

`trough curv. radius` is the other printed line the table does not cover, and it is
informational too. It is the centreline's radius of curvature at the inner trough,
shown against `HW` as a ratio, and the worry behind it is that a ribbon wider than its
own trough curvature fills its notch. The ratio falls steadily as lobes are added —
3.24 at `N = 3`, 1.56 at 4, **0.93 at the default 5**, 0.63 at 6, 0.15 at 12 — and
crossing below 1 turns out to predict nothing: every one of those returns an exact
`4N + 1`, and the sliver count does not track the ratio either (12 slivers at 1.56,
none at 0.93). Read it as a description of the curve, not a gate.

**The region count is the check that means something.** `4N + 1` above is stable
across resolutions and tells you whether the shape you get is the shape the curve
describes. If it matches, the welding only removed grazing tangencies. If it falls
short, real regions were under the cutting floor and are gone — and more grid will
not bring them back.

`content vs canvas` is the newest line and the only one that looks at the **document**
rather than the geometry. Every other check describes what the generator computed; none
of them noticed when the rim continuations reached 1mm past a canvas sized for the cut
layer alone, and files were shipped clipped while every invariant passed. It measures
the emitted points against the viewBox and names the shortfall if there is one.

## Traps already sprung here

Both were **silent** — they produced plausible-looking output:

- **Chaining on rounded float coordinates.** Shared cell edges compute to
  slightly different values from each neighbouring cell, so loops fragment.
  Contours are keyed on integer grid-edge IDs instead.
- **Plain RDP on a closed loop.** `pts[0] == pts[last]` makes the baseline
  zero-length, every perpendicular distance zero, and the entire loop
  collapses to two points. See `rdpClosed`.

The marching-squares table also has a load-bearing invariant: segments are
oriented **inside-on-left**. Chaining depends on it, and so does the
loose-island check. Case 5's centre-inside branch was wrong first time round —
re-derive rather than trust it.

## Caveats

- Polylines only — no arc/bezier fitting, no kerf/tool-offset compensation,
  no DXF export.
- `MIN_FEATURE` guards minimum **air** width, not material width. Ribbon
  thickness is uniform `2*HW` by construction so it is fine in practice, but
  nothing verifies a thin material neck.
- Open area is large (≈52% at defaults). For a real soundboard consider what
  that removes structurally.
- Cut in 3mm Baltic birch plywood. Ribbon width has not been swept against other stock,
  so treat the narrowest-cut column as the number to check against your kerf.

## Generated files

- `8-crossing_plait_radius30mm.svg` — 8 crossings (N=4)
- `10-crossing_plait_radius30mm.svg` — 10 crossings (N=5, default)
- `10-crossing_plait_radius50mm.svg` — the same 10 crossings at `R_HOLE = 50`, a 100mm
  hole, on `AMP=10.8 HW=3.3 BITE=2.5` — the scaling worked in the Usage section above,
  committed so the example names a file you have. Open area 52.9%, within half a point
  of the 30mm original, which is what scaling all three together is supposed to preserve.
