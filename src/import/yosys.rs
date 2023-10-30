//! Import circuits from Yosys JSON format
//!
//! Use the following command to generate compatible JSON files:</br>
//! `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`

use super::*;
use crate::*;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::num::NonZeroU8;
use std::rc::Rc;

trait EnsureLen {
    fn ensure_len(&mut self, len: usize);
}

impl<T: Default> EnsureLen for Vec<T> {
    fn ensure_len(&mut self, len: usize) {
        if self.len() < len {
            self.resize_with(len, || T::default());
        }
    }
}

/// The known Yosys cell types
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum CellType {
    Not,
    Pos,
    Neg,
    ReduceAnd,
    ReduceOr,
    ReduceXor,
    ReduceXnor,
    ReduceBool,
    LogicNot,
    And,
    Or,
    Xor,
    Xnor,
    Shl,
    Sshl,
    Shr,
    Sshr,
    LogicAnd,
    LogicOr,
    EqX,
    NeX,
    Pow,
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    DivFloor,
    ModFloor,
    Mux,
    Pmux,
    TriBuf,
    Sr,
    Dff,
    Dffe,
    Sdff,
    Sdffe,
    Sdffce,
    Dlatch,
    MemRdV2,
    MemWrV2,
    MemInitV2,
    MemV2,
    Unknown(Rc<str>),
}

impl From<String> for CellType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "$not" => Self::Not,
            "$pos" => Self::Pos,
            "$neg" => Self::Neg,
            "$reduce_and" => Self::ReduceAnd,
            "$reduce_or" => Self::ReduceOr,
            "$reduce_xor" => Self::ReduceXor,
            "$reduce_xnor" => Self::ReduceXnor,
            "$reduce_bool" => Self::ReduceBool,
            "$logic_not" => Self::LogicNot,
            "$and" => Self::And,
            "$or" => Self::Or,
            "$xor" => Self::Xor,
            "$xnor" => Self::Xnor,
            "$shl" => Self::Shl,
            "$sshl" => Self::Sshl,
            "$shr" => Self::Shr,
            "$sshr" => Self::Sshr,
            "$logic_and" => Self::LogicAnd,
            "$logic_or" => Self::LogicOr,
            "$eqx" => Self::EqX,
            "$nex" => Self::NeX,
            "$pow" => Self::Pow,
            "$lt" => Self::Lt,
            "$le" => Self::Le,
            "$eq" => Self::Eq,
            "$ne" => Self::Ne,
            "$ge" => Self::Ge,
            "$gt" => Self::Gt,
            "$add" => Self::Add,
            "$sub" => Self::Sub,
            "$mul" => Self::Mul,
            "$div" => Self::Div,
            "$mod" => Self::Mod,
            "$divfloor" => Self::DivFloor,
            "$modfloor" => Self::ModFloor,
            "$mux" => Self::Mux,
            "$pmux" => Self::Pmux,
            "$tribuf" => Self::TriBuf,
            "$sr" => Self::Sr,
            "$dff" => Self::Dff,
            "$dffe" => Self::Dffe,
            "$sdff" => Self::Sdff,
            "$sdffe" => Self::Sdffe,
            "$sdffce" => Self::Sdffce,
            "$dlatch" => Self::Dlatch,
            "$memrd_v2" => Self::MemRdV2,
            "$memwr_v2" => Self::MemWrV2,
            "$meminit_v2" => Self::MemInitV2,
            "$mem_v2" => Self::MemV2,
            _ => Self::Unknown(value.into()),
        }
    }
}

fn single_from_map<'de, D, T>(deserializer: D) -> Result<(&'de str, T), D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::de::Error;

    let map = HashMap::<&'de str, T>::deserialize(deserializer)?;
    if map.len() == 1 {
        Ok(map.into_iter().next().unwrap())
    } else {
        Err(Error::invalid_length(
            map.len(),
            &"object with exactly one key",
        ))
    }
}

fn cell_type<'de, D>(deserializer: D) -> Result<CellType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    Ok(name.into())
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
enum PortDirection {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
    #[serde(rename = "inout")]
    InOut,
}

type NetId = usize;

#[derive(Clone, Copy, Deserialize)]
#[serde(untagged)]
enum Signal {
    Value(LogicBitState),
    Net(NetId),
}

/// LSB first
type Bits = Vec<Signal>;

#[derive(Deserialize)]
struct Port {
    direction: PortDirection,
    bits: Bits,
}

#[derive(Deserialize)]
struct Cell<'a> {
    #[serde(rename = "type", deserialize_with = "cell_type")]
    cell_type: CellType,
    #[serde(default, borrow)]
    parameters: HashMap<&'a str, &'a str>,
    #[serde(borrow)]
    port_directions: HashMap<&'a str, PortDirection>,
    #[serde(borrow)]
    connections: IndexMap<&'a str, Bits>,
}

#[derive(Deserialize)]
struct Module<'a> {
    #[serde(borrow)]
    ports: IndexMap<&'a str, Port>,
    #[serde(default, borrow)]
    cells: IndexMap<&'a str, Cell<'a>>,
}

#[derive(Deserialize)]
struct Netlist<'a> {
    #[serde(borrow, rename = "modules", deserialize_with = "single_from_map")]
    module: (&'a str, Module<'a>),
}

struct PreprocCellPort {
    bits: Bits,
    direction: Option<PortDirection>,
    signed: Option<bool>,
    polarity: Option<ClockPolarity>,
}

struct PreprocCell {
    cell_type: CellType,
    ports: IndexMap<Rc<str>, PreprocCellPort>,
    parameters: HashMap<Rc<str>, LogicState>,
}

impl PreprocCell {
    fn create(mut cell: Cell<'_>) -> Self {
        let ports = cell
            .connections
            .into_iter()
            .map(|(name, bits)| {
                let preproc_cell = PreprocCellPort {
                    bits,
                    direction: cell.port_directions.remove(&name),
                    signed: cell
                        .parameters
                        .remove(format!("{name}_SIGNED").as_str())
                        .map(|v| u32::from_str_radix(v, 2).unwrap() > 0),
                    polarity: cell
                        .parameters
                        .remove(format!("{name}_POLARITY").as_str())
                        .map(|v| match u32::from_str_radix(v, 2).unwrap() {
                            0 => ClockPolarity::Falling,
                            _ => ClockPolarity::Rising,
                        }),
                };

                (name.into(), preproc_cell)
            })
            .collect();

        let parameters = cell
            .parameters
            .into_iter()
            .map(|(k, v)| (k.into(), LogicState::parse(v).unwrap()))
            .collect();

        Self {
            cell_type: cell.cell_type,
            ports,
            parameters,
        }
    }
}

struct PreprocModule {
    ports: IndexMap<Rc<str>, Port>,
    cells: IndexMap<Rc<str>, PreprocCell>,
}

impl PreprocModule {
    fn create(module: Module<'_>) -> Self {
        Self {
            ports: module
                .ports
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
            cells: module
                .cells
                .into_iter()
                .map(|(name, cell)| (name.into(), PreprocCell::create(cell)))
                .collect(),
        }
    }
}

/// Imports circuits generated by Yosys
///
/// Use the following command to generate compatible JSON files:</br>
/// `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`
pub struct YosysModuleImporter {
    module_name: Box<str>,
    module: PreprocModule,
}

impl YosysModuleImporter {
    fn preprocess(netlist: Netlist) -> Self {
        Self {
            module_name: netlist.module.0.into(),
            module: PreprocModule::create(netlist.module.1),
        }
    }

    /// Creates a Yosys module importer from a slice containing JSON data
    pub fn from_json_slice(slice: &[u8]) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_slice(slice)?;
        Ok(Self::preprocess(netlist))
    }

    /// Creates a Yosys module importer from a string containing JSON data
    pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_str(s)?;
        Ok(Self::preprocess(netlist))
    }
}

/// An error that can occure while importing a Yosys module
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum YosysModuleImportError {
    /// The simulators resource limit was reached while constructing the module
    ResourceLimitReached,
    /// The module has an `inout` port
    InOutPort {
        /// The name of the `inout` port
        port_name: Rc<str>,
    },
    /// The module contains a cell that has an `inout` port
    CellInOutPort {
        /// The name of the cell
        cell_name: Rc<str>,
        /// The name of the `inout` port
        port_name: Rc<str>,
    },
    /// The module or one of its cells has a port that is wider than `MAX_LOGIC_WIDTH`
    UnsupportedWireWidth {
        /// The width of the wire
        wire_width: usize,
    },
    /// The module contains a cell of unknown type
    UnknownCellType {
        /// The name of the cell
        cell_name: Rc<str>,
        /// The unknown type of the cell
        cell_type: Rc<str>,
    },
    /// The module contains a cell that is not supported for importing
    UnsupportedCellType {
        /// The name of the cell
        cell_name: Rc<str>,
        /// The type of the cell
        cell_type: CellType,
    },
    /// The module contains a cell that has a port with no specified direction
    MissingCellPortDirection {
        /// The name of the cell
        cell_name: Rc<str>,
        /// The name of the port
        port_name: Rc<str>,
    },
    /// The module contains a cell with an invalid port configuration
    InvalidCellPorts {
        /// The name of the cell
        cell_name: Rc<str>,
    },
    /// The module contains a cell with an invalid parameter configuration
    InvalidCellParameters {
        /// The name of the cell
        cell_name: Rc<str>,
    },
}

impl ModuleImporter for YosysModuleImporter {
    type Error = YosysModuleImportError;

    #[inline]
    fn module_name(&self) -> &str {
        &self.module_name
    }

    fn import_into(
        &self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<ModuleConnections, Self::Error> {
        // FIXME: merge nets to generate a more efficient graph

        let const_high_z = builder
            .add_wire(NonZeroU8::MIN)
            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
        builder.set_wire_drive(const_high_z, &LogicState::HIGH_Z);

        let const_undefined = builder
            .add_wire(NonZeroU8::MIN)
            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
        builder.set_wire_drive(const_undefined, &LogicState::UNDEFINED);

        let const_0 = builder
            .add_wire(NonZeroU8::MIN)
            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
        builder.set_wire_drive(const_0, &LogicState::LOGIC_0);

        let const_1 = builder
            .add_wire(NonZeroU8::MIN)
            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
        builder.set_wire_drive(const_1, &LogicState::LOGIC_1);

        fn get_wire_for_net_id(
            id: NetId,
            builder: &mut crate::SimulatorBuilder,
            net_map: &mut Vec<WireId>,
        ) -> Result<WireId, YosysModuleImportError> {
            net_map.ensure_len(id + 1);
            if net_map[id].is_invalid() {
                net_map[id] = builder
                    .add_wire(NonZeroU8::MIN)
                    .ok_or(YosysModuleImportError::ResourceLimitReached)?;
            }

            Ok(net_map[id])
        }

        let get_wire_for_bit = move |bit: Signal,
                                     builder: &mut crate::SimulatorBuilder,
                                     net_map: &mut Vec<WireId>|
              -> Result<WireId, YosysModuleImportError> {
            match bit {
                Signal::Value(LogicBitState::HighZ) => Ok(const_high_z),
                Signal::Value(LogicBitState::Undefined) => Ok(const_undefined),
                Signal::Value(LogicBitState::Logic0) => Ok(const_0),
                Signal::Value(LogicBitState::Logic1) => Ok(const_1),
                Signal::Net(id) => get_wire_for_net_id(id, builder, net_map),
            }
        };

        let mut net_map: Vec<WireId> = Vec::new();
        let mut connections = ModuleConnections::default();

        for (port_name, port) in &self.module.ports {
            if let &[bit] = port.bits.as_slice() {
                match port.direction {
                    PortDirection::Input => {
                        if let Signal::Net(id) = bit {
                            let wire = get_wire_for_net_id(id, builder, &mut net_map)?;
                            connections.inputs.insert(Rc::clone(port_name), wire);
                        }
                    }
                    PortDirection::Output => {
                        let wire = get_wire_for_bit(bit, builder, &mut net_map)?;
                        connections.outputs.insert(Rc::clone(port_name), wire);
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::InOutPort {
                            port_name: Rc::clone(port_name),
                        })
                    }
                }
            } else {
                let wire_width = port.bits.len();
                let wire_width: NonZeroU8 = u8::try_from(wire_width)
                    .and_then(NonZeroU8::try_from)
                    .map_err(|_| YosysModuleImportError::UnsupportedWireWidth { wire_width })?;
                let port_wire = builder
                    .add_wire(wire_width)
                    .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                match port.direction {
                    PortDirection::Input => {
                        for (i, bit) in port.bits.iter().copied().enumerate() {
                            if let Signal::Net(id) = bit {
                                let wire = get_wire_for_net_id(id, builder, &mut net_map)?;
                                builder.add_slice(port_wire, i as u8, wire).unwrap();
                            }
                        }

                        connections.inputs.insert(Rc::clone(port_name), port_wire);
                    }
                    PortDirection::Output => {
                        let wires = port
                            .bits
                            .iter()
                            .copied()
                            .map(|bit| get_wire_for_bit(bit, builder, &mut net_map))
                            .collect::<Result<Vec<_>, _>>()?;

                        builder.add_merge(&wires, port_wire).unwrap();

                        connections.outputs.insert(Rc::clone(port_name), port_wire);
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::InOutPort {
                            port_name: Rc::clone(port_name),
                        })
                    }
                }
            }
        }

        for (cell_name, cell) in &self.module.cells {
            let mut input_ports = IndexMap::new();
            let mut output_ports = IndexMap::new();
            for (port_name, port) in &cell.ports {
                let Some(port_direction) = port.direction else {
                    return Err(YosysModuleImportError::MissingCellPortDirection {
                        cell_name: Rc::clone(cell_name),
                        port_name: Rc::clone(port_name),
                    });
                };

                if let &[bit] = port.bits.as_slice() {
                    match port_direction {
                        PortDirection::Input => {
                            let wire = get_wire_for_bit(bit, builder, &mut net_map)?;
                            input_ports.insert(Rc::clone(port_name), wire);
                        }
                        PortDirection::Output => {
                            if let Signal::Net(id) = bit {
                                let wire = get_wire_for_net_id(id, builder, &mut net_map)?;
                                output_ports.insert(Rc::clone(port_name), wire);
                            }
                        }
                        PortDirection::InOut => {
                            return Err(YosysModuleImportError::CellInOutPort {
                                cell_name: Rc::clone(cell_name),
                                port_name: Rc::clone(port_name),
                            })
                        }
                    }
                } else {
                    let wire_width = port.bits.len();
                    let wire_width: NonZeroU8 = u8::try_from(wire_width)
                        .and_then(NonZeroU8::try_from)
                        .map_err(|_| YosysModuleImportError::UnsupportedWireWidth { wire_width })?;
                    let port_wire = builder
                        .add_wire(wire_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                    match port_direction {
                        PortDirection::Input => {
                            let wires = port
                                .bits
                                .iter()
                                .copied()
                                .map(|bit| get_wire_for_bit(bit, builder, &mut net_map))
                                .collect::<Result<Vec<_>, _>>()?;

                            builder.add_merge(&wires, port_wire).unwrap();

                            input_ports.insert(Rc::clone(port_name), port_wire);
                        }
                        PortDirection::Output => {
                            for (i, bit) in port.bits.iter().copied().enumerate() {
                                if let Signal::Net(id) = bit {
                                    let wire = get_wire_for_net_id(id, builder, &mut net_map)?;
                                    builder.add_slice(port_wire, i as u8, wire).unwrap();
                                }
                            }

                            output_ports.insert(Rc::clone(port_name), port_wire);
                        }
                        PortDirection::InOut => {
                            return Err(YosysModuleImportError::CellInOutPort {
                                cell_name: Rc::clone(cell_name),
                                port_name: Rc::clone(port_name),
                            })
                        }
                    }
                }
            }

            macro_rules! unary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    builder
                        .$add_gate(input_ports[0], output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }};
            }

            macro_rules! horizontal_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    builder
                        .$add_gate(input_ports[0], output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }};
            }

            macro_rules! binary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let input_ports: smallvec::SmallVec<[_; 2]> =
                        input_ports.values().copied().collect();

                    builder
                        .$add_gate(&input_ports, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }};
            }

            macro_rules! binary_op_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a);
                    let b_width = builder.get_wire_width(input_b);
                    let max_width = a_width.max(b_width);

                    let input_a = if a_width < max_width {
                        let a_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        if cell.ports["A"].signed == Some(true) {
                            builder.add_sign_extend(input_a, a_ext).unwrap();
                        } else {
                            builder.add_zero_extend(input_a, a_ext).unwrap();
                        }
                        a_ext
                    } else {
                        input_a
                    };

                    let input_b = if b_width < max_width {
                        let b_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        if cell.ports["B"].signed == Some(true) {
                            builder.add_sign_extend(input_b, b_ext).unwrap();
                        } else {
                            builder.add_zero_extend(input_b, b_ext).unwrap();
                        }
                        b_ext
                    } else {
                        input_b
                    };

                    builder
                        .$add_gate(input_a, input_b, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }};
            }

            macro_rules! cmp_op_cell {
                ($add_u:ident, $add_s:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a);
                    let b_width = builder.get_wire_width(input_b);
                    let max_width = a_width.max(b_width);

                    let input_a = if a_width < max_width {
                        let a_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        if cell.ports["A"].signed == Some(true) {
                            builder.add_sign_extend(input_a, a_ext).unwrap();
                        } else {
                            builder.add_zero_extend(input_a, a_ext).unwrap();
                        }
                        a_ext
                    } else {
                        input_a
                    };

                    let input_b = if b_width < max_width {
                        let b_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        if cell.ports["B"].signed == Some(true) {
                            builder.add_sign_extend(input_b, b_ext).unwrap();
                        } else {
                            builder.add_zero_extend(input_b, b_ext).unwrap();
                        }
                        b_ext
                    } else {
                        input_b
                    };

                    if (cell.ports["A"].signed == Some(true))
                        || (cell.ports["B"].signed == Some(true))
                    {
                        builder
                            .$add_s(input_a, input_b, output_ports[0])
                            .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                cell_name: Rc::clone(cell_name),
                            })?;
                    } else {
                        builder
                            .$add_u(input_a, input_b, output_ports[0])
                            .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                cell_name: Rc::clone(cell_name),
                            })?;
                    }
                }};
            }

            // https://yosyshq.readthedocs.io/projects/yosys/en/latest/CHAPTER_CellLib.html
            match &cell.cell_type {
                CellType::Not => unary_gate_cell!(add_not_gate),
                CellType::ReduceAnd => horizontal_gate_cell!(add_horizontal_and_gate),
                CellType::ReduceOr | CellType::ReduceBool => {
                    horizontal_gate_cell!(add_horizontal_or_gate)
                }
                CellType::LogicNot => horizontal_gate_cell!(add_horizontal_nor_gate),
                CellType::And => binary_gate_cell!(add_and_gate),
                CellType::Or => binary_gate_cell!(add_or_gate),
                CellType::Xor => binary_gate_cell!(add_xor_gate),
                CellType::Xnor => binary_gate_cell!(add_xnor_gate),
                //CellType::Shl | CellType::Sshl => binary_op_cell!(add_left_shift),
                //CellType::Shr => binary_op_cell!(add_logical_right_shift),
                //CellType::Sshr => binary_op_cell!(add_arithmetic_right_shift),
                CellType::Add => binary_op_cell!(add_add),
                CellType::Sub => binary_op_cell!(add_sub),
                //CellType::Mul => binary_op_cell!(add_mul),
                //CellType::Div => binary_op_cell!(add_div),
                //CellType::Mod => binary_op_cell!(add_rem),
                CellType::Eq => binary_op_cell!(add_compare_equal),
                CellType::Ne => binary_op_cell!(add_compare_not_equal),
                CellType::Lt => cmp_op_cell!(add_compare_less_than, add_compare_less_than_signed),
                CellType::Gt => {
                    cmp_op_cell!(add_compare_greater_than, add_compare_greater_than_signed)
                }
                CellType::Le => cmp_op_cell!(
                    add_compare_less_than_or_equal,
                    add_compare_less_than_or_equal_signed
                ),
                CellType::Ge => cmp_op_cell!(
                    add_compare_greater_than_or_equal,
                    add_compare_greater_than_or_equal_signed
                ),
                CellType::Mux => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let select = *input_ports.get("S").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_multiplexer(&[input_a, input_b], select, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                //CellType::Pmux => {
                //    if input_ports.len() != 3 {
                //        return Err(YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        });
                //    }

                //    if output_ports.len() != 1 {
                //        return Err(YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        });
                //    }

                //    let input_a = *input_ports.get("A").ok_or_else(|| {
                //        YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        }
                //    })?;

                //    let input_b = *input_ports.get("B").ok_or_else(|| {
                //        YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        }
                //    })?;

                //    let select = *input_ports.get("S").ok_or_else(|| {
                //        YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        }
                //    })?;

                //    let input_count = builder.get_wire_width(select).get() as usize;
                //    let input_width = builder.get_wire_width(input_a);

                //    let mut decoder_inputs = Vec::with_capacity(input_count);
                //    let mut mux_inputs = Vec::with_capacity(input_count + 1);
                //    mux_inputs.push(input_a);

                //    for i in 0..input_count {
                //        let select_bi = builder
                //            .add_wire(NonZeroU8::MIN)
                //            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                //        decoder_inputs.push(select_bi);
                //        builder
                //            .add_slice(
                //                select[i / (MAX_LOGIC_WIDTH as usize)],
                //                LogicOffset::new((i % (MAX_LOGIC_WIDTH as usize)) as u8).unwrap(),
                //                select_bi,
                //            )
                //            .unwrap();

                //        let offset = i * (input_width.get() as usize);
                //        let input_bi = builder
                //            .add_wire(input_width)
                //            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                //        builder.add_slice(input_b, offset, &input_bi).map_err(|_| {
                //            YosysModuleImportError::InvalidCellPorts {
                //                cell_name: Rc::clone(cell_name),
                //            }
                //        })?;

                //        mux_inputs.push(input_bi);
                //    }

                //    while !mux_inputs.len().is_power_of_two() {
                //        mux_inputs.push(input_a);
                //    }

                //    let mux_select_width =
                //        NonZeroU8::new((usize::BITS - decoder_inputs.len().leading_zeros()) as u8)
                //            .unwrap();
                //    let mux_select = builder
                //        .add_wire(mux_select_width)
                //        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                //    builder
                //        .add_priority_decoder(&decoder_inputs, mux_select)
                //        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        })?;

                //    builder
                //        .add_multiplexer(&mux_inputs, mux_select, output_ports[0])
                //        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                //            cell_name: Rc::clone(cell_name),
                //        })?;
                //}
                CellType::TriBuf => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_buffer(input, enable, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Dff => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_register(
                            data_in,
                            output_ports[0],
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Dffe => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_register(
                            data_in,
                            output_ports[0],
                            enable,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Sdff => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.set_wire_drive(
                        reset_value,
                        cell.parameters.get("SRST_VALUE").ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: Rc::clone(cell_name),
                            },
                        )?,
                    );

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;

                    builder
                        .add_register(
                            mux_out,
                            output_ports[0],
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Sdffe => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.set_wire_drive(
                        reset_value,
                        cell.parameters.get("SRST_VALUE").ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: Rc::clone(cell_name),
                            },
                        )?,
                    );

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;

                    let or_out = builder
                        .add_wire(NonZeroU8::MIN)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.add_or_gate(&[reset, enable], or_out).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_register(
                            mux_out,
                            output_ports[0],
                            or_out,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Sdffce => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.set_wire_drive(
                        reset_value,
                        cell.parameters.get("SRST_VALUE").ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: Rc::clone(cell_name),
                            },
                        )?,
                    );

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .add_multiplexer(&[reset_value, data_in], enable, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;

                    let or_out = builder
                        .add_wire(NonZeroU8::MIN)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.add_or_gate(&[reset, enable], or_out).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_register(
                            mux_out,
                            output_ports[0],
                            or_out,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;
                }
                CellType::Unknown(cell_type) => {
                    return Err(YosysModuleImportError::UnknownCellType {
                        cell_name: Rc::clone(cell_name),
                        cell_type: Rc::clone(cell_type),
                    })
                }
                cell_type => {
                    return Err(YosysModuleImportError::UnsupportedCellType {
                        cell_name: Rc::clone(cell_name),
                        cell_type: cell_type.clone(),
                    })
                }
            }
        }

        Ok(connections)
    }
}
