use super::*;

#[cfg(feature = "tracing")]
type TracedSimulator = Simulator<std::io::BufWriter<std::fs::File>>;

/// An opaque type representing a simulation.  
/// Use only behind a pointer.
pub enum FfiSimulator {
    NoTrace(Simulator),
    #[cfg(feature = "tracing")]
    Trace(TracedSimulator),
}

ffi_fn! {
    /// Creates a new `Simulator` object from a `Builder`.
    /// If the operation succeeded, the specified `Builder` will be freed and be set to `null`.
    /// The resulting `Simulator` must be freed by calling `simulator_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_build(builder: *mut *mut SimulatorBuilder, simulator: *mut *mut FfiSimulator) {
        let builder_outer = check_ptr(builder)?;
        let builder_inner = check_ptr(*builder_outer.as_ptr())?;
        let simulator_outer = check_ptr(simulator)?;

        let builder_box = Box::from_raw(builder_inner.as_ptr());
        builder.write(std::ptr::null_mut());

        let simulator_box = Box::new(FfiSimulator::NoTrace(builder_box.build()));
        let simulator_inner = Box::into_raw(simulator_box);
        simulator_outer.as_ptr().write(simulator_inner);

        Ok(ffi_status::SUCCESS)
    }
}

#[cfg(feature = "tracing")]
ffi_fn! {
    /// Creates a new `Simulator` object from a `Builder`, with VCD tracing enabled.
    /// If the operation succeeded, the specified `Builder` will be freed and be set to `null`.
    /// The `Builder` may be freed even if the operation failed. In this case it will also be set to `null`.
    /// The resulting `Simulator` must be freed by calling `simulator_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_build_with_trace(
        builder: *mut *mut SimulatorBuilder,
        trace_file: *const c_char,
        simulator: *mut *const FfiSimulator,
    ) {
        use std::fs::File;
        use std::io::BufWriter;

        let builder_outer = check_ptr(builder)?;
        let builder_inner = check_ptr(*builder_outer.as_ptr())?;
        let simulator_outer = check_ptr(simulator)?;
        let trace_file = BufWriter::new(File::create(cast_c_str(trace_file)?)?);

        let builder_box = Box::from_raw(builder_inner.as_ptr());
        builder.write(std::ptr::null_mut());

        let tracing_simulator = builder_box.build_with_trace(trace_file, Timescale::default())?;
        let simulator_box = Box::new(FfiSimulator::Trace(tracing_simulator));
        let simulator_inner = Box::into_raw(simulator_box).cast_const();
        simulator_outer.as_ptr().write(simulator_inner);

        Ok(ffi_status::SUCCESS)
    }
}

#[cfg(feature = "dot-export")]
ffi_fn! {
    /// Writes the simulation graph into a Graphviz DOT file.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_write_dot(
        simulator: *const FfiSimulator,
        dot_file: *const c_char,
        show_states: u8,
    ) {
        use std::fs::File;
        use std::io::BufWriter;

        let simulator = cast_ptr(simulator)?;
        let dot_file = BufWriter::new(File::create(cast_c_str(dot_file)?)?);
        let show_states = show_states > 0;

        match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.write_dot(dot_file, show_states)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.write_dot(dot_file, show_states)?,
        }

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the width of a wire.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_wire_width(simulator: *const FfiSimulator, wire: WireId, width: *mut u8) {
        let simulator = cast_ptr(simulator)?;
        let width_outer = check_ptr(width)?;

        let width_inner = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_width(wire)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_width(wire)?,
        };
        width_outer.as_ptr().write(width_inner.get());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Drives a wire to a certain state without needing a component.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_set_wire_drive(simulator: *mut FfiSimulator, wire: WireId, drive: *const LogicState) {
        let simulator = cast_mut_ptr(simulator)?;
        let drive = cast_ptr(drive)?;
        match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.set_wire_drive(wire, drive)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.set_wire_drive(wire, drive)?,
        }

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the current drive of a wire.
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_wire_drive(simulator: *const FfiSimulator, wire: WireId, drive: *mut *const LogicState) {
        let simulator = cast_ptr(simulator)?;
        let drive_outer = check_ptr(drive)?;

        let drive_box = Box::new(match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_drive(wire)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_drive(wire)?,
        });
        let drive_inner = Box::into_raw(drive_box).cast_const();
        drive_outer.as_ptr().write(drive_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the current state of a wire.
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_wire_state(simulator: *const FfiSimulator, wire: WireId, state: *mut *const LogicState) {
        let simulator = cast_ptr(simulator)?;
        let state_outer = check_ptr(state)?;

        let state_box = Box::new(match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_state(wire)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_state(wire)?,
        });
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the width of a register in the simulation.
    /// The ID passed to `register` must refer to a register component.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_register_width(
        simulator: *const FfiSimulator,
        register: ComponentId,
        width: *mut u8,
    ) {
        let simulator = cast_ptr(simulator)?;
        let width_outer = check_ptr(width)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(register)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(register)?,
        };
        let ComponentData::RegisterValue(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        width_outer.as_ptr().write(data.width().get());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the current state of a register in the simulation.
    /// The ID passed to `register` must refer to a register component.
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_read_register_state(
        simulator: *const FfiSimulator,
        register: ComponentId,
        state: *mut *const LogicState,
    ) {
        let simulator = cast_ptr(simulator)?;
        let state_outer = check_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(register)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(register)?,
        };
        let ComponentData::RegisterValue(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        let state_box = Box::new(data.read());
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Sets the state of a register in the simulation.
    /// The ID passed to `register` must refer to a register component.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_write_register_state(
        simulator: *mut FfiSimulator,
        register: ComponentId,
        state: *const LogicState,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let state = cast_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data_mut(register)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data_mut(register)?,
        };
        let ComponentData::RegisterValue(mut data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        data.write(state);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the size and width of a memory block in the simulation.
    /// The ID passed to `memory` must refer to a memory component.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_memory_metrics(
        simulator: *const FfiSimulator,
        memory: ComponentId,
        size: *mut usize,
        width: *mut u8,
    ) {
        let simulator = cast_ptr(simulator)?;
        let size_outer = check_ptr(size)?;
        let width_outer = check_ptr(width)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(memory)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(memory)?,
        };
        let ComponentData::MemoryBlock(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        size_outer.as_ptr().write(data.len());
        width_outer.as_ptr().write(data.width().get());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the current state of a memory location in the simulation.
    /// The ID passed to `memory` must refer to a memory component.
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_read_memory_state(
        simulator: *const FfiSimulator,
        memory: ComponentId,
        addr: usize,
        state: *mut *const LogicState,
    ) {
        let simulator = cast_ptr(simulator)?;
        let state_outer = check_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(memory)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(memory)?,
        };
        let ComponentData::MemoryBlock(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        let state_box = Box::new(data.read(addr).ok_or(FfiError::ArgumentOutOfRange)?);
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Sets the state of a memory location in the simulation.
    /// The ID passed to `memory` must refer to a memory component.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_write_memory_state(
        simulator: *mut FfiSimulator,
        memory: ComponentId,
        addr: usize,
        state: *const LogicState,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let state = cast_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data_mut(memory)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data_mut(memory)?,
        };
        let ComponentData::MemoryBlock(mut data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        match data.write(addr, state) {
            Ok(_) => Ok(ffi_status::SUCCESS),
            Err(_) => Err(FfiError::ArgumentOutOfRange),
        }
    }
}

ffi_fn! {
    /// Gets the name of a wire, if one has been assigned.
    /// If no name has been assigned to the wire, name will be set to `null`.
    /// The resulting string (if any) must be freed by calling `string_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_wire_name(
        simulator: *mut FfiSimulator,
        wire: WireId,
        name: *mut *const c_char,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let name_outer = check_ptr(name)?;

        let name_inner = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_name(wire)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_name(wire)?,
        };
        let name_inner = name_inner
            .map(|s| CString::new(s).unwrap().into_raw().cast_const())
            .unwrap_or(std::ptr::null());
        name_outer.as_ptr().write(name_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the name of a component, if one has been assigned.
    /// If no name has been assigned to the component, name will be set to `null`.
    /// The resulting string (if any) must be freed by calling `string_free`, only if the operation succeeded.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_get_component_name(
        simulator: *mut FfiSimulator,
        component: ComponentId,
        name: *mut *const c_char,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let name_outer = check_ptr(name)?;

        let name_inner = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_name(component)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_name(component)?,
        };
        let name_inner = name_inner
            .map(|s| CString::new(s).unwrap().into_raw().cast_const())
            .unwrap_or(std::ptr::null());
        name_outer.as_ptr().write(name_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Resets the simulation.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_reset(simulator: *mut FfiSimulator) {
        let simulator = cast_mut_ptr(simulator)?;
        match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.reset(),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.reset(),
        }

        Ok(ffi_status::SUCCESS)
    }
}

#[repr(C)]
pub struct SimulationErrors {
    conflicts_len: usize,
    conflicts: *const WireId,
}

impl SimulationErrors {
    #[allow(dead_code)]
    fn create(conflicts: Box<[WireId]>) -> Self {
        let conflicts_len = conflicts.len();
        let conflicts: *const WireId = Box::into_raw(conflicts) as _;
        Self {
            conflicts_len,
            conflicts,
        }
    }

    unsafe fn free(self) -> Result<(), FfiError> {
        let conflicts = check_ptr(self.conflicts.cast_mut())?.as_ptr().cast_const();
        let conflicts = std::ptr::slice_from_raw_parts(conflicts, self.conflicts_len);
        let conflicts = Box::from_raw(conflicts.cast_mut());
        std::mem::drop(conflicts);

        Ok(())
    }
}

ffi_fn! {
    /// Frees all allocations of a `SimulationErrors` struct that was returned by other functions in the API.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulation_errors_free(error: SimulationErrors) {
        error.free()?;
        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Runs the simulation until it settles, but at most for `max_steps` steps.
    /// On success, returns one of the following values:
    /// - `GSIM_RESULT_SUCCESS`: the simulation settled within `max_steps` steps
    /// - `GSIM_RESULT_MAX_STEPS_REACHED`: the simulation did not settle within `max_steps` steps
    ///
    /// If a `Conflict` failure is reported, `errors` will contain additional information about which wires had a driver conflict.
    /// In this case, `errors` must later be freed by calling `simulation_errors_free`.
    simulator_run_sim(simulator: *mut FfiSimulator, max_steps: u64, errors: *mut SimulationErrors) {
        let simulator = cast_mut_ptr(simulator)?;
        let errors = check_ptr(errors)?;
        let result = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.run_sim(max_steps),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.run_sim(max_steps),
        };

        match result {
            SimulationRunResult::Ok => Ok(ffi_status::SUCCESS),
            SimulationRunResult::MaxStepsReached => Ok(ffi_status::MAX_STEPS_REACHED),
            SimulationRunResult::Err(err) => {
                errors.as_ptr().write(SimulationErrors::create(err.conflicts));
                Err(FfiError::Conflict)
            }
        }
    }
}

#[cfg(feature = "tracing")]
ffi_fn! {
    /// Writes the current state of the simulation into the simulators associated VCD file at the specified time in nanoseconds.
    /// Calling this function with a `Simulator` that was not constructed by `simulator_build_with_trace` is not illegal, but will have no effect.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_trace(simulator: *mut FfiSimulator, time: u64) {
        let simulator = cast_mut_ptr(simulator)?;
        match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.trace(time)?,
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.trace(time)?,
        }

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Frees a `Simulator` object.
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    simulator_free(simulator: *mut FfiSimulator) {
        let simulator = check_ptr(simulator)?;
        let simulator_box = Box::from_raw(simulator.as_ptr());
        std::mem::drop(simulator_box);

        Ok(ffi_status::SUCCESS)
    }
}
