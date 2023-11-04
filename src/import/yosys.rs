//! Import circuits from Yosys JSON format
//!
//! Use the following command to generate compatible JSON files:</br>
//! `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`

use super::*;
use crate::*;
use serde::Deserialize;
use std::num::NonZeroU8;
use std::rc::Rc;

type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
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
    #[serde(default)]
    parameters: HashMap<String, String>,
    port_directions: HashMap<String, PortDirection>,
    connections: HashMap<String, Bits>,
}

#[derive(Deserialize)]
struct NetNameOpts {
    hide_name: u8,
    bits: Bits,
}

#[derive(Deserialize)]
struct Module {
    ports: HashMap<String, Port>,
    #[serde(default)]
    cells: HashMap<String, Cell>,
    #[serde(default, rename = "netnames")]
    net_names: HashMap<String, NetNameOpts>,
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
    polarity: Option<ClockPolarity>,
}

struct PreprocCell {
    cell_type: CellType,
    ports: HashMap<Rc<str>, PreprocCellPort>,
    parameters: HashMap<Rc<str>, LogicState>,
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
                        .remove(format!("{name}_SIGNED").as_str())
                        .map(|v| u32::from_str_radix(&v, 2).unwrap() > 0),
                    polarity: cell
                        .parameters
                        .remove(format!("{name}_POLARITY").as_str())
                        .map(|v| match u32::from_str_radix(&v, 2).unwrap() {
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
            .map(|(k, v)| (k.into(), LogicState::parse(&v).unwrap()))
            .collect();

        Self {
            cell_type: cell.cell_type,
            ports,
            parameters,
        }
    }
}

struct PreprocNetNameOpts {
    name: Rc<str>,
    hide_name: bool,
    bits: Bits,
}

struct PreprocModule {
    ports: IndexMap<Rc<str>, Port>,
    cells: HashMap<Rc<str>, PreprocCell>,
    net_names: Vec<PreprocNetNameOpts>,
}

impl PreprocModule {
    fn create(module: Module) -> Self {
        let mut ports: IndexMap<_, _> = module
            .ports
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect();

        let cells = module
            .cells
            .into_iter()
            .map(|(name, cell)| (name.into(), PreprocCell::create(cell)))
            .collect();

        let mut net_names: Vec<_> = module
            .net_names
            .into_iter()
            .map(|(name, opts)| PreprocNetNameOpts {
                name: name.into(),
                hide_name: opts.hide_name > 0,
                bits: opts.bits,
            })
            .collect();

        ports.sort_unstable_by(|_, a, _, b| b.bits.len().cmp(&a.bits.len()));
        net_names.sort_unstable_by_key(|opts| (opts.hide_name, std::cmp::Reverse(opts.bits.len())));

        Self {
            ports,
            cells,
            net_names,
        }
    }

    fn max_net_id(&self) -> NetId {
        let mut max_net_id = 0;

        for port in self.ports.values() {
            for &bit in &port.bits {
                if let Signal::Net(net_id) = bit {
                    max_net_id = max_net_id.max(net_id);
                }
            }
        }

        for cell in self.cells.values() {
            for port in cell.ports.values() {
                for &bit in &port.bits {
                    if let Signal::Net(net_id) = bit {
                        max_net_id = max_net_id.max(net_id);
                    }
                }
            }
        }

        max_net_id
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

fn add_wire(
    bits: &Bits,
    set_drive: Option<BusDirection>,
    builder: &mut crate::SimulatorBuilder,
) -> Result<WireId, YosysModuleImportError> {
    let bus_width = u8::try_from(bits.len())
        .and_then(NonZeroU8::try_from)
        .map_err(|_| YosysModuleImportError::UnsupportedWireWidth {
            wire_width: bits.len(),
        })?;

    let bus_wire = builder
        .add_wire(bus_width)
        .ok_or(YosysModuleImportError::ResourceLimitReached)?;

    if let Some(direction) = set_drive {
        let mut drive = vec![LogicBitState::HighZ; bits.len()];
        for (i, &bit) in bits.iter().rev().enumerate() {
            if let Signal::Value(value) = bit {
                match direction {
                    BusDirection::Read => drive[i] = value,
                    BusDirection::Write => panic!("illegal file format"),
                }
            }
        }

        builder.set_wire_drive(bus_wire, &LogicState::from_bits(&drive));
    }

    Ok(bus_wire)
}

fn to_contiguous_range(indices: &[u8]) -> Option<u8> {
    let mut iter = indices.iter().copied();
    if let Some(start) = iter.next() {
        let mut end = start;
        while let Some(next) = iter.next() {
            if next.checked_sub(end) == Some(1) {
                end = next;
            } else {
                return None;
            }
        }

        Some(start)
    } else {
        None
    }
}

#[derive(Default, Clone, Copy)]
struct NetMapping {
    wire: WireId,
    /// None means it is the only bit in the bus
    offset: Option<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BusDirection {
    Read,
    Write,
}

struct WireFixup {
    bus_wire: WireId,
    direction: BusDirection,
    bits: Bits,
}

struct WireMap {
    const_high_z: Option<WireId>,
    const_undefined: Option<WireId>,
    const_0: Option<WireId>,
    const_1: Option<WireId>,
    bus_map: HashMap<Bits, WireId>,
    net_map: Vec<NetMapping>,
    fixups: Vec<WireFixup>,
}

impl WireMap {
    fn new(max_net_id: usize) -> Self {
        Self {
            const_high_z: None,
            const_undefined: None,
            const_0: None,
            const_1: None,
            bus_map: HashMap::new(),
            net_map: vec![NetMapping::default(); max_net_id + 1],
            fixups: Vec::new(),
        }
    }

    fn const_high_z(
        &mut self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        if let Some(const_high_z) = self.const_high_z {
            Ok(const_high_z)
        } else {
            let const_high_z = builder
                .add_wire(NonZeroU8::MIN)
                .ok_or(YosysModuleImportError::ResourceLimitReached)?;
            builder.set_wire_drive(const_high_z, &LogicState::HIGH_Z);
            builder.set_wire_name(const_high_z, "Z");
            self.const_high_z = Some(const_high_z);
            Ok(const_high_z)
        }
    }

    fn const_undefined(
        &mut self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        if let Some(const_undefined) = self.const_undefined {
            Ok(const_undefined)
        } else {
            let const_undefined = builder
                .add_wire(NonZeroU8::MIN)
                .ok_or(YosysModuleImportError::ResourceLimitReached)?;
            builder.set_wire_drive(const_undefined, &LogicState::UNDEFINED);
            builder.set_wire_name(const_undefined, "X");
            self.const_undefined = Some(const_undefined);
            Ok(const_undefined)
        }
    }

    fn const_0(
        &mut self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        if let Some(const_0) = self.const_0 {
            Ok(const_0)
        } else {
            let const_0 = builder
                .add_wire(NonZeroU8::MIN)
                .ok_or(YosysModuleImportError::ResourceLimitReached)?;
            builder.set_wire_drive(const_0, &LogicState::LOGIC_0);
            builder.set_wire_name(const_0, "0");
            self.const_0 = Some(const_0);
            Ok(const_0)
        }
    }

    fn const_1(
        &mut self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        if let Some(const_1) = self.const_1 {
            Ok(const_1)
        } else {
            let const_1 = builder
                .add_wire(NonZeroU8::MIN)
                .ok_or(YosysModuleImportError::ResourceLimitReached)?;
            builder.set_wire_drive(const_1, &LogicState::LOGIC_1);
            builder.set_wire_name(const_1, "1");
            self.const_1 = Some(const_1);
            Ok(const_1)
        }
    }

    fn all_nets_invalid(&self, bits: &Bits) -> bool {
        bits.iter()
            .filter_map(|&bit| match bit {
                Signal::Value(_) => None,
                Signal::Net(net_id) => Some(net_id),
            })
            .all(|net_id| {
                let mapping = self.net_map[net_id];
                mapping.wire.is_invalid()
            })
    }

    fn add_named_bus(
        &mut self,
        bits: &Bits,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<Option<WireId>, YosysModuleImportError> {
        if self.all_nets_invalid(bits) {
            let bus_wire = add_wire(bits, None, builder)?;

            self.bus_map.insert(bits.clone(), bus_wire);
            if let &[single_bit] = bits.as_slice() {
                if let Signal::Net(net_id) = single_bit {
                    self.net_map[net_id] = NetMapping {
                        wire: bus_wire,
                        offset: None,
                    }
                }
            } else {
                for (offset, &bit) in bits.iter().enumerate() {
                    if let Signal::Net(net_id) = bit {
                        self.net_map[net_id] = NetMapping {
                            wire: bus_wire,
                            offset: Some(offset as u8),
                        }
                    }
                }
            }

            Ok(Some(bus_wire))
        } else {
            Ok(None)
        }
    }

    fn get_bus_wire(
        &mut self,
        bits: &Bits,
        direction: BusDirection,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        if let Some(&wire) = self.bus_map.get(bits) {
            // We have already seen this exact bus
            Ok(wire)
        } else {
            let all_values = bits.iter().all(|bit| matches!(bit, Signal::Value { .. }));
            if all_values {
                // All of the bits are values, not nets, so this is not a bus
                let bus_wire = add_wire(bits, Some(direction), builder)?;
                let width = builder.get_wire_width(bus_wire);
                let drive = builder.get_wire_drive(bus_wire);
                builder.set_wire_name(bus_wire, drive.display_string(width));
                Ok(bus_wire)
            } else {
                if self.all_nets_invalid(bits) {
                    // None of the wires are part of a bus yet, so we create a new one
                    let bus_wire = add_wire(bits, Some(direction), builder)?;

                    self.bus_map.insert(bits.clone(), bus_wire);
                    if let &[single_bit] = bits.as_slice() {
                        if let Signal::Net(net_id) = single_bit {
                            self.net_map[net_id] = NetMapping {
                                wire: bus_wire,
                                offset: None,
                            }
                        }
                    } else {
                        for (offset, &bit) in bits.iter().enumerate() {
                            if let Signal::Net(net_id) = bit {
                                self.net_map[net_id] = NetMapping {
                                    wire: bus_wire,
                                    offset: Some(offset as u8),
                                }
                            }
                        }
                    }

                    Ok(bus_wire)
                } else {
                    // Some of the wires are already part of a bus, so we
                    // create a dummy bus and postpone the connection
                    let bus_wire = add_wire(bits, None, builder)?;

                    self.fixups.push(WireFixup {
                        bus_wire,
                        direction,
                        bits: bits.clone(),
                    });

                    Ok(bus_wire)
                }
            }
        }
    }

    fn perform_fixups(
        &mut self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<(), YosysModuleImportError> {
        impl WireMap {
            fn get_mapping(
                &mut self,
                net_id: NetId,
                builder: &mut crate::SimulatorBuilder,
            ) -> Result<NetMapping, YosysModuleImportError> {
                let mapping = &mut self.net_map[net_id];
                if mapping.wire.is_invalid() {
                    // At this point, if a wire has no mapping yet, we didn't
                    // find a bus containing it, so we add it individually
                    mapping.wire = builder
                        .add_wire(NonZeroU8::MIN)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    mapping.offset = None;
                }
                Ok(*mapping)
            }

            fn get_bit(
                &mut self,
                net_id: NetId,
                builder: &mut crate::SimulatorBuilder,
            ) -> Result<WireId, YosysModuleImportError> {
                let mapping = self.get_mapping(net_id, builder)?;

                if let Some(offset) = mapping.offset {
                    let output = builder
                        .add_wire(NonZeroU8::MIN)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                    builder.add_slice(mapping.wire, offset, output).unwrap();

                    Ok(output)
                } else {
                    // If the bit is the only one in the bus we don't have to slice
                    Ok(mapping.wire)
                }
            }

            fn slice_bus(
                &mut self,
                fixup: &WireFixup,
                builder: &mut crate::SimulatorBuilder,
            ) -> Result<bool, YosysModuleImportError> {
                let mut indices = Vec::new();
                let mut src_bus: Option<WireId> = None;

                for &bit in &fixup.bits {
                    match bit {
                        Signal::Value(_) => return Ok(false),
                        Signal::Net(net_id) => {
                            let mapping = self.get_mapping(net_id, builder)?;
                            if *src_bus.get_or_insert(mapping.wire) != mapping.wire {
                                return Ok(false);
                            }

                            indices.push(mapping.offset.unwrap_or(0));
                        }
                    }
                }

                if let Some(offset) = to_contiguous_range(&indices) {
                    builder
                        .add_slice(src_bus.unwrap(), offset, fixup.bus_wire)
                        .unwrap();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        let mut fixups = Vec::new();
        std::mem::swap(&mut fixups, &mut self.fixups);
        for fixup in fixups {
            // TODO: if multiple bits connect to the same bus wire we can use one split/merge for all of them
            match fixup.direction {
                BusDirection::Read => {
                    if !self.slice_bus(&fixup, builder)? {
                        let mut bit_wires = Vec::new();
                        for bit in fixup.bits {
                            let bit_wire = match bit {
                                Signal::Value(LogicBitState::HighZ) => {
                                    self.const_high_z(builder)?
                                }
                                Signal::Value(LogicBitState::Undefined) => {
                                    self.const_undefined(builder)?
                                }
                                Signal::Value(LogicBitState::Logic0) => self.const_0(builder)?,
                                Signal::Value(LogicBitState::Logic1) => self.const_1(builder)?,
                                Signal::Net(net_id) => self.get_bit(net_id, builder)?,
                            };

                            bit_wires.push(bit_wire);
                        }

                        builder.add_merge(&bit_wires, fixup.bus_wire).unwrap();
                    }
                }
                BusDirection::Write => {
                    for (i, bit) in fixup.bits.into_iter().enumerate() {
                        match bit {
                            Signal::Value(_) => panic!("illegal file format"),
                            Signal::Net(net_id) => {
                                let mapping = self.get_mapping(net_id, builder)?;
                                if let Some(offset) = mapping.offset {
                                    let bit_wire = builder
                                        .add_wire(NonZeroU8::MIN)
                                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                                    builder
                                        .add_slice(fixup.bus_wire, i as u8, bit_wire)
                                        .unwrap();

                                    let mapping = self.net_map[net_id];
                                    let target_width = builder.get_wire_width(mapping.wire);
                                    let mut target_parts = Vec::new();
                                    if let Some(high_z_width) = NonZeroU8::new(offset) {
                                        let high_z_wire = builder
                                            .add_wire(high_z_width)
                                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                                        builder.set_wire_drive(high_z_wire, &LogicState::HIGH_Z);
                                        target_parts.push(high_z_wire);
                                    }
                                    target_parts.push(bit_wire);
                                    if let Some(high_z_width) =
                                        NonZeroU8::new(target_width.get() - offset - 1)
                                    {
                                        let high_z_wire = builder
                                            .add_wire(high_z_width)
                                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                                        builder.set_wire_drive(high_z_wire, &LogicState::HIGH_Z);
                                        target_parts.push(high_z_wire);
                                    }
                                    builder.add_merge(&target_parts, mapping.wire).unwrap();
                                } else {
                                    // If the bit is the only one in the bus we can drive directly
                                    builder
                                        .add_slice(fixup.bus_wire, i as u8, mapping.wire)
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
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
        let max_net_id = self.module.max_net_id();
        let mut wire_map = WireMap::new(max_net_id);

        let mut connections = ModuleConnections::default();
        for (port_name, port) in &self.module.ports {
            match port.direction {
                PortDirection::Input => {
                    let port_wire =
                        wire_map.get_bus_wire(&port.bits, BusDirection::Write, builder)?;
                    connections.inputs.insert(Rc::clone(port_name), port_wire);
                    builder.set_wire_name(port_wire, Rc::clone(port_name));
                }
                PortDirection::Output => {
                    let port_wire =
                        wire_map.get_bus_wire(&port.bits, BusDirection::Read, builder)?;
                    connections.outputs.insert(Rc::clone(port_name), port_wire);
                    builder.set_wire_name(port_wire, Rc::clone(port_name));
                }
                PortDirection::InOut => {
                    return Err(YosysModuleImportError::InOutPort {
                        port_name: Rc::clone(port_name),
                    });
                }
            }
        }

        for opts in &self.module.net_names {
            if opts
                .bits
                .iter()
                .all(|bit| matches!(bit, Signal::Net { .. }))
            {
                if let Some(bus_wire) = wire_map.add_named_bus(&opts.bits, builder)? {
                    builder.set_wire_name(bus_wire, Rc::clone(&opts.name));
                }
            }
        }

        for (cell_name, cell) in &self.module.cells {
            let mut input_ports = HashMap::new();
            let mut output_ports = HashMap::new();
            for (port_name, port) in &cell.ports {
                let Some(port_direction) = port.direction else {
                    return Err(YosysModuleImportError::MissingCellPortDirection {
                        cell_name: Rc::clone(cell_name),
                        port_name: Rc::clone(port_name),
                    });
                };

                match port_direction {
                    PortDirection::Input => {
                        let port_wire =
                            wire_map.get_bus_wire(&port.bits, BusDirection::Read, builder)?;
                        input_ports.insert(Rc::clone(port_name), port_wire);
                    }
                    PortDirection::Output => {
                        let port_wire =
                            wire_map.get_bus_wire(&port.bits, BusDirection::Write, builder)?;
                        output_ports.insert(Rc::clone(port_name), port_wire);
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::CellInOutPort {
                            cell_name: Rc::clone(cell_name),
                            port_name: Rc::clone(port_name),
                        });
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

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder.$add_gate(input, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?
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

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder.$add_gate(input, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .$add_gate(&[input_a, input_b], output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a);
                    let b_width = builder.get_wire_width(input_b);
                    let o_width = builder.get_wire_width(output);
                    let max_width = a_width.max(b_width).max(o_width);

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

                    builder.$add_gate(input_a, input_b, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a);
                    let b_width = builder.get_wire_width(input_b);
                    let o_width = builder.get_wire_width(output);
                    let max_width = a_width.max(b_width).max(o_width);

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
                        builder.$add_s(input_a, input_b, output).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: Rc::clone(cell_name),
                            }
                        })?
                    } else {
                        builder.$add_u(input_a, input_b, output).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: Rc::clone(cell_name),
                            }
                        })?
                    }
                }};
            }

            // https://yosyshq.readthedocs.io/projects/yosys/en/latest/CHAPTER_CellLib.html
            let cell_id = match &cell.cell_type {
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
                CellType::Shl | CellType::Sshl => binary_op_cell!(add_left_shift),
                CellType::Shr => binary_op_cell!(add_logical_right_shift),
                CellType::Sshr => binary_op_cell!(add_arithmetic_right_shift),
                CellType::Add => binary_op_cell!(add_add),
                CellType::Sub => binary_op_cell!(add_sub),
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_multiplexer(&[input_a, input_b], select, output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
                }
                CellType::Pmux => {
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let input_count = builder.get_wire_width(select).get() as usize;
                    let input_width = builder.get_wire_width(input_a);

                    let mut decoder_inputs = Vec::with_capacity(input_count);
                    let mut mux_inputs = Vec::with_capacity(input_count + 1);
                    mux_inputs.push(input_a);

                    for i in 0..input_count {
                        let select_bi = builder
                            .add_wire(NonZeroU8::MIN)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                        decoder_inputs.push(select_bi);
                        builder.add_slice(select, i as u8, select_bi).unwrap();

                        let offset = (i * (input_width.get() as usize)) as u8;
                        let input_bi = builder
                            .add_wire(input_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                        builder.add_slice(input_b, offset, input_bi).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: Rc::clone(cell_name),
                            }
                        })?;

                        mux_inputs.push(input_bi);
                    }

                    while !mux_inputs.len().is_power_of_two() {
                        mux_inputs.push(input_a);
                    }

                    let mux_select_width =
                        NonZeroU8::new((usize::BITS - decoder_inputs.len().leading_zeros()) as u8)
                            .unwrap();
                    let mux_select = builder
                        .add_wire(mux_select_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .add_priority_decoder(&decoder_inputs, mux_select)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?;

                    builder
                        .add_multiplexer(&mux_inputs, mux_select, output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
                }
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

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder.add_buffer(input, enable, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?
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

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    let const_1 = wire_map.const_1(builder)?;
                    builder
                        .add_register(
                            data_in,
                            output,
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_register(
                            data_in,
                            output,
                            enable,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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

                    let output = *output_ports.get("Q").ok_or_else(|| {
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

                    let const_1 = wire_map.const_1(builder)?;
                    builder
                        .add_register(
                            mux_out,
                            output,
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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

                    let output = *output_ports.get("Q").ok_or_else(|| {
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
                            output,
                            or_out,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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

                    let output = *output_ports.get("Q").ok_or_else(|| {
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
                            output,
                            or_out,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Rc::clone(cell_name),
                        })?
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
            };

            builder.set_component_name(cell_id, Rc::clone(cell_name));
        }

        wire_map.perform_fixups(builder)?;

        Ok(connections)
    }
}
