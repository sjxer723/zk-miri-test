use hex_literal::hex;
use openvm_algebra_moduli_macros;
use openvm_ecc_guest::{
    CyclicGroup, Group, msm,
    p256::{P256Coord, P256Point, P256Scalar},
    weierstrass::WeierstrassPoint,
};
use openvm_ecc_sw_macros;

mod tests {
    use super::*;

    #[test]
    pub fn test_cyclicgroup_for_p256point() {
        let (gen_x, gen_y) = P256Point::GENERATOR.into_coords();
        let generator = P256Point::from_xy(gen_x, gen_y).unwrap();
        let (neg_x, neg_y) = P256Point::NEG_GENERATOR.into_coords();
        let neg_generator = P256Point::from_xy(neg_x, neg_y).unwrap();
    }
}
