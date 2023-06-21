use super::{LogicState, LogicStorage, LogicWidth};

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
