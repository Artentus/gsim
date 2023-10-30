#![allow(missing_debug_implementations)]

mod ops;
use ops::*;

use crate::*;
use itertools::izip;
use smallvec::smallvec;

pub(crate) enum SmallComponentKind {
    AndGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    OrGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    XorGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    NandGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    NorGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    XnorGate {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    NotGate {
        input: WireStateId,
    },
    Buffer {
        input: WireStateId,
        enable: WireStateId,
    },
    Slice {
        input: WireStateId,
        offset: u8,
    },
    Add {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    Sub {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    LeftShift {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    LogicalRightShift {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    ArithmeticRightShift {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    HorizontalAnd {
        input: WireStateId,
    },
    HorizontalOr {
        input: WireStateId,
    },
    HorizontalXor {
        input: WireStateId,
    },
    HorizontalNand {
        input: WireStateId,
    },
    HorizontalNor {
        input: WireStateId,
    },
    HorizontalXnor {
        input: WireStateId,
    },
    CompareEqual {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareNotEqual {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareLessThan {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareGreaterThan {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareLessThanOrEqual {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareGreaterThanOrEqual {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareLessThanSigned {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareGreaterThanSigned {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareLessThanOrEqualSigned {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    CompareGreaterThanOrEqualSigned {
        input_a: WireStateId,
        input_b: WireStateId,
    },
    ZeroExtend {
        input: WireStateId,
    },
    SignExtend {
        input: WireStateId,
    },
}

pub(crate) struct SmallComponent {
    kind: SmallComponentKind,
    output: WireId,
}

impl SmallComponent {
    #[inline]
    pub(crate) fn new(kind: SmallComponentKind, output: WireId) -> Self {
        Self { kind, output }
    }

    fn update(
        &self,
        output_base: OutputStateId,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let result = match self.kind {
            SmallComponentKind::AndGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_and_3(width, out, lhs, rhs)
            }
            SmallComponentKind::OrGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_or_3(width, out, lhs, rhs)
            }
            SmallComponentKind::XorGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_xor_3(width, out, lhs, rhs)
            }
            SmallComponentKind::NandGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_nand_3(width, out, lhs, rhs)
            }
            SmallComponentKind::NorGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_nor_3(width, out, lhs, rhs)
            }
            SmallComponentKind::XnorGate { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                logic_xnor_3(width, out, lhs, rhs)
            }
            SmallComponentKind::NotGate { input } => {
                let val = wire_states.get_state(input);
                let (width, out) = output_states.get_data(output_base);
                logic_not_2(width, out, val)
            }
            SmallComponentKind::Buffer { input, enable } => {
                let val = wire_states.get_state(input);
                let en = wire_states.get_state(enable);
                let (width, out) = output_states.get_data(output_base);
                buffer(width, out, val, en[0].get_bit_state(AtomOffset::MIN))
            }
            SmallComponentKind::Slice { input, offset } => {
                let val = wire_states.get_state(input);
                let (width, out) = output_states.get_data(output_base);
                slice(width, out, val, offset)
            }
            SmallComponentKind::Add { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);

                add(
                    width,
                    out,
                    &mut LogicBitState::Undefined,
                    lhs,
                    rhs,
                    LogicBitState::Logic0,
                )
            }
            SmallComponentKind::Sub { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);

                sub(
                    width,
                    out,
                    &mut LogicBitState::Undefined,
                    lhs,
                    rhs,
                    LogicBitState::Logic1,
                )
            }
            SmallComponentKind::LeftShift { input_a, input_b } => {
                let val = wire_states.get_state(input_a);
                let shamnt_width = wire_states.get_width(input_b);
                let shamnt = wire_states.get_state(input_b)[0];
                let (width, out) = output_states.get_data(output_base);
                shl(width, shamnt_width, out, val, shamnt)
            }
            SmallComponentKind::LogicalRightShift { input_a, input_b } => {
                let val = wire_states.get_state(input_a);
                let shamnt_width = wire_states.get_width(input_b);
                let shamnt = wire_states.get_state(input_b)[0];
                let (width, out) = output_states.get_data(output_base);
                lshr(width, shamnt_width, out, val, shamnt)
            }
            SmallComponentKind::ArithmeticRightShift { input_a, input_b } => {
                let val = wire_states.get_state(input_a);
                let shamnt_width = wire_states.get_width(input_b);
                let shamnt = wire_states.get_state(input_b)[0];
                let (width, out) = output_states.get_data(output_base);
                ashr(width, shamnt_width, out, val, shamnt)
            }
            SmallComponentKind::HorizontalAnd { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_and(width, &mut out[0], val)
            }
            SmallComponentKind::HorizontalOr { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_or(width, &mut out[0], val)
            }
            SmallComponentKind::HorizontalXor { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_xor(width, &mut out[0], val)
            }
            SmallComponentKind::HorizontalNand { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_nand(width, &mut out[0], val)
            }
            SmallComponentKind::HorizontalNor { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_nor(width, &mut out[0], val)
            }
            SmallComponentKind::HorizontalXnor { input } => {
                let width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (_, out) = output_states.get_data(output_base);
                horizontal_logic_xnor(width, &mut out[0], val)
            }
            SmallComponentKind::CompareEqual { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                equal(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareNotEqual { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                not_equal(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareLessThan { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                less_than(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareGreaterThan { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                greater_than(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareLessThanOrEqual { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                less_than_or_equal(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareGreaterThanOrEqual { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                greater_than_or_equal(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareLessThanSigned { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                less_than_signed(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareGreaterThanSigned { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                greater_than_signed(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareLessThanOrEqualSigned { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                less_than_or_equal_signed(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::CompareGreaterThanOrEqualSigned { input_a, input_b } => {
                let width = wire_states.get_width(input_a);
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (_, out) = output_states.get_data(output_base);
                greater_than_or_equal_signed(width, &mut out[0], lhs, rhs)
            }
            SmallComponentKind::ZeroExtend { input } => {
                let val_width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (out_width, out) = output_states.get_data(output_base);
                zero_extend(val_width, out_width, val, out)
            }
            SmallComponentKind::SignExtend { input } => {
                let val_width = wire_states.get_width(input);
                let val = wire_states.get_state(input);
                let (out_width, out) = output_states.get_data(output_base);
                sign_extend(val_width, out_width, val, out)
            }
        };

        match result {
            OpResult::Unchanged => smallvec![],
            OpResult::Changed => smallvec![self.output],
        }
    }
}

#[repr(transparent)]
pub struct MemoryCell<'a> {
    mem: &'a mut [Atom],
}

impl MemoryCell<'_> {
    pub fn read(&self) -> LogicState {
        LogicState(LogicStateRepr::Bits(self.mem.iter().copied().collect()))
    }

    pub fn write(&mut self, value: &LogicState) {
        for (dst, src) in self.mem.iter_mut().zip(value.iter_atoms()) {
            *dst = src;
        }
    }
}

#[repr(transparent)]
pub struct MemoryBlock<'a> {
    mem: &'a mut Memory,
}

impl MemoryBlock<'_> {
    #[inline]
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline]
    pub fn read(&self, addr: usize) -> LogicState {
        self.mem.read(addr)
    }

    #[inline]
    pub fn write(&mut self, addr: usize, value: &LogicState) {
        self.mem.write(addr, value.iter_atoms());
    }
}

/// Contains mutable data of a component
pub enum ComponentData<'a> {
    /// The component does not store any data
    None,
    /// The component stores a single register value
    RegisterValue(MemoryCell<'a>),
    /// The component stores a memory block
    MemoryBlock(MemoryBlock<'a>),
}

pub(crate) trait LargeComponent: Send + Sync {
    fn alloc_size(&self) -> AllocationSize;

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::None
    }

    fn reset(&mut self) {}

    fn update(
        &mut self,
        wire_states: &WireStateList,
        output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId);
}

macro_rules! wide_gate {
    ($name:ident, $op3:ident, $op2:ident) => {
        pub(crate) struct $name {
            inputs: inline_vec!(WireStateId),
            output: OutputStateId,
            output_wire: WireId,
        }

        impl $name {
            #[inline]
            pub(crate) fn new(
                inputs: impl Into<inline_vec!(WireStateId)>,
                output: OutputStateId,
                output_wire: WireId,
            ) -> Self {
                let inputs = inputs.into();
                debug_assert!(inputs.len() > 2);

                Self {
                    inputs,
                    output,
                    output_wire,
                }
            }
        }

        impl LargeComponent for $name {
            fn alloc_size(&self) -> AllocationSize {
                AllocationSize(std::mem::size_of::<$name>())
            }

            fn update(
                &mut self,
                wire_states: &WireStateList,
                mut output_states: OutputStateSlice<'_>,
            ) -> inline_vec!(WireId) {
                let lhs = wire_states.get_state(self.inputs[0]);
                let rhs = wire_states.get_state(self.inputs[1]);
                let (width, out) = output_states.get_data(self.output);
                let mut result = $op3(width, out, lhs, rhs);

                for &input in self.inputs.iter().skip(2) {
                    let rhs = wire_states.get_state(input);
                    result |= $op2(width, out, rhs);
                }

                match result {
                    OpResult::Unchanged => smallvec![],
                    OpResult::Changed => smallvec![self.output_wire],
                }
            }
        }
    };
}

macro_rules! wide_gate_inv {
    ($name:ident, $op3:ident, $op2:ident) => {
        pub(crate) struct $name {
            inputs: inline_vec!(WireStateId),
            output: OutputStateId,
            output_wire: WireId,
        }

        impl $name {
            #[inline]
            pub(crate) fn new(
                inputs: impl Into<inline_vec!(WireStateId)>,
                output: OutputStateId,
                output_wire: WireId,
            ) -> Self {
                let inputs = inputs.into();
                debug_assert!(inputs.len() > 2);

                Self {
                    inputs,
                    output,
                    output_wire,
                }
            }
        }

        impl LargeComponent for $name {
            fn alloc_size(&self) -> AllocationSize {
                AllocationSize(std::mem::size_of::<$name>())
            }

            fn update(
                &mut self,
                wire_states: &WireStateList,
                mut output_states: OutputStateSlice<'_>,
            ) -> inline_vec!(WireId) {
                let lhs = wire_states.get_state(self.inputs[0]);
                let rhs = wire_states.get_state(self.inputs[1]);
                let (width, out) = output_states.get_data(self.output);
                let mut result = $op3(width, out, lhs, rhs);

                for &input in self.inputs.iter().skip(2) {
                    let rhs = wire_states.get_state(input);
                    result |= $op2(width, out, rhs);
                }

                result |= logic_not_1(width, out);

                match result {
                    OpResult::Unchanged => smallvec![],
                    OpResult::Changed => smallvec![self.output_wire],
                }
            }
        }
    };
}

wide_gate!(WideAndGate, logic_and_3, logic_and_2);
wide_gate!(WideOrGate, logic_or_3, logic_or_2);
wide_gate!(WideXorGate, logic_xor_3, logic_xor_2);
wide_gate_inv!(WideNandGate, logic_and_3, logic_and_2);
wide_gate_inv!(WideNorGate, logic_or_3, logic_or_2);
wide_gate_inv!(WideXnorGate, logic_xor_3, logic_xor_2);

#[derive(Debug)]
pub(crate) struct Merge {
    inputs: inline_vec!(WireStateId),
    output: OutputStateId,
    output_wire: WireId,
}

impl Merge {
    #[inline]
    pub(crate) fn new(
        inputs: impl Into<inline_vec!(WireStateId)>,
        output: OutputStateId,
        output_wire: WireId,
    ) -> Self {
        let inputs = inputs.into();
        debug_assert!(inputs.len() >= 1);

        Self {
            inputs,
            output,
            output_wire,
        }
    }
}

impl LargeComponent for Merge {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let (out_width, out) = output_states.get_data(self.output);

        const MAX_ATOM_COUNT: usize = NonZeroU8::MAX.get().div_ceil(Atom::BITS.get()) as usize;
        let mut tmp_state = [Atom::HIGH_Z; MAX_ATOM_COUNT];
        let tmp_state = &mut tmp_state[..out.len()];

        let mut shamnt = 0;
        for &input in &self.inputs {
            let width = wire_states.get_width(input);
            let val = wire_states.get_state(input);
            merge_one(tmp_state, width, val, shamnt);
            shamnt += width.get() as usize;
        }

        match copy(out_width, out, tmp_state) {
            OpResult::Unchanged => smallvec![],
            OpResult::Changed => smallvec![self.output_wire],
        }
    }
}

pub(crate) struct Adder {
    input_a: WireStateId,
    input_b: WireStateId,
    carry_in: WireStateId,
    output: OutputStateId,
    output_wire: WireId,
    carry_out: OutputStateId,
    carry_out_wire: WireId,
}

impl Adder {
    #[inline]
    pub(crate) fn new(
        input_a: WireStateId,
        input_b: WireStateId,
        carry_in: WireStateId,
        output: OutputStateId,
        output_wire: WireId,
        carry_out: OutputStateId,
        carry_out_wire: WireId,
    ) -> Self {
        Self {
            input_a,
            input_b,
            carry_in,
            output,
            output_wire,
            carry_out,
            carry_out_wire,
        }
    }
}

impl LargeComponent for Adder {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let lhs = wire_states.get_state(self.input_a);
        let rhs = wire_states.get_state(self.input_b);
        let cin = wire_states.get_state(self.carry_in);

        let carry_in = cin[0].get_bit_state(AtomOffset::MIN);
        let mut carry_out = LogicBitState::Undefined;
        let (width, out) = output_states.get_data(self.output);

        let sum_result = add(width, out, &mut carry_out, lhs, rhs, carry_in);

        let (_, cout) = output_states.get_data(self.carry_out);
        let carry_result = cout[0].get_bit_state(AtomOffset::MIN) != carry_out;
        cout[0] = carry_out.splat();

        match (sum_result, carry_result) {
            (OpResult::Unchanged, false) => smallvec![],
            (OpResult::Unchanged, true) => smallvec![self.carry_out_wire],
            (OpResult::Changed, false) => smallvec![self.output_wire],
            (OpResult::Changed, true) => {
                smallvec![self.output_wire, self.carry_out_wire]
            }
        }
    }
}

//#[derive(Debug)]
//pub(crate) struct Multiplier {
//    input_a: WireStateId,
//    input_b: WireStateId,
//    output_low: WireStateId,
//    output_high: WireStateId,
//}
//
//impl Multiplier {
//    #[inline]
//    pub(crate) fn new(
//        input_a: WireStateId,
//        input_b: WireStateId,
//        output_low: WireStateId,
//        output_high: WireStateId,
//    ) -> Self {
//        Self {
//            input_a,
//            input_b,
//            output_low,
//            output_high,
//        }
//    }
//}
//
//impl LargeComponent for Multiplier {
//    fn update(
//        &mut self,
//        wire_widths: &WireWidthList,
//        wire_states: &WireStateList,
//        outputs: &mut [Atom],
//    ) -> inline_vec!(WireId) {
//        let width = wire_widths[self.input_a];
//        let a = wire_states[self.input_a];
//        let b = wire_states[self.input_b];
//
//        let (new_low_state, new_high_state) = if (width.get() * 2) <= MAX_LOGIC_WIDTH {
//            let full_result = a.mul(b, width);
//
//            let shift_amount = unsafe {
//                // SAFETY: width is at most half of MAX_LOGIC_WIDTH, so it is always a valid offset.
//                AtomOffset::new_unchecked(width.get())
//            };
//
//            let high_result = Atom {
//                state: full_result.state >> shift_amount,
//                valid: full_result.valid >> shift_amount,
//            };
//
//            (full_result, high_result)
//        } else {
//            let mask = LogicStorage::mask(width);
//            let a_state = a.state & mask;
//            let b_state = b.state & mask;
//            let a_valid = a.valid | !mask;
//            let b_valid = b.valid | !mask;
//
//            if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
//                let (low_result, high_result) = a_state.widening_mul(b_state, width);
//
//                (
//                    Atom {
//                        state: low_result,
//                        valid: LogicStorage::ALL_ONE,
//                    },
//                    Atom {
//                        state: high_result,
//                        valid: LogicStorage::ALL_ONE,
//                    },
//                )
//            } else {
//                (Atom::UNDEFINED, Atom::UNDEFINED)
//            }
//        };
//
//        let low_changed = if !new_low_state.eq(outputs[0], width) {
//            outputs[0] = new_low_state;
//            true
//        } else {
//            false
//        };
//
//        let high_changed = if !new_high_state.eq(outputs[1], width) {
//            outputs[1] = new_high_state;
//            true
//        } else {
//            false
//        };
//
//        match (low_changed, high_changed) {
//            (true, true) => smallvec![self.output_low, self.output_high],
//            (true, false) => smallvec![self.output_low],
//            (false, true) => smallvec![self.output_high],
//            (false, false) => smallvec![],
//        }
//    }
//}

pub(crate) struct Multiplexer {
    inputs: inline_vec!(WireStateId),
    select: WireStateId,
    output: OutputStateId,
    output_wire: WireId,
}

impl Multiplexer {
    #[inline]
    pub(crate) fn new(
        inputs: impl Into<inline_vec!(WireStateId)>,
        select: WireStateId,
        output: OutputStateId,
        output_wire: WireId,
    ) -> Self {
        Self {
            inputs: inputs.into(),
            select,
            output,
            output_wire,
        }
    }
}

impl LargeComponent for Multiplexer {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let select_width = AtomWidth::new(wire_states.get_width(self.select).get())
            .expect("select signal too wide");
        let select = wire_states.get_state(self.select)[0];
        let (width, out) = output_states.get_data(self.output);

        let mut changed = false;
        let mut total_width = width.get();
        if select.is_valid(select_width) {
            let select_mask = LogicStorage::mask(select_width);
            let input_index = (select.state & select_mask).get() as usize;
            let input = wire_states.get_state(self.inputs[input_index]);

            for (out, &new) in izip!(out, input) {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                let new = new.high_z_to_undefined();
                if !out.eq(new, width) {
                    *out = new;
                    changed = true;
                }
            }
        } else {
            for out in out {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                if !out.eq(Atom::UNDEFINED, width) {
                    *out = Atom::UNDEFINED;
                    changed = true;
                }
            }
        }

        if changed {
            smallvec![self.output_wire]
        } else {
            smallvec![]
        }
    }
}

//#[derive(Debug)]
//pub(crate) struct PriorityDecoder {
//    inputs: inline_vec!(WireStateId),
//    output: WireStateId,
//}
//
//impl PriorityDecoder {
//    #[inline]
//    pub(crate) fn new(inputs: &[WireStateId], output: WireStateId) -> Self {
//        Self {
//            inputs: inputs.into(),
//            output,
//        }
//    }
//}
//
//impl LargeComponent for PriorityDecoder {
//    fn update(
//        &mut self,
//        wire_widths: &WireWidthList,
//        wire_states: &WireStateList,
//        outputs: &mut [Atom],
//    ) -> inline_vec!(WireId) {
//        let mut new_output_state = Atom::LOGIC_0;
//
//        for (i, input) in self.inputs.iter().copied().enumerate() {
//            match wire_states[input].get_bit_state(AtomOffset::MIN) {
//                LogicBitState::HighZ | LogicBitState::Undefined => {
//                    new_output_state = Atom::UNDEFINED;
//                    break;
//                }
//                LogicBitState::Logic1 => {
//                    new_output_state = Atom::from_int((i + 1) as u32);
//                    break;
//                }
//                LogicBitState::Logic0 => continue,
//            }
//        }
//
//        let output_width = wire_widths[self.output];
//        if !new_output_state.eq(outputs[0], output_width) {
//            outputs[0] = new_output_state;
//            smallvec![self.output]
//        } else {
//            smallvec![]
//        }
//    }
//}

struct ClockTrigger {
    prev: Option<bool>,
    polarity: ClockPolarity,
}

impl ClockTrigger {
    #[inline]
    const fn new(polarity: ClockPolarity) -> Self {
        Self {
            prev: None,
            polarity,
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.prev = None;
    }

    #[inline]
    fn update(&mut self, current: LogicBitState) -> bool {
        let current = match current {
            LogicBitState::HighZ | LogicBitState::Undefined => self.prev,
            LogicBitState::Logic0 => Some(false),
            LogicBitState::Logic1 => Some(true),
        };

        let edge = (self.prev == Some(self.polarity.inactive_state()))
            && (current == Some(self.polarity.active_state()));

        self.prev = current;
        edge
    }
}

pub(crate) struct Register {
    data_in: WireStateId,
    data_out: OutputStateId,
    data_out_wire: WireId,
    enable: WireStateId,
    clock: WireStateId,
    clock_trigger: ClockTrigger,
    data: inline_vec!(Atom),
}

impl Register {
    #[inline]
    pub(crate) fn new(
        width: NonZeroU8,
        data_in: WireStateId,
        data_out: OutputStateId,
        data_out_wire: WireId,
        enable: WireStateId,
        clock: WireStateId,
        clock_polarity: ClockPolarity,
    ) -> Self {
        let atom_count = width.safe_div_ceil(Atom::BITS).get() as usize;

        Self {
            data_in,
            data_out,
            data_out_wire,
            enable,
            clock,
            clock_trigger: ClockTrigger::new(clock_polarity),
            data: smallvec![Atom::UNDEFINED; atom_count],
        }
    }
}

impl LargeComponent for Register {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::RegisterValue(MemoryCell {
            mem: &mut self.data,
        })
    }

    fn reset(&mut self) {
        self.clock_trigger.reset();
        self.data.fill(Atom::UNDEFINED);
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let data_in = wire_states.get_state(self.data_in);
        let enable = wire_states.get_state(self.enable)[0].get_bit_state(AtomOffset::MIN);
        let clock = wire_states.get_state(self.clock)[0].get_bit_state(AtomOffset::MIN);

        if self.clock_trigger.update(clock) {
            match enable {
                LogicBitState::HighZ | LogicBitState::Undefined => {
                    self.data.fill(Atom::UNDEFINED);
                }
                LogicBitState::Logic0 => (),
                LogicBitState::Logic1 => {
                    for (dst, &src) in izip!(&mut self.data, data_in) {
                        *dst = src.high_z_to_undefined();
                    }
                }
            }
        }

        let (width, out) = output_states.get_data(self.data_out);
        let mut total_width = width.get();
        let mut changed = false;
        for (out, &new) in izip!(out, &self.data) {
            let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
            total_width -= width.get();

            if !out.eq(new, width) {
                *out = new;
                changed = true;
            }
        }

        if changed {
            smallvec![self.data_out_wire]
        } else {
            smallvec![]
        }
    }
}

enum Memory {
    U8(Box<[[u8; 2]]>),
    U16(Box<[[u16; 2]]>),
    U32(Box<[[u32; 2]]>),
    Big {
        atom_width: NonZeroU8,
        atoms: Box<[Atom]>,
    },
}

impl Memory {
    #[allow(clippy::unnecessary_cast)]
    fn new(width: NonZeroU8, len: usize) -> Self {
        const VALUE: (u32, u32) = Atom::UNDEFINED.to_state_valid();
        const STATE: u32 = VALUE.0;
        const VALID: u32 = VALUE.1;

        if width.get() <= 8 {
            let atoms = vec![[STATE as u8, VALID as u8]; len];
            Self::U8(atoms.into_boxed_slice())
        } else if width.get() <= 16 {
            let atoms = vec![[STATE as u16, VALID as u16]; len];
            Self::U16(atoms.into_boxed_slice())
        } else if width.get() <= 32 {
            let atoms = vec![[STATE as u32, VALID as u32]; len];
            Self::U32(atoms.into_boxed_slice())
        } else {
            let atom_width = width.safe_div_ceil(Atom::BITS);
            let atoms = vec![Atom::UNDEFINED; len * (atom_width.get() as usize)];
            Self::Big {
                atom_width,
                atoms: atoms.into_boxed_slice(),
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::U8(atoms) => atoms.len(),
            Self::U16(atoms) => atoms.len(),
            Self::U32(atoms) => atoms.len(),
            Self::Big { atom_width, atoms } => atoms.len() / (atom_width.get() as usize),
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn read(&self, addr: usize) -> LogicState {
        let (state, valid) = match self {
            Self::U8(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::U16(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::U32(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::Big { atom_width, atoms } => {
                let start = addr * (atom_width.get() as usize);
                let end = start + (atom_width.get() as usize);
                let slice = &atoms[start..end];
                return LogicState(LogicStateRepr::Bits(slice.iter().copied().collect()));
            }
        };

        let value = Atom::from_state_valid(state, valid);
        LogicState(LogicStateRepr::Bits(smallvec![value]))
    }

    #[allow(clippy::unnecessary_cast)]
    fn iter_cell(&self, addr: usize) -> MemoryCellIter<'_> {
        let (state, valid) = match self {
            Self::U8(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::U16(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::U32(atoms) => {
                let [state, valid] = atoms[addr];
                (state as u32, valid as u32)
            }
            Self::Big { atom_width, atoms } => {
                let start = addr * (atom_width.get() as usize);
                let end = start + (atom_width.get() as usize);
                let slice = &atoms[start..end];
                return MemoryCellIter::Multi(slice.iter());
            }
        };

        let value = Atom::from_state_valid(state, valid);
        MemoryCellIter::Single(Some(value))
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&mut self, addr: usize, mut value: impl Iterator<Item = Atom>) {
        match self {
            Self::U8(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                atoms[addr] = [state as u8, valid as u8];
            }
            Self::U16(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                atoms[addr] = [state as u16, valid as u16];
            }
            Self::U32(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                atoms[addr] = [state as u32, valid as u32];
            }
            Self::Big { atom_width, atoms } => {
                let start = addr * (atom_width.get() as usize);
                let end = start + (atom_width.get() as usize);
                let slice = &mut atoms[start..end];
                for (dst, src) in izip!(slice, value) {
                    *dst = src;
                }
            }
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn clear(&mut self) {
        const VALUE: (u32, u32) = Atom::UNDEFINED.to_state_valid();
        const STATE: u32 = VALUE.0;
        const VALID: u32 = VALUE.1;

        match self {
            Self::U8(atoms) => {
                atoms.fill([STATE as u8, VALID as u8]);
            }
            Self::U16(atoms) => {
                atoms.fill([STATE as u16, VALID as u16]);
            }
            Self::U32(atoms) => {
                atoms.fill([STATE as u32, VALID as u32]);
            }
            Self::Big { atoms, .. } => {
                atoms.fill(Atom::UNDEFINED);
            }
        }
    }
}

enum MemoryCellIter<'a> {
    Single(Option<Atom>),
    Multi(std::slice::Iter<'a, Atom>),
}

impl Iterator for MemoryCellIter<'_> {
    type Item = Atom;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MemoryCellIter::Single(value) => value.take(),
            MemoryCellIter::Multi(iter) => iter.next().copied(),
        }
    }
}

#[inline]
fn to_address(width: NonZeroU8, atoms: &[Atom]) -> Option<usize> {
    const MAX_ATOM_COUNT: usize = (std::mem::size_of::<usize>() * 8) / (Atom::BITS.get() as usize);

    let atom_count = width.safe_div_ceil(Atom::BITS).get() as usize;
    debug_assert!(atom_count <= MAX_ATOM_COUNT);

    let mut addr = 0;
    let mut total_width = width.get();
    for i in 0..atom_count {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        let atom = atoms[i];
        if !atom.is_valid(width) {
            return None;
        }

        let mask = LogicStorage::mask(width);
        let atom_value = (atom.state & mask).get() as usize;
        let shift = i * (Atom::BITS.get() as usize);
        addr |= atom_value << shift;
    }

    Some(addr)
}

pub(crate) struct Ram {
    write_addr: WireStateId,
    data_in: WireStateId,
    read_addr: WireStateId,
    data_out: OutputStateId,
    data_out_wire: WireId,
    write: WireStateId,
    clock: WireStateId,
    clock_trigger: ClockTrigger,
    mem: Memory,
}

impl Ram {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(crate) fn new(
        write_addr: WireStateId,
        data_in: WireStateId,
        read_addr: WireStateId,
        data_out: OutputStateId,
        data_out_wire: WireId,
        write: WireStateId,
        clock: WireStateId,
        clock_polarity: ClockPolarity,
        addr_width: NonZeroU8,
        data_width: NonZeroU8,
    ) -> Self {
        Self {
            write_addr,
            data_in,
            read_addr,
            data_out,
            data_out_wire,
            write,
            clock,
            clock_trigger: ClockTrigger::new(clock_polarity),
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }
}

impl LargeComponent for Ram {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::MemoryBlock(MemoryBlock { mem: &mut self.mem })
    }

    fn reset(&mut self) {
        self.clock_trigger.reset();
        self.mem.clear();
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let data_in = wire_states.get_state(self.data_in);
        let write = wire_states.get_state(self.write)[0].get_bit_state(AtomOffset::MIN);
        let clock = wire_states.get_state(self.clock)[0].get_bit_state(AtomOffset::MIN);

        if self.clock_trigger.update(clock) {
            let write_addr_width = wire_states.get_width(self.write_addr);
            let write_addr = wire_states.get_state(self.write_addr);
            let write_addr = to_address(write_addr_width, write_addr);

            if let Some(write_addr) = write_addr {
                match write {
                    LogicBitState::HighZ | LogicBitState::Undefined => {
                        let data_iter = std::iter::repeat(Atom::UNDEFINED);
                        self.mem.write(write_addr, data_iter);
                    }
                    LogicBitState::Logic0 => (),
                    LogicBitState::Logic1 => {
                        let data_iter = data_in.iter().copied().map(Atom::high_z_to_undefined);
                        self.mem.write(write_addr, data_iter);
                    }
                }
            } else {
                // NOTE:
                //   There is nothing sensible we can do here.
                //   In a real circuit a random address would be overwritten.
            }
        }

        let read_addr_width = wire_states.get_width(self.read_addr);
        let read_addr = wire_states.get_state(self.read_addr);
        let read_addr = to_address(read_addr_width, read_addr);

        let (width, out) = output_states.get_data(self.data_out);
        let mut total_width = width.get();
        let mut changed = false;
        if let Some(read_addr) = read_addr {
            for (out, new) in izip!(out, self.mem.iter_cell(read_addr)) {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                let new = new.high_z_to_undefined();
                if !out.eq(new, width) {
                    *out = new;
                    changed = true;
                }
            }
        } else {
            for out in out {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                if !out.eq(Atom::UNDEFINED, width) {
                    *out = Atom::UNDEFINED;
                    changed = true;
                }
            }
        }

        if changed {
            smallvec![self.data_out_wire]
        } else {
            smallvec![]
        }
    }
}

pub(crate) struct Rom {
    addr: WireStateId,
    data: OutputStateId,
    data_wire: WireId,
    mem: Memory,
}

impl Rom {
    #[inline]
    pub(crate) fn new(
        addr: WireStateId,
        data: OutputStateId,
        data_wire: WireId,
        addr_width: NonZeroU8,
        data_width: NonZeroU8,
    ) -> Self {
        Self {
            addr,
            data,
            data_wire,
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }
}

impl LargeComponent for Rom {
    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&mut self) -> ComponentData<'_> {
        ComponentData::MemoryBlock(MemoryBlock { mem: &mut self.mem })
    }

    fn reset(&mut self) {}

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let addr_width = wire_states.get_width(self.addr);
        let addr = wire_states.get_state(self.addr);
        let addr = to_address(addr_width, addr);

        let (width, out) = output_states.get_data(self.data);
        let mut total_width = width.get();
        let mut changed = false;
        if let Some(read_addr) = addr {
            for (out, new) in izip!(out, self.mem.iter_cell(read_addr)) {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                let new = new.high_z_to_undefined();
                if !out.eq(new, width) {
                    *out = new;
                    changed = true;
                }
            }
        } else {
            for out in out {
                let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                total_width -= width.get();

                if !out.eq(Atom::UNDEFINED, width) {
                    *out = Atom::UNDEFINED;
                    changed = true;
                }
            }
        }

        if changed {
            smallvec![self.data_wire]
        } else {
            smallvec![]
        }
    }
}

pub(crate) enum Component {
    Small {
        component: SmallComponent,
        output_base: OutputStateId,
        output_atom_count: u16,
    },
    Large {
        component: Box<dyn LargeComponent>,
        output_base: OutputStateId,
        output_atom_count: u16,
    },
}

impl Component {
    #[inline]
    pub(crate) fn new_small(
        component: SmallComponent,
        output_base: OutputStateId,
        output_atom_count: u16,
    ) -> Self {
        Self::Small {
            component,
            output_base,
            output_atom_count,
        }
    }

    #[inline]
    pub(crate) fn new_large<C: LargeComponent + 'static>(
        component: C,
        output_base: OutputStateId,
        output_atom_count: u16,
    ) -> Self {
        Self::Large {
            component: Box::new(component),
            output_base,
            output_atom_count,
        }
    }

    #[inline]
    pub(crate) fn alloc_size(&self) -> AllocationSize {
        match self {
            Component::Small { .. } => AllocationSize(0),
            Component::Large { component, .. } => component.alloc_size(),
        }
    }

    #[inline]
    pub(crate) fn output_range(&self) -> (OutputStateId, u16) {
        match self {
            &Self::Small {
                output_base,
                output_atom_count,
                ..
            }
            | &Self::Large {
                output_base,
                output_atom_count,
                ..
            } => (output_base, output_atom_count),
        }
    }

    #[inline]
    pub(crate) fn get_data(&mut self) -> ComponentData<'_> {
        match self {
            Self::Small { .. } => ComponentData::None,
            Self::Large { component, .. } => component.get_data(),
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        match self {
            Self::Small { .. } => {}
            Self::Large { component, .. } => component.reset(),
        }
    }

    #[inline]
    pub(crate) fn update(
        &mut self,
        wire_states: &WireStateList,
        output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        match self {
            &mut Self::Small {
                ref mut component,
                output_base,
                ..
            } => component.update(output_base, wire_states, output_states),
            Self::Large { component, .. } => component.update(wire_states, output_states),
        }
    }
}
