//! Import circuits from Yosys JSON format
//!
//! Use the following command to generate compatible JSON files:</br>
//! `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`

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
    #[serde(rename = "type")]
    cell_type: String,
    port_directions: IndexMap<String, PortDirection>,
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

/// Imports circuits generated by Yosys
///
/// Use the following command to generate compatible JSON files:</br>
/// `yosys -p "read_verilog <VERILOG-FILE>; synth -top <TOP-MODULE> -flatten -noalumacc -run begin:fine; hierarchy -check; check; write_json <OUTPUT-FILE>"`
pub struct YosysModuleImporter {
    module_name: String,
    module: Module,
}

impl YosysModuleImporter {
    /// Creates a Yosys module importer from a reader containing JSON data
    pub fn from_json_reader<R: std::io::Read>(reader: R) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_reader(reader)?;
        Ok(Self {
            module_name: netlist.module.0,
            module: netlist.module.1,
        })
    }

    /// Creates a Yosys module importer from a slice containing JSON data
    pub fn from_json_slice(slice: &[u8]) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_slice(slice)?;
        Ok(Self {
            module_name: netlist.module.0,
            module: netlist.module.1,
        })
    }

    /// Creates a Yosys module importer from a string containing JSON data
    pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
        let netlist: Netlist = serde_json::from_str(s)?;
        Ok(Self {
            module_name: netlist.module.0,
            module: netlist.module.1,
        })
    }
}

/// An error that can occure while importing a Yosys module
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum YosysModuleImportError {
    /// The module has an `inout` port
    InOutPort {
        /// The name of the `inout` port
        port_name: String,
    },
    /// The module contains a cell that has an `inout` port
    CellInOutPort {
        /// The name of the cell
        cell_name: String,
        /// The name of the `inout` port
        port_name: String,
    },
    /// The module or one of its cells has a port that is wider than `MAX_LOGIC_WIDTH`
    UnsupportedWireWidth {
        /// The width of the wire
        wire_width: usize,
    },
    /// The module contains a cell of unknown type
    UnknownCellType {
        /// The name of the cell
        cell_name: String,
        /// The unknown type of the cell
        cell_type: String,
    },
    /// The module contains a cell that has a port with no specified direction
    MissingCellPortDirection {
        /// The name of the cell
        cell_name: String,
        /// The name of the port
        port_name: String,
    },
    /// The module contains a cell with an invalid port configuration
    InvalidCellPorts {
        /// The name of the cell
        cell_name: String,
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
                            port_name: port_name.clone(),
                        })
                    }
                }
            }
        }

        for (cell_name, cell) in &self.module.cells {
            let mut input_ports = IndexMap::new();
            let mut output_ports = IndexMap::new();
            for (port_name, port_bits) in &cell.connections {
                let Some(port_direction) = cell.port_directions.get(port_name) else {
                    return Err(YosysModuleImportError::MissingCellPortDirection { cell_name: cell_name.clone(), port_name: port_name.clone() });
                };

                if let &[bit] = port_bits.as_slice() {
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
                    let wire_width = port_bits.len();
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

                            let wires: Vec<_> = port_bits
                                .iter()
                                .copied()
                                .map(|bit| get_wire_for_bit(bit, builder, &mut net_map))
                                .collect();

                            builder.add_merge(&wires, port_wire).unwrap();
                        }
                        PortDirection::Output => {
                            output_ports.insert(port_name.clone(), port_wire);

                            for (i, bit) in port_bits.iter().copied().enumerate() {
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

                    let input_ports: Vec<_> = input_ports.values().copied().collect();

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

                    builder
                        .$add_gate(input_ports[0], input_ports[1], output_ports[0])
                        .map_err(|_| YosysModuleImportError::InvalidCellPorts {
                            cell_name: cell_name.clone(),
                        })?;
                }};
            }

            // https://yosyshq.readthedocs.io/projects/yosys/en/latest/CHAPTER_CellLib.html
            match cell.cell_type.as_str() {
                "$not" => unary_gate_cell!(add_not_gate),
                "$reduce_and" => unary_gate_cell!(add_horizontal_and_gate),
                "$reduce_or" | "$reduce_bool" => unary_gate_cell!(add_horizontal_or_gate),
                "$and" => binary_gate_cell!(add_and_gate),
                "$or" => binary_gate_cell!(add_or_gate),
                "$xor" => binary_gate_cell!(add_xor_gate),
                "$xnor" => binary_gate_cell!(add_xnor_gate),
                "$shl" | "$sshl" => binary_op_cell!(add_left_shift),
                "$shr" => binary_op_cell!(add_logical_right_shift),
                "$sshr" => binary_op_cell!(add_arithmetic_right_shift),
                "$add" => binary_op_cell!(add_add),
                "$sub" => binary_op_cell!(add_sub),
                "$mul" => binary_op_cell!(add_mul),
                "$div" => binary_op_cell!(add_div),
                "$mod" => binary_op_cell!(add_rem),
                "$mux" => {
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
                "$dff" => {
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
                "$dffe" => {
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
                cell_type => {
                    return Err(YosysModuleImportError::UnknownCellType {
                        cell_name: cell_name.clone(),
                        cell_type: cell_type.to_owned(),
                    })
                }
            }
        }

        Ok(connections)
    }
}
