use crate::*;
use smallvec::smallvec;

#[repr(transparent)]
struct SyncCell<T> {
    inner: std::cell::Cell<T>,
}

impl<T> SyncCell<T> {
    #[inline]
    fn new(value: T) -> Self {
        Self {
            inner: std::cell::Cell::new(value),
        }
    }

    #[inline]
    fn get(&self) -> T
    where
        T: Copy,
    {
        self.inner.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    #[inline]
    fn set(&self, value: T) {
        self.inner.set(value)
    }
}

impl<T: Copy + std::fmt::Debug> std::fmt::Debug for SyncCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.get(), f)
    }
}

// SAFETY:
//   This is not safe in general, but through guarantees of the simulator
//   all structs in this module are only accesses by one thread at a time.
unsafe impl<T> Sync for SyncCell<T> {}

#[derive(Debug)]
pub(crate) enum SmallComponent {
    AndGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    OrGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    XorGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    NandGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    NorGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    XnorGate {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    NotGate {
        input: WireId,
        output: WireId,
    },
    Buffer {
        input: WireId,
        enable: WireId,
        output: WireId,
    },
    Slice {
        input: WireId,
        offset: LogicOffset,
        output: WireId,
    },
    Merge {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    Add {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    Sub {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    Mul {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    Div {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    Rem {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
}

impl SmallComponent {
    fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        output_state: &LogicStateCell,
    ) -> inline_vec!(WireId) {
        macro_rules! impl_gate {
            ($input_a:ident, $input_b:ident, $output:ident => $op:ident) => {{
                let state_a = unsafe { wire_states.get_unchecked($input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked($input_b).get() };
                ($output, state_a.$op(state_b))
            }};
        }

        macro_rules! impl_arithmetic {
            ($input_a:ident, $input_b:ident, $output:ident => $op:ident) => {{
                let wire_width = unsafe { *wire_widths.get_unchecked($input_a) };
                let state_a = unsafe { wire_states.get_unchecked($input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked($input_b).get() };
                ($output, state_a.$op(state_b, wire_width))
            }};
        }

        let (output, new_output_state) = match *self {
            SmallComponent::AndGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_and),
            SmallComponent::OrGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_or),
            SmallComponent::XorGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_xor),
            SmallComponent::NandGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_nand),
            SmallComponent::NorGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_nor),
            SmallComponent::XnorGate {
                input_a,
                input_b,
                output,
            } => impl_gate!(input_a, input_b, output => logic_xnor),
            SmallComponent::NotGate { input, output } => {
                let state = unsafe { wire_states.get_unchecked(input).get() };
                (output, state.logic_not())
            }
            SmallComponent::Buffer {
                input,
                enable,
                output,
            } => {
                let state = unsafe { wire_states.get_unchecked(input).get() };
                let state_enable = unsafe { wire_states.get_unchecked(enable).get() };

                let output_state = match state_enable.get_bit_state(LogicOffset::MIN) {
                    LogicBitState::HighZ | LogicBitState::Undefined => LogicState::UNDEFINED,
                    LogicBitState::Logic0 => LogicState::HIGH_Z,
                    LogicBitState::Logic1 => state,
                };

                (output, output_state)
            }
            SmallComponent::Slice {
                input,
                offset,
                output,
            } => {
                let state = unsafe { wire_states.get_unchecked(input).get() };

                (
                    output,
                    LogicState {
                        state: state.state >> offset,
                        valid: state.valid >> offset,
                    },
                )
            }
            SmallComponent::Merge {
                input_a,
                input_b,
                output,
            } => {
                let wire_a_width = unsafe { *wire_widths.get_unchecked(input_a) };
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                let mask = LogicStorage::mask(wire_a_width);
                let offset = width_to_offset(wire_a_width).expect("invalid merge offset");

                (
                    output,
                    LogicState {
                        state: (state_a.state & mask) | (state_b.state << offset),
                        valid: (state_a.valid & mask) | (state_b.valid << offset),
                    },
                )
            }
            SmallComponent::Add {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => add),
            SmallComponent::Sub {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => sub),
            SmallComponent::Mul {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => mul),
            SmallComponent::Div {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => div),
            SmallComponent::Rem {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => rem),
        };

        let changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `output_state` is a reference uniquely associated with this component
            output_state.set_unsafe(new_output_state)
        };

        if changed {
            smallvec![output]
        } else {
            smallvec![]
        }
    }
}

/// Contains mutable data of a component
#[derive(Debug)]
pub enum ComponentData<'a> {
    /// The component does not store any data
    None,
    /// The component stores a single register value
    RegisterValue(&'a mut LogicState),
}

pub(crate) trait LargeComponent: std::fmt::Debug + Send + Sync {
    fn output_count(&self) -> usize;

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::None
    }

    fn reset(&mut self) {}

    fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId);
}

macro_rules! wide_gate {
    ($name:ident, $op:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inputs: inline_vec!(WireId),
            output: WireId,
        }

        impl $name {
            #[inline]
            pub(crate) fn new(inputs: &[WireId], output: WireId) -> Self {
                Self {
                    inputs: inputs.into(),
                    output,
                }
            }
        }

        impl LargeComponent for $name {
            fn output_count(&self) -> usize {
                1
            }

            fn update(
                &self,
                _wire_widths: &WireWidthList,
                wire_states: &WireStateList,
                outputs: &[LogicStateCell],
            ) -> inline_vec!(WireId) {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| unsafe { wire_states.get_unchecked(input).get() })
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED);

                let changed = unsafe {
                    // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
                    //         and `outputs` is a slice uniquely associated with this component
                    outputs[0].set_unsafe(new_output_state)
                };

                if changed {
                    smallvec![self.output]
                } else {
                    smallvec![]
                }
            }
        }
    };
}

macro_rules! wide_gate_inv {
    ($name:ident, $op:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inputs: inline_vec!(WireId),
            output: WireId,
        }

        impl $name {
            #[inline]
            pub(crate) fn new(inputs: &[WireId], output: WireId) -> Self {
                Self {
                    inputs: inputs.into(),
                    output,
                }
            }
        }

        impl LargeComponent for $name {
            fn output_count(&self) -> usize {
                1
            }

            fn update(
                &self,
                _wire_widths: &WireWidthList,
                wire_states: &WireStateList,
                outputs: &[LogicStateCell],
            ) -> inline_vec!(WireId) {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| unsafe { wire_states.get_unchecked(input).get() })
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED)
                    .logic_not();

                let changed = unsafe {
                    // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
                    //         and `outputs` is a slice uniquely associated with this component
                    outputs[0].set_unsafe(new_output_state)
                };

                if changed {
                    smallvec![self.output]
                } else {
                    smallvec![]
                }
            }
        }
    };
}

wide_gate!(WideAndGate, logic_and);
wide_gate!(WideOrGate, logic_or);
wide_gate!(WideXorGate, logic_xor);
wide_gate_inv!(WideNandGate, logic_and);
wide_gate_inv!(WideNorGate, logic_or);
wide_gate_inv!(WideXnorGate, logic_xor);

#[derive(Debug)]
pub(crate) struct Register {
    data_in: WireId,
    data_out: WireId,
    enable: WireId,
    clock: WireId,
    prev_clock: SyncCell<Option<bool>>,
    data: SyncCell<LogicState>,
}

impl Register {
    #[inline]
    pub(crate) fn new(data_in: WireId, data_out: WireId, enable: WireId, clock: WireId) -> Self {
        Self {
            data_in,
            data_out,
            enable,
            clock,
            prev_clock: SyncCell::new(None),
            data: SyncCell::new(LogicState::UNDEFINED),
        }
    }
}

impl LargeComponent for Register {
    fn output_count(&self) -> usize {
        1
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::RegisterValue(self.data.get_mut())
    }

    fn reset(&mut self) {
        *self.prev_clock.get_mut() = None;
        *self.data.get_mut() = LogicState::UNDEFINED;
    }

    fn update(
        &self,
        _wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        let data_in_state = unsafe { wire_states.get_unchecked(self.data_in).get() };
        let enable_state = unsafe { wire_states.get_unchecked(self.enable).get() };
        let clock_state = unsafe { wire_states.get_unchecked(self.clock).get() };

        let new_clock = match clock_state.get_bit_state(LogicOffset::MIN) {
            LogicBitState::HighZ | LogicBitState::Undefined => self.prev_clock.get(),
            LogicBitState::Logic0 => Some(false),
            LogicBitState::Logic1 => Some(true),
        };

        let new_data = if let (Some(false), Some(true)) = (self.prev_clock.get(), new_clock) {
            match enable_state.get_bit_state(LogicOffset::MIN) {
                LogicBitState::HighZ | LogicBitState::Undefined => LogicState::UNDEFINED,
                LogicBitState::Logic0 => self.data.get(),
                LogicBitState::Logic1 => data_in_state.high_z_to_undefined(),
            }
        } else {
            self.data.get()
        };

        let changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[0].set_unsafe(new_data)
        };

        self.prev_clock.set(new_clock);
        self.data.set(new_data);

        if changed {
            smallvec![self.data_out]
        } else {
            smallvec![]
        }
    }
}

#[derive(Debug)]
enum ComponentKind {
    Small(SmallComponent),
    Large(Box<dyn LargeComponent>),
}

#[derive(Debug)]
pub(crate) struct Component {
    kind: ComponentKind,
    output_offset: usize,
}

impl Component {
    #[inline]
    pub(crate) fn new_small(component: SmallComponent, output_offset: usize) -> Self {
        Self {
            kind: ComponentKind::Small(component),
            output_offset,
        }
    }

    #[inline]
    pub(crate) fn new_large<C: LargeComponent + 'static>(
        component: C,
        output_offset: usize,
    ) -> Self {
        Self {
            kind: ComponentKind::Large(Box::new(component)),
            output_offset,
        }
    }

    #[inline]
    pub(crate) fn get_data(&mut self) -> ComponentData<'_> {
        match &mut self.kind {
            ComponentKind::Small(_) => ComponentData::None,
            ComponentKind::Large(component) => component.get_data(),
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        match &mut self.kind {
            ComponentKind::Small(_) => {}
            ComponentKind::Large(component) => component.reset(),
        }
    }

    #[inline]
    pub(crate) fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        match &self.kind {
            ComponentKind::Small(component) => {
                component.update(wire_widths, wire_states, &outputs[self.output_offset])
            }
            ComponentKind::Large(component) => {
                let output_range =
                    self.output_offset..(self.output_offset + component.output_count());
                component.update(wire_widths, wire_states, &outputs[output_range])
            }
        }
    }
}
