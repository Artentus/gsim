use super::*;

ffi_fn! {
    /// Creates a new `Builder` object.  
    /// The resulting `Builder` must be freed by calling `simulator_build`, only if the operation succeeded.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_new(builder: *mut *mut SimulatorBuilder) {
        let builder_outer = check_ptr(builder)?;

        let builder_box = Box::<SimulatorBuilder>::default();
        let builder_inner = Box::into_raw(builder_box);
        builder_outer.as_ptr().write(builder_inner);

        Ok(ffi_status::SUCCESS)
    }
}

#[cfg(feature = "dot-export")]
ffi_fn! {
    /// Writes the simulation graph into a Graphviz DOT file.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_write_dot(
        builder: *const SimulatorBuilder,
        dot_file: *const c_char,
    ) {
        use std::fs::File;
        use std::io::BufWriter;

        let builder = cast_ptr(builder)?;
        let dot_file = BufWriter::new(File::create(cast_c_str(dot_file)?)?);

        builder.write_dot(dot_file)?;

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the width of a wire.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_get_wire_width(builder: *const SimulatorBuilder, wire: WireId, width: *mut u8) {
        let builder = cast_ptr(builder)?;
        let width_outer = check_ptr(width)?;

        let width_inner = builder.get_wire_width(wire)?;
        width_outer.as_ptr().write(width_inner.get());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Drives a wire to a certain state without needing a component.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_set_wire_drive(builder: *mut SimulatorBuilder, wire: WireId, drive: *const LogicState) {
        let builder = cast_mut_ptr(builder)?;
        let drive = cast_ptr(drive)?;
        builder.set_wire_drive(wire, drive)?;

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the current drive of a wire.  
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_get_wire_drive(builder: *const SimulatorBuilder, wire: WireId, drive: *mut *const LogicState) {
        let builder = cast_ptr(builder)?;
        let drive_outer = check_ptr(drive)?;

        let drive_box = Box::new(builder.get_wire_drive(wire)?);
        let drive_inner = Box::into_raw(drive_box).cast_const();
        drive_outer.as_ptr().write(drive_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the width of a register in the simulation.  
    /// The ID passed to `register` must refer to a register component.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_get_register_width(
        builder: *const SimulatorBuilder,
        register: ComponentId,
        width: *mut u8,
    ) {
        let builder = cast_ptr(builder)?;
        let width_outer = check_ptr(width)?;

        let data = builder.get_component_data(register)?;
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
    builder_read_register_state(
        builder: *const SimulatorBuilder,
        register: ComponentId,
        state: *mut *const LogicState,
    ) {
        let builder = cast_ptr(builder)?;
        let state_outer = check_ptr(state)?;

        let data = builder.get_component_data(register)?;
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
    builder_write_register_state(
        builder: *mut SimulatorBuilder,
        register: ComponentId,
        state: *const LogicState,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let state = cast_ptr(state)?;

        let data = builder.get_component_data_mut(register)?;
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
    builder_get_memory_metrics(
        builder: *const SimulatorBuilder,
        memory: ComponentId,
        size: *mut usize,
        width: *mut u8,
    ) {
        let builder = cast_ptr(builder)?;
        let size_outer = check_ptr(size)?;
        let width_outer = check_ptr(width)?;

        let data = builder.get_component_data(memory)?;
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
    builder_read_memory_state(
        builder: *const SimulatorBuilder,
        memory: ComponentId,
        addr: usize,
        state: *mut *const LogicState,
    ) {
        let builder = cast_ptr(builder)?;
        let state_outer = check_ptr(state)?;

        let data = builder.get_component_data(memory)?;
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
    builder_write_memory_state(
        builder: *mut SimulatorBuilder,
        memory: ComponentId,
        addr: usize,
        state: *const LogicState,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let state = cast_ptr(state)?;

        let data = builder.get_component_data_mut(memory)?;
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
    /// Sets the name of a wire.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_set_wire_name(
        builder: *mut SimulatorBuilder,
        wire: WireId,
        name: *const c_char,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let name = cast_c_str(name)?;
        builder.set_wire_name(wire, name)?;

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the name of a wire, if one has been assigned.  
    /// If no name has been assigned to the wire, name will be set to `null`.  
    /// The resulting string (if any) must be freed by calling `string_free`, only if the operation succeeded.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_get_wire_name(
        builder: *const SimulatorBuilder,
        wire: WireId,
        name: *mut *const c_char,
    ) {
        let builder = cast_ptr(builder)?;
        let name_outer = check_ptr(name)?;

        let name_inner = builder.get_wire_name(wire)?;
        let name_inner = name_inner
            .map(|s| CString::new(s).unwrap().into_raw().cast_const())
            .unwrap_or(std::ptr::null());
        name_outer.as_ptr().write(name_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Sets the name of a wire.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_set_component_name(
        builder: *mut SimulatorBuilder,
        component: ComponentId,
        name: *const c_char,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let name = cast_c_str(name)?;
        builder.set_component_name(component, name)?;

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the name of a component, if one has been assigned.  
    /// If no name has been assigned to the component, name will be set to `null`.  
    /// The resulting string (if any) must be freed by calling `string_free`, only if the operation succeeded.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    builder_get_component_name(
        builder: *const SimulatorBuilder,
        component: ComponentId,
        name: *mut *const c_char,
    ) {
        let builder = cast_ptr(builder)?;
        let name_outer = check_ptr(name)?;

        let name_inner = builder.get_component_name(component)?;
        let name_inner = name_inner
            .map(|s| CString::new(s).unwrap().into_raw().cast_const())
            .unwrap_or(std::ptr::null());
        name_outer.as_ptr().write(name_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_wire(builder: *mut SimulatorBuilder, width: u8, wire: *mut WireId) {
        let builder = cast_mut_ptr(builder)?;
        let width = width.try_into()?;
        let wire_outer = check_ptr(wire)?;
        let wire_inner = builder.add_wire(width).ok_or(FfiError::ResourceLimitReached)?;
        wire_outer.as_ptr().write(wire_inner);

        Ok(ffi_status::SUCCESS)
    }
}

macro_rules! impl_add_wide_gate {
    ($name:ident, $inner_name:ident) => {
        ffi_fn! {
            $name(
                builder: *mut SimulatorBuilder,
                inputs: *const WireId,
                input_len: usize,
                output: WireId,
                component: *mut ComponentId,
            ) {
                let builder = cast_mut_ptr(builder)?;
                let inputs = check_ptr(inputs.cast_mut())?;
                let inputs = std::slice::from_raw_parts(inputs.as_ptr().cast_const(), input_len);
                let component_outer = check_ptr(component)?;

                let component_inner = builder.$inner_name(inputs, output)?;
                component_outer.as_ptr().write(component_inner);

                Ok(ffi_status::SUCCESS)
            }
        }
    };
}

impl_add_wide_gate!(builder_add_and_gate, add_and_gate);
impl_add_wide_gate!(builder_add_or_gate, add_or_gate);
impl_add_wide_gate!(builder_add_xor_gate, add_xor_gate);
impl_add_wide_gate!(builder_add_nand_gate, add_nand_gate);
impl_add_wide_gate!(builder_add_nor_gate, add_nor_gate);
impl_add_wide_gate!(builder_add_xnor_gate, add_xnor_gate);
impl_add_wide_gate!(builder_add_merge, add_merge);
impl_add_wide_gate!(builder_add_priority_decoder, add_priority_decoder);

macro_rules! impl_add_binary_gate {
    ($name:ident, $inner_name:ident) => {
        ffi_fn! {
            $name(
                builder: *mut SimulatorBuilder,
                input_a: WireId,
                input_b: WireId,
                output: WireId,
                component: *mut ComponentId,
            ) {
                let builder = cast_mut_ptr(builder)?;
                let component_outer = check_ptr(component)?;

                let component_inner = builder.$inner_name(input_a, input_b, output)?;
                component_outer.as_ptr().write(component_inner);

                Ok(ffi_status::SUCCESS)
            }
        }
    };
}

impl_add_binary_gate!(builder_add_buffer, add_buffer);
impl_add_binary_gate!(builder_add_add, add_add);
impl_add_binary_gate!(builder_add_sub, add_sub);
impl_add_binary_gate!(builder_add_mul, add_mul);
impl_add_binary_gate!(builder_add_left_shift, add_left_shift);
impl_add_binary_gate!(builder_add_logical_right_shift, add_logical_right_shift);
impl_add_binary_gate!(
    builder_add_arithmetic_right_shift,
    add_arithmetic_right_shift
);
impl_add_binary_gate!(builder_add_compare_equal, add_compare_equal);
impl_add_binary_gate!(builder_add_compare_not_equal, add_compare_not_equal);
impl_add_binary_gate!(builder_add_compare_less_than, add_compare_less_than);
impl_add_binary_gate!(builder_add_compare_greater_than, add_compare_greater_than);
impl_add_binary_gate!(
    builder_add_compare_less_than_or_equal,
    add_compare_less_than_or_equal
);
impl_add_binary_gate!(
    builder_add_compare_greater_than_or_equal,
    add_compare_greater_than_or_equal
);
impl_add_binary_gate!(
    builder_add_compare_less_than_signed,
    add_compare_less_than_signed
);
impl_add_binary_gate!(
    builder_add_compare_greater_than_signed,
    add_compare_greater_than_signed
);
impl_add_binary_gate!(
    builder_add_compare_less_than_or_equal_signed,
    add_compare_less_than_or_equal_signed
);
impl_add_binary_gate!(
    builder_add_compare_greater_than_or_equal_signed,
    add_compare_greater_than_or_equal_signed
);

macro_rules! impl_add_unary_gate {
    ($name:ident, $inner_name:ident) => {
        ffi_fn! {
            $name(
                builder: *mut SimulatorBuilder,
                input: WireId,
                output: WireId,
                component: *mut ComponentId,
            ) {
                let builder = cast_mut_ptr(builder)?;
                let component_outer = check_ptr(component)?;

                let component_inner = builder.$inner_name(input, output)?;
                component_outer.as_ptr().write(component_inner);

                Ok(ffi_status::SUCCESS)
            }
        }
    };
}

impl_add_unary_gate!(builder_add_not_gate, add_not_gate);
impl_add_unary_gate!(builder_add_neg, add_neg);
impl_add_unary_gate!(builder_add_horizontal_and_gate, add_horizontal_and_gate);
impl_add_unary_gate!(builder_add_horizontal_or_gate, add_horizontal_or_gate);
impl_add_unary_gate!(builder_add_horizontal_xor_gate, add_horizontal_xor_gate);
impl_add_unary_gate!(builder_add_horizontal_nand_gate, add_horizontal_nand_gate);
impl_add_unary_gate!(builder_add_horizontal_nor_gate, add_horizontal_nor_gate);
impl_add_unary_gate!(builder_add_horizontal_xnor_gate, add_horizontal_xnor_gate);
impl_add_unary_gate!(builder_add_zero_extend, add_zero_extend);
impl_add_unary_gate!(builder_add_sign_extend, add_sign_extend);

ffi_fn! {
    builder_add_slice(
        builder: *mut SimulatorBuilder,
        input: WireId,
        offset: u8,
        output: WireId,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let component_outer = check_ptr(component)?;

        let component_inner = builder.add_slice(input, offset, output)?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_adder(
        builder: *mut SimulatorBuilder,
        input_a: WireId,
        input_b: WireId,
        carry_in: WireId,
        output: WireId,
        carry_out: WireId,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let component_outer = check_ptr(component)?;

        let component_inner = builder.add_adder(input_a, input_b, carry_in, output, carry_out)?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_multiplexer(
        builder: *mut SimulatorBuilder,
        inputs: *const WireId,
        input_len: usize,
        select: WireId,
        output: WireId,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let inputs = check_ptr(inputs.cast_mut())?;
        let inputs = std::slice::from_raw_parts(inputs.as_ptr().cast_const(), input_len);
        let component_outer = check_ptr(component)?;

        let component_inner = builder.add_multiplexer(inputs, select, output)?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_register(
        builder: *mut SimulatorBuilder,
        data_in: WireId,
        data_out: WireId,
        enable: WireId,
        clock: WireId,
        clock_polarity: u8,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let component_outer = check_ptr(component)?;

        let clock_polarity = match clock_polarity {
            0 => ClockPolarity::Falling,
            1 => ClockPolarity::Rising,
            _ => return Err(FfiError::InvalidArgument),
        };

        let component_inner = builder.add_register(
            data_in,
            data_out,
            enable,
            clock,
            clock_polarity,
        )?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_ram(
        builder: *mut SimulatorBuilder,
        write_addr: WireId,
        data_in: WireId,
        read_addr: WireId,
        data_out: WireId,
        write: WireId,
        clock: WireId,
        clock_polarity: u8,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let component_outer = check_ptr(component)?;

        let clock_polarity = match clock_polarity {
            0 => ClockPolarity::Falling,
            1 => ClockPolarity::Rising,
            _ => return Err(FfiError::InvalidArgument),
        };

        let component_inner = builder.add_ram(
            write_addr,
            data_in,
            read_addr,
            data_out,
            write,
            clock,
            clock_polarity,
        )?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    builder_add_rom(
        builder: *mut SimulatorBuilder,
        addr: WireId,
        data: WireId,
        component: *mut ComponentId,
    ) {
        let builder = cast_mut_ptr(builder)?;
        let component_outer = check_ptr(component)?;

        let component_inner = builder.add_rom(addr, data)?;
        component_outer.as_ptr().write(component_inner);

        Ok(ffi_status::SUCCESS)
    }
}

#[repr(C)]
pub struct PortList {
    len: usize,
    names: *const *const c_char,
    wires: *const WireId,
}

impl PortList {
    #[allow(dead_code)]
    fn create(map: HashMap<Arc<str>, WireId>) -> Self {
        let len = map.len();
        let mut names = Vec::with_capacity(len);
        let mut wires = Vec::with_capacity(len);
        for (name, wire) in map {
            let name = str::to_owned(&*name);
            let name = CString::new(name).unwrap().into_raw().cast_const();
            names.push(name);
            wires.push(wire);
        }

        let names = Box::into_raw(names.into_boxed_slice());
        let wires = Box::into_raw(wires.into_boxed_slice());

        let names: *const *const c_char = names as _;
        let wires: *const WireId = wires as _;

        Self { len, names, wires }
    }

    unsafe fn free(self) -> Result<(), FfiError> {
        let names = check_ptr(self.names.cast_mut())?.as_ptr().cast_const();
        let wires = check_ptr(self.wires.cast_mut())?.as_ptr().cast_const();

        let names = std::ptr::slice_from_raw_parts(names, self.len);
        let wires = std::ptr::slice_from_raw_parts(wires, self.len);

        let names = Box::from_raw(names.cast_mut());
        let wires = Box::from_raw(wires.cast_mut());

        for &name in names.iter() {
            let name = check_ptr(name.cast_mut())?;
            let name = CString::from_raw(name.as_ptr());
            std::mem::drop(name);
        }

        std::mem::drop(names);
        std::mem::drop(wires);

        Ok(())
    }
}

ffi_fn! {
    /// Frees all allocations of a `PortList` struct that was returned by other functions in the API.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    port_list_free(port_list: PortList) {
        port_list.free()?;
        Ok(ffi_status::SUCCESS)
    }
}

#[cfg(feature = "yosys-import")]
ffi_fn! {
    /// Imports a module defined by a Yosys netgraph into the circuit.  
    /// On success, `inputs` and `outputs` will contain a list of the imported modules ports.  
    /// The resulting `PortList` objects must be freed by calling `port_list_free`, only if the operation succeeded.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    /// 
    /// Valid netgraphs can be generated with this command:  
    /// `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>`
    builder_import_yosys_module(
        builder: *mut SimulatorBuilder,
        json_file: *const c_char,
        inputs: *mut PortList,
        outputs: *mut PortList,
    ) {
        use std::fs::File;
        use std::io::BufReader;
        use crate::import::yosys::YosysModuleImporter;

        let builder = cast_mut_ptr(builder)?;
        let inputs = check_ptr(inputs)?;
        let outputs = check_ptr(outputs)?;
        let json_file = BufReader::new(File::open(cast_c_str(json_file)?)?);

        let importer = YosysModuleImporter::from_json_reader(json_file)?;
        let connections = builder.import_module(&importer)?;

        inputs.as_ptr().write(PortList::create(connections.inputs));
        outputs.as_ptr().write(PortList::create(connections.outputs));

        Ok(ffi_status::SUCCESS)
    }
}
