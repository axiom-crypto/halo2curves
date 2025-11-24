macro_rules! field_arithmetic_asm {
    (
        $field:ident,
        $modulus:ident,
        $inv:ident
    ) => {
        use std::arch::asm;
        impl $field {
            /// Doubles this field element.
            #[inline]
            pub fn double(&self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;
                unsafe {
                    asm!(
                        // load a array to former registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // // add a array and b array with carry
                        "add r8, r8",
                        "adc r9, r9",
                        "adc r10, r10",
                        "adc r11, r11",
                        // capture carry
                        "mov rax, 0",
                        "adc rax, 0",

                        // copy result array to latter registers
                        "mov r12, r8",
                        "mov r13, r9",
                        "mov r14, r10",
                        "mov r15, r11",

                        // mod reduction
                        "sub r12, qword ptr [{m_ptr} + 0]",
                        "sbb r13, qword ptr [{m_ptr} + 8]",
                        "sbb r14, qword ptr [{m_ptr} + 16]",
                        "sbb r15, qword ptr [{m_ptr} + 24]",
                        "sbb rax, 0",

                        // if carry copy former registers to out areas
                        "cmovc r12, r8",
                        "cmovc r13, r9",
                        "cmovc r14, r10",
                        "cmovc r15, r11",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        out("rax") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") r0,
                        out("r13") r1,
                        out("r14") r2,
                        out("r15") r3,
                        options(pure, readonly)
                    );
                }
                $field([r0, r1, r2, r3])
            }

            /// Squares this element.
            #[inline]
            pub fn square(&self) -> $field {
                self.mul(self)
            }

            #[inline(always)]
            pub(crate) fn montgomery_reduce_256(&self) -> $field {
                /*
                let (mut r0, mut r1, mut r2, mut r3) =
                    (self.0[0], self.0[1], self.0[2], self.0[3]);

                // k0
                let k0 = r0.wrapping_mul($inv);
                let (_, mut carry) = crate::arithmetic::macx(r0, k0, $modulus.0[0]);
                (r1, carry) = crate::arithmetic::mac(r1, k0, $modulus.0[1], carry);
                (r2, carry) = crate::arithmetic::mac(r2, k0, $modulus.0[2], carry);
                (r3, carry) = crate::arithmetic::mac(r3, k0, $modulus.0[3], carry);

                // k1
                let k1 = r1.wrapping_mul($inv);
                let (_, mut carry) = crate::arithmetic::macx(r1, k1, $modulus.0[0]);
                (r2, carry) = crate::arithmetic::mac(r2, k1, $modulus.0[1], carry);
                (r3, carry) = crate::arithmetic::mac(r3, k1, $modulus.0[2], carry);
                (r0, carry) = crate::arithmetic::mac(r0, k1, $modulus.0[3], carry);

                // k2
                let k2 = r2.wrapping_mul($inv);
                let (_, mut carry) = crate::arithmetic::macx(r2, k2, $modulus.0[0]);
                (r3, carry) = crate::arithmetic::mac(r3, k2, $modulus.0[1], carry);
                (r0, carry) = crate::arithmetic::mac(r0, k2, $modulus.0[2], carry);
                (r1, carry) = crate::arithmetic::mac(r1, k2, $modulus.0[3], carry);

                // k3
                let k3 = r3.wrapping_mul($inv);
                let (_, mut carry) = crate::arithmetic::macx(r3, k3, $modulus.0[0]);
                (r0, carry) = crate::arithmetic::mac(r0, k3, $modulus.0[1], carry);
                (r1, carry) = crate::arithmetic::mac(r1, k3, $modulus.0[2], carry);
                (r2, carry) = crate::arithmetic::mac(r2, k3, $modulus.0[3], carry);

                // conditional subtraction
                let (d0, borrow) = crate::arithmetic::sbb(r0, $modulus.0[0], 0);
                let (d1, borrow) = crate::arithmetic::sbb(r1, $modulus.0[1], borrow);
                let (d2, borrow) = crate::arithmetic::sbb(r2, $modulus.0[2], borrow);
                let (d3, borrow) = crate::arithmetic::sbb(r3, $modulus.0[3], borrow);
                let (_, borrow_flag) = crate::arithmetic::sbb(carry, 0, borrow);

                let (d0, carry) = crate::arithmetic::adc(d0, $modulus.0[0] & borrow_flag, 0);
                let (d1, carry) = crate::arithmetic::adc(d1, $modulus.0[1] & borrow_flag, carry);
                let (d2, carry) = crate::arithmetic::adc(d2, $modulus.0[2] & borrow_flag, carry);
                let (d3, _) = crate::arithmetic::adc(d3, $modulus.0[3] & borrow_flag, carry);

                $field([d0, d1, d2, d3])
                */
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                unsafe {
                    asm!(
                        // Load input limbs
                        "mov r8,  qword ptr [{a_ptr} + 0]",
                        "mov r9,  qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",
                        "mov r15, {inv}",

                        // k0 stage
                        "mov rdx, r8",
                        "mulx r13, r12, r15", // r12 = k0 (low)

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]", // k0 * m0
                        "add rax, r8",
                        "adc rcx, 0",
                        "mov r13, rcx", // carry

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]", // k0 * m1
                        "add r9, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]", // k0 * m2
                        "add r10, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]", // k0 * m3
                        "add r11, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx", // carry from k0 stage

                        // k1 stage (r1)
                        "mov rdx, r9",
                        "mulx rcx, r12, r15", // r12 = k1

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "add rax, r9",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "add r10, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "add r11, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "add r8, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        // k2 stage (r2)
                        "mov rdx, r10",
                        "mulx rcx, r12, r15", // r12 = k2

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "add rax, r10",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "add r11, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "add r8, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "add r9, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        // k3 stage (r3)
                        "mov rdx, r11",
                        "mulx rcx, r12, r15", // r12 = k3

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "add rax, r11",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "add r8, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "add r9, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx",

                        "mov rdx, r12",
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "add r10, rax",
                        "adc rcx, 0",
                        "add rcx, r13",
                        "adc rcx, 0",
                        "mov r13, rcx", // final carry

                        // Conditional subtraction
                        "mov r12, r8",
                        "mov r13, r9",
                        "mov r14, r10",
                        "mov r15, r11",
                        "sub r12, qword ptr [{m_ptr} + 0]",
                        "sbb r13, qword ptr [{m_ptr} + 8]",
                        "sbb r14, qword ptr [{m_ptr} + 16]",
                        "sbb r15, qword ptr [{m_ptr} + 24]",
                        "mov rax, 0",
                        "sbb rax, 0", // borrow flag in CF
                        "cmovc r12, r8",
                        "cmovc r13, r9",
                        "cmovc r14, r10",
                        "cmovc r15, r11",

                        a_ptr = in(reg) self.0.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        inv = in(reg) $inv,

                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") r0,
                        out("r13") r1,
                        out("r14") r2,
                        out("r15") r3,
                        options(pure, readonly)
                    );
                }
                $field([r0, r1, r2, r3])
            }
            /// Multiplies `rhs` by `self`, returning the result.
            #[inline]
            pub fn mul(&self, rhs: &Self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                
                /*
                fn montgomery_reduce(r: &[u64; 8]) -> $field {
                    let k = r[0].wrapping_mul($inv);
                    let (_, carry) = crate::arithmetic::mac(r[0], k, $modulus.0[0], 0);
                    let (r1, carry) = crate::arithmetic::mac(r[1], k, $modulus.0[1], carry);
                    let (r2, carry) = crate::arithmetic::mac(r[2], k, $modulus.0[2], carry);
                    let (r3, carry) = crate::arithmetic::mac(r[3], k, $modulus.0[3], carry);
                    let (r4, mut carry2) = crate::arithmetic::adc(r[4], 0, carry);

                    let k = r1.wrapping_mul($inv);
                    let (_, carry) = crate::arithmetic::mac(r1, k, $modulus.0[0], 0);
                    let (r2, carry) = crate::arithmetic::mac(r2, k, $modulus.0[1], carry);
                    let (r3, carry) = crate::arithmetic::mac(r3, k, $modulus.0[2], carry);
                    let (r4, carry) = crate::arithmetic::mac(r4, k, $modulus.0[3], carry);
                    let (r5, carry2_tmp) = crate::arithmetic::adc(r[5], carry2, carry);
                    carry2 = carry2_tmp;

                    let k = r2.wrapping_mul($inv);
                    let (_, carry) = crate::arithmetic::mac(r2, k, $modulus.0[0], 0);
                    let (r3, carry) = crate::arithmetic::mac(r3, k, $modulus.0[1], carry);
                    let (r4, carry) = crate::arithmetic::mac(r4, k, $modulus.0[2], carry);
                    let (r5, carry) = crate::arithmetic::mac(r5, k, $modulus.0[3], carry);
                    let (r6, carry2_tmp) = crate::arithmetic::adc(r[6], carry2, carry);
                    carry2 = carry2_tmp;

                    let k = r3.wrapping_mul($inv);
                    let (_, carry) = crate::arithmetic::mac(r3, k, $modulus.0[0], 0);
                    let (r4, carry) = crate::arithmetic::mac(r4, k, $modulus.0[1], carry);
                    let (r5, carry) = crate::arithmetic::mac(r5, k, $modulus.0[2], carry);
                    let (r6, carry) = crate::arithmetic::mac(r6, k, $modulus.0[3], carry);
                    let (r7, carry2) = crate::arithmetic::adc(r[7], carry2, carry);

                    let (d0, borrow) = crate::arithmetic::sbb(r4, $modulus.0[0], 0);
                    let (d1, borrow) = crate::arithmetic::sbb(r5, $modulus.0[1], borrow);
                    let (d2, borrow) = crate::arithmetic::sbb(r6, $modulus.0[2], borrow);
                    let (d3, borrow) = crate::arithmetic::sbb(r7, $modulus.0[3], borrow);
                    let (_, borrow) = crate::arithmetic::sbb(carry2, 0, borrow);

                    let (d0, carry) =
                        crate::arithmetic::adc(d0, $modulus.0[0] & borrow, 0);
                    let (d1, carry) =
                        crate::arithmetic::adc(d1, $modulus.0[1] & borrow, carry);
                    let (d2, carry) =
                        crate::arithmetic::adc(d2, $modulus.0[2] & borrow, carry);
                    let (d3, _) = crate::arithmetic::adc(d3, $modulus.0[3] & borrow, carry);

                    $field([d0, d1, d2, d3])
                }

                pub fn mul(&self, rhs: &Self) -> $field {
                    let (r0, carry) = crate::arithmetic::mac(0, self.0[0], rhs.0[0], 0);
                    let (r1, carry) = crate::arithmetic::mac(0, self.0[0], rhs.0[1], carry);
                    let (r2, carry) = crate::arithmetic::mac(0, self.0[0], rhs.0[2], carry);
                    let (r3, r4) = crate::arithmetic::mac(0, self.0[0], rhs.0[3], carry);

                    let (r1, carry) = crate::arithmetic::mac(r1, self.0[1], rhs.0[0], 0);
                    let (r2, carry) = crate::arithmetic::mac(r2, self.0[1], rhs.0[1], carry);
                    let (r3, carry) = crate::arithmetic::mac(r3, self.0[1], rhs.0[2], carry);
                    let (r4, r5) = crate::arithmetic::mac(r4, self.0[1], rhs.0[3], carry);

                    let (r2, carry) = crate::arithmetic::mac(r2, self.0[2], rhs.0[0], 0);
                    let (r3, carry) = crate::arithmetic::mac(r3, self.0[2], rhs.0[1], carry);
                    let (r4, carry) = crate::arithmetic::mac(r4, self.0[2], rhs.0[2], carry);
                    let (r5, r6) = crate::arithmetic::mac(r5, self.0[2], rhs.0[3], carry);

                    let (r3, carry) = crate::arithmetic::mac(r3, self.0[3], rhs.0[0], 0);
                    let (r4, carry) = crate::arithmetic::mac(r4, self.0[3], rhs.0[1], carry);
                    let (r5, carry) = crate::arithmetic::mac(r5, self.0[3], rhs.0[2], carry);
                    let (r6, r7) = crate::arithmetic::mac(r6, self.0[3], rhs.0[3], carry);

                    Self::montgomery_reduce(&[r0, r1, r2, r3, r4, r5, r6, r7])
                }
                */
                unsafe {
                    asm!(
                        // schoolbook multiplication
                        // load value to registers
                        "mov r13, qword ptr [{b_ptr} + 0]",
                        "mov r14, qword ptr [{b_ptr} + 8]",
                        "mov r15, qword ptr [{b_ptr} + 16]",

                        // `a0`
                        "mov rdx, qword ptr [{a_ptr} + 0]",

                        // a0 * b0
                        "mulx r9, r8, r13",

                        // a0 * b1
                        "mulx r10, rax, r14",
                        "add r9, rax",

                        // a0 * b2
                        "mulx r11, rax, r15",
                        "adcx r10, rax",

                        // a0 * b3
                        "mulx r12, rax, qword ptr [{b_ptr} + 24]",
                        "adcx r11, rax",
                        "adc r12, 0",

                        // `a1`
                        "mov rdx, [{a_ptr} + 8]",

                        // a1 * b0
                        "mulx rcx, rax, r13",
                        "add r9, rax",
                        "adcx r10, rcx",
                        "adc r11, 0",

                        // a1 * b1
                        "mulx rcx, rax, r14",
                        "add r10, rax",
                        "adcx r11, rcx",
                        "adc r12, 0",
                        // Keep carry flag intact while zeroing
                        "mov r13, 0",

                        // a1 * b2
                        "mulx rcx, rax, r15",
                        "add r11, rax",
                        "adcx r12, rcx",
                        "adc r13, 0",
                        // Keep carry flag intact while zeroing
                        "mov r14, 0",

                        // a1 * b3
                        "mulx rcx, rax, qword ptr [{b_ptr} + 24]",
                        "add r12, rax",
                        "adcx r13, rcx",
                        "adc r14, 0",

                        // `a2`
                        "mov rdx, [{a_ptr} + 16]",

                        // a2 * b0
                        "mulx rcx, rax, qword ptr [{b_ptr} + 0]",
                        "add r10, rax",
                        "adcx r11, rcx",
                        "adc r12, 0",

                        // a2 * b1
                        "mulx rcx, rax, qword ptr [{b_ptr} + 8]",
                        "add r11, rax",
                        "adcx r12, rcx",
                        "adc r13, 0",

                        // a2 * b2
                        "mulx rcx, rax, r15",
                        "add r12, rax",
                        "adcx r13, rcx",
                        "adc r14, 0",
                        // Keep carry flag intact while zeroing
                        "mov r15, 0",

                        // a2 * b3
                        "mulx rcx, rax, qword ptr [{b_ptr} + 24]",
                        "add r13, rax",
                        "adcx r14, rcx",
                        "adc r15, 0",

                        // `a3`
                        "mov rdx, [{a_ptr} + 24]",

                        // a3 * b0
                        "mulx rcx, rax, qword ptr [{b_ptr} + 0]",
                        "add r11, rax",
                        "adcx r12, rcx",
                        "adc r13, 0",

                        // a3 * b1
                        "mulx rcx, rax, qword ptr [{b_ptr} + 8]",
                        "add r12, rax",
                        "adcx r13, rcx",
                        "adc r14, 0",

                        // a3 * b2
                        "mulx rcx, rax, qword ptr [{b_ptr} + 16]",
                        "add r13, rax",
                        "adcx r14, rcx",
                        "adc r15, 0",

                        // a3 * b3
                        "mulx rcx, rax, qword ptr [{b_ptr} + 24]",
                        "add r14, rax",
                        "adc r15, rcx",

                        // Montgomery reduction
                        // r8..r15 hold the 512-bit product

                        // `r8` -> 0
                        "mov rdx, {inv}",
                        "mulx rax, rdx, r8", // k0 = r8 * inv (low in rax)

                        // k0 * m0
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "add r8,  rax",
                        "adcx r9, rcx",
                        "adc  r10, 0",

                        // k0 * m1
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "add r9,  rax",
                        "adcx r10, rcx",
                        "adc  r11, 0",

                        // k0 * m2
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "add r10, rax",
                        "adcx r11, rcx",
                        "adc  r12, 0",

                        // k0 * m3
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "add r11, rax",
                        "adcx r12, rcx",
                        "adc  r13, 0",

                        // `r9` -> 0
                        "mov rdx, {inv}",
                        "mulx rax, rdx, r9", // k1

                        // k1 * m0
                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r9,  rcx",
                        "adcx r10, rax",
                        "adc  r11, 0",

                        // k1 * m1
                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r10, rcx",
                        "adcx r11, rax",
                        "adc  r12, 0",

                        // k1 * m2
                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r11, rcx",
                        "adcx r12, rax",
                        "adc  r13, 0",

                        // k1 * m3
                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r12, rcx",
                        "adcx r13, rax",
                        "adc  r14, 0",

                        // `r10` -> 0
                        "mov rdx, {inv}",
                        "mulx rax, rdx, r10", // k2

                        // k2 * m0
                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r10, rcx",
                        "adcx r11, rax",
                        "adc  r12, 0",

                        // k2 * m1
                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r11, rcx",
                        "adcx r12, rax",
                        "adc  r13, 0",

                        // k2 * m2
                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r12, rcx",
                        "adcx r13, rax",
                        "adc  r14, 0",

                        // k2 * m3
                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r13, rcx",
                        "adcx r14, rax",
                        "adc  r15, 0",

                        // `r11` -> 0
                        "mov rdx, {inv}",
                        "mulx rax, rdx, r11",  // k3

                        // k3 * m0
                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r11, rcx",
                        "adcx r12, rax",
                        "adc  r13, 0",

                        // k3 * m1
                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r12, rcx",
                        "adcx r13, rax",
                        "adc  r14, 0",

                        // k3 * m2
                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r13, rcx",
                        "adcx r14, rax",
                        "adc  r15, 0",

                        // k3 * m3
                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r14, rcx",
                        "adcx r15, rax",

                        // Now r8..r11 should be zeroed, and r12..r15 plus an
                        // extra carry bit (carry2) form the 5-limb value.
                        // Capture carry2 in rax:
                        "mov rax, 0",
                        "adc rax, 0",  // rax = carry2 (0 or 1), CF cleared

                        // Final conditional subtraction (REDC normalization)
                        // Compare (r12..r15, carry2) with modulus, subtract
                        // modulus if the 5-limb value >= modulus.

                        // Working copy of r12..r15 into r8..r11
                        "mov r8,  r12",
                        "mov r9,  r13",
                        "mov r10, r14",
                        "mov r11, r15",

                        // Subtract modulus (low 4 limbs)
                        "sub r8,  qword ptr [{m_ptr} + 0]",
                        "sbb r9,  qword ptr [{m_ptr} + 8]",
                        "sbb r10, qword ptr [{m_ptr} + 16]",
                        "sbb r11, qword ptr [{m_ptr} + 24]",

                        // High limb subtract: sbb(carry2, 0, borrow)
                        "sbb rax, 0", // CF = borrow_flag

                        // If we underflowed (borrow_flag == 1), restore original
                        "cmovc r8,  r12",
                        "cmovc r9,  r13",
                        "cmovc r10, r14",
                        "cmovc r11, r15",

                        // Move result back to r12..r15 for outputs
                        "mov r12, r8",
                        "mov r13, r9",
                        "mov r14, r10",
                        "mov r15, r11",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        inv   = in(reg) $inv,
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("r8")  _,
                        out("r9")  _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") r0,
                        out("r13") r1,
                        out("r14") r2,
                        out("r15") r3,
                        options(pure, readonly, nostack)
                    );
                }

                $field([r0, r1, r2, r3])
            }


            /// Subtracts `rhs` from `self`, returning the result.
            #[inline]
            pub fn sub(&self, rhs: &Self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;
                unsafe {
                    asm!(
                        // init modulus area
                        "mov r12, qword ptr [{m_ptr} + 0]",
                        "mov r13, qword ptr [{m_ptr} + 8]",
                        "mov r14, qword ptr [{m_ptr} + 16]",
                        "mov r15, qword ptr [{m_ptr} + 24]",

                        // load a array to former registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // sub a array and b array with borrow
                        "sub r8, qword ptr [{b_ptr} + 0]",
                        "sbb r9, qword ptr [{b_ptr} + 8]",
                        "sbb r10, qword ptr [{b_ptr} + 16]",
                        "sbb r11, qword ptr [{b_ptr} + 24]",

                        // Mask: rax contains 0xFFFF if < m or 0x0000 otherwise
                        "sbb rax, rax",

                        // Zero-out the modulus if a-b < m or leave as-is otherwise
                        "and r12, rax",
                        "and r13, rax",
                        "and r14, rax",
                        "and r15, rax",

                        // Add zero if a-b < m or a-b+m otherwise
                        "add  r12, r8",
                        "adc  r13, r9",
                        "adc  r14, r10",
                        "adc  r15, r11",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        out("rax") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") r0,
                        out("r13") r1,
                        out("r14") r2,
                        out("r15") r3,
                        options(pure, readonly)
                    );
                }
                $field([r0, r1, r2, r3])
            }

            /// Adds `rhs` to `self`, returning the result.
            #[inline]
            pub fn add(&self, rhs: &Self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;
                unsafe {
                    asm!(
                        // load a array to former registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // add a array and b array with carry
                        "add r8, qword ptr [{b_ptr} + 0]",
                        "adc r9, qword ptr [{b_ptr} + 8]",
                        "adc r10, qword ptr [{b_ptr} + 16]",
                        "adc r11, qword ptr [{b_ptr} + 24]",
                        // capture final carry into rax
                        "mov rax, 0",
                        "adc rax, 0",

                        // copy result array to latter registers
                        "mov r12, r8",
                        "mov r13, r9",
                        "mov r14, r10",
                        "mov r15, r11",

                        // mod reduction
                        "sub r12, qword ptr [{m_ptr} + 0]",
                        "sbb r13, qword ptr [{m_ptr} + 8]",
                        "sbb r14, qword ptr [{m_ptr} + 16]",
                        "sbb r15, qword ptr [{m_ptr} + 24]",
                        "sbb rax, 0",

                        // if carry copy former registers to out areas
                        "cmovc r12, r8",
                        "cmovc r13, r9",
                        "cmovc r14, r10",
                        "cmovc r15, r11",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        out("rax") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") r0,
                        out("r13") r1,
                        out("r14") r2,
                        out("r15") r3,
                        options(pure, readonly)
                    );
                }
                $field([r0, r1, r2, r3])
            }

            /// Negates `self`.
            #[inline]
            pub fn neg(&self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;
                unsafe {
                    asm!(
                        // load a array to former registers
                        "mov r8, qword ptr [{m_ptr} + 0]",
                        "mov r9, qword ptr [{m_ptr} + 8]",
                        "mov r10, qword ptr [{m_ptr} + 16]",
                        "mov r11, qword ptr [{m_ptr} + 24]",

                        "sub r8, qword ptr [{a_ptr} + 0]",
                        "sbb r9, qword ptr [{a_ptr} + 8]",
                        "sbb r10, qword ptr [{a_ptr} + 16]",
                        "sbb r11, qword ptr [{a_ptr} + 24]",

                        "mov r12, qword ptr [{a_ptr} + 0]",
                        "mov r13, qword ptr [{a_ptr} + 8]",
                        "mov r14, qword ptr [{a_ptr} + 16]",
                        "mov r15, qword ptr [{a_ptr} + 24]",

                        "or r12, r13",
                        "or r14, r15",
                        "or r12, r14",

                        "mov r13, 0xffffffffffffffff",
                        "cmp r12, 0x0000000000000000",
                        "cmove r13, r12",

                        "and r8, r13",
                        "and r9, r13",
                        "and r10, r13",
                        "and r11, r13",

                        a_ptr = in(reg) self.0.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
                        out("r12") _,
                        out("r13") _,
                        out("r14") _,
                        out("r15") _,
                        options(pure, readonly)
                    )
                }
                $field([r0, r1, r2, r3])
            }
        }

        impl From<$field> for [u64; 4] {
            fn from(elt: $field) -> [u64; 4] {
                // Turn into canonical form by computing
                // (a.R) / R = a
                elt.montgomery_reduce_256().0
            }
        }
    };
}

pub(crate) use field_arithmetic_asm;
