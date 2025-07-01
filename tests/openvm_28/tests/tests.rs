use openvm_circuit_primitives::var_range::{VariableRangeCheckerBus, VariableRangeCheckerChip};
use openvm_stark_backend::p3_field::{Field, FieldAlgebra, PrimeField32};
use openvm_stark_sdk::p3_baby_bear::BabyBear;

pub fn decompose<F: Field>(
    chip: &VariableRangeCheckerChip,
    mut value: u32,
    bits: usize,
    limbs: &mut [F],
) {
    debug_assert!(
        limbs.len() <= bits.div_ceil(chip.range_max_bits()),
        "Not enough limbs: len {}",
        limbs.len()
    );
    let mask = (1 << chip.range_max_bits()) - 1;
    let mut bits_remaining = bits;
    for limb in limbs.iter_mut() {
        let limb_u32 = value & mask;
        *limb = F::from_canonical_u32(limb_u32);
        chip.add_count(limb_u32, bits_remaining.min(chip.range_max_bits()));

        value >>= chip.range_max_bits();
        bits_remaining = bits_remaining.saturating_sub(chip.range_max_bits());
    }
    debug_assert_eq!(value, 0);
    debug_assert_eq!(bits_remaining, 0);
}

mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Not enough limbs: len 4")]
    pub fn test_decompose() {
        let bus = VariableRangeCheckerBus::new(0, 5);

        let range_checker = VariableRangeCheckerChip::new(bus);

        let x = BabyBear::ONE;
        decompose(&range_checker, 0, 15, &mut [x, x, x, x]);
    }
}
