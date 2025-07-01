use std::{
    borrow::{Borrow, BorrowMut},
    sync::Arc,
};

use openvm_circuit_primitives::{
    is_less_than_array::{
        IsLtArrayAuxCols, IsLtArrayIo, IsLtArraySubAir, IsLtArrayWhenTransitionAir,
    },
    utils::implies,
    var_range::{SharedVariableRangeCheckerChip, VariableRangeCheckerBus},
    SubAir, TraceSubRowGenerator,
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    config::{StarkGenericConfig, Val},
    interaction::InteractionBuilder,
    p3_air::{Air, AirBuilder, BaseAir},
    p3_field::{Field, FieldAlgebra, PrimeField32},
    p3_matrix::{dense::RowMajorMatrix, Matrix},
    p3_maybe_rayon::prelude::*,
    prover::types::AirProofInput,
    rap::{BaseAirWithPublicValues, PartitionedBaseAir},
    AirRef, Chip, ChipUsageGetter,
};



use openvm_circuit::system::memory::{
    offline_checker::{MemoryBus},
    MemoryAddress,
};

pub const AUX_LEN: usize = 2;

/// Address stored as address space, pointer
const ADDR_ELTS: usize = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, AlignedBorrow)]
pub struct VolatileBoundaryCols<T> {
    pub addr_space: T,
    pub pointer: T,

    pub initial_data: T,
    pub final_data: T,
    pub final_timestamp: T,

    /// Boolean. `1` if a non-padding row with a valid touched address, `0` if it is a padding row.
    pub is_valid: T,
    pub addr_lt_aux: IsLtArrayAuxCols<T, ADDR_ELTS, AUX_LEN>,
}

#[derive(Clone, Debug)]
pub struct VolatileBoundaryAir {
    pub memory_bus: MemoryBus,
    pub addr_lt_air: IsLtArrayWhenTransitionAir<ADDR_ELTS>,
}

impl VolatileBoundaryAir {
    pub fn new(
        memory_bus: MemoryBus,
        addr_space_max_bits: usize,
        pointer_max_bits: usize,
        range_bus: VariableRangeCheckerBus,
    ) -> Self {
        let addr_lt_air =
            IsLtArraySubAir::<ADDR_ELTS>::new(range_bus, addr_space_max_bits.max(pointer_max_bits))
                .when_transition();
        Self {
            memory_bus,
            addr_lt_air,
        }
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for VolatileBoundaryAir {}
impl<F: Field> PartitionedBaseAir<F> for VolatileBoundaryAir {}
impl<F: Field> BaseAir<F> for VolatileBoundaryAir {
    fn width(&self) -> usize {
        VolatileBoundaryCols::<F>::width()
    }
}

impl<AB: InteractionBuilder> Air<AB> for VolatileBoundaryAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let [local, next] = [0, 1].map(|i| main.row_slice(i));
        let local: &VolatileBoundaryCols<_> = (*local).borrow();
        let next: &VolatileBoundaryCols<_> = (*next).borrow();

        builder.assert_bool(local.is_valid);

        // Ensuring all non-padding rows are at the bottom
        builder
            .when_transition()
            .assert_one(implies(next.is_valid, local.is_valid));

        // Assert local addr < next addr when next.is_valid
        // This ensures the addresses in non-padding rows are all sorted
        let lt_io = IsLtArrayIo {
            x: [local.addr_space, local.pointer].map(Into::into),
            y: [next.addr_space, next.pointer].map(Into::into),
            out: AB::Expr::ONE,
            count: next.is_valid.into(),
        };
        // N.B.: this will do range checks (but not other constraints) on the last row if the first row has is_valid = 1 due to wraparound
        self.addr_lt_air
            .eval(builder, (lt_io, (&local.addr_lt_aux).into()));

        // Write the initial memory values at initial timestamps
        self.memory_bus
            .send(
                MemoryAddress::new(local.addr_space, local.pointer),
                vec![local.initial_data],
                AB::Expr::ZERO,
            )
            .eval(builder, local.is_valid);

        // Read the final memory values at last timestamps when written to
        self.memory_bus
            .receive(
                MemoryAddress::new(local.addr_space, local.pointer),
                vec![local.final_data],
                local.final_timestamp,
            )
            .eval(builder, local.is_valid);
    }
}