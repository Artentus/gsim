//! High speed digital logic simulation
//!
//! ### Example
//! ```
//! use gsim::*;
//! use std::num::NonZeroU8;
//!
//! let mut builder = SimulatorBuilder::default();
//!
//! // Add wires and components to the simulation
//! let wire_width = bit_width!(1);
//! let input_a = builder.add_wire(wire_width).unwrap();
//! let input_b = builder.add_wire(wire_width).unwrap();
//! let output = builder.add_wire(wire_width).unwrap();
//! // The gate ID is not usefull to us because we don't intend on reading its data
//! let _gate = builder.add_and_gate(&[input_a, input_b], output).unwrap();
//!
//! // Create the simulation
//! let mut sim = builder.build();
//!
//! // Manually drive the input wires
//! sim.set_wire_drive(input_a, &LogicState::from_bool(true)).unwrap();
//! sim.set_wire_drive(input_b, &LogicState::from_bool(false)).unwrap();
//!
//! // Run the simulation
//! const MAX_STEPS: u64 = 2;
//! match sim.run_sim(MAX_STEPS) {
//!     SimulationRunResult::Ok => {}
//!     SimulationRunResult::MaxStepsReached => panic!("simulation did not settle within allowed steps"),
//!     SimulationRunResult::Err(err) => panic!("simulation error: {err:?}"),
//! }
//!
//! // Make sure we got the expected result
//! let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();
//! assert_eq!(output_state, LogicState::from_bool(false));
//! ```

// TODO: remove
#![feature(debug_closure_helpers)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::too_many_arguments)]

#[macro_use]
extern crate static_assertions;

mod component;
mod id;
pub mod import;
mod logic;
mod wire;

//#[cfg(feature = "c-api")]
//mod ffi;

//#[cfg(feature = "python-bindings")]
//mod python_bindings;

#[cfg(test)]
mod test;

use component::*;
use id::*;
use smallvec::SmallVec;
use std::fmt;
use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};
use wire::*;

pub use component::ComponentId;
pub use logic::*;
pub use wire::WireId;

#[allow(dead_code)]
type HashMap<K, V> = ahash::AHashMap<K, V>;
#[allow(dead_code)]
type HashSet<T> = ahash::AHashSet<T>;

const fn const_max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

trait InlineCount {
    const INLINE_COUNT: usize;
}

impl<T> InlineCount for T {
    const INLINE_COUNT: usize = const_max(
        std::mem::size_of::<[usize; 2]>() / std::mem::size_of::<T>(),
        1,
    );
}

macro_rules! inline_vec {
    ($t:ty) => {
        smallvec::SmallVec<[$t; <$t as $crate::InlineCount>::INLINE_COUNT]>
    };
    ($t:ty; $n:ty) => {
        smallvec::SmallVec<[$t; <$n as $crate::InlineCount>::INLINE_COUNT]>
    };
}

use inline_vec;

trait SafeDivCeil<Rhs = Self> {
    type Output;

    fn safe_div_ceil(self, rhs: Rhs) -> Self::Output;
}

impl SafeDivCeil for NonZeroU8 {
    type Output = Self;

    #[inline]
    fn safe_div_ceil(self, rhs: Self) -> Self::Output {
        unsafe {
            // SAFETY:
            //   - `self` is not zero, so ceiling division will always be > 0
            //   - `rhs` is not zero, so division by zero cannot occurr
            Self::new_unchecked(self.get().div_ceil(rhs.get()))
        }
    }
}

trait CLog2 {
    type Output;

    fn clog2(self) -> Self::Output;
}

impl CLog2 for NonZeroU8 {
    type Output = u8;

    #[inline]
    fn clog2(self) -> Self::Output {
        (self.ilog2() as u8) + ((!self.is_power_of_two()) as u8)
    }
}

impl CLog2 for usize {
    type Output = u32;

    #[inline]
    fn clog2(self) -> Self::Output {
        self.ilog2() + ((!self.is_power_of_two()) as u32)
    }
}

/// The size of a memory allocation
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct AllocationSize(usize);

impl std::ops::Add for AllocationSize {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for AllocationSize {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::iter::Sum for AllocationSize {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(AllocationSize(0), std::ops::Add::add)
    }
}

impl std::fmt::Display for AllocationSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
        const UNIT_STEP: f64 = 1024.0;

        let mut unit = 0;
        let mut amount_in_unit = self.0 as f64;
        while (unit < UNITS.len()) && (amount_in_unit >= UNIT_STEP) {
            unit += 1;
            amount_in_unit /= UNIT_STEP;
        }

        amount_in_unit = (amount_in_unit * 10.0).round() * 0.1;

        if amount_in_unit.fract().abs() <= f64::EPSILON {
            write!(f, "{:.0} {}", amount_in_unit, UNITS[unit])
        } else {
            write!(f, "{:.1} {}", amount_in_unit, UNITS[unit])
        }
    }
}

/// Memory usage statistics of a simulation
#[derive(Debug)]
pub struct SimulationStats {
    /// The number of wires in the simulation
    pub wire_count: usize,
    /// The size of the allocation storing wires
    pub wire_alloc_size: AllocationSize,
    /// The size of the allocation storing wire widths
    pub wire_width_alloc_size: AllocationSize,
    /// The size of the allocation storing wire drives
    pub wire_drive_alloc_size: AllocationSize,
    /// The size of the allocation storing wire states
    pub wire_state_alloc_size: AllocationSize,

    /// The number of components stored inline in the simulation
    pub small_component_count: usize,
    /// The number of components stored out-of-line in the simulation
    pub large_component_count: usize,
    /// The size of the allocation storing components
    pub component_alloc_size: AllocationSize,
    /// The size of out-of-line components
    pub large_component_alloc_size: AllocationSize,
    /// The size of the allocation storing output widths
    pub output_width_alloc_size: AllocationSize,
    /// The size of the allocation storing output states
    pub output_state_alloc_size: AllocationSize,
}

/// Contains data of all errors that occurred in a simulation
#[derive(Debug, Clone)]
pub struct SimulationErrors {
    /// A list of wires that had multiple drivers
    pub conflicts: Box<[WireId]>,
}

/// The result of a single simulation step
#[derive(Debug, Clone)]
#[must_use]
enum SimulationStepResult {
    /// The simulation did not change during this update
    Unchanged,
    /// The simulation changed during this update
    Changed,
    /// The update caused an error in the simulation
    Err(SimulationErrors),
}

/// The result of running a simulation
#[derive(Debug, Clone)]
#[must_use]
pub enum SimulationRunResult {
    /// The simulation settled
    Ok,
    /// The simulation did not settle within the maximum allowed steps
    MaxStepsReached,
    /// The simulation produced an error
    Err(SimulationErrors),
}

impl SimulationRunResult {
    /// Panics if the value is not `Ok`
    #[inline(never)]
    #[track_caller]
    pub fn unwrap(self) {
        match self {
            SimulationRunResult::Ok => (),
            SimulationRunResult::MaxStepsReached => panic!(
                "called `unwrap()` on a `MaxStepsReached` value: simulation exceeded allowed steps"
            ),
            SimulationRunResult::Err(_) => {
                panic!("called `unwrap()` on an `Err` value: driver conflict occurred")
            }
        }
    }
}

/// Errors that can occur when adding a wire to a simulator
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AddWireError {
    /// The memory limit for wires was reached
    TooManyWires,
}

impl From<OutOfMemoryError> for AddWireError {
    #[inline]
    fn from(_: OutOfMemoryError) -> Self {
        Self::TooManyWires
    }
}

impl From<CapacityOverflowError> for AddWireError {
    #[inline]
    fn from(_: CapacityOverflowError) -> Self {
        Self::TooManyWires
    }
}

impl fmt::Display for AddWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            AddWireError::TooManyWires => "the memory limit for wires was reached",
        };

        write!(f, "{msg}")
    }
}

impl std::error::Error for AddWireError {}

/// Errors that can occur when adding a component to a simulator
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AddComponentError {
    /// The memory limit for components was reached
    TooManyComponents,
    /// A specified wire ID was not part of the simulation
    InvalidWireId,
    /// Two or more wires that were expected to did not have the same width
    WireWidthMismatch,
    /// One or more wires had a width incompatible with the component
    WireWidthIncompatible,
    /// A specified offset was outside the range of its corresponding wire's width
    OffsetOutOfRange,
    /// Too few inputs were specified
    TooFewInputs,
    /// The number of inputs was not valid for the component
    InvalidInputCount,
}

impl From<InvalidWireIdError> for AddComponentError {
    #[inline]
    fn from(_: InvalidWireIdError) -> Self {
        AddComponentError::InvalidWireId
    }
}

impl From<OutOfMemoryError> for AddComponentError {
    #[inline]
    fn from(_: OutOfMemoryError) -> Self {
        Self::TooManyComponents
    }
}

impl fmt::Display for AddComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            AddComponentError::TooManyComponents => "the memory limit for components was reached",
            AddComponentError::InvalidWireId => {
                "a specified wire ID was not part of the simulation"
            }
            AddComponentError::WireWidthMismatch => {
                "two or more wires that were expected to did not have the same width"
            }
            AddComponentError::WireWidthIncompatible => {
                "one or more wires had a width incompatible with the component"
            }
            AddComponentError::OffsetOutOfRange => {
                "a specified offset was outside the range of its corresponding wire's width"
            }
            AddComponentError::TooFewInputs => "too few inputs were specified",
            AddComponentError::InvalidInputCount => {
                "the number of inputs was not valid for the component"
            }
        };

        write!(f, "{msg}")
    }
}

impl std::error::Error for AddComponentError {}

/// Errors that can occurr when setting a wires drive
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SetWireDriveError {
    /// The specified wire ID was not part of the simulation
    InvalidWireId,
    /// The width of the logic state did not match the width of the wire
    InvalidBitWidth,
}

impl From<InvalidWireIdError> for SetWireDriveError {
    #[inline]
    fn from(_: InvalidWireIdError) -> Self {
        SetWireDriveError::InvalidWireId
    }
}

impl fmt::Display for SetWireDriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            SetWireDriveError::InvalidWireId => {
                "the specified wire ID was not part of the simulation"
            }
            SetWireDriveError::InvalidBitWidth => {
                "the width of the logic state did not match the width of the wire"
            }
        };

        write!(f, "{msg}")
    }
}

impl std::error::Error for SetWireDriveError {}

/// The specified wire ID was not part of the simulation
#[derive(Debug, Clone)]
pub struct InvalidWireIdError;

impl fmt::Display for InvalidWireIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the specified wire ID was not part of the simulation")
    }
}

impl std::error::Error for InvalidWireIdError {}

/// The result of adding a wire to a simulator
pub type AddWireResult = Result<WireId, AddWireError>;

/// A specified component ID was not part of the simulation
#[derive(Debug, Clone)]
pub struct InvalidComponentIdError;

impl fmt::Display for InvalidComponentIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the specified component ID was not part of the simulation"
        )
    }
}

impl std::error::Error for InvalidComponentIdError {}

/// The result of adding a component to a simulator
pub type AddComponentResult = Result<ComponentId, AddComponentError>;

struct SimulatorData {
    wires: WireList,
    wire_states: WireStateAllocator,

    components: ComponentStorage,
    output_states: OutputStateAllocator,

    wire_update_queue: Vec<WireId>,
    component_update_queue: Vec<ComponentId>,

    wire_names: HashMap<WireId, Arc<str>>,
    component_names: HashMap<ComponentId, Arc<str>>,
}

impl SimulatorData {
    #[inline]
    fn new() -> Self {
        Self {
            wires: WireList::new(),
            wire_states: WireStateAllocator::new(),

            components: ComponentStorage::default(),
            output_states: OutputStateAllocator::new(),

            wire_update_queue: Vec::new(),
            component_update_queue: Vec::new(),

            wire_names: HashMap::new(),
            component_names: HashMap::new(),
        }
    }

    #[inline]
    fn iter_wire_ids(&self) -> impl Iterator<Item = WireId> + '_ {
        self.wires.ids()
    }

    #[inline]
    fn iter_component_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.components.ids()
    }

    fn set_wire_drive<'a>(
        &mut self,
        wire: WireId,
        new_drive: impl IntoLogicStateRef<'a>,
    ) -> Result<(), SetWireDriveError> {
        let wire = self
            .wires
            .get(wire)
            .ok_or(SetWireDriveError::InvalidWireId)?;
        let [_, mut drive] = self
            .wire_states
            .get_mut(wire.state_id(), wire.bit_width())
            .expect("invalid wire state ID");

        let new_drive = new_drive.into_logic_state_ref();
        if new_drive.bit_width() != wire.bit_width() {
            return Err(SetWireDriveError::InvalidBitWidth);
        }

        let (src_plane_0, src_plane_1) = new_drive.bit_planes();
        let (dst_plane_0, dst_plane_1) = drive.bit_planes_mut();
        dst_plane_0.copy_from_slice(src_plane_0);
        dst_plane_1.copy_from_slice(src_plane_1);

        Ok(())
    }

    fn get_wire_state_and_drive(
        &self,
        wire: WireId,
    ) -> Result<[LogicStateRef; 2], InvalidWireIdError> {
        let wire = self.wires.get(wire).ok_or(InvalidWireIdError)?;
        Ok(self
            .wire_states
            .get(wire.state_id(), wire.bit_width())
            .expect("invalid wire state ID"))
    }

    //fn get_component_data(
    //    &self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Immutable>, InvalidComponentIdError> {
    //    self.components
    //        .get(component)
    //        .map(Component::get_data)
    //        .ok_or(InvalidComponentIdError)
    //}

    //fn get_component_data_mut(
    //    &mut self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Mutable>, InvalidComponentIdError> {
    //    self.components
    //        .get_mut(component)
    //        .map(Component::get_data_mut)
    //        .ok_or(InvalidComponentIdError)
    //}

    fn set_wire_name<S: Into<Arc<str>>>(
        &mut self,
        wire: WireId,
        name: S,
    ) -> Result<(), InvalidWireIdError> {
        if self.wires.get(wire).is_none() {
            return Err(InvalidWireIdError);
        }

        self.wire_names.insert(wire, name.into());
        Ok(())
    }

    fn get_wire_name(&self, wire: WireId) -> Result<Option<&str>, InvalidWireIdError> {
        if self.wires.get(wire).is_none() {
            return Err(InvalidWireIdError);
        }

        Ok(self.wire_names.get(&wire).map(|name| &**name))
    }

    fn set_component_name<S: Into<Arc<str>>>(
        &mut self,
        component: ComponentId,
        name: S,
    ) -> Result<(), InvalidComponentIdError> {
        if self.components.component_exists(component) {
            return Err(InvalidComponentIdError);
        }

        self.component_names.insert(component, name.into());
        Ok(())
    }

    fn get_component_name(
        &self,
        component: ComponentId,
    ) -> Result<Option<&str>, InvalidComponentIdError> {
        if self.components.component_exists(component) {
            return Err(InvalidComponentIdError);
        }

        Ok(self.component_names.get(&component).map(|name| &**name))
    }

    fn stats(&self) -> SimulationStats {
        todo!()
        //    let (small_component_count, large_component_count) = self.components.component_counts();

        //    SimulationStats {
        //        wire_count: self.wires.wire_count(),
        //        wire_alloc_size: self.wires.alloc_size(),
        //        wire_width_alloc_size: self.wire_states.width_alloc_size(),
        //        wire_drive_alloc_size: self.wire_states.drive_alloc_size(),
        //        wire_state_alloc_size: self.wire_states.state_alloc_size(),
        //        small_component_count,
        //        large_component_count,
        //        component_alloc_size: self.components.alloc_size(),
        //        large_component_alloc_size: self.components.large_alloc_size(),
        //        output_width_alloc_size: self.output_states.width_alloc_size(),
        //        output_state_alloc_size: self.output_states.state_alloc_size(),
        //    }
    }

    #[cfg(feature = "dot-export")]
    fn write_dot<W: std::io::Write>(
        &self,
        mut writer: W,
        show_states: bool,
    ) -> std::io::Result<()> {
        todo!()
        //    writeln!(writer, "digraph {{")?;

        //    let mut wire_state_map = HashMap::new();
        //    for wire_id in self.wires.ids() {
        //        let wire = &self.wires.get(wire_id).unwrap();
        //        let width = self.wire_states.get_width(wire.state);
        //        wire_state_map.insert(wire.state, wire_id);

        //        #[allow(clippy::collapsible_else_if)]
        //        if show_states {
        //            if let Some(name) = self.wire_names.get(&wire_id) {
        //                let state = self.get_wire_state(wire_id).unwrap().display_string(width);
        //                if &**name == state.as_str() {
        //                    // Don't print constant wire states twice
        //                    writeln!(
        //                        writer,
        //                        "    W{}[label=\"{}\" shape=\"diamond\"];",
        //                        wire_id.to_bits(),
        //                        name,
        //                    )?;
        //                } else {
        //                    writeln!(
        //                        writer,
        //                        "    W{}[label=\"{} ({})\" shape=\"diamond\"];",
        //                        wire_id.to_bits(),
        //                        name,
        //                        state,
        //                    )?;
        //                }
        //            } else {
        //                writeln!(
        //                    writer,
        //                    "    W{}[label=\"{}\" shape=\"diamond\"];",
        //                    wire_id.to_bits(),
        //                    self.get_wire_state(wire_id).unwrap().display_string(width),
        //                )?;
        //            }
        //        } else {
        //            if let Some(name) = self.wire_names.get(&wire_id) {
        //                writeln!(
        //                    writer,
        //                    "    W{}[label=\"{} [{}]\" shape=\"diamond\"];",
        //                    wire_id.to_bits(),
        //                    name,
        //                    width,
        //                )?;
        //            } else {
        //                writeln!(
        //                    writer,
        //                    "    W{}[label=\"[{}]\" shape=\"diamond\"];",
        //                    wire_id.to_bits(),
        //                    width,
        //                )?;
        //            }
        //        }
        //    }

        //    let mut wire_drivers = ahash::AHashMap::<WireId, Vec<_>>::new();
        //    let mut wire_driving = ahash::AHashMap::<WireId, Vec<_>>::new();
        //    for component_id in self.components.ids() {
        //        let component = &self.components.get(component_id).unwrap();
        //        for (wire_id, port_name) in component.output_wires() {
        //            wire_drivers
        //                .entry(wire_id)
        //                .or_default()
        //                .push((component_id, port_name));
        //        }
        //        for (wire_id, port_name) in component.input_wires() {
        //            wire_driving
        //                .entry(wire_state_map[&wire_id])
        //                .or_default()
        //                .push((component_id, port_name));
        //        }

        //        let name = self
        //            .component_names
        //            .get(&component_id)
        //            .map(|name| (&**name).into())
        //            .unwrap_or_else(|| component.node_name(&self.output_states));

        //        'print: {
        //            if show_states {
        //                let data = self.get_component_data(component_id).unwrap();
        //                if let ComponentData::RegisterValue(value) = data {
        //                    writeln!(
        //                        writer,
        //                        "    C{}[label=\"{} ({})\" shape=\"box\"];",
        //                        component_id.to_bits(),
        //                        name,
        //                        value.read().display_string(value.width()),
        //                    )?;

        //                    break 'print;
        //                }
        //            }

        //            writeln!(
        //                writer,
        //                "    C{}[label=\"{}\" shape=\"box\"];",
        //                component_id.to_bits(),
        //                name,
        //            )?;
        //        }
        //    }

        //    for wire_id in self.wires.ids() {
        //        if let Some(drivers) = wire_drivers.get(&wire_id) {
        //            for (driver, port_name) in drivers {
        //                writeln!(
        //                    writer,
        //                    "    C{} -> W{}[taillabel=\"{}\"];",
        //                    driver.to_bits(),
        //                    wire_id.to_bits(),
        //                    port_name,
        //                )?;
        //            }
        //        }

        //        if let Some(driving) = wire_driving.get(&wire_id) {
        //            for (driving, port_name) in driving {
        //                writeln!(
        //                    writer,
        //                    "    W{} -> C{}[headlabel=\"{}\"];",
        //                    wire_id.to_bits(),
        //                    driving.to_bits(),
        //                    port_name,
        //                )?;
        //            }
        //        }
        //    }

        //    writeln!(writer, "}}")
    }
}

/// A digital circuit simulator
///
/// See crate level documentation for a usage example
#[allow(missing_debug_implementations)]
pub struct Simulator<VCD: std::io::Write = std::io::Sink> {
    data: SimulatorData,
    #[allow(dead_code)]
    vcd: VCD,
}

impl<VCD: std::io::Write> Simulator<VCD> {
    /// Iterates over all wire IDs in the graph
    #[inline]
    pub fn iter_wire_ids(&self) -> impl Iterator<Item = WireId> + '_ {
        self.data.iter_wire_ids()
    }

    /// Iterates over all component IDs in the graph
    #[inline]
    pub fn iter_component_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.data.iter_component_ids()
    }

    /// Drives a wire to a certain state without needing a component
    ///
    /// Any unspecified bits will be set to Z
    #[inline]
    pub fn set_wire_drive<'a>(
        &mut self,
        wire: WireId,
        new_drive: impl IntoLogicStateRef<'a>,
    ) -> Result<(), SetWireDriveError> {
        self.data.set_wire_drive(wire, new_drive)
    }

    /// Gets the current state of a wire
    #[inline]
    pub fn get_wire_state_and_drive(
        &self,
        wire: WireId,
    ) -> Result<[LogicStateRef; 2], InvalidWireIdError> {
        self.data.get_wire_state_and_drive(wire)
    }

    ///// Gets a components data
    //#[inline]
    //pub fn get_component_data(
    //    &self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Immutable>, InvalidComponentIdError> {
    //    self.data.get_component_data(component)
    //}

    ///// Gets a components data mutably
    //#[inline]
    //pub fn get_component_data_mut(
    //    &mut self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Mutable>, InvalidComponentIdError> {
    //    self.data.get_component_data_mut(component)
    //}

    /// Gets the name of a wire, if one has been assigned
    #[inline]
    pub fn get_wire_name(&self, wire: WireId) -> Result<Option<&str>, InvalidWireIdError> {
        self.data.get_wire_name(wire)
    }

    /// Gets the name of a component, if one has been assigned
    #[inline]
    pub fn get_component_name(
        &self,
        component: ComponentId,
    ) -> Result<Option<&str>, InvalidComponentIdError> {
        self.data.get_component_name(component)
    }

    /// Collects statistics of the simulation
    #[inline]
    pub fn stats(&self) -> SimulationStats {
        self.data.stats()
    }

    /// Writes the simulation graph into a Graphviz DOT file
    #[cfg(feature = "dot-export")]
    #[inline]
    pub fn write_dot<W: std::io::Write>(
        &self,
        writer: W,
        show_states: bool,
    ) -> std::io::Result<()> {
        self.data.write_dot(writer, show_states)
    }
}

/*

Simulation algorithm:

    The circuit graph is divided into two distinct subsets: wires and components.
    Wires are only connected to components and components are only connected to wires,
    meaning there are no edges in the graph that connect two nodes of the same subset.
    We alternate between updating wires and components to make use of this property:
    all wires as well as all components are updated in parallel since they have no
    dependencies amongst themselves.
    To avoid updating all nodes in each step, the previous step of the opposite subset
    builds an update queue. If the next queue is empty, we are done.

*/
impl<VCD: std::io::Write> Simulator<VCD> {
    fn update_wires(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.data.component_update_queue.clear();

        let conflicts = Mutex::new(Vec::new());

        let perform = |wire_id| {
            let wire = unsafe {
                // SAFETY: `sort_unstable` + `dedup` ensure the ID is unique between all iterations
                self.data.wires.get_unsafe(wire_id)
            };

            let states = unsafe {
                // SAFETY: since the wire is unique, so is its state
                self.data.wire_states.range_unsafe(
                    wire.state_id(),
                    wire.state_id(),
                    wire.bit_width(),
                )
            };

            match wire.update(states, self.data.output_states.view()) {
                WireUpdateResult::Unchanged => [].as_slice(),
                WireUpdateResult::Changed => wire.driving(),
                WireUpdateResult::Conflict => {
                    // Locking here is ok because we are in the error path
                    let mut conflict_list = conflicts.lock().expect("failed to aquire mutex");
                    conflict_list.push(wire_id);

                    [].as_slice()
                }
            }
        };

        let component_update_queue_iter = self
            .data
            .wire_update_queue
            .par_iter()
            .with_min_len(200)
            .copied()
            .flat_map_iter(perform);

        self.data
            .component_update_queue
            .par_extend(component_update_queue_iter);

        // Make sure the component update queue contains no duplicates,
        // otherwise all our safety guarantees do not hold.
        self.data.component_update_queue.par_sort_unstable();
        self.data.component_update_queue.dedup();

        let conflicts = conflicts
            .into_inner()
            .expect("failed to aquire mutex")
            .into_boxed_slice();

        if !conflicts.is_empty() {
            SimulationStepResult::Err(SimulationErrors { conflicts })
        } else if self.data.component_update_queue.is_empty() {
            SimulationStepResult::Unchanged
        } else {
            SimulationStepResult::Changed
        }
    }

    fn update_components(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.data.wire_update_queue.clear();

        let perform = |component_id| {
            unsafe {
                // SAFETY: `sort_unstable` + `dedup` ensure the ID is unique between all iterations
                self.data.components.update_component(
                    component_id,
                    self.data.wire_states.view(),
                    &self.data.output_states,
                )
            }
        };

        let wire_update_queue_iter = self
            .data
            .component_update_queue
            .par_iter()
            .with_min_len(200)
            .copied()
            .flat_map_iter(perform);

        self.data
            .wire_update_queue
            .par_extend(wire_update_queue_iter);

        // Make sure the wire update queue contains no duplicates,
        // otherwise all our safety guarantees do not hold.
        self.data.wire_update_queue.par_sort_unstable();
        self.data.wire_update_queue.dedup();

        if self.data.wire_update_queue.is_empty() {
            SimulationStepResult::Unchanged
        } else {
            SimulationStepResult::Changed
        }
    }

    /// Resets the simulation
    pub fn reset(&mut self) {
        self.data.wire_states.clear_states();
        self.data.output_states.clear_states();

        self.data.components.reset_components();
    }

    fn begin_sim(&mut self) -> SimulationStepResult {
        // We have to perform the first update step on all nodes in the graph,
        // so we insert all IDs into the queues.

        self.data.wire_update_queue.clear();
        self.data.wire_update_queue.extend(self.data.wires.ids());
        if let SimulationStepResult::Err(err) = self.update_wires() {
            return SimulationStepResult::Err(err);
        }

        self.data.component_update_queue.clear();
        self.data
            .component_update_queue
            .extend(self.data.components.ids());
        self.update_components()
    }

    fn step_sim(&mut self) -> SimulationStepResult {
        match self.update_wires() {
            SimulationStepResult::Unchanged => SimulationStepResult::Unchanged,
            SimulationStepResult::Changed => self.update_components(),
            SimulationStepResult::Err(err) => SimulationStepResult::Err(err),
        }
    }

    /// Runs the simulation until it settles, but at most for `max_steps` steps
    pub fn run_sim(&mut self, max_steps: u64) -> SimulationRunResult {
        let mut steps = 0;
        let mut result = self.begin_sim();
        loop {
            match result {
                SimulationStepResult::Unchanged => return SimulationRunResult::Ok,
                SimulationStepResult::Changed => {
                    if steps > max_steps {
                        return SimulationRunResult::MaxStepsReached;
                    }

                    steps += 1;
                    result = self.step_sim();
                }
                SimulationStepResult::Err(err) => return SimulationRunResult::Err(err),
            }
        }
    }
}

/// Defines the polarity of a clock signal
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockPolarity {
    /// The clock will trigger on a rising edge
    #[default]
    Rising = 1,
    /// The clock will trigger on a falling edge
    Falling = 0,
}

impl ClockPolarity {
    #[inline]
    const fn active_state(self) -> bool {
        match self {
            ClockPolarity::Rising => true,
            ClockPolarity::Falling => false,
        }
    }

    #[inline]
    const fn inactive_state(self) -> bool {
        match self {
            ClockPolarity::Rising => false,
            ClockPolarity::Falling => true,
        }
    }
}

/// Builds a simulator
///
/// See crate level documentation for a usage example
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub struct SimulatorBuilder {
    data: SimulatorData,
}

impl Default for SimulatorBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            data: SimulatorData::new(),
        }
    }
}

impl SimulatorBuilder {
    /// Iterates over all wire IDs in the graph
    #[inline]
    pub fn iter_wire_ids(&self) -> impl Iterator<Item = WireId> + '_ {
        self.data.iter_wire_ids()
    }

    /// Iterates over all component IDs in the graph
    #[inline]
    pub fn iter_component_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.data.iter_component_ids()
    }

    /// Drives a wire to a certain state without needing a component
    ///
    /// Any unspecified bits will be set to Z
    #[inline]
    pub fn set_wire_drive<'a>(
        &mut self,
        wire: WireId,
        new_drive: impl IntoLogicStateRef<'a>,
    ) -> Result<(), SetWireDriveError> {
        self.data.set_wire_drive(wire, new_drive)
    }

    /// Gets the current drive of a wire
    #[inline]
    pub fn get_wire_drive(&self, wire: WireId) -> Result<LogicStateRef, InvalidWireIdError> {
        self.data
            .get_wire_state_and_drive(wire)
            .map(|[_, drive]| drive)
    }

    ///// Gets a components data
    //#[inline]
    //pub fn get_component_data(
    //    &self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Immutable>, InvalidComponentIdError> {
    //    self.data.get_component_data(component)
    //}

    ///// Gets a components data mutably
    //#[inline]
    //pub fn get_component_data_mut(
    //    &mut self,
    //    component: ComponentId,
    //) -> Result<ComponentData<'_, Mutable>, InvalidComponentIdError> {
    //    self.data.get_component_data_mut(component)
    //}

    /// Assigns a name to a wire
    #[inline]
    pub fn set_wire_name<S: Into<Arc<str>>>(
        &mut self,
        wire: WireId,
        name: S,
    ) -> Result<(), InvalidWireIdError> {
        self.data.set_wire_name(wire, name)
    }

    /// Gets the name of a wire, if one has been assigned
    #[inline]
    pub fn get_wire_name(&self, wire: WireId) -> Result<Option<&str>, InvalidWireIdError> {
        self.data.get_wire_name(wire)
    }

    /// Assigns a name to a component
    #[inline]
    pub fn set_component_name<S: Into<Arc<str>>>(
        &mut self,
        component: ComponentId,
        name: S,
    ) -> Result<(), InvalidComponentIdError> {
        self.data.set_component_name(component, name)
    }

    /// Gets the name of a component, if one has been assigned
    #[inline]
    pub fn get_component_name(
        &self,
        component: ComponentId,
    ) -> Result<Option<&str>, InvalidComponentIdError> {
        self.data.get_component_name(component)
    }

    /// Collects statistics of the simulation
    #[inline]
    pub fn stats(&self) -> SimulationStats {
        self.data.stats()
    }

    /// Writes the simulation graph into a Graphviz DOT file
    #[cfg(feature = "dot-export")]
    #[inline]
    pub fn write_dot<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        self.data.write_dot(writer, false)
    }
}

macro_rules! def_add_binary_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(
            &mut self,
            input_a: WireId,
            input_b: WireId,
            output: WireId,
        ) -> AddComponentResult {
            let width = self.check_wire_widths_match(&[input_a, input_b, output])?;

            let output_state = self.data
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = self.data.wires.get(input_a).ok_or(AddComponentError::InvalidWireId)?;
            let wire_b = self.data.wires.get(input_b).ok_or(AddComponentError::InvalidWireId)?;
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id)?;
            self.mark_driving(input_b, id)?;
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

macro_rules! def_add_unary_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, input: WireId, output: WireId) -> AddComponentResult {
            let width = self.check_wire_widths_match(&[input, output])?;

            let output_state = self.data
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire = self.data.wires.get(input).ok_or(AddComponentError::InvalidWireId)?;
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input: wire.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input, id)?;
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

macro_rules! def_add_wide_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident, $wide_gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, inputs: &[WireId], output: WireId) -> AddComponentResult {
            if inputs.len() < 2 {
                return Err(AddComponentError::TooFewInputs);
            }

            let width = self.check_wire_widths_match(inputs)?;
            self.check_wire_widths_match(&[inputs[0], output])?;

            let output_state = self.data
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let id = if inputs.len() == 2 {
                let wire_a = self.data.wires.get(inputs[0]).ok_or(AddComponentError::InvalidWireId)?;
                let wire_b = self.data.wires.get(inputs[1]).ok_or(AddComponentError::InvalidWireId)?;
                let gate = SmallComponent::new(SmallComponentKind::$gate {
                    input_a: wire_a.state,
                    input_b: wire_b.state,
                }, output);
                self.add_small_component(gate, &[output_state])
            } else {
                let inputs: SmallVec<_> = inputs
                    .iter()
                    .map(|&input| self.data.wires.get(input).map(|wire| wire.state))
                    .collect::<Option<_>>().ok_or(AddComponentError::InvalidWireId)?;
                let gate = $wide_gate::new(inputs, output_state, output);
                self.add_large_component(gate, &[output_state])
            }?;

            for &input in inputs {
                self.mark_driving(input, id)?;
            }
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

macro_rules! def_add_shifter {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(
            &mut self,
            input_a: WireId,
            input_b: WireId,
            output: WireId,
        ) -> AddComponentResult {
            let width = self.check_wire_widths_match(&[input_a, output])?;
            let Some(shamnt_width) = NonZeroU8::new(width.clog2()) else {
                return Err(AddComponentError::WireWidthIncompatible);
            };
            self.check_wire_width_eq(input_b, shamnt_width)?;

            let output_state = self.data
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = self.data.wires.get(input_a).ok_or(AddComponentError::InvalidWireId)?;
            let wire_b = self.data.wires.get(input_b).ok_or(AddComponentError::InvalidWireId)?;
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id)?;
            self.mark_driving(input_b, id)?;
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

macro_rules! def_add_horizontal_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, input: WireId, output: WireId) -> AddComponentResult {
            self.check_wire_width_eq(output, NonZeroU8::MIN)?;

            let output_state = self.data
                .output_states
                .push(NonZeroU8::MIN)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire = self.data.wires.get(input).ok_or(AddComponentError::InvalidWireId)?;
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input: wire.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input, id)?;
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

macro_rules! def_add_comparator {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(
            &mut self,
            input_a: WireId,
            input_b: WireId,
            output: WireId,
        ) -> AddComponentResult {
            let width = self.check_wire_widths_match(&[input_a, input_b])?;
            self.check_wire_width_eq(output, NonZeroU8::MIN)?;

            let output_state = self.data
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = self.data.wires.get(input_a).ok_or(AddComponentError::InvalidWireId)?;
            let wire_b = self.data.wires.get(input_b).ok_or(AddComponentError::InvalidWireId)?;
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id)?;
            self.mark_driving(input_b, id)?;
            self.mark_driver(output, output_state)?;

            Ok(id)
        }
    };
}

impl SimulatorBuilder {
    /// Adds a wire to the simulation
    ///
    /// Returns `None` if the memory limit for wires has been reached
    pub fn add_wire(&mut self, bit_width: BitWidth) -> AddWireResult {
        let state_id = self.data.wire_states.alloc(bit_width)?;
        let wire = Wire::new(bit_width, state_id);
        let id = self.data.wires.push(wire)?;
        Ok(id)
    }

    #[inline]
    fn add_component<T: ComponentAuto>(
        &mut self,
        args: T::Args<'_>,
    ) -> Result<ComponentId, AddComponentError> {
        let component = T::new(args, &mut self.data.wires, &mut self.data.output_states)?;
        if let Some(id) = self.data.components.push(component) {
            args.connect_drivers(id, &mut self.data.wires)?;
            Ok(id)
        } else {
            Err(AddComponentError::TooManyComponents)
        }
    }

    /// Adds an `AND Gate` component to the simulation
    pub fn add_and_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<AndGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideAndGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds an `OR Gate` component to the simulation
    pub fn add_or_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<OrGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideOrGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds an `XOR Gate` component to the simulation
    pub fn add_xor_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<XorGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideXorGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds a `NAND Gate` component to the simulation
    pub fn add_nand_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<NandGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideNandGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds a `NOR Gate` component to the simulation
    pub fn add_nor_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<NorGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideNorGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds an `XNOR Gate` component to the simulation
    pub fn add_xnor_gate(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        match inputs {
            &[input_a, input_b] => self.add_component::<XnorGate>(BinaryGateArgs {
                input_a,
                input_b,
                output,
            }),
            _ => self.add_component::<WideXnorGate>(WideGateArgs { inputs, output }),
        }
    }

    /// Adds a `NOT Gate` component to the simulation
    pub fn add_not_gate(
        &mut self,
        input: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<NotGate>(UnaryGateArgs { input, output })
    }

    /// Adds a `Buffer` component to the simulation
    pub fn add_buffer(
        &mut self,
        input: WireId,
        enable: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<Buffer>(BinaryGateArgs {
            input_a: input,
            input_b: enable,
            output,
        })
    }

    /// Adds an `ADD` component to the simulation
    pub fn add_add(
        &mut self,
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<Add>(BinaryGateArgs {
            input_a,
            input_b,
            output,
        })
    }

    /// Adds a `SUB` component to the simulation
    pub fn add_sub(
        &mut self,
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<Sub>(BinaryGateArgs {
            input_a,
            input_b,
            output,
        })
    }

    /// Adds a `NEG` component to the simulation
    pub fn add_neg(
        &mut self,
        input: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<Neg>(UnaryGateArgs { input, output })
    }

    /// Adds a `MUL` component to the simulation
    pub fn add_mul(
        &mut self,
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<Mul>(BinaryGateArgs {
            input_a,
            input_b,
            output,
        })
    }

    /// Adds a `Left Shift` component to the simulation
    pub fn add_left_shift(
        &mut self,
        input: WireId,
        shift_amount: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<LeftShift>(BinaryGateArgs {
            input_a: input,
            input_b: shift_amount,
            output,
        })
    }

    /// Adds a `Logical Right Shift` component to the simulation
    pub fn add_logical_right_shift(
        &mut self,
        input: WireId,
        shift_amount: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<LogicalRightShift>(BinaryGateArgs {
            input_a: input,
            input_b: shift_amount,
            output,
        })
    }

    /// Adds an `Arithmetic Right Shift` component to the simulation
    pub fn add_arithmetic_right_shift(
        &mut self,
        input: WireId,
        shift_amount: WireId,
        output: WireId,
    ) -> Result<ComponentId, AddComponentError> {
        self.add_component::<ArithmeticRightShift>(BinaryGateArgs {
            input_a: input,
            input_b: shift_amount,
            output,
        })
    }

    /*
    /// Adds a `Slice` component to the simulation
    pub fn add_slice(&mut self, input: WireId, offset: u8, output: WireId) -> AddComponentResult {
        let input_width = self.get_wire_width(input)?;
        let output_width = self.get_wire_width(output)?;

        if output_width > input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        if ((offset as usize) + (output_width.get() as usize)) > (input_width.get() as usize) {
            return Err(AddComponentError::OffsetOutOfRange);
        }

        let output_state = self
            .data
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = &self
            .data
            .wires
            .get(input)
            .ok_or(AddComponentError::InvalidWireId)?;
        let gate = SmallComponent::new(
            SmallComponentKind::Slice {
                input: wire.state,
                offset,
            },
            output,
        );
        let id = self.add_small_component(gate, &[output_state])?;

        self.mark_driving(input, id)?;
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds a `Merge` component to the simulation
    pub fn add_merge(&mut self, inputs: &[WireId], output: WireId) -> AddComponentResult {
        if inputs.is_empty() {
            return Err(AddComponentError::TooFewInputs);
        }

        let output_width = self.get_wire_width(output)?;
        let total_input_width = inputs
            .iter()
            .map(|&input| self.get_wire_width(input).map(NonZeroU8::get))
            .try_fold(0u8, |a, b| {
                a.checked_add(b?)
                    .ok_or(AddComponentError::WireWidthIncompatible)
            })?;
        if total_input_width != output_width.get() {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .data
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let input_states: SmallVec<_> = inputs
            .iter()
            .map(|&input| self.data.wires.get(input).map(|wire| wire.state))
            .collect::<Option<_>>()
            .ok_or(AddComponentError::InvalidWireId)?;
        let gate = Merge::new(input_states, output_state, output);
        let id = self.add_large_component(gate, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id)?;
        }
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds an `Adder` component to the simulation
    pub fn add_adder(
        &mut self,
        input_a: WireId,
        input_b: WireId,
        carry_in: WireId,
        output: WireId,
        carry_out: WireId,
    ) -> AddComponentResult {
        let width = self.check_wire_widths_match(&[input_a, input_b, output])?;
        self.check_wire_width_eq(carry_in, NonZeroU8::MIN)?;
        self.check_wire_width_eq(carry_out, NonZeroU8::MIN)?;

        let output_state = self
            .data
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let cout_state = self
            .data
            .output_states
            .push(NonZeroU8::MIN)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_a = self
            .data
            .wires
            .get(input_a)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_b = self
            .data
            .wires
            .get(input_b)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_cin = self
            .data
            .wires
            .get(carry_in)
            .ok_or(AddComponentError::InvalidWireId)?;
        let gate = Adder::new(
            wire_a.state,
            wire_b.state,
            wire_cin.state,
            output_state,
            output,
            cout_state,
            carry_out,
        );
        let id = self.add_large_component(gate, &[output_state, cout_state])?;

        self.mark_driving(input_a, id)?;
        self.mark_driving(input_b, id)?;
        self.mark_driving(carry_in, id)?;
        self.mark_driver(output, output_state)?;
        self.mark_driver(carry_out, cout_state)?;

        Ok(id)
    }

    /// Adds a `Multiplexer` component to the simulation
    pub fn add_multiplexer(
        &mut self,
        inputs: &[WireId],
        select: WireId,
        output: WireId,
    ) -> AddComponentResult {
        if !inputs.len().is_power_of_two() {
            return Err(AddComponentError::InvalidInputCount);
        }

        let expected_select_bits = inputs.len().ilog2();
        if expected_select_bits > (Atom::BITS.get() as u32) {
            return Err(AddComponentError::InvalidInputCount);
        }

        let select_width = self.get_wire_width(select)?;
        if (select_width.get() as u32) != expected_select_bits {
            return Err(AddComponentError::InvalidInputCount);
        }

        let width = self.check_wire_widths_match(inputs)?;
        self.check_wire_widths_match(&[inputs[0], output])?;

        let output_state = self
            .data
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wires: SmallVec<_> = inputs
            .iter()
            .map(|&input| self.data.wires.get(input).map(|wire| wire.state))
            .collect::<Option<_>>()
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_sel = self
            .data
            .wires
            .get(select)
            .ok_or(AddComponentError::InvalidWireId)?;
        let mux = Multiplexer::new(wires, wire_sel.state, output_state, output);
        let id = self.add_large_component(mux, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id)?;
        }
        self.mark_driving(select, id)?;
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds a `Priority Decoder` component to the simulation
    pub fn add_priority_decoder(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> AddComponentResult {
        if inputs.is_empty() {
            return Err(AddComponentError::TooFewInputs);
        }

        for &input in inputs {
            self.check_wire_width_eq(input, NonZeroU8::MIN)?;
        }

        let output_width = self.get_wire_width(output)?;
        let expected_width = (inputs.len() + 1).clog2();
        if (output_width.get() as u32) != expected_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .data
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wires: SmallVec<_> = inputs
            .iter()
            .map(|&input| self.data.wires.get(input).map(|wire| wire.state))
            .collect::<Option<_>>()
            .ok_or(AddComponentError::InvalidWireId)?;
        let decoder = PriorityDecoder::new(wires, output_state, output);
        let id = self.add_large_component(decoder, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id)?;
        }
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds a `Register` component to the simulation
    pub fn add_register(
        &mut self,
        data_in: WireId,
        data_out: WireId,
        enable: WireId,
        clock: WireId,
        clock_polarity: ClockPolarity,
    ) -> AddComponentResult {
        let width = self.check_wire_widths_match(&[data_in, data_out])?;
        self.check_wire_width_eq(enable, NonZeroU8::MIN)?;
        self.check_wire_width_eq(clock, NonZeroU8::MIN)?;

        let output_state = self
            .data
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_din = self
            .data
            .wires
            .get(data_in)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_en = self
            .data
            .wires
            .get(enable)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_clk = self
            .data
            .wires
            .get(clock)
            .ok_or(AddComponentError::InvalidWireId)?;
        let register = Register::new(
            width,
            wire_din.state,
            output_state,
            data_out,
            wire_en.state,
            wire_clk.state,
            clock_polarity,
        );
        let id = self.add_large_component(register, &[output_state])?;

        self.mark_driving(data_in, id)?;
        self.mark_driving(enable, id)?;
        self.mark_driving(clock, id)?;
        self.mark_driver(data_out, output_state)?;

        Ok(id)
    }

    def_add_horizontal_gate!(
        /// Adds a `Horizontal AND Gate` component to the simulation
        add_horizontal_and_gate,
        HorizontalAnd
    );

    def_add_horizontal_gate!(
        /// Adds a `Horizontal OR Gate` component to the simulation
        add_horizontal_or_gate,
        HorizontalOr
    );

    def_add_horizontal_gate!(
        /// Adds a `Horizontal XOR Gate` component to the simulation
        add_horizontal_xor_gate,
        HorizontalXor
    );

    def_add_horizontal_gate!(
        /// Adds a `Horizontal NAND Gate` component to the simulation
        add_horizontal_nand_gate,
        HorizontalNand
    );

    def_add_horizontal_gate!(
        /// Adds a `Horizontal NOR Gate` component to the simulation
        add_horizontal_nor_gate,
        HorizontalNor
    );

    def_add_horizontal_gate!(
        /// Adds a `Horizontal XNOR Gate` component to the simulation
        add_horizontal_xnor_gate,
        HorizontalXnor
    );

    def_add_comparator!(
        /// Adds an equality comparator component to the simulation
        add_compare_equal,
        CompareEqual
    );

    def_add_comparator!(
        /// Adds an inequality comparator component to the simulation
        add_compare_not_equal,
        CompareNotEqual
    );

    def_add_comparator!(
        /// Adds a 'less than' comparator component to the simulation
        add_compare_less_than,
        CompareLessThan
    );

    def_add_comparator!(
        /// Adds a 'greater than' comparator component to the simulation
        add_compare_greater_than,
        CompareGreaterThan
    );

    def_add_comparator!(
        /// Adds a 'less than or equal' comparator component to the simulation
        add_compare_less_than_or_equal,
        CompareLessThanOrEqual
    );

    def_add_comparator!(
        /// Adds a 'greater than or equal' comparator component to the simulation
        add_compare_greater_than_or_equal,
        CompareGreaterThanOrEqual
    );

    def_add_comparator!(
        /// Adds a 'signed less than' comparator component to the simulation
        add_compare_less_than_signed,
        CompareLessThanSigned
    );

    def_add_comparator!(
        /// Adds a 'signed greater than' comparator component to the simulation
        add_compare_greater_than_signed,
        CompareGreaterThanSigned
    );

    def_add_comparator!(
        /// Adds a 'signed less than or equal' comparator component to the simulation
        add_compare_less_than_or_equal_signed,
        CompareLessThanOrEqualSigned
    );

    def_add_comparator!(
        /// Adds a 'signed greater than or equal' comparator component to the simulation
        add_compare_greater_than_or_equal_signed,
        CompareGreaterThanOrEqualSigned
    );

    /// Adds a `zero extension` component to the simulation
    pub fn add_zero_extend(&mut self, input: WireId, output: WireId) -> AddComponentResult {
        let input_width = self.get_wire_width(input)?;
        let output_width = self.get_wire_width(output)?;

        if output_width < input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .data
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = self
            .data
            .wires
            .get(input)
            .ok_or(AddComponentError::InvalidWireId)?;
        let extend =
            SmallComponent::new(SmallComponentKind::ZeroExtend { input: wire.state }, output);
        let id = self.add_small_component(extend, &[output_state])?;

        self.mark_driving(input, id)?;
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds a `sign extension` component to the simulation
    pub fn add_sign_extend(&mut self, input: WireId, output: WireId) -> AddComponentResult {
        let input_width = self.get_wire_width(input)?;
        let output_width = self.get_wire_width(output)?;

        if output_width < input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .data
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = self
            .data
            .wires
            .get(input)
            .ok_or(AddComponentError::InvalidWireId)?;
        let extend =
            SmallComponent::new(SmallComponentKind::SignExtend { input: wire.state }, output);
        let id = self.add_small_component(extend, &[output_state])?;

        self.mark_driving(input, id)?;
        self.mark_driver(output, output_state)?;

        Ok(id)
    }

    /// Adds a `RAM` component to the simulation
    pub fn add_ram(
        &mut self,
        write_addr: WireId,
        data_in: WireId,
        read_addr: WireId,
        data_out: WireId,
        write: WireId,
        clock: WireId,
        clock_polarity: ClockPolarity,
    ) -> AddComponentResult {
        let addr_width = self.check_wire_widths_match(&[write_addr, read_addr])?;
        let data_width = self.check_wire_widths_match(&[data_in, data_out])?;
        self.check_wire_width_eq(write, NonZeroU8::MIN)?;
        self.check_wire_width_eq(clock, NonZeroU8::MIN)?;

        let output_state = self
            .data
            .output_states
            .push(data_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_waddr = self
            .data
            .wires
            .get(write_addr)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_din = self
            .data
            .wires
            .get(data_in)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_raddr = self
            .data
            .wires
            .get(read_addr)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_w = self
            .data
            .wires
            .get(write)
            .ok_or(AddComponentError::InvalidWireId)?;
        let wire_clk = self
            .data
            .wires
            .get(clock)
            .ok_or(AddComponentError::InvalidWireId)?;
        let ram = Ram::new(
            wire_waddr.state,
            wire_din.state,
            wire_raddr.state,
            output_state,
            data_out,
            wire_w.state,
            wire_clk.state,
            clock_polarity,
            addr_width,
            data_width,
        );
        let id = self.add_large_component(ram, &[output_state])?;

        self.mark_driving(write_addr, id)?;
        self.mark_driving(read_addr, id)?;
        self.mark_driving(data_in, id)?;
        self.mark_driving(write, id)?;
        self.mark_driving(clock, id)?;
        self.mark_driver(data_out, output_state)?;

        Ok(id)
    }

    /// Adds a `ROM` component to the simulation
    pub fn add_rom(&mut self, addr: WireId, data: WireId) -> AddComponentResult {
        let addr_width = self.get_wire_width(addr)?;
        let data_width = self.get_wire_width(data)?;

        let output_state = self
            .data
            .output_states
            .push(data_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_addr = self
            .data
            .wires
            .get(addr)
            .ok_or(AddComponentError::InvalidWireId)?;
        let rom = Rom::new(wire_addr.state, output_state, data, addr_width, data_width);
        let id = self.add_large_component(rom, &[output_state])?;

        self.mark_driving(addr, id)?;
        self.mark_driver(data, output_state)?;

        Ok(id)
    }
    */

    /// Imports a module into this circuit
    #[inline]
    pub fn import_module<T: import::ModuleImporter>(
        &mut self,
        importer: &T,
    ) -> Result<import::ModuleConnections, T::Error> {
        importer.import_into(self)
    }

    /// Creates the simulator
    #[inline]
    pub fn build(self) -> Simulator {
        let mut sim = Simulator {
            data: self.data,
            vcd: std::io::sink(),
        };

        sim.reset();
        sim
    }
}

assert_impl_all!(SimulatorBuilder: Send);
assert_impl_all!(Simulator: Send);

//#[cfg(feature = "tracing")]
//mod tracing;
//#[cfg(feature = "tracing")]
//pub use tracing::Timescale;
//
//#[cfg(feature = "tracing")]
//impl SimulatorBuilder {
//    /// Creates the simulator and attaches VCD tracing
//    pub fn build_with_trace<VCD: std::io::Write>(
//        mut self,
//        mut vcd: VCD,
//        timescale: Timescale,
//    ) -> std::io::Result<Simulator<VCD>> {
//        self.data.wires.shrink_to_fit();
//        self.data.wire_states.shrink_to_fit();
//
//        self.data.components.shrink_to_fit();
//        self.data.output_states.shrink_to_fit();
//
//        tracing::write_vcd_header(&self.data, &mut vcd, timescale)?;
//
//        Ok(Simulator {
//            data: self.data,
//            vcd,
//        })
//    }
//}
//
//#[cfg(feature = "tracing")]
//impl<VCD: std::io::Write> Simulator<VCD> {
//    /// Traces the current state of the simulation
//    pub fn trace(&mut self, time: u64) -> std::io::Result<()> {
//        tracing::trace_vcd(&self.data, &mut self.vcd, time)
//    }
//}
