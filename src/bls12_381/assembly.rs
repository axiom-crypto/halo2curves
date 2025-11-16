#![cfg(all(feature = "asm", target_arch = "x86_64"))]

use blst::{blst_fp, blst_fp_add, blst_fp_cneg, blst_fp_mul, blst_fp_sqr, blst_fp_sub};

use super::fp::Fp;

#[inline(always)]
fn fp_as_blst(fp: &Fp) -> *const blst_fp {
    fp as *const Fp as *const blst_fp
}

#[inline(always)]
fn fp_as_mut_blst(fp: &mut Fp) -> *mut blst_fp {
    fp as *mut Fp as *mut blst_fp
}

impl Fp {
    #[inline]
    pub fn add(&self, rhs: &Fp) -> Fp {
        let mut out = Fp::zero();
        unsafe {
            blst_fp_add(fp_as_mut_blst(&mut out), fp_as_blst(self), fp_as_blst(rhs));
        }
        out
    }

    #[inline]
    pub fn sub(&self, rhs: &Fp) -> Fp {
        let mut out = Fp::zero();
        unsafe {
            blst_fp_sub(fp_as_mut_blst(&mut out), fp_as_blst(self), fp_as_blst(rhs));
        }
        out
    }

    #[inline]
    pub fn neg(&self) -> Fp {
        let mut out = Fp::zero();
        unsafe {
            // cneg computes (-a) when flag == true and leaves it unchanged otherwise.
            blst_fp_cneg(fp_as_mut_blst(&mut out), fp_as_blst(self), true);
        }
        out
    }

    #[inline]
    pub fn mul(&self, rhs: &Fp) -> Fp {
        let mut out = Fp::zero();
        unsafe {
            blst_fp_mul(fp_as_mut_blst(&mut out), fp_as_blst(self), fp_as_blst(rhs));
        }
        out
    }

    #[inline]
    pub fn square(&self) -> Fp {
        let mut out = Fp::zero();
        unsafe {
            blst_fp_sqr(fp_as_mut_blst(&mut out), fp_as_blst(self));
        }
        out
    }
}
