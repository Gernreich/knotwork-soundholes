# Knotwork Sound Holes

**[← All projects](https://gernreich.github.io/)**

**[Read the writeup](https://gernreich.github.io/knotwork-soundholes/)** · plait:
[`plait_soundhole.md`](plait_soundhole.md) · knot:
[`knot_soundhole.md`](knot_soundhole.md)

**[Turn it in 3D](https://gernreich.github.io/knotwork-soundholes/turn.html)**

Two Rust generators that produce cut-ready SVG rosettes for an instrument sound hole.
Output is millimetre-true — `1 user unit = 1mm`, with a physical `width`/`height` — so it
prints and cuts at real size.

<p>
<img src="previews/2-lead_3-bight_knot_radius30mm.svg" alt="A 2-lead 3-bight knot rosette in deep gold: one continuous ribbon crossing itself three times across a 60 mm hole and running on into the paler surrounding board at three anchors; the white shapes are the material that drops out" width="46%">
<img src="previews/10-crossing_plait_radius30mm.svg" alt="A 10-crossing plait rosette in deep gold: two ribbons woven around a 60 mm hole and running on into the paler surrounding board; the white shapes are the material that drops out" width="46%">
</p>

*Left: one strand, 2 leads × 3 bights. Right: two ribbons, 10 crossings. Both at a 60 mm
hole. Display renderings — **deep gold is the rosette, pale gold the board it fuses into,
white is what drops out.** The ribbon crosses between the two golds without a break,
because the rosette is not a ring sitting in a hole: it is board material, and that is what
stops it falling through. The cut files carry no fill, so a browser shows them as thin
lines on a transparency checkerboard. **[See all thirteen](https://gernreich.github.io/knotwork-soundholes/#every-sample-at-a-glance)**.*

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**, where the cutting
and assembly are shown.

**[The rest of the build files](https://gernreich.github.io/lasermademusic.html)** — every instrument,
generator and tool, indexed.

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
| Program | `plait_soundhole.rs` | `knot_soundhole.rs` |
| Open area at radius 30 mm | 52.4% (default `N = 5`) | 63.3% at 2 × 3, 48.2% at 3 × 5 |
| Rim anchors | `2N` | `BIGHTS` |

In the plait each strand meets every crossing once, so the count is even by
construction. A self-crossing strand visits each crossing twice, which is why a
3-crossing trefoil is a valid alternating knot. Asking the knot generator for a
leads/bights pair that is not coprime is refused with an error.

## Quick start

```
cargo build --release                                                # once; programs land in target/release/
target/release/plait_soundhole                                       # report only, writes nothing
OUT=10-crossing_plait_radius30mm.svg target/release/plait_soundhole  # 10-crossing plait, 60mm hole
LEADS=2 BIGHTS=5 OUT=2-lead_5-bight_knot_radius30mm.svg target/release/knot_soundhole
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
very open — **37% to 63%** of the disc removed, measured across the thirteen shipped samples. Cut in
**3 mm Baltic birch plywood** — the void-free core matters when the ribbon narrows to a
couple of millimetres.

## Files

| | |
|---|---|
| `plait_soundhole.rs` · `.md` | even-crossing plait generator and its writeup |
| `knot_soundhole.rs` · `.md` | single-strand knot generator, any coprime leads × bights |
| `src/contour.rs` | what both generators share: flood fill, sliver welding, marching squares, chaining, simplification |
| `src/jsmath.rs` · `src/jsfmt.rs` | Node's own sin, cos, atan2 and hypot, and JavaScript's number-to-text, so every SVG drawn while the generators were JavaScript redraws byte for byte |
| `src/lib.rs` · `Cargo.toml` · `Cargo.lock` | the Rust package: four programs and the library above; only `viewer.rs` needs another crate, lasermade-tools, from the checkout beside this one |
| `*-bight_knot_radius*.svg` | cut-ready knot samples, named leads × bights × radius |
| `*-crossing_plait_radius*.svg` | cut-ready plait samples |
| `preview.rs` | draws `previews/` from the cut files: the cut layer filled in gold and white |
| `previews/` | display renderings of every sample — **not** cut files |
| `turn.html` | every sample in 3D, its waste taken out of a square of 3 mm plywood — drag to turn |
| `viewer.rs` | builds `turn.html` from the cut files, with the turn3d module in lasermade-tools, and the SolidCraft designs (`--solidcraft=DIR`) |
| `solidcraft/` | each sample's soundboard as a SolidCraft design, and the script it was built from |
| `index.md` · `*.html` | the published pages; the markdown is the source |

Requires Rust ([rustup](https://rustup.rs)); no crates beyond the standard library. The generators were JavaScript until 2026-11-02. Released under [CC0 1.0](LICENSE).

## SolidCraft designs

`target/release/viewer --solidcraft=DIR` writes each rosette's soundboard as a
SolidCraft design, one per sample: a square of board with the cut file's waste taken
out, every point of it as the cut file has it. Open `solidcraft/NAME.sc` in SolidCraft. The parameters are under the **SOLID**
tab, **MODIFY** panel, **Change Parameters**; `solidcraft/NAME.solidcraft.json` is the
script it was built from (`solidcraft-cli run FILE --save NAME.sc`). The `.sc` holds
the design without SolidCraft's build cache, which it rebuilds on opening, so it is the
same file every time.

| Parameter | Default | What it is |
|---|---|---|
| `ply` | 3 mm | the soundboard's thickness |
| `margin` | 12 mm | board past the rim, at least; a fifth of the reach on the big ones |
| `reach` · `half` | read only | the rosette's reach from the cut file, and the board's half-width, to the mm |

Only the board follows its parameters. The waste is the cut file's own: the generators
contour it out of a field sampled on a grid, which no expression can follow, so it stays
where the cut file has it. To change the rosette itself, run the generator with new
parameters and write the design again.

The designs, each beside its script:

- `solidcraft/knotwork-soundholes-2-3-trefoil.sc` · `solidcraft/knotwork-soundholes-2-3-trefoil.solidcraft.json`
- `solidcraft/knotwork-soundholes-2-5-cinquefoil.sc` · `solidcraft/knotwork-soundholes-2-5-cinquefoil.solidcraft.json`
- `solidcraft/knotwork-soundholes-2-7-septafoil.sc` · `solidcraft/knotwork-soundholes-2-7-septafoil.solidcraft.json`
- `solidcraft/knotwork-soundholes-3-2.sc` · `solidcraft/knotwork-soundholes-3-2.solidcraft.json`
- `solidcraft/knotwork-soundholes-3-4.sc` · `solidcraft/knotwork-soundholes-3-4.solidcraft.json`
- `solidcraft/knotwork-soundholes-3-5.sc` · `solidcraft/knotwork-soundholes-3-5.solidcraft.json`
- `solidcraft/knotwork-soundholes-4-3-39mm.sc` · `solidcraft/knotwork-soundholes-4-3-39mm.solidcraft.json`
- `solidcraft/knotwork-soundholes-4-5-39mm.sc` · `solidcraft/knotwork-soundholes-4-5-39mm.solidcraft.json`
- `solidcraft/knotwork-soundholes-5-4-60mm.sc` · `solidcraft/knotwork-soundholes-5-4-60mm.solidcraft.json`
- `solidcraft/knotwork-soundholes-9-11-300mm.sc` · `solidcraft/knotwork-soundholes-9-11-300mm.solidcraft.json`
- `solidcraft/knotwork-soundholes-plait-10.sc` · `solidcraft/knotwork-soundholes-plait-10.solidcraft.json`
- `solidcraft/knotwork-soundholes-plait-10-50mm.sc` · `solidcraft/knotwork-soundholes-plait-10-50mm.solidcraft.json`
- `solidcraft/knotwork-soundholes-plait-8.sc` · `solidcraft/knotwork-soundholes-plait-8.solidcraft.json`

**Checked** body by body against the 3D view: all thirteen boards match it exactly.

## STEP files

`step/` holds each SolidCraft design, built by SolidCraft, as a STEP file to open or
import in a CAD program such as Fusion or FreeCAD: one solid body, in millimetres,
with no parameters. Every face is flat, so a curve comes in as short straight runs;
engraving has no depth and is left out. `.repro` checks them:

```
target/release/viewer --solidcraft=solidcraft
solidcraft-cli run solidcraft/NAME.solidcraft.json --out step/NAME.step
```

The files:

- `step/knotwork-soundholes-2-3-trefoil.step`
- `step/knotwork-soundholes-2-5-cinquefoil.step`
- `step/knotwork-soundholes-2-7-septafoil.step`
- `step/knotwork-soundholes-3-2.step`
- `step/knotwork-soundholes-3-4.step`
- `step/knotwork-soundholes-3-5.step`
- `step/knotwork-soundholes-4-3-39mm.step`
- `step/knotwork-soundholes-4-5-39mm.step`
- `step/knotwork-soundholes-5-4-60mm.step`
- `step/knotwork-soundholes-9-11-300mm.step`
- `step/knotwork-soundholes-plait-10-50mm.step`
- `step/knotwork-soundholes-plait-10.step`
- `step/knotwork-soundholes-plait-8.step`

## Credits

Made by Michael Gernreich with [Claude](https://www.anthropic.com/claude) (Anthropic), working in Claude Code, which co-wrote the code and the documentation. Commits Claude made carry a `Co-Authored-By: Claude` line.
