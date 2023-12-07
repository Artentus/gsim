//! Import circuits from Yosys JSON format
//!
//! Use the following command to generate compatible JSON files:</br>
//! `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -nordff -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`

use super::*;
use crate::*;
use serde::Deserialize;
use std::num::NonZeroU8;
use std::sync::Arc;

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
    Unknown(Arc<str>),
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
    #[serde(default)]
    hide_name: u8,
    #[serde(rename = "type", deserialize_with = "cell_type")]
    cell_type: CellType,
    #[serde(default)]
    parameters: HashMap<String, String>,
    port_directions: HashMap<String, PortDirection>,
    connections: HashMap<String, Bits>,
}

#[derive(Deserialize)]
struct NetNameOpts {
    #[serde(default)]
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
    hide_name: bool,
    cell_type: CellType,
    ports: HashMap<Arc<str>, PreprocCellPort>,
    parameters: HashMap<Arc<str>, LogicState>,
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
            hide_name: cell.hide_name > 0,
            cell_type: cell.cell_type,
            ports,
            parameters,
        }
    }
}

struct PreprocNetNameOpts {
    name: Arc<str>,
    hide_name: bool,
    bits: Bits,
}

struct PreprocModule {
    ports: IndexMap<Arc<str>, Port>,
    cells: HashMap<Arc<str>, PreprocCell>,
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

    /// Creates a Yosys module importer from a stream containing JSON data
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
    /// The simulators resource limit was reached while constructing the module
    ResourceLimitReached,
    /// The module has an `inout` port
    InOutPort {
        /// The name of the `inout` port
        port_name: Arc<str>,
    },
    /// The module contains a cell that has an `inout` port
    CellInOutPort {
        /// The name of the cell
        cell_name: Arc<str>,
        /// The name of the `inout` port
        port_name: Arc<str>,
    },
    /// The module or one of its cells has a port that is wider than `MAX_LOGIC_WIDTH`
    UnsupportedWireWidth {
        /// The width of the wire
        wire_width: usize,
    },
    /// The module contains a cell of unknown type
    UnknownCellType {
        /// The name of the cell
        cell_name: Arc<str>,
        /// The unknown type of the cell
        cell_type: Arc<str>,
    },
    /// The module contains a cell that is not supported for importing
    UnsupportedCellType {
        /// The name of the cell
        cell_name: Arc<str>,
        /// The type of the cell
        cell_type: CellType,
    },
    /// The module contains a cell that has a port with no specified direction
    MissingCellPortDirection {
        /// The name of the cell
        cell_name: Arc<str>,
        /// The name of the port
        port_name: Arc<str>,
    },
    /// The module contains a cell with an invalid port configuration
    InvalidCellPorts {
        /// The name of the cell
        cell_name: Arc<str>,
    },
    /// The module contains a cell with an invalid parameter configuration
    InvalidCellParameters {
        /// The name of the cell
        cell_name: Arc<str>,
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

        builder
            .set_wire_drive(
                bus_wire,
                &LogicState::from_bits(&drive)
                    .unwrap()
                    .undefined_to_logic_0(),
            )
            .unwrap();
    }

    Ok(bus_wire)
}

#[derive(Default, Clone, Copy)]
struct NetMapping {
    wire: WireId,
    offset: u8,
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
    bus_map: HashMap<Bits, WireId>,
    net_map: Vec<NetMapping>,
    fixups: Vec<WireFixup>,
}

impl WireMap {
    fn new(max_net_id: usize) -> Self {
        Self {
            bus_map: HashMap::new(),
            net_map: vec![NetMapping::default(); max_net_id + 1],
            fixups: Vec::new(),
        }
    }

    fn add_const_wire(
        &mut self,
        width: NonZeroU8,
        mut drive: LogicState,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<WireId, YosysModuleImportError> {
        let wire = builder
            .add_wire(width)
            .ok_or(YosysModuleImportError::ResourceLimitReached)?;
        drive = drive.undefined_to_logic_0();
        builder.set_wire_drive(wire, &drive).unwrap();
        builder
            .set_wire_name(wire, drive.display_string(width))
            .unwrap();
        Ok(wire)
    }

    fn add_named_bus(
        &mut self,
        bits: &Bits,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<Option<WireId>, YosysModuleImportError> {
        if self.bus_map.get(bits).is_none() {
            for bit in bits.iter() {
                if let &Signal::Net(net_id) = bit {
                    if let Some(mapping) = self.net_map.get(net_id) {
                        if !mapping.wire.is_invalid() {
                            // Net is already assigned to a bus, abort
                            return Ok(None);
                        }
                    } else {
                        // Named bus refers to a net that doesn't actually exist, abort
                        return Ok(None);
                    }
                }
            }

            let bus_wire = add_wire(bits, None, builder)?;
            self.bus_map.insert(bits.clone(), bus_wire);

            for (offset, &bit) in bits.iter().enumerate() {
                if let Signal::Net(net_id) = bit {
                    self.net_map[net_id] = NetMapping {
                        wire: bus_wire,
                        offset: offset as u8,
                    }
                }
            }

            return Ok(Some(bus_wire));
        }

        Ok(None)
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
                let width = builder.get_wire_width(bus_wire).unwrap();
                let drive = builder.get_wire_drive(bus_wire).unwrap();
                builder
                    .set_wire_name(bus_wire, drive.display_string(width))
                    .unwrap();
                Ok(bus_wire)
            } else {
                let all_nets_invalid = bits
                    .iter()
                    .filter_map(|&bit| match bit {
                        Signal::Value(_) => None,
                        Signal::Net(net_id) => Some(net_id),
                    })
                    .all(|net_id| {
                        let mapping = self.net_map[net_id];
                        mapping.wire.is_invalid()
                    });

                if all_nets_invalid {
                    // None of the wires are part of a bus yet, so we create a new one
                    let bus_wire = add_wire(bits, Some(direction), builder)?;

                    self.bus_map.insert(bits.clone(), bus_wire);
                    for (offset, &bit) in bits.iter().enumerate() {
                        if let Signal::Net(net_id) = bit {
                            self.net_map[net_id] = NetMapping {
                                wire: bus_wire,
                                offset: offset as u8,
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
                    mapping.offset = 0;
                }
                Ok(*mapping)
            }

            fn slice_bus_read(
                &mut self,
                fixup: &WireFixup,
                builder: &mut crate::SimulatorBuilder,
            ) -> Result<(), YosysModuleImportError> {
                enum Slice {
                    Value {
                        width: NonZeroU8,
                        drive: LogicState,
                    },
                    Bus {
                        src: WireId,
                        src_start: u8,
                        src_end: u8,
                    },
                }

                let dst = fixup.bus_wire;
                let dst_width = builder.get_wire_width(dst).unwrap();
                debug_assert_eq!(dst_width.get() as usize, fixup.bits.len());

                let mut slices = Vec::new();
                let mut iter = fixup.bits.iter().copied().peekable();

                // Create slices while we still have more bits
                while let Some(first) = iter.next() {
                    match first {
                        Signal::Value(first_bit) => {
                            let mut bits = Vec::new();
                            bits.push(first_bit);

                            // Advance until we find a bit that is not a value
                            while let Some(&Signal::Value(bit)) = iter.peek() {
                                bits.push(bit);
                                iter.next();
                            }

                            // We didn't find any more bits that are part of this slice, so add it to the list
                            slices.push(Slice::Value {
                                width: NonZeroU8::new(bits.len() as u8).unwrap(),
                                drive: LogicState::from_bits(&bits).unwrap(),
                            });
                        }
                        Signal::Net(first_net_id) => {
                            let first_mapping = self.get_mapping(first_net_id, builder)?;
                            let src = first_mapping.wire;
                            let src_start = first_mapping.offset;
                            let mut src_end = src_start;

                            // Advance until we find a bit that targets a different wire or is not in a contiguous region
                            while let Some(&Signal::Net(net_id)) = iter.peek() {
                                let mapping = self.get_mapping(net_id, builder)?;

                                if (mapping.wire != src)
                                    || (mapping.offset.checked_sub(src_end) != Some(1))
                                {
                                    // This bit has to be part of a different slice
                                    break;
                                }

                                src_end = mapping.offset;
                                iter.next();
                            }

                            // We didn't find any more bits that are part of this slice, so add it to the list
                            slices.push(Slice::Bus {
                                src,
                                src_start,
                                src_end,
                            });
                        }
                    }
                }

                if slices.len() == 1 {
                    let slice = slices.into_iter().next().unwrap();
                    match slice {
                        Slice::Value { .. } => {
                            // Should not have been made into a fixup in the first place
                            unreachable!("illegal fixup");
                        }
                        Slice::Bus { src, src_start, .. } => {
                            builder.add_slice(src, src_start, dst).unwrap();
                        }
                    }
                } else {
                    let mut wires = Vec::new();
                    for slice in slices {
                        match slice {
                            Slice::Value { width, drive } => {
                                let wire = self.add_const_wire(width, drive, builder)?;
                                wires.push(wire);
                            }
                            Slice::Bus {
                                src,
                                src_start,
                                src_end,
                            } => {
                                let width = NonZeroU8::new(src_end - src_start + 1).unwrap();
                                let wire = builder.add_wire(width).unwrap();
                                builder.add_slice(src, src_start, wire).unwrap();
                                wires.push(wire);
                            }
                        }
                    }

                    builder.add_merge(&wires, dst).unwrap();
                }

                Ok(())
            }

            fn slice_bus_write(
                &mut self,
                fixup: &WireFixup,
                builder: &mut crate::SimulatorBuilder,
            ) -> Result<(), YosysModuleImportError> {
                struct Slice {
                    src_start: u8,
                    src_end: u8,
                    dst: WireId,
                    dst_start: u8,
                    dst_end: u8,
                }

                let src = fixup.bus_wire;
                let src_width = builder.get_wire_width(src).unwrap();
                debug_assert_eq!(src_width.get() as usize, fixup.bits.len());

                let mut slices = Vec::new();
                let mut iter = fixup.bits.iter().copied().enumerate().peekable();

                // Create slices while we still have more bits
                while let Some((src_start, first)) = iter.next() {
                    match first {
                        Signal::Value(_) => panic!("illegal file format"),
                        Signal::Net(first_net_id) => {
                            let first_mapping = self.get_mapping(first_net_id, builder)?;
                            let dst = first_mapping.wire;
                            let dst_start = first_mapping.offset;
                            let mut dst_end = dst_start;
                            let mut src_end = src_start;

                            // Advance until we find a bit that targets a different wire or is not in a contiguous region
                            while let Some(&(i, Signal::Net(net_id))) = iter.peek() {
                                let mapping = self.get_mapping(net_id, builder)?;

                                if (mapping.wire != dst)
                                    || (mapping.offset.checked_sub(dst_end) != Some(1))
                                {
                                    // This bit has to be part of a different slice
                                    break;
                                }

                                dst_end = mapping.offset;
                                src_end = i;
                                iter.next();
                            }

                            // We didn't find any more bits that are part of this slice, so add it to the list
                            slices.push(Slice {
                                src_start: src_start as u8,
                                src_end: src_end as u8,
                                dst,
                                dst_start,
                                dst_end,
                            });
                        }
                    }
                }

                debug_assert!(!slices.is_empty());
                debug_assert_eq!(slices.first().unwrap().src_start, 0);
                debug_assert_eq!(slices.last().unwrap().src_end, src_width.get() - 1);

                for Slice {
                    src_start,
                    src_end,
                    dst,
                    dst_start,
                    dst_end,
                } in slices
                {
                    debug_assert_eq!(src_end - src_start, dst_end - dst_start);
                    let slice_width = NonZeroU8::new(src_end - src_start + 1).unwrap();
                    let dst_width = builder.get_wire_width(dst).unwrap();

                    if slice_width == dst_width {
                        debug_assert_eq!(dst_start, 0);
                        debug_assert_eq!(dst_end, dst_width.get() - 1);

                        builder.add_slice(src, src_start, dst).unwrap();
                    } else if slice_width == src_width {
                        debug_assert_eq!(src_start, 0);
                        debug_assert_eq!(src_end, src_width.get() - 1);

                        let mut dst_parts = SmallVec::<[WireId; 3]>::new();
                        if let Some(high_z_width) = NonZeroU8::new(dst_start) {
                            let high_z_wire =
                                self.add_const_wire(high_z_width, LogicState::HIGH_Z, builder)?;
                            dst_parts.push(high_z_wire);
                        }
                        dst_parts.push(src);
                        if let Some(high_z_width) = NonZeroU8::new(dst_width.get() - dst_end - 1) {
                            let high_z_wire =
                                self.add_const_wire(high_z_width, LogicState::HIGH_Z, builder)?;
                            dst_parts.push(high_z_wire);
                        }
                        builder.add_merge(&dst_parts, dst).unwrap();
                    } else {
                        let slice_wire = builder.add_wire(slice_width).unwrap();
                        builder.add_slice(src, src_start, slice_wire).unwrap();

                        let mut dst_parts = SmallVec::<[WireId; 3]>::new();
                        if let Some(high_z_width) = NonZeroU8::new(dst_start) {
                            let high_z_wire =
                                self.add_const_wire(high_z_width, LogicState::HIGH_Z, builder)?;
                            dst_parts.push(high_z_wire);
                        }
                        dst_parts.push(slice_wire);
                        if let Some(high_z_width) = NonZeroU8::new(dst_width.get() - dst_end - 1) {
                            let high_z_wire =
                                self.add_const_wire(high_z_width, LogicState::HIGH_Z, builder)?;
                            dst_parts.push(high_z_wire);
                        }
                        builder.add_merge(&dst_parts, dst).unwrap();
                    }
                }

                Ok(())
            }
        }

        let mut fixups = Vec::new();
        std::mem::swap(&mut fixups, &mut self.fixups);
        for fixup in fixups {
            // TODO: if multiple bits connect to the same bus wire we can use one split/merge for all of them
            match fixup.direction {
                BusDirection::Read => {
                    self.slice_bus_read(&fixup, builder)?;
                }
                BusDirection::Write => {
                    self.slice_bus_write(&fixup, builder)?;
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
                    connections.inputs.insert(Arc::clone(port_name), port_wire);
                    builder
                        .set_wire_name(port_wire, Arc::clone(port_name))
                        .unwrap();
                }
                PortDirection::Output => {
                    let port_wire =
                        wire_map.get_bus_wire(&port.bits, BusDirection::Read, builder)?;
                    connections.outputs.insert(Arc::clone(port_name), port_wire);
                    builder
                        .set_wire_name(port_wire, Arc::clone(port_name))
                        .unwrap();
                }
                PortDirection::InOut => {
                    return Err(YosysModuleImportError::InOutPort {
                        port_name: Arc::clone(port_name),
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
                    if !opts.hide_name {
                        builder
                            .set_wire_name(bus_wire, Arc::clone(&opts.name))
                            .unwrap();
                    }
                }
            }
        }

        for (cell_name, cell) in &self.module.cells {
            let mut input_ports = HashMap::new();
            let mut output_ports = HashMap::new();
            for (port_name, port) in &cell.ports {
                let Some(port_direction) = port.direction else {
                    return Err(YosysModuleImportError::MissingCellPortDirection {
                        cell_name: Arc::clone(cell_name),
                        port_name: Arc::clone(port_name),
                    });
                };

                match port_direction {
                    PortDirection::Input => {
                        let port_wire =
                            wire_map.get_bus_wire(&port.bits, BusDirection::Read, builder)?;
                        input_ports.insert(Arc::clone(port_name), port_wire);
                    }
                    PortDirection::Output => {
                        let port_wire =
                            wire_map.get_bus_wire(&port.bits, BusDirection::Write, builder)?;
                        output_ports.insert(Arc::clone(port_name), port_wire);
                    }
                    PortDirection::InOut => {
                        return Err(YosysModuleImportError::CellInOutPort {
                            cell_name: Arc::clone(cell_name),
                            port_name: Arc::clone(port_name),
                        });
                    }
                }
            }

            macro_rules! unary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    builder.$add_gate(input, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?
                }};
            }

            macro_rules! horizontal_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    builder.$add_gate(input, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?
                }};
            }

            macro_rules! binary_gate_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    builder
                        .$add_gate(&[input_a, input_b], output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }};
            }

            macro_rules! shift_op_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a).unwrap();
                    let o_width = builder.get_wire_width(output).unwrap();
                    let max_width = a_width.max(o_width);

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

                    let output = if o_width < max_width {
                        let o_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        builder.add_slice(o_ext, 0, output).unwrap();

                        o_ext
                    } else {
                        output
                    };

                    let b_width = builder.get_wire_width(input_b).unwrap();
                    let target_b_width = NonZeroU8::new(max_width.clog2()).ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = if b_width < target_b_width {
                        let b_ext = builder
                            .add_wire(target_b_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        builder.add_zero_extend(input_b, b_ext).unwrap();

                        b_ext
                    } else if b_width > target_b_width {
                        let b_ext = builder
                            .add_wire(target_b_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        builder.add_slice(input_b, 0, b_ext).unwrap();

                        b_ext
                    } else {
                        input_b
                    };

                    builder.$add_gate(input_a, input_b, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?
                }};
            }

            macro_rules! binary_op_cell {
                ($add_gate:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a).unwrap();
                    let b_width = builder.get_wire_width(input_b).unwrap();
                    let o_width = builder.get_wire_width(output).unwrap();
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

                    let output = if o_width < max_width {
                        let o_ext = builder
                            .add_wire(max_width)
                            .ok_or(YosysModuleImportError::ResourceLimitReached)?;

                        builder.add_slice(o_ext, 0, output).unwrap();

                        o_ext
                    } else {
                        output
                    };

                    builder.$add_gate(input_a, input_b, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?
                }};
            }

            macro_rules! cmp_op_cell {
                ($add_u:ident, $add_s:ident) => {{
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let a_width = builder.get_wire_width(input_a).unwrap();
                    let b_width = builder.get_wire_width(input_b).unwrap();
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
                        builder.$add_s(input_a, input_b, output).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: Arc::clone(cell_name),
                            }
                        })?
                    } else {
                        builder.$add_u(input_a, input_b, output).map_err(|_| {
                            YosysModuleImportError::InvalidCellPorts {
                                cell_name: Arc::clone(cell_name),
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
                CellType::Shl | CellType::Sshl => shift_op_cell!(add_left_shift),
                CellType::Shr => shift_op_cell!(add_logical_right_shift),
                CellType::Sshr => shift_op_cell!(add_arithmetic_right_shift),
                CellType::Add => binary_op_cell!(add_add),
                CellType::Sub => binary_op_cell!(add_sub),
                CellType::Eq => cmp_op_cell!(add_compare_equal, add_compare_equal),
                CellType::Ne => cmp_op_cell!(add_compare_not_equal, add_compare_not_equal),
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
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let select = *input_ports.get("S").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    builder
                        .add_multiplexer(&[input_a, input_b], select, output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Pmux => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input_a = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_b = *input_ports.get("B").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let select = *input_ports.get("S").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let input_count = builder.get_wire_width(select).unwrap().get() as usize;
                    let input_width = builder.get_wire_width(input_a).unwrap();

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
                                cell_name: Arc::clone(cell_name),
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
                            cell_name: Arc::clone(cell_name),
                        })?;

                    builder
                        .add_multiplexer(&mux_inputs, mux_select, output)
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::TriBuf => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let input = *input_ports.get("A").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Y").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    builder.add_buffer(input, enable, output).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?
                }
                CellType::Dff => {
                    if input_ports.len() != 2 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let const_1 =
                        wire_map.add_const_wire(NonZeroU8::MIN, LogicState::LOGIC_1, builder)?;
                    builder
                        .add_register(
                            data_in,
                            output,
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Dffe => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
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
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Sdff => {
                    if input_ports.len() != 3 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in).unwrap();

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .set_wire_drive(
                            reset_value,
                            cell.parameters.get("SRST_VALUE").ok_or(
                                YosysModuleImportError::InvalidCellParameters {
                                    cell_name: Arc::clone(cell_name),
                                },
                            )?,
                        )
                        .unwrap();

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    match cell.ports["SRST"].polarity.unwrap_or_default() {
                        ClockPolarity::Rising => {
                            builder
                                .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                        ClockPolarity::Falling => {
                            builder
                                .add_multiplexer(&[reset_value, data_in], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                    }

                    let const_1 =
                        wire_map.add_const_wire(NonZeroU8::MIN, LogicState::LOGIC_1, builder)?;
                    builder
                        .add_register(
                            mux_out,
                            output,
                            const_1,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Sdffe => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in).unwrap();

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .set_wire_drive(
                            reset_value,
                            cell.parameters.get("SRST_VALUE").ok_or(
                                YosysModuleImportError::InvalidCellParameters {
                                    cell_name: Arc::clone(cell_name),
                                },
                            )?,
                        )
                        .unwrap();

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    match cell.ports["SRST"].polarity.unwrap_or_default() {
                        ClockPolarity::Rising => {
                            builder
                                .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                        ClockPolarity::Falling => {
                            builder
                                .add_multiplexer(&[reset_value, data_in], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                    }

                    let or_out = builder
                        .add_wire(NonZeroU8::MIN)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder.add_or_gate(&[reset, enable], or_out).map_err(|_| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
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
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Sdffce => {
                    if input_ports.len() != 4 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    if output_ports.len() != 1 {
                        return Err(YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        });
                    }

                    let data_in = *input_ports.get("D").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let clock = *input_ports.get("CLK").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let reset = *input_ports.get("SRST").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let enable = *input_ports.get("EN").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let output = *output_ports.get("Q").ok_or_else(|| {
                        YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        }
                    })?;

                    let data_width = builder.get_wire_width(data_in).unwrap();

                    let reset_value = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    builder
                        .set_wire_drive(
                            reset_value,
                            cell.parameters.get("SRST_VALUE").ok_or(
                                YosysModuleImportError::InvalidCellParameters {
                                    cell_name: Arc::clone(cell_name),
                                },
                            )?,
                        )
                        .unwrap();

                    let mux_out = builder
                        .add_wire(data_width)
                        .ok_or(YosysModuleImportError::ResourceLimitReached)?;
                    match cell.ports["SRST"].polarity.unwrap_or_default() {
                        ClockPolarity::Rising => {
                            builder
                                .add_multiplexer(&[data_in, reset_value], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                        ClockPolarity::Falling => {
                            builder
                                .add_multiplexer(&[reset_value, data_in], reset, mux_out)
                                .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                                    cell_name: Arc::clone(cell_name),
                                })?;
                        }
                    }

                    builder
                        .add_register(
                            mux_out,
                            output,
                            enable,
                            clock,
                            cell.ports["CLK"].polarity.unwrap_or_default(),
                        )
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: Arc::clone(cell_name),
                        })?
                }
                CellType::Unknown(cell_type) => {
                    return Err(YosysModuleImportError::UnknownCellType {
                        cell_name: Arc::clone(cell_name),
                        cell_type: Arc::clone(cell_type),
                    })
                }
                cell_type => {
                    return Err(YosysModuleImportError::UnsupportedCellType {
                        cell_name: Arc::clone(cell_name),
                        cell_type: cell_type.clone(),
                    })
                }
            };

            if let ComponentData::RegisterValue(mut reg) =
                builder.get_component_data_mut(cell_id).unwrap()
            {
                // Yosys optimizes designs in a way that doesn't account for undefined register values,
                // so we have to set registers to a valid logic state to make the design work in the simulation.
                reg.write(&LogicState::LOGIC_0);
            }

            if !cell.hide_name {
                builder
                    .set_component_name(cell_id, Arc::clone(cell_name))
                    .unwrap();
            }
        }

        wire_map.perform_fixups(builder)?;

        Ok(connections)
    }
}
