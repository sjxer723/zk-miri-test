use openvm_circuit_primitives::{
    utils::next_power_of_two_or_zero,
    var_range::{SharedVariableRangeCheckerChip, VariableRangeCheckerBus},
};
use openvm_circuit::{arch::{SystemConfig, ExecutionBus}};
use openvm_circuit::system::{
    memory::{
        interface::MemoryInterface,
        merkle::{DirectCompressionBus, MemoryMerkleBus},
        offline_checker::{MemoryBridge, MemoryBus},
        online::MemoryLogEntry,
        MemoryController, MemoryImage, OfflineMemory, BOUNDARY_AIR_OFFSET, MERKLE_AIR_OFFSET,
    }
};

use openvm_stark_backend::p3_field::{FieldAlgebra, PrimeField32};

const EXECUTION_BUS: ExecutionBus = ExecutionBus(0);
const MEMORY_BUS: MemoryBus = MemoryBus(1);
const RANGE_CHECKER_BUS: usize = 3;

fn new_systemcomplex<F: PrimeField32>() {
    let config = SystemConfig::default();

    let range_bus =
        VariableRangeCheckerBus::new(RANGE_CHECKER_BUS, config.memory_config.decomp);
    let mut bus_idx_max = RANGE_CHECKER_BUS;

    let range_checker = SharedVariableRangeCheckerChip::new(range_bus);
    let memory_controller: openvm_circuit::system::memory::MemoryController<F>  = if config.continuation_enabled {
        bus_idx_max += 2;
        MemoryController::with_persistent_memory(
            MEMORY_BUS,
            config.memory_config,
            range_checker.clone(),
            MemoryMerkleBus(bus_idx_max - 2),
            DirectCompressionBus(bus_idx_max - 1),
        )
    } else {
        MemoryController::with_volatile_memory(
            MEMORY_BUS,
            config.memory_config,
            range_checker.clone(),
        )
    };
}

