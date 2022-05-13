use std::marker::PhantomData;

use super::*;

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
