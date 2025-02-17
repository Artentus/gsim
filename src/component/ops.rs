use crate::{bit_width, logic::*};
use crate::{CLog2, SafeDivCeil};
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

#[inline]
pub(super) fn high_z_to_undefined(i: [u32; 2]) -> [u32; 2] {
    //  I_1 | I_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined

    [i[0] | i[1], i[1]]
}

#[inline]
pub(super) fn logic_not(i: [u32; 2]) -> [u32; 2] {
    //  I_1 | I_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined

    [!i[0] | i[1], i[1]]
}

#[inline]
pub(super) fn logic_and(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  0  |  0  | Logic 0
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [
        (a[0] & b[0]) | (a[0] & b[1]) | (a[1] & b[0]) | (a[1] & b[1]),
        (a[0] & b[1]) | (a[1] & b[0]) | (a[1] & b[1]),
    ]
}

#[inline]
pub(super) fn logic_or(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  0  |  1  | Logic 1
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [
        a[0] | a[1] | b[0] | b[1],
        (!a[0] & b[1]) | (a[1] & !b[0]) | (a[1] & b[1]),
    ]
}

#[inline]
pub(super) fn logic_xor(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [(!a[0] & b[0]) | (a[0] & !b[0]) | a[1] | b[1], a[1] | b[1]]
}

#[inline]
pub(super) fn logic_nand(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  0  |  1  | Logic 1
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [
        !a[0] | !b[0] | a[1] | b[1],
        (a[0] & b[1]) | (a[1] & b[0]) | (a[1] & b[1]),
    ]
}

#[inline]
pub(super) fn logic_nor(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  0  |  0  | Logic 0
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [
        (!a[0] & !b[0]) | (!a[0] & b[1]) | (a[1] & !b[0]) | (a[1] & b[1]),
        (!a[0] & b[1]) | (a[1] & !b[0]) | (a[1] & b[1]),
    ]
}

#[inline]
pub(super) fn logic_xnor(a: [u32; 2], b: [u32; 2]) -> [u32; 2] {
    //  A_1 | A_0 |     A     | B_1 | B_0 |     B     | O_1 | O_0 |     O
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  0  | Logic 0   |  0  |  0  | Logic 0   |  0  |  1  | Logic 1
    //   0  |  0  | Logic 0   |  0  |  1  | Logic 1   |  0  |  0  | Logic 0
    //   0  |  0  | Logic 0   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  0  | Logic 0   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   0  |  1  | Logic 1   |  0  |  0  | Logic 0   |  0  |  0  | Logic 0
    //   0  |  1  | Logic 1   |  0  |  1  | Logic 1   |  0  |  1  | Logic 1
    //   0  |  1  | Logic 1   |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   0  |  1  | Logic 1   |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  0  | High-Z    |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  0  | High-Z    |  1  |  1  | Undefined |  1  |  1  | Undefined
    // -----|-----|-----------|-----|-----|-----------|-----|-----|-----------
    //   1  |  1  | Undefined |  0  |  0  | Logic 0   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  0  |  1  | Logic 1   |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  0  | High-Z    |  1  |  1  | Undefined
    //   1  |  1  | Undefined |  1  |  1  | Undefined |  1  |  1  | Undefined

    [(!a[0] & !b[0]) | (a[0] & b[0]) | a[1] | b[1], a[1] | b[1]]
}

#[inline]
pub(super) fn add(a: [u32; 2], b: [u32; 2], c: LogicBitState) -> ([u32; 2], LogicBitState) {
    let (sum, c0) = a[0].overflowing_add(b[0]);
    let (sum, c1) = sum.overflowing_add((c as u32) & 0b1);

    let valid_count = if c.to_bool().is_some() {
        (a[1] | b[1]).trailing_zeros()
    } else {
        0
    };

    let invalid_mask = if valid_count >= 32 {
        0
    } else {
        u32::MAX << valid_count
    };

    (
        [sum | invalid_mask, invalid_mask],
        if invalid_mask > 0 {
            LogicBitState::Undefined
        } else {
            LogicBitState::from_bool(c0 | c1)
        },
    )
}

#[inline]
pub(super) fn sub(a: [u32; 2], b: [u32; 2], c: LogicBitState) -> ([u32; 2], LogicBitState) {
    add(a, [!b[0], b[1]], c)
}

#[inline]
pub(super) fn unary_op(
    mut output: LogicStateMut,
    input: LogicStateRef,
    op: impl Fn([u32; 2]) -> [u32; 2],
) {
    assert_eq!(output.bit_width(), input.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (output_plane_0, output_plane_1) = output.bit_planes_mut();
    let (input_plane_0, input_plane_1) = input.bit_planes();
    for i in 0..word_len {
        [output_plane_0[i], output_plane_1[i]] = op([input_plane_0[i], input_plane_1[i]]);
    }
}

#[inline]
pub(super) fn unary_op_mut(mut output: LogicStateMut, op: impl Fn([u32; 2]) -> [u32; 2]) {
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (output_plane_0, output_plane_1) = output.bit_planes_mut();
    for i in 0..word_len {
        [output_plane_0[i], output_plane_1[i]] = op([output_plane_0[i], output_plane_1[i]]);
    }
}

#[inline]
pub(super) fn binary_op(
    mut output: LogicStateMut,
    input_a: LogicStateRef,
    input_b: LogicStateRef,
    op: impl Fn([u32; 2], [u32; 2]) -> [u32; 2],
) {
    assert_eq!(output.bit_width(), input_a.bit_width());
    assert_eq!(output.bit_width(), input_b.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (output_plane_0, output_plane_1) = output.bit_planes_mut();
    let (input_a_plane_0, input_a_plane_1) = input_a.bit_planes();
    let (input_b_plane_0, input_b_plane_1) = input_b.bit_planes();

    for i in 0..word_len {
        [output_plane_0[i], output_plane_1[i]] = op(
            [input_a_plane_0[i], input_a_plane_1[i]],
            [input_b_plane_0[i], input_b_plane_1[i]],
        );
    }
}

#[inline]
pub(super) fn binary_op_mut(
    mut output: LogicStateMut,
    input: LogicStateRef,
    op: impl Fn([u32; 2], [u32; 2]) -> [u32; 2],
) {
    assert_eq!(output.bit_width(), input.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (output_plane_0, output_plane_1) = output.bit_planes_mut();
    let (input_plane_0, input_plane_1) = input.bit_planes();

    for i in 0..word_len {
        [output_plane_0[i], output_plane_1[i]] = op(
            [output_plane_0[i], output_plane_1[i]],
            [input_plane_0[i], input_plane_1[i]],
        );
    }
}

#[inline]
pub(super) fn carrying_binary_op(
    mut sum: LogicStateMut,
    input_a: LogicStateRef,
    input_b: LogicStateRef,
    carry_in: LogicBitState,
    op: impl Fn([u32; 2], [u32; 2], LogicBitState) -> ([u32; 2], LogicBitState),
) {
    assert_eq!(sum.bit_width(), input_a.bit_width());
    assert_eq!(sum.bit_width(), input_b.bit_width());
    let bit_width = sum.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (sum_plane_0, sum_plane_1) = sum.bit_planes_mut();
    let (input_a_plane_0, input_a_plane_1) = input_a.bit_planes();
    let (input_b_plane_0, input_b_plane_1) = input_b.bit_planes();

    let mut carry = carry_in;
    for i in 0..word_len {
        ([sum_plane_0[i], sum_plane_1[i]], carry) = op(
            [input_a_plane_0[i], input_a_plane_1[i]],
            [input_b_plane_0[i], input_b_plane_1[i]],
            carry,
        );
    }
}

fn carrying_mul(a: u32, b: u32, c: u32, d: u32) -> (u32, u32) {
    let product = (a as u64) * (b as u64) + (c as u64) + (d as u64);
    (product as u32, (product >> 32) as u32)
}

#[inline]
pub(super) fn mul(mut product: LogicStateMut, input_a: LogicStateRef, input_b: LogicStateRef) {
    assert_eq!(product.bit_width(), input_a.bit_width());
    assert_eq!(product.bit_width(), input_b.bit_width());
    let bit_width = product.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (product_plane_0, product_plane_1) = product.bit_planes_mut();
    let (input_a_plane_0, input_a_plane_1) = input_a.bit_planes();
    let (input_b_plane_0, input_b_plane_1) = input_b.bit_planes();

    product_plane_0.fill(0);

    for i_a in 0..word_len {
        let mut carry = 0;
        for i_b in 0..(word_len - i_a) {
            let i = i_a + i_b;

            (product_plane_0[i], carry) = carrying_mul(
                input_a_plane_0[i_a],
                input_b_plane_0[i_b],
                carry,
                product_plane_0[i],
            );
        }
    }

    for i in 0..word_len {
        let valid_count = (input_a_plane_1[i] | input_b_plane_1[i]).trailing_zeros();
        let invalid_mask = if valid_count >= 32 {
            0
        } else {
            u32::MAX << valid_count
        };

        product_plane_0[i] |= invalid_mask;
        product_plane_1[i] = invalid_mask;

        if invalid_mask > 0 {
            product_plane_0[(i + 1)..].fill(u32::MAX);
            product_plane_1[(i + 1)..].fill(u32::MAX);
            break;
        }
    }
}

#[inline]
pub(super) fn shift_left(
    mut output: LogicStateMut,
    input: LogicStateRef,
    shift_amount: LogicStateRef,
) {
    assert_eq!(output.bit_width(), input.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    if let Some(shift_width) = bit_width.clog2() {
        let (output_plane_0, output_plane_1) = output.bit_planes_mut();
        let (input_plane_0, input_plane_1) = input.bit_planes();
        let (shift_amount_plane_0, shift_amount_plane_1) = shift_amount.bit_planes();

        let shift_mask = shift_width.last_word_mask();
        let (shift_amount, shift_amount_valid) = (
            shift_amount_plane_0[0] & shift_mask,
            shift_amount_plane_1[0] & shift_mask,
        );

        if shift_amount_valid == 0 {
            let word_shift = (shift_amount / u32::BITS) as usize;
            let bit_shift = shift_amount % u32::BITS;

            for dst_index in 0..word_len {
                if let Some(src_index_high) = dst_index.checked_sub(word_shift) {
                    output_plane_0[dst_index] = input_plane_0[src_index_high] << bit_shift;
                    output_plane_1[dst_index] = input_plane_1[src_index_high] << bit_shift;

                    if bit_shift > 0 {
                        if let Some(src_index_low) = src_index_high.checked_sub(1) {
                            output_plane_0[dst_index] |=
                                input_plane_0[src_index_low] >> (u32::BITS - bit_shift);
                            output_plane_1[dst_index] |=
                                input_plane_1[src_index_low] >> (u32::BITS - bit_shift);
                        }
                    }

                    output_plane_0[dst_index] |= output_plane_1[dst_index]; // high-z to undefined
                } else {
                    output_plane_0[dst_index] = 0;
                    output_plane_1[dst_index] = 0;
                }
            }
        } else {
            output_plane_0.fill(u32::MAX);
            output_plane_1.fill(u32::MAX);
        }
    } else {
        output.copy_from(input);
    }
}

#[inline]
pub(super) fn shift_right_logical(
    mut output: LogicStateMut,
    input: LogicStateRef,
    shift_amount: LogicStateRef,
) {
    assert_eq!(output.bit_width(), input.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    if let Some(shift_width) = bit_width.clog2() {
        let (output_plane_0, output_plane_1) = output.bit_planes_mut();
        let (input_plane_0, input_plane_1) = input.bit_planes();
        let (shift_amount_plane_0, shift_amount_plane_1) = shift_amount.bit_planes();

        let shift_mask = shift_width.last_word_mask();
        let (shift_amount, shift_amount_valid) = (
            shift_amount_plane_0[0] & shift_mask,
            shift_amount_plane_1[0] & shift_mask,
        );

        if shift_amount_valid == 0 {
            let word_shift = (shift_amount / u32::BITS) as usize;
            let bit_shift = shift_amount % u32::BITS;

            for dst_index in 0..word_len {
                let src_index_low = dst_index + word_shift;
                if src_index_low < word_len {
                    output_plane_0[dst_index] = input_plane_0[src_index_low] >> bit_shift;
                    output_plane_1[dst_index] = input_plane_1[src_index_low] >> bit_shift;

                    let src_index_high = src_index_low + 1;
                    if (src_index_high < word_len) && (bit_shift > 0) {
                        output_plane_0[dst_index] |=
                            input_plane_0[src_index_high] << (u32::BITS - bit_shift);
                        output_plane_1[dst_index] |=
                            input_plane_1[src_index_high] << (u32::BITS - bit_shift);
                    }

                    output_plane_0[dst_index] |= output_plane_1[dst_index]; // high-z to undefined
                } else {
                    output_plane_0[dst_index] = 0;
                    output_plane_1[dst_index] = 0;
                }
            }
        } else {
            output_plane_0.fill(u32::MAX);
            output_plane_1.fill(u32::MAX);
        }
    } else {
        output.copy_from(input);
    }
}

#[inline]
pub(super) fn shift_right_arithmetic(
    mut output: LogicStateMut,
    input: LogicStateRef,
    shift_amount: LogicStateRef,
) {
    assert_eq!(output.bit_width(), input.bit_width());
    let bit_width = output.bit_width();
    let word_len = bit_width.word_len() as usize;

    if let Some(shift_width) = bit_width.clog2() {
        let (output_plane_0, output_plane_1) = output.bit_planes_mut();
        let (input_plane_0, input_plane_1) = input.bit_planes();
        let (shift_amount_plane_0, shift_amount_plane_1) = shift_amount.bit_planes();

        let shift_mask = shift_width.last_word_mask();
        let (shift_amount, shift_amount_valid) = (
            shift_amount_plane_0[0] & shift_mask,
            shift_amount_plane_1[0] & shift_mask,
        );

        if shift_amount_valid == 0 {
            let word_shift = (shift_amount / u32::BITS) as usize;
            let bit_shift = shift_amount % u32::BITS;

            let mut extend_word = [0; 2];
            for dst_index in 0..word_len {
                let src_index_low = dst_index + word_shift;
                if src_index_low < word_len {
                    let src_index_high = src_index_low + 1;
                    if (src_index_high < word_len) && (bit_shift > 0) {
                        output_plane_0[dst_index] = input_plane_0[src_index_low] >> bit_shift;
                        output_plane_1[dst_index] = input_plane_1[src_index_low] >> bit_shift;

                        output_plane_0[dst_index] |=
                            input_plane_0[src_index_high] << (u32::BITS - bit_shift);
                        output_plane_1[dst_index] |=
                            input_plane_1[src_index_high] << (u32::BITS - bit_shift);
                    } else {
                        let last_word_width = bit_width.last_word_width();
                        let shift_up = u32::BITS - last_word_width.get();
                        let shift_down = shift_up + bit_shift;

                        output_plane_0[dst_index] =
                            ((input_plane_0[src_index_low] as i32) << shift_up >> shift_down)
                                as u32;
                        output_plane_1[dst_index] =
                            ((input_plane_1[src_index_low] as i32) << shift_up >> shift_down)
                                as u32;

                        extend_word[0] =
                            ((output_plane_0[dst_index] as i32) >> (u32::BITS - 1)) as u32;
                        extend_word[1] =
                            ((output_plane_1[dst_index] as i32) >> (u32::BITS - 1)) as u32;
                    }

                    output_plane_0[dst_index] |= output_plane_1[dst_index]; // high-z to undefined
                } else {
                    output_plane_0[dst_index] = extend_word[0];
                    output_plane_1[dst_index] = extend_word[1];
                }
            }
        } else {
            output_plane_0.fill(u32::MAX);
            output_plane_1.fill(u32::MAX);
        }
    } else {
        output.copy_from(input);
    }
}

pub(super) fn horizontal_op<const INIT: u32, const INVERT: bool>(
    input: LogicStateRef,
    op: impl Fn([u32; 2], [u32; 2]) -> [u32; 2],
) -> [u32; 2] {
    let bit_width = input.bit_width();
    let word_len = bit_width.word_len() as usize;

    let (input_plane_0, input_plane_1) = input.bit_planes();

    let mut result = [INIT, u32::MIN];
    for i in 0..word_len {
        let input = if i == (word_len - 1) {
            let mask = bit_width.last_word_mask();
            [
                (input_plane_0[i] & mask) | (INIT & !mask),
                input_plane_1[i] & mask,
            ]
        } else {
            [input_plane_0[i], input_plane_1[i]]
        };

        result = op(result, input);
    }

    result = op(result, [result[0] >> 16, result[1] >> 16]);
    result = op(result, [result[0] >> 8, result[1] >> 8]);
    result = op(result, [result[0] >> 4, result[1] >> 4]);
    result = op(result, [result[0] >> 2, result[1] >> 2]);
    result = op(result, [result[0] >> 1, result[1] >> 1]);

    if INVERT {
        result = logic_not(result);
    }

    result[0] &= 0b1;
    result[1] &= 0b1;

    result
}
