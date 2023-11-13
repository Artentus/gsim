use super::*;

const HIGH_Z: *const LogicState = &LogicState::HIGH_Z;
const UNDEFINED: *const LogicState = &LogicState::UNDEFINED;
const LOGIC_0: *const LogicState = &LogicState::LOGIC_0;
const LOGIC_1: *const LogicState = &LogicState::LOGIC_1;

#[no_mangle]
unsafe extern "C" fn logic_state_high_z() -> *const LogicState {
    HIGH_Z
}

#[no_mangle]
unsafe extern "C" fn logic_state_undefined() -> *const LogicState {
    UNDEFINED
}

#[no_mangle]
unsafe extern "C" fn logic_state_logic_0() -> *const LogicState {
    LOGIC_0
}

#[no_mangle]
unsafe extern "C" fn logic_state_logic_1() -> *const LogicState {
    LOGIC_1
}

#[no_mangle]
unsafe extern "C" fn logic_state_from_int(value: u32) -> *const LogicState {
    let state_box = Box::new(LogicState::from_int(value));
    Box::into_raw(state_box).cast_const()
}

ffi_fn! {
    logic_state_parse(s: *const c_char, state: *mut *const LogicState) {
        let s = cast_c_str(s)?;
        let state_outer = check_ptr(state)?;

        let state_box = Box::new(LogicState::parse(s).ok_or(FfiError::InvalidArgument)?);
        let state_inner = Box::into_raw(state_box).cast_const();
        state_outer.as_ptr().write(state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
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
    logic_state_get_bit_state(state: *const LogicState, bit_index: u8, bit_state: *mut LogicBitState) {
        let state = cast_ptr(state)?;
        let bit_state_outer = check_ptr(bit_state)?;

        let bit_state_inner = state.get_bit_state(bit_index);
        bit_state_outer.as_ptr().write(bit_state_inner);

        Ok(ffi_status::SUCCESS)
    }
}

ffi_fn! {
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
