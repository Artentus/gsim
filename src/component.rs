use crate::*;
use smallvec::{smallvec, SmallVec};

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
}

impl SmallComponent {
    fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        output_state: &LogicStateCell,
    ) -> SmallVec<[WireId; 4]> {
        let (output, new_output_state) = match self {
            &SmallComponent::AndGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_and(state_b))
            }
            &SmallComponent::OrGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_or(state_b))
            }
            &SmallComponent::XorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_xor(state_b))
            }
            &SmallComponent::NandGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_nand(state_b))
            }
            &SmallComponent::NorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_nor(state_b))
            }
            &SmallComponent::XnorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };
                (output, state_a.logic_xnor(state_b))
            }
            &SmallComponent::NotGate { input, output } => {
                let state = unsafe { wire_states.get_unchecked(input).get() };
                (output, state.logic_not())
            }
            &SmallComponent::Buffer {
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
            &SmallComponent::Slice {
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
            &SmallComponent::Merge {
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
        };

        // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
        //         and `output` is a reference uniquely associated with this component
        let output_state = unsafe { output_state.get_mut_unsafe() };

        if new_output_state != *output_state {
            *output_state = new_output_state;

            smallvec![output]
        } else {
            smallvec![]
        }
    }
}

pub(crate) trait LargeComponent: std::fmt::Debug + Send + Sync {
    fn output_count(&self) -> usize;

    fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> SmallVec<[WireId; 4]>;
}

macro_rules! wide_gate {
    ($name:ident, $op:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inputs: SmallVec<[WireId; 4]>,
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
            ) -> SmallVec<[WireId; 4]> {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| unsafe { wire_states.get_unchecked(input).get() })
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED);

                // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
                //         and `outputs` is a slice uniquely associated with this component
                let output_state = unsafe { outputs[0].get_mut_unsafe() };

                if new_output_state != *output_state {
                    *output_state = new_output_state;

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
            inputs: SmallVec<[WireId; 4]>,
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
            ) -> SmallVec<[WireId; 4]> {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| unsafe { wire_states.get_unchecked(input).get() })
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED)
                    .logic_not();

                // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
                //         and `outputs` is a slice uniquely associated with this component
                let output_state = unsafe { outputs[0].get_mut_unsafe() };

                if new_output_state != *output_state {
                    *output_state = new_output_state;

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
    pub(crate) fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> SmallVec<[WireId; 4]> {
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
