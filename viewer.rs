//! The thirteen rosettes in 3D, each cut through a patch of 3mm plywood.
//!
//!     target/release/viewer [--out=turn.html]
//!
//! A rosette is a cut-out, not a part: every closed path in the cut layer is
//! waste, and the ribbon is the board that is left. So the cut file has no outline
//! for the turn3d module in lasermade-tools to find -- it reads each waste region as a part of
//! its own. Here they become the holes in a square of soundboard instead, at the
//! 3mm the writeups assume (3mm Baltic birch), and the blue engrave lines that mark
//! the over/under at each crossing are drawn on the face.
//!
//! The square runs a margin past the rim (MARGIN, or a fifth of the radius on the
//! big ones) so the anchors read as board running on into board, which is the
//! point the previews make in two golds.
use lasermade_tools::py::{max, round};
use lasermade_tools::turn3d::*;

const HERE: &str = env!("CARGO_MANIFEST_DIR");
const ROSETTES: [(&str, &str); 13] = [
    ("2 × 3 trefoil", "2-lead_3-bight_knot_radius30mm.svg"),
    ("2 × 5 cinquefoil", "2-lead_5-bight_knot_radius30mm.svg"),
    ("2 × 7 septafoil", "2-lead_7-bight_knot_radius30mm.svg"),
    ("3 × 2", "3-lead_2-bight_knot_radius30mm.svg"),
    ("3 × 4", "3-lead_4-bight_knot_radius30mm.svg"),
    ("3 × 5", "3-lead_5-bight_knot_radius30mm.svg"),
    ("4 × 3 · 39mm", "4-lead_3-bight_knot_radius39mm.svg"),
    ("4 × 5 · 39mm", "4-lead_5-bight_knot_radius39mm.svg"),
    ("5 × 4 · 60mm", "5-lead_4-bight_knot_radius60mm.svg"),
    ("9 × 11 · 300mm", "9-lead_11-bight_knot_radius300mm.svg"),
    ("plait · 8", "8-crossing_plait_radius30mm.svg"),
    ("plait · 10", "10-crossing_plait_radius30mm.svg"),
    ("plait · 10 · 50mm", "10-crossing_plait_radius50mm.svg"),
];
const MARGIN: f64 = 12.0; // mm of board past the rim

/// -> one part: a square of soundboard with the cut layer's waste as its holes.
fn board(svg: &str) -> Part {
    let waste = parts(svg);
    // a waste region with a hole in it would be an island of ribbon that drops
    // out with it -- the one thing these designs are built not to do
    assert!(waste.iter().all(|w| w.h.is_empty()), "{svg}");
    let (geo, vbw) = geometry(svg).unwrap_or_else(|e| lasermade_tools::die(e));
    let reach = vbw / 2.0; // rim + bite + 0.5, as preview.rs reads it
    let s = round(reach + max(MARGIN, reach / 5.0));
    let engr = geo.iter().filter(|(c, _, _)| c == ENGRAVE).map(|(_, pts, _)| pts.iter().map(|&(x, y)| [F(x), F(-y)]).collect()).collect();
    Part {
        o: vec![[I(-s), I(-s)], [I(s), I(-s)], [I(s), I(s)], [I(-s), I(s)]],
        h: waste.into_iter().map(|w| w.o).collect(),
        e: engr,
        ink: String::new(),
        bx: [I(-s), I(-s), I(s), I(s)],
    }
}

fn build() -> Vec<(String, J)> {
    ROSETTES
        .iter()
        .map(|(name, f)| {
            let b = board(&format!("{HERE}/{f}"));
            (
                name.to_string(),
                design(vec![
                    ("parts", J::A(vec![placed(&b, &T(0, 0, 0), STILL, "ply")])),
                    ("ex", J::N(I(0))),
                    ("view", nums(&[F(-0.35), F(0.75)])),
                ]),
            )
        })
        .collect()
}

fn main() {
    let path = out_arg(&format!("{HERE}/turn.html"));
    write(&path, "Knotwork Sound Holes", &mut build(), "Drag to turn; right-drag, shift-drag or move two fingers to slide the view; scroll or pinch to zoom in on whatever is under the pointer; double-click to turn round that point, and Reset view to go back. Pick a rosette above. Each is its cut file’s waste taken out of a square of 3 mm plywood soundboard, with the engrave lines that mark the over/under on the face. The ribbon is the board that is left, and it runs on into the board round the rim at every anchor.", "knotwork-soundholes/viewer.rs", false);
    println!("  wrote {}", relpath(&path));
}
