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
//! let wire_width = NonZeroU8::new(1).unwrap();
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
//! sim.set_wire_drive(input_a, &LogicState::from_bool(true));
//! sim.set_wire_drive(input_b, &LogicState::from_bool(false));
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
//! let output_state = sim.get_wire_state(output);
//! assert!(output_state.eq(&LogicState::from_bool(false), wire_width));
//! ```

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

#[macro_use]
extern crate static_assertions;

mod component;
mod id_lists;
mod id_vec;
pub mod import;
mod logic;

#[cfg(test)]
mod test;

use component::*;
use id_lists::*;
use id_vec::IdVec;
use itertools::izip;
use logic::*;
use smallvec::SmallVec;
use std::num::NonZeroU8;
use std::ops::{Add, AddAssign};
use std::sync::Mutex;

pub use component::ComponentData;
pub use id_lists::{ComponentId, Id, WireId};
pub use logic::{LogicBitState, LogicState};

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

impl Add for AllocationSize {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for AllocationSize {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::iter::Sum for AllocationSize {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(AllocationSize(0), Add::add)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireUpdateResult {
    Unchanged,
    Changed,
    Conflict,
}

struct Wire {
    state: WireStateId,
    drivers: IdVec<OutputStateId>,
    driving: IdVec<ComponentId>,
}

impl Wire {
    #[inline]
    fn new(state: WireStateId) -> Self {
        Self {
            state,
            drivers: IdVec::new(),
            driving: IdVec::new(),
        }
    }

    fn add_driving(&mut self, component: ComponentId) {
        // This is a linear search which may appear slow, but the list is usually very small so the overhead
        // of a hashset is not actually worth it.
        // In particular, the lookup only occurs while building the graph, whereas during simulation, when speed
        // is important, reading a vector is much faster than reading a hashset.
        if !self.driving.contains(component) {
            self.driving.push(component);
        }
    }

    #[inline]
    fn update(
        &self,
        width: NonZeroU8,
        drive: &[Atom],
        state: &mut [Atom],
        output_states: &OutputStateList,
    ) -> WireUpdateResult {
        // SAFETY:
        // These functions are on the hot path of the simulation,
        // so it is important to optimize them as much as possible.
        // Therefore in release mode we turn off all bounds checks
        // and assume our invariants hold. This is technically not
        // safe so proper testing in debug mode is required.

        #[cfg(not(debug_assertions))]
        macro_rules! get {
            ($slice:expr, $i:expr) => {
                unsafe { *$slice.get_unchecked($i) }
            };
        }

        #[cfg(debug_assertions)]
        macro_rules! get {
            ($slice:expr, $i:expr) => {
                $slice[$i]
            };
        }

        #[cfg(not(debug_assertions))]
        macro_rules! get_mut {
            ($slice:expr, $i:expr) => {
                unsafe { $slice.get_unchecked_mut($i) }
            };
        }

        #[cfg(debug_assertions)]
        macro_rules! get_mut {
            ($slice:expr, $i:expr) => {
                &mut $slice[$i]
            };
        }

        #[inline]
        fn combine(a: Atom, b: Atom) -> (Atom, LogicStorage) {
            //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning | conflict
            // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------|----------
            //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    0    |    0    | High-Z    | no
            //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined | no
            //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    0    |    1    | Logic 0   | no
            //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    1    | Logic 1   | no
            //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined | no
            //     1    |    0    | Undefined |    1    |    0    | Undefined |    -    |    -    | -         | yes
            //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    -    |    -    | -         | yes
            //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    -    |    -    | -         | yes
            //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    0    |    1    | Logic 0   | no
            //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    -    |    -    | -         | yes
            //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    -    |    -    | -         | yes
            //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    -    |    -    | -         | yes
            //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    1    | Logic 1   | no
            //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    -    |    -    | -         | yes
            //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    -    |    -    | -         | yes
            //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    -    |    -    | -         | yes

            let result = Atom {
                state: a.state | b.state,
                valid: a.valid | b.valid,
            };

            let conflict = {
                (a.state & b.state)
                    | (a.state & b.valid)
                    | (a.valid & b.state)
                    | (a.valid & b.valid)
            };

            (result, conflict)
        }

        const MAX_ATOM_COUNT: usize = NonZeroU8::MAX.get().div_ceil(Atom::BITS.get()) as usize;

        let mut tmp_state = [Atom::UNDEFINED; MAX_ATOM_COUNT];
        let tmp_state = get_mut!(tmp_state, ..drive.len());
        tmp_state.copy_from_slice(drive);

        let mut conflict = LogicStorage::ALL_ZERO;
        for driver in self.drivers.iter() {
            let driver = output_states.get_state(driver);
            debug_assert_eq!(drive.len(), driver.len());

            for (&driver, tmp_state) in izip!(driver, tmp_state.iter_mut()) {
                let (new_state, new_conflict) = combine(*tmp_state, driver);
                *tmp_state = new_state;
                conflict |= new_conflict;
            }
        }

        let mut changed = false;
        let mut i = 0;
        let mut total_width = width.get();
        while total_width >= Atom::BITS.get() {
            let state = get_mut!(state, i);
            let tmp_state = get!(tmp_state, i);

            if !state.eq(tmp_state, AtomWidth::MAX) {
                changed = true;
            }
            *state = tmp_state;

            i += 1;
            total_width -= Atom::BITS.get();
        }

        if total_width > 0 {
            let state = get_mut!(state, i);
            let tmp_state = get!(tmp_state, i);

            let last_width = unsafe {
                // SAFETY: the loop and if condition ensure that 0 < total_width < Atom::BITS
                AtomWidth::new_unchecked(total_width)
            };

            if !state.eq(tmp_state, last_width) {
                changed = true;
            }
            *state = tmp_state;
        }

        if conflict != LogicStorage::ALL_ZERO {
            WireUpdateResult::Conflict
        } else if changed {
            WireUpdateResult::Changed
        } else {
            WireUpdateResult::Unchanged
        }
    }
}

/// Contains data of all errors that occurred in a simulation
#[derive(Debug, Clone)]
pub struct SimulationErrors {
    /// A list of wires that had multiple drivers
    pub conflicts: Box<[WireId]>,
}

/// The result of a single simulation step
#[derive(Debug, Clone)]
pub enum SimulationStepResult {
    /// The simulation did not change during this update
    Unchanged,
    /// The simulation changed during this update
    Changed,
    /// The update caused an error in the simulation
    Err(SimulationErrors),
}

/// The result of running a simulation
#[derive(Debug, Clone)]
pub enum SimulationRunResult {
    /// The simulation settled
    Ok,
    /// The simulation did not settle within the maximum allowed steps
    MaxStepsReached,
    /// The simulation produced an error
    Err(SimulationErrors),
}

/// Errors that can occur when adding a component to a simulator
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AddComponentError {
    /// The memory limit for components was reached
    TooManyComponents,
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

/// The result of adding a component to a simulator
pub type AddComponentResult = Result<ComponentId, AddComponentError>;

/// A digital circuit simulator
///
/// See crate level documentation for a usage example
#[allow(missing_debug_implementations)]
pub struct Simulator {
    wires: WireList,
    wire_states: WireStateList,

    components: ComponentList,
    output_states: OutputStateList,

    wire_update_queue: Vec<WireId>,
    component_update_queue: Vec<ComponentId>,
}

impl Simulator {
    #[inline]
    fn new() -> Self {
        Self {
            wires: WireList::new(),
            wire_states: WireStateList::new(),

            components: ComponentList::new(),
            output_states: OutputStateList::new(),

            wire_update_queue: Vec::new(),
            component_update_queue: Vec::new(),
        }
    }

    /// Gets the width of a wire
    pub fn get_wire_width(&self, wire: WireId) -> NonZeroU8 {
        let wire = &self.wires[wire];
        self.wire_states.get_width(wire.state)
    }

    /// Drives a wire to a certain state without needing a component
    ///
    /// Any unspecified bits will be set to Z
    pub fn set_wire_drive(&mut self, wire: WireId, new_drive: &LogicState) {
        let wire = &self.wires[wire];
        let drive = self.wire_states.get_drive_mut(wire.state);

        for (dst, src) in drive.iter_mut().zip(new_drive.iter_atoms()) {
            *dst = src;
        }
    }

    /// Gets the current base drive of a wire
    pub fn get_wire_drive(&self, wire: WireId) -> LogicState {
        let wire = &self.wires[wire];
        let drive = self.wire_states.get_drive(wire.state);

        LogicState(LogicStateRepr::Bits(drive.iter().copied().collect()))
    }

    /// Gets the current state of a wire
    pub fn get_wire_state(&self, wire: WireId) -> LogicState {
        let wire = &self.wires[wire];
        let state = self.wire_states.get_state(wire.state);

        LogicState(LogicStateRepr::Bits(state.iter().copied().collect()))
    }

    /// Gets a components data
    pub fn get_component_data(&mut self, component: ComponentId) -> ComponentData<'_> {
        self.components[component].get_data()
    }

    /// Collects statistics of the simulation
    pub fn stats(&self) -> SimulationStats {
        let (small_component_count, large_component_count) = self.components.component_counts();

        SimulationStats {
            wire_count: self.wires.wire_count(),
            wire_alloc_size: self.wires.alloc_size(),
            wire_width_alloc_size: self.wire_states.width_alloc_size(),
            wire_drive_alloc_size: self.wire_states.drive_alloc_size(),
            wire_state_alloc_size: self.wire_states.state_alloc_size(),
            small_component_count,
            large_component_count,
            component_alloc_size: self.components.alloc_size(),
            large_component_alloc_size: self.components.large_alloc_size(),
            output_width_alloc_size: self.output_states.width_alloc_size(),
            output_state_alloc_size: self.output_states.state_alloc_size(),
        }
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
impl Simulator {
    fn update_wires(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.component_update_queue.clear();

        let conflicts = Mutex::new(Vec::new());

        let perform = |wire_id| {
            let wire: &Wire = &self.wires[wire_id];
            let (width, drive, state) = unsafe {
                // SAFETY: `sort_unstable` + `dedup` ensure the ID is unique between all iterations
                self.wire_states.get_data_unsafe(wire.state)
            };

            match wire.update(width, drive, state, &self.output_states) {
                WireUpdateResult::Unchanged => [].as_slice(),
                WireUpdateResult::Changed => wire.driving.as_slice(),
                WireUpdateResult::Conflict => {
                    // Locking here is ok because we are in the error path
                    let mut conflict_list = conflicts.lock().expect("failed to aquire mutex");
                    conflict_list.push(wire_id);

                    [].as_slice()
                }
            }
        };

        if self.wire_update_queue.len() > 400 {
            let component_update_queue_iter = self
                .wire_update_queue
                .par_iter()
                .with_min_len(200)
                .copied()
                .flat_map_iter(perform);

            self.component_update_queue
                .par_extend(component_update_queue_iter);
        } else {
            let component_update_queue_iter =
                self.wire_update_queue.iter().copied().flat_map(perform);

            self.component_update_queue
                .extend(component_update_queue_iter);
        }

        // Make sure the component update queue contains no duplicates,
        // otherwise all our safety guarantees do not hold.
        self.component_update_queue.par_sort_unstable();
        self.component_update_queue.dedup();

        let conflicts = conflicts
            .into_inner()
            .expect("failed to aquire mutex")
            .into_boxed_slice();

        if !conflicts.is_empty() {
            SimulationStepResult::Err(SimulationErrors {
                conflicts: Vec::new().into_boxed_slice(),
            })
        } else if self.component_update_queue.is_empty() {
            SimulationStepResult::Unchanged
        } else {
            SimulationStepResult::Changed
        }
    }

    fn update_components(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.wire_update_queue.clear();

        let perform = |component_id| {
            let component = unsafe {
                // SAFETY: `sort_unstable` + `dedup` ensure the ID is unique between all iterations
                self.components.get_unsafe(component_id)
            };

            let (output_base, output_atom_count) = component.output_range();
            let output_states = unsafe {
                // SAFETY: since the component is unique, so are the outputs
                self.output_states
                    .get_slice_unsafe(output_base, output_atom_count)
            };

            // `Component::update` returns all the wires that need to be inserted into the next update queue.
            component.update(&self.wire_states, output_states)
        };

        if self.component_update_queue.len() > 400 {
            let wire_update_queue_iter = self
                .component_update_queue
                .par_iter()
                .with_min_len(200)
                .copied()
                .flat_map_iter(perform);

            self.wire_update_queue.par_extend(wire_update_queue_iter);
        } else {
            let wire_update_queue_iter = self
                .component_update_queue
                .iter()
                .copied()
                .flat_map(perform);

            self.wire_update_queue.extend(wire_update_queue_iter);
        }

        // Make sure the wire update queue contains no duplicates,
        // otherwise all our safety guarantees do not hold.
        self.wire_update_queue.par_sort_unstable();
        self.wire_update_queue.dedup();

        if self.wire_update_queue.is_empty() {
            SimulationStepResult::Unchanged
        } else {
            SimulationStepResult::Changed
        }
    }

    /// Resets the simulation
    pub fn reset(&mut self) {
        self.wire_states.clear_states();
        self.output_states.clear_states();

        for component in self.components.iter_mut() {
            component.reset();
        }
    }

    /// Begins simulating
    ///
    /// Must be called before any calls to `step_sim`
    pub fn begin_sim(&mut self) -> SimulationStepResult {
        // We have to perform the first update step on all nodes in the graph,
        // so we insert all IDs into the queues.

        self.wire_update_queue.clear();
        self.wire_update_queue.extend(self.wires.ids());
        if let SimulationStepResult::Err(err) = self.update_wires() {
            return SimulationStepResult::Err(err);
        }

        self.component_update_queue.clear();
        self.component_update_queue.extend(self.components.ids());
        self.update_components()
    }

    /// Performs one simulation step
    ///
    /// Must only be called after `begin_sim`
    pub fn step_sim(&mut self) -> SimulationStepResult {
        match self.update_wires() {
            SimulationStepResult::Unchanged => SimulationStepResult::Unchanged,
            SimulationStepResult::Changed => self.update_components(),
            SimulationStepResult::Err(err) => SimulationStepResult::Err(err),
        }
    }

    /// Runs the simulation until it settles, but at most for `max_steps` steps
    ///
    /// It is **not** necessary to call `begin_sim` before this function
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

            let output_state = self
                .sim
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = &self.sim.wires[input_a];
            let wire_b = &self.sim.wires[input_b];
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id);
            self.mark_driving(input_b, id);
            self.mark_driver(output, output_state);

            Ok(id)
        }
    };
}

macro_rules! def_add_unary_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, input: WireId, output: WireId) -> AddComponentResult {
            let width = self.check_wire_widths_match(&[input, output])?;

            let output_state = self
                .sim
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire = &self.sim.wires[input];
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input: wire.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input, id);
            self.mark_driver(output, output_state);

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

            let output_state = self
                .sim
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let id = if inputs.len() == 2 {
                let wire_a = &self.sim.wires[inputs[0]];
                let wire_b = &self.sim.wires[inputs[1]];
                let gate = SmallComponent::new(SmallComponentKind::$gate {
                    input_a: wire_a.state,
                    input_b: wire_b.state,
                }, output);
                self.add_small_component(gate, &[output_state])
            } else {
                let inputs: SmallVec<_> = inputs
                    .iter()
                    .map(|&input| self.sim.wires[input].state)
                    .collect();
                let gate = $wide_gate::new(inputs, output_state, output);
                self.add_large_component(gate, &[output_state])
            }?;

            for &input in inputs {
                self.mark_driving(input, id);
            }
            self.mark_driver(output, output_state);

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

            let output_state = self
                .sim
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = &self.sim.wires[input_a];
            let wire_b = &self.sim.wires[input_b];
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id);
            self.mark_driving(input_b, id);
            self.mark_driver(output, output_state);

            Ok(id)
        }
    };
}

macro_rules! def_add_horizontal_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, input: WireId, output: WireId) -> AddComponentResult {
            self.check_wire_width_eq(output, NonZeroU8::MIN)?;

            let output_state = self
                .sim
                .output_states
                .push(NonZeroU8::MIN)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire = &self.sim.wires[input];
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input: wire.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input, id);
            self.mark_driver(output, output_state);

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

            let output_state = self
                .sim
                .output_states
                .push(width)
                .ok_or(AddComponentError::TooManyComponents)?;

            let wire_a = &self.sim.wires[input_a];
            let wire_b = &self.sim.wires[input_b];
            let gate = SmallComponent::new(SmallComponentKind::$gate {
                input_a: wire_a.state,
                input_b: wire_b.state,
            }, output);
            let id = self.add_small_component(gate, &[output_state])?;

            self.mark_driving(input_a, id);
            self.mark_driving(input_b, id);
            self.mark_driver(output, output_state);

            Ok(id)
        }
    };
}

/// Defines the polarity of a clock signal
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockPolarity {
    /// The clock will trigger on a rising edge
    #[default]
    Rising,
    /// The clock will trigger on a falling edge
    Falling,
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
    sim: Simulator,
}

impl Default for SimulatorBuilder {
    fn default() -> Self {
        Self {
            sim: Simulator::new(),
        }
    }
}

impl SimulatorBuilder {
    /// Adds a wire to the simulation
    ///
    /// Returns `None` if the memory limit for wires has been reached
    pub fn add_wire(&mut self, width: NonZeroU8) -> Option<WireId> {
        let state_id = self.sim.wire_states.push(width)?;
        let wire = Wire::new(state_id);
        self.sim.wires.push(wire)
    }

    /// Gets the width of a wire
    #[inline]
    pub fn get_wire_width(&self, wire: WireId) -> NonZeroU8 {
        self.sim.get_wire_width(wire)
    }

    /// Drives a wire to a certain state without needing a component
    #[inline]
    pub fn set_wire_drive(&mut self, wire: WireId, new_drive: &LogicState) {
        self.sim.set_wire_drive(wire, new_drive)
    }

    /// Gets the current drive of a wire
    #[inline]
    pub fn get_wire_drive(&self, wire: WireId) -> LogicState {
        self.sim.get_wire_drive(wire)
    }

    /// Gets a components data
    #[inline]
    pub fn get_component_data(&mut self, component: ComponentId) -> ComponentData<'_> {
        self.sim.get_component_data(component)
    }

    /// Collects statistics of the simulation
    #[inline]
    pub fn stats(&self) -> SimulationStats {
        self.sim.stats()
    }

    #[inline]
    fn mark_driving(&mut self, wire: WireId, component: ComponentId) {
        let wire = &mut self.sim.wires[wire];
        wire.add_driving(component);
    }

    #[inline]
    fn mark_driver(&mut self, wire: WireId, output_state: OutputStateId) {
        let wire = &mut self.sim.wires[wire];
        wire.drivers.push(output_state);
    }

    fn check_wire_widths_match(&self, wires: &[WireId]) -> Result<NonZeroU8, AddComponentError> {
        let mut iter = wires.iter().copied();

        if let Some(first) = iter.next() {
            let first_width = self.get_wire_width(first);

            for wire in iter {
                let width = self.get_wire_width(wire);
                if width != first_width {
                    return Err(AddComponentError::WireWidthMismatch);
                }
            }

            Ok(first_width)
        } else {
            Err(AddComponentError::TooFewInputs)
        }
    }

    fn check_wire_width_eq(&self, wire: WireId, width: NonZeroU8) -> Result<(), AddComponentError> {
        let wire_width = self.get_wire_width(wire);
        if wire_width != width {
            Err(AddComponentError::WireWidthIncompatible)
        } else {
            Ok(())
        }
    }

    fn add_small_component(
        &mut self,
        component: SmallComponent,
        outputs: &[OutputStateId],
    ) -> Result<ComponentId, AddComponentError> {
        let output_atom_count = outputs
            .iter()
            .copied()
            .map(|id| self.sim.output_states.get_width(id))
            .map(|width| width.safe_div_ceil(Atom::BITS))
            .map(NonZeroU8::get)
            .map(u16::from)
            .try_fold(0, u16::checked_add)
            .expect("combined output width too large");

        let component = Component::new_small(
            component,
            outputs.get(0).copied().unwrap_or(OutputStateId::INVALID),
            output_atom_count,
        );

        self.sim
            .components
            .push(component)
            .ok_or(AddComponentError::TooManyComponents)
    }

    fn add_large_component<C: LargeComponent + 'static>(
        &mut self,
        component: C,
        outputs: &[OutputStateId],
    ) -> Result<ComponentId, AddComponentError> {
        let output_atom_count = outputs
            .iter()
            .copied()
            .map(|id| self.sim.output_states.get_width(id))
            .map(|width| width.safe_div_ceil(Atom::BITS))
            .map(NonZeroU8::get)
            .map(u16::from)
            .try_fold(0, u16::checked_add)
            .expect("combined output width too large");

        let component = Component::new_large(
            component,
            outputs.get(0).copied().unwrap_or(OutputStateId::INVALID),
            output_atom_count,
        );

        self.sim
            .components
            .push(component)
            .ok_or(AddComponentError::TooManyComponents)
    }

    def_add_wide_gate!(
        /// Adds an `AND Gate` component to the simulation
        add_and_gate,
        AndGate,
        WideAndGate
    );

    def_add_wide_gate!(
        /// Adds an `OR Gate` component to the simulation
        add_or_gate,
        OrGate,
        WideOrGate
    );

    def_add_wide_gate!(
        /// Adds an `XOR Gate` component to the simulation
        add_xor_gate,
        XorGate,
        WideXorGate
    );

    def_add_wide_gate!(
        /// Adds a `NAND Gate` component to the simulation
        add_nand_gate,
        NandGate,
        WideNandGate
    );

    def_add_wide_gate!(
        /// Adds a `NOR Gate` component to the simulation
        add_nor_gate,
        NorGate,
        WideNorGate
    );

    def_add_wide_gate!(
        /// Adds an `XNOR Gate` component to the simulation
        add_xnor_gate,
        XnorGate,
        WideXnorGate
    );

    def_add_binary_gate!(
        /// Adds an `ADD` component to the simulation
        add_add,
        Add
    );

    def_add_binary_gate!(
        /// Adds a `SUB` component to the simulation
        add_sub,
        Sub
    );

    def_add_shifter!(
        /// Adds a `Left Shift` component to the simulation
        add_left_shift,
        LeftShift
    );

    def_add_shifter!(
        /// Adds a `Logical Right Shift` component to the simulation
        add_logical_right_shift,
        LogicalRightShift
    );

    def_add_shifter!(
        /// Adds an `Arithmetic Right Shift` component to the simulation
        add_arithmetic_right_shift,
        ArithmeticRightShift
    );

    def_add_unary_gate!(
        /// Adds a `NOT Gate` component to the simulation
        add_not_gate,
        NotGate
    );

    /// Adds a `Buffer` component to the simulation
    pub fn add_buffer(
        &mut self,
        input: WireId,
        enable: WireId,
        output: WireId,
    ) -> AddComponentResult {
        let width = self.check_wire_widths_match(&[input, output])?;
        self.check_wire_width_eq(enable, NonZeroU8::MIN)?;

        let output_state = self
            .sim
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = &self.sim.wires[input];
        let wire_en = &self.sim.wires[enable];
        let gate = SmallComponent::new(
            SmallComponentKind::Buffer {
                input: wire.state,
                enable: wire_en.state,
            },
            output,
        );
        let id = self.add_small_component(gate, &[output_state])?;

        self.mark_driving(input, id);
        self.mark_driving(enable, id);
        self.mark_driver(output, output_state);

        Ok(id)
    }

    /// Adds a `Slice` component to the simulation
    pub fn add_slice(&mut self, input: WireId, offset: u8, output: WireId) -> AddComponentResult {
        let input_width = self.get_wire_width(input);
        let output_width = self.get_wire_width(output);

        if output_width > input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        if ((offset as usize) + (output_width.get() as usize)) > (input_width.get() as usize) {
            return Err(AddComponentError::OffsetOutOfRange);
        }

        let output_state = self
            .sim
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = &self.sim.wires[input];
        let gate = SmallComponent::new(
            SmallComponentKind::Slice {
                input: wire.state,
                offset,
            },
            output,
        );
        let id = self.add_small_component(gate, &[output_state])?;

        self.mark_driving(input, id);
        self.mark_driver(output, output_state);

        Ok(id)
    }

    /// Adds a `Merge` component to the simulation
    pub fn add_merge(&mut self, inputs: &[WireId], output: WireId) -> AddComponentResult {
        if inputs.len() < 1 {
            return Err(AddComponentError::TooFewInputs);
        }

        let output_width = self.get_wire_width(output);
        let total_input_width = inputs
            .iter()
            .map(|&input| self.get_wire_width(input))
            .map(NonZeroU8::get)
            .try_fold(0, u8::checked_add)
            .ok_or(AddComponentError::WireWidthIncompatible)?;
        if total_input_width != output_width.get() {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .sim
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let input_states: SmallVec<_> = inputs
            .iter()
            .map(|&input| self.sim.wires[input].state)
            .collect();
        let gate = Merge::new(input_states, output_state, output);
        let id = self.add_large_component(gate, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id);
        }
        self.mark_driver(output, output_state);

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
            .sim
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let cout_state = self
            .sim
            .output_states
            .push(NonZeroU8::MIN)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_a = &self.sim.wires[input_a];
        let wire_b = &self.sim.wires[input_b];
        let wire_cin = &self.sim.wires[carry_in];
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

        self.mark_driving(input_a, id);
        self.mark_driving(input_b, id);
        self.mark_driving(carry_in, id);
        self.mark_driver(output, output_state);
        self.mark_driver(carry_out, cout_state);

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

        let select_width = self.get_wire_width(select);
        if (select_width.get() as u32) != expected_select_bits {
            return Err(AddComponentError::InvalidInputCount);
        }

        let width = self.check_wire_widths_match(inputs)?;
        self.check_wire_widths_match(&[inputs[0], output])?;

        let output_state = self
            .sim
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wires: SmallVec<_> = inputs
            .iter()
            .map(|&wire| self.sim.wires[wire].state)
            .collect();
        let wire_sel = &self.sim.wires[select];
        let mux = Multiplexer::new(wires, wire_sel.state, output_state, output);
        let id = self.add_large_component(mux, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id);
        }
        self.mark_driving(select, id);
        self.mark_driver(output, output_state);

        Ok(id)
    }

    /// Adds a `Priority Decoder` component to the simulation
    pub fn add_priority_decoder(
        &mut self,
        inputs: &[WireId],
        output: WireId,
    ) -> AddComponentResult {
        if inputs.len() < 1 {
            return Err(AddComponentError::TooFewInputs);
        }

        for &input in inputs {
            self.check_wire_width_eq(input, NonZeroU8::MIN)?;
        }

        let output_width = self.get_wire_width(output);
        let expected_width = (inputs.len() + 1).clog2();
        if (output_width.get() as u32) != expected_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .sim
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wires: SmallVec<_> = inputs
            .iter()
            .map(|&input| self.sim.wires[input].state)
            .collect();
        let decoder = PriorityDecoder::new(wires, output_state, output);
        let id = self.add_large_component(decoder, &[output_state])?;

        for &input in inputs {
            self.mark_driving(input, id);
        }
        self.mark_driver(output, output_state);

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
            .sim
            .output_states
            .push(width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_din = &self.sim.wires[data_in];
        let wire_en = &self.sim.wires[enable];
        let wire_clk = &self.sim.wires[clock];
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

        self.mark_driving(data_in, id);
        self.mark_driving(enable, id);
        self.mark_driving(clock, id);
        self.mark_driver(data_out, output_state);

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
        let input_width = self.get_wire_width(input);
        let output_width = self.get_wire_width(output);

        if output_width < input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .sim
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = &self.sim.wires[input];
        let extend =
            SmallComponent::new(SmallComponentKind::ZeroExtend { input: wire.state }, output);
        let id = self.add_small_component(extend, &[output_state])?;

        self.mark_driving(input, id);
        self.mark_driver(output, output_state);

        Ok(id)
    }

    /// Adds a `sign extension` component to the simulation
    pub fn add_sign_extend(&mut self, input: WireId, output: WireId) -> AddComponentResult {
        let input_width = self.get_wire_width(input);
        let output_width = self.get_wire_width(output);

        if output_width < input_width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let output_state = self
            .sim
            .output_states
            .push(output_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire = &self.sim.wires[input];
        let extend =
            SmallComponent::new(SmallComponentKind::SignExtend { input: wire.state }, output);
        let id = self.add_small_component(extend, &[output_state])?;

        self.mark_driving(input, id);
        self.mark_driver(output, output_state);

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
            .sim
            .output_states
            .push(data_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_waddr = &self.sim.wires[write_addr];
        let wire_din = &self.sim.wires[data_in];
        let wire_raddr = &self.sim.wires[read_addr];
        let wire_w = &self.sim.wires[write];
        let wire_clk = &self.sim.wires[clock];
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

        self.mark_driving(write_addr, id);
        self.mark_driving(read_addr, id);
        self.mark_driving(data_in, id);
        self.mark_driving(write, id);
        self.mark_driving(clock, id);
        self.mark_driver(data_out, output_state);

        Ok(id)
    }

    /// Adds a `ROM` component to the simulation
    pub fn add_rom(&mut self, addr: WireId, data: WireId) -> AddComponentResult {
        let addr_width = self.get_wire_width(addr);
        let data_width = self.get_wire_width(data);

        let output_state = self
            .sim
            .output_states
            .push(data_width)
            .ok_or(AddComponentError::TooManyComponents)?;

        let wire_addr = &self.sim.wires[addr];
        let rom = Rom::new(wire_addr.state, output_state, data, addr_width, data_width);
        let id = self.add_large_component(rom, &[output_state])?;

        self.mark_driving(addr, id);
        self.mark_driver(data, output_state);

        Ok(id)
    }

    /// Imports a module into this circuit
    #[inline]
    pub fn import_module<T: import::ModuleImporter>(
        &mut self,
        importer: &T,
    ) -> Result<import::ModuleConnections, T::Error> {
        importer.import_into(self)
    }

    /// Creates the simulator
    pub fn build(mut self) -> Simulator {
        self.sim.wires.shrink_to_fit();
        self.sim.wire_states.shrink_to_fit();

        self.sim.components.shrink_to_fit();
        self.sim.output_states.shrink_to_fit();

        self.sim
    }
}
