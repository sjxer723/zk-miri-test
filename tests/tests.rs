use miri_test::{
    bus::BitwiseOperationLookupBus,
    core::{Rv32AuipcCoreAir, VmCoreAir},
    openvm_stark_backend::{
        air::AirBuilder,
        field::{F, FieldAlgebra},
        interaction::{BusIndex, InteractionBuilder},
    },
};

mod tests {
    use super::*;

    // My own implementation for InteractionBuilder
    struct Rv32AuipcCoreAirBuilder {
        bus: BitwiseOperationLookupBus,
    }

    impl Rv32AuipcCoreAirBuilder {
        pub fn new() -> Self {
            Rv32AuipcCoreAirBuilder { bus: BitwiseOperationLookupBus::new(0) }
        }
    }

    impl AirBuilder for Rv32AuipcCoreAirBuilder {
        type F = F; // Placeholder for the actual field type
        type Expr = F; // Placeholder for the actual expression type
        type Var = F; // Placeholder for the actual variable type

        fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
            // Implementation of assert_zero
        }

        fn assert_one<I: Into<Self::Expr>>(&mut self, x: I) {
            self.assert_zero(x.into() - Self::Expr::ONE);
        }
    }

    impl InteractionBuilder for Rv32AuipcCoreAirBuilder {
        fn push_interaction<E: Into<Self::Expr>>(
            &mut self,
            bus_index: BusIndex,
            fields: impl IntoIterator<Item = E>,
            count: impl Into<Self::Expr>,
            count_weight: u32,
        ) {
        }
    }

    #[test]
    pub fn test_Rv32AuipcCoreAir_eval() {
        let mut builder = Rv32AuipcCoreAirBuilder::new();
        let local_core = vec![
            F::from_i32(0),  // is_valid
            F::from_i32(1),  // imm_limbs[0]
            F::from_i32(2),  // imm_limbs[1]
            F::from_i32(3),  // imm_limbs[2]
            F::from_i32(4),  // imm_limbs[3]
            F::from_i32(5),  // pc_limbs[0]
            F::from_i32(6),  // pc_limbs[1]
            F::from_i32(7),  // pc_limbs[2]
            F::from_i32(8),  // pc_limbs[3]
            F::from_i32(9),  // rd_data[0]
            F::from_i32(10), // rd_data[1]
            F::from_i32(11), // rd_data[2]
            F::from_i32(12), // rd_data[3]
        ];
        let from_pc = F::from_i32(14); // Example value for from_pc
        let local_core_slice: &[F] = &local_core;
        Rv32AuipcCoreAir { bus: BitwiseOperationLookupBus::new(0) }.eval(
            &mut builder,
            local_core_slice,
            from_pc,
        );
    }
}
