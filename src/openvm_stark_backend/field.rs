// Please refer to
// https://github.com/Plonky3/Plonky3/blob/f37dc2a59ad93fe6153091e11671d3d53708bcbb/field/src/field.rs
// for full implementation details.

use core::{
    fmt::Debug,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub trait FieldAlgebra:
    Sized
    + Default
    + Clone
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<Output = Self>
    + MulAssign
    + Sum
    + Product
    + Debug
{
    type F: Field;

    const ZERO: Self;
    const ONE: Self;
}

pub trait Field: FieldAlgebra<F = Self> {}

// My own implementation of a field type
#[derive(Clone, Debug, Copy)]
pub struct F {
    value: i32, // Placeholder for the actual field value type
}

impl F {
    pub fn new(value: i32) -> Self {
        F { value }
    }

    pub fn zero() -> Self {
        F { value: 0 }
    }

    pub fn from_i32(value: i32) -> Self {
        F { value }
    }
}

impl Default for F {
    fn default() -> Self {
        F { value: 0 }
    }
}

impl Add for F {
    type Output = F;

    fn add(self, rhs: F) -> F {
        F { value: self.value + rhs.value }
    }
}

impl AddAssign for F {
    fn add_assign(&mut self, rhs: F) {
        self.value += rhs.value;
    }
}

impl Sub for F {
    type Output = F;

    fn sub(self, rhs: F) -> F {
        F { value: self.value - rhs.value }
    }
}

impl SubAssign for F {
    fn sub_assign(&mut self, rhs: F) {
        self.value -= rhs.value;
    }
}

impl Mul for F {
    type Output = F;

    fn mul(self, rhs: F) -> F {
        F { value: self.value * rhs.value }
    }
}

impl MulAssign for F {
    fn mul_assign(&mut self, rhs: F) {
        self.value *= rhs.value;
    }
}

impl Neg for F {
    type Output = F;

    fn neg(self) -> F {
        F { value: -self.value }
    }
}

impl Sum for F {
    fn sum<I: Iterator<Item = F>>(iter: I) -> F {
        iter.fold(F { value: 0 }, |a, b| a + b)
    }
}

impl Product for F {
    fn product<I: Iterator<Item = F>>(iter: I) -> F {
        iter.fold(F { value: 1 }, |a, b| a * b)
    }
}

impl Field for F {}

impl FieldAlgebra for F {
    type F = F;

    const ZERO: Self = F { value: 0 };
    const ONE: Self = F { value: 1 };
}
