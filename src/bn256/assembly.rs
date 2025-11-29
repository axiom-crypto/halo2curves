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
                        // load a array to registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // double by adding to itself
                        "add r8, r8",
                        "adc r9, r9",
                        "adc r10, r10",
                        "adc r11, r11",

                        // try subtracting modulus into rax, rcx, rdx, rsi
                        "mov rax, r8",
                        "mov rcx, r9",
                        "mov rdx, r10",
                        "mov rsi, r11",

                        "sub rax, qword ptr [{m_ptr} + 0]",
                        "sbb rcx, qword ptr [{m_ptr} + 8]",
                        "sbb rdx, qword ptr [{m_ptr} + 16]",
                        "sbb rsi, qword ptr [{m_ptr} + 24]",

                        // if no carry, use the subtracted value; otherwise keep the original
                        "cmovnc r8, rax",
                        "cmovnc r9, rcx",
                        "cmovnc r10, rdx",
                        "cmovnc r11, rsi",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
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
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                unsafe {
                    asm!(
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",
                        "mov r15, {inv}",
                        "xor r12, r12",

                        // i0
                        "mov rdx, r8",
                        "mulx rcx, rdx, r15",

                        // j0
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "adox r8, rax",
                        "adcx r9, rcx",
                        // j1
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "adox r9, rax",
                        "adcx r10, rcx",
                        // j2
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "adox r10, rax",
                        "adcx r11, rcx",
                        // j3
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "adox r11, rax",
                        "adcx r8, rcx",
                        "adox r8, r12",

                        // i1
                        "mov rdx, r9",
                        "mulx rcx, rdx, r15",

                        // j0
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "adox r9, rax",
                        "adcx r10, rcx",

                        // j1
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "adox r10, rax",
                        "adcx r11, rcx",
                        // j2
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "adox r11, rax",
                        "adcx r8, rcx",
                        // j3
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "adox r8, rax",
                        "adcx r9, rcx",
                        "adox r9, r12",

                        // i2
                        "mov rdx, r10",
                        "mulx rcx, rdx, r15",

                        // j0
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "adox r10, rax",
                        "adcx r11, rcx",

                        // j1
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "adox r11, rax",
                        "adcx r8, rcx",

                        // j2
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "adox r8, rax",
                        "adcx r9, rcx",

                        // j3
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "adox r9, rax",
                        "adcx r10, rcx",
                        "adox r10, r12",

                        // i3
                        "mov rdx, r11",
                        "mulx rcx, rdx, r15",
                        // j0
                        "mulx rcx, rax, qword ptr [{m_ptr} + 0]",
                        "adox r11, rax",
                        "adcx r8, rcx",
                        // j1
                        "mulx rcx, rax, qword ptr [{m_ptr} + 8]",
                        "adox r8, rax",
                        "adcx r9, rcx",
                        // j2
                        "mulx rcx, rax, qword ptr [{m_ptr} + 16]",
                        "adox r9, rax",
                        "adcx r10, rcx",
                        // j3
                        "mulx rcx, rax, qword ptr [{m_ptr} + 24]",
                        "adox r10, rax",
                        "adcx r11, rcx",
                        "adox r11, r12",

                        // modular reduction is not required since:
                        // high(inv * p3) + 2 < p3

                        a_ptr = in(reg) self.0.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        inv = in(reg) $inv,

                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
                        out("r12") _,
                        out("r15") _,
                        options(pure, readonly)
                    )
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
                unsafe {
                    asm!(

                        // Coarsely Integrated Operand Scanning:
                        // - Analyzing and Comparing Montgomery Multiplication Algorithms
                        //   Cetin Kaya Koc and Tolga Acar and Burton S. Kaliski Jr.
                        //   http://pdfs.semanticscholar.org/5e39/41ff482ec3ee41dc53c3298f0be085c69483.pdf
                        //
                        // No-carry optimization
                        // - https://hackmd.io/@gnark/modular_multiplication
                        //
                        // Code generator
                        // - https://github.com/mratsim/constantine/blob/151f284/constantine/math/arithmetic/assembly/limbs_asm_mul_mont_x86_adx_bmi2.nim#L231-L269
                        //
                        // Assembly generated
                        // - https://github.com/ethereum/evmone/blob/d006d81/lib/evmmax/mulMont256_spare_bits_asm_adx.S

                        // Algorithm
                        // -----------------------------------------
                        // for i=0 to N-1
                        //   for j=0 to N-1
                        // 		(A,t[j])  := t[j] + a[j]*b[i] + A
                        //   m := t[0]*m0ninv mod W
                        // 	C,_ := t[0] + m*M[0]
                        // 	for j=1 to N-1
                        // 		(C,t[j-1]) := t[j] + m*M[j] + C
                        //   t[N-1] = C + A

                        // Outer loop i = 0
                        //   Multiplication
                        "mov  rdx, qword ptr [{b_ptr} + 0]",
                        "mulx r13, r11, qword ptr [{a_ptr} + 0]",
                        "mulx rax, r15, qword ptr [{a_ptr} + 8]",
                        "add  r13, r15",
                        "mulx r14, r15, qword ptr [{a_ptr} + 16]",
                        "adc  rax, r15",
                        //   Multiplication. last limb
                        "mulx r12, r15, qword ptr [{a_ptr} + 24]",
                        "adc  r14, r15",
                        "adc  r12, 0", // accumulate last carries in hi word

                        //   Reduction
                        //   m = t[0] * m0ninv mod 2^w
                        "mov  rdx, r11",
                        "imul rdx, {inv}",
                        "xor  r15, r15",
                        //   C,_ := t[0] + m*M[0]
                        "mulx r15, r10, qword ptr [{m_ptr} + 0]",
                        "adcx r10, r11",
                        "mov  r11, r15",
                        "mov  r10, 0",
                        //   for j=1 to N-1
                        //     (C, t[j-1]) := t[j] + m*M[j] + C
                        "adcx r11, r13",
                        "mulx r13, r15, qword ptr [{m_ptr} + 8]",
                        "adox r11, r15",
                        "adcx r13, rax",
                        "mulx rax, r15, qword ptr [{m_ptr} + 16]",
                        "adox r13, r15",
                        "adcx rax, r14",
                        "mulx r14, r15, qword ptr [{m_ptr} + 24]",
                        "adox rax, r15",
                        //   Reduction carry
                        "adcx r10, r12",
                        "adox r14, r10",

                        // Outer loop i = 1, j in [0, 4)
                        "mov  rdx, qword ptr [{b_ptr} + 8]",
                        "xor  r12, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 0]",
                        "adox r11, r15",
                        "adcx r13, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 8]",
                        "adox r13, r15",
                        "adcx rax, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 16]",
                        "adox rax, r15",
                        "adcx r14, r12",
                        //   Multiplication, last limb
                        "mulx r12, r15, qword ptr [{a_ptr} + 24]",
                        "adox r14, r15",
                        "mov  rdx, 0", // accumulate last carries in hi word
                        "adcx r12, rdx",
                        "adox r12, rdx",

                        //   Reduction
                        //   m = t[0] * m0ninv mod 2^w
                        "mov  rdx, r11",
                        "imul rdx, {inv}",
                        "xor  r15, r15",
                        //   C,_ := t[0] + m*M[0]
                        "mulx r15, r10, qword ptr [{m_ptr} + 0]",
                        "adcx r10, r11",
                        "mov  r11, r15",
                        "mov  r10, 0",
                        //   for j=1 to N-1
                        //     (C, t[j-1]) := t[j] + m*M[j] + C
                        "adcx r11, r13",
                        "mulx r13, r15, qword ptr [{m_ptr} + 8]",
                        "adox r11, r15",
                        "adcx r13, rax",
                        "mulx rax, r15, qword ptr [{m_ptr} + 16]",
                        "adox r13, r15",
                        "adcx rax, r14",
                        "mulx r14, r15, qword ptr [{m_ptr} + 24]",
                        "adox rax, r15",
                        //   Reduction carry
                        "adcx r10, r12",
                        "adox r14, r10",

                        // Outer loop i = 2, j in [0, 4)
                        "mov  rdx, qword ptr [{b_ptr} + 16]",
                        "xor  r12, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 0]",
                        "adox r11, r15",
                        "adcx r13, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 8]",
                        "adox r13, r15",
                        "adcx rax, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 16]",
                        "adox rax, r15",
                        "adcx r14, r12",
                        //   Multiplication, last limb
                        "mulx r12, r15, qword ptr [{a_ptr} + 24]",
                        "adox r14, r15",
                        "mov  rdx, 0", // accumulate last carries in hi word
                        "adcx r12, rdx",
                        "adox r12, rdx",

                        //   Reduction
                        //   m = t[0] * m0ninv mod 2^w
                        "mov  rdx, r11",
                        "imul rdx, {inv}",
                        "xor  r15, r15",
                        //   C,_ := t[0] + m*M[0]
                        "mulx r15, r10, qword ptr [{m_ptr} + 0]",
                        "adcx r10, r11",
                        "mov  r11, r15",
                        "mov  r10, 0",
                        //   for j=1 to N-1
                        //     (C, t[j-1]) := t[j] + m*M[j] + C
                        "adcx r11, r13",
                        "mulx r13, r15, qword ptr [{m_ptr} + 8]",
                        "adox r11, r15",
                        "adcx r13, rax",
                        "mulx rax, r15, qword ptr [{m_ptr} + 16]",
                        "adox r13, r15",
                        "adcx rax, r14",
                        "mulx r14, r15, qword ptr [{m_ptr} + 24]",
                        "adox rax, r15",
                        //   Reduction carry
                        "adcx r10, r12",
                        "adox r14, r10",

                        // Outer loop i = 3, j in [0, 4)
                        "mov  rdx, qword ptr [{b_ptr} + 24]",
                        "xor  r12, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 0]",
                        "adox r11, r15",
                        "adcx r13, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 8]",
                        "adox r13, r15",
                        "adcx rax, r12",
                        "mulx r12, r15, qword ptr [{a_ptr} + 16]",
                        "adox rax, r15",
                        "adcx r14, r12",
                        //   Multiplication, last limb
                        "mulx r12, r15, qword ptr [{a_ptr} + 24]",
                        "adox r14, r15",
                        "mov  rdx, 0", // accumulate last carries in hi word
                        "adcx r12, rdx",
                        "adox r12, rdx",

                        //   Reduction
                        //   m = t[0] * m0ninv mod 2^w
                        "mov  rdx, r11",
                        "imul rdx, {inv}",
                        "xor  r15, r15",
                        //   C,_ := t[0] + m*M[0]
                        "mulx r15, r10, qword ptr [{m_ptr} + 0]",
                        "adcx r10, r11",
                        "mov  r11, r15",
                        "mov  r10, 0",
                        //   for j=1 to N-1
                        //     (C, t[j-1]) := t[j] + m*M[j] + C
                        "adcx r11, r13",
                        "mulx r13, r15, qword ptr [{m_ptr} + 8]",
                        "adox r11, r15",
                        "adcx r13, rax",
                        "mulx rax, r15, qword ptr [{m_ptr} + 16]",
                        "adox r13, r15",
                        "adcx rax, r14",
                        "mulx r14, r15, qword ptr [{m_ptr} + 24]",
                        "adox rax, r15",
                        //   Reduction carry
                        "adcx r10, r12",
                        "adox r14, r10",

                        //   Final subtraction
                        "mov  r12, r11",
                        "sub  r12, qword ptr [{m_ptr} + 0]",
                        "mov  r10, r13",
                        "sbb  r10, qword ptr [{m_ptr} + 8]",
                        "mov  rdx, rax",
                        "sbb  rdx, qword ptr [{m_ptr} + 16]",
                        "mov  r15, r14",
                        "sbb  r15, qword ptr [{m_ptr} + 24]",

                        "cmovnc r11, r12",
                        "cmovnc r13, r10",
                        "cmovnc rax, rdx",
                        "cmovnc r14, r15",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        inv = in(reg) $inv,
                        out("rax") r2,
                        out("rdx") _,
                        out("r10") _,
                        out("r11") r0,
                        out("r12") _,
                        out("r13") r1,
                        out("r14") r3,
                        out("r15") _,
                        options(pure, readonly)
                    )
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
                        // load a array to registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // sub b array from a with borrow
                        "sub r8, qword ptr [{b_ptr} + 0]",
                        "sbb r9, qword ptr [{b_ptr} + 8]",
                        "sbb r10, qword ptr [{b_ptr} + 16]",
                        "sbb r11, qword ptr [{b_ptr} + 24]",

                        // Create mask: rsi contains 0xFFFFFFFFFFFFFFFF if borrow or 0x0 otherwise
                        "sbb rsi, rsi",

                        // Load and conditionally mask modulus
                        "mov rax, qword ptr [{m_ptr} + 0]",
                        "mov rcx, qword ptr [{m_ptr} + 8]",
                        "mov rdx, qword ptr [{m_ptr} + 16]",
                        "mov rdi, qword ptr [{m_ptr} + 24]",

                        "and rax, rsi",
                        "and rcx, rsi",
                        "and rdx, rsi",
                        "and rdi, rsi",

                        // Add masked modulus (0 if no borrow, modulus if borrow)
                        "add r8, rax",
                        "adc r9, rcx",
                        "adc r10, rdx",
                        "adc r11, rdi",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("rdi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
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
                        // load a array to registers
                        "mov r8, qword ptr [{a_ptr} + 0]",
                        "mov r9, qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // add a array and b array with carry
                        "add r8, qword ptr [{b_ptr} + 0]",
                        "adc r9, qword ptr [{b_ptr} + 8]",
                        "adc r10, qword ptr [{b_ptr} + 16]",
                        "adc r11, qword ptr [{b_ptr} + 24]",

                        // try subtracting modulus into rax, rcx, rdx, rsi
                        "mov rax, r8",
                        "mov rcx, r9",
                        "mov rdx, r10",
                        "mov rsi, r11",

                        "sub rax, qword ptr [{m_ptr} + 0]",
                        "sbb rcx, qword ptr [{m_ptr} + 8]",
                        "sbb rdx, qword ptr [{m_ptr} + 16]",
                        "sbb rsi, qword ptr [{m_ptr} + 24]",

                        // if no carry, use the subtracted value; otherwise keep the original
                        "cmovnc r8, rax",
                        "cmovnc r9, rcx",
                        "cmovnc r10, rdx",
                        "cmovnc r11, rsi",

                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
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
                        // load modulus and subtract self
                        "mov r8, qword ptr [{m_ptr} + 0]",
                        "mov r9, qword ptr [{m_ptr} + 8]",
                        "mov r10, qword ptr [{m_ptr} + 16]",
                        "mov r11, qword ptr [{m_ptr} + 24]",

                        "sub r8, qword ptr [{a_ptr} + 0]",
                        "sbb r9, qword ptr [{a_ptr} + 8]",
                        "sbb r10, qword ptr [{a_ptr} + 16]",
                        "sbb r11, qword ptr [{a_ptr} + 24]",

                        // Check if self is zero by ORing all limbs
                        "mov rax, qword ptr [{a_ptr} + 0]",
                        "mov rcx, qword ptr [{a_ptr} + 8]",
                        "mov rdx, qword ptr [{a_ptr} + 16]",
                        "mov rsi, qword ptr [{a_ptr} + 24]",

                        "or rax, rcx",
                        "or rdx, rsi",
                        "or rax, rdx",

                        // Create mask: all 1s if non-zero, all 0s if zero
                        "mov rcx, 0xffffffffffffffff",
                        "cmp rax, 0x0000000000000000",
                        "cmove rcx, rax",

                        // Apply mask to result
                        "and r8, rcx",
                        "and r9, rcx",
                        "and r10, rcx",
                        "and r11, rcx",

                        a_ptr = in(reg) self.0.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
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
