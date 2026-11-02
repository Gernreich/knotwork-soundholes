//
// plait sound hole -> cut-ready SVG
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
//   knot_soundhole -- any coprime LEADS x BIGHTS.
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
//   (everything after the field is shared with knot_soundhole: src/contour.rs)
//
//   field(x,y) > 0 means "cut away (air)", < 0 means "material".
//
// USAGE   (all knobs are env vars; nothing is written unless OUT is set)
//   cargo run --release --bin plait_soundhole               report only, no file
//   OUT=10-crossing_plait_radius30mm.svg cargo run --release --bin plait_soundhole
//   N=4 OUT=8-crossing_plait_radius30mm.svg cargo run --release --bin plait_soundhole
//   R_HOLE=50 AMP=10.8 HW=3.3 BITE=2.5 OUT=10-crossing_plait_radius50mm.svg
//   DIAG=1 cargo run --release --bin plait_soundhole        per-region dump
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
// RUST SINCE 2026-11-02. This was plait_soundhole.js. The port keeps Node's
// arithmetic exactly (see src/lib.rs), and every shipped SVG redraws
// byte-identically; .repro checks that.

use soundhole::contour::{self, d_of, d_open, f2, max_abs, max_of, min_of, rdp, shoe, Grid, Pt};
use soundhole::jsfmt::{fixed, ins, number, round};
use soundhole::jsmath::{atan2, cos, hypot, sin};
use std::f64::consts::PI;

// ---------------------------------------------------------------- params
// Every knob, in one place: --help prints this list and the defaults below are
// read from it, so the two cannot drift apart.
const PARAMS: [(&str, &str, &str); 7] = [
    ("N", "5", "lobes per ribbon -> 2N crossings and 2N rim anchors"),
    ("R_HOLE", "30", "sound hole radius, mm"),
    ("AMP", "6.5", "radial swing of the weave"),
    ("HW", "2.0", "ribbon half-width -> ribbon is 2*HW mm wide"),
    ("BITE", "1.5", "rim overrun; this is what anchors the rosette"),
    ("NG", "1400", "sampling grid resolution"),
    ("MIN_FEATURE", "0.25", "gaps narrower than this are welded to solid material"),
];
const FLAGS: [(&str, &str); 2] = [
    ("OUT", "output path -- NOTHING IS WRITTEN unless this is set"),
    ("DIAG", "verbose per-region report"),
];

fn help() {
    println!(
        "
plait_soundhole -- two woven ribbons, always an even crossing count

  Settings are environment variables, not flags:

    N=4 OUT=8-crossing_plait_radius30mm.svg cargo run --release --bin plait_soundhole

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
  The crossing count is 2N and is always even -- no parameter changes that. For an
  odd count, or any coprime leads x bights, use knot_soundhole.

  AMP and HW do not scale with R_HOLE. Roughly AMP ~ 0.217*R_HOLE and
  HW ~ 0.067*R_HOLE reproduces the shipped proportions; a guard refuses geometry
  that cannot close and suggests values.

  Full documentation: plait_soundhole.md
"
    );
}

fn env_flag(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.is_empty())
}
fn setting(k: &str) -> f64 {
    let d = PARAMS.iter().find(|p| p.0 == k).unwrap().1;
    number(&env_flag(k).unwrap_or_else(|| d.to_string()))
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
    let amp = setting("AMP"); // radial swing of the weave
    let hw = setting("HW"); // ribbon half-width -> 4mm ribbon
    let n = setting("N"); // lobes per ribbon -> 2N crossings
    let bite = setting("BITE"); // how far ribbon peak overruns the rim
    // anchor the ribbon INTO the soundboard: peak outer edge = R_HOLE + BITE
    let r_mid = r_hole + bite - amp - hw;
    let phi = PI / n; // rotation mapping ribbon A onto ribbon B
    let ng_f = setting("NG");

    // The weave has to fit inside the hole: the ribbon troughs dip to
    // R_MID - AMP - HW, and that has to leave a real opening in the middle.
    let r_centre = r_mid - amp - hw;
    if r_centre < 1.0 {
        eprintln!(
            "Geometry does not close: centre opening would be {}mm.\n  R_HOLE={} BITE={} AMP={} HW={} -> R_MID={}\n  The weave needs AMP + HW well under R_HOLE + BITE. For a hole this\n  size try AMP={} HW={} (the shipped 30mm proportions scaled).",
            fixed(r_centre, 2), ins(r_hole), ins(bite), ins(amp), ins(hw), fixed(r_mid, 2),
            fixed(r_hole * 0.217, 1), fixed(r_hole * 0.067, 1)
        );
        return 1;
    }
    if !(ng_f.is_finite() && ng_f.floor() == ng_f && ng_f >= 1.0) {
        eprintln!("NG must be a whole number >= 1 (got {}).", ins(ng_f));
        return 1;
    }
    let ng = ng_f as usize;
    // smallest hole any cutter can actually make; anything narrower
    // must become solid material, not a micron-wide sliver
    let min_feature = setting("MIN_FEATURE");

    // Centreline curvature at the inner trough. If this drops near HW the
    // ribbon self-fills its own notch and leaves unmanufacturable slivers.
    let r_t = r_mid - amp;
    let rpp_t = amp * n * n;
    let rho_trough = r_t * r_t / (r_t - rpp_t).abs();

    // --------------------------------------------- ribbon A centreline
    const M: usize = 4096;
    let mf = M as f64;
    let mut cx = vec![0.0; M];
    let mut cy = vec![0.0; M];
    for i in 0..M {
        let t = 2.0 * PI * i as f64 / mf;
        let r = r_mid + amp * sin(n * t);
        cx[i] = r * cos(t);
        cy[i] = r * sin(t);
    }
    // distA searches only a window of centreline samples around the query
    // point's own angle, which is what makes the field cheap. The window must
    // be a full half-period: at the tight inner troughs the nearest centreline
    // point can sit a long way off in theta, and too small a window silently
    // overestimates the distance (reported as extra "cut"), which shows up as
    // phantom slivers.
    let win = (mf / (2.0 * n)).ceil() as i64 + 60;

    let dist_a = |x: f64, y: f64| -> f64 {
        let mut th = atan2(y, x);
        if th < 0.0 {
            th += 2.0 * PI;
        }
        let i0 = round(th / (2.0 * PI) * mf);
        let mut best = f64::INFINITY;
        for d in -win..=win {
            let mut i = (i0 + d as f64) % mf;
            if i < 0.0 {
                i += mf;
            }
            let i = i as usize;
            let (dx, dy) = (x - cx[i], y - cy[i]);
            let s = dx * dx + dy * dy;
            if s < best {
                best = s;
            }
        }
        best.sqrt()
    };
    let (cp, sp) = (cos(phi), sin(phi));
    let dist_b = |x: f64, y: f64| dist_a(x * cp + y * sp, -x * sp + y * cp);

    // ---------------------------------------------------------- sample
    // Domain just has to extend past the rim so the border is all material;
    // nothing is ever cut beyond r = R_HOLE.
    let ext = r_hole + 2.0;
    // signed field: >0 = cut away (air), <0 = material
    let jmin = soundhole::jsfmt::min;
    let mut grid = Grid::sample(ng, ext, |x, y| {
        let d = hypot(x, y);
        jmin(r_hole - d, jmin(dist_a(x, y), dist_b(x, y)) - hw)
    });

    // --------------------------- regions, welding, contours, simplification
    let c = contour::contour(&mut grid, min_feature);
    let simp = &c.simp;
    let open_area = simp.iter().fold(0.0, |s, p| s + shoe(p).abs());

    // --------------------------------------- interlace engrave lines
    // Start the scan a quarter-period away from a crossing so no engrave run
    // straddles the array seam and gets split into two polylines.
    let seam = round(mf / (4.0 * n)) as usize;
    let offset_curve = |sign: f64| -> Vec<Pt> {
        (0..M)
            .map(|s0| {
                let i = (s0 + seam) % M;
                let (a, b) = ((i + M - 1) % M, (i + 1) % M);
                let (mut tx, mut ty) = (cx[b] - cx[a], cy[b] - cy[a]);
                let l = hypot(tx, ty);
                tx /= l;
                ty /= l;
                [cx[i] + sign * hw * -ty, cy[i] + sign * hw * tx]
            })
            .collect()
    };
    let rot = |p: &Pt, s: f64| -> Pt {
        [
            p[0] * cos(s * phi) - p[1] * sin(s * phi),
            p[0] * sin(s * phi) + p[1] * cos(s * phi),
        ]
    };
    let cross_idx = |p: &Pt| -> f64 {
        let mut th = atan2(p[1], p[0]);
        if th < 0.0 {
            th += 2.0 * PI;
        }
        round(th / (PI / n)) % (2.0 * n)
    };
    // Interlace convention: at a crossing you engrave the OVER strand's two
    // edges continuing across the under strand's body, which reads as the
    // under strand passing beneath. Crossing k sits at theta = k*pi/N; strand A
    // is over at even k, B at odd k. Alternating like this is only consistent
    // because a strand meets an even number of crossings (2N) on its way
    // round, so it always arrives back where it started with the same
    // over/under parity.
    let mut engrave: Vec<Vec<Pt>> = Vec::new();
    for sign in [1.0, -1.0] {
        let base = offset_curve(sign);
        let rotated: Vec<Pt> = base.iter().map(|p| rot(p, 1.0)).collect();
        // ribbon A edges, engraved where they cross ribbon B and A is "over"
        let passes: [(&Vec<Pt>, &dyn Fn(f64, f64) -> f64, bool); 2] =
            [(&base, &dist_b, true), (&rotated, &dist_a, false)];
        for (curve, other, want_even) in passes {
            let mut run: Vec<Pt> = Vec::new();
            for p in curve {
                let in_other = other(p[0], p[1]) < hw;
                let k = cross_idx(p);
                let over = (k % 2.0 == 0.0) == want_even;
                let keep = in_other && over && hypot(p[0], p[1]) < r_hole - 0.05;
                if keep {
                    run.push(*p);
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
    // Where a ribbon overruns the rim it fuses into the soundboard, and the cut
    // layer stops dead at R_HOLE — past the rim there is nothing to cut. That
    // leaves every anchor reading as a flat pad and the rosette looking sliced
    // off by a circle. The ribbon edges do carry on out there, as far as
    // R_HOLE + BITE, so engrave them: the outline continues across the anchor
    // and the eye reads the ribbon as passing into the board. Both ribbons get
    // it, A directly and B by the same rotation the weave uses.
    //
    // Purely decorative, and kept apart from the interlace lines above, whose
    // count is tied to the crossings. Reported separately so the two are never
    // confused.
    let mut rim_lines: Vec<Vec<Pt>> = Vec::new();
    for sign in [1.0, -1.0] {
        let base = offset_curve(sign);
        let rotated: Vec<Pt> = base.iter().map(|p| rot(p, 1.0)).collect();
        for curve in [&base, &rotated] {
            let mut run: Vec<Pt> = Vec::new();
            for p in curve {
                let r = hypot(p[0], p[1]);
                let keep = r >= r_hole - 0.05 && r <= r_hole + bite + hw;
                if keep {
                    run.push(*p);
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
    }

    // ---------------------------------------------------------- output
    let cut_d: String = simp.iter().map(|p| d_of(p)).collect();
    let eng_d: String = engrave.iter().chain(rim_lines.iter()).map(|p| d_open(p)).collect();

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
  <title>{twon}-crossing plait sound hole - {rh}mm radius</title>
  <!-- 1 user unit = 1mm, so this prints/cuts at true size.
       Sound hole radius {rh}mm ({dia}mm diameter).
       Two interwoven ribbons {width}mm wide, {twon} crossings.
       The ribbon overruns the rim by {bite}mm at {twon} points, fusing it
       into the soundboard, so the rosette cannot drop out when cut.
       Open area {oa}mm2 = {pct}% of a plain {dia}mm hole
       (acoustically equivalent to a plain round hole of
       {eq}mm diameter).
       Narrowest cut region: {narrow}mm across. -->


  <!-- CUT: every closed path here is waste that drops out -->
  <g id="cut" fill="none" stroke="#000000" stroke-width="0.1">
    <path d="{cut_d}"/>
  </g>

  <!-- ENGRAVE (optional): over/under interlace breaks -->
  <g id="engrave" fill="none" stroke="#0000ff" stroke-width="0.1"
     stroke-linecap="round">
    <path d="{eng_d}"/>
  </g>
</svg>
"##,
        twon = ins(2.0 * n),
        rh = ins(r_hole),
        dia = ins(2.0 * r_hole),
        width = ins(2.0 * hw),
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
    // AGAINST THE PREDICTION, not just against itself -- the same fault the
    // sibling knot generator carried, and here there was no prediction printed
    // at all. Contours against flood regions is two measurements of one
    // drawing agreeing; it holds whenever the drawing is self-consistent,
    // whether or not it is the plait that was asked for. A plait is the
    // (2, 2N) row of the knot family, so its region count is L*B+1 = 4N+1: one
    // centre, 2N lenses between the passes and 2N gaps at the rim.
    let want_regions = 4.0 * n + 1.0;
    let regions_ok = simp.len() == c.nreg && c.nreg as f64 == want_regions;
    println!(
        "closed contours     : {}  flood regions: {}  expected 4N+1 = {} {}",
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
    println!("N,AMP,HW,R_MID     : {} {} {} {}", ins(n), ins(amp), ins(hw), fixed(r_mid, 2));
    println!(
        "trough curv. radius: {} mm  (HW = {} ) ratio {}",
        fixed(rho_trough, 3),
        ins(hw),
        fixed(rho_trough / hw, 2)
    );
    println!("tightest region inradius (mm): {}", fixed(min_inrad, 3));
    println!("largest  region inradius (mm): {}", fixed(max_of(&c.inrad), 3));
    println!("open area (mm^2)    : {}", fixed(open_area, 2));
    println!("plain hole area     : {}", fixed(disc_area, 2));
    println!("open fraction       : {}%", fixed(open_area / disc_area * 100.0, 1));
    println!("equiv. round hole dia (mm): {}", fixed(2.0 * (open_area / PI).sqrt(), 2));
    // Two edges of the over pass at each of the 2N crossings, so 4N. The count
    // was printed bare, with nothing to compare it to.
    let want_eng = 4.0 * n;
    println!(
        "engrave polylines   : {}  expected 4N = {} {}",
        engrave.len(),
        ins(want_eng),
        if engrave.len() as f64 == want_eng { "OK" } else { "CHECK" }
    );
    println!(
        "rim continuations   : {}  (decorative; the ribbons crossing their anchors)",
        rim_lines.len()
    );
    println!("ribbon width (mm)   : {}", ins(2.0 * hw));
    // MEASURED, not restated. This printed 2*N, which is the prediction -- a
    // line that cannot disagree with itself. An anchor is one excursion of a
    // ribbon's outer edge past the rim, so count the contiguous runs of sample
    // standing outside R_HOLE, on both ribbons. B is A rotated by pi/N, and
    // sin(N*t - pi) is -sin(N*t), which is the second radius below.
    let mut anchors = 0usize;
    {
        let outside = |r: f64| r + hw > r_hole;
        for sign in [1.0, -1.0] {
            let r = |i: usize| r_mid + sign * amp * sin(n * 2.0 * PI * i as f64 / mf);
            for i in 0..M {
                if outside(r(i)) && !outside(r((i + M - 1) % M)) {
                    anchors += 1;
                }
            }
        }
    }
    let want_anchors = 2.0 * n;
    println!(
        "rim anchors         : {}  expected 2N = {} {}",
        anchors,
        ins(want_anchors),
        if anchors as f64 == want_anchors { "OK" } else { "MISMATCH" }
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
    // The report was written to be read and nothing acted on it: the process
    // exited 0 whatever it said, so anything that RAN this generator rather
    // than reading its output could not tell a good plait from a broken one.
    let invariants = [
        ("regions are 4N+1", regions_ok),
        ("engrave lines are 4N", engrave.len() as f64 == want_eng),
        ("rim anchors are 2N", anchors as f64 == want_anchors),
        ("no unclosed chain", c.open_chains == 0),
        ("nothing falls out", all_ccw),
        ("content fits the canvas", fits),
    ];
    let broken: Vec<&str> = invariants.iter().filter(|(_, ok)| !ok).map(|(w, _)| *w).collect();
    if !broken.is_empty() {
        println!("\n*** {} invariant(s) failed: {}", broken.len(), broken.join("; "));
        println!("    These pin the topology. Do not cut a plait that fails one.");
    }

    if env_flag("DIAG").is_some() {
        contour::print_regions(&grid, &c.lab);
        let mut areas: Vec<String> = simp.iter().map(|p| fixed(shoe(p).abs(), 3)).collect();
        areas.sort_by(|a, b| {
            number(a).partial_cmp(&number(b)).unwrap_or(std::cmp::Ordering::Equal)
        });
        println!("\ncontour areas: {}", areas.join(" "));
    }
    if broken.is_empty() { 0 } else { 1 }
}
