use super::components::*;
use super::*;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Bool32(u32);
impl const From<bool> for Bool32 {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Self(1)
        } else {
            Self(0)
        }
    }
}
impl const Into<bool> for Bool32 {
    #[inline]
    fn into(self) -> bool {
        self.0 > 0
    }
}
impl const PartialEq for Bool32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.0 > 0) == (other.0 > 0)
    }
}
impl const PartialEq<bool> for Bool32 {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        (self.0 > 0) == *other
    }
}
impl Eq for Bool32 {}
impl std::hash::Hash for Bool32 {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 > 0).hash(state);
    }
}
impl std::fmt::Debug for Bool32 {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&(self.0 > 0), f)
    }
}
impl std::fmt::Display for Bool32 {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&(self.0 > 0), f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Result {
    Success = 0,
    DriverAlreadyPresent = 1,
    DriverDoesNotExist = 2,

    NullPointerError = -1,
    InvalidComponentIdError = -2,
    InvalidWireIdError = -3,
    InvalidComponentConfigurationError = -4,
    InvalidOutputIndexError = -5,
    ConflictError = -6,
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_create(out_simulator: *mut *mut Simulator) -> Result {
    if out_simulator.is_null() {
        return Result::NullPointerError;
    }

    let simulator = Box::new(Simulator::new());
    let ptr = Box::into_raw(simulator);
    (*out_simulator) = ptr;
    Result::Success
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_destroy(simulator: *mut Simulator) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    let simulator = Box::from_raw(simulator);
    std::mem::drop(simulator);
    Result::Success
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentKind(u32);
impl ComponentKind {
    const CONSTANT: Self = Self(0);
    const UNARY: Self = Self(1);
    const BINARY: Self = Self(2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentSubKind(u32);
impl ComponentSubKind {
    const CONSTANT_PULL_DOWN: Self = Self(0);
    const CONSTANT_PULL_UP: Self = Self(1);

    const UNARY_NOT: Self = Self(0);

    const BINARY_AND: Self = Self(0);
    const BINARY_NAND: Self = Self(1);
    const BINARY_OR: Self = Self(2);
    const BINARY_NOR: Self = Self(3);
    const BINARY_XOR: Self = Self(4);
    const BINARY_XNOR: Self = Self(5);
}

#[repr(C)]
pub struct ComponentCreateInfo {
    kind: ComponentKind,
    sub_kind: ComponentSubKind,
    width: u32,
    input_count: u32,
}

macro_rules! unary_behaviour {
    ($op:ty, $create_info:expr) => {
        Box::new(UnaryBehaviour::<$op>::new($create_info.width))
    };
}

macro_rules! binary_behaviour {
    ($op:ty, $create_info:expr) => {
        Box::new(BinaryBehaviour::<$op>::new(
            $create_info.width,
            $create_info.input_count,
        ))
    };
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_add_component(
    simulator: *mut Simulator,
    create_info: ComponentCreateInfo,
    out_id: *mut ComponentId,
) -> Result {
    if simulator.is_null() || out_id.is_null() {
        return Result::NullPointerError;
    }

    let behaviour: Box<dyn ComponentBehaviour> = match create_info.kind {
        ComponentKind::CONSTANT => match create_info.sub_kind {
            ComponentSubKind::CONSTANT_PULL_DOWN => {
                Box::new(ConstantBehaviour::new_pull_down(create_info.width))
            }
            ComponentSubKind::CONSTANT_PULL_UP => {
                Box::new(ConstantBehaviour::new_pull_up(create_info.width))
            }
            _ => return Result::InvalidComponentConfigurationError,
        },
        ComponentKind::UNARY => match create_info.sub_kind {
            ComponentSubKind::UNARY_NOT => unary_behaviour!(Not, create_info),
            _ => return Result::InvalidComponentConfigurationError,
        },
        ComponentKind::BINARY => match create_info.sub_kind {
            ComponentSubKind::BINARY_AND => binary_behaviour!(And, create_info),
            ComponentSubKind::BINARY_NAND => binary_behaviour!(Nand, create_info),
            ComponentSubKind::BINARY_OR => binary_behaviour!(Or, create_info),
            ComponentSubKind::BINARY_NOR => binary_behaviour!(Nor, create_info),
            ComponentSubKind::BINARY_XOR => binary_behaviour!(Xor, create_info),
            ComponentSubKind::BINARY_XNOR => binary_behaviour!(Xnor, create_info),
            _ => return Result::InvalidComponentConfigurationError,
        },
        _ => return Result::InvalidComponentConfigurationError,
    };

    let component = Component::new(behaviour);
    let id = (*simulator).add_component(component);
    (*out_id) = id;
    Result::Success
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_remove_component(
    simulator: *mut Simulator,
    id: ComponentId,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    if (*simulator).remove_component(id).is_some() {
        Result::Success
    } else {
        Result::InvalidComponentIdError
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_add_wire(
    simulator: *mut Simulator,
    out_id: *mut WireId,
) -> Result {
    if simulator.is_null() || out_id.is_null() {
        return Result::NullPointerError;
    }

    let wire = Wire::new();
    let id = (*simulator).add_wire(wire);
    (*out_id) = id;
    Result::Success
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_remove_wire(
    simulator: *mut Simulator,
    id: WireId,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    if (*simulator).remove_wire(id).is_some() {
        Result::Success
    } else {
        Result::InvalidWireIdError
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn simulator_step(
    simulator: *mut Simulator,
    out_changed: *mut Bool32,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    match (*simulator).step() {
        Ok(changed) => {
            (*out_changed) = changed.into();
            Result::Success
        }
        Err(err) => match err {
            SimulationError::Conflict => Result::ConflictError,
            SimulationError::InvalidComponentId => Result::InvalidComponentIdError,
            SimulationError::InvalidWireId => Result::InvalidWireIdError,
            SimulationError::InvalidOutputIndex => Result::InvalidOutputIndexError,
        },
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn component_connect_input(
    simulator: *mut Simulator,
    component_id: ComponentId,
    input_index: u32,
    wire_count: u32,
    wires: *const WireId,
) -> Result {
    if simulator.is_null() || wires.is_null() {
        return Result::NullPointerError;
    }

    if let Some(component) = (*simulator).get_component_mut(component_id) {
        let wires = std::slice::from_raw_parts(wires, wire_count as usize);
        component.connect_input(input_index, wires);
        Result::Success
    } else {
        Result::InvalidComponentIdError
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn component_disconnect_input(
    simulator: *mut Simulator,
    component_id: ComponentId,
    input_index: u32,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    if let Some(component) = (*simulator).get_component_mut(component_id) {
        component.disconnect_input(input_index);
        Result::Success
    } else {
        Result::InvalidComponentIdError
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn wire_add_driver(
    simulator: *mut Simulator,
    wire_id: WireId,
    component: ComponentId,
    output_index: u32,
    output_sub_index: u32,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    if let Some(wire) = (*simulator).get_wire_mut(wire_id) {
        if wire.add_driver(component, (output_index, output_sub_index)) {
            Result::DriverAlreadyPresent
        } else {
            Result::Success
        }
    } else {
        Result::InvalidWireIdError
    }
}

#[no_mangle]
pub unsafe extern "cdecl" fn wire_remove_driver(
    simulator: *mut Simulator,
    wire_id: WireId,
    component: ComponentId,
    output_index: u32,
    output_sub_index: u32,
) -> Result {
    if simulator.is_null() {
        return Result::NullPointerError;
    }

    if let Some(wire) = (*simulator).get_wire_mut(wire_id) {
        if wire.remove_driver(component, (output_index, output_sub_index)) {
            Result::DriverDoesNotExist
        } else {
            Result::Success
        }
    } else {
        Result::InvalidWireIdError
    }
}
