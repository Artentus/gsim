use super::{LogicOffset, LogicSizeInteger, LogicState, LogicStorage, LogicWidth, MAX_LOGIC_WIDTH};

#[inline]
pub(super) fn logic_and(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: (a.state & b.state)
            | (!a.valid & !b.valid)
            | (a.state & !b.valid)
            | (b.state & !a.valid),
        valid: (a.valid & b.valid) | (!a.state & a.valid) | (!b.state & b.valid),
    }
}

#[inline]
pub(super) fn logic_or(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: a.state | !a.valid | b.state | !b.valid,
        valid: (a.state & a.valid) | (b.state & b.valid) | (a.valid & b.valid),
    }
}

#[inline]
pub(super) fn logic_xor(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: (a.state ^ b.state) | !a.valid | !b.valid,
        valid: a.valid & b.valid,
    }
}

#[inline]
pub(super) fn logic_nand(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: !a.state | !a.valid | !b.state | !b.valid,
        valid: (a.valid & b.valid) | (!a.state & a.valid) | (!b.state & b.valid),
    }
}

#[inline]
pub(super) fn logic_nor(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: (!a.state & !b.state)
            | (!a.valid & !b.valid)
            | (!a.state & !b.valid)
            | (!b.state & !a.valid),
        valid: (a.state & a.valid) | (b.state & b.valid) | (a.valid & b.valid),
    }
}

#[inline]
pub(super) fn logic_xnor(a: LogicState, b: LogicState) -> LogicState {
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

    LogicState {
        state: !(a.state ^ b.state) | !a.valid | !b.valid,
        valid: a.valid & b.valid,
    }
}

#[inline]
pub(super) fn logic_not(v: LogicState) -> LogicState {
    //  I state | I valid | I meaning | O state | O valid | O meaning
    // ---------|---------|-----------|---------|---------|-----------
    //     0    |    0    | High-Z    |    1    |    0    | Undefined
    //     1    |    0    | Undefined |    1    |    0    | Undefined
    //     0    |    1    | Logic 0   |    1    |    1    | Logic 1
    //     1    |    1    | Logic 1   |    0    |    1    | Logic 0

    LogicState {
        state: !v.state | !v.valid,
        valid: v.valid,
    }
}

#[inline]
pub(super) fn add(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        LogicState {
            state: a_state + b_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn sub(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        LogicState {
            state: a_state - b_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn mul(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        LogicState {
            state: a_state * b_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn div(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE)
        && (b_valid == LogicStorage::ALL_ONE)
        && (b_state != LogicStorage::ALL_ZERO)
    {
        LogicState {
            state: a_state / b_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn rem(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE)
        && (b_valid == LogicStorage::ALL_ONE)
        && (b_state != LogicStorage::ALL_ZERO)
    {
        LogicState {
            state: a_state % b_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn shl(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
            LogicStorage::ALL_ZERO
        } else {
            let shift_amount = unsafe {
                // SAFETY: we just checked that `b_state` is within the required range.
                LogicOffset::new_unchecked(b_state.0 as u8)
            };

            a_state << shift_amount
        };

        LogicState {
            state: result_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn lshr(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
            LogicStorage::ALL_ZERO
        } else {
            let shift_amount = unsafe {
                // SAFETY: we just checked that `b_state` is within the required range.
                LogicOffset::new_unchecked(b_state.0 as u8)
            };

            a_state >> shift_amount
        };

        LogicState {
            state: result_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

#[inline]
pub(super) fn ashr(a: LogicState, b: LogicState, width: LogicWidth) -> LogicState {
    let mask = LogicStorage::mask(width);
    let a_state = a.state & mask;
    let b_state = b.state & mask;
    let a_valid = a.valid | !mask;
    let b_valid = b.valid | !mask;

    if (a_valid == LogicStorage::ALL_ONE) && (b_valid == LogicStorage::ALL_ONE) {
        let result_state = if b_state.0 >= (width.get() as LogicSizeInteger) {
            LogicStorage::ALL_ZERO
        } else {
            let shift_amount = unsafe {
                // SAFETY: we just checked that `b_state` is within the required range.
                LogicOffset::new_unchecked(b_state.0 as u8)
            };

            let logical_shift = a_state >> shift_amount;

            let post_shift_width = width.get() - shift_amount.get();
            let arithmetic_shift_amount = unsafe {
                // SAFETY:
                //   shift_amount < width <= MAX_LOGIC_WIDTH
                //   => 0 < post_shift_width <= MAX_LOGIC_WIDTH
                //      => 0 <= arithmetic_shift_amount < MAX_LOGIC_WIDTH
                LogicOffset::new_unchecked(MAX_LOGIC_WIDTH - post_shift_width)
            };

            (logical_shift << arithmetic_shift_amount).ashr(arithmetic_shift_amount)
        };

        LogicState {
            state: result_state,
            valid: LogicStorage::ALL_ONE,
        }
    } else {
        LogicState::UNDEFINED
    }
}

// TODO:
//   Maybe the horizontal operations can be implemented more efficiently using popcount.
//   This needs further investigation as it is unclear how to handle the Z and X states.

#[inline]
pub(super) fn horizontal_logic_and(mut a: LogicState, mut width: LogicWidth) -> LogicState {
    while width > 1 {
        if (width.get() & 0x1) == 0 {
            // Even width

            width = unsafe {
                // SAFETY: `width > 1` so `(width / 2) > 0`
                LogicWidth::new_unchecked(width.get() / 2)
            };

            let shift_amount = unsafe {
                // SAFETY: we divided width by 2 which is guaranteed to yield a valid `LogicOffset`.
                LogicOffset::new_unchecked(width.get())
            };

            let b = LogicState {
                state: a.state >> shift_amount,
                valid: a.valid >> shift_amount,
            };

            a = logic_and(a, b);
        } else {
            // Odd width

            width = unsafe {
                // SAFETY: `width > 1` so `(width - 1) > 0`
                LogicWidth::new_unchecked(width.get() - 1)
            };

            const SHIFT_AMOUNT: LogicOffset = unsafe { LogicOffset::new_unchecked(1) };
            let b = LogicState {
                state: a.state >> SHIFT_AMOUNT,
                valid: a.valid >> SHIFT_AMOUNT,
            };

            a = logic_and(a, b);
        }
    }

    a
}

#[inline]
pub(super) fn horizontal_logic_or(mut a: LogicState, mut width: LogicWidth) -> LogicState {
    while width > 1 {
        if (width.get() & 0x1) == 0 {
            // Even width

            width = unsafe {
                // SAFETY: `width > 1` so `(width / 2) > 0`
                LogicWidth::new_unchecked(width.get() / 2)
            };

            let shift_amount = unsafe {
                // SAFETY: we divided width by 2 which is guaranteed to yield a valid `LogicOffset`.
                LogicOffset::new_unchecked(width.get())
            };

            let b = LogicState {
                state: a.state >> shift_amount,
                valid: a.valid >> shift_amount,
            };

            a = logic_or(a, b);
        } else {
            // Odd width

            width = unsafe {
                // SAFETY: `width > 1` so `(width - 1) > 0`
                LogicWidth::new_unchecked(width.get() - 1)
            };

            const SHIFT_AMOUNT: LogicOffset = unsafe { LogicOffset::new_unchecked(1) };
            let b = LogicState {
                state: a.state >> SHIFT_AMOUNT,
                valid: a.valid >> SHIFT_AMOUNT,
            };

            a = logic_or(a, b);
        }
    }

    a
}

#[inline]
pub(super) fn horizontal_logic_nand(a: LogicState, width: LogicWidth) -> LogicState {
    logic_not(horizontal_logic_and(a, width))
}

#[inline]
pub(super) fn horizontal_logic_nor(a: LogicState, width: LogicWidth) -> LogicState {
    logic_not(horizontal_logic_or(a, width))
}
