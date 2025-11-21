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

                        // final conditional subtraction
                        "mov r12, r8",
                        "mov r13, r9",
                        "mov r14, r10",
                        "mov r15, r11",
                        "sub r12, qword ptr [{m_ptr} + 0]",
                        "sbb r13, qword ptr [{m_ptr} + 8]",
                        "sbb r14, qword ptr [{m_ptr} + 16]",
                        "sbb r15, qword ptr [{m_ptr} + 24]",
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
                    )
                }
                $field([r0, r1, r2, r3])
            }
            #[inline(always)]
            pub(crate) fn montgomery_reduce(r: &[u64; 8]) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                unsafe {
                    asm!(
                        // load 512-bit input
                        "mov r8,  qword ptr [{r_ptr} + 0]",
                        "mov r9,  qword ptr [{r_ptr} + 8]",
                        "mov r10, qword ptr [{r_ptr} + 16]",
                        "mov r11, qword ptr [{r_ptr} + 24]",
                        "mov r12, qword ptr [{r_ptr} + 32]",
                        "mov r13, qword ptr [{r_ptr} + 40]",
                        "mov r14, qword ptr [{r_ptr} + 48]",
                        "mov r15, qword ptr [{r_ptr} + 56]",
                        "xor rdi, rdi", // carry2 = 0

                        // i0
                        "mov rdx, r8",
                        "mulx rax, rcx, qword ptr [{inv}]",
                        "mov rdx, rcx",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r8, rcx",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r9, rcx",
                        "adc r9, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r10, rcx",
                        "adc r10, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r11, rcx",
                        "adc r11, rsi",
                        "adc rax, 0",

                        "add r12, rdi",
                        "adc r12, rax",
                        "mov rdi, 0",
                        "adc rdi, 0",

                        // i1
                        "mov rdx, r9",
                        "mulx rax, rcx, qword ptr [{inv}]",
                        "mov rdx, rcx",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r9, rcx",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r10, rcx",
                        "adc r10, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r11, rcx",
                        "adc r11, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r12, rcx",
                        "adc r12, rsi",
                        "adc rax, 0",

                        "add r13, rdi",
                        "adc r13, rax",
                        "mov rdi, 0",
                        "adc rdi, 0",

                        // i2
                        "mov rdx, r10",
                        "mulx rax, rcx, qword ptr [{inv}]",
                        "mov rdx, rcx",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r10, rcx",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r11, rcx",
                        "adc r11, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r12, rcx",
                        "adc r12, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r13, rcx",
                        "adc r13, rsi",
                        "adc rax, 0",

                        "add r14, rdi",
                        "adc r14, rax",
                        "mov rdi, 0",
                        "adc rdi, 0",

                        // i3
                        "mov rdx, r11",
                        "mulx rax, rcx, qword ptr [{inv}]",
                        "mov rdx, rcx",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 0]",
                        "add r11, rcx",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 8]",
                        "add r12, rcx",
                        "adc r12, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 16]",
                        "add r13, rcx",
                        "adc r13, rsi",
                        "adc rax, 0",
                        "mov rsi, rax",

                        "mulx rax, rcx, qword ptr [{m_ptr} + 24]",
                        "add r14, rcx",
                        "adc r14, rsi",
                        "adc rax, 0",

                        "add r15, rdi",
                        "adc r15, rax",
                        "mov rdi, 0",
                        "adc rdi, 0",

                        // conditional subtraction: if (r15..r12, carry2) >= modulus keep subtraction
                        "mov r8, r12",
                        "mov r9, r13",
                        "mov r10, r14",
                        "mov r11, r15",

                        "sub r8, qword ptr [{m_ptr} + 0]",
                        "sbb r9, qword ptr [{m_ptr} + 8]",
                        "sbb r10, qword ptr [{m_ptr} + 16]",
                        "sbb r11, qword ptr [{m_ptr} + 24]",
                        "sbb rdi, 0",

                        "cmovc r8, r12",
                        "cmovc r9, r13",
                        "cmovc r10, r14",
                        "cmovc r11, r15",

                        r_ptr = in(reg) r.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        inv = sym $inv,
 
                        out("rax") _,
                        out("rdi") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
                        out("r12") _,
                        out("r13") _,
                        out("r14") _,
                        out("r15") _,
                        options(pure, readonly)
                    );
                }

                $field([r0, r1, r2, r3])
            }

            /// Multiplies `rhs` by `self`, returning the result.
            #[inline]
            pub fn mul(&self, rhs: &Self) -> $field {
                let mut tmp = [0u64; 8];
                unsafe {
                    asm!(
                        // load operands
                        "mov r8,  qword ptr [{a_ptr} + 0]",
                        "mov r9,  qword ptr [{a_ptr} + 8]",
                        "mov r10, qword ptr [{a_ptr} + 16]",
                        "mov r11, qword ptr [{a_ptr} + 24]",

                        // row 0 (a0 * b)
                        "mov rax, r8",
                        "mul qword ptr [{b_ptr} + 0]",
                        "add qword ptr [{tmp_ptr} + 0], rax",
                        "adc qword ptr [{tmp_ptr} + 8], rdx",

                        "mov rax, r8",
                        "mul qword ptr [{b_ptr} + 8]",
                        "adc qword ptr [{tmp_ptr} + 8], rax",
                        "adc qword ptr [{tmp_ptr} + 16], rdx",

                        "mov rax, r8",
                        "mul qword ptr [{b_ptr} + 16]",
                        "adc qword ptr [{tmp_ptr} + 16], rax",
                        "adc qword ptr [{tmp_ptr} + 24], rdx",

                        "mov rax, r8",
                        "mul qword ptr [{b_ptr} + 24]",
                        "adc qword ptr [{tmp_ptr} + 24], rax",
                        "adc qword ptr [{tmp_ptr} + 32], rdx",
                        "adc qword ptr [{tmp_ptr} + 40], 0",

                        // row 1 (a1 * b)
                        "clc",
                        "mov rax, r9",
                        "mul qword ptr [{b_ptr} + 0]",
                        "add qword ptr [{tmp_ptr} + 8], rax",
                        "adc qword ptr [{tmp_ptr} + 16], rdx",

                        "mov rax, r9",
                        "mul qword ptr [{b_ptr} + 8]",
                        "adc qword ptr [{tmp_ptr} + 16], rax",
                        "adc qword ptr [{tmp_ptr} + 24], rdx",

                        "mov rax, r9",
                        "mul qword ptr [{b_ptr} + 16]",
                        "adc qword ptr [{tmp_ptr} + 24], rax",
                        "adc qword ptr [{tmp_ptr} + 32], rdx",

                        "mov rax, r9",
                        "mul qword ptr [{b_ptr} + 24]",
                        "adc qword ptr [{tmp_ptr} + 32], rax",
                        "adc qword ptr [{tmp_ptr} + 40], rdx",
                        "adc qword ptr [{tmp_ptr} + 48], 0",

                        // row 2 (a2 * b)
                        "clc",
                        "mov rax, r10",
                        "mul qword ptr [{b_ptr} + 0]",
                        "add qword ptr [{tmp_ptr} + 16], rax",
                        "adc qword ptr [{tmp_ptr} + 24], rdx",

                        "mov rax, r10",
                        "mul qword ptr [{b_ptr} + 8]",
                        "adc qword ptr [{tmp_ptr} + 24], rax",
                        "adc qword ptr [{tmp_ptr} + 32], rdx",

                        "mov rax, r10",
                        "mul qword ptr [{b_ptr} + 16]",
                        "adc qword ptr [{tmp_ptr} + 32], rax",
                        "adc qword ptr [{tmp_ptr} + 40], rdx",

                        "mov rax, r10",
                        "mul qword ptr [{b_ptr} + 24]",
                        "adc qword ptr [{tmp_ptr} + 40], rax",
                        "adc qword ptr [{tmp_ptr} + 48], rdx",
                        "adc qword ptr [{tmp_ptr} + 56], 0",

                        // row 3 (a3 * b)
                        "clc",
                        "mov rax, r11",
                        "mul qword ptr [{b_ptr} + 0]",
                        "add qword ptr [{tmp_ptr} + 24], rax",
                        "adc qword ptr [{tmp_ptr} + 32], rdx",

                        "mov rax, r11",
                        "mul qword ptr [{b_ptr} + 8]",
                        "adc qword ptr [{tmp_ptr} + 32], rax",
                        "adc qword ptr [{tmp_ptr} + 40], rdx",

                        "mov rax, r11",
                        "mul qword ptr [{b_ptr} + 16]",
                        "adc qword ptr [{tmp_ptr} + 40], rax",
                        "adc qword ptr [{tmp_ptr} + 48], rdx",

                        "mov rax, r11",
                        "mul qword ptr [{b_ptr} + 24]",
                        "adc qword ptr [{tmp_ptr} + 48], rax",
                        "adc qword ptr [{tmp_ptr} + 56], rdx",
                        "adc rax, 0",

                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        tmp_ptr = in(reg) tmp.as_mut_ptr(),
                        out("rax") _,
                        out("rdx") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        options()
                    );
                }

                Self::montgomery_reduce(&tmp)
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
