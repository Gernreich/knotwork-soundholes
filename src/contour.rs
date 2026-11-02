//! From a sampled signed field to cut contours: the pipeline both generators
//! share, line for line what each JavaScript generator carried its own copy of.
//!
//!   flood fill regions -> weld unmanufacturable slivers shut ->
//!   marching squares -> chain segments into loops -> RDP simplify
//!
//! field > 0 means "cut away (air)", < 0 means "material".
//!
//! Two bugs this pipeline once caught, both silent, both worth not
//! reintroducing: chaining loops on rounded float coordinates (shared cell
//! edges disagree in the last bits), and running plain RDP on a closed loop
//! (see `rdp_closed`).

use crate::jsfmt::{fixed, num, round};
use crate::jsmath::{atan2, hypot};
use std::collections::HashMap;

pub type Pt = [f64; 2];

/// The field sampled on an (NG+1) x (NG+1) grid over [-EXT, EXT]^2.
pub struct Grid {
    pub ng: usize,
    pub ext: f64,
    pub step: f64,
    pub f: Vec<f64>,
}

impl Grid {
    pub fn sample(ng: usize, ext: f64, field: impl Fn(f64, f64) -> f64) -> Grid {
        let step = (2.0 * ext) / ng as f64;
        let mut g = Grid { ng, ext, step, f: vec![0.0; (ng + 1) * (ng + 1)] };
        for j in 0..=ng {
            for i in 0..=ng {
                g.f[j * (ng + 1) + i] = field(g.gx(i), g.gx(j));
            }
        }
        g
    }
    #[inline]
    pub fn gx(&self, i: usize) -> f64 {
        -self.ext + i as f64 * self.step
    }
    #[inline]
    fn at(&self, i: usize, j: usize) -> f64 {
        self.f[j * (self.ng + 1) + i]
    }
}

pub struct Region {
    pub inr: f64,
    pub cells: Vec<usize>,
}

/// Connected cut regions (4-neighbour), each with its largest field value,
/// which is its inradius.
pub fn flood_regions(g: &Grid) -> (Vec<i32>, Vec<Region>) {
    let n1 = g.ng + 1;
    let mut lab = vec![-1i32; n1 * n1];
    let mut regs: Vec<Region> = Vec::new();
    for j0 in 0..n1 {
        for i0 in 0..n1 {
            let id0 = j0 * n1 + i0;
            if g.f[id0] <= 0.0 || lab[id0] != -1 {
                continue;
            }
            let rid = regs.len() as i32;
            let mut st = vec![id0];
            let mut cells = Vec::new();
            lab[id0] = rid;
            let mut mx = 0.0f64;
            while let Some(id) = st.pop() {
                let i = id % n1;
                let j = (id - i) / n1;
                cells.push(id);
                if g.f[id] > mx {
                    mx = g.f[id];
                }
                let (i, j) = (i as i64, j as i64);
                for (a, b) in [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)] {
                    if a < 0 || b < 0 || a > g.ng as i64 || b > g.ng as i64 {
                        continue;
                    }
                    let q = b as usize * n1 + a as usize;
                    if g.f[q] > 0.0 && lab[q] == -1 {
                        lab[q] = rid;
                        st.push(q);
                    }
                }
            }
            regs.push(Region { inr: mx, cells });
        }
    }
    (lab, regs)
}

/// Everything the contouring produces, plus the measurements the reports use.
pub struct Contours {
    pub lab: Vec<i32>,
    pub nreg: usize,
    pub inrad: Vec<f64>,
    pub welded: usize,
    pub flood_area: f64,
    pub open_chains: usize,
    /// Simplified closed loops, each ending on its first point.
    pub simp: Vec<Vec<Pt>>,
}

/// Weld regions narrower than `min_feature` shut, then contour what is left.
pub fn contour(g: &mut Grid, min_feature: f64) -> Contours {
    // Flood fill FIRST so sub-cutter-width slivers can be welded shut before
    // contouring; otherwise they emit unmanufacturable micro-loops.
    let (_, regs) = flood_regions(g);
    let mut welded = 0;
    for r in &regs {
        if r.inr >= min_feature {
            continue; // keep: physically cuttable
        }
        for &id in &r.cells {
            g.f[id] = -1e-9; // weld shut: becomes material
        }
        welded += 1;
    }
    let (lab, after) = flood_regions(g);
    let nreg = after.len();
    let inrad: Vec<f64> = after.iter().map(|r| r.inr).collect();
    let cut_cells = g.f.iter().filter(|v| **v > 0.0).count();
    let flood_area = cut_cells as f64 * g.step * g.step;

    // ---------------------------------------------- marching squares
    // corners bl,br,tr,tl = bits 1,2,4,8 ; edges 0=bottom 1=right 2=top 3=left
    //
    // INVARIANT: every segment is emitted [from, to] oriented so that the
    // INSIDE (F > 0, air) lies on its LEFT. Two things depend on this and break
    // silently if an entry is reversed:
    //   - chaining matches one segment's end to the next one's start, so a
    //     reversed entry dead-ends the walk (shows up as "unclosed chains");
    //   - consistent orientation makes every cut contour come out counter-
    //     clockwise, which is what lets a clockwise contour be detected as a
    //     loose island of material that would fall out.
    // Cases 5 and 10 are the ambiguous saddles, resolved by the cell-centre
    // value; the centre-inside branch of case 5 is the one that was wrong first
    // time round, so re-derive rather than trust it if you touch this.
    fn table(c: u8) -> &'static [(u8, u8)] {
        match c {
            1 => &[(0, 3)],
            2 => &[(1, 0)],
            4 => &[(2, 1)],
            8 => &[(3, 2)],
            3 => &[(1, 3)],
            6 => &[(2, 0)],
            12 => &[(3, 1)],
            9 => &[(0, 2)],
            7 => &[(2, 3)],
            14 => &[(3, 0)],
            13 => &[(0, 1)],
            11 => &[(1, 2)],
            _ => &[],
        }
    }
    // Every contour vertex lies on a grid edge. Identify it by an integer edge
    // id and compute its point from the canonical corner order, so both cells
    // sharing an edge get a bit-identical vertex and chaining is exact.
    let ng = g.ng;
    let n1 = ng + 1;
    let nh = n1 * n1;
    let h_id = |i: usize, j: usize| j * n1 + i; // horizontal edge (i,j)-(i+1,j)
    let v_id = |i: usize, j: usize| nh + j * n1 + i; // vertical edge (i,j)-(i,j+1)
    let gid = |e: u8, i: usize, j: usize| match e {
        0 => h_id(i, j),
        1 => v_id(i + 1, j),
        2 => h_id(i, j + 1),
        _ => v_id(i, j),
    };
    let edge_point = |id: usize| -> Pt {
        if id < nh {
            let i = id % n1;
            let j = (id - i) / n1;
            let (a, b) = (g.at(i, j), g.at(i + 1, j));
            [g.gx(i) + g.step * (a / (a - b)), g.gx(j)]
        } else {
            let r = id - nh;
            let i = r % n1;
            let j = (r - i) / n1;
            let (a, b) = (g.at(i, j), g.at(i, j + 1));
            [g.gx(i), g.gx(j) + g.step * (a / (a - b))]
        }
    };

    let mut segs: Vec<(usize, usize)> = Vec::new();
    for j in 0..ng {
        for i in 0..ng {
            let f = [g.at(i, j), g.at(i + 1, j), g.at(i + 1, j + 1), g.at(i, j + 1)];
            let mut c = 0u8;
            if f[0] > 0.0 {
                c |= 1;
            }
            if f[1] > 0.0 {
                c |= 2;
            }
            if f[2] > 0.0 {
                c |= 4;
            }
            if f[3] > 0.0 {
                c |= 8;
            }
            if c == 0 || c == 15 {
                continue;
            }
            let list: &[(u8, u8)] = if c == 5 || c == 10 {
                let ctr = (f[0] + f[1] + f[2] + f[3]) / 4.0 > 0.0;
                match (c, ctr) {
                    (5, true) => &[(0, 1), (2, 3)],
                    (5, false) => &[(0, 3), (2, 1)],
                    (_, true) => &[(3, 0), (1, 2)],
                    (_, false) => &[(1, 0), (3, 2)],
                }
            } else {
                table(c)
            };
            for &(a, b) in list {
                segs.push((gid(a, i, j), gid(b, i, j)));
            }
        }
    }

    // ---------------------------------------------- chain segments
    let mut by_start: HashMap<usize, Vec<usize>> = HashMap::new();
    for (k, s) in segs.iter().enumerate() {
        by_start.entry(s.0).or_default().push(k);
    }
    let mut used = vec![false; segs.len()];
    let mut loops: Vec<Vec<Pt>> = Vec::new();
    let mut open_chains = 0;
    for s in 0..segs.len() {
        if used[s] {
            continue;
        }
        let start = segs[s].0;
        let mut ids = vec![start];
        let mut cur = s;
        let mut closed = false;
        loop {
            used[cur] = true;
            let end = segs[cur].1;
            if end == start {
                closed = true;
                break;
            }
            ids.push(end);
            let Some(cand) = by_start.get(&end) else { break };
            let Some(&nxt) = cand.iter().find(|&&k| !used[k]) else { break };
            cur = nxt;
        }
        if !closed {
            open_chains += 1;
            continue;
        }
        if ids.len() >= 3 {
            let mut pts: Vec<Pt> = ids.iter().map(|&id| edge_point(id)).collect();
            pts.push(edge_point(start));
            loops.push(pts);
        }
    }

    let simp = loops.iter().map(|l| rdp_closed(l, 0.015)).collect();
    Contours { lab, nreg, inrad, welded, flood_area, open_chains, simp }
}

/// Ramer-Douglas-Peucker on an open polyline.
pub fn rdp(pts: &[Pt], eps: f64) -> Vec<Pt> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut idx = 0;
    let mut dmax = 0.0;
    let [ax, ay] = pts[0];
    let [bx, by] = pts[pts.len() - 1];
    let l = hypot(bx - ax, by - ay);
    let l = if l == 0.0 || l.is_nan() { 1.0 } else { l }; // `|| 1`
    for (i, p) in pts.iter().enumerate().take(pts.len() - 1).skip(1) {
        let d = ((bx - ax) * (ay - p[1]) - (ax - p[0]) * (by - ay)).abs() / l;
        if d > dmax {
            dmax = d;
            idx = i;
        }
    }
    if dmax > eps {
        let mut l = rdp(&pts[..=idx], eps);
        let r = rdp(&pts[idx..], eps);
        l.pop();
        l.extend(r);
        return l;
    }
    vec![pts[0], pts[pts.len() - 1]]
}

/// RDP on a CLOSED loop must not use pts[0]..pts[last] as the baseline: they
/// are the same point, every perpendicular distance is 0, and the whole loop
/// collapses to two points. Split at the antipode and simplify each half.
pub fn rdp_closed(pts: &[Pt], eps: f64) -> Vec<Pt> {
    let n = pts.len() - 1; // pts[n] == pts[0]
    if n < 4 {
        return pts.to_vec();
    }
    let k = n >> 1;
    let mut a = rdp(&pts[..=k], eps);
    let b = rdp(&pts[k..], eps);
    a.pop();
    a.extend(b);
    a
}

/// Signed area of a closed polyline: positive is counter-clockwise.
pub fn shoe(p: &[Pt]) -> f64 {
    let mut a = 0.0;
    for i in 0..p.len() - 1 {
        a += p[i][0] * p[i + 1][1] - p[i + 1][0] * p[i][1];
    }
    a / 2.0
}

/// A coordinate in the SVG: three decimals, trailing zeros dropped.
pub fn f2(n: f64) -> String {
    num(round(n * 1000.0) / 1000.0)
}

fn path(p: &[Pt]) -> String {
    let parts: Vec<String> = p.iter().map(|q| format!("{} {}", f2(q[0]), f2(q[1]))).collect();
    format!("M{}", parts.join("L"))
}

/// Closed path data.
pub fn d_of(p: &[Pt]) -> String {
    path(p) + "Z"
}

/// Open path data.
pub fn d_open(p: &[Pt]) -> String {
    path(p)
}

/// Largest |x| or |y| over every point drawn, for the canvas check.
pub fn max_abs(groups: &[&[Vec<Pt>]]) -> f64 {
    let mut m = 0.0f64;
    for group in groups {
        for poly in group.iter() {
            for p in poly {
                let v = crate::jsfmt::max(p[0].abs(), p[1].abs());
                if v > m {
                    m = v;
                }
            }
        }
    }
    m
}

/// `Math.min(...xs)`, `Math.max(...xs)`.
pub fn min_of(xs: &[f64]) -> f64 {
    xs.iter().fold(f64::INFINITY, |a, &b| crate::jsfmt::min(a, b))
}
pub fn max_of(xs: &[f64]) -> f64 {
    xs.iter().fold(f64::NEG_INFINITY, |a, &b| crate::jsfmt::max(a, b))
}

/// DIAG: one line per region, sorted by inradius.
pub fn print_regions(g: &Grid, lab: &[i32]) {
    struct Info {
        n: usize,
        mx: f64,
        sx: f64,
        sy: f64,
    }
    let mut info: Vec<Option<Info>> = Vec::new();
    for j in 0..=g.ng {
        for i in 0..=g.ng {
            let id = j * (g.ng + 1) + i;
            let l = lab[id];
            if l < 0 {
                continue;
            }
            let l = l as usize;
            if info.len() <= l {
                info.resize_with(l + 1, || None);
            }
            let o = info[l].get_or_insert(Info { n: 0, mx: 0.0, sx: 0.0, sy: 0.0 });
            o.n += 1;
            o.sx += g.gx(i);
            o.sy += g.gx(j);
            if g.f[id] > o.mx {
                o.mx = g.f[id];
            }
        }
    }
    println!("\n-- regions (sorted by inradius) --");
    let mut rows: Vec<(usize, f64, f64, f64, f64)> = info
        .iter()
        .enumerate()
        .filter_map(|(k, o)| o.as_ref().map(|o| (k, o)))
        .map(|(k, o)| {
            let n = o.n as f64;
            (
                k,
                n * g.step * g.step,
                o.mx,
                hypot(o.sx / n, o.sy / n),
                atan2(o.sy / n, o.sx / n) * 180.0 / std::f64::consts::PI,
            )
        })
        .collect();
    rows.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    for (k, area, inr, r, th) in rows {
        println!(
            "reg{:>2} inr={}mm area={}mm2 centroid r={} th={}",
            k,
            fixed(inr, 4),
            fixed(area, 3),
            fixed(r, 2),
            fixed(th, 1)
        );
    }
}

/// Run `f` on a thread with a large stack: RDP recurses once per kept vertex.
pub fn with_big_stack<F: FnOnce() -> i32 + Send + 'static>(f: F) -> i32 {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(f)
        .expect("spawn")
        .join()
        .unwrap_or(101)
}
