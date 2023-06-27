#![deny(unsafe_op_in_unsafe_fn)]

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
}

impl SmallComponent {
    unsafe fn update(
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

        macro_rules! impl_horizontal_gate {
            ($input:ident, $output:ident => $op:ident) => {{
                let width = unsafe { *wire_widths.get_unchecked($input) };
                let state = unsafe { wire_states.get_unchecked($input).get() };
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

    unsafe fn update(
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

            unsafe fn update(
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

            unsafe fn update(
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

    unsafe fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        let width = unsafe { *wire_widths.get_unchecked(self.input_a) };
        let a = unsafe { wire_states.get_unchecked(self.input_a).get() };
        let b = unsafe { wire_states.get_unchecked(self.input_b).get() };
        let c = unsafe { wire_states.get_unchecked(self.carry_in).get() };

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

        let output_changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[0].set_unsafe(new_output)
        };

        let carry_out_changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[1].set_unsafe(new_carry_out)
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

    unsafe fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        let width = unsafe { *wire_widths.get_unchecked(self.input_a) };
        let a = unsafe { wire_states.get_unchecked(self.input_a).get() };
        let b = unsafe { wire_states.get_unchecked(self.input_b).get() };

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

        let low_changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[0].set_unsafe(new_low_state)
        };

        let high_changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[1].set_unsafe(new_high_state)
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

    unsafe fn update(
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

// This has to contain raw pointers because using vectors would require a mutex.
enum Memory {
    U8(*mut [u8; 2], usize),
    U16(*mut [u16; 2], usize),
    U32(*mut [u32; 2], usize),
}

impl Memory {
    #[allow(clippy::unnecessary_cast)]
    fn new(width: LogicWidth, len: usize) -> Self {
        const VALUE: (LogicSizeInteger, LogicSizeInteger) = LogicState::UNDEFINED.to_state_valid();
        const STATE: LogicSizeInteger = VALUE.0;
        const VALID: LogicSizeInteger = VALUE.1;

        if width <= 8 {
            let mut mem = vec![[STATE as u8, VALID as u8]; len];
            debug_assert_eq!(mem.len(), mem.capacity());

            let ptr = mem.as_mut_ptr();
            std::mem::forget(mem);

            Self::U8(ptr, len)
        } else if width <= 16 {
            let mut mem = vec![[STATE as u16, VALID as u16]; len];
            debug_assert_eq!(mem.len(), mem.capacity());

            let ptr = mem.as_mut_ptr();
            std::mem::forget(mem);

            Self::U16(ptr, len)
        } else {
            let mut mem = vec![[STATE as u32, VALID as u32]; len];
            debug_assert_eq!(mem.len(), mem.capacity());

            let ptr = mem.as_mut_ptr();
            std::mem::forget(mem);

            Self::U32(ptr, len)
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match *self {
            Self::U8(_, len) | Self::U16(_, len) | Self::U32(_, len) => len,
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn read(&self, addr: usize) -> LogicState {
        unsafe {
            // SAFETY: we have a shared reference and are only reading

            let (state, valid) = match *self {
                Self::U8(ptr, len) => {
                    assert!(addr < len);
                    let [state, valid] = ptr.add(addr).read();
                    (state as LogicSizeInteger, valid as LogicSizeInteger)
                }
                Self::U16(ptr, len) => {
                    assert!(addr < len);
                    let [state, valid] = ptr.add(addr).read();
                    (state as LogicSizeInteger, valid as LogicSizeInteger)
                }
                Self::U32(ptr, len) => {
                    assert!(addr < len);
                    let [state, valid] = ptr.add(addr).read();
                    (state as LogicSizeInteger, valid as LogicSizeInteger)
                }
            };

            LogicState::from_state_valid(state, valid)
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&mut self, addr: usize, value: LogicState) {
        unsafe {
            // SAFETY: we have a mutable reference

            let (state, valid) = value.to_state_valid();

            match *self {
                Self::U8(ptr, len) => {
                    assert!(addr < len);
                    ptr.add(addr).write([state as u8, valid as u8])
                }
                Self::U16(ptr, len) => {
                    assert!(addr < len);
                    ptr.add(addr).write([state as u16, valid as u16])
                }
                Self::U32(ptr, len) => {
                    assert!(addr < len);
                    ptr.add(addr).write([state as u32, valid as u32])
                }
            }
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn clear(&mut self) {
        unsafe {
            // SAFETY:
            //   - we have a mutable reference
            //   - the pointer was created from a vector so constructing a slice from it is ok

            const VALUE: (LogicSizeInteger, LogicSizeInteger) =
                LogicState::UNDEFINED.to_state_valid();
            const STATE: LogicSizeInteger = VALUE.0;
            const VALID: LogicSizeInteger = VALUE.1;

            match *self {
                Self::U8(ptr, len) => {
                    let mem = std::slice::from_raw_parts_mut(ptr, len);
                    mem.fill([STATE as u8, VALID as u8]);
                }
                Self::U16(ptr, len) => {
                    let mem = std::slice::from_raw_parts_mut(ptr, len);
                    mem.fill([STATE as u16, VALID as u16]);
                }
                Self::U32(ptr, len) => {
                    let mem = std::slice::from_raw_parts_mut(ptr, len);
                    mem.fill([STATE as u32, VALID as u32]);
                }
            }
        }
    }

    #[allow(clippy::unnecessary_cast)]
    unsafe fn write_unsafe(&self, addr: usize, value: LogicState) {
        let (state, valid) = value.to_state_valid();

        match *self {
            Self::U8(ptr, len) => {
                assert!(addr < len);
                unsafe { ptr.add(addr).write([state as u8, valid as u8]) }
            }
            Self::U16(ptr, len) => {
                assert!(addr < len);
                unsafe { ptr.add(addr).write([state as u16, valid as u16]) }
            }
            Self::U32(ptr, len) => {
                assert!(addr < len);
                unsafe { ptr.add(addr).write([state as u32, valid as u32]) }
            }
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: the pointer was created from a vector so constructing a vector from it is ok
            match *self {
                Self::U8(ptr, len) => {
                    let mem = Vec::from_raw_parts(ptr, len, len);
                    std::mem::drop(mem)
                }
                Self::U16(ptr, len) => {
                    let mem = Vec::from_raw_parts(ptr, len, len);
                    std::mem::drop(mem)
                }
                Self::U32(ptr, len) => {
                    let mem = Vec::from_raw_parts(ptr, len, len);
                    std::mem::drop(mem)
                }
            }
        }
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            // SAFETY:
            //   - we have a shared reference and are only reading
            //   - the pointer was created from a vector so constructing a slice from it is ok

            match *self {
                Self::U8(ptr, len) => {
                    let mem = std::slice::from_raw_parts(ptr, len);
                    std::fmt::Debug::fmt(mem, f)
                }
                Self::U16(ptr, len) => {
                    let mem = std::slice::from_raw_parts(ptr, len);
                    std::fmt::Debug::fmt(mem, f)
                }
                Self::U32(ptr, len) => {
                    let mem = std::slice::from_raw_parts(ptr, len);
                    std::fmt::Debug::fmt(mem, f)
                }
            }
        }
    }
}

unsafe impl Send for Memory {}

// SAFETY:
//   This is not safe in general, but through guarantees of the simulator
//   all structs in this module are only accesses by one thread at a time.
unsafe impl Sync for Memory {}

#[derive(Debug)]
pub(crate) struct Ram {
    write_addr: WireId,
    data_in: WireId,
    read_addr: WireId,
    data_out: WireId,
    write: WireId,
    clock: WireId,
    prev_clock: SyncCell<Option<bool>>,
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
            prev_clock: SyncCell::new(None),
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
        *self.prev_clock.get_mut() = None;
        self.mem.clear();
    }

    unsafe fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        let addr_width = unsafe { *wire_widths.get_unchecked(self.write_addr) };
        let write_addr_state = unsafe { wire_states.get_unchecked(self.write_addr).get() };
        let read_addr_state = unsafe { wire_states.get_unchecked(self.read_addr).get() };
        let data_in_state = unsafe { wire_states.get_unchecked(self.data_in).get() };
        let write_state = unsafe { wire_states.get_unchecked(self.write).get() };
        let clock_state = unsafe { wire_states.get_unchecked(self.clock).get() };

        let addr_mask = LogicStorage::mask(addr_width);

        let new_clock = match clock_state.get_bit_state(LogicOffset::MIN) {
            LogicBitState::HighZ | LogicBitState::Undefined => self.prev_clock.get(),
            LogicBitState::Logic0 => Some(false),
            LogicBitState::Logic1 => Some(true),
        };

        if let (Some(false), Some(true)) = (self.prev_clock.get(), new_clock) {
            let write_addr = write_addr_state.state & addr_mask;
            let write_addr_valid = write_addr_state.valid | !addr_mask;

            if write_addr_valid == LogicStorage::ALL_ONE {
                unsafe {
                    // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
                    //         so we have explusive access to the memory

                    match write_state.get_bit_state(LogicOffset::MIN) {
                        LogicBitState::HighZ | LogicBitState::Undefined => self
                            .mem
                            .write_unsafe(write_addr.get() as usize, LogicState::UNDEFINED),
                        LogicBitState::Logic0 => {}
                        LogicBitState::Logic1 => self.mem.write_unsafe(
                            write_addr.get() as usize,
                            data_in_state.high_z_to_undefined(),
                        ),
                    }
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

        let changed = unsafe {
            // SAFETY: sort_unstable + dedup ensure every iteration updates a unique component,
            //         and `outputs` is a slice uniquely associated with this component
            outputs[0].set_unsafe(new_data)
        };

        self.prev_clock.set(new_clock);

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
    pub(crate) unsafe fn update(
        &self,
        wire_widths: &WireWidthList,
        wire_states: &WireStateList,
        outputs: &[LogicStateCell],
    ) -> inline_vec!(WireId) {
        match &self.kind {
            ComponentKind::Small(component) => unsafe {
                component.update(wire_widths, wire_states, &outputs[self.output_offset])
            },
            ComponentKind::Large(component) => {
                let output_range =
                    self.output_offset..(self.output_offset + component.output_count());
                unsafe { component.update(wire_widths, wire_states, &outputs[output_range]) }
            }
        }
    }
}
