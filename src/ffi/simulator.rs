use super::*;

#[cfg(feature = "tracing")]
type TracedSimulator = Simulator<std::io::BufWriter<std::fs::File>>;

enum FfiSimulator {
    NoTrace(Simulator),
    #[cfg(feature = "tracing")]
    Trace(TracedSimulator),
}

ffi_fn! {
    simulator_build(builder: *mut *mut SimulatorBuilder, simulator: *mut *const FfiSimulator) {
        let builder_outer = check_ptr(builder)?;
        let builder_inner = check_ptr(*builder_outer.as_ptr())?;
        let simulator_outer = check_ptr(simulator)?;

        let builder_box = Box::from_raw(builder_inner.as_ptr());
        builder.write(std::ptr::null_mut());

        let simulator_box = Box::new(FfiSimulator::NoTrace(builder_box.build()));
        let simulator_inner = Box::into_raw(simulator_box).cast_const();
        simulator_outer.as_ptr().write(simulator_inner);

        Ok(ffi_status::SUCCESS)
    }
}

#[cfg(feature = "tracing")]
ffi_fn! {
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
    simulator_get_wire_width(simulator: *const FfiSimulator, wire: WireId, width: *mut u8) {
        let simulator = cast_ptr(simulator)?;
        let width_outer = check_ptr(width)?;

        let width_inner = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_width(wire),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_width(wire),
        };
        width_outer.as_ptr().write(width_inner.get());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_set_wire_drive(simulator: *mut FfiSimulator, wire: WireId, drive: *const LogicState) {
        let simulator = cast_mut_ptr(simulator)?;
        let drive = cast_ptr(drive)?;
        match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.set_wire_drive(wire, drive),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.set_wire_drive(wire, drive),
        }

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_get_wire_drive(simulator: *const FfiSimulator, wire: WireId, drive: *mut *const LogicState) {
        let simulator = cast_ptr(simulator)?;
        let drive_outer = check_ptr(drive)?;

        let drive_box = Box::new(match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_drive(wire),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_drive(wire),
        });
        let drive_inner = Box::into_raw(drive_box).cast_const();
        drive_outer.as_ptr().write(drive_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_get_wire_state(simulator: *const FfiSimulator, wire: WireId, state: *mut *const LogicState) {
        let simulator = cast_ptr(simulator)?;
        let state_outer = check_ptr(state)?;

        let state_box = Box::new(match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_wire_state(wire),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_wire_state(wire),
        });
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_read_register_state(
        simulator: *const FfiSimulator,
        register: ComponentId,
        width: *mut u8,
        state: *mut *const LogicState,
    ) {
        let simulator = cast_ptr(simulator)?;
        let width_outer = check_ptr(width)?;
        let state_outer = check_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(register),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(register),
        };
        let ComponentData::RegisterValue(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        let state_box = Box::new(data.read());
        let state_inner = Box::into_raw(state_box).cast_const();
        width_outer.as_ptr().write(data.width().get());
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_write_register_state(
        simulator: *mut FfiSimulator,
        register: ComponentId,
        state: *const LogicState,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let state = cast_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data_mut(register),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data_mut(register),
        };
        let ComponentData::RegisterValue(mut data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        data.write(state);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_get_memory_size(
        simulator: *const FfiSimulator,
        memory: ComponentId,
        size: *mut usize,
    ) {
        let simulator = cast_ptr(simulator)?;
        let size_outer = check_ptr(size)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(memory),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(memory),
        };
        let ComponentData::MemoryBlock(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        size_outer.as_ptr().write(data.len());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_read_memory_state(
        simulator: *const FfiSimulator,
        memory: ComponentId,
        addr: usize,
        width: *mut u8,
        state: *mut *const LogicState,
    ) {
        let simulator = cast_ptr(simulator)?;
        let width_outer = check_ptr(width)?;
        let state_outer = check_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data(memory),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data(memory),
        };
        let ComponentData::MemoryBlock(data) = data else {
            return Err(FfiError::InvalidComponentType);
        };

        let state_box = Box::new(data.read(addr).ok_or(FfiError::ArgumentOutOfRange)?);
        let state_inner = Box::into_raw(state_box).cast_const();
        width_outer.as_ptr().write(data.width().get());
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    simulator_write_memory_state(
        simulator: *mut FfiSimulator,
        memory: ComponentId,
        addr: usize,
        state: *const LogicState,
    ) {
        let simulator = cast_mut_ptr(simulator)?;
        let state = cast_ptr(state)?;

        let data = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.get_component_data_mut(memory),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.get_component_data_mut(memory),
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

ffi_fn! {
    simulator_begin_sim(simulator: *mut FfiSimulator) {
        let simulator = cast_mut_ptr(simulator)?;
        let result = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.begin_sim(),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.begin_sim(),
        };

        match result {
            SimulationStepResult::Unchanged => Ok(ffi_status::UNCHANGED),
            SimulationStepResult::Changed => Ok(ffi_status::CHANGED),
            SimulationStepResult::Err(_) => Err(FfiError::Conflict),
        }
    }
}

ffi_fn! {
    simulator_step_sim(simulator: *mut FfiSimulator) {
        let simulator = cast_mut_ptr(simulator)?;
        let result = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.step_sim(),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.step_sim(),
        };

        match result {
            SimulationStepResult::Unchanged => Ok(ffi_status::UNCHANGED),
            SimulationStepResult::Changed => Ok(ffi_status::CHANGED),
            SimulationStepResult::Err(_) => Err(FfiError::Conflict),
        }
    }
}

ffi_fn! {
    simulator_run_sim(simulator: *mut FfiSimulator, max_steps: u64) {
        let simulator = cast_mut_ptr(simulator)?;
        let result = match simulator {
            FfiSimulator::NoTrace(simulator) => simulator.run_sim(max_steps),
            #[cfg(feature = "tracing")]
            FfiSimulator::Trace(simulator) => simulator.run_sim(max_steps),
        };

        match result {
            SimulationRunResult::Ok => Ok(ffi_status::SUCCESS),
            SimulationRunResult::MaxStepsReached => Ok(ffi_status::MAX_STEPS_REACHED),
            SimulationRunResult::Err(_) => Err(FfiError::Conflict),
        }
    }
}

ffi_fn! {
    simulator_free(simulator: *mut FfiSimulator) {
        let simulator = check_ptr(simulator)?;
        let simulator_box = Box::from_raw(simulator.as_ptr());
        std::mem::drop(simulator_box);

        Ok(ffi_status::SUCCESS)
    }
}
