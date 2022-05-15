use super::*;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

pub struct ConstantBehaviour {
    state: Box<[LogicState]>,
    strength: OutputStrength,
    outputs: [u32; 1],
}
impl ConstantBehaviour {
    #[inline]
    pub const fn new(state: Box<[LogicState]>, strength: OutputStrength) -> Self {
        assert!(state.len() <= (u32::MAX as usize));
        let width = state.len() as u32;
        assert!(width > 0);

        Self {
            state,
            strength,
            outputs: [width],
        }
    }

    pub fn new_pull_down(width: u32) -> Self {
        let state = vec![LogicState::Logic0; width as usize].into_boxed_slice();
        Self::new(state, OutputStrength::Weak)
    }

    pub fn new_pull_up(width: u32) -> Self {
        let state = vec![LogicState::Logic1; width as usize].into_boxed_slice();
        Self::new(state, OutputStrength::Weak)
    }

    pub fn new_from_value(value: &[u32], width: u32) -> Self {
        assert!((value.len() * 32) >= (width as usize));

        fn get_bit(value: &[u32], index: u32) -> bool {
            let word_index = index / 32;
            let bit_index = index % 32;

            let word = value[word_index as usize];
            let bit = (word >> bit_index) & 1;
            bit > 0
        }

        let mut state = Vec::with_capacity(width as usize);
        for i in 0..width {
            let bit = get_bit(value, i);
            state.push(if bit {
                LogicState::Logic1
            } else {
                LogicState::Logic0
            });
        }

        Self::new(state.into_boxed_slice(), OutputStrength::Strong)
    }
}
impl ComponentBehaviour for ConstantBehaviour {
    fn output_configuration(&self) -> &[u32] {
        &self.outputs
    }

    fn input_configuration(&self) -> &[u32] {
        &[]
    }

    fn update(
        &mut self,
        outputs: &mut [Output],
        _inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool> {
        let mut changed = outputs[0].strength != self.strength;
        outputs[0].strength = self.strength;

        for (i, state) in outputs[0].state.iter_mut().enumerate() {
            changed |= *state != self.state[i];
            *state = self.state[i];
        }

        Ok(changed)
    }
}

pub trait UnaryOperation: Send + Sync {
    fn execute(value: LogicState) -> LogicState;
}

pub trait BinaryOperation: Send + Sync {
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState;
}

pub struct Not;
impl UnaryOperation for Not {
    #[inline]
    fn execute(value: LogicState) -> LogicState {
        match value {
            LogicState::Undefined => LogicState::Undefined,
            LogicState::HighZ => LogicState::Undefined,
            LogicState::Logic0 => LogicState::Logic1,
            LogicState::Logic1 => LogicState::Logic0,
        }
    }
}

pub struct And;
impl BinaryOperation for And {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Logic0, _) => LogicState::Logic0,
            (_, LogicState::Logic0) => LogicState::Logic0,

            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic1, LogicState::Logic1) => LogicState::Logic1,
        }
    }
}

pub struct Nand;
impl BinaryOperation for Nand {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Logic0, _) => LogicState::Logic1,
            (_, LogicState::Logic0) => LogicState::Logic1,

            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic1, LogicState::Logic1) => LogicState::Logic0,
        }
    }
}

pub struct Or;
impl BinaryOperation for Or {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Logic1, _) => LogicState::Logic1,
            (_, LogicState::Logic1) => LogicState::Logic1,

            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic0, LogicState::Logic0) => LogicState::Logic0,
        }
    }
}

pub struct Nor;
impl BinaryOperation for Nor {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Logic1, _) => LogicState::Logic0,
            (_, LogicState::Logic1) => LogicState::Logic0,

            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic0, LogicState::Logic0) => LogicState::Logic1,
        }
    }
}

pub struct Xor;
impl BinaryOperation for Xor {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic0, LogicState::Logic0) => LogicState::Logic0,
            (LogicState::Logic0, LogicState::Logic1) => LogicState::Logic1,
            (LogicState::Logic1, LogicState::Logic0) => LogicState::Logic1,
            (LogicState::Logic1, LogicState::Logic1) => LogicState::Logic0,
        }
    }
}

pub struct Xnor;
impl BinaryOperation for Xnor {
    #[inline]
    fn execute(lhs: LogicState, rhs: LogicState) -> LogicState {
        match (lhs, rhs) {
            (LogicState::Undefined, _) => LogicState::Undefined,
            (_, LogicState::Undefined) => LogicState::Undefined,

            (LogicState::HighZ, _) => LogicState::Undefined,
            (_, LogicState::HighZ) => LogicState::Undefined,

            (LogicState::Logic0, LogicState::Logic0) => LogicState::Logic1,
            (LogicState::Logic0, LogicState::Logic1) => LogicState::Logic0,
            (LogicState::Logic1, LogicState::Logic0) => LogicState::Logic0,
            (LogicState::Logic1, LogicState::Logic1) => LogicState::Logic1,
        }
    }
}

pub struct UnaryBehaviour<Op: UnaryOperation> {
    outputs: [u32; 1],
    inputs: [u32; 1],
    _op: PhantomData<Op>,
}
impl<Op: UnaryOperation> UnaryBehaviour<Op> {
    #[inline]
    pub const fn new(width: u32) -> Self {
        assert!(width > 0);

        Self {
            outputs: [width],
            inputs: [width],
            _op: PhantomData,
        }
    }
}
impl<Op: UnaryOperation> ComponentBehaviour for UnaryBehaviour<Op> {
    fn output_configuration(&self) -> &[u32] {
        &self.outputs
    }

    fn input_configuration(&self) -> &[u32] {
        &self.inputs
    }

    fn update(
        &mut self,
        outputs: &mut [Output],
        inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool> {
        let mut changed = outputs[0].strength != OutputStrength::Strong;
        outputs[0].strength = OutputStrength::Strong;

        for (i, state) in outputs[0].state.iter_mut().enumerate() {
            let new_state = Op::execute(inputs[0][i]);

            changed |= new_state != *state;
            *state = new_state;
        }

        Ok(changed)
    }
}

pub struct BinaryBehaviour<Op: BinaryOperation> {
    outputs: [u32; 1],
    inputs: Box<[u32]>,
    _op: PhantomData<Op>,
}
impl<Op: BinaryOperation> BinaryBehaviour<Op> {
    #[inline]
    pub fn new(width: u32, input_count: u32) -> Self {
        assert!(width > 0);
        assert!(input_count >= 2);

        let inputs = vec![width; input_count as usize].into_boxed_slice();

        Self {
            outputs: [width],
            inputs,
            _op: PhantomData,
        }
    }
}
impl<Op: BinaryOperation> ComponentBehaviour for BinaryBehaviour<Op> {
    fn output_configuration(&self) -> &[u32] {
        &self.outputs
    }

    fn input_configuration(&self) -> &[u32] {
        &self.inputs
    }

    fn update(
        &mut self,
        outputs: &mut [Output],
        inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool> {
        let mut changed = outputs[0].strength != OutputStrength::Strong;
        outputs[0].strength = OutputStrength::Strong;

        for (i, state) in outputs[0].state.iter_mut().enumerate() {
            let mut new_state = inputs[0][i];
            for input in inputs.iter().skip(1) {
                new_state = Op::execute(new_state, input[i]);
            }

            changed |= new_state != *state;
            *state = new_state;
        }

        Ok(changed)
    }
}

#[repr(transparent)]
struct AtomicLogicState(AtomicU8);
impl AtomicLogicState {
    #[inline]
    pub fn new_array(val: LogicState, n: usize) -> Box<[AtomicLogicState]> {
        let u8_array = vec![(val as u32) as u8; n].into_boxed_slice();
        let u8_raw = Box::into_raw(u8_array);

        unsafe {
            let atomic_raw = u8_raw as *mut [AtomicLogicState];
            Box::from_raw(atomic_raw)
        }
    }

    #[inline]
    pub fn load(&self, order: Ordering) -> LogicState {
        unsafe { std::mem::transmute(self.0.load(order) as u32) }
    }

    #[inline]
    pub fn store(&self, val: LogicState, order: Ordering) {
        self.0.store((val as u32) as u8, order);
    }
}

pub struct InputPin {
    state: Box<[AtomicLogicState]>,
    changed: AtomicBool,
}
impl InputPin {
    pub fn new(width: u32) -> Arc<Self> {
        assert!(width > 0);

        Arc::new(Self {
            state: AtomicLogicState::new_array(LogicState::HighZ, width as usize),
            changed: AtomicBool::new(false),
        })
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.state.len() as u32
    }

    pub fn set(&self, state: &[LogicState]) {
        for (i, s) in self.state.iter().enumerate() {
            s.store(
                state.get(i).copied().unwrap_or(LogicState::HighZ),
                Ordering::Relaxed,
            );
        }

        self.changed.store(true, Ordering::Relaxed);
    }
}

pub struct InputPinBehaviour {
    pin: Arc<InputPin>,
    outputs: [u32; 1],
}
impl InputPinBehaviour {
    pub fn new(pin: Arc<InputPin>) -> Self {
        let outputs = [pin.width()];

        Self { pin, outputs }
    }
}
impl ComponentBehaviour for InputPinBehaviour {
    fn output_configuration(&self) -> &[u32] {
        &self.outputs
    }

    fn input_configuration(&self) -> &[u32] {
        &[]
    }

    fn update(
        &mut self,
        outputs: &mut [Output],
        _inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool> {
        let changed = (outputs[0].strength != OutputStrength::Strong)
            | self.pin.changed.swap(false, Ordering::Relaxed);
        outputs[0].strength = OutputStrength::Strong;

        for (i, state) in outputs[0].state.iter_mut().enumerate() {
            *state = self.pin.state[i].load(Ordering::Relaxed);
        }

        Ok(changed)
    }
}

pub struct OutputPin {
    state: Box<[AtomicLogicState]>,
}
impl OutputPin {
    pub fn new(width: u32) -> Arc<Self> {
        assert!(width > 0);

        Arc::new(Self {
            state: AtomicLogicState::new_array(LogicState::HighZ, width as usize),
        })
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.state.len() as u32
    }

    pub fn get(&self, state: &mut [LogicState]) {
        for (i, s) in state.iter_mut().enumerate() {
            (*s) = self
                .state
                .get(i)
                .map(|a| a.load(Ordering::Relaxed))
                .unwrap_or(LogicState::HighZ);
        }
    }
}

pub struct OutputPinBehaviour {
    pin: Arc<OutputPin>,
    inputs: [u32; 1],
}
impl OutputPinBehaviour {
    pub fn new(pin: Arc<OutputPin>) -> Self {
        let inputs = [pin.width()];

        Self { pin, inputs }
    }
}
impl ComponentBehaviour for OutputPinBehaviour {
    fn output_configuration(&self) -> &[u32] {
        &[]
    }

    fn input_configuration(&self) -> &[u32] {
        &self.inputs
    }

    fn update(
        &mut self,
        _outputs: &mut [Output],
        inputs: &[Box<[LogicState]>],
    ) -> SimulationResult<bool> {
        for (i, s) in self.pin.state.iter().enumerate() {
            s.store(inputs[0][i], Ordering::Relaxed);
        }

        Ok(false)
    }
}
