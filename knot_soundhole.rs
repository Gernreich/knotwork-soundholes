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
//   cargo run --release --bin knot_soundhole               report only
//   LEADS=3 BIGHTS=5 OUT=3-lead_5-bight_knot_radius30mm.svg cargo run --release --bin knot_soundhole
//   LEADS=2 BIGHTS=3 cargo run --release --bin knot_soundhole   2 leads: the classic trefoil
//   SELFTEST=1 cargo run --release --bin knot_soundhole    check the spatial hash, exit
//   DIAG=1 cargo run --release --bin knot_soundhole        per-region dump
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
//   regions. (5,3) is where that shows: 13 regions against a predicted 16, and
//   the crossing detector loses them too, finding 4 of 12.
//
//   THE (4,5) CLAIM THAT USED TO STAND HERE WAS WRONG. It said the region count
//   there falls short by the welded lenses. It does not: (4,5) gives 21 of 21
//   at the defaults and again at the shipped R_HOLE=39 with AMP and HW brought
//   down, and so do (5,4) and (9,11). All ten shipped knots meet every
//   invariant. The claim survived because nothing compared the region count
//   with L*B+1 -- the line printed the two measured counts against each other
//   and said OK -- so the prose could not be contradicted by a run.
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
// RUST SINCE 2026-11-02. This was knot_soundhole.js. The port keeps Node's
// arithmetic exactly (see src/lib.rs), and every shipped SVG redraws
// byte-identically; .repro checks that.

use soundhole::contour::{self, d_of, d_open, f2, max_abs, max_of, min_of, rdp, shoe, Grid, Pt};
use soundhole::jsfmt::{exponential, fixed, ins, number, round};
use soundhole::jsmath::{cos, hypot, sin};
use std::collections::HashMap;
use std::f64::consts::PI;

// ---------------------------------------------------------------- params
// Every knob, in one place: --help prints this list and the defaults below are
// read from it, so the two cannot drift apart. Add a parameter here and it is
// documented by construction.
const PARAMS: [(&str, &str, &str); 8] = [
    ("LEADS", "3", "times round before the strand closes"),
    ("BIGHTS", "5", "scallops at the rim; must be coprime to LEADS"),
    ("R_HOLE", "30", "sound hole radius, mm"),
    ("AMP", "7.5", "radial swing of the weave"),
    ("HW", "2.0", "ribbon half-width -> ribbon is 2*HW mm wide"),
    ("BITE", "1.5", "rim overrun; this is what anchors the rosette"),
    ("NG", "1400", "sampling grid resolution"),
    ("MIN_FEATURE", "0.25", "gaps narrower than this are welded to solid material"),
];
const FLAGS: [(&str, &str); 3] = [
    ("OUT", "output path -- NOTHING IS WRITTEN unless this is set"),
    ("SELFTEST", "verify the spatial hash against brute force, then exit"),
    ("DIAG", "verbose per-region report"),
];

const RUN: &str = "cargo run --release --bin knot_soundhole";

fn help() {
    println!(
        "
knot_soundhole -- one continuous strand, any coprime leads x bights

  Settings are environment variables, not flags:

    LEADS=3 BIGHTS=5 OUT=3-lead_5-bight_knot_radius30mm.svg {RUN}

  Numbers (default):
"
    );
    for p in PARAMS {
        println!("    {:<13} {:<6} {}", p.0, p.1, p.2);
    }
    println!("\n  Switches (unset by default), any non-empty value turns them on:\n");
    for f in FLAGS {
        println!("    {:<13}        {}", f.0, f.1);
    }
    println!(
        "
  LEADS and BIGHTS must be coprime -- gcd(L,B) is the number of separate strands,
  so anything but 1 gives several loops rather than one knot.

  AMP and HW do not scale with R_HOLE. Roughly AMP ~ 0.25*R_HOLE and
  HW ~ 0.067*R_HOLE at two or three leads; above that the ribbon must thin and
  AMP/HW has to grow with LEADS -- about 4 at four leads, 6 at five, 16 at nine.
  AMP = LEADS*HW is not enough on its own. The panel has to grow too, or the
  rim gaps weld shut.

  Full documentation: knot_soundhole.md
"
    );
}

/// An environment setting: set and non-empty means Number(value).
fn env_flag(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.is_empty())
}
fn setting(k: &str) -> f64 {
    let d = PARAMS.iter().find(|p| p.0 == k).unwrap().1;
    number(&env_flag(k).unwrap_or_else(|| d.to_string()))
}

fn gcd(a: f64, b: f64) -> f64 {
    if b != 0.0 && !b.is_nan() { gcd(b, a % b) } else { a }
}

fn main() {
    if std::env::args().skip(1).any(|a| a == "--help" || a == "-h") {
        help();
        std::process::exit(0);
    }
    std::process::exit(contour::with_big_stack(run));
}

fn run() -> i32 {
    let r_hole = setting("R_HOLE"); // sound-hole radius (mm)
    let l = setting("LEADS"); // leads: times round before closing
    let b = setting("BIGHTS"); // bights: scallops at the rim
    let amp = setting("AMP"); // radial swing of the weave
    let hw = setting("HW"); // ribbon half-width
    let bite = setting("BITE"); // how far ribbon peak overruns the rim
    let ng_f = setting("NG");
    let min_feature = setting("MIN_FEATURE");

    // Q was the retired two-lead generator's bight count. Silently ignoring it
    // would hand back the defaults and call them the answer -- Q=5 would give
    // 3x5, not the cinquefoil asked for. Refuse and say what to type instead.
    if let Ok(q) = std::env::var("Q") {
        eprintln!(
            "Q is not a setting here (got Q={q}).\n  It was the bight count of the two-lead generator this replaced.\n  That one was always 2 leads, so the same shape is:\n    LEADS=2 BIGHTS={q} {RUN}"
        );
        return 1;
    }
    let is_int = |v: f64| v.is_finite() && v.floor() == v;
    if !is_int(l) || l < 2.0 {
        eprintln!("LEADS must be an integer >= 2 (got {}).", ins(l));
        return 1;
    }
    if !is_int(b) || b < 2.0 {
        eprintln!("BIGHTS must be an integer >= 2 (got {}).", ins(b));
        return 1;
    }
    if gcd(l, b) != 1.0 {
        let g = gcd(l, b);
        let near: Vec<String> = (2..18)
            .map(|v| v as f64)
            .filter(|&v| gcd(l, v) == 1.0 && v != l)
            .map(ins)
            .collect();
        eprintln!(
            "LEADS and BIGHTS must be coprime (got {} and {}, gcd = {}).\n  gcd(L,B) is the number of separate strands: with gcd = {} this closes\n  into {} pieces, not one, so there is no single ribbon to trace and no\n  interlace to alternate over. Nearby coprime bight counts for {} leads:\n  {}\n  For an even count of crossings on two leads use plait_soundhole.",
            ins(l), ins(b), ins(g), ins(g), ins(g), ins(l), near.join(" ")
        );
        return 1;
    }
    if !is_int(ng_f) || ng_f < 1.0 {
        eprintln!("NG must be a whole number >= 1 (got {}).", ins(ng_f));
        return 1;
    }
    let ng = ng_f as usize;

    // anchor the ribbon INTO the soundboard: peak outer edge = R_HOLE + BITE
    let r_mid = r_hole + bite - amp - hw;
    let r_centre = r_mid - amp - hw;
    if r_centre < 1.0 {
        eprintln!(
            "Geometry does not close: centre opening would be {}mm.\n  R_HOLE={} BITE={} AMP={} HW={} -> R_MID={}\n  Try AMP={} HW={}.",
            fixed(r_centre, 2), ins(r_hole), ins(bite), ins(amp), ins(hw), fixed(r_mid, 2),
            fixed(r_hole * 0.25, 1), fixed(r_hole * 0.067, 1)
        );
        return 1;
    }

    // --------------------------------------- centreline: ONE strand, L turns
    // Distance is measured to these SAMPLES, not to the true curve, so it
    // overestimates by up to half the sample spacing. M points cover L turns,
    // so spacing grows with L as well as with B and the AMP*B/L radial
    // derivative. It stays well under the 0.25mm manufacturability floor over
    // the tested range, but the margin narrows as either number rises -- and
    // long before sampling becomes the limit, MIN_FEATURE does: see the
    // envelope note in the header.
    const M: usize = 20000;
    let mf = M as f64;
    let mut cx = vec![0.0; M];
    let mut cy = vec![0.0; M];
    for i in 0..M {
        let t = 2.0 * PI * l * i as f64 / mf;
        let r = r_mid + amp * cos(b * t / l);
        cx[i] = r * cos(t);
        cy[i] = r * sin(t);
    }

    // ----------------------------------------------- spatial hash for dist
    // Two radii per angle here, so no angular-window shortcut. Bucket the
    // centreline samples into a uniform grid and do an expanding-ring search.
    let ext = r_hole + 2.0;
    // CELL trades bucket count against points-per-bucket. Correctness does not
    // depend on it -- the ring search is exact for any positive CELL -- only
    // speed does. ~1.5mm keeps buckets to a handful of samples while the
    // typical query still terminates within a ring or two.
    const CELL: f64 = 1.5;
    let ncx = ((2.0 * ext) / CELL).ceil() as i64;
    let cell_of = |v: f64| -> i64 {
        let c = ((v + ext) / CELL).floor();
        (c.min((ncx - 1) as f64)).max(0.0) as i64
    };
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); (ncx * ncx) as usize];
    for i in 0..M {
        buckets[(cell_of(cy[i]) * ncx + cell_of(cx[i])) as usize].push(i);
    }
    let dist_curve = |x: f64, y: f64| -> f64 {
        let (ci, cj) = (cell_of(x), cell_of(y));
        let mut best = f64::INFINITY; // squared
        for ring in 0..ncx {
            for dj in -ring..=ring {
                let bb = cj + dj;
                if bb < 0 || bb >= ncx {
                    continue;
                }
                let edge = dj.abs() == ring;
                for di in -ring..=ring {
                    // only the shell at Chebyshev distance == ring
                    if !edge && di.abs() != ring {
                        continue;
                    }
                    let a = ci + di;
                    if a < 0 || a >= ncx {
                        continue;
                    }
                    for &i in &buckets[(bb * ncx + a) as usize] {
                        let (dx, dy) = (x - cx[i], y - cy[i]);
                        let s = dx * dx + dy * dy;
                        if s < best {
                            best = s;
                        }
                    }
                }
            }
            // A point in an unvisited cell (Chebyshev ring >= ring+1) is at
            // least ring*CELL away, so once best beats that no further ring
            // can improve it.
            let bound = ring as f64 * CELL;
            if best <= bound * bound {
                break;
            }
        }
        best.sqrt()
    };

    // SELFTEST=1 checks the hash against brute force. The expanding-ring
    // stopping bound is the one thing here that fails SILENTLY if it is wrong
    // -- it would just return distances that are too large, i.e. phantom
    // extra "cut" area.
    if env_flag("SELFTEST").is_some() {
        let brute = |x: f64, y: f64| {
            let mut b = f64::INFINITY;
            for i in 0..M {
                let (dx, dy) = (x - cx[i], y - cy[i]);
                let s = dx * dx + dy * dy;
                if s < b {
                    b = s;
                }
            }
            b.sqrt()
        };
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
            | 1;
        let mut random = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let (mut worst, mut n) = (0.0f64, 0);
        for _ in 0..4000 {
            let x = (random() * 2.0 - 1.0) * ext;
            let y = (random() * 2.0 - 1.0) * ext;
            let e = (dist_curve(x, y) - brute(x, y)).abs();
            if e > worst {
                worst = e;
            }
            n += 1;
        }
        println!(
            "SELFTEST spatial hash vs brute force: {} points, max error {}mm {}",
            n,
            exponential(worst, 3),
            if worst < 1e-9 { "OK" } else { "*** HASH IS WRONG ***" }
        );
        return if worst < 1e-9 { 0 } else { 1 };
    }

    // ---------------------------------------------------------- sample
    // signed field: >0 = cut away (air), <0 = material
    let mut grid = Grid::sample(ng, ext, |x, y| {
        soundhole::jsfmt::min(r_hole - hypot(x, y), dist_curve(x, y) - hw)
    });

    // --------------------------- regions, welding, contours, simplification
    let c = contour::contour(&mut grid, min_feature);
    let simp = &c.simp;
    let open_area = simp.iter().fold(0.0, |s, p| s + shoe(p).abs());

    // --------------------------------------- interlace engrave lines
    // With L leads a point has L-1 other passes, at parameter offsets k*M/L,
    // so the sibling's closed form ("over if m is even", valid when the only
    // other pass sits half the array away) does not survive the
    // generalisation. Instead: find the crossing events numerically, order
    // them along the strand, label them alternately, then VERIFY that the two
    // visits to each crossing got opposite labels. An alternating weave exists
    // only if that verification passes; if it fails the engrave layer is
    // dropped rather than drawn wrong.
    let step_lead = mf / l; // one lead, in samples
    let win = (mf / (2.0 * b * (l - 1.0))).ceil() as i64; // half a crossing spacing

    // distance from (x,y) to the nearest point on any OTHER pass of the strand
    let dist_other_pass = |x: f64, y: f64, i: usize| -> f64 {
        let mut best = f64::INFINITY;
        let mut k = 1.0;
        while k < l {
            let base = i as f64 + k * step_lead;
            for d in -win..=win {
                let mut j = round(base + d as f64) % mf;
                if j < 0.0 {
                    j += mf;
                }
                let j = j as usize;
                let (dx, dy) = (x - cx[j], y - cy[j]);
                let q = dx * dx + dy * dy;
                if q < best {
                    best = q;
                }
            }
            k += 1.0;
        }
        best.sqrt()
    };

    // how close the centreline comes to another pass, per sample
    let d_other: Vec<f64> = (0..M).map(|i| dist_other_pass(cx[i], cy[i], i)).collect();

    // A crossing shows as a run of small dOther; its minimum is the event.
    // Start the scan at the sample FURTHEST from any other pass, so no run
    // straddles index 0 and gets split in two -- the same class of trap as the
    // sibling's engrave seam, avoided by construction rather than by an offset.
    let mut scan0 = 0;
    for i in 1..M {
        if d_other[i] > d_other[scan0] {
            scan0 = i;
        }
    }
    let mut events: Vec<usize> = Vec::new();
    {
        let thresh = 2.0 * hw;
        let mut run: Option<(usize, f64)> = None;
        for s in 0..M {
            let i = (scan0 + s) % M;
            if d_other[i] < thresh {
                match run {
                    None => run = Some((i, d_other[i])),
                    Some((_, d)) if d_other[i] < d => run = Some((i, d_other[i])),
                    _ => {}
                }
            } else if let Some((at, _)) = run.take() {
                events.push(at);
            }
        }
        if let Some((at, _)) = run {
            events.push(at);
        }
    }
    // The scan already visited events in strand order starting from scan0, so
    // the list is in the order the strand meets them; alternate along it.
    let walk = events;
    let mut label: HashMap<usize, bool> = HashMap::new(); // sample index -> over
    for (j, &i) in walk.iter().enumerate() {
        label.insert(i, j % 2 == 0);
    }

    // VERIFY: pair events by position; the two visits to one crossing must differ
    let (mut crossings, mut weave_ok, mut unpaired) = (0usize, true, 0usize);
    {
        let mut used = vec![false; walk.len()];
        for j in 0..walk.len() {
            if used[j] {
                continue;
            }
            let (mut m, mut md) = (None, f64::INFINITY);
            for k in 0..walk.len() {
                if k == j || used[k] {
                    continue;
                }
                let d = hypot(cx[walk[j]] - cx[walk[k]], cy[walk[j]] - cy[walk[k]]);
                if d < md {
                    md = d;
                    m = Some(k);
                }
            }
            match m {
                Some(m) if md < hw => {
                    used[j] = true;
                    used[m] = true;
                    crossings += 1;
                    if label.get(&walk[j]) == label.get(&walk[m]) {
                        weave_ok = false;
                    }
                }
                _ => {
                    used[j] = true;
                    unpaired += 1;
                }
            }
        }
    }

    // nearest event to a sample, for deciding which pass we are on
    let over_at = |i: usize| -> bool {
        let Some(&first) = walk.first() else { return false };
        let (mut best, mut bd) = (first, f64::INFINITY);
        for &e in &walk {
            let d = (e as f64 - i as f64).abs();
            let d = d.min(mf - d);
            if d < bd {
                bd = d;
                best = e;
            }
        }
        label.get(&best).copied().unwrap_or(false)
    };

    // The ribbon's two edges, sample by sample from scan0: (i, x, y).
    let edge = |sign: f64, s0: usize| -> (usize, f64, f64) {
        let i = (s0 + scan0) % M;
        let (a, b2) = ((i + M - 1) % M, (i + 1) % M);
        let (mut tx, mut ty) = (cx[b2] - cx[a], cy[b2] - cy[a]);
        let seg = hypot(tx, ty);
        tx /= seg;
        ty /= seg;
        (i, cx[i] + sign * hw * -ty, cy[i] + sign * hw * tx)
    };

    let mut engrave: Vec<Vec<Pt>> = Vec::new();
    if weave_ok {
        for sign in [1.0, -1.0] {
            let mut run: Vec<Pt> = Vec::new();
            for s0 in 0..M {
                let (i, px, py) = edge(sign, s0);
                let keep = over_at(i)
                    && dist_other_pass(px, py, i) < hw
                    && hypot(px, py) < r_hole - 0.05;
                if keep {
                    run.push([px, py]);
                } else {
                    if run.len() > 4 {
                        engrave.push(rdp(&run, 0.01));
                    }
                    run.clear();
                }
            }
            if run.len() > 4 {
                engrave.push(rdp(&run, 0.01));
            }
        }
    }

    // --------------------------------------- rim continuations (engrave)
    // Where the ribbon overruns the rim it fuses into the soundboard, and the
    // cut layer stops dead at R_HOLE — which leaves the anchors reading as flat
    // pads and the whole rosette looking chopped at the edge. The ribbon's own
    // edges do carry on out there, as far as R_HOLE + BITE; the cut simply
    // cannot show them, because beyond the rim there is nothing to cut.
    // Engraving them puts the ribbon's outline back across the anchor, so the
    // eye reads it as passing into the board rather than being sliced off by
    // a circle.
    //
    // Purely decorative, and deliberately kept apart from the interlace lines
    // above: those are derived from crossings and their count is a
    // topological check. These are counted and reported separately so the two
    // are never confused.
    let mut rim_lines: Vec<Vec<Pt>> = Vec::new();
    for sign in [1.0, -1.0] {
        let mut run: Vec<Pt> = Vec::new();
        for s0 in 0..M {
            let (_, px, py) = edge(sign, s0);
            // outside the rim, and outside only because of the overrun — never the bore
            let r = hypot(px, py);
            let keep = r >= r_hole - 0.05 && r <= r_hole + bite + hw;
            if keep {
                run.push([px, py]);
            } else {
                if run.len() > 2 {
                    rim_lines.push(rdp(&run, 0.01));
                }
                run.clear();
            }
        }
        if run.len() > 2 {
            rim_lines.push(rdp(&run, 0.01));
        }
    }

    let cut_d: String = simp.iter().map(|p| d_of(p)).collect();
    let eng_d: String = engrave.iter().chain(rim_lines.iter()).map(|p| d_open(p)).collect();

    let name = format!("{}-lead {}-bight knot", ins(l), ins(b));
    // The canvas has to hold the rim continuations, which follow the ribbon out
    // to R_HOLE + BITE -- past the rim, where the cut layer stops. Sizing this
    // to the cut alone clipped 1mm off every file. Half a millimetre on top
    // covers the 0.1mm stroke.
    let pad = bite + 0.5;
    let size = f2(2.0 * (r_hole + pad));
    let org = f2(-(r_hole + pad));
    let min_inrad = min_of(&c.inrad);
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg"
     width="{size}mm" height="{size}mm" viewBox="{org} {org} {size} {size}">
  <title>{name} sound hole - {rh}mm radius</title>
  <!-- 1 user unit = 1mm, so this prints/cuts at true size.
       Sound hole radius {rh}mm ({dia}mm diameter).
       {name}: ONE self-crossing ribbon {width}mm wide, {ncross} crossings.
       Single strand r = {rmid} + {amp}*cos({b}*theta/{l}), theta in [0,{twol}pi),
       winding twice before it closes.
       The ribbon overruns the rim by {bite}mm at {b} points, fusing it
       into the soundboard, so the rosette cannot drop out when cut.
       Open area {oa}mm2 = {pct}% of a plain {dia}mm hole
       (acoustically equivalent to a plain round hole of
       {eq}mm diameter).
       Narrowest cut region: {narrow}mm across. -->


  <!-- CUT: every closed path here is waste that drops out -->
  <g id="cut" fill="none" stroke="#000000" stroke-width="0.1">
    <path d="{cut_d}"/>
  </g>

  <!-- ENGRAVE (optional): over/under interlace hints -->
  <g id="engrave" fill="none" stroke="#0000ff" stroke-width="0.1"
     stroke-linecap="round">
    <path d="{eng_d}"/>
  </g>
</svg>
"##,
        rh = ins(r_hole),
        dia = ins(2.0 * r_hole),
        width = ins(2.0 * hw),
        ncross = ins(b * (l - 1.0)),
        rmid = fixed(r_mid, 2),
        amp = ins(amp),
        b = ins(b),
        l = ins(l),
        twol = ins(2.0 * l),
        bite = ins(bite),
        oa = fixed(open_area, 0),
        pct = fixed(100.0 * open_area / (PI * r_hole * r_hole), 0),
        eq = fixed(2.0 * (open_area / PI).sqrt(), 1),
        narrow = fixed(2.0 * min_inrad, 2),
    );
    if let Some(out) = env_flag("OUT") {
        if let Err(e) = std::fs::write(&out, &svg) {
            eprintln!("cannot write {out}: {e}");
            return 1;
        }
    }

    // ---------------------------------------------------------- report
    let disc_area = PI * r_hole * r_hole;
    let signed: Vec<f64> = simp.iter().map(|p| shoe(p)).collect();
    let all_ccw = signed.iter().all(|&a| a > 0.0);
    println!("-- validation --");
    // AGAINST THE PREDICTION, not just against itself. This compared the
    // contour count with the flood count and stopped there -- two measurements
    // of the same drawing, which agree whenever the drawing is self-consistent
    // whether or not it is the knot that was asked for. The header lists
    // L*B+1 as an invariant and says it is checked in the report; it was
    // printed and never checked. At (5,3) the count comes out 13 against a
    // predicted 16, three lenses having welded shut below the cutting floor,
    // and the line read "13 flood regions: 13 OK".
    let want_regions = l * b + 1.0;
    let regions_ok = simp.len() == c.nreg && c.nreg as f64 == want_regions;
    println!(
        "closed contours     : {}  flood regions: {}  expected L*B+1 = {} {}",
        simp.len(),
        c.nreg,
        ins(want_regions),
        if regions_ok { "OK" } else { "*** MISMATCH ***" }
    );
    println!(
        "unclosed chains     : {} {}",
        c.open_chains,
        if c.open_chains == 0 { "OK" } else { "*** LEAK ***" }
    );
    println!("slivers welded shut : {} (< {}mm, unmanufacturable)", c.welded, ins(min_feature));
    println!(
        "loose islands (CW)  : {} {}",
        signed.iter().filter(|&&a| a < 0.0).count(),
        if all_ccw { "OK - nothing falls out" } else { "*** ISLAND ***" }
    );
    println!(
        "contour area        : {}  flood area: {}  delta: {}%",
        fixed(open_area, 2),
        fixed(c.flood_area, 2),
        fixed(100.0 * (open_area - c.flood_area).abs() / c.flood_area, 2)
    );
    println!("-- geometry --");
    println!("knot                : {name}");
    println!(
        "leads x bights      : {} x {}  gcd = {} (coprime, so one strand)",
        ins(l),
        ins(b),
        ins(gcd(l, b))
    );
    println!("AMP,HW,R_MID        : {} {} {}", ins(amp), ins(hw), fixed(r_mid, 2));
    println!("centre opening (mm) : {}", fixed(r_centre, 2));
    println!("tightest region inradius (mm): {}", fixed(min_inrad, 3));
    println!("largest  region inradius (mm): {}", fixed(max_of(&c.inrad), 3));
    println!("open area (mm^2)    : {}", fixed(open_area, 2));
    println!("plain hole area     : {}", fixed(disc_area, 2));
    println!("open fraction       : {}%", fixed(open_area / disc_area * 100.0, 1));
    println!("equiv. round hole dia (mm): {}", fixed(2.0 * (open_area / PI).sqrt(), 2));
    let want_cross = b * (l - 1.0);
    println!(
        "crossings           : {}  expected B*(L-1) = {} {}",
        crossings,
        ins(want_cross),
        if crossings as f64 == want_cross { "OK" } else { "MISMATCH" }
    );
    println!(
        "weave alternates    : {}",
        if weave_ok { "OK - every crossing has one over, one under" } else { "NO - engrave layer suppressed" }
    );
    if unpaired > 0 {
        println!("unpaired events     : {unpaired} <- crossing detection is off");
    }
    let want_eng = 2.0 * b * (l - 1.0);
    println!(
        "engrave polylines   : {}  expected 2*B*(L-1) = {} {}",
        engrave.len(),
        ins(want_eng),
        if engrave.len() as f64 == want_eng { "OK" } else { "CHECK" }
    );
    println!(
        "rim continuations   : {}  (decorative; the ribbon crossing its anchors)",
        rim_lines.len()
    );
    println!("ribbon width (mm)   : {}", ins(2.0 * hw));
    // MEASURED, not restated. This printed B, which is the prediction: a line
    // that cannot disagree with itself is not a check, and the header counts
    // it as one of the four. An anchor is one excursion of the ribbon's outer
    // edge past the rim, so count the contiguous runs of centreline sample
    // that stand outside R_HOLE. Cyclic, because the strand closes.
    let outside = |i: usize| hypot(cx[i], cy[i]) + hw > r_hole;
    let anchors = (0..M).filter(|&i| outside(i) && !outside((i + M - 1) % M)).count();
    println!(
        "rim anchors         : {}  expected B = {} {}",
        anchors,
        ins(b),
        if anchors as f64 == b { "OK" } else { "MISMATCH" }
    );
    // --------------------------------------- does it all fit on the page?
    // Every other line in this report describes geometry the generator
    // computed; none of them look at the document it gets written into. On
    // 2026-08-06 the rim continuations reached 1mm past a canvas sized for the
    // cut layer alone, and every invariant kept passing while the files were
    // clipped. Measured from the point arrays rather than by re-parsing the
    // emitted path strings, which is where the arithmetic is actually known.
    let half_canvas = r_hole + pad;
    let max_abs = max_abs(&[simp, &engrave, &rim_lines]);
    let fits = max_abs <= half_canvas;
    println!(
        "content vs canvas   : {}mm of {}mm half-extent {}",
        fixed(max_abs, 2),
        fixed(half_canvas, 2),
        if fits {
            format!("OK ({}mm margin)", fixed(half_canvas - max_abs, 2))
        } else {
            format!("CLIPPED by {}mm -- raise PAD", fixed(max_abs - half_canvas, 2))
        }
    );
    println!("cut path bytes      : {}", cut_d.len());

    // --------------------------------------- and say so in the exit status
    // Every one of these was reported and none of them was acted on: the
    // process exited 0 whatever the report said, so a knot whose weave is
    // wrong looked exactly like a good one to anything that ran the generator
    // rather than read its output. (5,3) prints two MISMATCH lines and used to
    // exit 0.
    let invariants = [
        ("regions are L*B+1", regions_ok),
        ("crossings are B*(L-1)", crossings as f64 == want_cross),
        ("engrave lines are 2*B*(L-1)", engrave.len() as f64 == want_eng),
        ("rim anchors are B", anchors as f64 == b),
        ("the weave alternates", weave_ok),
        ("no unclosed chain", c.open_chains == 0),
        ("nothing falls out", all_ccw),
        ("content fits the canvas", fits),
    ];
    let broken: Vec<&str> = invariants.iter().filter(|(_, ok)| !ok).map(|(w, _)| *w).collect();
    if !broken.is_empty() {
        println!("\n*** {} invariant(s) failed: {}", broken.len(), broken.join("; "));
        println!("    These pin the topology. Do not cut a knot that fails one.");
    }

    if env_flag("DIAG").is_some() {
        contour::print_regions(&grid, &c.lab);
    }
    if broken.is_empty() { 0 } else { 1 }
}
