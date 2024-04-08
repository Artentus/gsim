use super::*;

const HIGH_Z: *const LogicState = &LogicState::HIGH_Z;
const UNDEFINED: *const LogicState = &LogicState::UNDEFINED;
const LOGIC_0: *const LogicState = &LogicState::LOGIC_0;
const LOGIC_1: *const LogicState = &LogicState::LOGIC_1;

/// Creates a `LogicState` with all bits set to the high impedance state.  
/// The returned `LogicState` must be freed by calling `logic_state_free`.
#[no_mangle]
#[must_use]
pub unsafe extern "C" fn logic_state_high_z() -> *const LogicState {
    HIGH_Z
}

/// Creates a `LogicState` with all bits set to an undefined state.  
/// The returned `LogicState` must be freed by calling `logic_state_free`.
#[no_mangle]
#[must_use]
pub unsafe extern "C" fn logic_state_undefined() -> *const LogicState {
    UNDEFINED
}

/// Creates a `LogicState` with all bits set to the logic low state.  
/// The returned `LogicState` must be freed by calling `logic_state_free`.
#[no_mangle]
#[must_use]
pub unsafe extern "C" fn logic_state_logic_0() -> *const LogicState {
    LOGIC_0
}

/// Creates a `LogicState` with all bits set to the logic high state.  
/// The returned `LogicState` must be freed by calling `logic_state_free`.
#[no_mangle]
#[must_use]
pub unsafe extern "C" fn logic_state_logic_1() -> *const LogicState {
    LOGIC_1
}

/// Creates a `LogicState` representing the given integer. High bits are set to 0.  
// The returned `LogicState` must be freed by calling `logic_state_free`.
#[no_mangle]
#[must_use]
pub unsafe extern "C" fn logic_state_from_int(value: u32) -> *const LogicState {
    let state_box = Box::new(LogicState::from_int(value));
    Box::into_raw(state_box).cast_const()
}

ffi_fn! {
    /// Parses a `LogicState` from a string representation. High bits are set to high impedance.  
    /// Will fail if the string is longer than 255 characters, shorter than one character, or contains characters other than `'z'`, `'Z'`, `'x'`, `'X'`, `'0'` and `'1'`.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.  
    /// The resulting `LogicState` must be freed by calling `logic_state_free`, only if the operation succeeded.
    logic_state_parse(s: *const c_char, state: *mut *const LogicState) {
        let s = cast_c_str(s)?;
        let state_outer = check_ptr(state)?;

        let state_box = Box::new(LogicState::parse(s).map_err(|_| FfiError::InvalidArgument)?);
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Clones a `LogicState` into a new allocation.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.  
    /// The cloned `LogicState` must be freed separately by calling `logic_state_free`, only if the operation succeeded.
    logic_state_clone(state: *const LogicState, clone: *mut *const LogicState) {
        let clone_outer = check_ptr(clone)?;

        if std::ptr::eq(state, HIGH_Z)
            || std::ptr::eq(state, UNDEFINED)
            || std::ptr::eq(state, LOGIC_0)
            || std::ptr::eq(state, LOGIC_1)
        {
            clone_outer.as_ptr().write(state);
            return Ok(ffi_status::SUCCESS);
        }

        let state = cast_ptr(state)?;
        let clone_inner = Box::into_raw(Box::new(state.clone())).cast_const();
        clone_outer.as_ptr().write(clone_inner);

        return Ok(ffi_status::SUCCESS);
    }
}

ffi_fn! {
    /// Attempts to convert the first `width` bits of a `LogicState` to an integer.  
    /// `width`  must be between 1 and 32 inclusive. Will fail if any of the bits are either in the `Z` or `X` state.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    logic_state_to_int(state: *const LogicState, width: u8, value: *mut u32) {
        let state = cast_ptr(state)?;
        let value = check_ptr(value)?;
        let width = width.try_into()?;

        value.as_ptr().write(state.to_int(width)?);
        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Gets the state of a single bit in a `LogicState`.  
    /// On success, returns one of the following values: `GSIM_RESULT_HIGH_Z`, `GSIM_RESULT_UNDEFINED`, `GSIM_RESULT_LOGIC_0`, `GSIM_RESULT_LOGIC_1`
    logic_state_get_bit_state(state: *const LogicState, bit_index: u8) {
        let state = cast_ptr(state)?;

        let bit_state = state.get_bit_state(bit_index);
        match bit_state {
            LogicBitState::HighZ => Ok(ffi_status::HIGH_Z),
            LogicBitState::Undefined => Ok(ffi_status::UNDEFINED),
            LogicBitState::Logic0 => Ok(ffi_status::LOGIC_0),
            LogicBitState::Logic1 => Ok(ffi_status::LOGIC_1),
        }
    }
}

ffi_fn! {
    /// Prints the string representation of a `LogicState` into a buffer.  
    /// The buffer must be big enough to hold at least `width` bytes and will not be null terminated by this function.  
    /// Since the buffer is owned by the caller, it must not be freed by `string_free`.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    logic_state_print(state: *const LogicState, width: u8, buffer: *mut c_char) {
        let state = cast_ptr(state)?;
        let buffer = check_ptr(buffer)?;
        let buffer = std::slice::from_raw_parts_mut(buffer.as_ptr().cast(), width as usize);
        let width = width.try_into()?;

        let s = state.display_string(width);
        buffer.copy_from_slice(s.as_bytes());

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
    /// Checks the first `width` bits of two `LogicState` objects for equality.  
    /// On success, returns one of the following values: `GSIM_RESULT_FALSE`, `GSIM_RESULT_TRUE`
    logic_state_eq(a: *const LogicState, b: *const LogicState, width: u8) {
        let a = cast_ptr(a)?;
        let b = cast_ptr(b)?;
        let width = width.try_into()?;

        match a.eq(b, width) {
            false => Ok(ffi_status::FALSE),
            true => Ok(ffi_status::TRUE),
        }
    }
}

ffi_fn! {
    /// Frees a `LogicState` that was returned by other functions in the API.  
    /// Returns `GSIM_RESULT_SUCCESS` on success.
    logic_state_free(state: *mut LogicState) {
        if std::ptr::eq(state.cast_const(), HIGH_Z)
            || std::ptr::eq(state.cast_const(), UNDEFINED)
            || std::ptr::eq(state.cast_const(), LOGIC_0)
            || std::ptr::eq(state.cast_const(), LOGIC_1)
        {
            return Ok(ffi_status::SUCCESS);
        }

        let state = check_ptr(state)?;
        let state_box = Box::from_raw(state.as_ptr());
        std::mem::drop(state_box);

        Ok(ffi_status::SUCCESS)
    }
}
