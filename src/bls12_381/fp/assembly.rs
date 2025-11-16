use blst::{blst_fp, blst_fp_add, blst_fp_mul, blst_fp_sqr, blst_fp_sub};

use super::Fp;

#[inline(always)]
fn to_blst_fp(value: &Fp) -> blst_fp {
    blst_fp { l: value.0 }
}

#[inline(always)]
fn from_blst_fp(value: &blst_fp) -> Fp {
    Fp(value.l)
}

#[inline(always)]
pub(super) fn add(lhs: &Fp, rhs: &Fp) -> Fp {
    let a = to_blst_fp(lhs);
    let b = to_blst_fp(rhs);
    let mut out = blst_fp { l: [0; 6] };
    unsafe {
        blst_fp_add(&mut out, &a, &b);
    }
    from_blst_fp(&out)
}

#[inline(always)]
pub(super) fn sub(lhs: &Fp, rhs: &Fp) -> Fp {
    let a = to_blst_fp(lhs);
    let b = to_blst_fp(rhs);
    let mut out = blst_fp { l: [0; 6] };
    unsafe {
        blst_fp_sub(&mut out, &a, &b);
    }
    from_blst_fp(&out)
}

#[inline(always)]
pub(super) fn neg(value: &Fp) -> Fp {
    let zero = blst_fp { l: [0; 6] };
    let a = to_blst_fp(value);
    let mut out = blst_fp { l: [0; 6] };
    unsafe {
        blst_fp_sub(&mut out, &zero, &a);
    }
    from_blst_fp(&out)
}

#[inline(always)]
pub(super) fn mul(lhs: &Fp, rhs: &Fp) -> Fp {
    let a = to_blst_fp(lhs);
    let b = to_blst_fp(rhs);
    let mut out = blst_fp { l: [0; 6] };
    unsafe {
        blst_fp_mul(&mut out, &a, &b);
    }
    from_blst_fp(&out)
}

#[inline(always)]
pub(super) fn square(value: &Fp) -> Fp {
    let a = to_blst_fp(value);
    let mut out = blst_fp { l: [0; 6] };
    unsafe {
        blst_fp_sqr(&mut out, &a);
    }
    from_blst_fp(&out)
}
