#![allow(unsafe_op_in_unsafe_fn)]

use crate::*;
use pyo3::create_exception;
use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

create_exception!(
    gsim,
    SimulatorAlreadyBuiltError,
    PyException,
    "The simulator has already been built."
);

create_exception!(
    gsim,
    ResourceLimitReachedError,
    PyException,
    "Resource limit reached."
);

create_exception!(
    gsim,
    InvalidWireIdError,
    PyException,
    "Wire ID is not part of the simulation."
);

create_exception!(
    gsim,
    InvalidComponentIdError,
    PyException,
    "Component ID is not part of the simulation."
);

create_exception!(
    gsim,
    WireWidthMismatchError,
    PyException,
    "Wire widths didn't match."
);

create_exception!(
    gsim,
    WireWidthIncompatibleError,
    PyException,
    "Wire width is incompatible with the component."
);

create_exception!(
    gsim,
    OffsetOutOfRangeError,
    PyException,
    "Offset out of range."
);

create_exception!(
    gsim,
    TooFewInputsError,
    PyException,
    "Too few inputs for this component."
);

create_exception!(
    gsim,
    InvalidInputCountError,
    PyException,
    "Invalid input count for this component."
);

create_exception!(
    gsim,
    MaxStepsReachedError,
    PyException,
    "Reached maximum allowed simulation steps."
);

create_exception!(
    gsim,
    SimulationConflictError,
    PyException,
    "The simulation caused a conflict."
);

create_exception!(
    gsim,
    ComponentTypeError,
    PyException,
    "The component had the wrong type."
);

create_exception!(
    gsim,
    NetgraphFormatError,
    PyException,
    "Invalid netgraph format."
);

create_exception!(
    gsim,
    UnsupportedNetgraphError,
    PyException,
    "Netgraph is not supported."
);

impl From<AddComponentError> for PyErr {
    fn from(err: AddComponentError) -> Self {
        match err {
            AddComponentError::TooManyComponents => ResourceLimitReachedError::new_err(()),
            AddComponentError::InvalidWireId => InvalidWireIdError::new_err(()),
            AddComponentError::WireWidthMismatch => WireWidthMismatchError::new_err(()),
            AddComponentError::WireWidthIncompatible => WireWidthIncompatibleError::new_err(()),
            AddComponentError::OffsetOutOfRange => OffsetOutOfRangeError::new_err(()),
            AddComponentError::TooFewInputs => TooFewInputsError::new_err(()),
            AddComponentError::InvalidInputCount => InvalidInputCountError::new_err(()),
        }
    }
}

impl From<crate::InvalidWireIdError> for PyErr {
    fn from(_: crate::InvalidWireIdError) -> Self {
        InvalidWireIdError::new_err(())
    }
}

impl From<crate::InvalidComponentIdError> for PyErr {
    fn from(_: crate::InvalidComponentIdError) -> Self {
        InvalidComponentIdError::new_err(())
    }
}

macro_rules! def_py_id {
    ($name:ident($id:ident), $py_name:literal) => {
        #[pyclass(name = $py_name, frozen)]
        struct $name($id);

        #[pymethods]
        impl $name {
            #[staticmethod]
            fn invalid() -> Self {
                Self(<$id>::INVALID)
            }

            fn is_invalid(&self) -> bool {
                self.0.is_invalid()
            }

            fn __str__(&self) -> String {
                format!(concat!($py_name, "({})"), self.0.to_u32())
            }

            fn __repr__(&self) -> String {
                format!(concat!($py_name, "({})"), self.0.to_u32())
            }

            fn __hash__(&self) -> isize {
                self.0.to_u32() as isize
            }

            fn __richcmp__(&self, other: &$name, op: CompareOp) -> bool {
                op.matches(self.0.cmp(&other.0))
            }

            fn __bool__(&self) -> bool {
                !self.0.is_invalid()
            }
        }
    };
}

def_py_id!(PyWireId(WireId), "WireId");
def_py_id!(PyComponentId(ComponentId), "ComponentId");

#[pyclass(name = "LogicState", frozen)]
struct PyLogicState {
    width: NonZeroU8,
    state: LogicState,
}

#[pymethods]
impl PyLogicState {
    #[staticmethod]
    fn high_z(width: u8) -> PyResult<Self> {
        Ok(Self {
            width: NonZeroU8::try_from(width)?,
            state: LogicState::HIGH_Z,
        })
    }

    #[staticmethod]
    fn undefined(width: u8) -> PyResult<Self> {
        Ok(Self {
            width: NonZeroU8::try_from(width)?,
            state: LogicState::UNDEFINED,
        })
    }

    #[staticmethod]
    fn logic_0(width: u8) -> PyResult<Self> {
        Ok(Self {
            width: NonZeroU8::try_from(width)?,
            state: LogicState::LOGIC_0,
        })
    }

    #[staticmethod]
    fn logic_1(width: u8) -> PyResult<Self> {
        Ok(Self {
            width: NonZeroU8::try_from(width)?,
            state: LogicState::LOGIC_1,
        })
    }

    #[new]
    fn new(value: &PyAny, width: u8) -> PyResult<Self> {
        let width = NonZeroU8::try_from(width)?;

        if let Ok(value) = value.extract::<bool>() {
            Ok(Self {
                width,
                state: LogicState::from_bool(value),
            })
        } else if let Ok(value) = value.extract::<u32>() {
            Ok(Self {
                width,
                state: LogicState::from_int(value),
            })
        } else if let Ok(value) = value.extract::<&str>() {
            Ok(Self {
                width,
                state: LogicState::parse(value).ok_or_else(|| PyValueError::new_err(()))?,
            })
        } else if let Ok(value) = value.extract::<PyRef<PyLogicState>>() {
            Ok(Self {
                width,
                state: value.state.clone(),
            })
        } else {
            Err(PyTypeError::new_err(()))
        }
    }

    fn width(&self) -> u8 {
        self.width.get()
    }

    fn __str__(&self) -> String {
        self.state.display_string(self.width)
    }

    fn __repr__(&self) -> String {
        self.state.display_string(self.width)
    }

    fn __eq__(&self, other: &PyLogicState) -> bool {
        (self.width == other.width) && self.state.eq(&other.state, self.width)
    }

    fn __ne__(&self, other: &PyLogicState) -> bool {
        (self.width != other.width) || !self.state.eq(&other.state, self.width)
    }
}

#[pyclass(name = "ClockPolarity")]
enum PyClockPolarity {
    RISING = 1,
    FALLING = 0,
}

impl PyClockPolarity {
    fn to_clock_polarity(&self) -> ClockPolarity {
        match self {
            PyClockPolarity::RISING => ClockPolarity::Rising,
            PyClockPolarity::FALLING => ClockPolarity::Falling,
        }
    }
}

#[cfg(feature = "tracing")]
type TraceStream = std::io::BufWriter<std::fs::File>;

#[cfg(feature = "tracing")]
type TracedSimulator = Simulator<TraceStream>;

enum PySimulatorInner {
    NoTrace(Simulator),
    #[cfg(feature = "tracing")]
    Trace(TracedSimulator),
}

#[pyclass(name = "Simulator")]
struct PySimulator(PySimulatorInner);

macro_rules! with_simulator {
    ($sim_outer:expr, $sim_inner:ident => $body:expr) => {
        match &$sim_outer {
            PySimulatorInner::NoTrace($sim_inner) => $body,
            #[cfg(feature = "tracing")]
            PySimulatorInner::Trace($sim_inner) => $body,
        }
    };
    ($sim_outer:expr, mut $sim_inner:ident => $body:expr) => {
        match &mut $sim_outer {
            PySimulatorInner::NoTrace($sim_inner) => $body,
            #[cfg(feature = "tracing")]
            PySimulatorInner::Trace($sim_inner) => $body,
        }
    };
}

#[pymethods]
impl PySimulator {
    #[cfg(feature = "dot-export")]
    fn write_dot(&self, dot_file: &str, show_states: bool) -> PyResult<()> {
        with_simulator!(self.0, simulator => {
            use std::fs::File;
            use std::io::BufWriter;

            let dot_file = BufWriter::new(File::create(dot_file)?);
            simulator.write_dot(dot_file, show_states)?;

            Ok(())
        })
    }

    fn get_wire_width(&self, wire: &PyWireId) -> PyResult<u8> {
        with_simulator!(self.0, simulator => Ok(simulator.get_wire_width(wire.0)?.get()))
    }

    fn set_wire_drive(&mut self, wire: &PyWireId, new_drive: &PyLogicState) -> PyResult<()> {
        with_simulator!(self.0, mut simulator => Ok(simulator.set_wire_drive(wire.0, &new_drive.state)?))
    }

    fn get_wire_drive(&self, wire: &PyWireId) -> PyResult<PyLogicState> {
        with_simulator!(self.0, simulator => Ok(PyLogicState {
            width: simulator.get_wire_width(wire.0)?,
            state: simulator.get_wire_drive(wire.0)?,
        }))
    }

    fn get_wire_state(&self, wire: &PyWireId) -> PyResult<PyLogicState> {
        with_simulator!(self.0, simulator => Ok(PyLogicState {
            width: simulator.get_wire_width(wire.0)?,
            state: simulator.get_wire_state(wire.0)?,
        }))
    }

    fn read_register_state(&self, register: &PyComponentId) -> PyResult<PyLogicState> {
        with_simulator!(self.0, simulator => {
            let data = simulator.get_component_data(register.0)?;
            if let ComponentData::RegisterValue(data) = data {
                Ok(PyLogicState {
                    width: data.width(),
                    state: data.read(),
                })
            } else {
                Err(ComponentTypeError::new_err(()))
            }
        })
    }

    fn write_register_state(
        &mut self,
        register: &PyComponentId,
        state: &PyLogicState,
    ) -> PyResult<()> {
        with_simulator!(self.0, mut simulator => {
            let data = simulator.get_component_data_mut(register.0)?;
            if let ComponentData::RegisterValue(mut data) = data {
                data.write(&state.state);
                Ok(())
            } else {
                Err(ComponentTypeError::new_err(()))
            }
        })
    }

    fn get_memory_size(&self, register: &PyComponentId) -> PyResult<usize> {
        with_simulator!(self.0, simulator => {
            let data = simulator.get_component_data(register.0)?;
            if let ComponentData::MemoryBlock(data) = data {
                Ok(data.len())
            } else {
                Err(ComponentTypeError::new_err(()))
            }
        })
    }

    fn read_memory_state(&self, register: &PyComponentId, addr: usize) -> PyResult<PyLogicState> {
        with_simulator!(self.0, simulator => {
            let data = simulator.get_component_data(register.0)?;
            if let ComponentData::MemoryBlock(data) = data {
                Ok(PyLogicState {
                    width: data.width(),
                    state: data.read(addr).ok_or_else(|| PyIndexError::new_err(()))?,
                })
            } else {
                Err(ComponentTypeError::new_err(()))
            }
        })
    }

    fn write_memory_state(
        &mut self,
        register: &PyComponentId,
        addr: usize,
        state: &PyLogicState,
    ) -> PyResult<()> {
        with_simulator!(self.0, mut simulator => {
            let data = simulator.get_component_data_mut(register.0)?;
            if let ComponentData::MemoryBlock(mut data) = data {
                data.write(addr, &state.state).map_err(|_| PyIndexError::new_err(()))
            } else {
                Err(ComponentTypeError::new_err(()))
            }
        })
    }

    fn reset(&mut self) {
        with_simulator!(self.0, mut simulator => simulator.reset());
    }

    fn run_sim(&mut self, max_steps: u64) -> PyResult<()> {
        with_simulator!(self.0, mut simulator => match simulator.run_sim(max_steps) {
            SimulationRunResult::Ok => Ok(()),
            SimulationRunResult::MaxStepsReached => Err(MaxStepsReachedError::new_err(())),
            SimulationRunResult::Err(err) => {
                let conflicts: Vec<_> = err.conflicts.iter().copied().map(PyWireId).collect();
                Err(SimulationConflictError::new_err(conflicts))
            }
        })
    }

    #[cfg(feature = "tracing")]
    fn trace(&mut self, time: u64) -> PyResult<()> {
        with_simulator!(self.0, mut simulator => Ok(simulator.trace(time)?))
    }
}

#[allow(dead_code)]
type PyPortMap = std::collections::HashMap<String, PyWireId>;
#[allow(dead_code)]
type PyModuleConnections = (PyPortMap, PyPortMap);

#[allow(dead_code)]
fn convert_module_connections(
    connections: crate::import::ModuleConnections,
) -> PyModuleConnections {
    let inputs: PyPortMap = connections
        .inputs
        .into_iter()
        .map(|(name, wire)| (str::to_owned(&name), PyWireId(wire)))
        .collect();

    let outputs: PyPortMap = connections
        .outputs
        .into_iter()
        .map(|(name, wire)| (str::to_owned(&name), PyWireId(wire)))
        .collect();

    (inputs, outputs)
}

#[pyclass(name = "SimulatorBuilder")]
struct PySimulatorBuilder(Option<SimulatorBuilder>);

fn get_builder(builder: &PySimulatorBuilder) -> PyResult<&SimulatorBuilder> {
    match &builder.0 {
        Some(builder) => Ok(builder),
        None => Err(SimulatorAlreadyBuiltError::new_err(())),
    }
}

fn get_builder_mut(builder: &mut PySimulatorBuilder) -> PyResult<&mut SimulatorBuilder> {
    match &mut builder.0 {
        Some(builder) => Ok(builder),
        None => Err(SimulatorAlreadyBuiltError::new_err(())),
    }
}

#[pymethods]
impl PySimulatorBuilder {
    #[new]
    fn new() -> Self {
        Self(Some(SimulatorBuilder::default()))
    }

    #[cfg(feature = "dot-export")]
    fn write_dot(&self, dot_file: &str) -> PyResult<()> {
        use std::fs::File;
        use std::io::BufWriter;

        let builder = get_builder(self)?;
        let dot_file = BufWriter::new(File::create(dot_file)?);
        builder.write_dot(dot_file)?;

        Ok(())
    }

    fn get_wire_width(&self, wire: &PyWireId) -> PyResult<u8> {
        let builder = get_builder(self)?;
        Ok(builder.get_wire_width(wire.0)?.get())
    }

    fn set_wire_drive(&mut self, wire: &PyWireId, new_drive: &PyLogicState) -> PyResult<()> {
        let builder = get_builder_mut(self)?;
        builder.set_wire_drive(wire.0, &new_drive.state)?;
        Ok(())
    }

    fn get_wire_drive(&self, wire: &PyWireId) -> PyResult<PyLogicState> {
        let builder = get_builder(self)?;
        let width = builder.get_wire_width(wire.0)?;
        let state = builder.get_wire_drive(wire.0)?;
        Ok(PyLogicState { width, state })
    }

    fn read_register_state(&self, register: &PyComponentId) -> PyResult<PyLogicState> {
        let builder = get_builder(self)?;
        let data = builder.get_component_data(register.0)?;
        if let ComponentData::RegisterValue(data) = data {
            Ok(PyLogicState {
                width: data.width(),
                state: data.read(),
            })
        } else {
            Err(ComponentTypeError::new_err(()))
        }
    }

    fn write_register_state(
        &mut self,
        register: &PyComponentId,
        state: &PyLogicState,
    ) -> PyResult<()> {
        let builder = get_builder_mut(self)?;
        let data = builder.get_component_data_mut(register.0)?;
        if let ComponentData::RegisterValue(mut data) = data {
            data.write(&state.state);
            Ok(())
        } else {
            Err(ComponentTypeError::new_err(()))
        }
    }

    fn get_memory_size(&self, register: &PyComponentId) -> PyResult<usize> {
        let builder = get_builder(self)?;
        let data = builder.get_component_data(register.0)?;
        if let ComponentData::MemoryBlock(data) = data {
            Ok(data.len())
        } else {
            Err(ComponentTypeError::new_err(()))
        }
    }

    fn read_memory_state(&self, register: &PyComponentId, addr: usize) -> PyResult<PyLogicState> {
        let builder = get_builder(self)?;
        let data = builder.get_component_data(register.0)?;
        if let ComponentData::MemoryBlock(data) = data {
            Ok(PyLogicState {
                width: data.width(),
                state: data.read(addr).ok_or_else(|| PyIndexError::new_err(()))?,
            })
        } else {
            Err(ComponentTypeError::new_err(()))
        }
    }

    fn write_memory_state(
        &mut self,
        register: &PyComponentId,
        addr: usize,
        state: &PyLogicState,
    ) -> PyResult<()> {
        let builder = get_builder_mut(self)?;
        let data = builder.get_component_data_mut(register.0)?;
        if let ComponentData::MemoryBlock(mut data) = data {
            data.write(addr, &state.state)
                .map_err(|_| PyIndexError::new_err(()))
        } else {
            Err(ComponentTypeError::new_err(()))
        }
    }

    fn set_wire_name(&mut self, wire: &PyWireId, name: &str) -> PyResult<()> {
        let builder = get_builder_mut(self)?;
        builder.set_wire_name(wire.0, name)?;
        Ok(())
    }

    fn set_component_name(&mut self, component: &PyComponentId, name: &str) -> PyResult<()> {
        let builder = get_builder_mut(self)?;
        builder.set_component_name(component.0, name)?;
        Ok(())
    }

    fn add_wire(&mut self, width: u8) -> PyResult<PyWireId> {
        let builder = get_builder_mut(self)?;
        let width = NonZeroU8::try_from(width)?;
        let id = builder
            .add_wire(width)
            .ok_or_else(|| ResourceLimitReachedError::new_err(()))?;

        Ok(PyWireId(id))
    }

    fn add_slice(
        &mut self,
        input: &PyWireId,
        offset: u8,
        output: &PyWireId,
    ) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let id = builder.add_slice(input.0, offset, output.0)?;
        Ok(PyComponentId(id))
    }

    fn add_adder(
        &mut self,
        input_a: &PyWireId,
        input_b: &PyWireId,
        carry_in: &PyWireId,
        output: &PyWireId,
        carry_out: &PyWireId,
    ) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let id = builder.add_adder(input_a.0, input_b.0, carry_in.0, output.0, carry_out.0)?;
        Ok(PyComponentId(id))
    }

    fn add_multiplexer(
        &mut self,
        inputs: &PyAny,
        select: &PyWireId,
        output: &PyWireId,
    ) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let inputs: Vec<_> = inputs
            .iter()?
            .map(|input| {
                input
                    .and_then(PyAny::extract::<PyRef<PyWireId>>)
                    .map(|id| id.0)
            })
            .collect::<PyResult<_>>()?;

        let id = builder.add_multiplexer(&inputs, select.0, output.0)?;
        Ok(PyComponentId(id))
    }

    fn add_register(
        &mut self,
        data_in: &PyWireId,
        data_out: &PyWireId,
        enable: &PyWireId,
        clock: &PyWireId,
        clock_polarity: Option<&PyClockPolarity>,
    ) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let clock_polarity = clock_polarity
            .map(PyClockPolarity::to_clock_polarity)
            .unwrap_or_default();
        let id = builder.add_register(data_in.0, data_out.0, enable.0, clock.0, clock_polarity)?;
        Ok(PyComponentId(id))
    }

    fn add_ram(
        &mut self,
        write_addr: &PyWireId,
        data_in: &PyWireId,
        read_addr: &PyWireId,
        data_out: &PyWireId,
        write: &PyWireId,
        clock: &PyWireId,
        clock_polarity: Option<&PyClockPolarity>,
    ) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let clock_polarity = clock_polarity
            .map(PyClockPolarity::to_clock_polarity)
            .unwrap_or_default();
        let id = builder.add_ram(
            write_addr.0,
            data_in.0,
            read_addr.0,
            data_out.0,
            write.0,
            clock.0,
            clock_polarity,
        )?;
        Ok(PyComponentId(id))
    }

    fn add_rom(&mut self, addr: &PyWireId, data: &PyWireId) -> PyResult<PyComponentId> {
        let builder = get_builder_mut(self)?;
        let id = builder.add_rom(addr.0, data.0)?;
        Ok(PyComponentId(id))
    }

    #[cfg(feature = "yosys-import")]
    fn import_yosys_module(&mut self, json_file: &str) -> PyResult<PyModuleConnections> {
        use crate::import::yosys::{YosysModuleImportError, YosysModuleImporter};
        use serde_json::error::Category;
        use std::fs::File;
        use std::io::BufReader;

        let builder = get_builder_mut(self)?;
        let json_file = BufReader::new(File::open(json_file)?);

        let importer = YosysModuleImporter::from_json_reader(json_file).map_err(|err| match err
            .classify()
        {
            Category::Io => PyIOError::new_err(()),
            Category::Syntax | Category::Data | Category::Eof => NetgraphFormatError::new_err(()),
        })?;

        let connections = builder.import_module(&importer).map_err(|err| match err {
            YosysModuleImportError::ResourceLimitReached => ResourceLimitReachedError::new_err(()),
            YosysModuleImportError::InOutPort { .. }
            | YosysModuleImportError::CellInOutPort { .. }
            | YosysModuleImportError::UnsupportedWireWidth { .. }
            | YosysModuleImportError::UnknownCellType { .. }
            | YosysModuleImportError::UnsupportedCellType { .. } => {
                UnsupportedNetgraphError::new_err(())
            }
            YosysModuleImportError::MissingCellPortDirection { .. }
            | YosysModuleImportError::InvalidCellPorts { .. }
            | YosysModuleImportError::InvalidCellParameters { .. } => {
                NetgraphFormatError::new_err(())
            }
        })?;

        Ok(convert_module_connections(connections))
    }

    fn build(&mut self) -> PyResult<PySimulator> {
        match self.0.take() {
            Some(builder) => {
                let simulator = builder.build();
                Ok(PySimulator(PySimulatorInner::NoTrace(simulator)))
            }
            None => Err(SimulatorAlreadyBuiltError::new_err(())),
        }
    }

    #[cfg(feature = "tracing")]
    fn build_with_trace(&mut self, vcd_file: &str) -> PyResult<PySimulator> {
        use std::fs::File;
        use std::io::BufWriter;

        match self.0.take() {
            Some(builder) => {
                let vcd_file = BufWriter::new(File::create(vcd_file)?);
                let simulator = builder.build_with_trace(vcd_file, Timescale::default())?;
                Ok(PySimulator(PySimulatorInner::Trace(simulator)))
            }
            None => Err(SimulatorAlreadyBuiltError::new_err(())),
        }
    }
}

macro_rules! impl_add_wide_gate {
    ($name:ident) => {
        #[pymethods]
        impl PySimulatorBuilder {
            fn $name(&mut self, inputs: &PyAny, output: &PyWireId) -> PyResult<PyComponentId> {
                let builder = get_builder_mut(self)?;
                let inputs: Vec<_> = inputs
                    .iter()?
                    .map(|input| {
                        input
                            .and_then(PyAny::extract::<PyRef<PyWireId>>)
                            .map(|id| id.0)
                    })
                    .collect::<PyResult<_>>()?;

                let id = builder.$name(&inputs, output.0)?;
                Ok(PyComponentId(id))
            }
        }
    };
}

impl_add_wide_gate!(add_and_gate);
impl_add_wide_gate!(add_or_gate);
impl_add_wide_gate!(add_xor_gate);
impl_add_wide_gate!(add_nand_gate);
impl_add_wide_gate!(add_nor_gate);
impl_add_wide_gate!(add_xnor_gate);
impl_add_wide_gate!(add_merge);
impl_add_wide_gate!(add_priority_decoder);

macro_rules! impl_add_binary_gate {
    ($name:ident) => {
        #[pymethods]
        impl PySimulatorBuilder {
            fn $name(
                &mut self,
                input_a: &PyWireId,
                input_b: &PyWireId,
                output: &PyWireId,
            ) -> PyResult<PyComponentId> {
                let builder = get_builder_mut(self)?;
                let id = builder.$name(input_a.0, input_b.0, output.0)?;
                Ok(PyComponentId(id))
            }
        }
    };
}

impl_add_binary_gate!(add_buffer);
impl_add_binary_gate!(add_add);
impl_add_binary_gate!(add_sub);
impl_add_binary_gate!(add_left_shift);
impl_add_binary_gate!(add_logical_right_shift);
impl_add_binary_gate!(add_arithmetic_right_shift);
impl_add_binary_gate!(add_compare_equal);
impl_add_binary_gate!(add_compare_not_equal);
impl_add_binary_gate!(add_compare_less_than);
impl_add_binary_gate!(add_compare_greater_than);
impl_add_binary_gate!(add_compare_less_than_or_equal);
impl_add_binary_gate!(add_compare_greater_than_or_equal);
impl_add_binary_gate!(add_compare_less_than_signed);
impl_add_binary_gate!(add_compare_greater_than_signed);
impl_add_binary_gate!(add_compare_less_than_or_equal_signed);
impl_add_binary_gate!(add_compare_greater_than_or_equal_signed);

macro_rules! impl_add_unary_gate {
    ($name:ident) => {
        #[pymethods]
        impl PySimulatorBuilder {
            fn $name(&mut self, input: &PyWireId, output: &PyWireId) -> PyResult<PyComponentId> {
                let builder = get_builder_mut(self)?;
                let id = builder.$name(input.0, output.0)?;
                Ok(PyComponentId(id))
            }
        }
    };
}

impl_add_unary_gate!(add_not_gate);
impl_add_unary_gate!(add_horizontal_and_gate);
impl_add_unary_gate!(add_horizontal_or_gate);
impl_add_unary_gate!(add_horizontal_xor_gate);
impl_add_unary_gate!(add_horizontal_nand_gate);
impl_add_unary_gate!(add_horizontal_nor_gate);
impl_add_unary_gate!(add_horizontal_xnor_gate);
impl_add_unary_gate!(add_zero_extend);
impl_add_unary_gate!(add_sign_extend);

#[pymodule]
fn gsim(py: Python, m: &PyModule) -> PyResult<()> {
    macro_rules! add_error {
        ($err:ty) => {
            m.add(stringify!($err), py.get_type::<$err>())?;
        };
    }

    add_error!(SimulatorAlreadyBuiltError);
    add_error!(ResourceLimitReachedError);
    add_error!(InvalidWireIdError);
    add_error!(InvalidComponentIdError);
    add_error!(WireWidthMismatchError);
    add_error!(WireWidthIncompatibleError);
    add_error!(OffsetOutOfRangeError);
    add_error!(TooFewInputsError);
    add_error!(InvalidInputCountError);
    add_error!(MaxStepsReachedError);
    add_error!(SimulationConflictError);
    add_error!(ComponentTypeError);
    add_error!(NetgraphFormatError);
    add_error!(UnsupportedNetgraphError);

    m.add_class::<PyWireId>()?;
    m.add_class::<PyComponentId>()?;
    m.add_class::<PyLogicState>()?;
    m.add_class::<PyClockPolarity>()?;
    m.add_class::<PySimulator>()?;
    m.add_class::<PySimulatorBuilder>()?;

    Ok(())
}
