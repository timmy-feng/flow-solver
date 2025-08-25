use std::ops::{Add, Mul, Sub};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Modnum(pub i32);

const HASH_MODULO: i32 = 1_000_000_007;

impl Add for Modnum {
    type Output = Self;

    fn add(self, oth: Self) -> Self {
        let sum = self.0 + oth.0;
        return Self(if sum >= HASH_MODULO { sum - HASH_MODULO } else { sum });
    }
}

impl Sub for Modnum {
    type Output = Self;

    fn sub(self, oth: Self) -> Self {
        let diff = self.0 - oth.0;
        return Self(if diff < 0 { diff + HASH_MODULO } else { diff });
    }
}

impl Mul for Modnum {
    type Output = Self;

    fn mul(self, oth: Self) -> Self {
        return Self(((self.0 as i64) * (oth.0 as i64) % (HASH_MODULO as i64)) as i32);
    }
}