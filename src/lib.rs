//! High speed digital logic simulation

#![feature(sync_unsafe_cell)]
#![feature(array_windows)]
#![feature(ptr_as_uninit)]
#![deny(missing_docs)]

#[macro_use]
extern crate static_assertions;

mod component;
use component::*;

mod logic;
pub use logic::*;

#[cfg(test)]
mod test;

#[doc(hidden)]
#[allow(missing_docs)]
pub mod ffi;

use smallvec::{smallvec, SmallVec};
use std::sync::Mutex;

macro_rules! def_id_type {
    ($(#[$attr:meta])* $ns:ident::$id_name:ident) => {
        #[allow(dead_code)]
        mod $ns {
            use std::marker::PhantomData;
            use std::num::NonZeroUsize;
            use std::ops::RangeInclusive;

            $(#[$attr])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct $id_name(NonZeroUsize);
            assert_eq_size!($id_name, usize);
            assert_eq_align!($id_name, usize);
            assert_eq_size!(Option<$id_name>, usize);
            assert_eq_align!(Option<$id_name>, usize);

            #[derive(Debug)]
            #[repr(transparent)]
            pub(crate) struct IdList<T> {
                list: Vec<T>,
            }

            impl<T> IdList<T> {
                #[inline]
                pub const fn new() -> Self {
                    Self { list: Vec::new() }
                }

                #[inline]
                pub fn insert(&mut self, item: T) -> $id_name {
                    self.list.push(item);

                    unsafe {
                        // SAFETY: the list contains at least one item now so its length is > 0
                        $id_name(NonZeroUsize::new_unchecked(self.list.len()))
                    }
                }

                #[inline]
                pub fn get(&self, id: $id_name) -> Option<&T> {
                    self.list.get(id.0.get() - 1)
                }

                #[inline]
                pub fn get_mut(&mut self, id: $id_name) -> Option<&mut T> {
                    self.list.get_mut(id.0.get() - 1)
                }

                #[inline]
                pub fn ids<'a>(&'a self) -> IdIter<'a> {
                    IdIter::new(self.list.len())
                }

                #[inline]
                pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
                    self.list.iter()
                }

                #[inline]
                pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, T> {
                    self.list.iter_mut()
                }
            }

            #[derive(Clone)]
            #[repr(transparent)]
            pub(crate) struct IdIter<'a> {
                range: RangeInclusive<usize>,
                _a: PhantomData<&'a ()>,
            }

            impl<'a> IdIter<'a> {
                #[inline]
                fn new(len: usize) -> Self {
                    Self {
                        range: 1..=len,
                        _a: PhantomData,
                    }
                }
            }

            impl<'a> Iterator for IdIter<'a> {
                type Item = $id_name;

                fn next(&mut self) -> Option<Self::Item> {
                    self.range.next().map(|i| unsafe {
                        // SAFETY: the range starts from 1
                        $id_name(NonZeroUsize::new_unchecked(i))
                    })
                }
            }
        }

        pub use $ns::$id_name;
    };
}

def_id_type!(
    /// A unique identifier for a wire inside a simulation
    wire_id::WireId
);

def_id_type!(
    /// A unique identifier for a component inside a simulation
    component_id::ComponentId
);

enum WireUpdateResult {
    Ok(LogicState),
    Conflict,
}

#[derive(Debug)]
struct Wire {
    width: LogicWidth,
    base_drive: LogicState,
    // TODO: test what inline size yields the best performance
    drivers: SmallVec<[usize; 1]>,
    driving: SmallVec<[ComponentId; 1]>,
}

impl Wire {
    #[inline]
    fn new(width: LogicWidth) -> Self {
        Self {
            width,
            base_drive: LogicState::HIGH_Z,
            drivers: smallvec![],
            driving: smallvec![],
        }
    }

    fn update(&self, component_outputs: &[LogicStateCell]) -> WireUpdateResult {
        #[inline]
        fn combine(a: LogicState, b: LogicState, mask: LogicStorage) -> WireUpdateResult {
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

            let conflict = {
                (a.state & b.state)
                    | (a.state & b.valid)
                    | (a.valid & b.state)
                    | (a.valid & b.valid)
            } & mask;

            if conflict == LogicStorage::ALL_ZERO {
                WireUpdateResult::Ok(LogicState {
                    state: (a.state | b.state) & mask,
                    valid: (a.valid | b.valid) & mask,
                })
            } else {
                WireUpdateResult::Conflict
            }
        }

        let mask = LogicStorage::mask(self.width);

        let mut new_state = LogicState {
            state: self.base_drive.state & mask,
            valid: self.base_drive.valid & mask,
        };

        for driver in self.drivers.iter().copied() {
            let output = component_outputs[driver].get();

            match combine(new_state, output, mask) {
                WireUpdateResult::Ok(state) => new_state = state,
                WireUpdateResult::Conflict => return WireUpdateResult::Conflict,
            }
        }

        WireUpdateResult::Ok(new_state)
    }
}

/// Contains data of all errors that occurred in a simulation
#[derive(Debug, Clone)]
pub struct SimulationErrors {
    /// A list of wires that had multiple drivers
    pub conflicts: Box<[WireId]>,
}

/// The result of a single simulation step
pub enum SimulationStepResult {
    /// The simulation did not change during this update
    Unchanged,
    /// The simulation changed during this update
    Changed,
    /// The update caused an error in the simulation
    Err(SimulationErrors),
}

/// The result of running a simulation
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
pub enum AddComponentError {
    /// Two or more wires that were expected to did not have the same width
    WireWidthMismatch,
    /// One or more wires had a width incompatible with the component
    WireWidthIncompatible,
    /// A specified offset was outside the range of its corresponding wire's width
    OffsetOutOfRange,
}

/// The result of adding a component to a simulator
pub type AddComponentResult = Result<ComponentId, AddComponentError>;

type WireList = wire_id::IdList<Wire>;
type WireStateList = wire_id::IdList<LogicStateCell>;

macro_rules! def_add_binary_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(
            &mut self,
            input_a: WireId,
            input_b: WireId,
            output: WireId,
        ) -> AddComponentResult {
            self.check_wire_widths_match(&[input_a, input_b, output])?;

            let gate = SmallComponent::$gate {
                input_a,
                input_b,
                output,
            };
            let (output_offset, id) = self.add_small_component(gate);

            let input_wire_a = self.wires.get_mut(input_a).unwrap();
            input_wire_a.driving.push(id);
            let input_wire_b = self.wires.get_mut(input_b).unwrap();
            input_wire_b.driving.push(id);
            let output_wire = self.wires.get_mut(output).unwrap();
            output_wire.drivers.push(output_offset);

            Ok(id)
        }
    };
}

macro_rules! def_add_unary_gate {
    ($(#[$attr:meta])* $name:ident, $gate:ident) => {
        $(#[$attr])*
        pub fn $name(&mut self, input: WireId, output: WireId) -> AddComponentResult {
            self.check_wire_widths_match(&[input, output])?;

            let gate = SmallComponent::$gate { input, output };
            let (output_offset, id) = self.add_small_component(gate);

            let input_wire = self.wires.get_mut(input).unwrap();
            input_wire.driving.push(id);
            let output_wire = self.wires.get_mut(output).unwrap();
            output_wire.drivers.push(output_offset);

            Ok(id)
        }
    };
}

/// A digital circuit simulator
pub struct Simulator {
    wires: WireList,
    wire_states: WireStateList,

    small_component_heap: Vec<SmallComponent>,

    components: component_id::IdList<Component>,
    component_outputs: Vec<LogicStateCell>,

    wire_update_queue: Vec<WireId>,
    component_update_queue: Vec<ComponentId>,
}

impl Default for Simulator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Simulator {
    /// Creates a new simulator
    pub fn new() -> Self {
        Self {
            wires: WireList::new(),
            wire_states: WireStateList::new(),

            small_component_heap: Vec::new(),

            components: component_id::IdList::new(),
            component_outputs: Vec::new(),

            wire_update_queue: Vec::new(),
            component_update_queue: Vec::new(),
        }
    }

    /// Adds a wire to the simulation
    pub fn add_wire(&mut self, width: LogicWidth) -> WireId {
        let wire_id = self.wires.insert(Wire::new(width));
        let state_id = self
            .wire_states
            .insert(LogicStateCell::new(LogicState::HIGH_Z));
        assert_eq!(wire_id, state_id);
        wire_id
    }

    /// Gets the width of a wire
    pub fn get_wire_width(&self, wire: WireId) -> LogicWidth {
        self.wires.get(wire).expect("invalid wire ID").width
    }

    /// Drives a wire to a certain state without needing a component
    pub fn set_wire_base_drive(&mut self, wire: WireId, drive: LogicState) {
        let wire = self.wires.get_mut(wire).expect("invalid wire ID");
        let mask = LogicStorage::mask(wire.width);

        wire.base_drive = LogicState {
            state: drive.state & mask,
            valid: drive.valid & mask,
        };
    }

    /// Gets the current base drive of a wire
    pub fn get_wire_base_drive(&self, wire: WireId) -> LogicState {
        self.wires.get(wire).expect("invalid wire ID").base_drive
    }

    /// Gets the current state of a wire
    pub fn get_wire_state(&self, wire: WireId) -> LogicState {
        self.wire_states.get(wire).expect("invalid wire ID").get()
    }

    fn add_small_component(&mut self, component: SmallComponent) -> (usize, ComponentId) {
        let index = self.small_component_heap.len();
        self.small_component_heap.push(component);

        let output_offset = self.component_outputs.len();
        self.component_outputs
            .push(LogicStateCell::new(LogicState::HIGH_Z));

        (
            output_offset,
            self.components
                .insert(Component::new_small(index, output_offset)),
        )
    }

    fn add_large_component<C: LargeComponent + 'static>(
        &mut self,
        component: C,
    ) -> (usize, ComponentId) {
        let output_offset = self.component_outputs.len();
        for _ in 0..component.output_count() {
            self.component_outputs
                .push(LogicStateCell::new(LogicState::HIGH_Z));
        }

        (
            output_offset,
            self.components
                .insert(Component::new_large(component, output_offset)),
        )
    }

    fn check_wire_widths_match(&self, wires: &[WireId]) -> Result<(), AddComponentError> {
        if wires.array_windows::<2>().all(|w| {
            let w0 = self.wires.get(w[0]).expect("invalid wire ID");
            let w1 = self.wires.get(w[1]).expect("invalid wire ID");
            w0.width == w1.width
        }) {
            Ok(())
        } else {
            Err(AddComponentError::WireWidthMismatch)
        }
    }

    def_add_binary_gate!(
        /// Adds an `AND Gate` component to the simulation
        add_and_gate,
        AndGate
    );

    def_add_binary_gate!(
        /// Adds an `OR Gate` component to the simulation
        add_or_gate,
        OrGate
    );

    def_add_binary_gate!(
        /// Adds an `XOR Gate` component to the simulation
        add_xor_gate,
        XorGate
    );

    def_add_binary_gate!(
        /// Adds a `NAND Gate` component to the simulation
        add_nand_gate,
        NandGate
    );

    def_add_binary_gate!(
        /// Adds a `NOR Gate` component to the simulation
        add_nor_gate,
        NorGate
    );

    def_add_binary_gate!(
        /// Adds an `XNOR Gate` component to the simulation
        add_xnor_gate,
        XnorGate
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
        self.check_wire_widths_match(&[input, output])?;

        let enable_wire = self.wires.get(enable).expect("invalid wire ID");
        if enable_wire.width != 1 {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let buffer = SmallComponent::Buffer {
            input,
            enable,
            output,
        };
        let (output_offset, id) = self.add_small_component(buffer);

        let input_wire = self.wires.get_mut(input).unwrap();
        input_wire.driving.push(id);
        let enable_wire = self.wires.get_mut(enable).unwrap();
        enable_wire.driving.push(id);
        let output_wire = self.wires.get_mut(output).unwrap();
        output_wire.drivers.push(output_offset);

        Ok(id)
    }

    /// Adds a `Slice` component to the simulation
    pub fn add_slice(
        &mut self,
        input: WireId,
        offset: LogicOffset,
        output: WireId,
    ) -> AddComponentResult {
        let input_wire = self.wires.get(input).expect("invalid wire ID");
        let output_wire = self.wires.get(output).expect("invalid wire ID");

        if output_wire.width > input_wire.width {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        if (offset.get() + output_wire.width.get()) > input_wire.width.get() {
            return Err(AddComponentError::OffsetOutOfRange);
        }

        let slice = SmallComponent::Slice {
            input,
            offset,
            output,
        };
        let (output_offset, id) = self.add_small_component(slice);

        let input_wire = self.wires.get_mut(input).unwrap();
        input_wire.driving.push(id);
        let output_wire = self.wires.get_mut(output).unwrap();
        output_wire.drivers.push(output_offset);

        Ok(id)
    }

    /// Adds a `Merge` component to the simulation
    pub fn add_merge(
        &mut self,
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    ) -> AddComponentResult {
        let input_wire_a = self.wires.get(input_a).expect("invalid wire ID");
        let input_wire_b = self.wires.get(input_b).expect("invalid wire ID");
        let output_wire = self.wires.get(output).expect("invalid wire ID");

        if (input_wire_a.width.get() + input_wire_b.width.get()) != output_wire.width.get() {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let merge = SmallComponent::Merge {
            input_a,
            input_b,
            output,
        };
        let (output_offset, id) = self.add_small_component(merge);

        let input_wire_a = self.wires.get_mut(input_a).unwrap();
        input_wire_a.driving.push(id);
        let input_wire_b = self.wires.get_mut(input_b).unwrap();
        input_wire_b.driving.push(id);
        let output_wire = self.wires.get_mut(output).unwrap();
        output_wire.drivers.push(output_offset);

        Ok(id)
    }

    fn update_wires(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.wire_update_queue.par_sort_unstable();
        self.wire_update_queue.dedup();
        self.component_update_queue.clear();

        let conflicts = Mutex::new(Vec::new());

        let component_update_queue_iter =
            self.wire_update_queue
                .par_iter()
                .copied()
                .flat_map_iter(|wire_id| {
                    let wire = self.wires.get(wire_id).expect("invalid wire ID");
                    let state = self.wire_states.get(wire_id).expect("invalid wire ID");
                    // SAFETY: sort_unstable + dedup ensure wire_id is unique between all iterations
                    let state = unsafe { state.get_mut_unsafe() };

                    match wire.update(&self.component_outputs) {
                        WireUpdateResult::Ok(new_state) => {
                            if new_state != *state {
                                *state = new_state;

                                wire.driving.as_slice()
                            } else {
                                [].as_slice()
                            }
                        }
                        WireUpdateResult::Conflict => {
                            // Locking here is ok because we are in the error path
                            let mut conflict_list =
                                conflicts.lock().expect("failed to aquire mutex");
                            conflict_list.push(wire_id);

                            [].as_slice()
                        }
                    }
                });

        self.component_update_queue
            .par_extend(component_update_queue_iter);

        let conflicts = conflicts
            .into_inner()
            .expect("failed to aquire mutex")
            .into_boxed_slice();

        if conflicts.len() > 0 {
            SimulationStepResult::Err(SimulationErrors { conflicts })
        } else if self.component_update_queue.len() > 0 {
            SimulationStepResult::Changed
        } else {
            SimulationStepResult::Unchanged
        }
    }

    fn update_components(&mut self) -> SimulationStepResult {
        use rayon::prelude::*;

        self.component_update_queue.par_sort_unstable();
        self.component_update_queue.dedup();
        self.wire_update_queue.clear();

        let wire_update_queue_iter = self
            .component_update_queue
            .par_iter()
            .copied()
            .flat_map_iter(|component_id| {
                let component = self
                    .components
                    .get(component_id)
                    .expect("invalid component ID");

                component.update(
                    &self.small_component_heap,
                    &self.wires,
                    &self.wire_states,
                    &self.component_outputs,
                )
            });

        self.wire_update_queue.par_extend(wire_update_queue_iter);

        if self.wire_update_queue.len() > 0 {
            SimulationStepResult::Changed
        } else {
            SimulationStepResult::Unchanged
        }
    }

    /// Resets the simulation
    pub fn reset(&mut self) {
        for state in self.wire_states.iter_mut() {
            let state = state.get_mut();
            *state = LogicState::HIGH_Z;
        }

        for output in self.component_outputs.iter_mut() {
            let output = output.get_mut();
            *output = LogicState::HIGH_Z;
        }
    }

    /// Begins simulating
    ///
    /// Must be called before any calls to `step_sim`
    pub fn begin_sim(&mut self) -> SimulationStepResult {
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
