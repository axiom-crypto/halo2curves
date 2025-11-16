use halo2curves_axiom::bls12_381::Fp;
use halo2curves_axiom::ff::Field;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::Zero;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

const MODULUS_HEX: &str =
    "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";

lazy_static! {
    static ref MODULUS: BigUint = BigUint::from_str_radix(MODULUS_HEX, 16).unwrap();
}

#[test]
fn fp_arithmetic_matches_reference_model() {
    let mut rng = ChaCha20Rng::seed_from_u64(0xdecafbadcafef00d);

    for _ in 0..512 {
        let a = Fp::random(&mut rng);
        let b = Fp::random(&mut rng);

        assert_eq!(a + b, reference_add(&a, &b), "add mismatch");
        assert_eq!(a - b, reference_sub(&a, &b), "sub mismatch");
        assert_eq!(a.neg(), reference_neg(&a), "neg mismatch");
        assert_eq!(a.double(), reference_double(&a), "double mismatch");
        assert_eq!(a.mul(&b), reference_mul(&a, &b), "mul mismatch");
        assert_eq!(a.square(), reference_square(&a), "square mismatch");

        if bool::from(!a.is_zero()) {
            assert_eq!(a.invert().unwrap(), reference_inv(&a), "inverse mismatch");
        }
    }
}

fn reference_add(a: &Fp, b: &Fp) -> Fp {
    fp_from_biguint(fp_to_biguint(a) + fp_to_biguint(b))
}

fn reference_sub(a: &Fp, b: &Fp) -> Fp {
    let modulus = &*MODULUS;
    fp_from_biguint(fp_to_biguint(a) + modulus - fp_to_biguint(b))
}

fn reference_neg(a: &Fp) -> Fp {
    if bool::from(a.is_zero()) {
        return Fp::zero();
    }
    let modulus = &*MODULUS;
    fp_from_biguint(modulus - fp_to_biguint(a))
}

fn reference_double(a: &Fp) -> Fp {
    fp_from_biguint(fp_to_biguint(a) << 1)
}

fn reference_mul(a: &Fp, b: &Fp) -> Fp {
    fp_from_biguint(fp_to_biguint(a) * fp_to_biguint(b))
}

fn reference_square(a: &Fp) -> Fp {
    fp_from_biguint(fp_to_biguint(a).pow(2u32))
}

fn reference_inv(a: &Fp) -> Fp {
    let modulus = &*MODULUS;
    let a_big = fp_to_biguint(a);
    let exponent = modulus - BigUint::from(2u32);
    fp_from_biguint(a_big.modpow(&exponent, modulus))
}

fn fp_to_biguint(value: &Fp) -> BigUint {
    BigUint::from_bytes_le(&value.to_bytes())
}

fn fp_from_biguint<N>(n: N) -> Fp
where
    N: Into<BigUint>,
{
    let modulus = &*MODULUS;
    let mut reduced = n.into() % modulus;
    if reduced.is_zero() {
        return Fp::zero();
    }

    let mut bytes = reduced.to_bytes_le();
    bytes.resize(48, 0);
    let array: [u8; 48] = bytes.try_into().expect("48-byte array");
    Fp::from_bytes(&array).unwrap()
}
