// Build the display rendering of a cut file in this repository.
//
//   cargo run --release --bin preview -- 2-lead_3-bight_knot_radius30mm.svg [OUT.svg]
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
//
// RUST SINCE 2026-11-02. This was preview.js; every preview redraws
// byte-identically.

use soundhole::jsfmt::{fixed, num, number};
use std::path::{Path, PathBuf};

const GROUND: &str = "#ffffff"; // white: the waste that drops out (cream until 2026-10-13)
const BOARD: &str = "#e6d9ae"; // pale gold: the soundboard the ribbon fuses into
const ROSETTE: &str = "#c9a227"; // deep gold: the ribbon itself, inside the rim

/// The text between `open` and the next `close`, from the first `open`.
fn between<'a>(s: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = s.find(open)? + open.len();
    let b = s[a..].find(close)?;
    Some(&s[a..a + b])
}

fn build(src: &str, src_name: &str) -> Result<String, String> {
    // viewBox="([^"]+)"
    let vb = between(src, "viewBox=\"", "\"")
        .filter(|v| !v.is_empty())
        .ok_or(format!("no viewBox in {src_name}"))?;
    let nums: Vec<f64> = vb
        .trim()
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .map(number)
        .collect();
    let at = |k: usize| nums.get(k).copied().unwrap_or(f64::NAN);
    let (x, y, w, h) = (at(0), at(1), at(2), at(3));

    // <title>([^<]*)</title>, else the file name
    let title = between(src, "<title>", "</title>")
        .filter(|t| !t.contains('<'))
        .unwrap_or(src_name);

    // The bite is stated in the cut file's own comment, in the sentence that
    // says what anchors the rosette. Parsed rather than passed in: the two
    // cannot then disagree about the drawing they describe.
    let bite = {
        let key = "overruns the rim by ";
        let mut found = None;
        let mut from = 0;
        while let Some(p) = src[from..].find(key) {
            let rest = &src[from + p + key.len()..];
            let n = rest.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(rest.len());
            if n > 0 && rest[n..].starts_with("mm") {
                found = Some(&rest[..n]);
                break;
            }
            from += p + 1;
        }
        found.ok_or(format!("no rim overrun stated in {src_name}"))?
    };

    // The cut layer only. The engrave layer suggests the over/under weave and
    // is not material; filling it would paint the interlace hints as if they
    // were ribbon.
    let cut_group = {
        let start = src.find("<g id=\"cut\"").ok_or(format!("no <g id=\"cut\"> in {src_name}"))?;
        let rest = &src[start..];
        let gt = rest.find('>').ok_or(format!("no <g id=\"cut\"> in {src_name}"))?;
        let body = &rest[gt + 1..];
        let end = body.find("</g>").ok_or(format!("no <g id=\"cut\"> in {src_name}"))?;
        &body[..end]
    };
    // \bd="([^"]+)"
    let mut paths: Vec<&str> = Vec::new();
    let bytes = cut_group.as_bytes();
    let mut from = 0;
    while let Some(p) = cut_group[from..].find("d=\"") {
        let at = from + p;
        let word_before = at > 0 && {
            let c = bytes[at - 1];
            c.is_ascii_alphanumeric() || c == b'_'
        };
        let val_start = at + 3;
        if !word_before {
            if let Some(len) = cut_group[val_start..].find('"') {
                if len > 0 {
                    paths.push(&cut_group[val_start..val_start + len]);
                    from = val_start + len + 1;
                    continue;
                }
            }
        }
        from = at + 1;
    }
    if paths.is_empty() {
        return Err(format!("the cut layer of {src_name} is empty"));
    }

    // An even-odd fill needs something to be outside-of. The frame is that
    // something: every closed contour inside it reverses the parity, so the
    // waste comes out white and the ribbon comes out gold without either being
    // named.
    let f1 = |n: f64| fixed(n, 1);
    let frame = format!("M{} {}H{}V{}H{}Z", f1(x), f1(y), f1(x + w), f1(y + h), f1(x));
    let body = frame + &paths.concat();

    // The two golds part AT THE RIM, and the ribbon runs on through it.
    // Clipping the rosette to r = R_HOLE + BITE rather than to R_HOLE is the
    // whole point: bounding it at the hole sliced every lobe tip flat along a
    // circle and drew a rosette that does not exist. The viewBox is padded by
    // exactly half a millimetre past that, which is where this reads it from.
    let rim = num(w / 2.0 - 0.5);

    let mm = |n: f64| {
        let s = f1(n);
        s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
    };
    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}mm" height="{h}mm" viewBox="{vb}">
  <title>{title} — display only</title>
  <!-- DISPLAY ONLY - not a cut file. The cut layer of {src_name}, filled even-odd.
       Deep gold is the rosette, pale gold the board it fuses into, white the waste
       that drops out. The ribbon overruns the rim by BITE ({bite}mm here) and becomes
       board, so the two golds part at r={rim}mm with the ribbon running straight on
       through - bounding the fill at R_HOLE instead sliced every lobe tip flat along a
       circle and showed a rosette that does not exist. The white is painted, not
       transparent, so the picture reads the same on a light or a dark page.
       Cut files carry no fill; browsing one shows a transparency checkerboard. -->
  <defs><clipPath id="rosette"><circle cx="0" cy="0" r="{rim}"/></clipPath></defs>
  <rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" fill="{GROUND}"/>
  <path fill="{BOARD}" fill-rule="evenodd" d="{body}"/>
  <g clip-path="url(#rosette)"><path fill="{ROSETTE}" fill-rule="evenodd" d="{body}"/></g>
</svg>
"##,
        w = mm(w),
        h = mm(h),
        rx = f1(x),
        ry = f1(y),
        rw = f1(w),
        rh = f1(h),
    ))
}

/// Where Node's path.join(path.dirname(src), 'previews', name) lands.
fn default_out(src: &str, name: &str) -> PathBuf {
    let dir = Path::new(src).parent().unwrap_or(Path::new(""));
    let mut p = PathBuf::new();
    for c in dir.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir if p.file_name().is_some() => {
                p.pop();
            }
            other => p.push(other),
        }
    }
    p.join("previews").join(name)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(src) = args.get(1) else {
        eprintln!("usage: cargo run --release --bin preview -- CUTFILE.svg [OUT.svg]");
        eprintln!("  Writes previews/<name> unless an output path is given.");
        std::process::exit(2);
    };
    let name = Path::new(src).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let out = match args.get(2).filter(|s| !s.is_empty()) {
        Some(o) => PathBuf::from(o),
        None => default_out(src, &name),
    };
    let text = match std::fs::read_to_string(src) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {src}: {e}");
            std::process::exit(1);
        }
    };
    let svg = match build(&text, &name) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(&out, &svg) {
        eprintln!("cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    // kB as JavaScript counts string length: UTF-16 code units, not bytes.
    let units = svg.encode_utf16().count();
    println!("  {}  ({} kB)", out.display(), fixed(units as f64 / 1024.0, 1));
}
