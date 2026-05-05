//! Lagged Fibonacci PRNG and integer math utilities.
//!
//! This module implements the core PRNG algorithm shared by src/xrand.c and
//! src/db.c. Both use the same lagged Fibonacci generator; they differ only
//! in seeding:
//!
//! - **xrand.c** (standalone test): self-seeds with {1, 1, fib...}
//! - **db.c** (runtime): seeds via `init_mm()` using `current_time`
//!
//! The current Rust implementation uses the deterministic self-seed path
//! (matching xrand.c). To replace db.c symbols at runtime, `init_mm` seeding
//! from an external timestamp must be added first.
//!
//! `isquare()` matches src/db.c (line 3445) which uses an iterative algorithm
//! with a known off-by-one for perfect squares (strict-less-than loop).

use std::sync::Mutex;

/// PRNG state: mirrors the C `static int rgiState[3+55]`.
struct PrngState {
    /// rgiState[3..58] holds the 55-element lagged Fibonacci state.
    /// rgiState[0] = initialized flag, [1] = iState1, [2] = iState2.
    rgi_state: [i32; 58],
}

const MASK: i32 = (1 << 30) - 1;

impl PrngState {
    fn new() -> Self {
        PrngState { rgi_state: [0; 58] }
    }

    /// Initialize the state (matches the C init block in number_mm).
    fn init(&mut self) {
        self.rgi_state[3] = 1; // piState[3+0] = 1
        self.rgi_state[4] = 1; // piState[3+1] = 1
        for i in 5..58 {
            // piState[iState] = (piState[iState-1] + piState[iState-2]) & MASK
            self.rgi_state[i] = (self.rgi_state[i - 1] + self.rgi_state[i - 2]) & MASK;
        }
        self.rgi_state[0] = 1; // initialized = 1
        self.rgi_state[1] = 58 - 55; // 3+55 - 55 = 3
        self.rgi_state[2] = 58 - 24; // 3+55 - 24 = 34
    }

    /// Generate next random value (matches C `number_mm`).
    fn number_mm(&mut self) -> i32 {
        if self.rgi_state[0] == 0 {
            self.init();
        }

        let i_state1 = self.rgi_state[1] as usize;
        let i_state2 = self.rgi_state[2] as usize;

        let i_rand = (self.rgi_state[i_state1] + self.rgi_state[i_state2]) & MASK;
        self.rgi_state[i_state1] = i_rand;

        let mut new_state1 = i_state1 + 1;
        if new_state1 == 58 {
            // 3+55
            new_state1 = 3;
        }

        let mut new_state2 = i_state2 + 1;
        if new_state2 == 58 {
            new_state2 = 3;
        }

        self.rgi_state[1] = new_state1 as i32;
        self.rgi_state[2] = new_state2 as i32;

        i_rand >> 6
    }
}

/// Global PRNG state behind a Mutex for thread safety.
static PRNG: Mutex<Option<PrngState>> = Mutex::new(None);

/// Generate a random number using the lagged Fibonacci PRNG.
/// Returns a value in [0, 2^24 - 1].
///
/// This matches the shared C generation step after this module's
/// deterministic xrand-compatible seed path is initialized.
pub fn number_mm() -> i32 {
    let mut guard = PRNG.lock().unwrap_or_else(|p| p.into_inner());
    let state = guard.get_or_insert_with(PrngState::new);
    state.number_mm()
}

/// Generate a random number in the range [from, to] inclusive.
///
/// Improvement over C: uses i64 internally to prevent overflow when
/// `to - from + 1` exceeds i32::MAX or when power-of-2 computation
/// would shift past 31 bits. The C version silently overflows on
/// ranges wider than ~2^30; this version handles the full i32 range
/// by combining multiple number_mm() calls for wide ranges.
pub fn number_range(from: i32, to: i32) -> i32 {
    let range = (to as i64) - (from as i64) + 1;
    if range <= 1 {
        return from;
    }

    // number_mm() produces 24 bits of randomness (0..2^24-1).
    // For ranges that fit in 24 bits, use single-call rejection sampling
    // matching C behavior for typical game use. For wider ranges, combine
    // two calls to get 48 bits of randomness.
    let raw = if range <= (1i64 << 24) {
        // Find next power of 2 >= range
        let mut power: i64 = 2;
        while power < range {
            power <<= 1;
        }
        // Rejection sampling (single call, matches C for normal game ranges)
        loop {
            let number = (number_mm() as i64) & (power - 1);
            if number < range {
                break number;
            }
        }
    } else {
        // Wide range: combine two 24-bit values for 48 bits of entropy
        let mut power: i64 = 2;
        while power < range {
            power <<= 1;
        }
        loop {
            let hi = (number_mm() as i64) << 24;
            let lo = number_mm() as i64;
            let number = (hi | lo) & (power - 1);
            if number < range {
                break number;
            }
        }
    };

    (from as i64 + raw) as i32
}

/// Integer square root matching the runtime implementation in src/db.c
/// (line 3445).
///
/// Note: this differs from src/xrand.c which uses `(int)sqrt((double)num)`.
/// The db.c version has a known off-by-one for perfect squares (e.g.,
/// isquare(4) returns 1, not 2) and returns 1 for negative inputs because
/// the loop never runs. Both behaviors are preserved for gameplay parity.
pub fn isquare(num: i32) -> i32 {
    if num == 0 {
        return 0;
    }
    if num == 1 {
        return 1;
    }
    let mut i: i32 = 2;
    while (i as i64) * (i as i64) < (num as i64) {
        i += 1;
    }
    i - 1
}

/// Reset the PRNG state (for testing reproducibility).
/// Not part of the C API -- used only in tests.
#[cfg(test)]
pub fn reset_state() {
    let mut guard = PRNG.lock().unwrap_or_else(|p| p.into_inner());
    *guard = None;
}

// --- FFI exports ---

/// FFI wrapper for number_mm.
#[no_mangle]
pub extern "C" fn number_mm_ffi() -> i32 {
    std::panic::catch_unwind(number_mm).unwrap_or(0)
}

/// FFI wrapper for number_range.
#[no_mangle]
pub extern "C" fn number_range_ffi(from: i32, to: i32) -> i32 {
    std::panic::catch_unwind(|| number_range(from, to)).unwrap_or(from)
}

/// FFI wrapper for isquare.
#[no_mangle]
pub extern "C" fn isquare_ffi(num: i32) -> i32 {
    std::panic::catch_unwind(|| isquare(num)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_number_mm_deterministic_sequence() {
        reset_state();
        // After init, first few values should be deterministic
        let v1 = number_mm();
        let v2 = number_mm();
        let v3 = number_mm();

        // All should be in [0, 2^24 - 1]
        assert!(v1 >= 0 && v1 < (1 << 24));
        assert!(v2 >= 0 && v2 < (1 << 24));
        assert!(v3 >= 0 && v3 < (1 << 24));

        // Reset and verify same sequence
        reset_state();
        assert_eq!(number_mm(), v1);
        assert_eq!(number_mm(), v2);
        assert_eq!(number_mm(), v3);
    }

    #[test]
    #[serial]
    fn test_number_range_bounds() {
        reset_state();
        for _ in 0..1000 {
            let val = number_range(5, 10);
            assert!(val >= 5 && val <= 10, "Got {} outside [5,10]", val);
        }
    }

    #[test]
    fn test_number_range_single_value() {
        assert_eq!(number_range(7, 7), 7);
        assert_eq!(number_range(7, 6), 7); // to < from => returns from
    }

    #[test]
    #[serial]
    fn test_number_range_wide_range_no_overflow() {
        // This would overflow i32 in the C version: range = i32::MAX - i32::MIN + 1
        // Our Rust version handles it safely via i64 promotion.
        let val = number_range(i32::MIN, i32::MAX);
        // Just verify it doesn't panic and returns something in range
        assert!(val >= i32::MIN && val <= i32::MAX);

        // Large positive range that overflows i32 in C
        let val2 = number_range(-1_000_000_000, 1_000_000_000);
        assert!(val2 >= -1_000_000_000 && val2 <= 1_000_000_000);
    }

    #[test]
    #[serial]
    fn test_number_range_distribution() {
        reset_state();
        let mut counts = [0u32; 6];
        let trials = 60000;
        for _ in 0..trials {
            let val = number_range(0, 5);
            counts[val as usize] += 1;
        }
        // Each bucket should have roughly trials/6 = 10000
        // Allow 30% deviation
        let expected = trials / 6;
        for (i, &count) in counts.iter().enumerate() {
            assert!(
                count > expected * 70 / 100 && count < expected * 130 / 100,
                "Bucket {} has {} (expected ~{})",
                i,
                count,
                expected
            );
        }
    }

    #[test]
    fn test_isquare_values() {
        // Matches db.c behavior: returns largest i such that i*i < num
        assert_eq!(isquare(0), 0);
        assert_eq!(isquare(-5), 1); // db.c: loop does not run for negatives, returns 2-1=1
        assert_eq!(isquare(1), 1);
        assert_eq!(isquare(4), 1); // db.c: 2*2=4, 4<4 false, return 2-1=1
        assert_eq!(isquare(5), 2); // db.c: 2*2=4, 4<5 true; 3*3=9, 9<5 false, return 3-1=2
        assert_eq!(isquare(9), 2); // db.c: 3*3=9, 9<9 false, return 3-1=2
        assert_eq!(isquare(10), 3); // db.c: 3*3=9, 9<10 true; 4*4=16, 16<10 false, return 4-1=3
        assert_eq!(isquare(15), 3);
        assert_eq!(isquare(16), 3); // db.c: 4*4=16, 16<16 false, return 4-1=3
        assert_eq!(isquare(17), 4); // db.c: 4*4=16, 16<17 true; 5*5=25, 25<17 false, return 5-1=4
        assert_eq!(isquare(100), 9); // db.c: 10*10=100, 100<100 false, return 10-1=9
        assert_eq!(isquare(99), 9); // db.c: 9*9=81, 81<99; 10*10=100, 100<99 false, return 10-1=9
    }

    #[test]
    fn test_isquare_large_values() {
        // db.c: for 1000000, i iterates until i*i >= 1000000
        // 1000*1000 = 1000000, 1000000 < 1000000 false, return 1000-1 = 999
        assert_eq!(isquare(1000000), 999);
        // For i32::MAX (2147483647): sqrt ~= 46340.95
        // 46341*46341 = 2147488281 > i32::MAX, so loop: 46341^2 >= 2147483647
        // Actually in i64: 46340*46340 = 2147395600 < 2147483647, 46341*46341 = 2147488281 >= 2147483647
        // So return 46341-1 = 46340
        assert_eq!(isquare(2147483647), 46340);
    }

    #[test]
    #[serial]
    fn test_number_mm_range() {
        reset_state();
        // Generate many values, all must be in valid range
        for _ in 0..10000 {
            let v = number_mm();
            assert!(v >= 0 && v < (1 << 24), "number_mm returned {}", v);
        }
    }
}
