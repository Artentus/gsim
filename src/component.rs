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
        outputs: &[LogicStateCell],
    ) -> SmallVec<[WireId; 2]> {
        let (output, new_output_state) = match self {
            &SmallComponent::AndGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1

                (
                    output,
                    LogicState {
                        state: (state_a.state & state_b.state)
                            | (!state_a.valid & !state_b.valid)
                            | (state_a.state & !state_b.valid)
                            | (state_b.state & !state_a.valid),
                        valid: (state_a.valid & state_b.valid)
                            | (!state_a.state & state_a.valid)
                            | (!state_b.state & state_b.valid),
                    },
                )
            }
            &SmallComponent::OrGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    1    | Logic 1
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    1    |    1    | Logic 1
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    1    | Logic 1
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    1    |    1    | Logic 1
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1

                (
                    output,
                    LogicState {
                        state: state_a.state | !state_a.valid | state_b.state | !state_b.valid,
                        valid: (state_a.state & state_a.valid)
                            | (state_b.state & state_b.valid)
                            | (state_a.valid & state_b.valid),
                    },
                )
            }
            &SmallComponent::XorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0

                (
                    output,
                    LogicState {
                        state: (state_a.state ^ state_b.state) | !state_a.valid | !state_b.valid,
                        valid: state_a.valid & state_b.valid,
                    },
                )
            }
            &SmallComponent::NandGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0

                (
                    output,
                    LogicState {
                        state: !state_a.state | !state_a.valid | !state_b.state | !state_b.valid,
                        valid: (state_a.valid & state_b.valid)
                            | (!state_a.state & state_a.valid)
                            | (!state_b.state & state_b.valid),
                    },
                )
            }
            &SmallComponent::NorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    0    |    1    | Logic 0
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    0    |    1    | Logic 0
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    0    |    1    | Logic 0
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    0    |    1    | Logic 0
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0

                (
                    output,
                    LogicState {
                        state: (!state_a.state & !state_b.state)
                            | (!state_a.valid & !state_b.valid)
                            | (!state_a.state & !state_b.valid)
                            | (!state_b.state & !state_a.valid),
                        valid: (state_a.state & state_a.valid)
                            | (state_b.state & state_b.valid)
                            | (state_a.valid & state_b.valid),
                    },
                )
            }
            &SmallComponent::XnorGate {
                input_a,
                input_b,
                output,
            } => {
                let state_a = unsafe { wire_states.get_unchecked(input_a).get() };
                let state_b = unsafe { wire_states.get_unchecked(input_b).get() };

                //  A state | A valid | A meaning | B state | B valid | B meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    0    |    0    | High-Z    |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     1    |    1    | Logic 1   |    1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    0    | High-Z    |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    0    |    1    | Logic 0   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0   |    0    |    1    | Logic 0
                //     0    |    0    | High-Z    |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    1    | Logic 1   |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1   |    0    |    1    | Logic 0
                //     1    |    1    | Logic 1   |    1    |    1    | Logic 1   |    1    |    1    | Logic 1

                (
                    output,
                    LogicState {
                        state: !(state_a.state ^ state_b.state) | !state_a.valid | !state_b.valid,
                        valid: state_a.valid & state_b.valid,
                    },
                )
            }
            &SmallComponent::NotGate { input, output } => {
                let state = unsafe { wire_states.get_unchecked(input).get() };

                //  I state | I valid | I meaning | O state | O valid | O meaning
                // ---------|---------|-----------|---------|---------|-----------
                //     0    |    0    | High-Z    |    1    |    0    | Undefined
                //     1    |    0    | Undefined |    1    |    0    | Undefined
                //     0    |    1    | Logic 0   |    1    |    1    | Logic 1
                //     1    |    1    | Logic 1   |    0    |    1    | Logic 0

                (
                    output,
                    LogicState {
                        state: !state.state | !state.valid,
                        valid: state.valid,
                    },
                )
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
        //         and `outputs` is a slice uniquely associated with this component
        let output_state = unsafe { outputs[0].get_mut_unsafe() };

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
    ) -> SmallVec<[WireId; 2]>;
}

#[derive(Debug)]
enum ComponentKind {
    Small(usize),
    Large(Box<dyn LargeComponent>),
}

#[derive(Debug)]
pub(crate) struct Component {
    kind: ComponentKind,
    output_offset: usize,
}

impl Component {
    #[inline]
    pub(crate) fn new_small(index: usize, output_offset: usize) -> Self {
        Self {
            kind: ComponentKind::Small(index),
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

    pub(crate) fn update(
        &self,
        small_component_heap: &[SmallComponent],
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> SmallVec<[WireId; 2]> {
        match &self.kind {
            &ComponentKind::Small(index) => {
                let component = unsafe { small_component_heap.get_unchecked(index) };

                let output_range = self.output_offset..(self.output_offset + 1);
                component.update(wire_widths, wire_states, &outputs[output_range])
            }
            ComponentKind::Large(component) => {
                let output_range =
                    self.output_offset..(self.output_offset + component.output_count());
                component.update(wire_widths, wire_states, &outputs[output_range])
            }
        }
    }
}
