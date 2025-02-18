#![allow(missing_debug_implementations)]

mod ops;
use ops::*;

use crate::logic::{OutputStateAllocator, OutputStateViewMut};
use crate::*;
use itertools::izip;
use smallvec::smallvec;
#[cfg(feature = "dot-export")]
use std::borrow::Cow;
use sync_unsafe_cell::SyncUnsafeCell;

def_id_type!(
    /// A unique identifier for a component inside a simulation
    pub ComponentId
);

impl ComponentId {
    #[inline]
    const fn kind(self) -> u8 {
        (self.0 >> 24) as u8
    }

    #[inline]
    const fn index(self) -> usize {
        (self.0 & 0xFFFFFF) as usize
    }
}

pub(crate) trait ComponentArgs: Copy {
    fn connect_drivers(
        self,
        component: ComponentId,
        wires: &mut WireList,
    ) -> Result<(), AddComponentError>;
}

pub(crate) trait Component: Sized {
    type Args<'a>: ComponentArgs;

    fn new(
        args: Self::Args<'_>,
        wires: &mut WireList,
        output_states: &mut OutputStateAllocator,
    ) -> Result<Self, AddComponentError>;

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str>;

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]>;

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]>;

    fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth);

    fn update(
        &mut self,
        wire_states: WireStateView,
        output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId);

    #[inline]
    fn reset(&mut self) {}
}

pub(crate) trait ComponentAuto: Component {
    const ID: u8;

    fn extract_storage(storage: &ComponentStorage) -> &[SyncUnsafeCell<Self>];
    fn extract_storage_mut(storage: &mut ComponentStorage) -> &mut Vec<SyncUnsafeCell<Self>>;
}

macro_rules! def_components {
    (@SINGLE $id:expr;
        struct $component_name:ident {
            $($field_name:ident : $field_ty:ty,)*
        }
    ) => {
        pub(crate) struct $component_name {
            $($field_name : $field_ty,)*
        }

        impl ComponentAuto for $component_name {
            const ID: u8 = $id;

            #[inline]
            fn extract_storage(storage: &ComponentStorage) -> &[SyncUnsafeCell<Self>] {
                &storage.$component_name
            }

            #[inline]
            fn extract_storage_mut(storage: &mut ComponentStorage) -> &mut Vec<SyncUnsafeCell<Self>> {
                &mut storage.$component_name
            }
        }
    };

    (@REC $id:expr;
        struct $component_name:ident {
            $($field_name:ident : $field_ty:ty,)*
        }
    ) => {
        def_components! {
            @SINGLE $id;

            struct $component_name {
                $($field_name : $field_ty,)*
            }
        }
    };

    (@REC $id:expr;
        struct $first_component_name:ident {
            $($first_field_name:ident : $first_field_ty:ty,)*
        }

        $(
            struct $component_name:ident {
                $($field_name:ident : $field_ty:ty,)*
            }
        )+
    ) => {
        def_components! {
            @SINGLE $id;

            struct $first_component_name {
                $($first_field_name : $first_field_ty,)*
            }
        }

        def_components! {
            @REC $id + 1;

            $(
                struct $component_name {
                    $($field_name : $field_ty,)*
                }
            )+
        }
    };

    (
        $(
            struct $component_name:ident {
                $($field_name:ident : $field_ty:ty,)*
            }
        )+
    ) => {
        def_components! {
            @REC 0;

            $(
                struct $component_name {
                    $($field_name : $field_ty,)*
                }
            )+
        }

        #[derive(Default)]
        pub(crate) struct ComponentStorage {
            $(
                #[allow(non_snake_case)]
                $component_name: Vec<SyncUnsafeCell<$component_name>>,
            )+
        }

        impl ComponentStorage {
            pub(crate) fn push<T: ComponentAuto>(&mut self, component: T) -> Option<ComponentId> {
                let storage = T::extract_storage_mut(self);

                let index = storage.len();
                if index > 0xFFFFFF {
                    return None;
                }

                storage.push(SyncUnsafeCell::new(component));

                let id = ((T::ID as u32) << 24) | (index as u32);
                Some(ComponentId(id))
            }

            pub(crate) fn ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
                let iter = std::iter::empty();
                $(
                    let iter = iter.chain((0..self.$component_name.len()).map(|index| {
                        let id = ((<$component_name>::ID as u32) << 24) | (index as u32);
                        ComponentId(id)
                    }));
                )+
                iter
            }

            pub(crate) fn component_exists(&self, id: ComponentId) -> bool {
                match id.kind() {
                    $(
                        <$component_name>::ID => {
                            let storage = <$component_name>::extract_storage(self);
                            id.index() < storage.len()
                        }
                    )+
                    _ => panic!("invalid component kind"),
                }
            }

            // TODO: instead of matching on kind, divide the update queue by kind and then loop over each kind.
            /// SAFETY: caller must ensure the component ID is valid and unique.
            pub(crate) unsafe fn update_component(
                &self,
                id: ComponentId,
                wire_states: WireStateView,
                output_states: &OutputStateAllocator,
            ) -> inline_vec!(WireId) {
                match id.kind() {
                    $(
                        <$component_name>::ID => {
                            let storage = <$component_name>::extract_storage(self);
                            let component = unsafe { &mut *storage[id.index()].get() };

                            let (output_start, output_end, output_end_width) = component.output_range();
                            let output_states = unsafe {
                                // SAFETY: since the component is unique, so is its output range
                                output_states.range_unsafe(output_start, output_end, output_end_width)
                            };

                            component.update(wire_states, output_states)
                        }
                    )+
                    _ => panic!("invalid component kind"),
                }
            }

            pub(crate) fn reset_components(&mut self) {
                $(
                    let storage = <$component_name>::extract_storage_mut(self);
                    for component in storage {
                        component.get_mut().reset();
                    }
                )+
            }
        }
    };
}

def_components! {
    struct AndGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct OrGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct XorGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct NandGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct NorGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct XnorGate {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideAndGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideOrGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideXorGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideNandGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideNorGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct WideXnorGate {
        bit_width: BitWidth,
        inputs: IdVec<WireStateId>,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct NotGate {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Buffer {
        bit_width: BitWidth,
        input: WireStateId,
        enable: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Slice {
        bit_width: BitWidth,
        input: WireStateId,
        start_offset: u16,
        end_offset: u16,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Add {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Sub {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Neg {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct Mul {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct LeftShift {
        bit_width: BitWidth,
        input: WireStateId,
        shift_amount: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct LogicalRightShift {
        bit_width: BitWidth,
        input: WireStateId,
        shift_amount: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct ArithmeticRightShift {
        bit_width: BitWidth,
        input: WireStateId,
        shift_amount: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalAnd {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalOr {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalXor {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalNand {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalNor {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct HorizontalXnor {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareEqual {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareNotEqual {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareLessThan {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareGreaterThan {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareLessThanOrEqual {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareGreaterThanOrEqual {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareLessThanSigned {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareGreaterThanSigned {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareLessThanOrEqualSigned {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct CompareGreaterThanOrEqualSigned {
        bit_width: BitWidth,
        input_a: WireStateId,
        input_b: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct ZeroExtend {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }

    struct SignExtend {
        bit_width: BitWidth,
        input: WireStateId,
        output_state: OutputStateId,
        output_wire: WireId,
    }
}

impl ComponentArgs for () {
    fn connect_drivers(
        self,
        component: ComponentId,
        wires: &mut WireList,
    ) -> Result<(), AddComponentError> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UnaryGateArgs {
    pub(crate) input: WireId,
    pub(crate) output: WireId,
}

impl ComponentArgs for UnaryGateArgs {
    fn connect_drivers(
        self,
        component: ComponentId,
        wires: &mut WireList,
    ) -> Result<(), AddComponentError> {
        let wire = wires.get_mut(self.input).ok_or(InvalidWireIdError)?;
        wire.add_driving(component);
        Ok(())
    }
}

macro_rules! unary_gate_impl {
    ($name:literal) => {
        type Args<'a> = UnaryGateArgs;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_wire = wires
                .get(args.input)
                .ok_or(AddComponentError::InvalidWireId)?;

            if input_wire.bit_width() != output_wire.bit_width() {
                return Err(AddComponentError::WireWidthMismatch);
            }

            let input = input_wire.state_id();

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width: output_wire.bit_width(),
                input,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            smallvec![(self.input, format!("In").into())]
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! horizontal_gate_impl {
    ($name:literal) => {
        type Args<'a> = UnaryGateArgs;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_wire = wires
                .get(args.input)
                .ok_or(AddComponentError::InvalidWireId)?;

            if output_wire.bit_width() != BitWidth::MIN {
                return Err(AddComponentError::WireWidthIncompatible);
            }

            let bit_width = input_wire.bit_width();
            let input = input_wire.state_id();

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width,
                input,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            smallvec![(self.input, format!("In").into())]
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! horizontal_gate_update_impl {
    ($op:expr, $init:expr, $invert:literal) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let [input, _] = wire_states
                .get(self.input, self.bit_width)
                .expect("invalid wire state ID");
            let new_state = horizontal_op::<{ $init }, $invert>(input, $op);

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from_one(new_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

#[derive(Clone, Copy)]
pub(crate) struct BinaryGateArgs {
    pub(crate) input_a: WireId,
    pub(crate) input_b: WireId,
    pub(crate) output: WireId,
}

impl ComponentArgs for BinaryGateArgs {
    fn connect_drivers(
        self,
        component: ComponentId,
        wires: &mut WireList,
    ) -> Result<(), AddComponentError> {
        let wire_a = wires.get_mut(self.input_a).ok_or(InvalidWireIdError)?;
        wire_a.add_driving(component);
        let wire_b = wires.get_mut(self.input_b).ok_or(InvalidWireIdError)?;
        wire_b.add_driving(component);
        Ok(())
    }
}

macro_rules! binary_gate_impl {
    ($name:literal) => {
        type Args<'a> = BinaryGateArgs;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_a_wire = wires
                .get(args.input_a)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_b_wire = wires
                .get(args.input_b)
                .ok_or(AddComponentError::InvalidWireId)?;

            if input_a_wire.bit_width() != output_wire.bit_width() {
                return Err(AddComponentError::WireWidthMismatch);
            }
            if input_b_wire.bit_width() != output_wire.bit_width() {
                return Err(AddComponentError::WireWidthMismatch);
            }

            let input_a = input_a_wire.state_id();
            let input_b = input_b_wire.state_id();

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width: output_wire.bit_width(),
                input_a,
                input_b,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            smallvec![
                (self.input_a, format!("A").into()),
                (self.input_b, format!("B").into()),
            ]
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! binary_gate_update_impl {
    ($op:expr) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let mut tmp_state = InlineLogicState::undefined(self.bit_width);

            let [input_a, _] = wire_states
                .get(self.input_a, self.bit_width)
                .expect("invalid wire state ID");
            let [input_b, _] = wire_states
                .get(self.input_b, self.bit_width)
                .expect("invalid wire state ID");
            binary_op(tmp_state.borrow_mut(), input_a, input_b, $op);

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from(&tmp_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

macro_rules! carrying_binary_gate_update_impl {
    ($op:expr, $c_in:ident) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let mut tmp_state = InlineLogicState::undefined(self.bit_width);

            let [input_a, _] = wire_states
                .get(self.input_a, self.bit_width)
                .expect("invalid wire state ID");
            let [input_b, _] = wire_states
                .get(self.input_b, self.bit_width)
                .expect("invalid wire state ID");

            carrying_binary_op(
                tmp_state.borrow_mut(),
                input_a,
                input_b,
                LogicBitState::$c_in,
                $op,
            );

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from(&tmp_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

macro_rules! shifter_impl {
    ($name:literal) => {
        type Args<'a> = BinaryGateArgs;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_wire = wires
                .get(args.input_a)
                .ok_or(AddComponentError::InvalidWireId)?;
            let shift_amount_wire = wires
                .get(args.input_b)
                .ok_or(AddComponentError::InvalidWireId)?;

            if input_wire.bit_width() == BitWidth::MIN {
                return Err(AddComponentError::WireWidthIncompatible);
            }
            if input_wire.bit_width() != output_wire.bit_width() {
                return Err(AddComponentError::WireWidthMismatch);
            }
            if shift_amount_wire.bit_width() != input_wire.bit_width().clog2().unwrap() {
                return Err(AddComponentError::WireWidthIncompatible);
            }

            let input = input_wire.state_id();
            let shift_amount = shift_amount_wire.state_id();

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width: output_wire.bit_width(),
                input,
                shift_amount,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            smallvec![
                (self.input, format!("In").into()),
                (self.shift_amount, format!("ShAmnt").into()),
            ]
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! shifter_update_impl {
    ($op:expr) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let mut tmp_state = InlineLogicState::undefined(self.bit_width);

            let [input, _] = wire_states
                .get(self.input, self.bit_width)
                .expect("invalid wire state ID");
            let [shift_amount, _] = wire_states
                .get(self.shift_amount, self.bit_width)
                .expect("invalid wire state ID");
            $op(tmp_state.borrow_mut(), input, shift_amount);

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from(&tmp_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

macro_rules! cmp_impl {
    ($name:literal) => {
        type Args<'a> = BinaryGateArgs;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_a_wire = wires
                .get(args.input_a)
                .ok_or(AddComponentError::InvalidWireId)?;
            let input_b_wire = wires
                .get(args.input_b)
                .ok_or(AddComponentError::InvalidWireId)?;

            if input_a_wire.bit_width() != input_b_wire.bit_width() {
                return Err(AddComponentError::WireWidthMismatch);
            }
            if output_wire.bit_width() != BitWidth::MIN {
                return Err(AddComponentError::WireWidthIncompatible);
            }

            let bit_width = input_a_wire.bit_width();
            let input_a = input_a_wire.state_id();
            let input_b = input_b_wire.state_id();

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width,
                input_a,
                input_b,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            smallvec![
                (self.input_a, format!("A").into()),
                (self.input_b, format!("B").into()),
            ]
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! cmp_update_impl {
    (|$flags:ident| $body:expr) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let [input_a, _] = wire_states
                .get(self.input_a, self.bit_width)
                .expect("invalid wire state ID");
            let [input_b, _] = wire_states
                .get(self.input_b, self.bit_width)
                .expect("invalid wire state ID");
            let $flags = cmp_flags(input_a, input_b);
            let new_state = $body;

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from_bit(new_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

#[derive(Clone, Copy)]
pub(crate) struct WideGateArgs<'a> {
    pub(crate) inputs: &'a [WireId],
    pub(crate) output: WireId,
}

impl ComponentArgs for WideGateArgs<'_> {
    fn connect_drivers(
        self,
        component: ComponentId,
        wires: &mut WireList,
    ) -> Result<(), AddComponentError> {
        for &input in self.inputs {
            let wire = wires.get_mut(input).ok_or(InvalidWireIdError)?;
            wire.add_driving(component);
        }

        Ok(())
    }
}

macro_rules! wide_gate_impl {
    ($name:literal) => {
        type Args<'a> = WideGateArgs<'a>;

        fn new(
            args: Self::Args<'_>,
            wires: &mut WireList,
            output_states: &mut OutputStateAllocator,
        ) -> Result<Self, AddComponentError> {
            let output_wire = wires
                .get(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let mut inputs = IdVec::new();
            for &input in args.inputs {
                let input_wire = wires.get(input).ok_or(AddComponentError::InvalidWireId)?;

                if input_wire.bit_width() != output_wire.bit_width() {
                    return Err(AddComponentError::WireWidthMismatch);
                }

                inputs.push(input_wire.state_id());
            }

            let output_wire = wires
                .get_mut(args.output)
                .ok_or(AddComponentError::InvalidWireId)?;

            let output_state = output_states.alloc(output_wire.bit_width())?;
            output_wire.add_driver(output_state);

            Ok(Self {
                bit_width: output_wire.bit_width(),
                inputs,
                output_state,
                output_wire: args.output,
            })
        }

        #[cfg(feature = "dot-export")]
        fn node_name(&self) -> Cow<'static, str> {
            $name.into()
        }

        #[cfg(feature = "dot-export")]
        fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
            smallvec![(self.output_wire, "Out".into())]
        }

        #[cfg(feature = "dot-export")]
        fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
            self.inputs
                .iter()
                .enumerate()
                .map(|(i, input)| (input, format!("In{i}").into()))
                .collect()
        }

        #[inline]
        fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
            (self.output_state, self.output_state, self.bit_width)
        }
    };
}

macro_rules! wide_gate_update_impl {
    ($op:expr) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let mut tmp_state = InlineLogicState::undefined(self.bit_width);

            let mut input_iter = self.inputs.iter();
            if let Some(first_input) = input_iter.next() {
                let [first_input, _] = wire_states
                    .get(first_input, self.bit_width)
                    .expect("invalid wire state ID");
                tmp_state.copy_from(first_input);
            }

            for input in input_iter {
                let [input, _] = wire_states
                    .get(input, self.bit_width)
                    .expect("invalid wire state ID");
                binary_op_mut(tmp_state.borrow_mut(), input, $op);
            }

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from(&tmp_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

macro_rules! wide_gate_inv_update_impl {
    ($op:expr) => {
        fn update(
            &mut self,
            wire_states: WireStateView,
            mut output_states: OutputStateViewMut,
        ) -> inline_vec!(WireId) {
            let mut tmp_state = InlineLogicState::undefined(self.bit_width);

            let mut input_iter = self.inputs.iter();
            if let Some(first_input) = input_iter.next() {
                let [first_input, _] = wire_states
                    .get(first_input, self.bit_width)
                    .expect("invalid wire state ID");
                tmp_state.copy_from(first_input);
            }

            for input in input_iter {
                let [input, _] = wire_states
                    .get(input, self.bit_width)
                    .expect("invalid wire state ID");
                binary_op_mut(tmp_state.borrow_mut(), input, $op);
            }

            unary_op_mut(tmp_state.borrow_mut(), logic_not);

            let [mut output] = output_states
                .get_mut(self.output_state, self.bit_width)
                .expect("invalid output state ID");

            match output.copy_from(&tmp_state) {
                CopyFromResult::Unchanged => smallvec![],
                CopyFromResult::Changed => smallvec![self.output_wire],
            }
        }
    };
}

impl Component for AndGate {
    binary_gate_impl!("AND");
    binary_gate_update_impl!(logic_and);
}

impl Component for OrGate {
    binary_gate_impl!("OR");
    binary_gate_update_impl!(logic_or);
}

impl Component for XorGate {
    binary_gate_impl!("XOR");
    binary_gate_update_impl!(logic_xor);
}

impl Component for NandGate {
    binary_gate_impl!("NAND");
    binary_gate_update_impl!(logic_nand);
}

impl Component for NorGate {
    binary_gate_impl!("NOR");
    binary_gate_update_impl!(logic_nor);
}

impl Component for XnorGate {
    binary_gate_impl!("XNOR");
    binary_gate_update_impl!(logic_xnor);
}

impl Component for WideAndGate {
    wide_gate_impl!("AND");
    wide_gate_update_impl!(logic_and);
}

impl Component for WideOrGate {
    wide_gate_impl!("OR");
    wide_gate_update_impl!(logic_or);
}

impl Component for WideXorGate {
    wide_gate_impl!("XOR");
    wide_gate_update_impl!(logic_xor);
}

impl Component for WideNandGate {
    wide_gate_impl!("NAND");
    wide_gate_inv_update_impl!(logic_and);
}

impl Component for WideNorGate {
    wide_gate_impl!("NOR");
    wide_gate_inv_update_impl!(logic_or);
}

impl Component for WideXnorGate {
    wide_gate_impl!("XNOR");
    wide_gate_inv_update_impl!(logic_xor);
}

impl Component for NotGate {
    unary_gate_impl!("NOT");

    fn update(
        &mut self,
        wire_states: WireStateView,
        mut output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        let mut tmp_state = InlineLogicState::undefined(self.bit_width);

        let [input, _] = wire_states
            .get(self.input, self.bit_width)
            .expect("invalid wire state ID");
        unary_op(tmp_state.borrow_mut(), input, logic_not);

        let [mut output] = output_states
            .get_mut(self.output_state, self.bit_width)
            .expect("invalid output state ID");

        match output.copy_from(&tmp_state) {
            CopyFromResult::Unchanged => smallvec![],
            CopyFromResult::Changed => smallvec![self.output_wire],
        }
    }
}

impl Component for Buffer {
    type Args<'a> = BinaryGateArgs;

    fn new(
        args: Self::Args<'_>,
        wires: &mut WireList,
        output_states: &mut OutputStateAllocator,
    ) -> Result<Self, AddComponentError> {
        let output_wire = wires
            .get(args.output)
            .ok_or(AddComponentError::InvalidWireId)?;
        let input_wire = wires
            .get(args.input_a)
            .ok_or(AddComponentError::InvalidWireId)?;
        let enable_wire = wires
            .get(args.input_b)
            .ok_or(AddComponentError::InvalidWireId)?;

        if input_wire.bit_width() != output_wire.bit_width() {
            return Err(AddComponentError::WireWidthMismatch);
        }
        if enable_wire.bit_width() != BitWidth::MIN {
            return Err(AddComponentError::WireWidthIncompatible);
        }

        let input = input_wire.state_id();
        let enable = enable_wire.state_id();

        let output_wire = wires
            .get_mut(args.output)
            .ok_or(AddComponentError::InvalidWireId)?;

        let output_state = output_states.alloc(output_wire.bit_width())?;
        output_wire.add_driver(output_state);

        Ok(Self {
            bit_width: output_wire.bit_width(),
            input,
            enable,
            output_state,
            output_wire: args.output,
        })
    }

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "Buffer".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
        smallvec![(self.output_wire, "Out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        smallvec![
            (self.input, format!("In").into()),
            (self.enable, format!("En").into()),
        ]
    }

    #[inline]
    fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
        (self.output_state, self.output_state, self.bit_width)
    }

    fn update(
        &mut self,
        wire_states: WireStateView,
        mut output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        let mut tmp_state = InlineLogicState::logic_0(self.bit_width);

        let [input, _] = wire_states
            .get(self.input, self.bit_width)
            .expect("invalid wire state ID");
        let [enable, _] = wire_states
            .get(self.enable, self.bit_width)
            .expect("invalid wire state ID");

        match enable.bit(0).expect("invalid wire width") {
            LogicBitState::Logic0 | LogicBitState::HighZ => tmp_state.set_high_z(),
            LogicBitState::Logic1 => unary_op(tmp_state.borrow_mut(), input, high_z_to_undefined),
            LogicBitState::Undefined => tmp_state.set_undefined(),
        }

        let [mut output] = output_states
            .get_mut(self.output_state, self.bit_width)
            .expect("invalid output state ID");

        match output.copy_from(&tmp_state) {
            CopyFromResult::Unchanged => smallvec![],
            CopyFromResult::Changed => smallvec![self.output_wire],
        }
    }
}

impl Component for Slice {
    type Args<'a> = ();

    fn new(
        args: Self::Args<'_>,
        wires: &mut WireList,
        output_states: &mut OutputStateAllocator,
    ) -> Result<Self, AddComponentError> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        todo!()
    }

    #[inline]
    fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
        (self.output_state, self.output_state, self.bit_width)
    }

    fn update(
        &mut self,
        wire_states: WireStateView,
        output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        todo!()
    }
}

impl Component for Add {
    binary_gate_impl!("ADD");
    carrying_binary_gate_update_impl!(add, Logic0);
}

impl Component for Sub {
    binary_gate_impl!("SUB");
    carrying_binary_gate_update_impl!(sub, Logic1);
}

impl Component for Neg {
    unary_gate_impl!("NEG");

    fn update(
        &mut self,
        wire_states: WireStateView,
        mut output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        let mut tmp_state = InlineLogicState::undefined(self.bit_width);

        let [input, _] = wire_states
            .get(self.input, self.bit_width)
            .expect("invalid wire state ID");

        let zero = LogicState::logic_0(self.bit_width);

        carrying_binary_op(
            tmp_state.borrow_mut(),
            zero.borrow(),
            input,
            LogicBitState::Logic1,
            sub,
        );

        let [mut output] = output_states
            .get_mut(self.output_state, self.bit_width)
            .expect("invalid output state ID");

        match output.copy_from(&tmp_state) {
            CopyFromResult::Unchanged => smallvec![],
            CopyFromResult::Changed => smallvec![self.output_wire],
        }
    }
}

impl Component for Mul {
    binary_gate_impl!("MUL");

    fn update(
        &mut self,
        wire_states: WireStateView,
        mut output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        let mut tmp_state = InlineLogicState::undefined(self.bit_width);

        let [input_a, _] = wire_states
            .get(self.input_a, self.bit_width)
            .expect("invalid wire state ID");
        let [input_b, _] = wire_states
            .get(self.input_b, self.bit_width)
            .expect("invalid wire state ID");
        mul(tmp_state.borrow_mut(), input_a, input_b);

        let [mut output] = output_states
            .get_mut(self.output_state, self.bit_width)
            .expect("invalid output state ID");

        match output.copy_from(&tmp_state) {
            CopyFromResult::Unchanged => smallvec![],
            CopyFromResult::Changed => smallvec![self.output_wire],
        }
    }
}

impl Component for LeftShift {
    shifter_impl!("SHL");
    shifter_update_impl!(shift_left);
}

impl Component for LogicalRightShift {
    shifter_impl!("LSHR");
    shifter_update_impl!(shift_right_logical);
}

impl Component for ArithmeticRightShift {
    shifter_impl!("ASHR");
    shifter_update_impl!(shift_right_arithmetic);
}

impl Component for HorizontalAnd {
    horizontal_gate_impl!("HAND");
    horizontal_gate_update_impl!(logic_and, u32::MAX, false);
}

impl Component for HorizontalOr {
    horizontal_gate_impl!("HOR");
    horizontal_gate_update_impl!(logic_or, u32::MIN, false);
}

impl Component for HorizontalXor {
    horizontal_gate_impl!("HXOR");
    horizontal_gate_update_impl!(logic_xor, u32::MIN, false);
}

impl Component for HorizontalNand {
    horizontal_gate_impl!("HNAND");
    horizontal_gate_update_impl!(logic_and, u32::MAX, true);
}

impl Component for HorizontalNor {
    horizontal_gate_impl!("HNOR");
    horizontal_gate_update_impl!(logic_or, u32::MIN, true);
}

impl Component for HorizontalXnor {
    horizontal_gate_impl!("HXNOR");
    horizontal_gate_update_impl!(logic_xor, u32::MIN, true);
}

impl Component for CompareEqual {
    cmp_impl!("CMP-EQ");

    cmp_update_impl!(|flags| flags
        .zero
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareNotEqual {
    cmp_impl!("CMP-NEQ");

    cmp_update_impl!(|flags| flags
        .zero
        .map(std::ops::Not::not)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareLessThan {
    cmp_impl!("CMP-LT");

    cmp_update_impl!(|flags| flags
        .carry
        .map(std::ops::Not::not)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareGreaterThan {
    cmp_impl!("CMP-GT");

    cmp_update_impl!(|flags| flags
        .carry
        .zip(flags.zero)
        .map(|(carry, zero)| carry & !zero)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareLessThanOrEqual {
    cmp_impl!("CMP-LTEQ");

    cmp_update_impl!(|flags| flags
        .carry
        .zip(flags.zero)
        .map(|(carry, zero)| !carry | zero)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareGreaterThanOrEqual {
    cmp_impl!("CMP-GTEQ");

    cmp_update_impl!(|flags| flags
        .carry
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareLessThanSigned {
    cmp_impl!("SCMP-LT");

    cmp_update_impl!(|flags| flags
        .sign
        .zip(flags.overflow)
        .map(|(sign, overflow)| sign != overflow)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareGreaterThanSigned {
    cmp_impl!("SCMP-GT");

    cmp_update_impl!(|flags| flags
        .sign
        .zip(flags.overflow)
        .zip(flags.zero)
        .map(|((sign, overflow), zero)| (sign == overflow) & !zero)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareLessThanOrEqualSigned {
    cmp_impl!("SCMP-LTEQ");

    cmp_update_impl!(|flags| flags
        .sign
        .zip(flags.overflow)
        .zip(flags.zero)
        .map(|((sign, overflow), zero)| (sign != overflow) | zero)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for CompareGreaterThanOrEqualSigned {
    cmp_impl!("SCMP-GTEQ");

    cmp_update_impl!(|flags| flags
        .sign
        .zip(flags.overflow)
        .map(|(sign, overflow)| sign == overflow)
        .map(LogicBitState::from_bool)
        .unwrap_or(LogicBitState::Undefined));
}

impl Component for ZeroExtend {
    type Args<'a> = ();

    fn new(
        args: Self::Args<'_>,
        wires: &mut WireList,
        output_states: &mut OutputStateAllocator,
    ) -> Result<Self, AddComponentError> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        todo!()
    }

    #[inline]
    fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
        (self.output_state, self.output_state, self.bit_width)
    }

    fn update(
        &mut self,
        wire_states: WireStateView,
        output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        todo!()
    }
}

impl Component for SignExtend {
    type Args<'a> = ();

    fn new(
        args: Self::Args<'_>,
        wires: &mut WireList,
        output_states: &mut OutputStateAllocator,
    ) -> Result<Self, AddComponentError> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> SmallVec<[(WireId, Cow<'static, str>); 1]> {
        todo!()
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        todo!()
    }

    #[inline]
    fn output_range(&self) -> (OutputStateId, OutputStateId, BitWidth) {
        (self.output_state, self.output_state, self.bit_width)
    }

    fn update(
        &mut self,
        wire_states: WireStateView,
        output_states: OutputStateViewMut,
    ) -> inline_vec!(WireId) {
        todo!()
    }
}

/*
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

pub(crate) struct SmallComponent {
    kind: SmallComponentKind,
}

impl SmallComponent {
    #[inline]
    pub(crate) fn new(kind: SmallComponentKind, output: WireId) -> Self {
        Self { kind, output }
    }

    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        match self.kind {
            SmallComponentKind::AndGate { .. } => "AND".into(),
            SmallComponentKind::OrGate { .. } => "OR".into(),
            SmallComponentKind::XorGate { .. } => "XOR".into(),
            SmallComponentKind::NandGate { .. } => "NAND".into(),
            SmallComponentKind::NorGate { .. } => "NOR".into(),
            SmallComponentKind::XnorGate { .. } => "XNOR".into(),
            SmallComponentKind::NotGate { .. } => "NOT".into(),
            SmallComponentKind::Buffer { .. } => "Buffer".into(),
            SmallComponentKind::Slice {
                start_offset,
                end_offset,
                ..
            } => format!("[{end_offset}:{start_offset}]").into(),
            SmallComponentKind::Add { .. } => "ADD".into(),
            SmallComponentKind::Sub { .. } => "SUB".into(),
            SmallComponentKind::Neg { .. } => "NEG".into(),
            SmallComponentKind::Mul { .. } => "MUL".into(),
            SmallComponentKind::LeftShift { .. } => "<<".into(),
            SmallComponentKind::LogicalRightShift { .. } => ">>".into(),
            SmallComponentKind::ArithmeticRightShift { .. } => ">>>".into(),
            SmallComponentKind::HorizontalAnd { .. } => "AND".into(),
            SmallComponentKind::HorizontalOr { .. } => "OR".into(),
            SmallComponentKind::HorizontalXor { .. } => "XOR".into(),
            SmallComponentKind::HorizontalNand { .. } => "NAND".into(),
            SmallComponentKind::HorizontalNor { .. } => "NOR".into(),
            SmallComponentKind::HorizontalXnor { .. } => "XNOR".into(),
            SmallComponentKind::CompareEqual { .. } => "==".into(),
            SmallComponentKind::CompareNotEqual { .. } => "!=".into(),
            SmallComponentKind::CompareLessThan { .. } => "<".into(),
            SmallComponentKind::CompareGreaterThan { .. } => ">".into(),
            SmallComponentKind::CompareLessThanOrEqual { .. } => "<=".into(),
            SmallComponentKind::CompareGreaterThanOrEqual { .. } => ">=".into(),
            SmallComponentKind::CompareLessThanSigned { .. } => "<".into(),
            SmallComponentKind::CompareGreaterThanSigned { .. } => ">".into(),
            SmallComponentKind::CompareLessThanOrEqualSigned { .. } => "<=".into(),
            SmallComponentKind::CompareGreaterThanOrEqualSigned { .. } => ">=".into(),
            SmallComponentKind::ZeroExtend { .. } => "ZEXT".into(),
            SmallComponentKind::SignExtend { .. } => "SEXT".into(),
        }
    }

    #[cfg(feature = "dot-export")]
    pub(crate) fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        match self.kind {
            SmallComponentKind::AndGate { input_a, input_b }
            | SmallComponentKind::OrGate { input_a, input_b }
            | SmallComponentKind::XorGate { input_a, input_b }
            | SmallComponentKind::NandGate { input_a, input_b }
            | SmallComponentKind::NorGate { input_a, input_b }
            | SmallComponentKind::XnorGate { input_a, input_b }
            | SmallComponentKind::Add { input_a, input_b }
            | SmallComponentKind::Sub { input_a, input_b }
            | SmallComponentKind::Mul { input_a, input_b }
            | SmallComponentKind::CompareEqual { input_a, input_b }
            | SmallComponentKind::CompareNotEqual { input_a, input_b }
            | SmallComponentKind::CompareLessThan { input_a, input_b }
            | SmallComponentKind::CompareGreaterThan { input_a, input_b }
            | SmallComponentKind::CompareLessThanOrEqual { input_a, input_b }
            | SmallComponentKind::CompareGreaterThanOrEqual { input_a, input_b }
            | SmallComponentKind::CompareLessThanSigned { input_a, input_b }
            | SmallComponentKind::CompareGreaterThanSigned { input_a, input_b }
            | SmallComponentKind::CompareLessThanOrEqualSigned { input_a, input_b }
            | SmallComponentKind::CompareGreaterThanOrEqualSigned { input_a, input_b } => {
                smallvec![(input_a, "A".into()), (input_b, "B".into())]
            }
            SmallComponentKind::NotGate { input }
            | SmallComponentKind::Neg { input }
            | SmallComponentKind::HorizontalAnd { input }
            | SmallComponentKind::HorizontalOr { input }
            | SmallComponentKind::HorizontalXor { input }
            | SmallComponentKind::HorizontalNand { input }
            | SmallComponentKind::HorizontalNor { input }
            | SmallComponentKind::HorizontalXnor { input }
            | SmallComponentKind::ZeroExtend { input }
            | SmallComponentKind::SignExtend { input }
            | SmallComponentKind::Slice { input, .. } => {
                smallvec![(input, "In".into())]
            }
            SmallComponentKind::Buffer { input, enable } => {
                smallvec![(input, "In".into()), (enable, "En".into())]
            }
            SmallComponentKind::LeftShift { input_a, input_b }
            | SmallComponentKind::LogicalRightShift { input_a, input_b }
            | SmallComponentKind::ArithmeticRightShift { input_a, input_b } => {
                smallvec![(input_a, "In".into()), (input_b, "Shamnt".into())]
            }
        }
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
            SmallComponentKind::Slice {
                input,
                start_offset,
                end_offset,
            } => {
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
            SmallComponentKind::Neg { input } => {
                let val = wire_states.get_state(input);
                let (width, out) = output_states.get_data(output_base);

                neg(
                    width,
                    out,
                    &mut LogicBitState::Undefined,
                    val,
                    LogicBitState::Logic1,
                )
            }
            SmallComponentKind::Mul { input_a, input_b } => {
                let lhs = wire_states.get_state(input_a);
                let rhs = wire_states.get_state(input_b);
                let (width, out) = output_states.get_data(output_base);
                mul(width, out, lhs, rhs)
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

pub trait Mutability {
    type Ref<'a, T: ?Sized>: std::ops::Deref<Target = T>
    where
        T: 'a;
}

pub enum Immutable {}
impl Mutability for Immutable {
    type Ref<'a, T: ?Sized> = &'a T where T: 'a;
}

pub enum Mutable {}
impl Mutability for Mutable {
    type Ref<'a, T: ?Sized> = &'a mut T where T: 'a;
}

pub struct MemoryCell<'a, M: Mutability> {
    width: NonZeroU8,
    mem: M::Ref<'a, [Atom]>,
    reset_value: M::Ref<'a, LogicState>,
}

impl<M: Mutability> MemoryCell<'_, M> {
    #[inline]
    pub fn width(&self) -> NonZeroU8 {
        self.width
    }

    pub fn read(&self) -> LogicState {
        LogicState(LogicStateRepr::Bits(self.mem.iter().copied().collect()))
    }

    #[inline]
    pub fn reset_value(&self) -> &LogicState {
        &self.reset_value
    }
}

impl MemoryCell<'_, Mutable> {
    pub fn write(&mut self, value: &LogicState) {
        for (dst, src) in self.mem.iter_mut().zip(value.iter_atoms()) {
            *dst = src;
        }
    }

    #[inline]
    pub fn set_reset_value(&mut self, value: LogicState) {
        *self.reset_value = value;
    }

    pub fn reset(&mut self) {
        for (dst, src) in self.mem.iter_mut().zip(self.reset_value.iter_atoms()) {
            *dst = src;
        }
    }
}

pub struct MemoryBlock<'a, M: Mutability> {
    width: NonZeroU8,
    mem: M::Ref<'a, Memory>,
    clear_value: M::Ref<'a, LogicState>,
}

impl<M: Mutability> MemoryBlock<'_, M> {
    #[inline]
    pub fn width(&self) -> NonZeroU8 {
        self.width
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    #[inline]
    pub fn read(&self, addr: usize) -> Option<LogicState> {
        self.mem.read(addr)
    }

    #[inline]
    pub fn clear_value(&self) -> &LogicState {
        &self.clear_value
    }
}

impl MemoryBlock<'_, Mutable> {
    #[inline]
    pub fn write(&mut self, addr: usize, value: &LogicState) -> Result<(), ()> {
        self.mem.write(addr, value.iter_atoms()).ok_or(())
    }

    #[inline]
    pub fn set_clear_value(&mut self, value: LogicState) {
        *self.clear_value = value;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.mem.fill(self.clear_value);
    }
}

/// Contains mutable data of a component
pub enum ComponentData<'a, M: Mutability> {
    /// The component does not store any data
    None,
    /// The component stores a single register value
    RegisterValue(MemoryCell<'a, M>),
    /// The component stores a memory block
    MemoryBlock(MemoryBlock<'a, M>),
}

pub(crate) trait LargeComponent: Send + Sync {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str>;

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)>;

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]>;

    fn alloc_size(&self) -> AllocationSize;

    fn get_data(&self) -> ComponentData<'_, Immutable> {
        ComponentData::None
    }

    fn get_data_mut(&mut self) -> ComponentData<'_, Mutable> {
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
    ($name:ident, $op3:ident, $op2:ident, $node_name:literal) => {
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
            #[cfg(feature = "dot-export")]
            fn node_name(&self) -> Cow<'static, str> {
                $node_name.into()
            }

            #[cfg(feature = "dot-export")]
            fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
                vec![(self.output_wire, "Out".into())]
            }

            #[cfg(feature = "dot-export")]
            fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
                self.inputs
                    .iter()
                    .enumerate()
                    .map(|(i, &wire)| (wire, format!("In{i}").into()))
                    .collect()
            }

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
    ($name:ident, $op3:ident, $op2:ident, $node_name:literal) => {
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
            #[cfg(feature = "dot-export")]
            fn node_name(&self) -> Cow<'static, str> {
                $node_name.into()
            }

            #[cfg(feature = "dot-export")]
            fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
                vec![(self.output_wire, "Out".into())]
            }

            #[cfg(feature = "dot-export")]
            fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
                self.inputs
                    .iter()
                    .enumerate()
                    .map(|(i, &wire)| (wire, format!("In{i}").into()))
                    .collect()
            }

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

wide_gate!(WideAndGate, logic_and_3, logic_and_2, "AND");
wide_gate!(WideOrGate, logic_or_3, logic_or_2, "OR");
wide_gate!(WideXorGate, logic_xor_3, logic_xor_2, "XOR");
wide_gate_inv!(WideNandGate, logic_and_3, logic_and_2, "NAND");
wide_gate_inv!(WideNorGate, logic_or_3, logic_or_2, "NOR");
wide_gate_inv!(WideXnorGate, logic_xor_3, logic_xor_2, "XNOR");

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
        debug_assert!(!inputs.is_empty());

        Self {
            inputs,
            output,
            output_wire,
        }
    }
}

impl LargeComponent for Merge {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "{,}".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.output_wire, "Out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, &wire)| (wire, format!("In{i}").into()))
            .collect()
    }

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
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "Adder".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![
            (self.output_wire, "Sum".into()),
            (self.carry_out_wire, "Carry out".into()),
        ]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        vec![
            (self.input_a, "A".into()),
            (self.input_b, "B".into()),
            (self.carry_in, "Carry in".into()),
        ]
    }

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
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "MUX".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.output_wire, "Out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        let mut result: Vec<_> = self
            .inputs
            .iter()
            .enumerate()
            .map(|(i, &wire)| (wire, format!("In{i}").into()))
            .collect();
        result.push((self.select, "Select".into()));
        result
    }

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

#[derive(Debug)]
pub(crate) struct PriorityDecoder {
    inputs: inline_vec!(WireStateId),
    output: OutputStateId,
    output_wire: WireId,
}

impl PriorityDecoder {
    #[inline]
    pub(crate) fn new(
        inputs: impl Into<inline_vec!(WireStateId)>,
        output: OutputStateId,
        output_wire: WireId,
    ) -> Self {
        Self {
            inputs: inputs.into(),
            output,
            output_wire,
        }
    }
}

impl LargeComponent for PriorityDecoder {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "Decoder".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.output_wire, "Out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, &wire)| (wire, format!("In{i}").into()))
            .collect()
    }

    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn update(
        &mut self,
        wire_states: &WireStateList,
        mut output_states: OutputStateSlice<'_>,
    ) -> inline_vec!(WireId) {
        let mut new_output_state = Atom::LOGIC_0;

        for (i, input) in self.inputs.iter().copied().enumerate() {
            match wire_states.get_state(input)[0].get_bit_state(AtomOffset::MIN) {
                LogicBitState::HighZ | LogicBitState::Undefined => {
                    new_output_state = Atom::UNDEFINED;
                    break;
                }
                LogicBitState::Logic1 => {
                    new_output_state = Atom::from_int((i + 1) as u32);
                    break;
                }
                LogicBitState::Logic0 => continue,
            }
        }

        let (width, out) = output_states.get_data(self.output);
        if !out[0].eq(new_output_state, AtomWidth::new(width.get()).unwrap()) {
            out[0] = new_output_state;
            smallvec![self.output_wire]
        } else {
            smallvec![]
        }
    }
}

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
    width: NonZeroU8,
    data_in: WireStateId,
    data_out: OutputStateId,
    data_out_wire: WireId,
    enable: WireStateId,
    clock: WireStateId,
    clock_trigger: ClockTrigger,
    reset_value: LogicState,
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
            width,
            data_in,
            data_out,
            data_out_wire,
            enable,
            clock,
            clock_trigger: ClockTrigger::new(clock_polarity),
            reset_value: LogicState::UNDEFINED,
            data: smallvec![Atom::UNDEFINED; atom_count],
        }
    }

    #[inline]
    fn cell(&self) -> MemoryCell<'_, Immutable> {
        MemoryCell {
            width: self.width,
            mem: &self.data,
            reset_value: &self.reset_value,
        }
    }

    #[inline]
    fn cell_mut(&mut self) -> MemoryCell<'_, Mutable> {
        MemoryCell {
            width: self.width,
            mem: &mut self.data,
            reset_value: &mut self.reset_value,
        }
    }
}

impl LargeComponent for Register {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "Register".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.data_out_wire, "Data out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        vec![
            (self.data_in, "Data in".into()),
            (self.enable, "En".into()),
            (self.clock, "Clk".into()),
        ]
    }

    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&self) -> ComponentData<'_, Immutable> {
        ComponentData::RegisterValue(self.cell())
    }

    fn get_data_mut(&mut self) -> ComponentData<'_, Mutable> {
        ComponentData::RegisterValue(self.cell_mut())
    }

    fn reset(&mut self) {
        self.clock_trigger.reset();
        self.cell_mut().reset();
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
    fn read(&self, addr: usize) -> Option<LogicState> {
        let (state, valid) = match self {
            Self::U8(atoms) => {
                let &[state, valid] = atoms.get(addr)?;
                (state as u32, valid as u32)
            }
            Self::U16(atoms) => {
                let &[state, valid] = atoms.get(addr)?;
                (state as u32, valid as u32)
            }
            Self::U32(atoms) => {
                let &[state, valid] = atoms.get(addr)?;
                (state as u32, valid as u32)
            }
            Self::Big { atom_width, atoms } => {
                let start = addr * (atom_width.get() as usize);
                let end = start + (atom_width.get() as usize);
                let &slice = &atoms.get(start..end)?;

                return Some(LogicState(LogicStateRepr::Bits(
                    slice.iter().copied().collect(),
                )));
            }
        };

        let value = Atom::from_state_valid(state, valid);
        Some(LogicState(LogicStateRepr::Bits(smallvec![value])))
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
    fn write(&mut self, addr: usize, mut value: impl Iterator<Item = Atom>) -> Option<()> {
        match self {
            Self::U8(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                let atom = atoms.get_mut(addr)?;
                *atom = [state as u8, valid as u8];
            }
            Self::U16(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                let atom = atoms.get_mut(addr)?;
                *atom = [state as u16, valid as u16];
            }
            Self::U32(atoms) => {
                let (state, valid) = value.next().unwrap().to_state_valid();
                let atom = atoms.get_mut(addr)?;
                *atom = [state as u32, valid as u32];
            }
            Self::Big { atom_width, atoms } => {
                let start = addr * (atom_width.get() as usize);
                let end = start + (atom_width.get() as usize);
                let slice = atoms.get_mut(start..end)?;
                for (dst, src) in izip!(slice, value) {
                    *dst = src;
                }
            }
        }

        Some(())
    }

    #[allow(clippy::unnecessary_cast)]
    fn fill(&mut self, value: &LogicState) {
        match self {
            Self::U8(atoms) => {
                let (state, valid) = value.iter_atoms().next().unwrap().to_state_valid();
                atoms.fill([state as u8, valid as u8]);
            }
            Self::U16(atoms) => {
                let (state, valid) = value.iter_atoms().next().unwrap().to_state_valid();
                atoms.fill([state as u16, valid as u16]);
            }
            Self::U32(atoms) => {
                let (state, valid) = value.iter_atoms().next().unwrap().to_state_valid();
                atoms.fill([state as u32, valid as u32]);
            }
            &mut Self::Big {
                atom_width,
                ref mut atoms,
            } => {
                let len = atoms.len() / (atom_width.get() as usize);
                let mut atoms = atoms.iter_mut();
                for _ in 0..len {
                    atoms
                        .by_ref()
                        .zip(value.iter_atoms())
                        .take(atom_width.get() as usize)
                        .for_each(|(dst, src)| *dst = src);
                }
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
    for (i, atom) in atoms.iter().enumerate() {
        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
        total_width -= width.get();

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
    data_width: NonZeroU8,
    clear_value: LogicState,
    mem: Memory,
}

impl Ram {
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
            data_width,
            clear_value: LogicState::UNDEFINED,
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }

    #[inline]
    fn block(&self) -> MemoryBlock<'_, Immutable> {
        MemoryBlock {
            width: self.data_width,
            mem: &self.mem,
            clear_value: &self.clear_value,
        }
    }

    #[inline]
    fn block_mut(&mut self) -> MemoryBlock<'_, Mutable> {
        MemoryBlock {
            width: self.data_width,
            mem: &mut self.mem,
            clear_value: &mut self.clear_value,
        }
    }
}

impl LargeComponent for Ram {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "RAM".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.data_out_wire, "Data out".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        vec![
            (self.write_addr, "Write addr".into()),
            (self.data_in, "Data in".into()),
            (self.read_addr, "Read addr".into()),
            (self.write, "Write".into()),
            (self.clock, "Clk".into()),
        ]
    }

    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&self) -> ComponentData<'_, Immutable> {
        ComponentData::MemoryBlock(self.block())
    }

    fn get_data_mut(&mut self) -> ComponentData<'_, Mutable> {
        ComponentData::MemoryBlock(self.block_mut())
    }

    fn reset(&mut self) {
        self.clock_trigger.reset();
        self.block_mut().clear();
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
                        let result = self.mem.write(write_addr, data_iter);
                        debug_assert!(result.is_some());
                    }
                    LogicBitState::Logic0 => (),
                    LogicBitState::Logic1 => {
                        let data_iter = data_in.iter().copied().map(Atom::high_z_to_undefined);
                        let result = self.mem.write(write_addr, data_iter);
                        debug_assert!(result.is_some());
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
    data_width: NonZeroU8,
    clear_value: LogicState,
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
            data_width,
            clear_value: LogicState::UNDEFINED,
            mem: Memory::new(data_width, 1usize << addr_width.get()),
        }
    }

    #[inline]
    fn block(&self) -> MemoryBlock<'_, Immutable> {
        MemoryBlock {
            width: self.data_width,
            mem: &self.mem,
            clear_value: &self.clear_value,
        }
    }

    #[inline]
    fn block_mut(&mut self) -> MemoryBlock<'_, Mutable> {
        MemoryBlock {
            width: self.data_width,
            mem: &mut self.mem,
            clear_value: &mut self.clear_value,
        }
    }
}

impl LargeComponent for Rom {
    #[cfg(feature = "dot-export")]
    fn node_name(&self) -> Cow<'static, str> {
        "ROM".into()
    }

    #[cfg(feature = "dot-export")]
    fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        vec![(self.data_wire, "Data".into())]
    }

    #[cfg(feature = "dot-export")]
    fn input_wires(&self) -> Vec<(WireStateId, Cow<'static, str>)> {
        vec![(self.addr, "Addr".into())]
    }

    fn alloc_size(&self) -> AllocationSize {
        AllocationSize(std::mem::size_of::<Self>())
    }

    fn get_data(&self) -> ComponentData<'_, Immutable> {
        ComponentData::MemoryBlock(self.block())
    }

    fn get_data_mut(&mut self) -> ComponentData<'_, Mutable> {
        ComponentData::MemoryBlock(self.block_mut())
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
        output_start: OutputStateId,
        output_end: OutputStateId,
    },
    Large {
        component: Box<dyn LargeComponent>,
        output_start: OutputStateId,
        output_end: OutputStateId,
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
            output_start: output_base,
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
            output_start: output_base,
            output_atom_count,
        }
    }

    #[cfg(feature = "dot-export")]
    pub(crate) fn output_wires(&self) -> Vec<(WireId, Cow<'static, str>)> {
        match self {
            Component::Small { component, .. } => vec![(component.output, "Out".into())],
            Component::Large { component, .. } => component.output_wires(),
        }
    }

    #[cfg(feature = "dot-export")]
    pub(crate) fn input_wires(&self) -> SmallVec<[(WireStateId, Cow<'static, str>); 2]> {
        match self {
            Component::Small { component, .. } => component.input_wires(),
            Component::Large { component, .. } => component.input_wires(),
        }
    }

    #[cfg(feature = "dot-export")]
    pub(crate) fn node_name(&self, output_states: &OutputStateList) -> Cow<'static, str> {
        match self {
            Component::Small {
                component,
                output_start: output_base,
                ..
            } => component.node_name(*output_base, output_states),
            Component::Large { component, .. } => component.node_name(),
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
    pub(crate) fn output_range(&self) -> (OutputStateId, OutputStateId) {
        match self {
            &Self::Small {
                output_start: output_base,
                output_atom_count,
                ..
            }
            | &Self::Large {
                output_start: output_base,
                output_atom_count,
                ..
            } => (output_base, output_atom_count),
        }
    }

    #[inline]
    pub(crate) fn get_data(&self) -> ComponentData<'_, Immutable> {
        match self {
            Self::Small { .. } => ComponentData::None,
            Self::Large { component, .. } => component.get_data(),
        }
    }

    #[inline]
    pub(crate) fn get_data_mut(&mut self) -> ComponentData<'_, Mutable> {
        match self {
            Self::Small { .. } => ComponentData::None,
            Self::Large { component, .. } => component.get_data_mut(),
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
                output_start: output_base,
                ..
            } => component.update(output_base, wire_states, output_states),
            Self::Large { component, .. } => component.update(wire_states, output_states),
        }
    }
}
*/
