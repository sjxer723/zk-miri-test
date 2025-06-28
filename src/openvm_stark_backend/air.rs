use super::field::{Field, FieldAlgebra};
use core::ops::{Add, Mul, Sub};

// Please refer to
// https://github.com/Plonky3/Plonky3/blob/b2f9bf3fbba465f1a04f595ae369889ffd4b66ca/air/src/air.rs#L29
// for full implementation details.

pub trait AirBuilder {
    type F: Field;

    type Expr: FieldAlgebra
        + From<Self::F>
        + Add<Self::Var, Output = Self::Expr>
        + Add<Self::F, Output = Self::Expr>
        + Sub<Self::Var, Output = Self::Expr>
        + Sub<Self::F, Output = Self::Expr>
        + Mul<Self::Var, Output = Self::Expr>
        + Mul<Self::F, Output = Self::Expr>;

    type Var: Into<Self::Expr>
        + Copy
        + Send
        + Sync
        + Add<Self::F, Output = Self::Expr>
        + Add<Self::Var, Output = Self::Expr>
        + Add<Self::Expr, Output = Self::Expr>
        + Sub<Self::F, Output = Self::Expr>
        + Sub<Self::Var, Output = Self::Expr>
        + Sub<Self::Expr, Output = Self::Expr>
        + Mul<Self::F, Output = Self::Expr>
        + Mul<Self::Var, Output = Self::Expr>
        + Mul<Self::Expr, Output = Self::Expr>;

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I);

    fn assert_one<I: Into<Self::Expr>>(&mut self, x: I) {
        self.assert_zero(x.into() - Self::Expr::ONE);
    }

    fn assert_eq<I1: Into<Self::Expr>, I2: Into<Self::Expr>>(&mut self, x: I1, y: I2) {
        self.assert_zero(x.into() - y.into());
    }

    /// Assert that `x` is a boolean, i.e. either 0 or 1.
    fn assert_bool<I: Into<Self::Expr>>(&mut self, x: I) {
        let x = x.into();
        self.assert_zero(x.clone() * (x - Self::Expr::ONE));
    }
}
