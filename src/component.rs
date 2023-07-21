use crate::*;
use smallvec::smallvec;

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
    LeftShift {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    LogicalRightShift {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    ArithmeticRightShift {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    HorizontalAnd {
        input: WireId,
        output: WireId,
    },
    HorizontalOr {
        input: WireId,
        output: WireId,
    },
    HorizontalNand {
        input: WireId,
        output: WireId,
    },
    HorizontalNor {
        input: WireId,
        output: WireId,
    },
    CompareEqual {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareNotEqual {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareLessThan {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareGreaterThan {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareLessThanOrEqual {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareGreaterThanOrEqual {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareLessThanSigned {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareGreaterThanSigned {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareLessThanOrEqualSigned {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    CompareGreaterThanOrEqualSigned {
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
}

impl SmallComponent {
    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        output_state: &mut LogicState,
    ) -> inline_vec!(WireId) {
        macro_rules! impl_gate {
            ($input_a:ident, $input_b:ident, $output:ident => $op:ident) => {{
                let state_a = wire_states[$input_a];
                let state_b = wire_states[$input_b];
                ($output, state_a.$op(state_b))
            }};
        }

        macro_rules! impl_arithmetic {
            ($input_a:ident, $input_b:ident, $output:ident => $op:ident) => {{
                let width = wire_widths[$input_a];
                let state_a = wire_states[$input_a];
                let state_b = wire_states[$input_b];
                ($output, state_a.$op(state_b, width))
            }};
        }

        macro_rules! impl_horizontal_gate {
            ($input:ident, $output:ident => $op:ident) => {{
                let width = wire_widths[$input];
                let state = wire_states[$input];
                ($output, state.$op(width))
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
                let state = wire_states[input];
                (output, state.logic_not())
            }
            SmallComponent::Buffer {
                input,
                enable,
                output,
            } => {
                let state = wire_states[input];
                let state_enable = wire_states[enable];

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
                let state = wire_states[input];

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
                let wire_a_width = wire_widths[input_a];
                let state_a = wire_states[input_a];
                let state_b = wire_states[input_b];

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
            SmallComponent::LeftShift {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => shl),
            SmallComponent::LogicalRightShift {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => lshr),
            SmallComponent::ArithmeticRightShift {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => ashr),
            SmallComponent::HorizontalAnd { input, output } => {
                impl_horizontal_gate!(input, output => horizontal_logic_and)
            }
            SmallComponent::HorizontalOr { input, output } => {
                impl_horizontal_gate!(input, output => horizontal_logic_or)
            }
            SmallComponent::HorizontalNand { input, output } => {
                impl_horizontal_gate!(input, output => horizontal_logic_nand)
            }
            SmallComponent::HorizontalNor { input, output } => {
                impl_horizontal_gate!(input, output => horizontal_logic_nor)
            }
            SmallComponent::CompareEqual {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => equal),
            SmallComponent::CompareNotEqual {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => not_equal),
            SmallComponent::CompareLessThan {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => less_than),
            SmallComponent::CompareGreaterThan {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => greater_than),
            SmallComponent::CompareLessThanOrEqual {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => less_than_or_equal),
            SmallComponent::CompareGreaterThanOrEqual {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => greater_than_or_equal),
            SmallComponent::CompareLessThanSigned {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => less_than_signed),
            SmallComponent::CompareGreaterThanSigned {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => greater_than_signed),
            SmallComponent::CompareLessThanOrEqualSigned {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => less_than_or_equal_signed),
            SmallComponent::CompareGreaterThanOrEqualSigned {
                input_a,
                input_b,
                output,
            } => impl_arithmetic!(input_a, input_b, output => greater_than_or_equal_signed),
        };

        let output_width = wire_widths[output];
        if !new_output_state.eq(*output_state, output_width) {
            *output_state = new_output_state;
            smallvec![output]
        } else {
            smallvec![]
        }
    }
}

#[repr(transparent)]
pub struct MemoryBlock<'a> {
    mem: &'a mut Memory,
}

impl<'a> MemoryBlock<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline]
    pub fn read(&self, addr: usize) -> LogicState {
        self.mem.read(addr)
    }

    #[inline]
    pub fn write(&mut self, addr: usize, value: LogicState) {
        self.mem.write(addr, value);
    }
}

impl<'a> std::fmt::Debug for MemoryBlock<'a> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.mem, f)
    }
}

/// Contains mutable data of a component
#[derive(Debug)]
pub enum ComponentData<'a> {
    /// The component does not store any data
    None,
    /// The component stores a single register value
    RegisterValue(&'a mut LogicState),
    /// The component stores a memory block
    MemoryBlock(MemoryBlock<'a>),
}

pub(crate) trait LargeComponent: std::fmt::Debug + Send + Sync {
    fn output_count(&self) -> usize;

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::None
    }

    fn reset(&mut self) {}

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
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
                &mut self,
                wire_widths: &WireWidthList,
                wire_states: &WireStateList,
                outputs: &mut [LogicState],
            ) -> inline_vec!(WireId) {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| wire_states[input])
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED);

                let output_width = wire_widths[self.output];
                if !new_output_state.eq(outputs[0], output_width) {
                    outputs[0] = new_output_state;
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
                &mut self,
                wire_widths: &WireWidthList,
                wire_states: &WireStateList,
                outputs: &mut [LogicState],
            ) -> inline_vec!(WireId) {
                let new_output_state = self
                    .inputs
                    .iter()
                    .map(|&input| wire_states[input])
                    .reduce(|a, b| a.$op(b))
                    .unwrap_or(LogicState::UNDEFINED)
                    .logic_not();

                let output_width = wire_widths[self.output];
                if !new_output_state.eq(outputs[0], output_width) {
                    outputs[0] = new_output_state;
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
pub(crate) struct WideMerge {
    inputs: inline_vec!(WireId),
    output: WireId,
}

impl WideMerge {
    #[inline]
    pub(crate) fn new(inputs: &[WireId], output: WireId) -> Self {
        Self {
            inputs: inputs.into(),
            output,
        }
    }
}

impl LargeComponent for WideMerge {
    fn output_count(&self) -> usize {
        1
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let mut new_output_state = LogicState::HIGH_Z;
        let mut offset = 0;
        for input in self.inputs.iter().copied() {
            let input_state = wire_states[input];
            let input_width = wire_widths[input];

            let input_mask = LogicStorage::mask(input_width);
            let input_offset = LogicOffset::new(offset).expect("invalid merge offset");

            new_output_state.state |= (input_state.state & input_mask) << input_offset;
            new_output_state.valid |= (input_state.valid & input_mask) << input_offset;

            offset += input_width.get();
        }

        let output_width = wire_widths[self.output];
        if !new_output_state.eq(outputs[0], output_width) {
            outputs[0] = new_output_state;
            smallvec![self.output]
        } else {
            smallvec![]
        }
    }
}

#[derive(Debug)]
pub(crate) struct Adder {
    input_a: WireId,
    input_b: WireId,
    carry_in: WireId,
    output: WireId,
    carry_out: WireId,
}

impl Adder {
    #[inline]
    pub(crate) fn new(
        input_a: WireId,
        input_b: WireId,
        carry_in: WireId,
        output: WireId,
        carry_out: WireId,
    ) -> Self {
        Self {
            input_a,
            input_b,
            carry_in,
            output,
            carry_out,
        }
    }
}

impl LargeComponent for Adder {
    fn output_count(&self) -> usize {
        2
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let width = wire_widths[self.input_a];
        let a = wire_states[self.input_a];
        let b = wire_states[self.input_b];
        let c = wire_states[self.carry_in];

        let (new_output, new_carry_out) = 'compute: {
            let c_in = match c.get_bit_state(LogicOffset::MIN) {
                LogicBitState::HighZ | LogicBitState::Undefined => {
                    break 'compute (LogicState::UNDEFINED, LogicState::UNDEFINED)
                }
                LogicBitState::Logic0 => false,
                LogicBitState::Logic1 => true,
            };

            let mask = LogicStorage::mask(width);
            let a_state = a.state & mask;
            let b_state = b.state & mask;
            let a_valid = a.valid | !mask;
            let b_valid = b.valid | !mask;

            if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
                let (sum, c_out) = a_state.carrying_add(b_state, c_in);

                let c_out = if let Some(c_index) = LogicOffset::new(width.get()) {
                    sum.get_bit(c_index)
                } else {
                    c_out
                };

                (
                    LogicState {
                        state: sum,
                        valid: LogicStorage::ALL_ONE,
                    },
                    LogicState::from_bool(c_out),
                )
            } else {
                (LogicState::UNDEFINED, LogicState::UNDEFINED)
            }
        };

        let output_changed = if !new_output.eq(outputs[0], width) {
            outputs[0] = new_output;
            true
        } else {
            false
        };

        let carry_out_changed = if !new_carry_out.eq(outputs[1], width) {
            outputs[1] = new_carry_out;
            true
        } else {
            false
        };

        match (output_changed, carry_out_changed) {
            (true, true) => smallvec![self.output, self.carry_out],
            (true, false) => smallvec![self.output],
            (false, true) => smallvec![self.carry_out],
            (false, false) => smallvec![],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Multiplier {
    input_a: WireId,
    input_b: WireId,
    output_low: WireId,
    output_high: WireId,
}

impl Multiplier {
    #[inline]
    pub(crate) fn new(
        input_a: WireId,
        input_b: WireId,
        output_low: WireId,
        output_high: WireId,
    ) -> Self {
        Self {
            input_a,
            input_b,
            output_low,
            output_high,
        }
    }
}

impl LargeComponent for Multiplier {
    fn output_count(&self) -> usize {
        2
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let width = wire_widths[self.input_a];
        let a = wire_states[self.input_a];
        let b = wire_states[self.input_b];

        let (new_low_state, new_high_state) = if (width.get() * 2) <= MAX_LOGIC_WIDTH {
            let full_result = a.mul(b, width);

            let shift_amount = unsafe {
                // SAFETY: width is at most half of MAX_LOGIC_WIDTH, so it is always a valid offset.
                LogicOffset::new_unchecked(width.get())
            };

            let high_result = LogicState {
                state: full_result.state >> shift_amount,
                valid: full_result.valid >> shift_amount,
            };

            (full_result, high_result)
        } else {
            let mask = LogicStorage::mask(width);
            let a_state = a.state & mask;
            let b_state = b.state & mask;
            let a_valid = a.valid | !mask;
            let b_valid = b.valid | !mask;

            if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
                let (low_result, high_result) = a_state.widening_mul(b_state, width);

                (
                    LogicState {
                        state: low_result,
                        valid: LogicStorage::ALL_ONE,
                    },
                    LogicState {
                        state: high_result,
                        valid: LogicStorage::ALL_ONE,
                    },
                )
            } else {
                (LogicState::UNDEFINED, LogicState::UNDEFINED)
            }
        };

        let low_changed = if !new_low_state.eq(outputs[0], width) {
            outputs[0] = new_low_state;
            true
        } else {
            false
        };

        let high_changed = if !new_high_state.eq(outputs[1], width) {
            outputs[1] = new_high_state;
            true
        } else {
            false
        };

        match (low_changed, high_changed) {
            (true, true) => smallvec![self.output_low, self.output_high],
            (true, false) => smallvec![self.output_low],
            (false, true) => smallvec![self.output_high],
            (false, false) => smallvec![],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Multiplexer {
    inputs: inline_vec!(WireId),
    select: WireId,
    output: WireId,
}

impl Multiplexer {
    #[inline]
    pub(crate) fn new(inputs: &[WireId], select: WireId, output: WireId) -> Self {
        Self {
            inputs: inputs.into(),
            select,
            output,
        }
    }
}

impl LargeComponent for Multiplexer {
    fn output_count(&self) -> usize {
        1
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let select_state = wire_states[self.select];
        let select_width = wire_widths[self.select];

        let select_mask = LogicStorage::mask(select_width);
        let select_valid = select_state.valid | !select_mask;

        let new_output_state = if select_valid == LogicStorage::ALL_ONE {
            let index = select_state.state & select_mask;
            let input = self.inputs[index.get() as usize];
            wire_states[input].high_z_to_undefined()
        } else {
            LogicState::UNDEFINED
        };

        let output_width = wire_widths[self.output];
        if !new_output_state.eq(outputs[0], output_width) {
            outputs[0] = new_output_state;
            smallvec![self.output]
        } else {
            smallvec![]
        }
    }
}

#[derive(Debug)]
pub(crate) struct Register {
    data_in: WireId,
    data_out: WireId,
    enable: WireId,
    clock: WireId,
    prev_clock: Option<bool>,
    data: LogicState,
}

impl Register {
    #[inline]
    pub(crate) fn new(data_in: WireId, data_out: WireId, enable: WireId, clock: WireId) -> Self {
        Self {
            data_in,
            data_out,
            enable,
            clock,
            prev_clock: None,
            data: LogicState::UNDEFINED,
        }
    }
}

impl LargeComponent for Register {
    fn output_count(&self) -> usize {
        1
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::RegisterValue(&mut self.data)
    }

    fn reset(&mut self) {
        self.prev_clock = None;
        self.data = LogicState::UNDEFINED;
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let data_in_state = wire_states[self.data_in];
        let enable_state = wire_states[self.enable];
        let clock_state = wire_states[self.clock];

        let new_clock = match clock_state.get_bit_state(LogicOffset::MIN) {
            LogicBitState::HighZ | LogicBitState::Undefined => self.prev_clock,
            LogicBitState::Logic0 => Some(false),
            LogicBitState::Logic1 => Some(true),
        };

        let new_data = if let (Some(false), Some(true)) = (self.prev_clock, new_clock) {
            match enable_state.get_bit_state(LogicOffset::MIN) {
                LogicBitState::HighZ | LogicBitState::Undefined => LogicState::UNDEFINED,
                LogicBitState::Logic0 => self.data,
                LogicBitState::Logic1 => data_in_state.high_z_to_undefined(),
            }
        } else {
            self.data
        };

        self.prev_clock = new_clock;
        self.data = new_data;

        let data_width = wire_widths[self.data_out];
        if !new_data.eq(outputs[0], data_width) {
            outputs[0] = new_data;
            smallvec![self.data_out]
        } else {
            smallvec![]
        }
    }
}

// This has to contain raw pointers because using vectors would require a mutex.
enum Memory {
    U8(Box<[[u8; 2]]>),
    U16(Box<[[u16; 2]]>),
    U32(Box<[[u32; 2]]>),
}

impl Memory {
    #[allow(clippy::unnecessary_cast)]
    fn new(width: LogicWidth, len: usize) -> Self {
        const VALUE: (LogicSizeInteger, LogicSizeInteger) = LogicState::UNDEFINED.to_state_valid();
        const STATE: LogicSizeInteger = VALUE.0;
        const VALID: LogicSizeInteger = VALUE.1;

        if width <= 8 {
            let mem = vec![[STATE as u8, VALID as u8]; len];
            Self::U8(mem.into_boxed_slice())
        } else if width <= 16 {
            let mem = vec![[STATE as u16, VALID as u16]; len];
            Self::U16(mem.into_boxed_slice())
        } else {
            let mem = vec![[STATE as u32, VALID as u32]; len];
            Self::U32(mem.into_boxed_slice())
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::U8(mem) => mem.len(),
            Self::U16(mem) => mem.len(),
            Self::U32(mem) => mem.len(),
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn read(&self, addr: usize) -> LogicState {
        let (state, valid) = match self {
            Self::U8(mem) => {
                let [state, valid] = mem[addr];
                (state as LogicSizeInteger, valid as LogicSizeInteger)
            }
            Self::U16(mem) => {
                let [state, valid] = mem[addr];
                (state as LogicSizeInteger, valid as LogicSizeInteger)
            }
            Self::U32(mem) => {
                let [state, valid] = mem[addr];
                (state as LogicSizeInteger, valid as LogicSizeInteger)
            }
        };

        LogicState::from_state_valid(state, valid)
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&mut self, addr: usize, value: LogicState) {
        let (state, valid) = value.to_state_valid();

        match self {
            Self::U8(mem) => {
                mem[addr] = [state as u8, valid as u8];
            }
            Self::U16(mem) => {
                mem[addr] = [state as u16, valid as u16];
            }
            Self::U32(mem) => {
                mem[addr] = [state as u32, valid as u32];
            }
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn clear(&mut self) {
        const VALUE: (LogicSizeInteger, LogicSizeInteger) = LogicState::UNDEFINED.to_state_valid();
        const STATE: LogicSizeInteger = VALUE.0;
        const VALID: LogicSizeInteger = VALUE.1;

        match self {
            Self::U8(mem) => {
                mem.fill([STATE as u8, VALID as u8]);
            }
            Self::U16(mem) => {
                mem.fill([STATE as u16, VALID as u16]);
            }
            Self::U32(mem) => {
                mem.fill([STATE as u32, VALID as u32]);
            }
        }
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(mem) => std::fmt::Debug::fmt(&**mem, f),
            Self::U16(mem) => std::fmt::Debug::fmt(&**mem, f),
            Self::U32(mem) => std::fmt::Debug::fmt(&**mem, f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Ram {
    write_addr: WireId,
    data_in: WireId,
    read_addr: WireId,
    data_out: WireId,
    write: WireId,
    clock: WireId,
    prev_clock: Option<bool>,
    mem: Memory,
}

impl Ram {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(crate) fn new(
        write_addr: WireId,
        data_in: WireId,
        read_addr: WireId,
        data_out: WireId,
        write: WireId,
        clock: WireId,
        addr_width: LogicWidth,
        data_width: LogicWidth,
    ) -> Self {
        Self {
            write_addr,
            data_in,
            read_addr,
            data_out,
            write,
            clock,
            prev_clock: None,
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }
}

impl LargeComponent for Ram {
    fn output_count(&self) -> usize {
        1
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::MemoryBlock(MemoryBlock { mem: &mut self.mem })
    }

    fn reset(&mut self) {
        self.prev_clock = None;
        self.mem.clear();
    }

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let addr_width = wire_widths[self.write_addr];
        let data_width = wire_widths[self.data_out];
        let write_addr_state = wire_states[self.write_addr];
        let read_addr_state = wire_states[self.read_addr];
        let data_in_state = wire_states[self.data_in];
        let write_state = wire_states[self.write];
        let clock_state = wire_states[self.clock];

        let addr_mask = LogicStorage::mask(addr_width);

        let new_clock = match clock_state.get_bit_state(LogicOffset::MIN) {
            LogicBitState::HighZ | LogicBitState::Undefined => self.prev_clock,
            LogicBitState::Logic0 => Some(false),
            LogicBitState::Logic1 => Some(true),
        };

        if let (Some(false), Some(true)) = (self.prev_clock, new_clock) {
            let write_addr = write_addr_state.state & addr_mask;
            let write_addr_valid = write_addr_state.valid | !addr_mask;

            if write_addr_valid == LogicStorage::ALL_ONE {
                match write_state.get_bit_state(LogicOffset::MIN) {
                    LogicBitState::HighZ | LogicBitState::Undefined => self
                        .mem
                        .write(write_addr.get() as usize, LogicState::UNDEFINED),
                    LogicBitState::Logic0 => {}
                    LogicBitState::Logic1 => self.mem.write(
                        write_addr.get() as usize,
                        data_in_state.high_z_to_undefined(),
                    ),
                }
            } else {
                // NOTE:
                //   There is nothing sensible we can do here.
                //   In a real circuit a random address would be overwritten.
            }
        };

        let read_addr = read_addr_state.state & addr_mask;
        let read_addr_valid = read_addr_state.valid | !addr_mask;

        let new_data = if read_addr_valid == LogicStorage::ALL_ONE {
            self.mem.read(read_addr.get() as usize)
        } else {
            LogicState::UNDEFINED
        };

        self.prev_clock = new_clock;

        if !new_data.eq(outputs[0], data_width) {
            outputs[0] = new_data;
            smallvec![self.data_out]
        } else {
            smallvec![]
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rom {
    addr: WireId,
    data: WireId,
    mem: Memory,
}

impl Rom {
    #[inline]
    pub(crate) fn new(
        addr: WireId,
        data: WireId,
        addr_width: LogicWidth,
        data_width: LogicWidth,
    ) -> Self {
        Self {
            addr,
            data,
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }
}

impl LargeComponent for Rom {
    fn output_count(&self) -> usize {
        1
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::MemoryBlock(MemoryBlock { mem: &mut self.mem })
    }

    fn reset(&mut self) {}

    fn update(
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        let addr_width = wire_widths[self.addr];
        let data_width = wire_widths[self.data];
        let addr_state = wire_states[self.addr];

        let addr_mask = LogicStorage::mask(addr_width);

        let addr = addr_state.state & addr_mask;
        let addr_valid = addr_state.valid | !addr_mask;

        let new_data = if addr_valid == LogicStorage::ALL_ONE {
            self.mem.read(addr.get() as usize)
        } else {
            LogicState::UNDEFINED
        };

        if !new_data.eq(outputs[0], data_width) {
            outputs[0] = new_data;
            smallvec![self.data]
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
    pub(crate) fn output_offset(&self) -> usize {
        self.output_offset
    }

    #[inline]
    pub(crate) fn output_count(&self) -> usize {
        match &self.kind {
            ComponentKind::Small(_) => 1,
            ComponentKind::Large(component) => component.output_count(),
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
        &mut self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &mut [LogicState],
    ) -> inline_vec!(WireId) {
        match &mut self.kind {
            ComponentKind::Small(component) => {
                component.update(wire_widths, wire_states, &mut outputs[0])
            }
            ComponentKind::Large(component) => component.update(wire_widths, wire_states, outputs),
        }
    }
}
