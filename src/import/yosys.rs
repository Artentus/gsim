//! Import circuits from Yosys JSON format
//!
//! Use the following command to generate compatible JSON files:</br>
//! `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`

use super::*;
use crate::*;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;

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
    Unknown(Box<str>),
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

fn single_from_map<'de, D, T>(deserializer: D) -> Result<(String, T), D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::de::Error;

    let map = HashMap::<String, T>::deserialize(deserializer)?;
    if map.len() == 1 {
        Ok(map.into_iter().next().unwrap())
    } else {
        Err(Error::invalid_length(
            map.len(),
            &"object with exactly one key",
        ))
    }
}

fn parameter_map<'de, D>(deserializer: D) -> Result<HashMap<String, u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map = HashMap::<String, String>::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (k, u32::from_str_radix(&v, 2).unwrap()))
        .collect())
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
struct Cell {
    #[serde(rename = "type", deserialize_with = "cell_type")]
    cell_type: CellType,
    #[serde(default, deserialize_with = "parameter_map")]
    parameters: HashMap<String, u32>,
    port_directions: HashMap<String, PortDirection>,
    connections: IndexMap<String, Bits>,
}

#[derive(Deserialize)]
struct Module {
    ports: IndexMap<String, Port>,
    #[serde(default)]
    cells: IndexMap<String, Cell>,
}

#[derive(Deserialize)]
struct Netlist {
    #[serde(rename = "modules", deserialize_with = "single_from_map")]
    module: (String, Module),
}

struct PreprocCellPort {
    bits: Bits,
    direction: Option<PortDirection>,
    signed: Option<bool>,
    polarity: Option<bool>,
}

struct PreprocCell {
    cell_type: CellType,
    ports: IndexMap<Box<str>, PreprocCellPort>,
    parameters: HashMap<String, u32>,
}

impl PreprocCell {
    fn create(mut cell: Cell) -> Self {
        let ports = cell
            .connections
            .into_iter()
            .map(|(name, bits)| {
                let preproc_cell = PreprocCellPort {
                    bits,
                    direction: cell.port_directions.remove(&name),
                    signed: cell
                        .parameters
                        .remove(&format!("{name}_SIGNED"))
                        .map(|v| v > 0),
                    polarity: cell
                        .parameters
                        .remove(&format!("{name}_POLARITY"))
                        .map(|v| v > 0),
                };

                (name.into_boxed_str(), preproc_cell)
            })
            .collect();

        Self {
            cell_type: cell.cell_type,
            ports,
            parameters: cell.parameters,
        }
    }
}

struct PreprocModule {
    ports: IndexMap<String, Port>,
    cells: IndexMap<Box<str>, PreprocCell>,
}

impl PreprocModule {
    fn create(module: Module) -> Self {
        Self {
            ports: module.ports,
            cells: module
                .cells
                .into_iter()
                .map(|(name, cell)| (name.into_boxed_str(), PreprocCell::create(cell)))
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
            module_name: netlist.module.0.into_boxed_str(),
            module: PreprocModule::create(netlist.module.1),
        }
    }

    /// Creates a Yosys module importer from a reader containing JSON data
    pub fn from_json_reader<R: std::io::Read>(reader: R) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_reader(reader)?;
        Ok(Self::preprocess(netlist))
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
    /// The module has an `inout` port
    InOutPort {
        /// The name of the `inout` port
        port_name: Box<str>,
    },
    /// The module contains a cell that has an `inout` port
    CellInOutPort {
        /// The name of the cell
        cell_name: Box<str>,
        /// The name of the `inout` port
        port_name: Box<str>,
    },
    /// The module or one of its cells has a port that is wider than `MAX_LOGIC_WIDTH`
    UnsupportedWireWidth {
        /// The width of the wire
        wire_width: usize,
    },
    /// The module contains a cell of unknown type
    UnknownCellType {
        /// The name of the cell
        cell_name: Box<str>,
        /// The unknown type of the cell
        cell_type: Box<str>,
    },
    /// The module contains a cell that is not supported for importing
    UnsupportedCellType {
        /// The name of the cell
        cell_name: Box<str>,
        /// The type of the cell
        cell_type: CellType,
    },
    /// The module contains a cell that has a port with no specified direction
    MissingCellPortDirection {
        /// The name of the cell
        cell_name: Box<str>,
        /// The name of the port
        port_name: Box<str>,
    },
    /// The module contains a cell with an invalid port configuration
    InvalidCellPorts {
        /// The name of the cell
        cell_name: Box<str>,
    },
    /// The module contains a cell with an invalid parameter configuration
    InvalidCellParameters {
        /// The name of the cell
        cell_name: Box<str>,
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

        let const_high_z = builder.add_wire(LogicWidth::MIN);
        builder.set_wire_base_drive(const_high_z, LogicState::HIGH_Z);

        let const_undefined = builder.add_wire(LogicWidth::MIN);
        builder.set_wire_base_drive(const_undefined, LogicState::UNDEFINED);

        let const_0 = builder.add_wire(LogicWidth::MIN);
        builder.set_wire_base_drive(const_0, LogicState::LOGIC_0);

        let const_1 = builder.add_wire(LogicWidth::MIN);
        builder.set_wire_base_drive(const_1, LogicState::LOGIC_1);

        fn get_wire_for_net_id(
            id: NetId,
            builder: &mut crate::SimulatorBuilder,
            net_map: &mut Vec<WireId>,
        ) -> WireId {
            net_map.ensure_len(id + 1);
            if net_map[id] == WireId::INVALID {
                net_map[id] = builder.add_wire(LogicWidth::MIN);
            }

            net_map[id]
        }

        let get_wire_for_bit = move |bit: Signal,
                                     builder: &mut crate::SimulatorBuilder,
                                     net_map: &mut Vec<WireId>|
              -> WireId {
            match bit {
                Signal::Value(LogicBitState::HighZ) => const_high_z,
                Signal::Value(LogicBitState::Undefined) => const_undefined,
                Signal::Value(LogicBitState::Logic0) => const_0,
                Signal::Value(LogicBitState::Logic1) => const_1,
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
                            let wire = get_wire_for_net_id(id, builder, &mut net_map);
                            connections.inputs.insert(port_name.clone(), wire);
                        }
                    }
                    PortDirection::Output => {
                        let wire = get_wire_for_bit(bit, builder, &mut net_map);
                        connections.outputs.insert(port_name.clone(), wire);
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::InOutPort {
                            port_name: port_name.as_str().into(),
                        })
                    }
                }
            } else {
                let wire_width = port.bits.len();
                let wire_width: u8 = wire_width
                    .try_into()
                    .map_err(|_| YosysModuleImportError::UnsupportedWireWidth { wire_width })?;
                let wire_width = LogicWidth::new(wire_width).ok_or(
                    YosysModuleImportError::UnsupportedWireWidth {
                        wire_width: wire_width as usize,
                    },
                )?;
                let port_wire = builder.add_wire(wire_width);

                match port.direction {
                    PortDirection::Input => {
                        connections.inputs.insert(port_name.clone(), port_wire);

                        for (i, bit) in port.bits.iter().copied().enumerate() {
                            if let Signal::Net(id) = bit {
                                let wire = get_wire_for_net_id(id, builder, &mut net_map);

                                builder
                                    .add_slice(port_wire, LogicOffset::new(i as u8).unwrap(), wire)
                                    .unwrap();
                            }
                        }
                    }
                    PortDirection::Output => {
                        connections.outputs.insert(port_name.clone(), port_wire);

                        let wires: Vec<_> = port
                            .bits
                            .iter()
                            .copied()
                            .map(|bit| get_wire_for_bit(bit, builder, &mut net_map))
                            .collect();

                        builder.add_merge(&wires, port_wire).unwrap();
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::InOutPort {
                            port_name: port_name.as_str().into(),
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
                    return Err(YosysModuleImportError::MissingCellPortDirection { cell_name: cell_name.clone(), port_name: port_name.clone() });
                };

                if let &[bit] = port.bits.as_slice() {
                    match port_direction {
                        PortDirection::Input => {
                            let wire = get_wire_for_bit(bit, builder, &mut net_map);
                            input_ports.insert(port_name.clone(), wire);
                        }
                        PortDirection::Output => {
                            if let Signal::Net(id) = bit {
                                let wire = get_wire_for_net_id(id, builder, &mut net_map);
                                output_ports.insert(port_name.clone(), wire);
                            }
                        }
                        PortDirection::InOut => {
                            return Err(YosysModuleImportError::CellInOutPort {
                                cell_name: cell_name.clone(),
                                port_name: port_name.clone(),
                            })
                        }
                    }
                } else {
                    let wire_width = port.bits.len();
                    let wire_width: u8 = wire_width
                        .try_into()
                        .map_err(|_| YosysModuleImportError::UnsupportedWireWidth { wire_width })?;
                    let wire_width = LogicWidth::new(wire_width).ok_or(
                        YosysModuleImportError::UnsupportedWireWidth {
                            wire_width: wire_width as usize,
                        },
                    )?;
                    let port_wire = builder.add_wire(wire_width);

                    match port_direction {
                        PortDirection::Input => {
                            input_ports.insert(port_name.clone(), port_wire);

                            let wires: Vec<_> = port
                                .bits
                                .iter()
                                .copied()
                                .map(|bit| get_wire_for_bit(bit, builder, &mut net_map))
                                .collect();

                            builder.add_merge(&wires, port_wire).unwrap();
                        }
                        PortDirection::Output => {
                            output_ports.insert(port_name.clone(), port_wire);

                            for (i, bit) in port.bits.iter().copied().enumerate() {
                                if let Signal::Net(id) = bit {
                                    let wire = get_wire_for_net_id(id, builder, &mut net_map);

                                    builder
                                        .add_slice(
                                            port_wire,
                                            LogicOffset::new(i as u8).unwrap(),
                                            wire,
                                        )
                                        .unwrap();
                                }
                            }
                        }
                        PortDirection::InOut => {
                            return Err(YosysModuleImportError::CellInOutPort {
                                cell_name: cell_name.clone(),
                                port_name: port_name.clone(),
                            })
                        }
                    }
                }
            }

            macro_rules! unary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    builder
                        .$add_gate(input_ports[0], output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }};
            }

            macro_rules! binary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let input_ports: smallvec::SmallVec<[_; 2]> =
                        input_ports.values().copied().collect();

                    builder
                        .$add_gate(&input_ports, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }};
            }

            macro_rules! binary_op_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .$add_gate(input_a, input_b, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }};
            }

            // https://yosyshq.readthedocs.io/projects/yosys/en/latest/CHAPTER_CellLib.html
            match &cell.cell_type {
                CellType::Not => unary_gate_cell!(add_not_gate),
                CellType::ReduceAnd => unary_gate_cell!(add_horizontal_and_gate),
                CellType::ReduceOr | CellType::ReduceBool => {
                    unary_gate_cell!(add_horizontal_or_gate)
                }
                CellType::LogicNot => unary_gate_cell!(add_horizontal_nor_gate),
                CellType::And => binary_gate_cell!(add_and_gate),
                CellType::Or => binary_gate_cell!(add_or_gate),
                CellType::Xor => binary_gate_cell!(add_xor_gate),
                CellType::Xnor => binary_gate_cell!(add_xnor_gate),
                CellType::Shl | CellType::Sshl => binary_op_cell!(add_left_shift),
                CellType::Shr => binary_op_cell!(add_logical_right_shift),
                CellType::Sshr => binary_op_cell!(add_arithmetic_right_shift),
                CellType::Add => binary_op_cell!(add_add),
                CellType::Sub => binary_op_cell!(add_sub),
                CellType::Mul => binary_op_cell!(add_mul),
                CellType::Div => binary_op_cell!(add_div),
                CellType::Mod => binary_op_cell!(add_rem),
                CellType::Eq => binary_op_cell!(add_compare_equal),
                CellType::Ne => binary_op_cell!(add_compare_not_equal),
                CellType::Lt => binary_op_cell!(add_compare_less_than),
                CellType::Gt => binary_op_cell!(add_compare_greater_than),
                CellType::Le => binary_op_cell!(add_compare_less_than_or_equal),
                CellType::Ge => binary_op_cell!(add_compare_greater_than_or_equal),
                CellType::Mux => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let select = *input_ports.get("S").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_multiplexer(&[input_a, input_b], select, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Pmux => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let select = *input_ports.get("S").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let input_count = builder.get_wire_width(select).get();
                    let input_width = builder.get_wire_width(input_a);

                    let mut decoder_inputs = Vec::with_capacity(input_count as usize);
                    let mut mux_inputs = Vec::with_capacity((input_count as usize) + 1);
                    mux_inputs.push(input_a);

                    for i in 0..input_count {
                        let select_bi = builder.add_wire(LogicWidth::MIN);
                        decoder_inputs.push(select_bi);
                        builder
                            .add_slice(select, LogicOffset::new(i).unwrap(), select_bi)
                            .unwrap();

                        let offset = LogicOffset::new(i * input_width.get()).ok_or_else(|| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: cell_name.clone(),
                            }
                        })?;

                        let input_bi = builder.add_wire(input_width);
                        builder.add_slice(input_b, offset, input_bi).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: cell_name.clone(),
                            }
                        })?;

                        mux_inputs.push(input_bi);
                    }

                    while !mux_inputs.len().is_power_of_two() {
                        mux_inputs.push(input_a);
                    }

                    let mux_select_width =
                        LogicWidth::new((usize::BITS - decoder_inputs.len().leading_zeros()) as u8)
                            .unwrap();
                    let mux_select = builder.add_wire(mux_select_width);
                    builder
                        .add_priority_decoder(&decoder_inputs, mux_select)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;

                    builder
                        .add_multiplexer(&mux_inputs, mux_select, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::TriBuf => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_buffer(input, enable, output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Dff => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_register(data_in, output_ports[0], const_1, clock)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Dffe => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_register(data_in, output_ports[0], enable, clock)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Sdff => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder.add_wire(data_width);
                    builder.set_wire_base_drive(
                        reset_value,
                        LogicState::from_int(cell.parameters.get("SRST_VALUE").copied().ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: cell_name.clone(),
                            },
                        )?),
                    );

                    let mux_out = builder.add_wire(data_width);
                    builder
                        .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;

                    builder
                        .add_register(mux_out, output_ports[0], const_1, clock)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Sdffe => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder.add_wire(data_width);
                    builder.set_wire_base_drive(
                        reset_value,
                        LogicState::from_int(cell.parameters.get("SRST_VALUE").copied().ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: cell_name.clone(),
                            },
                        )?),
                    );

                    let mux_out = builder.add_wire(data_width);
                    builder
                        .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;

                    let or_out = builder.add_wire(LogicWidth::MIN);
                    builder.add_or_gate(&[reset, enable], or_out).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_register(mux_out, output_ports[0], or_out, clock)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Sdffce => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in);

                    let reset_value = builder.add_wire(data_width);
                    builder.set_wire_base_drive(
                        reset_value,
                        LogicState::from_int(cell.parameters.get("SRST_VALUE").copied().ok_or(
                            YosysModuleImportError::InvalidCellParameters {
                                cell_name: cell_name.clone(),
                            },
                        )?),
                    );

                    let mux_out = builder.add_wire(data_width);
                    builder
                        .add_multiplexer(&[reset_value, data_in], enable, mux_out)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;

                    let or_out = builder.add_wire(LogicWidth::MIN);
                    builder.add_or_gate(&[reset, enable], or_out).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        }
                    })?;

                    builder
                        .add_register(mux_out, output_ports[0], or_out, clock)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }
                CellType::Unknown(cell_type) => {
                    return Err(YosysModuleImportError::UnknownCellType {
                        cell_name: cell_name.clone(),
                        cell_type: cell_type.clone(),
                    })
                }
                cell_type => {
                    return Err(YosysModuleImportError::UnsupportedCellType {
                        cell_name: cell_name.clone(),
                        cell_type: cell_type.clone(),
                    })
                }
            }
        }

        Ok(connections)
    }
}
