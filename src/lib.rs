#![feature(const_trait_impl)]
#![feature(box_into_inner)]

pub mod components;
pub mod ffi;

use ahash::{AHashMap, AHashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationError {
    Conflict,
    InvalidComponentId,
    InvalidWireId,
    InvalidOutputIndex,
}

pub type SimulationResult<T> = Result<T, SimulationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WireId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum LogicState {
    HighZ = 0,
    Undefined = 1,
    Logic0 = 2,
    Logic1 = 3,
}
impl LogicState {
    pub const fn to_char(self) -> char {
        match self {
            Self::Undefined => 'X',
            Self::HighZ => 'Z',
            Self::Logic0 => '0',
            Self::Logic1 => '1',
        }
    }

    #[inline]
    const fn is_valid(v: u32) -> bool {
        v <= 3
    }
}
impl const Default for LogicState {
    #[inline]
    fn default() -> Self {
        Self::HighZ
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum OutputStrength {
    Weak = 0,
    Strong = 1,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Output {
    state: Vec<LogicState>,
    pub strength: OutputStrength,
}
impl Output {
    #[inline]
    pub fn state(&self) -> &[LogicState] {
        &self.state
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut [LogicState] {
        &mut self.state
    }
}

struct Input {
    wires: Vec<Option<WireId>>,
}

pub trait ComponentBehaviour: Send + Sync {
    fn output_configuration(&self) -> &[u32];
    fn input_configuration(&self) -> &[u32];

    fn update(
        &mut self,
        outputs: &mut [Output],
        inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool>;
}

pub struct Component {
    outputs: Box<[Output]>,
    inputs: Box<[Input]>,
    input_values: Box<[Box<[LogicState]>]>,
    behaviour: Box<dyn ComponentBehaviour>,
}
impl Component {
    pub fn new(behaviour: Box<dyn ComponentBehaviour>) -> Self {
        let output_config = behaviour.output_configuration();
        let mut outputs = Vec::with_capacity(output_config.len());
        for width in output_config.iter().copied() {
            outputs.push(Output {
                state: vec![LogicState::Undefined; width as usize],
                strength: OutputStrength::Weak,
            })
        }

        let input_config = behaviour.input_configuration();
        let mut inputs = Vec::with_capacity(input_config.len());
        let mut input_values = Vec::with_capacity(input_config.len());
        for width in input_config.iter().copied() {
            inputs.push(Input {
                wires: vec![None; width as usize],
            });

            input_values.push(vec![LogicState::HighZ; width as usize].into_boxed_slice());
        }

        Self {
            outputs: outputs.into_boxed_slice(),
            inputs: inputs.into_boxed_slice(),
            input_values: input_values.into_boxed_slice(),
            behaviour,
        }
    }

    pub fn connect_input(&mut self, input_index: u32, wires: &[WireId]) {
        let input = &mut self.inputs[input_index as usize];

        for (i, wire) in input.wires.iter_mut().enumerate() {
            let in_wire = wires.get(i).copied();
            *wire = in_wire;
        }
    }

    pub fn disconnect_input(&mut self, input_index: u32) {
        let input = &mut self.inputs[input_index as usize];

        for wire in input.wires.iter_mut() {
            *wire = None;
        }
    }

    fn update(&mut self, wires: &AHashMap<WireId, Wire>) -> SimulationResult<bool> {
        for (i, input) in self.inputs.iter().enumerate() {
            let values = &mut self.input_values[i];

            for (w, wire) in input.wires.iter().enumerate() {
                let value = if let Some(wire_id) = wire {
                    if let Some(wire) = wires.get(&wire_id) {
                        wire.state
                    } else {
                        return Err(SimulationError::InvalidWireId);
                    }
                } else {
                    LogicState::HighZ
                };

                values[w] = value;
            }
        }

        self.behaviour.update(&mut self.outputs, &self.input_values)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Driver {
    component: ComponentId,
    output_index: (u32, u32),
}

pub struct Wire {
    state: LogicState,
    drivers: AHashSet<Driver>,
}
impl Wire {
    #[inline]
    pub fn new() -> Self {
        Self {
            state: LogicState::default(),
            drivers: AHashSet::new(),
        }
    }

    #[inline]
    pub const fn state(&self) -> LogicState {
        self.state
    }

    #[inline]
    pub fn add_driver(&mut self, component: ComponentId, output_index: (u32, u32)) -> bool {
        self.drivers.insert(Driver {
            component,
            output_index,
        })
    }

    #[inline]
    pub fn remove_driver(&mut self, component: ComponentId, output_index: (u32, u32)) -> bool {
        self.drivers.remove(&Driver {
            component,
            output_index,
        })
    }

    fn update(&mut self, components: &AHashMap<ComponentId, Component>) -> SimulationResult<bool> {
        let mut new_state = LogicState::HighZ;
        let mut strength = OutputStrength::Weak;

        fn combine(
            lhs: (LogicState, OutputStrength),
            rhs: (LogicState, OutputStrength),
        ) -> SimulationResult<(LogicState, OutputStrength)> {
            if lhs.0 == LogicState::HighZ {
                Ok(rhs)
            } else if rhs.0 == LogicState::HighZ {
                Ok(lhs)
            } else {
                match (lhs.1, rhs.1) {
                    (OutputStrength::Strong, OutputStrength::Strong) => {
                        Err(SimulationError::Conflict)
                    }
                    (OutputStrength::Strong, OutputStrength::Weak) => Ok(lhs),
                    (OutputStrength::Weak, OutputStrength::Strong) => Ok(rhs),
                    (OutputStrength::Weak, OutputStrength::Weak) => {
                        if lhs.0 == rhs.0 {
                            Ok((lhs.0, OutputStrength::Weak))
                        } else {
                            Ok((LogicState::Undefined, OutputStrength::Weak))
                        }
                    }
                }
            }
        }

        for driver in self.drivers.iter() {
            let component = components
                .get(&driver.component)
                .ok_or(SimulationError::InvalidComponentId)?;

            let output = component
                .outputs
                .get(driver.output_index.0 as usize)
                .ok_or(SimulationError::InvalidOutputIndex)?;

            let state = output
                .state()
                .get(driver.output_index.1 as usize)
                .ok_or(SimulationError::InvalidOutputIndex)?;

            (new_state, strength) = combine((new_state, strength), (*state, output.strength))?;
        }

        let changed = new_state != self.state;
        self.state = new_state;
        Ok(changed)
    }
}

pub struct Simulator {
    next_component_id: ComponentId,
    components: AHashMap<ComponentId, Component>,
    next_wire_id: WireId,
    wires: AHashMap<WireId, Wire>,
}
impl Simulator {
    pub fn new() -> Self {
        Self {
            next_component_id: ComponentId(0),
            components: AHashMap::new(),
            next_wire_id: WireId(0),
            wires: AHashMap::new(),
        }
    }

    pub fn add_component(&mut self, component: Component) -> ComponentId {
        let id = self.next_component_id;
        self.next_component_id.0 += 1;

        self.components.insert(id, component);
        id
    }

    #[inline]
    pub fn remove_component(&mut self, id: ComponentId) -> Option<Component> {
        self.components.remove(&id)
    }

    #[inline]
    pub fn get_component(&self, id: ComponentId) -> Option<&Component> {
        self.components.get(&id)
    }

    #[inline]
    pub fn get_component_mut(&mut self, id: ComponentId) -> Option<&mut Component> {
        self.components.get_mut(&id)
    }

    pub fn add_wire(&mut self, wire: Wire) -> WireId {
        let id = self.next_wire_id;
        self.next_wire_id.0 += 1;

        self.wires.insert(id, wire);
        id
    }

    #[inline]
    pub fn remove_wire(&mut self, id: WireId) -> Option<Wire> {
        self.wires.remove(&id)
    }

    #[inline]
    pub fn get_wire(&self, id: WireId) -> Option<&Wire> {
        self.wires.get(&id)
    }

    #[inline]
    pub fn get_wire_mut(&mut self, id: WireId) -> Option<&mut Wire> {
        self.wires.get_mut(&id)
    }

    pub fn step(&mut self) -> SimulationResult<bool> {
        use rayon::prelude::*;

        fn reduce_result(
            a: SimulationResult<bool>,
            b: SimulationResult<bool>,
        ) -> SimulationResult<bool> {
            match a {
                Ok(a_changed) => match b {
                    Ok(b_changed) => Ok(a_changed | b_changed),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            }
        }

        let components = &self.components;
        let wires_changed = self
            .wires
            .par_iter_mut()
            .map(|(_, wire)| wire.update(components))
            .reduce(|| Ok(false), reduce_result)?;

        let wires = &self.wires;
        let components_changed = self
            .components
            .par_iter_mut()
            .map(|(_, component)| component.update(wires))
            .reduce(|| Ok(false), reduce_result)?;

        Ok(wires_changed | components_changed)
    }
}
