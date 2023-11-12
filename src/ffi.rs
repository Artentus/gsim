#![allow(unsafe_op_in_unsafe_fn)]

use crate::*;
use std::ffi::{c_char, CStr, CString};
use std::ptr::NonNull;

mod ffi_status {
    pub const SUCCESS: i32 = 0;

    pub const MAX_STEPS_REACHED: i32 = 1;

    pub const FALSE: i32 = 0;
    pub const TRUE: i32 = 1;
}

#[allow(dead_code)]
#[rustfmt::skip]
#[repr(i32)]
enum FfiError {
    // Generic errors
    NullPointer        = -0x0000_0001,
    PointerMisaligned  = -0x0000_0002,
    InvalidArgument    = -0x0000_0003,
    ArgumentOutOfRange = -0x0000_0004,
    Utf8Encoding       = -0x0000_0005,
    Io                 = -0x0000_0006,

    // Builder errors
    ResourceLimitReached  = -0x0001_0001,
    WireWidthMismatch     = -0x0001_0002,
    WireWidthIncompatible = -0x0001_0003,
    OffsetOutOfRange      = -0x0001_0004,
    TooFewInputs          = -0x0001_0005,
    InvalidInputCount     = -0x0001_0006,
    InvalidComponentType  = -0x0001_0007,

    // Simulator errors
    Conflict = -0x0002_0001,

    // Import errors
    MalformedFormat = 0x0003_0001,
    Unsupported     = 0x0003_0002,
}

impl From<std::str::Utf8Error> for FfiError {
    #[inline]
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8Encoding
    }
}

impl From<std::io::Error> for FfiError {
    #[inline]
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}

impl From<std::num::TryFromIntError> for FfiError {
    #[inline]
    fn from(_: std::num::TryFromIntError) -> Self {
        Self::ArgumentOutOfRange
    }
}

#[cfg(feature = "yosys-import")]
impl From<serde_json::Error> for FfiError {
    fn from(value: serde_json::Error) -> Self {
        use serde_json::error::Category;

        match value.classify() {
            Category::Io => Self::Io,
            Category::Syntax | Category::Data | Category::Eof => Self::MalformedFormat,
        }
    }
}

#[cfg(feature = "yosys-import")]
impl From<crate::import::yosys::YosysModuleImportError> for FfiError {
    fn from(value: crate::import::yosys::YosysModuleImportError) -> Self {
        use crate::import::yosys::YosysModuleImportError;

        match value {
            YosysModuleImportError::ResourceLimitReached => Self::ResourceLimitReached,
            YosysModuleImportError::InOutPort { .. }
            | YosysModuleImportError::CellInOutPort { .. }
            | YosysModuleImportError::UnsupportedWireWidth { .. }
            | YosysModuleImportError::UnknownCellType { .. }
            | YosysModuleImportError::UnsupportedCellType { .. } => Self::Unsupported,
            YosysModuleImportError::MissingCellPortDirection { .. }
            | YosysModuleImportError::InvalidCellPorts { .. }
            | YosysModuleImportError::InvalidCellParameters { .. } => Self::MalformedFormat,
        }
    }
}

impl From<AddComponentError> for FfiError {
    fn from(value: AddComponentError) -> Self {
        match value {
            AddComponentError::TooManyComponents => Self::ResourceLimitReached,
            AddComponentError::WireWidthMismatch => Self::WireWidthMismatch,
            AddComponentError::WireWidthIncompatible => Self::WireWidthIncompatible,
            AddComponentError::OffsetOutOfRange => Self::OffsetOutOfRange,
            AddComponentError::TooFewInputs => Self::TooFewInputs,
            AddComponentError::InvalidInputCount => Self::InvalidInputCount,
        }
    }
}

#[repr(transparent)]
struct FfiResult(i32);

impl From<Result<i32, FfiError>> for FfiResult {
    #[inline]
    fn from(value: Result<i32, FfiError>) -> Self {
        match value {
            Ok(stat) => FfiResult(stat),
            Err(err) => FfiResult(err as i32),
        }
    }
}

#[inline]
fn check_ptr<T>(ptr: *mut T) -> Result<NonNull<T>, FfiError> {
    let ptr = NonNull::new(ptr).ok_or(FfiError::NullPointer)?;
    if (ptr.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
        return Err(FfiError::PointerMisaligned);
    }
    Ok(ptr)
}

#[inline]
unsafe fn cast_ptr<'a, T>(ptr: *const T) -> Result<&'a T, FfiError> {
    Ok(&*check_ptr(ptr.cast_mut())?.as_ptr())
}

#[inline]
unsafe fn cast_mut_ptr<'a, T>(ptr: *mut T) -> Result<&'a mut T, FfiError> {
    Ok(&mut *check_ptr(ptr)?.as_ptr())
}

unsafe fn cast_c_str<'a>(ptr: *const c_char) -> Result<&'a str, FfiError> {
    let ptr = NonNull::new(ptr.cast_mut()).ok_or(FfiError::NullPointer)?;
    let c_str = CStr::from_ptr(ptr.as_ptr().cast_const());
    let r_str = c_str.to_str()?;
    Ok(r_str)
}

macro_rules! ffi_fn {
    (
        $name:ident(
            $($param_name:ident : $param_ty:ty),* $(,)?
        ) $body:block
    ) => {
        #[no_mangle]
        unsafe extern "C" fn $name($($param_name : $param_ty),*) -> FfiResult {
            #[inline]
            unsafe fn fn_impl($($param_name : $param_ty),*) -> Result<i32, FfiError> $body

            fn_impl($($param_name),*).into()
        }
    };
}

mod builder;
mod logic;
mod simulator;
