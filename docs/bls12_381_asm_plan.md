# BLS12-381 Assembly Backend Plan

## Objectives
- Provide an x86_64 assembly backend for the BLS12-381 base field (`Fp`) that mirrors the existing BN256 implementation in `src/bn256/assembly.rs`.
- Preserve the current constant-time semantics while targeting BMI2/ADX instruction sets.
- Make the backend pluggable behind the existing `asm` feature flag so downstream users can opt-in without API changes.

## Architectural Constraints
- Target architecture: x86_64 only (enforced by `build.rs` today). Require `target_feature = "bmi2"` and `target_feature = "adx"` for the Montgomery multiply/reduce path.
- `Fp` elements are 6×64-bit limbs (384 bits); we cannot directly reuse the 4-limb macro from BN256.
- The module must remain `no_std` friendly and should avoid touching the stack (`nostack`) wherever possible to keep ABI compatibility with previous releases.

## Implementation Steps

1. **Create `src/bls12_381/assembly.rs`:**
   - Start from `src/bn256/assembly.rs` as a reference.
   - Implement a new macro (e.g. `field_arithmetic_asm_fp!`) specialized for 6-limb Montgomery fields. The macro should emit:
     - `double`, `add`, `sub`, `neg` routines that mirror the limb-wise logic currently written in Rust.
     - `square` implemented by delegating to `mul` just like the BN256 version.
     - `mul` using a 6×6 Coarsely Integrated Operand Scanning (CIOS) variant with BMI2/ADX (similar structure to the 4-limb version but extended to 6 iterations). Carry handling will need two extra temporaries (`r16`, `r17`) or stack spill; prefer additional callee-saved registers plus `pushfq`/`popfq` if required.
     - `montgomery_reduce_384` helper that mirrors the `montgomery_reduce_256` routine but iterates over 6 limbs (t0…t5) and emits the correct number of reduction rounds (6).
   - Expose the macro via `pub(crate) use` like the BN256 module.

2. **Thread the macro into `Fp`:**
   - In `src/bls12_381/mod.rs`, add `#[cfg(feature = "asm")] mod assembly;`.
   - In `src/bls12_381/fp.rs`, gate the existing pure-Rust implementation with `#[cfg(not(feature = "asm"))]` and, under `#[cfg(feature = "asm")]`, `use crate::bls12_381::assembly::field_arithmetic_asm_fp;` followed by `field_arithmetic_asm_fp!(Fp, MODULUS_CONST, INV);`.
   - Keep the current Rust implementations (renamed or `cfg`-guarded) available for tests/reference builds so we still have a canonical fallback even when ASM is enabled.
   - Future work (optional): repeat the same pattern for `scalar.rs` once `Fp` is stable, reusing the 4-limb macro from BN256.

3. **Instruction Scheduling & Safety:**
   - Follow the same safety invariants as BN256: mark blocks as `options(pure, readonly, nostack)` when no stack usage occurs.
   - Validate register allocation so we avoid clobbering reserved registers (e.g. keep `r12`…`r15` for temporaries). Document any assumptions about Windows vs. SysV calling conventions (BN256 currently assumes SysV; BLS12 implementation should do the same).
   - Consider generating the long multiply/reduce sections via a script (Constantine / evmone references) to avoid manual mistakes. Capture that script or methodology in comments for maintainability.

4. **Testing & Verification:**
   - Add randomized equivalence tests (introduced in this PR) that compare `Fp` operations against a BigUint-based reference reducer. These run in both pure and ASM builds.
   - Once assembly is written, run `cargo test --all-features --release` on x86_64 hardware to exercise the new backend plus the existing suite (`src/tests`, `bls12_381/tests`, benches where applicable).
   - Use `RUSTFLAGS='-C target-feature=+bmi2,+adx'` during local testing to guarantee the necessary instructions are available. Fallback to pure Rust on unsupported CPUs by keeping the feature opt-in.

5. **Benchmarking & Rollout:**
   - Extend `benches/bn256_field.rs` with an optional `bls12_381_field` bench (or create a new one) to quantify improvements once assembly lands.
   - Document the feature flag behavior in `README.md` (e.g. enabling `asm` now speeds up both BN256 and BLS12-381 fields).
   - After landing, monitor downstream consumers for build issues, especially on Windows where inline assembly ABI differences may surface.

## Open Questions / Follow-ups
- Do we want a shared 6-limb macro for other curves (e.g. Pasta) to avoid duplicating assembly later?
- Should we gate at runtime on `is_x86_feature_detected!` to fail gracefully if ADX/BMI2 are missing even though the crate was compiled with `asm`?

This plan keeps the critical path small: get a working 6-limb macro, wire it into `Fp`, rely on the new reference tests to ensure functional parity, and iterate on performance once correctness is nailed down.
