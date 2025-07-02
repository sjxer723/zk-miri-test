use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Arc, Mutex, OnceLock},
    str::FromStr
};
use num_bigint::BigUint;
use openvm_circuit::{
    arch::{
        ExecutionBridge, ExecutionBus, ExecutionError, ExecutionState, InstructionExecutor, Streams,
    },
    system::{
        memory::{
            offline_checker::{MemoryBridge, MemoryReadAuxCols, MemoryBus, MemoryWriteAuxCols},
            MemoryAddress, MemoryAuxColsFactory, MemoryController, OfflineMemory, RecordId,
        },
        program::ProgramBus,
    },
};
use openvm_circuit_primitives::var_range::{
    SharedVariableRangeCheckerChip, VariableRangeCheckerBus, VariableRangeCheckerChip
};
use openvm_circuit_primitives::{
    bitwise_op_lookup::{BitwiseOperationLookupBus, SharedBitwiseOperationLookupChip},
    utils::next_power_of_two_or_zero,
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_instructions::{
    instruction::Instruction,
    program::DEFAULT_PC_STEP,
    riscv::{RV32_CELL_BITS, RV32_MEMORY_AS, RV32_REGISTER_AS, RV32_REGISTER_NUM_LIMBS},
    LocalOpcode,
};
use openvm_rv32im_transpiler::{
    Rv32HintStoreOpcode,
    Rv32HintStoreOpcode::{HINT_BUFFER, HINT_STOREW},
};
use openvm_stark_backend::{
    config::{StarkGenericConfig, Val},
    interaction::InteractionBuilder,
    air_builders::{debug::DebugConstraintBuilder, symbolic::SymbolicRapBuilder},
    p3_air::{Air, AirBuilder, BaseAir},
    p3_field::{Field, FieldAlgebra, PrimeField32},
    p3_matrix::{dense::RowMajorMatrix, Matrix},
    prover::types::AirProofInput,
    rap::{AnyRap, BaseAirWithPublicValues, PartitionedBaseAir},
    Chip, ChipUsageGetter, Stateful,
};
use openvm_mod_circuit_builder::{ExprBuilder, ExprBuilderConfig};
use serde::{Deserialize, Serialize};


#[repr(C)]
#[derive(AlignedBorrow, Debug)]
pub struct Rv32HintStoreCols<T> {
    // common
    pub is_single: T,
    pub is_buffer: T,
    // should be 1 for single
    pub rem_words_limbs: [T; RV32_REGISTER_NUM_LIMBS],

    pub from_state: ExecutionState<T>,
    pub mem_ptr_ptr: T,
    pub mem_ptr_limbs: [T; RV32_REGISTER_NUM_LIMBS],
    pub mem_ptr_aux_cols: MemoryReadAuxCols<T>,

    pub write_aux: MemoryWriteAuxCols<T, RV32_REGISTER_NUM_LIMBS>,
    pub data: [T; RV32_REGISTER_NUM_LIMBS],

    // only buffer
    pub is_buffer_start: T,
    pub num_words_ptr: T,
    pub num_words_aux_cols: MemoryReadAuxCols<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct Rv32HintStoreAir {
    pub execution_bridge: ExecutionBridge,
    pub memory_bridge: MemoryBridge,
    pub bitwise_operation_lookup_bus: BitwiseOperationLookupBus,
    pub offset: usize,
}

impl<F: Field> BaseAir<F> for Rv32HintStoreAir {
    fn width(&self) -> usize {
        Rv32HintStoreCols::<F>::width()
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for Rv32HintStoreAir {}
impl<F: Field> PartitionedBaseAir<F> for Rv32HintStoreAir {}

impl<AB: InteractionBuilder> Air<AB> for Rv32HintStoreAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local_cols: &Rv32HintStoreCols<AB::Var> = (*local).borrow();
        let next = main.row_slice(1);
        let next_cols: &Rv32HintStoreCols<AB::Var> = (*next).borrow();

        let timestamp: AB::Var = local_cols.from_state.timestamp;
        let mut timestamp_delta: usize = 0;
        let mut timestamp_pp = || {
            timestamp_delta += 1;
            timestamp + AB::Expr::from_canonical_usize(timestamp_delta - 1)
        };

        builder.assert_bool(local_cols.is_single);
        builder.assert_bool(local_cols.is_buffer);
        builder.assert_bool(local_cols.is_buffer_start);
        builder
            .when(local_cols.is_buffer_start)
            .assert_one(local_cols.is_buffer);
        builder.assert_bool(local_cols.is_single + local_cols.is_buffer);

        let is_valid = local_cols.is_single + local_cols.is_buffer;
        let is_start = local_cols.is_single + local_cols.is_buffer_start;
        // should only be used when is_buffer is true
        let is_end = AB::Expr::ONE - next_cols.is_buffer + next_cols.is_buffer_start;

        let mut rem_words = AB::Expr::ZERO;
        let mut next_rem_words = AB::Expr::ZERO;
        let mut mem_ptr = AB::Expr::ZERO;
        let mut next_mem_ptr = AB::Expr::ZERO;
        for i in (0..RV32_REGISTER_NUM_LIMBS).rev() {
            rem_words = rem_words * AB::F::from_canonical_u32(1 << RV32_CELL_BITS)
                + local_cols.rem_words_limbs[i];
            next_rem_words = next_rem_words * AB::F::from_canonical_u32(1 << RV32_CELL_BITS)
                + next_cols.rem_words_limbs[i];
            mem_ptr = mem_ptr * AB::F::from_canonical_u32(1 << RV32_CELL_BITS)
                + local_cols.mem_ptr_limbs[i];
            next_mem_ptr = next_mem_ptr * AB::F::from_canonical_u32(1 << RV32_CELL_BITS)
                + next_cols.mem_ptr_limbs[i];
        }

        // read mem_ptr
        self.memory_bridge
            .read(
                MemoryAddress::new(
                    AB::F::from_canonical_u32(RV32_REGISTER_AS),
                    local_cols.mem_ptr_ptr,
                ),
                local_cols.mem_ptr_limbs,
                timestamp_pp(),
                &local_cols.mem_ptr_aux_cols,
            )
            .eval(builder, is_start.clone());

        // read num_words
        self.memory_bridge
            .read(
                MemoryAddress::new(
                    AB::F::from_canonical_u32(RV32_REGISTER_AS),
                    local_cols.num_words_ptr,
                ),
                local_cols.rem_words_limbs,
                timestamp_pp(),
                &local_cols.num_words_aux_cols,
            )
            .eval(builder, local_cols.is_buffer_start);

        builder
            .when(local_cols.is_single)
            .assert_one(rem_words.clone());

        for i in 0..RV32_REGISTER_NUM_LIMBS / 2 {
            // preventing mem_ptr overflow
            self.bitwise_operation_lookup_bus
                .send_range(
                    local_cols.mem_ptr_limbs[2 * i],
                    local_cols.mem_ptr_limbs[(2 * i) + 1],
                )
                .eval(builder, is_valid.clone());
            // checking that hint is bytes
            self.bitwise_operation_lookup_bus
                .send_range(local_cols.data[2 * i], local_cols.data[(2 * i) + 1])
                .eval(builder, is_valid.clone());
        }

        // write hint
        self.memory_bridge
            .write(
                MemoryAddress::new(AB::F::from_canonical_u32(RV32_MEMORY_AS), mem_ptr.clone()),
                local_cols.data,
                timestamp_pp(),
                &local_cols.write_aux,
            )
            .eval(builder, is_valid.clone());

        let expected_opcode = (local_cols.is_single
            * AB::F::from_canonical_usize(HINT_STOREW as usize + self.offset))
            + (local_cols.is_buffer
                * AB::F::from_canonical_usize(HINT_BUFFER as usize + self.offset));
        let to_pc = local_cols.from_state.pc + AB::F::from_canonical_u32(DEFAULT_PC_STEP);
        self.execution_bridge
            .execute(
                expected_opcode,
                [
                    local_cols.is_buffer * (local_cols.num_words_ptr),
                    local_cols.mem_ptr_ptr.into(),
                    AB::Expr::ZERO,
                    AB::Expr::from_canonical_u32(RV32_REGISTER_AS),
                    AB::Expr::from_canonical_u32(RV32_MEMORY_AS),
                ],
                local_cols.from_state,
                ExecutionState {
                    pc: to_pc,
                    timestamp: timestamp
                        + (rem_words.clone() * AB::F::from_canonical_usize(timestamp_delta)),
                },
            )
            .eval(builder, is_start.clone());

        // buffer transition

        builder
            .when(local_cols.is_buffer)
            .when(is_end.clone())
            .assert_one(rem_words.clone());

        let mut when_buffer_transition =
            builder.when(local_cols.is_buffer * (AB::Expr::ONE - is_end.clone()));
        when_buffer_transition.assert_one(rem_words.clone() - next_rem_words.clone());
        when_buffer_transition.assert_eq(
            next_mem_ptr.clone() - mem_ptr.clone(),
            AB::F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS),
        );
        when_buffer_transition.assert_eq(
            timestamp + AB::F::from_canonical_usize(timestamp_delta),
            next_cols.from_state.timestamp,
        );
    }
}


// mod tests {
//     use super::*;

//     pub const LIMB_BITS: usize = 8;

//     fn test_eval(){
//         // Initialize the builder for the Air
//         // TBD
//         // let builder = ...;

//         let execution_bus = ExecutionBus(1);
//         let memory_bus = MemoryBus(2);
//         let program_bus = ProgramBus(3);
//         let variable_range_checker_bus = VariableRangeCheckerBus::new(4, 0);
//         let bitwise_operation_lookup_bus = BitwiseOperationLookupBus::new(5);
//         let execution_bridge = ExecutionBridge::new(execution_bus, program_bus);
//         let memory_bridge = MemoryBridge::new(memory_bus, 0, variable_range_checker_bus);
//         let offset = 0; // for HINT_STOREW and HINT_BUFFER

//         let air = Rv32HintStoreAir {
//             execution_bridge,
//             memory_bridge,
//             bitwise_operation_lookup_bus,
//             offset,
//         };
//         // air.eval(&mut builder);
//     }

// }


