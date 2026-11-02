//! sin, cos, atan, atan2 and hypot exactly as V8 (Node's JavaScript engine)
//! computes them, bit for bit.
//!
//! The sine, cosine and arctangent are fdlibm 5.3 as carried in V8's
//! src/base/ieee754.cc (Node builds V8 without its glibc trig option, so these
//! are the ones `Math.sin` and friends run). macOS's libm and the `libm` crate
//! (a musl port, whose __kernel_cos was rewritten) disagree with it in the
//! last bit on a few percent of inputs. hypot is V8's Torque builtin: scale by
//! the largest argument, Kahan-sum the squares, take the square root.
//!
//! WHY THIS AND NOT std, MEASURED 2026-11-02 rather than assumed. Graded
//! against a 512-bit reference on 400,000 inputs, none of these is more
//! accurate than the platform's: cos and sin are within 0.80 ulp everywhere,
//! atan2 1.22 ulp and hypot 1.84 ulp, where macOS manages 0.65 and 1.18. And
//! swapping in std for all four changes NONE of the 13 shipped cut files --
//! not one of their 39,886 numbers. The output only starts to move when the
//! maths is off by about 1e-7, nine orders of magnitude above a last-bit
//! difference. So this module is not what makes the files reproduce today.
//! It is kept because it removes the question: with it the generators compute
//! what Node computed, bit for bit, for ANY settings and on any machine, so a
//! difference from a JavaScript-era drawing can only ever mean a code change.
//!
//! FUSED MULTIPLY-ADD. Node for Apple silicon is compiled by clang, whose
//! default -ffp-contract=on turns `a*b + c` WITHIN ONE C EXPRESSION into a
//! single fma (one rounding instead of two). Every such expression below is
//! written as mul_add, following clang's rule: the left operand is fused if
//! it is a product, otherwise the right; `a - b*c` becomes fma(-b, c, a).
//! Leaving them unfused differs from Node on ~0.3% of inputs. JavaScript
//! arithmetic itself is never fused by V8, and neither is hypot.
//!
//! Original fdlibm notice: Copyright (C) 1993 by Sun Microsystems, Inc.
//! Developed at SunSoft, a Sun Microsystems, Inc. business. Permission to use,
//! copy, modify, and distribute this software is freely granted, provided that
//! this notice is preserved.

#![allow(clippy::excessive_precision, clippy::many_single_char_names)]

#[inline]
fn high(d: f64) -> i32 {
    (d.to_bits() >> 32) as u32 as i32
}
#[inline]
fn low(d: f64) -> u32 {
    d.to_bits() as u32
}
#[inline]
fn from_words(hi: i32, lo: u32) -> f64 {
    f64::from_bits(((hi as u32 as u64) << 32) | lo as u64)
}
#[inline]
fn set_high(d: f64, hi: i32) -> f64 {
    from_words(hi, low(d))
}
#[inline]
fn set_low(d: f64, lo: u32) -> f64 {
    from_words(high(d), lo)
}

/// x * 2^n, exactly as C scalbn does for normal results.
fn scalbn(x: f64, mut n: i32) -> f64 {
    let p1023 = f64::from_bits(0x7FE0_0000_0000_0000); // 2^1023
    let m1022 = f64::from_bits(0x0010_0000_0000_0000); // 2^-1022
    let p53 = f64::from_bits(0x4340_0000_0000_0000); // 2^53
    let mut y = x;
    if n > 1023 {
        y *= p1023;
        n -= 1023;
        if n > 1023 {
            y *= p1023;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        y *= m1022 * p53;
        n += 1022 - 53;
        if n < -1022 {
            y *= m1022 * p53;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    y * f64::from_bits(((0x3FF + n) as u64) << 52)
}

const TWO_OVER_PI: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163,
    0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C,
    0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292,
    0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA,
    0x73A8C9, 0x60E27B, 0xC08C6B,
];

const NPIO2_HW: [i32; 32] = [
    0x3FF921FB, 0x400921FB, 0x4012D97C, 0x401921FB, 0x401F6A7A, 0x4022D97C, 0x4025FDBB,
    0x402921FB, 0x402C463A, 0x402F6A7A, 0x4031475C, 0x4032D97C, 0x40346B9C, 0x4035FDBB,
    0x40378FDB, 0x403921FB, 0x403AB41B, 0x403C463A, 0x403DD85A, 0x403F6A7A, 0x40407E4C,
    0x4041475C, 0x4042106C, 0x4042D97C, 0x4043A28C, 0x40446B9C, 0x404534AC, 0x4045FDBB,
    0x4046C6CB, 0x40478FDB, 0x404858EB, 0x404921FB,
];

fn rem_pio2(x: f64, y: &mut [f64; 2]) -> i32 {
    const HALF: f64 = 5.00000000000000000000e-01;
    const TWO24: f64 = 1.67772160000000000000e+07;
    const INVPIO2: f64 = 6.36619772367581382433e-01;
    const PIO2_1: f64 = 1.57079632673412561417e+00;
    const PIO2_1T: f64 = 6.07710050650619224932e-11;
    const PIO2_2: f64 = 6.07710050630396597660e-11;
    const PIO2_2T: f64 = 2.02226624879595063154e-21;
    const PIO2_3: f64 = 2.02226624871116645580e-21;
    const PIO2_3T: f64 = 8.47842766036889956997e-32;

    let hx = high(x);
    let ix = hx & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }
    if ix < 0x4002D97C {
        if hx > 0 {
            let mut z = x - PIO2_1;
            if ix != 0x3FF921FB {
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        } else {
            let mut z = x + PIO2_1;
            if ix != 0x3FF921FB {
                y[0] = z + PIO2_1T;
                y[1] = (z - y[0]) + PIO2_1T;
            } else {
                z += PIO2_2;
                y[0] = z + PIO2_2T;
                y[1] = (z - y[0]) + PIO2_2T;
            }
            return -1;
        }
    }
    if ix <= 0x413921FB {
        let mut t = x.abs();
        let n = t.mul_add(INVPIO2, HALF) as i32;
        let fn_ = n as f64;
        let mut r = (-fn_).mul_add(PIO2_1, t);
        let mut w = fn_ * PIO2_1T;
        if n < 32 && ix != NPIO2_HW[(n - 1) as usize] {
            y[0] = r - w;
        } else {
            let j = ix >> 20;
            y[0] = r - w;
            let mut i = j - ((high(y[0]) >> 20) & 0x7FF);
            if i > 16 {
                t = r;
                w = fn_ * PIO2_2;
                r = t - w;
                w = fn_.mul_add(PIO2_2T, -((t - r) - w));
                y[0] = r - w;
                i = j - ((high(y[0]) >> 20) & 0x7FF);
                if i > 49 {
                    t = r;
                    w = fn_ * PIO2_3;
                    r = t - w;
                    w = fn_.mul_add(PIO2_3T, -((t - r) - w));
                    y[0] = r - w;
                }
            }
        }
        y[1] = (r - y[0]) - w;
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            return -n;
        }
        return n;
    }
    if ix >= 0x7FF00000 {
        y[0] = x - x;
        y[1] = y[0];
        return 0;
    }
    let mut z = set_low(0.0, low(x));
    let e0 = (ix >> 20) - 1046;
    z = set_high(z, ix - ((e0 as u32) << 20) as i32);
    let mut tx = [0.0f64; 3];
    for t in tx.iter_mut().take(2) {
        *t = (z as i32) as f64;
        z = (z - *t) * TWO24;
    }
    tx[2] = z;
    let mut nx = 3;
    while tx[nx - 1] == 0.0 {
        nx -= 1;
    }
    let mut yy = [0.0f64; 3];
    let n = kernel_rem_pio2(&tx, &mut yy, e0, nx as i32, 2);
    y[0] = yy[0];
    y[1] = yy[1];
    if hx < 0 {
        y[0] = -y[0];
        y[1] = -y[1];
        return -n;
    }
    n
}

fn kernel_rem_pio2(x: &[f64], y: &mut [f64; 3], e0: i32, nx: i32, prec: usize) -> i32 {
    const INIT_JK: [i32; 4] = [2, 3, 4, 6];
    const PIO2: [f64; 8] = [
        1.57079625129699707031e+00,
        7.54978941586159635335e-08,
        5.39030252995776476554e-15,
        3.28200341580791294123e-22,
        1.27065575308067607349e-29,
        1.22933308981111328932e-36,
        2.73370053816464559624e-44,
        2.16741683877804819444e-51,
    ];
    const TWO24: f64 = 1.67772160000000000000e+07;
    const TWON24: f64 = 5.96046447753906250000e-08;
    let ipio2 = &TWO_OVER_PI;

    let mut iq = [0i32; 20];
    let mut f = [0.0f64; 20];
    let mut fq = [0.0f64; 20];
    let mut q = [0.0f64; 20];

    let jk = INIT_JK[prec];
    let jp = jk;
    let jx = nx - 1;
    let mut jv = (e0 - 3) / 24;
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 24 * (jv + 1);
    let mut j = jv - jx;
    let m = jx + jk;
    for i in 0..=m {
        f[i as usize] = if j < 0 { 0.0 } else { ipio2[j as usize] as f64 };
        j += 1;
    }
    for i in 0..=jk {
        let mut fw = 0.0;
        for j in 0..=jx {
            fw = x[j as usize].mul_add(f[(jx + i - j) as usize], fw);
        }
        q[i as usize] = fw;
    }
    let mut jz = jk;
    let (mut z, mut n, mut ih);
    loop {
        // recompute:
        let mut i = 0;
        let mut j = jz;
        z = q[jz as usize];
        while j > 0 {
            let fw = ((TWON24 * z) as i32) as f64;
            iq[i] = (-TWO24).mul_add(fw, z) as i32;
            z = q[(j - 1) as usize] + fw;
            i += 1;
            j -= 1;
        }
        z = scalbn(z, q0);
        z = (-8.0f64).mul_add((z * 0.125).floor(), z);
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0 {
            let i = iq[(jz - 1) as usize] >> (24 - q0);
            n += i;
            iq[(jz - 1) as usize] -= i << (24 - q0);
            ih = iq[(jz - 1) as usize] >> (23 - q0);
        } else if q0 == 0 {
            ih = iq[(jz - 1) as usize] >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }
        if ih > 0 {
            n += 1;
            let mut carry = 0;
            for i in 0..jz as usize {
                let j = iq[i];
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        iq[i] = 0x1000000 - j;
                    }
                } else {
                    iq[i] = 0xFFFFFF - j;
                }
            }
            if q0 > 0 {
                match q0 {
                    1 => iq[(jz - 1) as usize] &= 0x7FFFFF,
                    2 => iq[(jz - 1) as usize] &= 0x3FFFFF,
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1.0 - z;
                if carry != 0 {
                    z -= scalbn(1.0, q0);
                }
            }
        }
        if z == 0.0 {
            let mut j = 0;
            let mut i = jz - 1;
            while i >= jk {
                j |= iq[i as usize];
                i -= 1;
            }
            if j == 0 {
                let mut k = 1;
                while jk >= k && iq[(jk - k) as usize] == 0 {
                    k += 1;
                }
                for i in (jz + 1)..=(jz + k) {
                    f[(jx + i) as usize] = ipio2[(jv + i) as usize] as f64;
                    let mut fw = 0.0;
                    for j in 0..=jx {
                        fw = x[j as usize].mul_add(f[(jx + i - j) as usize], fw);
                    }
                    q[i as usize] = fw;
                }
                jz += k;
                continue;
            }
        }
        break;
    }
    if z == 0.0 {
        jz -= 1;
        q0 -= 24;
        while iq[jz as usize] == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        z = scalbn(z, -q0);
        if z >= TWO24 {
            let fw = ((TWON24 * z) as i32) as f64;
            iq[jz as usize] = (-TWO24).mul_add(fw, z) as i32;
            jz += 1;
            q0 += 24;
            iq[jz as usize] = fw as i32;
        } else {
            iq[jz as usize] = z as i32;
        }
    }
    let mut fw = scalbn(1.0, q0);
    let mut i = jz;
    while i >= 0 {
        q[i as usize] = fw * iq[i as usize] as f64;
        fw *= TWON24;
        i -= 1;
    }
    let mut i = jz;
    while i >= 0 {
        let mut fw = 0.0;
        let mut k = 0;
        while k <= jp && k <= jz - i {
            fw = PIO2[k as usize].mul_add(q[(i + k) as usize], fw);
            k += 1;
        }
        fq[(jz - i) as usize] = fw;
        i -= 1;
    }
    // prec is always 2 here (double precision).
    let mut fw = 0.0;
    let mut i = jz;
    while i >= 0 {
        fw += fq[i as usize];
        i -= 1;
    }
    y[0] = if ih == 0 { fw } else { -fw };
    fw = fq[0] - fw;
    for i in 1..=jz {
        fw += fq[i as usize];
    }
    y[1] = if ih == 0 { fw } else { -fw };
    n & 7
}

fn kernel_cos(x: f64, y: f64) -> f64 {
    const C1: f64 = 4.16666666666666019037e-02;
    const C2: f64 = -1.38888888888741095749e-03;
    const C3: f64 = 2.48015872894767294178e-05;
    const C4: f64 = -2.75573143513906633035e-07;
    const C5: f64 = 2.08757232129817482790e-09;
    const C6: f64 = -1.13596475577881948265e-11;
    let ix = high(x) & 0x7FFFFFFF;
    if ix < 0x3E400000 && (x as i32) == 0 {
        return 1.0;
    }
    let z = x * x;
    let r = z * z.mul_add(z.mul_add(z.mul_add(z.mul_add(z.mul_add(C6, C5), C4), C3), C2), C1);
    if ix < 0x3FD33333 {
        1.0 - 0.5f64.mul_add(z, -z.mul_add(r, -(x * y)))
    } else {
        let qx = if ix > 0x3FE90000 { 0.28125 } else { from_words(ix - 0x00200000, 0) };
        let iz = 0.5f64.mul_add(z, -qx);
        let a = 1.0 - qx;
        a - (iz - z.mul_add(r, -(x * y)))
    }
}

fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    const S1: f64 = -1.66666666666666324348e-01;
    const S2: f64 = 8.33333333332248946124e-03;
    const S3: f64 = -1.98412698298579493134e-04;
    const S4: f64 = 2.75573137070700676789e-06;
    const S5: f64 = -2.50507602534068634195e-08;
    const S6: f64 = 1.58969099521155010221e-10;
    let ix = high(x) & 0x7FFFFFFF;
    if ix < 0x3E400000 && (x as i32) == 0 {
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = z.mul_add(z.mul_add(z.mul_add(S6, S5), S4), S3).mul_add(z, S2);
    if iy == 0 {
        v.mul_add(z.mul_add(r, S1), x)
    } else {
        x - (-v).mul_add(S1, z.mul_add(0.5f64.mul_add(y, -(v * r)), -y))
    }
}

/// JavaScript `Math.cos`.
pub fn cos(x: f64) -> f64 {
    let ix = high(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7FF00000 {
        return x - x;
    }
    let mut y = [0.0; 2];
    match rem_pio2(x, &mut y) & 3 {
        0 => kernel_cos(y[0], y[1]),
        1 => -kernel_sin(y[0], y[1], 1),
        2 => -kernel_cos(y[0], y[1]),
        _ => kernel_sin(y[0], y[1], 1),
    }
}

/// JavaScript `Math.sin`.
pub fn sin(x: f64) -> f64 {
    let ix = high(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB {
        return kernel_sin(x, 0.0, 0);
    }
    if ix >= 0x7FF00000 {
        return x - x;
    }
    let mut y = [0.0; 2];
    match rem_pio2(x, &mut y) & 3 {
        0 => kernel_sin(y[0], y[1], 1),
        1 => kernel_cos(y[0], y[1]),
        2 => -kernel_sin(y[0], y[1], 1),
        _ => -kernel_cos(y[0], y[1]),
    }
}

/// JavaScript `Math.atan`.
pub fn atan(mut x: f64) -> f64 {
    const ATANHI: [f64; 4] = [
        4.63647609000806093515e-01,
        7.85398163397448278999e-01,
        9.82793723247329054082e-01,
        1.57079632679489655800e+00,
    ];
    const ATANLO: [f64; 4] = [
        2.26987774529616870924e-17,
        3.06161699786838301793e-17,
        1.39033110312309984516e-17,
        6.12323399573676603587e-17,
    ];
    const AT: [f64; 11] = [
        3.33333333333329318027e-01,
        -1.99999999998764832476e-01,
        1.42857142725034663711e-01,
        -1.11111104054623557880e-01,
        9.09088713343650656196e-02,
        -7.69187620504482999495e-02,
        6.66107313738753120669e-02,
        -5.83357013379057348645e-02,
        4.97687799461593236017e-02,
        -3.65315727442169155270e-02,
        1.62858201153657823623e-02,
    ];
    let hx = high(x);
    let ix = hx & 0x7FFFFFFF;
    if ix >= 0x44100000 {
        if ix > 0x7FF00000 || (ix == 0x7FF00000 && low(x) != 0) {
            return x + x;
        }
        return if hx > 0 { ATANHI[3] + ATANLO[3] } else { -ATANHI[3] - ATANLO[3] };
    }
    let id: i32;
    if ix < 0x3FDC0000 {
        if ix < 0x3E400000 && 1.0e300 + x > 1.0 {
            return x;
        }
        id = -1;
    } else {
        x = x.abs();
        if ix < 0x3FF30000 {
            if ix < 0x3FE60000 {
                id = 0;
                x = 2.0f64.mul_add(x, -1.0) / (2.0 + x);
            } else {
                id = 1;
                x = (x - 1.0) / (x + 1.0);
            }
        } else if ix < 0x40038000 {
            id = 2;
            x = (x - 1.5) / 1.5f64.mul_add(x, 1.0);
        } else {
            id = 3;
            x = -1.0 / x;
        }
    }
    let z = x * x;
    let w = z * z;
    let s1 = z * w.mul_add(w.mul_add(w.mul_add(w.mul_add(w.mul_add(AT[10], AT[8]), AT[6]), AT[4]), AT[2]), AT[0]);
    let s2 = w * w.mul_add(w.mul_add(w.mul_add(w.mul_add(AT[9], AT[7]), AT[5]), AT[3]), AT[1]);
    if id < 0 {
        (-x).mul_add(s1 + s2, x)
    } else {
        let id = id as usize;
        let z = ATANHI[id] - (x.mul_add(s1 + s2, -ATANLO[id]) - x);
        if hx < 0 { -z } else { z }
    }
}

/// JavaScript `Math.atan2(y, x)`.
pub fn atan2(y: f64, x: f64) -> f64 {
    const TINY: f64 = 1.0e-300;
    const PI_O_4: f64 = 7.8539816339744827900E-01;
    const PI_O_2: f64 = 1.5707963267948965580E+00;
    const PI: f64 = 3.1415926535897931160E+00;
    const PI_LO: f64 = 1.2246467991473531772E-16;
    let (hx, lx) = (high(x), low(x));
    let ix = hx & 0x7FFFFFFF;
    let (hy, ly) = (high(y), low(y));
    let iy = hy & 0x7FFFFFFF;
    let nan_x = (ix as u32 | ((lx | (lx as i32).wrapping_neg() as u32) >> 31)) > 0x7FF00000;
    let nan_y = (iy as u32 | ((ly | (ly as i32).wrapping_neg() as u32) >> 31)) > 0x7FF00000;
    if nan_x || nan_y {
        return x + y;
    }
    if (hx.wrapping_sub(0x3FF00000) as u32 | lx) == 0 {
        return atan(y);
    }
    let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2);
    if (iy as u32 | ly) == 0 {
        match m {
            0 | 1 => return y,
            2 => return PI + TINY,
            _ => return -PI - TINY,
        }
    }
    if (ix as u32 | lx) == 0 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }
    if ix == 0x7FF00000 {
        if iy == 0x7FF00000 {
            return match m {
                0 => PI_O_4 + TINY,
                1 => -PI_O_4 - TINY,
                2 => 3.0f64.mul_add(PI_O_4, TINY),
                _ => (-3.0f64).mul_add(PI_O_4, -TINY),
            };
        } else {
            return match m {
                0 => 0.0,
                1 => -0.0,
                2 => PI + TINY,
                _ => -PI - TINY,
            };
        }
    }
    if iy == 0x7FF00000 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }
    let k = (iy - ix) >> 20;
    let z;
    if k > 60 {
        z = 0.5f64.mul_add(PI_LO, PI_O_2);
        m &= 1;
    } else if hx < 0 && k < -60 {
        z = 0.0;
    } else {
        z = atan((y / x).abs());
    }
    match m {
        0 => z,
        1 => -z,
        2 => PI - (z - PI_LO),
        _ => (z - PI_LO) - PI,
    }
}

/// JavaScript `Math.hypot(a, b)` as V8 computes it.
pub fn hypot(a: f64, b: f64) -> f64 {
    if a.is_infinite() || b.is_infinite() {
        return f64::INFINITY;
    }
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    let (a, b) = (a.abs(), b.abs());
    let max = if b > a { b } else { a };
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut comp) = (0.0f64, 0.0f64);
    for v in [a, b] {
        let n = v / max;
        let s = n * n - comp;
        let p = sum + s;
        comp = (p - sum) - s;
        sum = p;
    }
    sum.sqrt() * max
}
