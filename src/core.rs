use crate::{
    bus::BitwiseOperationLookupBus, openvm_stark_backend::interaction::InteractionBuilder,
};

// Please refer to
// https://github.com/openvm-org/openvm/blob/336f1a475e5aa3513c4c5a266399f4128c119bba/extensions/rv32im/circuit/src/auipc/core.rs#L106-L110
// for full implementation details.

pub const RV32_REGISTER_NUM_LIMBS: usize = 4;

#[derive(Clone)]
pub struct Rv32AuipcCoreCols<T> {
    pub is_valid: T,
    // The limbs of the immediate except the least significant limb since it is always 0
    pub imm_limbs: [T; RV32_REGISTER_NUM_LIMBS - 1],
    // The limbs of the PC except the most significant and the least significant limbs
    pub pc_limbs: [T; RV32_REGISTER_NUM_LIMBS - 1],
    pub rd_data: [T; RV32_REGISTER_NUM_LIMBS],
}

pub trait VmCoreAir<AB>
where
    AB: InteractionBuilder,
{
    /// Returns `(to_pc, interface)`.
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var], from_pc: AB::Var);
}

#[derive(Clone)]
pub struct Rv32AuipcCoreAir {
    pub bus: BitwiseOperationLookupBus,
}

// My own implementation for borrow
fn trusted_borrow<T: std::marker::Copy>(x: &[T]) -> Rv32AuipcCoreCols<T> {
    // x.borrow().clone()
    Rv32AuipcCoreCols {
        is_valid: x[0],
        imm_limbs: [x[1]; RV32_REGISTER_NUM_LIMBS - 1],
        pc_limbs: [x[2]; RV32_REGISTER_NUM_LIMBS - 1],
        rd_data: [x[3]; RV32_REGISTER_NUM_LIMBS],
    }
}

impl<AB> VmCoreAir<AB> for Rv32AuipcCoreAir
where
    AB: InteractionBuilder,
{
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var], from_pc: AB::Var) {
        // let cols: &Rv32AuipcCoreCols<AB::Var> = (*local_core).borrow();
        let cols: &Rv32AuipcCoreCols<AB::Var> = &trusted_borrow(local_core);

        let Rv32AuipcCoreCols { is_valid, imm_limbs, pc_limbs, rd_data } = *cols;
        builder.assert_bool(is_valid);
        let limbs = [imm_limbs, pc_limbs].concat();
        // Repalce the following code with the bottom code
        // for i in 0..(RV32_REGISTER_NUM_LIMBS - 2) {
        //     self.bus
        //         .send_range(limbs[i * 2], limbs[i * 2 + 1])
        //         .eval(builder, is_valid);
        // }
        let mut i = 0;
        while i < RV32_REGISTER_NUM_LIMBS - 2 {
            self.bus.send_range(limbs[i * 2], limbs[i * 2 + 1]).eval(builder, is_valid);
            i += 1;
        }
    }
}
