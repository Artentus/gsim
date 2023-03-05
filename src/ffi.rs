use crate::*;

pub type Result = u32;

pub const SUCCESS: Result = 0x00_000000;
const GENERAL_ERROR: Result = 0x01_000000;
const SPECIFIC_ERROR: Result = 0x02_000000;

// Simulation-specific results
pub const SIMULATION_UNCHANGED: Result = SUCCESS + 0;
pub const SIMULATION_CHANGED: Result = SUCCESS + 1;

pub const MAX_STEPS_REACHED: Result = SUCCESS + 1;

// General errors
pub const NULL_POINTER_ERROR: Result = GENERAL_ERROR + 0;
pub const ARGUMENT_OUT_OF_RANGE_ERROR: Result = GENERAL_ERROR + 1;

// Component-specific erros
pub const WIRE_WIDTH_MISMATCH_ERROR: Result = SPECIFIC_ERROR + 0;
pub const WIRE_WIDTH_INCOMPATIBLE_ERROR: Result = SPECIFIC_ERROR + 1;
pub const OFFSET_OUT_OF_RANGE_ERROR: Result = SPECIFIC_ERROR + 2;

// Simulation-specific errors
pub const SIMULATION_ERROR: Result = SPECIFIC_ERROR + 0;

fn boxed_slice_into_raw_parts<T>(b: Box<[T]>) -> (*mut T, usize) {
    let slice = Box::leak(b);
    let data = slice.as_mut_ptr();
    let len = slice.len();
    (data, len)
}

unsafe fn boxed_slice_from_raw_parts<T>(data: *mut T, len: usize) -> Box<[T]> {
    let slice = std::slice::from_raw_parts_mut(data, len);
    Box::from_raw(slice)
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_create(simulator: *mut *mut Simulator) -> Result {
    let Some(simulator) = simulator.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    let boxed_simulator = Box::new(Simulator::new());
    simulator.write(Box::into_raw(boxed_simulator));
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_free(simulator: *mut Simulator) -> Result {
    if simulator.is_null() {
        return NULL_POINTER_ERROR;
    }

    let boxed_simulator = Box::from_raw(simulator);
    std::mem::drop(boxed_simulator);
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_add_wire(
    simulator: *mut Simulator,
    width: u8,
    wire_id: *mut WireId,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    let Some(width) = LogicWidth::new(width) else {
        return ARGUMENT_OUT_OF_RANGE_ERROR;
    };

    let Some(wire_id) = wire_id.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    wire_id.write(simulator.add_wire(width));
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_get_wire_width(
    simulator: *mut Simulator,
    wire: WireId,
    width: *mut u8,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    let Some(width) = width.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    width.write(simulator.get_wire_width(wire).get());
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_set_wire_base_drive(
    simulator: *mut Simulator,
    wire: WireId,
    drive: LogicState,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    simulator.set_wire_base_drive(wire, drive);
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_get_wire_base_drive(
    simulator: *mut Simulator,
    wire: WireId,
    drive: *mut LogicState,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    let Some(drive) = drive.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    drive.write(simulator.get_wire_base_drive(wire));
    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_get_wire_state(
    simulator: *mut Simulator,
    wire: WireId,
    state: *mut LogicState,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    let Some(state) = state.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    state.write(simulator.get_wire_state(wire));
    SUCCESS
}

#[rustfmt::skip]
macro_rules! def_add_component {
    ($name:ident($($arg:ident: $arg_ty:ty),* $(,)?), $inner_name:ident) => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn $name(
            simulator: *mut Simulator,
            $($arg: $arg_ty,)*
            component_id: *mut ComponentId,
        ) -> Result {
            let Some(simulator) = simulator.as_mut() else {
                return NULL_POINTER_ERROR;
            };

            let Some(component_id) = component_id.as_uninit_mut() else {
                return NULL_POINTER_ERROR;
            };

            match simulator.$inner_name($($arg),*) {
                Ok(id) => {
                    component_id.write(id);
                    SUCCESS
                }
                Err(AddComponentError::WireWidthMismatch) => WIRE_WIDTH_MISMATCH_ERROR,
                Err(AddComponentError::WireWidthIncompatible) => WIRE_WIDTH_INCOMPATIBLE_ERROR,
                Err(AddComponentError::OffsetOutOfRange) => OFFSET_OUT_OF_RANGE_ERROR,
            }
        }
    };
}

def_add_component!(
    simulator_add_and_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_and_gate
);

def_add_component!(
    simulator_add_or_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_or_gate
);

def_add_component!(
    simulator_add_xor_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_xor_gate
);

def_add_component!(
    simulator_add_nand_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_nand_gate
);

def_add_component!(
    simulator_add_nor_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_nor_gate
);

def_add_component!(
    simulator_add_xnor_gate(input_a: WireId, input_b: WireId, output: WireId),
    add_xnor_gate
);

def_add_component!(
    simulator_add_not_gate(input: WireId, output: WireId),
    add_not_gate
);

def_add_component!(
    simulator_add_buffer(input: WireId, enable: WireId, output: WireId),
    add_buffer
);

def_add_component!(
    simulator_add_merge(input_a: WireId, input_b: WireId, output: WireId),
    add_merge
);

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_add_slice(
    simulator: *mut Simulator,
    input: WireId,
    offset: u8,
    output: WireId,
    component_id: *mut ComponentId,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    let Some(offset) = LogicOffset::new(offset) else {
        return ARGUMENT_OUT_OF_RANGE_ERROR;
    };

    let Some(component_id) = component_id.as_uninit_mut() else {
        return NULL_POINTER_ERROR;
    };

    match simulator.add_slice(input, offset, output) {
        Ok(id) => {
            component_id.write(id);
            SUCCESS
        }
        Err(AddComponentError::WireWidthMismatch) => WIRE_WIDTH_MISMATCH_ERROR,
        Err(AddComponentError::WireWidthIncompatible) => WIRE_WIDTH_INCOMPATIBLE_ERROR,
        Err(AddComponentError::OffsetOutOfRange) => OFFSET_OUT_OF_RANGE_ERROR,
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_reset(simulator: *mut Simulator) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    simulator.reset();
    SUCCESS
}

#[repr(C)]
pub struct SimulationErrors {
    conflicts: *mut WireId,
    conflicts_len: usize,
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulation_errors_free(errors: SimulationErrors) -> Result {
    let conflicts = boxed_slice_from_raw_parts(errors.conflicts, errors.conflicts_len);
    std::mem::drop(conflicts);

    SUCCESS
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_begin_sim(
    simulator: *mut Simulator,
    errors: *mut SimulationErrors,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    match simulator.begin_sim() {
        SimulationStepResult::Unchanged => SIMULATION_UNCHANGED,
        SimulationStepResult::Changed => SIMULATION_CHANGED,
        SimulationStepResult::Err(err) => {
            if let Some(errors) = errors.as_uninit_mut() {
                let (conflicts, conflicts_len) = boxed_slice_into_raw_parts(err.conflicts);

                errors.write(SimulationErrors {
                    conflicts,
                    conflicts_len,
                });
            }

            SIMULATION_ERROR
        }
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_step_sim(
    simulator: *mut Simulator,
    errors: *mut SimulationErrors,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    match simulator.step_sim() {
        SimulationStepResult::Unchanged => SIMULATION_UNCHANGED,
        SimulationStepResult::Changed => SIMULATION_CHANGED,
        SimulationStepResult::Err(err) => {
            if let Some(errors) = errors.as_uninit_mut() {
                let (conflicts, conflicts_len) = boxed_slice_into_raw_parts(err.conflicts);

                errors.write(SimulationErrors {
                    conflicts,
                    conflicts_len,
                });
            }

            SIMULATION_ERROR
        }
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_run_sim(
    simulator: *mut Simulator,
    max_steps: u64,
    errors: *mut SimulationErrors,
) -> Result {
    let Some(simulator) = simulator.as_mut() else {
        return NULL_POINTER_ERROR;
    };

    match simulator.run_sim(max_steps) {
        SimulationRunResult::Ok => SUCCESS,
        SimulationRunResult::MaxStepsReached => MAX_STEPS_REACHED,
        SimulationRunResult::Err(err) => {
            if let Some(errors) = errors.as_uninit_mut() {
                let (conflicts, conflicts_len) = boxed_slice_into_raw_parts(err.conflicts);

                errors.write(SimulationErrors {
                    conflicts,
                    conflicts_len,
                });
            }

            SIMULATION_ERROR
        }
    }
}
