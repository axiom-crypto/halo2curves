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

            /// Montgomery reduction for 256-bit input
            #[inline(always)]
            pub(crate) fn montgomery_reduce_256(&self) -> $field {
                let r = [self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0];
                Self::montgomery_reduce(&r)
            }


            /// Rust reference implementation:
            // ```rust
            /// fn montgomery_reduce(r: [u64; 8], modulus: [u64; 4], inv: u64) -> [u64; 4] {
            ///     let (mut r0, mut r1, mut r2, mut r3, mut r4, mut r5, mut r6, mut r7) =
            ///         (r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
            ///     let mut carry2 = 0u64;
            ///
            ///     // Round 0
            ///     let k = r0.wrapping_mul(inv);
            ///     let (_, mut carry) = mac(r0, k, modulus[0], 0);
            ///     (r1, carry) = mac(r1, k, modulus[1], carry);
            ///     (r2, carry) = mac(r2, k, modulus[2], carry);
            ///     (r3, carry) = mac(r3, k, modulus[3], carry);
            ///     (r4, carry2) = adc(r4, 0, carry);
            ///
            ///     // Round 1
            ///     let k = r1.wrapping_mul(inv);
            ///     let (_, mut carry) = mac(r1, k, modulus[0], 0);
            ///     (r2, carry) = mac(r2, k, modulus[1], carry);
            ///     (r3, carry) = mac(r3, k, modulus[2], carry);
            ///     (r4, carry) = mac(r4, k, modulus[3], carry);
            ///     (r5, carry2) = adc(r5, carry2, carry);
            ///
            ///     // Round 2
            ///     let k = r2.wrapping_mul(inv);
            ///     let (_, mut carry) = mac(r2, k, modulus[0], 0);
            ///     (r3, carry) = mac(r3, k, modulus[1], carry);
            ///     (r4, carry) = mac(r4, k, modulus[2], carry);
            ///     (r5, carry) = mac(r5, k, modulus[3], carry);
            ///     (r6, carry2) = adc(r6, carry2, carry);
            ///
            ///     // Round 3
            ///     let k = r3.wrapping_mul(inv);
            ///     let (_, mut carry) = mac(r3, k, modulus[0], 0);
            ///     (r4, carry) = mac(r4, k, modulus[1], carry);
            ///     (r5, carry) = mac(r5, k, modulus[2], carry);
            ///     (r6, carry) = mac(r6, k, modulus[3], carry);
            ///     (r7, carry2) = adc(r7, carry2, carry);
            ///
            ///     // Final reduction: if result >= modulus, subtract modulus
            ///     let (d0, borrow) = sbb(r4, modulus[0], 0);
            ///     let (d1, borrow) = sbb(r5, modulus[1], borrow);
            ///     let (d2, borrow) = sbb(r6, modulus[2], borrow);
            ///     let (d3, borrow) = sbb(r7, modulus[3], borrow);
            ///     let (_, borrow) = sbb(carry2, 0, borrow);
            ///
            ///     // If borrow, add modulus back (conditional select)
            ///     let (d0, carry) = adc(d0, modulus[0] & borrow, 0);
            ///     let (d1, carry) = adc(d1, modulus[1] & borrow, carry);
            ///     let (d2, carry) = adc(d2, modulus[2] & borrow, carry);
            ///     let (d3, _) = adc(d3, modulus[3] & borrow, carry);
            ///
            ///     [d0, d1, d2, d3]
            /// }
            // ```
            #[inline]
            pub fn montgomery_reduce(r: &[u64; 8]) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                unsafe {
                    asm!(
                        // Save callee-saved registers
                        "push rbx",
                        "push rbp",
                        "push r12",
                        "push r13",
                        "push r14",
                        "push r15",

                        // First load all inputs into scratch registers
                        "mov rax, {a_ptr}",
                        "mov rcx, {m_ptr}",
                        "mov rsi, {inv}",

                        // Allocate 8 bytes on stack for carry2
                        "sub rsp, 8",
                        "mov qword ptr [rsp], 0",   // carry2 = 0

                        // Copy to target registers
                        "mov rbx, rcx",          // rbx = modulus pointer
                        "mov rbp, rsi",          // rbp = inv

                        // Load all 8 limbs into r8-r15
                        "mov r8, qword ptr [rax + 0]",
                        "mov r9, qword ptr [rax + 8]",
                        "mov r10, qword ptr [rax + 16]",
                        "mov r11, qword ptr [rax + 24]",
                        "mov r12, qword ptr [rax + 32]",
                        "mov r13, qword ptr [rax + 40]",
                        "mov r14, qword ptr [rax + 48]",
                        "mov r15, qword ptr [rax + 56]",

                        // Round 0: k = r8 * inv
                        "mov rdx, r8",
                        "imul rdx, rbp",         // k = r8 * inv

                        // mac(r8, k, m[0], 0) - first multiply, no input carry
                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r8, rax",           // r8 + k*m[0].lo, r8 now effectively 0
                        "adc rcx, 0",            // carry = k*m[0].hi + CF

                        // mac(r9, k, m[1], carry)
                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",          // tmp = k*m[1].lo + carry
                        "adc rax, 0",            // carry = k*m[1].hi + CF
                        "add r9, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        // mac(r10, k, m[2], carry)
                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r10, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        // mac(r11, k, m[3], carry)
                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",
                        // carry = rax

                        // adc(r12, 0, carry) -> (r12, carry2)
                        "add r12, rax",
                        "mov rax, 0",
                        "adc rax, 0",
                        "mov qword ptr [rsp], rax",  // carry2 = overflow

                        // Round 1: k = r9 * inv
                        "mov rdx, r9",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r9, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r10, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",
                        // carry = rax

                        // adc(r13, carry2, carry) -> (r13, carry2)
                        "add rax, qword ptr [rsp]",  // rax = carry + carry2
                        "mov rcx, 0",
                        "adc rcx, 0",                // rcx = overflow from carry + carry2
                        "add r13, rax",
                        "adc rcx, 0",                // rcx = total overflow
                        "mov qword ptr [rsp], rcx",  // carry2 = overflow

                        // Round 2: k = r10 * inv
                        "mov rdx, r10",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r10, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r13, rsi",
                        "adc rax, 0",
                        // carry = rax

                        // adc(r14, carry2, carry)
                        "add rax, qword ptr [rsp]",
                        "mov rcx, 0",
                        "adc rcx, 0",
                        "add r14, rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp], rcx",

                        // Round 3: k = r11 * inv
                        "mov rdx, r11",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r11, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r13, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r14, rsi",
                        "adc rax, 0",
                        // carry = rax

                        // adc(r15, carry2, carry) -> (r15, carry2)
                        "add rax, qword ptr [rsp]",
                        "mov rcx, 0",
                        "adc rcx, 0",
                        "add r15, rax",
                        "adc rcx, 0",
                        // carry2 is now in rcx

                        // Final reduction
                        // Result is in r12-r15, carry2 in rcx
                        // Try subtracting modulus: d = result - modulus
                        "mov r8, r12",
                        "mov r9, r13",
                        "mov r10, r14",
                        "mov r11, r15",

                        "sub r8, qword ptr [rbx + 0]",
                        "sbb r9, qword ptr [rbx + 8]",
                        "sbb r10, qword ptr [rbx + 16]",
                        "sbb r11, qword ptr [rbx + 24]",
                        "sbb rcx, 0",   // borrow from carry2

                        // If borrow (CF=1 after sbb), result < modulus, keep original
                        // Otherwise use subtracted result
                        "cmovc r8, r12",
                        "cmovc r9, r13",
                        "cmovc r10, r14",
                        "cmovc r11, r15",

                        // Deallocate stack space for carry2
                        "add rsp, 8",

                        // Restore callee-saved registers
                        "pop r15",
                        "pop r14",
                        "pop r13",
                        "pop r12",
                        "pop rbp",
                        "pop rbx",

                        a_ptr = in(reg) r.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        inv = in(reg) $inv,
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
                    );
                }

                $field([r0, r1, r2, r3])
            }

            /// Multiplies two field elements using x86-64 assembly
            /// ```rust
            /// fn mul(a: [u64; 4], b: [u64; 4], modulus: [u64; 4], inv: u64) -> [u64; 4] {
            ///     // Step 1: Schoolbook multiplication to get 512-bit product
            ///     fn mul_512(a: [u64; 4], b: [u64; 4]) -> [u64; 8] {
            ///         let (r0, carry) = macx(0, a[0], b[0]);
            ///         let (r1, carry) = macx(carry, a[0], b[1]);
            ///         let (r2, carry) = macx(carry, a[0], b[2]);
            ///         let (r3, carry_out) = macx(carry, a[0], b[3]);
            ///
            ///         let (r1, carry) = macx(r1, a[1], b[0]);
            ///         let (r2, carry) = mac(r2, a[1], b[1], carry);
            ///         let (r3, carry) = mac(r3, a[1], b[2], carry);
            ///         let (r4, carry_out) = mac(carry_out, a[1], b[3], carry);
            ///
            ///         let (r2, carry) = macx(r2, a[2], b[0]);
            ///         let (r3, carry) = mac(r3, a[2], b[1], carry);
            ///         let (r4, carry) = mac(r4, a[2], b[2], carry);
            ///         let (r5, carry_out) = mac(carry_out, a[2], b[3], carry);
            ///
            ///         let (r3, carry) = macx(r3, a[3], b[0]);
            ///         let (r4, carry) = mac(r4, a[3], b[1], carry);
            ///         let (r5, carry) = mac(r5, a[3], b[2], carry);
            ///         let (r6, r7) = mac(carry_out, a[3], b[3], carry);
            ///
            ///         [r0, r1, r2, r3, r4, r5, r6, r7]
            ///     }
            ///
            ///     // Step 2: Montgomery reduction (see montgomery_reduce)
            ///     let product = mul_512(a, b);
            ///     montgomery_reduce(product, modulus, inv)
            /// }
            /// ```
            #[inline]
            pub fn mul(&self, rhs: &Self) -> $field {
                let mut r0: u64;
                let mut r1: u64;
                let mut r2: u64;
                let mut r3: u64;

                unsafe {
                    asm!(
                        // Save callee-saved registers
                        "push rbx",
                        "push rbp",
                        "push r12",
                        "push r13",
                        "push r14",
                        "push r15",

                        // Load inputs into scratch registers first
                        "mov rax, {a_ptr}",
                        "mov rcx, {b_ptr}",
                        "mov rsi, {m_ptr}",
                        "mov rdi, {inv}",

                        // Allocate stack: 64 bytes for product + 8 bytes for carry2
                        "sub rsp, 72",

                        // Copy pointers to callee-saved registers
                        "mov r12, rax",          // r12 = a pointer
                        "mov r13, rcx",          // r13 = b pointer
                        "mov r14, rsi",          // r14 = modulus pointer
                        "mov r15, rdi",          // r15 = inv

                        // Step 1: Schoolbook multiplication (512-bit product)
                        // Result goes to [rsp+8] through [rsp+64]

                        // Initialize product to zero
                        "mov qword ptr [rsp + 8], 0",
                        "mov qword ptr [rsp + 16], 0",
                        "mov qword ptr [rsp + 24], 0",
                        "mov qword ptr [rsp + 32], 0",
                        "mov qword ptr [rsp + 40], 0",
                        "mov qword ptr [rsp + 48], 0",
                        "mov qword ptr [rsp + 56], 0",
                        "mov qword ptr [rsp + 64], 0",

                        // Row 0: product += a[0] * b
                        "mov rdx, qword ptr [r12 + 0]",    // a[0]

                        "mulx rcx, rax, qword ptr [r13 + 0]",
                        "mov qword ptr [rsp + 8], rax",
                        "mov rax, rcx",

                        "mulx rcx, rbx, qword ptr [r13 + 8]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 16], rax",
                        "mov rax, rcx",

                        "mulx rcx, rbx, qword ptr [r13 + 16]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 24], rax",
                        "mov rax, rcx",

                        "mulx rcx, rbx, qword ptr [r13 + 24]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 32], rax",
                        "mov qword ptr [rsp + 40], rcx",

                        // Row 1: product += a[1] * b << 64
                        "mov rdx, qword ptr [r12 + 8]",    // a[1]

                        "mulx rcx, rax, qword ptr [r13 + 0]",
                        "add qword ptr [rsp + 16], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 8]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 24], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 16]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 32], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 24]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 40], rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 48], rcx",

                        // Row 2: product += a[2] * b << 128
                        "mov rdx, qword ptr [r12 + 16]",   // a[2]

                        "mulx rcx, rax, qword ptr [r13 + 0]",
                        "add qword ptr [rsp + 24], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 8]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 32], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 16]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 40], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 24]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 48], rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 56], rcx",

                        // Row 3: product += a[3] * b << 192
                        "mov rdx, qword ptr [r12 + 24]",   // a[3]

                        "mulx rcx, rax, qword ptr [r13 + 0]",
                        "add qword ptr [rsp + 32], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 8]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 40], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 16]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 48], rax",
                        "adc rcx, 0",
                        "mov rbx, rcx",

                        "mulx rcx, rax, qword ptr [r13 + 24]",
                        "add rax, rbx",
                        "adc rcx, 0",
                        "add qword ptr [rsp + 56], rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 64], rcx",

                        // Step 2: Montgomery reduction
                        // 512-bit product is at [rsp+8..rsp+64]

                        // Load product into r8-r15 (reuse r12-r15 since we no longer need a_ptr/b_ptr)
                        "mov r8, qword ptr [rsp + 8]",
                        "mov r9, qword ptr [rsp + 16]",
                        "mov r10, qword ptr [rsp + 24]",
                        "mov r11, qword ptr [rsp + 32]",
                        "mov r12, qword ptr [rsp + 40]",
                        "mov r13, qword ptr [rsp + 48]",
                        // r14 still has modulus pointer, r15 still has inv
                        // Load remaining product limbs
                        "mov rbx, qword ptr [rsp + 56]",   // save to rbx temporarily
                        "mov rbp, qword ptr [rsp + 64]",   // save to rbp temporarily

                        // Initialize carry2
                        "mov qword ptr [rsp], 0",

                        // Now we need r14, r15 from product. Store modulus/inv on stack temporarily
                        "push r14",                         // save modulus ptr
                        "push r15",                         // save inv
                        "mov r14, rbx",                    // r14 = product[6]
                        "mov r15, rbp",                    // r15 = product[7]

                        // Recover modulus and inv from stack
                        "mov rbp, qword ptr [rsp]",        // rbp = inv
                        "mov rbx, qword ptr [rsp + 8]",    // rbx = modulus ptr

                        // Round 0: k = r8 * inv
                        "mov rdx, r8",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r8, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r9, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r10, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",

                        "add r12, rax",
                        "mov rax, 0",
                        "adc rax, 0",
                        "mov qword ptr [rsp + 16], rax",   // carry2 at [rsp+16]

                        // Round 1: k = r9 * inv
                        "mov rdx, r9",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r9, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r10, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",

                        "add rax, qword ptr [rsp + 16]",
                        "mov rcx, 0",
                        "adc rcx, 0",
                        "add r13, rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 16], rcx",

                        // Round 2: k = r10 * inv
                        "mov rdx, r10",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r10, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r11, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r13, rsi",
                        "adc rax, 0",

                        "add rax, qword ptr [rsp + 16]",
                        "mov rcx, 0",
                        "adc rcx, 0",
                        "add r14, rax",
                        "adc rcx, 0",
                        "mov qword ptr [rsp + 16], rcx",

                        // Round 3: k = r11 * inv
                        "mov rdx, r11",
                        "imul rdx, rbp",

                        "mulx rcx, rax, qword ptr [rbx + 0]",
                        "add r11, rax",
                        "adc rcx, 0",

                        "mulx rax, rsi, qword ptr [rbx + 8]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r12, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 16]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r13, rsi",
                        "adc rax, 0",
                        "mov rcx, rax",

                        "mulx rax, rsi, qword ptr [rbx + 24]",
                        "add rsi, rcx",
                        "adc rax, 0",
                        "add r14, rsi",
                        "adc rax, 0",

                        "add rax, qword ptr [rsp + 16]",
                        "mov rcx, 0",
                        "adc rcx, 0",
                        "add r15, rax",
                        "adc rcx, 0",
                        // carry2 in rcx

                        // Final reduction
                        "mov r8, r12",
                        "mov r9, r13",
                        "mov r10, r14",
                        "mov r11, r15",

                        "sub r8, qword ptr [rbx + 0]",
                        "sbb r9, qword ptr [rbx + 8]",
                        "sbb r10, qword ptr [rbx + 16]",
                        "sbb r11, qword ptr [rbx + 24]",
                        "sbb rcx, 0",

                        "cmovc r8, r12",
                        "cmovc r9, r13",
                        "cmovc r10, r14",
                        "cmovc r11, r15",

                        // Pop the saved modulus/inv
                        "add rsp, 16",

                        // Deallocate stack space (72 bytes)
                        "add rsp, 72",

                        // Restore callee-saved registers
                        "pop r15",
                        "pop r14",
                        "pop r13",
                        "pop r12",
                        "pop rbp",
                        "pop rbx",

                        a_ptr = in(reg) self.0.as_ptr(),
                        b_ptr = in(reg) rhs.0.as_ptr(),
                        m_ptr = in(reg) $modulus.0.as_ptr(),
                        inv = in(reg) $inv,
                        out("rax") _,
                        out("rcx") _,
                        out("rdx") _,
                        out("rsi") _,
                        out("rdi") _,
                        out("r8") r0,
                        out("r9") r1,
                        out("r10") r2,
                        out("r11") r3,
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
