//! The thirteen rosettes in 3D, each cut through a patch of 3mm plywood.
//!
//!     target/release/viewer [--out=turn.html]
//!     target/release/viewer --solidcraft=DIR     # each rosette as a SolidCraft design
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
/// (eps: see parts_eps; the 3D view drops points within 0.02 mm of a line,
/// the design keeps the cut file's every point)
fn board(svg: &str, eps: f64) -> Part {
    let waste = parts_eps(svg, eps);
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
            let b = board(&format!("{HERE}/{f}"), 0.02);
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

/// Each rosette's board as a SolidCraft script (2026-11-08): the ply and
/// the board's margin are parameters; the waste is the cut file's own, which
/// the generators contour out of a field no expression can follow, so it
/// stays where the cut file has it.
fn solidcraft(dir: &str) {
    use lasermade_tools::solidcraft::{n, Script};
    std::fs::create_dir_all(dir).unwrap_or_else(|e| lasermade_tools::die(format!("{dir}: {e}")));
    // the STEP files' names, step-export's slugs of the rosettes' names
    const SLUGS: [&str; 13] = ["2-3-trefoil", "2-5-cinquefoil", "2-7-septafoil", "3-2", "3-4", "3-5", "4-3-39mm", "4-5-39mm", "5-4-60mm", "9-11-300mm", "plait-8", "plait-10", "plait-10-50mm"];
    for ((name, f), slug) in ROSETTES.iter().zip(SLUGS) {
        let svg = format!("{HERE}/{f}");
        let b = board(&svg, 0.0);
        let (_, vbw) = geometry(&svg).unwrap_or_else(|e| lasermade_tools::die(e));
        let mut sc = Script::new();
        sc.len("ply", "3 mm", "the soundboard's thickness");
        sc.len("reach", format!("{} mm", n(vbw / 2.0)), "read only: the rosette's reach, its rim and bite and half a mm, from the cut file");
        sc.len("margin", format!("{} mm", n(MARGIN)), "board past the rim, at least; a fifth of the reach on the big ones");
        sc.len("half", "round((reach + max(margin, reach/5))/(1 mm))*1 mm", "read only: the board's half-width, to the mm");
        let sk = sc.sketch("XY", None, "board sketch");
        sc.poly(&[["-half", "-half"], ["half", "-half"], ["half", "half"], ["-half", "half"]]);
        for h in &b.h {
            let pts: Vec<[String; 2]> = h.iter().map(|q| [format!("{} mm", n(q[0].f())), format!("{} mm", n(q[1].f()))]).collect();
            sc.poly(&pts);
        }
        sc.finish();
        sc.extrude(&sk, "ply", false, "new", &[], "board");
        sc.measure();
        let stem = format!("knotwork-soundholes-{slug}");
        let out = format!("{dir}/{stem}.solidcraft.json");
        std::fs::write(&out, sc.text()).unwrap_or_else(|e| lasermade_tools::die(format!("{out}: {e}")));
        println!("  wrote {stem}.solidcraft.json ({name})");
    }
}

fn main() {
    if let Some(dir) = std::env::args().skip(1).find_map(|a| a.strip_prefix("--solidcraft=").map(String::from)) {
        return solidcraft(&dir);
    }
    let path = out_arg(&format!("{HERE}/turn.html"));
    write(&path, "Knotwork Sound Holes", &mut build(), "Drag to turn; right-drag, shift-drag or move two fingers to slide the view; scroll or pinch to zoom in on whatever is under the pointer; double-click to turn round that point, and Reset view to go back. Pick a rosette above. Each is its cut file’s waste taken out of a square of 3 mm plywood soundboard, with the engrave lines that mark the over/under on the face. The ribbon is the board that is left, and it runs on into the board round the rim at every anchor.", "knotwork-soundholes/viewer.rs", false);
    println!("  wrote {}", relpath(&path));
}
