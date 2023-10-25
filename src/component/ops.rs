use crate::logic::{Atom, AtomOffset, AtomWidth, LogicBitState, LogicStorage};
use itertools::izip;
use std::num::NonZeroU8;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpResult {
    Unchanged,
    Changed,
}

impl BitAnd for OpResult {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Changed, Self::Changed) => Self::Changed,
            _ => Self::Unchanged,
        }
    }
}

impl BitAndAssign for OpResult {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for OpResult {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
            _ => Self::Changed,
        }
    }
}

impl BitOrAssign for OpResult {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

trait Perform<'a>: IntoIterator<Item = &'a mut Atom> + Sized
where
    Self::IntoIter: Iterator + ExactSizeIterator,
{
    #[inline]
    fn perform_no_input<'out, Op>(self, width: NonZeroU8, mut op: Op) -> Option<OpResult>
    where
        Op: FnMut(AtomWidth, Atom) -> Option<Atom>,
    {
        let mut result = OpResult::Unchanged;

        let mut iter = self.into_iter();
        while iter.len() > 1 {
            let out = iter.next().unwrap();

            let new = op(AtomWidth::MAX, *out)?;
            if !out.eq(new, AtomWidth::MAX) {
                *out = new;
                result = OpResult::Changed;
            }
        }

        let tail_width = AtomWidth::new(width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);
        let out = iter.next().unwrap();

        let new = op(tail_width, *out)?;
        if !out.eq(new, tail_width) {
            *out = new;
            result = OpResult::Changed;
        }

        Some(result)
    }

    #[inline]
    fn perform<'out, Input, Op>(
        self,
        width: NonZeroU8,
        input: Input,
        mut op: Op,
    ) -> Option<OpResult>
    where
        Input: IntoIterator,
        Input::IntoIter: ExactSizeIterator,
        Op: FnMut(AtomWidth, Atom, Input::Item) -> Option<Atom>,
    {
        let mut input = input.into_iter();
        self.perform_no_input(width, |tail_width, out| {
            op(tail_width, out, input.next().unwrap())
        })
    }
}

impl<'a, T> Perform<'a> for T
where
    T: IntoIterator<Item = &'a mut Atom> + Sized,
    T::IntoIter: Iterator + ExactSizeIterator,
{
}

#[inline]
fn logic_and_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: (a.state & b.state)
            | (!a.valid & !b.valid)
            | (a.state & !b.valid)
            | (b.state & !a.valid),
        valid: (a.valid & b.valid) | (!a.state & a.valid) | (!b.state & b.valid),
    }
}

#[inline]
fn logic_or_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: a.state | !a.valid | b.state | !b.valid,
        valid: (a.state & a.valid) | (b.state & b.valid) | (a.valid & b.valid),
    }
}

#[inline]
fn logic_xor_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: (a.state ^ b.state) | !a.valid | !b.valid,
        valid: a.valid & b.valid,
    }
}

#[inline]
fn logic_nand_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: !a.state | !a.valid | !b.state | !b.valid,
        valid: (a.valid & b.valid) | (!a.state & a.valid) | (!b.state & b.valid),
    }
}

#[inline]
fn logic_nor_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: (!a.state & !b.state)
            | (!a.valid & !b.valid)
            | (!a.state & !b.valid)
            | (!b.state & !a.valid),
        valid: (a.state & a.valid) | (b.state & b.valid) | (a.valid & b.valid),
    }
}

#[inline]
fn logic_xnor_impl(a: Atom, b: Atom) -> Atom {
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

    Atom {
        state: !(a.state ^ b.state) | !a.valid | !b.valid,
        valid: a.valid & b.valid,
    }
}

macro_rules! def_binary_op {
    ($op_impl:ident -> $name3:ident, $name2:ident) => {
        pub(super) fn $name3(
            width: NonZeroU8,
            out: &mut [Atom],
            lhs: &[Atom],
            rhs: &[Atom],
        ) -> OpResult {
            out.perform(width, izip!(lhs, rhs), |_, _, (&a, &b)| {
                Some($op_impl(a, b))
            })
            .unwrap()
        }

        #[allow(dead_code)]
        pub(super) fn $name2(width: NonZeroU8, out: &mut [Atom], rhs: &[Atom]) -> OpResult {
            out.perform(width, rhs, |_, a, &b| Some($op_impl(a, b)))
                .unwrap()
        }
    };
}

def_binary_op!(logic_and_impl -> logic_and_3, logic_and_2);
def_binary_op!(logic_or_impl -> logic_or_3, logic_or_2);
def_binary_op!(logic_xor_impl -> logic_xor_3, logic_xor_2);
def_binary_op!(logic_nand_impl -> logic_nand_3, logic_nand_2);
def_binary_op!(logic_nor_impl -> logic_nor_3, logic_nor_2);
def_binary_op!(logic_xnor_impl -> logic_xnor_3, logic_xnor_2);

#[inline]
fn logic_not_impl(v: Atom) -> Atom {
    //  I state | I valid | I meaning | O state | O valid | O meaning
    // ---------|---------|-----------|---------|---------|-----------
    //     0    |    0    | High-Z    |    1    |    0    | Undefined
    //     1    |    0    | Undefined |    1    |    0    | Undefined
    //     0    |    1    | Logic 0   |    1    |    1    | Logic 1
    //     1    |    1    | Logic 1   |    0    |    1    | Logic 0

    Atom {
        state: !v.state | !v.valid,
        valid: v.valid,
    }
}

macro_rules! def_unary_op {
    ($op_impl:ident -> $name2:ident, $name1:ident) => {
        pub(super) fn $name2(width: NonZeroU8, out: &mut [Atom], val: &[Atom]) -> OpResult {
            out.perform(width, val, |_, _, &v| Some($op_impl(v)))
                .unwrap()
        }

        pub(super) fn $name1(width: NonZeroU8, out: &mut [Atom]) -> OpResult {
            out.perform_no_input(width, |_, v| Some($op_impl(v)))
                .unwrap()
        }
    };
}

def_unary_op!(logic_not_impl -> logic_not_2, logic_not_1);

pub(super) fn buffer(
    width: NonZeroU8,
    out: &mut [Atom],
    val: &[Atom],
    en: LogicBitState,
) -> OpResult {
    match en {
        LogicBitState::Undefined => out.perform_no_input(width, |_, _| Some(Atom::UNDEFINED)),
        LogicBitState::Logic1 => {
            out.perform(width, val, |_, _, &val| Some(val.high_z_to_undefined()))
        }
        _ => out.perform_no_input(width, |_, _| Some(Atom::HIGH_Z)),
    }
    .unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AddMode {
    Add,
    Subtract,
}

pub(super) fn add(
    width: NonZeroU8,
    out: &mut [Atom],
    carry_out: &mut LogicBitState,
    lhs: &[Atom],
    rhs: &[Atom],
    carry_in: LogicBitState,
    mode: AddMode,
) -> OpResult {
    if let Some(mut carry) = carry_in.to_bool() {
        let result = out.perform(width, izip!(lhs, rhs), |width, _, (&lhs, &rhs)| {
            if !lhs.is_valid(width) || !rhs.is_valid(width) {
                return None;
            }

            let rhs_state = match mode {
                AddMode::Add => rhs.state,
                AddMode::Subtract => !rhs.state,
            };

            let new;
            (new, carry) = lhs.state.carrying_add(rhs_state, carry);

            *carry_out = if let Some(offset) = AtomOffset::new(width.get()) {
                new.get_bit(offset)
            } else {
                carry
            }
            .into();

            Some(Atom {
                state: new,
                valid: LogicStorage::ALL_ONE,
            })
        });

        if let Some(result) = result {
            return result;
        }
    }

    *carry_out = LogicBitState::Undefined;
    out.perform_no_input(width, |_, _| Some(Atom::UNDEFINED))
        .unwrap()
}

//#[inline]
//pub(super) fn mul(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
//        Atom {
//            state: a_state * b_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}
//
//#[inline]
//pub(super) fn div(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE)
//        && (b_valid == LogicStorage::ALL_ONE)
//        && (b_state != LogicStorage::ALL_ZERO)
//    {
//        Atom {
//            state: a_state / b_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}
//
//#[inline]
//pub(super) fn rem(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE)
//        && (b_valid == LogicStorage::ALL_ONE)
//        && (b_state != LogicStorage::ALL_ZERO)
//    {
//        Atom {
//            state: a_state % b_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}

//#[inline]
//pub(super) fn shl(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
//        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
//            LogicStorage::ALL_ZERO
//        } else {
//            let shift_amount = unsafe {
//                // SAFETY: we just checked that `b_state` is within the required range.
//                LogicOffset::new_unchecked(b_state.0 as u8)
//            };
//
//            a_state << shift_amount
//        };
//
//        Atom {
//            state: result_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}
//
//#[inline]
//pub(super) fn lshr(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
//        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
//            LogicStorage::ALL_ZERO
//        } else {
//            let shift_amount = unsafe {
//                // SAFETY: we just checked that `b_state` is within the required range.
//                LogicOffset::new_unchecked(b_state.0 as u8)
//            };
//
//            a_state >> shift_amount
//        };
//
//        Atom {
//            state: result_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}
//
//#[inline]
//pub(super) fn ashr(a: Atom, b: Atom, width: LogicWidth) -> Atom {
//    let mask = LogicStorage::mask(width);
//    let a_state = a.state & mask;
//    let b_state = b.state & mask;
//    let a_valid = a.valid | !mask;
//    let b_valid = b.valid | !mask;
//
//    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
//        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
//            LogicStorage::ALL_ZERO
//        } else {
//            let shift_amount = unsafe {
//                // SAFETY: we just checked that `b_state` is within the required range.
//                LogicOffset::new_unchecked(b_state.0 as u8)
//            };
//
//            let logical_shift = a_state >> shift_amount;
//
//            let post_shift_width = width.get() - shift_amount.get();
//            let arithmetic_shift_amount = unsafe {
//                // SAFETY:
//                //   shift_amount < width <= MAX_LOGIC_WIDTH
//                //   => 0 < post_shift_width <= MAX_LOGIC_WIDTH
//                //      => 0 <= arithmetic_shift_amount < MAX_LOGIC_WIDTH
//                LogicOffset::new_unchecked(MAX_LOGIC_WIDTH - post_shift_width)
//            };
//
//            (logical_shift << arithmetic_shift_amount).ashr(arithmetic_shift_amount)
//        };
//
//        Atom {
//            state: result_state,
//            valid: LogicStorage::ALL_ONE,
//        }
//    } else {
//        Atom::UNDEFINED
//    }
//}

#[inline]
fn reduce_atom<Op>(mut val: Atom, mut op: Op) -> Atom
where
    Op: FnMut(Atom, Atom) -> Atom,
{
    const O16: AtomOffset = unsafe { AtomOffset::new_unchecked(16) };
    const O8: AtomOffset = unsafe { AtomOffset::new_unchecked(8) };
    const O4: AtomOffset = unsafe { AtomOffset::new_unchecked(4) };
    const O2: AtomOffset = unsafe { AtomOffset::new_unchecked(2) };
    const O1: AtomOffset = unsafe { AtomOffset::new_unchecked(1) };

    let sh16 = Atom {
        state: val.state >> O16,
        valid: val.valid >> O16,
    };
    val = op(val, sh16);

    let sh8 = Atom {
        state: val.state >> O8,
        valid: val.valid >> O8,
    };
    val = op(val, sh8);

    let sh4 = Atom {
        state: val.state >> O4,
        valid: val.valid >> O4,
    };
    val = op(val, sh4);

    let sh2 = Atom {
        state: val.state >> O2,
        valid: val.valid >> O2,
    };
    val = op(val, sh2);

    let sh1 = Atom {
        state: val.state >> O1,
        valid: val.valid >> O1,
    };
    val = op(val, sh1);

    val
}

// TODO:
//   Maybe the horizontal operations can be implemented more efficiently using popcount.
//   This needs further investigation as it is unclear how to handle the Z and X states.

#[inline]
pub(super) fn horizontal_logic_and_impl(width: NonZeroU8, val: &[Atom]) -> Atom {
    let mut new = Atom::LOGIC_1;

    let mut total_width = width.get();
    for &(mut val) in val {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        if width < AtomWidth::MAX {
            // set bits outside the width to Logic1
            let mask = LogicStorage::mask(width);
            val.state |= !mask;
            val.valid |= !mask;
        }

        new = logic_and_impl(new, val);
    }

    reduce_atom(new, logic_and_impl)
}

#[inline]
pub(super) fn horizontal_logic_or_impl(width: NonZeroU8, val: &[Atom]) -> Atom {
    let mut new = Atom::LOGIC_0;

    let mut total_width = width.get();
    for &(mut val) in val {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        if width < AtomWidth::MAX {
            // set bits outside the width to Logic0
            let mask = LogicStorage::mask(width);
            val.state &= mask;
            val.valid |= !mask;
        }

        new = logic_or_impl(new, val);
    }

    reduce_atom(new, logic_or_impl)
}

pub(super) fn horizontal_logic_xor_impl(width: NonZeroU8, val: &[Atom]) -> Atom {
    let mut new = Atom::LOGIC_0;

    let mut total_width = width.get();
    for &(mut val) in val {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        if width < AtomWidth::MAX {
            // set bits outside the width to Logic0
            let mask = LogicStorage::mask(width);
            val.state &= mask;
            val.valid |= !mask;
        }

        new = logic_xor_impl(new, val);
    }

    reduce_atom(new, logic_xor_impl)
}

pub(super) fn horizontal_logic_and(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_and_impl(width, val);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn horizontal_logic_or(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_or_impl(width, val);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn horizontal_logic_xor(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_xor_impl(width, val);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn horizontal_logic_nand(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_and_impl(width, val);
    let new = logic_not_impl(new);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn horizontal_logic_nor(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_or_impl(width, val);
    let new = logic_not_impl(new);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn horizontal_logic_xnor(width: NonZeroU8, out: &mut Atom, val: &[Atom]) -> OpResult {
    let new = horizontal_logic_xor_impl(width, val);
    let new = logic_not_impl(new);

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn equal(width: NonZeroU8, out: &mut Atom, lhs: &[Atom], rhs: &[Atom]) -> OpResult {
    let mut new = Atom::LOGIC_1;

    let mut total_width = width.get();
    for (&lhs, &rhs) in izip!(lhs, rhs) {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        if !lhs.is_valid(width) || !rhs.is_valid(width) {
            new = Atom::UNDEFINED;
            break;
        }

        if !lhs.eq(rhs, width) {
            new = Atom::LOGIC_0;
        }
    }

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn not_equal(width: NonZeroU8, out: &mut Atom, lhs: &[Atom], rhs: &[Atom]) -> OpResult {
    let mut new = Atom::LOGIC_0;

    let mut total_width = width.get();
    for (&lhs, &rhs) in izip!(lhs, rhs) {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

        if !lhs.is_valid(width) || !rhs.is_valid(width) {
            new = Atom::UNDEFINED;
            break;
        }

        if !lhs.eq(rhs, width) {
            new = Atom::LOGIC_1;
        }
    }

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

#[inline]
fn cmp<Cmp>(width: NonZeroU8, out: &mut Atom, lhs: &[Atom], rhs: &[Atom], inv_cmp: Cmp) -> OpResult
where
    Cmp: Fn(LogicStorage, LogicStorage) -> bool,
{
    let mut new = Atom::LOGIC_1;

    'valid: {
        let mut iter = izip!(lhs, rhs).rev();

        let head_width = AtomWidth::new(width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);
        if let Some((&lhs, &rhs)) = iter.next() {
            if !lhs.is_valid(head_width) || !rhs.is_valid(head_width) {
                new = Atom::UNDEFINED;
                break 'valid;
            }

            let mask = LogicStorage::mask(head_width);
            if inv_cmp(lhs.state & mask, rhs.state & mask) {
                new = Atom::LOGIC_0;
            }
        }

        for (&lhs, &rhs) in iter {
            if !lhs.is_valid(AtomWidth::MAX) || !rhs.is_valid(AtomWidth::MAX) {
                new = Atom::UNDEFINED;
                break 'valid;
            }

            if inv_cmp(lhs.state, rhs.state) {
                new = Atom::LOGIC_0;
            }
        }
    }

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn less_than(width: NonZeroU8, out: &mut Atom, lhs: &[Atom], rhs: &[Atom]) -> OpResult {
    cmp(width, out, lhs, rhs, |a, b| a >= b)
}

pub(super) fn greater_than(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp(width, out, lhs, rhs, |a, b| a <= b)
}

pub(super) fn less_than_or_equal(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp(width, out, lhs, rhs, |a, b| a > b)
}

pub(super) fn greater_than_or_equal(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp(width, out, lhs, rhs, |a, b| a < b)
}

#[inline]
fn cmp_signed<Cmp>(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
    inv_cmp: Cmp,
) -> OpResult
where
    Cmp: Fn(LogicStorage, LogicStorage, AtomWidth) -> bool,
{
    let mut new = Atom::LOGIC_1;

    'valid: {
        let mut iter = izip!(lhs, rhs).rev();

        let head_width = AtomWidth::new(width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);
        if let Some((&lhs, &rhs)) = iter.next() {
            if !lhs.is_valid(head_width) || !rhs.is_valid(head_width) {
                new = Atom::UNDEFINED;
                break 'valid;
            }

            let mask = LogicStorage::mask(head_width);
            if inv_cmp(lhs.state & mask, rhs.state & mask, head_width) {
                new = Atom::LOGIC_0;
            }
        }

        for (&lhs, &rhs) in iter {
            if !lhs.is_valid(AtomWidth::MAX) || !rhs.is_valid(AtomWidth::MAX) {
                new = Atom::UNDEFINED;
                break 'valid;
            }

            if inv_cmp(lhs.state, rhs.state, AtomWidth::MAX) {
                new = Atom::LOGIC_0;
            }
        }
    }

    if !out.eq(new, AtomWidth::MIN) {
        *out = new;
        OpResult::Changed
    } else {
        OpResult::Unchanged
    }
}

pub(super) fn less_than_signed(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp_signed(width, out, lhs, rhs, LogicStorage::ges)
}

pub(super) fn greater_than_signed(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp_signed(width, out, lhs, rhs, LogicStorage::les)
}

pub(super) fn less_than_or_equal_signed(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp_signed(width, out, lhs, rhs, LogicStorage::gts)
}

pub(super) fn greater_than_or_equal_signed(
    width: NonZeroU8,
    out: &mut Atom,
    lhs: &[Atom],
    rhs: &[Atom],
) -> OpResult {
    cmp_signed(width, out, lhs, rhs, LogicStorage::lts)
}

pub(super) fn zero_extend(
    val_width: NonZeroU8,
    out_width: NonZeroU8,
    val: &[Atom],
    out: &mut [Atom],
) -> OpResult {
    let val_tail_width =
        AtomWidth::new(val_width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);
    let out_tail_width =
        AtomWidth::new(out_width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);

    let mut val_iter = val.into_iter();
    let mut out_iter = out.into_iter();

    let mut result = OpResult::Unchanged;

    while val_iter.len() > 1 {
        let val = *val_iter.next().unwrap();
        let out = out_iter.next().unwrap();

        if !out.eq(val, AtomWidth::MAX) {
            *out = val;
            result = OpResult::Changed;
        }
    }

    let val = *val_iter.next().unwrap();
    let out = out_iter.next().unwrap();

    let mask = LogicStorage::mask(val_tail_width);
    let extend = Atom::LOGIC_0;
    let val = Atom {
        state: (val.state & mask) | (extend.state & !mask),
        valid: (val.valid & mask) | (extend.valid & !mask),
    };

    let tail_width = if out_iter.len() == 0 {
        out_tail_width
    } else {
        AtomWidth::MAX
    };

    if !out.eq(val, tail_width) {
        *out = val;
        result = OpResult::Changed;
    }

    if out_iter.len() > 0 {
        while out_iter.len() > 1 {
            let out = out_iter.next().unwrap();

            if !out.eq(extend, AtomWidth::MAX) {
                *out = extend;
                result = OpResult::Changed;
            }
        }

        let out = out_iter.next().unwrap();

        if !out.eq(extend, out_tail_width) {
            *out = extend;
            result = OpResult::Changed;
        }
    }

    result
}

pub(super) fn sign_extend(
    val_width: NonZeroU8,
    out_width: NonZeroU8,
    val: &[Atom],
    out: &mut [Atom],
) -> OpResult {
    let val_tail_width =
        AtomWidth::new(val_width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);
    let out_tail_width =
        AtomWidth::new(out_width.get() % Atom::BITS.get()).unwrap_or(AtomWidth::MAX);

    let mut val_iter = val.into_iter();
    let mut out_iter = out.into_iter();

    let mut result = OpResult::Unchanged;

    while val_iter.len() > 1 {
        let val = *val_iter.next().unwrap();
        let out = out_iter.next().unwrap();

        if !out.eq(val, AtomWidth::MAX) {
            *out = val;
            result = OpResult::Changed;
        }
    }

    let val = *val_iter.next().unwrap();
    let out = out_iter.next().unwrap();

    let mask = LogicStorage::mask(val_tail_width);
    let extend = val
        .get_bit_state(AtomOffset::new(val_tail_width.get() - 1).unwrap())
        .splat();
    let val = Atom {
        state: (val.state & mask) | (extend.state & !mask),
        valid: (val.valid & mask) | (extend.valid & !mask),
    };

    let tail_width = if out_iter.len() == 0 {
        out_tail_width
    } else {
        AtomWidth::MAX
    };

    if !out.eq(val, tail_width) {
        *out = val;
        result = OpResult::Changed;
    }

    if out_iter.len() > 0 {
        while out_iter.len() > 1 {
            let out = out_iter.next().unwrap();

            if !out.eq(extend, AtomWidth::MAX) {
                *out = extend;
                result = OpResult::Changed;
            }
        }

        let out = out_iter.next().unwrap();

        if !out.eq(extend, out_tail_width) {
            *out = extend;
            result = OpResult::Changed;
        }
    }

    result
}
