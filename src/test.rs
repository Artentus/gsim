use crate::*;

mod component;
//#[cfg(feature = "dot-export")]
//mod dot_export;
//mod import;

macro_rules! logic_state {
    ($width:expr; $state:ident) => {
        LogicState::$state($width)
    };
    ($width:expr; {% $($bit:tt),*}) => {
        $crate::bits!($($bit),*)
    };
    ($width:expr; {$value:expr}) => {
        LogicState::from_u64($value, $width)
    };
    ($width:expr; [$($value:expr),+ $(,)?]) => {
        LogicState::from_big_int($width, [$($value),+].as_slice())
    };
    ($width:expr; $value:expr) => {
        LogicState::from_u64($value, $width)
    };
}

use logic_state;

struct BinaryGateTestData {
    input_a: LogicState,
    input_b: LogicState,
    output: LogicState,
}

fn test_binary_gate<F>(
    add_gate: F,
    width: BitWidth,
    test_data: &[BinaryGateTestData],
    max_steps: u64,
) where
    F: Fn(&mut SimulatorBuilder, WireId, WireId, WireId) -> AddComponentResult,
{
    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input_a = builder.add_wire(width).unwrap();
        builder.set_wire_drive(input_a, &test_data.input_a).unwrap();
        let input_b = builder.add_wire(width).unwrap();
        builder.set_wire_drive(input_b, &test_data.input_b).unwrap();
        let output = builder.add_wire(width).unwrap();
        let _gate = add_gate(&mut builder, input_a, input_b, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

        assert_eq!(
            output_state, test_data.output,
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output, output_state,
        );
    }
}

fn test_shifter<F>(add_gate: F, width: BitWidth, test_data: &[BinaryGateTestData], max_steps: u64)
where
    F: Fn(&mut SimulatorBuilder, WireId, WireId, WireId) -> AddComponentResult,
{
    let shamnt_width = width.clog2().unwrap();

    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input_a = builder.add_wire(width).unwrap();
        builder.set_wire_drive(input_a, &test_data.input_a).unwrap();
        let input_b = builder.add_wire(shamnt_width).unwrap();
        builder.set_wire_drive(input_b, &test_data.input_b).unwrap();
        let output = builder.add_wire(width).unwrap();
        let _gate = add_gate(&mut builder, input_a, input_b, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

        assert_eq!(
            output_state, test_data.output,
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output, output_state,
        );
    }
}

//fn test_binary_module(
//    sim: &mut Simulator,
//    input_a: WireId,
//    input_b: WireId,
//    output: WireId,
//    width: NonZeroU8,
//    test_data: &[BinaryGateTestData],
//    max_steps: u64,
//) {
//    for (i, test_data) in test_data.iter().enumerate() {
//        sim.set_wire_drive(input_a, &test_data.input_a).unwrap();
//        sim.set_wire_drive(input_b, &test_data.input_b).unwrap();
//
//        match sim.run_sim(max_steps) {
//            SimulationRunResult::Ok => {}
//            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
//            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
//        }
//
//        let output_state = sim.get_wire_state(output).unwrap();
//
//        assert!(
//            output_state.eq(&test_data.output, width),
//            "[TEST {i}]  expected: {}  actual: {}",
//            test_data.output.display_string(width),
//            output_state.display_string(width),
//        );
//    }
//}

macro_rules! binary_gate_test_data {
    ($width:expr; $(($a:tt, $b:tt) -> $o:tt),* $(,)?) => {
        &[
            $(
                BinaryGateTestData {
                    input_a: logic_state!($width; $a),
                    input_b: logic_state!($width; $b),
                    output: logic_state!($width; $o),
                },
            )*
        ]
    };
}

use binary_gate_test_data;

macro_rules! shifter_test_data {
    ($width:expr; $(($a:tt, $b:tt) -> $o:tt),* $(,)?) => {
        &[
            $(
                BinaryGateTestData {
                    input_a: logic_state!($width; $a),
                    input_b: logic_state!($width.clog2().unwrap(); $b),
                    output: logic_state!($width; $o),
                },
            )*
        ]
    };
}

use shifter_test_data;

struct UnaryGateTestData {
    input: LogicState,
    output: LogicState,
}

fn test_unary_gate<F>(add_gate: F, width: BitWidth, test_data: &[UnaryGateTestData], max_steps: u64)
where
    F: Fn(&mut SimulatorBuilder, WireId, WireId) -> AddComponentResult,
{
    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input = builder.add_wire(width).unwrap();
        builder.set_wire_drive(input, &test_data.input).unwrap();
        let output = builder.add_wire(width).unwrap();
        let _gate = add_gate(&mut builder, input, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

        assert_eq!(
            output_state, test_data.output,
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output, output_state,
        );
    }
}

fn test_horizontal_gate<F>(
    add_gate: F,
    width: BitWidth,
    test_data: &[UnaryGateTestData],
    max_steps: u64,
) where
    F: Fn(&mut SimulatorBuilder, WireId, WireId) -> AddComponentResult,
{
    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input = builder.add_wire(width).unwrap();
        builder.set_wire_drive(input, &test_data.input).unwrap();
        let output = builder.add_wire(BitWidth::MIN).unwrap();
        let _gate = add_gate(&mut builder, input, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

        assert_eq!(
            output_state, test_data.output,
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output, output_state,
        );
    }
}

macro_rules! unary_gate_test_data {
    ($width:expr; $($i:tt -> $o:tt),* $(,)?) => {
        &[
            $(
                UnaryGateTestData {
                    input: logic_state!($width; $i),
                    output: logic_state!($width; $o),
                },
            )*
        ]
    };
}

use unary_gate_test_data;

macro_rules! horizontal_gate_test_data {
    ($width:expr; $($i:tt -> $o:tt),* $(,)?) => {
        &[
            $(
                UnaryGateTestData {
                    input: logic_state!($width; $i),
                    output: logic_state!(BitWidth::MIN; $o),
                },
            )*
        ]
    };
}

use horizontal_gate_test_data;

macro_rules! extend_test_data {
    ($input_width:expr; $output_width:expr; $($i:tt -> $o:tt),* $(,)?) => {
        &[
            $(
                UnaryGateTestData {
                    input: logic_state!($input_width; $i),
                    output: logic_state!($output_width; $o),
                },
            )*
        ]
    };
}

use extend_test_data;

struct WideGateTestData<'a> {
    inputs: &'a [LogicState],
    output: LogicState,
}

fn test_wide_gate<F>(add_gate: F, width: BitWidth, test_data: &[WideGateTestData], max_steps: u64)
where
    F: Fn(&mut SimulatorBuilder, &[WireId], WireId) -> AddComponentResult,
{
    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|drive| {
                let wire = builder.add_wire(width).unwrap();
                builder.set_wire_drive(wire, drive).unwrap();
                wire
            })
            .collect();
        let output = builder.add_wire(width).unwrap();
        let _gate = add_gate(&mut builder, &inputs, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

        assert_eq!(
            output_state, test_data.output,
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output, output_state,
        );
    }
}

macro_rules! wide_gate_test_data {
    ($width:expr; $(($($i:tt),+) -> $o:tt),* $(,)?) => {
        &[
            $(
                WideGateTestData {
                    inputs: &[$(logic_state!($width; $i)),+],
                    output: logic_state!($width; $o),
                },
            )*
        ]
    };
}

use wide_gate_test_data;

fn test_comparator<F>(add_comparator: F, compare_op: impl Fn(u32, u32) -> bool)
where
    F: Fn(&mut SimulatorBuilder, WireId, WireId, WireId) -> AddComponentResult,
{
    const WIDTH: BitWidth = bit_width!(4);

    let mut builder = SimulatorBuilder::default();

    let input_a = builder.add_wire(WIDTH).unwrap();
    let input_b = builder.add_wire(WIDTH).unwrap();
    let output = builder.add_wire(BitWidth::MIN).unwrap();
    let _comparator = add_comparator(&mut builder, input_a, input_b, output).unwrap();

    let mut sim = builder.build();

    for a in 0..16 {
        for b in 0..16 {
            sim.set_wire_drive(input_a, &LogicState::from_u32(a, WIDTH))
                .unwrap();
            sim.set_wire_drive(input_b, &LogicState::from_u32(b, WIDTH))
                .unwrap();

            match sim.run_sim(2) {
                SimulationRunResult::Ok => {}
                SimulationRunResult::MaxStepsReached => {
                    panic!("[TEST ({a}, {b})] exceeded max steps")
                }
                SimulationRunResult::Err(err) => panic!("[TEST ({a}, {b})] {err:?}"),
            }

            let expected = LogicState::from_bool(compare_op(a, b));
            let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

            assert_eq!(
                output_state, expected,
                "[TEST ({a}, {b})]  expected: {}  actual: {}",
                expected, output_state,
            );
        }
    }
}

fn test_signed_comparator<F>(add_comparator: F, compare_op: impl Fn(i32, i32) -> bool)
where
    F: Fn(&mut SimulatorBuilder, WireId, WireId, WireId) -> AddComponentResult,
{
    const WIDTH: BitWidth = bit_width!(4);

    let mut builder = SimulatorBuilder::default();

    let input_a = builder.add_wire(WIDTH).unwrap();
    let input_b = builder.add_wire(WIDTH).unwrap();
    let output = builder.add_wire(BitWidth::MIN).unwrap();
    let _comparator = add_comparator(&mut builder, input_a, input_b, output).unwrap();

    let mut sim = builder.build();

    for a in -8..8 {
        for b in -8..8 {
            sim.set_wire_drive(input_a, &LogicState::from_u32(a as u32, WIDTH))
                .unwrap();
            sim.set_wire_drive(input_b, &LogicState::from_u32(b as u32, WIDTH))
                .unwrap();

            match sim.run_sim(2) {
                SimulationRunResult::Ok => {}
                SimulationRunResult::MaxStepsReached => {
                    panic!("[TEST ({a}, {b})] exceeded max steps")
                }
                SimulationRunResult::Err(err) => panic!("[TEST ({a}, {b})] {err:?}"),
            }

            let expected = LogicState::from_bool(compare_op(a, b));
            let [output_state, _] = sim.get_wire_state_and_drive(output).unwrap();

            assert_eq!(
                output_state, expected,
                "[TEST ({a}, {b})]  expected: {}  actual: {}",
                expected, output_state,
            );
        }
    }
}
