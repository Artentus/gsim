use crate::*;

macro_rules! logic_state {
    ($state:ident) => {
        LogicState::$state
    };
    ($value:expr) => {
        LogicState::from_int($value)
    };
}

struct BinaryGateTestData {
    input_a: LogicState,
    input_b: LogicState,
    output: LogicState,
}

fn test_binary_gate<F>(
    add_gate: F,
    width: LogicWidth,
    test_data: &[BinaryGateTestData],
    max_steps: u64,
) where
    F: FnOnce(&mut SimulatorBuilder, WireId, WireId, WireId) -> AddComponentResult,
{
    let mut builder = SimulatorBuilder::default();

    let input_a = builder.add_wire(width);
    let input_b = builder.add_wire(width);
    let output = builder.add_wire(width);
    let _gate = add_gate(&mut builder, input_a, input_b, output).unwrap();

    let mut sim = builder.build();

    for (i, test_data) in test_data.iter().enumerate() {
        sim.set_wire_base_drive(input_a, test_data.input_a);
        sim.set_wire_base_drive(input_b, test_data.input_b);

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq_width(&test_data.output, width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(width),
            output_state.display_string(width),
        );
    }
}

macro_rules! binary_gate_test_data {
    ($(($a:tt, $b:tt) -> $o:tt),* $(,)?) => {
        &[
            $(
                BinaryGateTestData {
                    input_a: logic_state!($a),
                    input_b: logic_state!($b),
                    output: logic_state!($o),
                },
            )*
        ]
    };
}

struct UnaryGateTestData {
    input: LogicState,
    output: LogicState,
}

fn test_unary_gate<F>(
    add_gate: F,
    width: LogicWidth,
    test_data: &[UnaryGateTestData],
    max_steps: u64,
) where
    F: FnOnce(&mut SimulatorBuilder, WireId, WireId) -> AddComponentResult,
{
    let mut builder = SimulatorBuilder::default();

    let input = builder.add_wire(width);
    let output = builder.add_wire(width);
    let _gate = add_gate(&mut builder, input, output).unwrap();

    let mut sim = builder.build();

    for (i, test_data) in test_data.iter().enumerate() {
        sim.set_wire_base_drive(input, test_data.input);

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq_width(&test_data.output, width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(width),
            output_state.display_string(width),
        );
    }
}

macro_rules! unary_gate_test_data {
    ($($i:tt -> $o:tt),* $(,)?) => {
        &[
            $(
                UnaryGateTestData {
                    input: logic_state!($i),
                    output: logic_state!($o),
                },
            )*
        ]
    };
}

struct WideGateTestData {
    inputs: &'static [LogicState],
    output: LogicState,
}

fn test_wide_gate<F>(add_gate: F, width: LogicWidth, test_data: &[WideGateTestData], max_steps: u64)
where
    F: Fn(&mut SimulatorBuilder, &[WireId], WireId) -> AddComponentResult,
{
    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|&drive| {
                let wire = builder.add_wire(width);
                builder.set_wire_base_drive(wire, drive);
                wire
            })
            .collect();
        let output = builder.add_wire(width);
        let _gate = add_gate(&mut builder, &inputs, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(max_steps) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq_width(&test_data.output, width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(width),
            output_state.display_string(width),
        );
    }
}

macro_rules! wide_gate_test_data {
    ($(($($i:tt),+) -> $o:tt),* $(,)?) => {
        &[
            $(
                WideGateTestData {
                    inputs: &[$(logic_state!($i)),+],
                    output: logic_state!($o),
                },
            )*
        ]
    };
}

#[test]
fn test_and_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> LOGIC_0,
        (HIGH_Z, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0) -> LOGIC_0,
        (UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0, HIGH_Z) -> LOGIC_0,
        (LOGIC_1, HIGH_Z) -> UNDEFINED,
        (LOGIC_0, UNDEFINED) -> LOGIC_0,
        (LOGIC_1, UNDEFINED) -> UNDEFINED,
        (LOGIC_0, LOGIC_0) -> LOGIC_0,
        (LOGIC_0, LOGIC_1) -> LOGIC_0,
        (LOGIC_1, LOGIC_0) -> LOGIC_0,
        (LOGIC_1, LOGIC_1) -> LOGIC_1,
    );

    test_binary_gate(
        SimulatorBuilder::add_and_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_binary_gate(
        SimulatorBuilder::add_and_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_or_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z)       -> UNDEFINED,
        (HIGH_Z, UNDEFINED)    -> UNDEFINED,
        (UNDEFINED, HIGH_Z)    -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0)      -> UNDEFINED,
        (HIGH_Z, LOGIC_1)      -> LOGIC_1,
        (UNDEFINED, LOGIC_0)   -> UNDEFINED,
        (UNDEFINED, LOGIC_1)   -> LOGIC_1,
        (LOGIC_0, HIGH_Z)      -> UNDEFINED,
        (LOGIC_1, HIGH_Z)      -> LOGIC_1,
        (LOGIC_0, UNDEFINED)   -> UNDEFINED,
        (LOGIC_1, UNDEFINED)   -> LOGIC_1,
        (LOGIC_0, LOGIC_0)     -> LOGIC_0,
        (LOGIC_0, LOGIC_1)     -> LOGIC_1,
        (LOGIC_1, LOGIC_0)     -> LOGIC_1,
        (LOGIC_1, LOGIC_1)     -> LOGIC_1,
    );

    test_binary_gate(SimulatorBuilder::add_or_gate, LogicWidth::MIN, TEST_DATA, 2);
    test_binary_gate(SimulatorBuilder::add_or_gate, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_xor_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z)       -> UNDEFINED,
        (HIGH_Z, UNDEFINED)    -> UNDEFINED,
        (UNDEFINED, HIGH_Z)    -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0)      -> UNDEFINED,
        (HIGH_Z, LOGIC_1)      -> UNDEFINED,
        (UNDEFINED, LOGIC_0)   -> UNDEFINED,
        (UNDEFINED, LOGIC_1)   -> UNDEFINED,
        (LOGIC_0, HIGH_Z)      -> UNDEFINED,
        (LOGIC_1, HIGH_Z)      -> UNDEFINED,
        (LOGIC_0, UNDEFINED)   -> UNDEFINED,
        (LOGIC_1, UNDEFINED)   -> UNDEFINED,
        (LOGIC_0, LOGIC_0)     -> LOGIC_0,
        (LOGIC_0, LOGIC_1)     -> LOGIC_1,
        (LOGIC_1, LOGIC_0)     -> LOGIC_1,
        (LOGIC_1, LOGIC_1)     -> LOGIC_0,
    );

    test_binary_gate(
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_binary_gate(
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_nand_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> LOGIC_1,
        (HIGH_Z, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0) -> LOGIC_1,
        (UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0, HIGH_Z) -> LOGIC_1,
        (LOGIC_1, HIGH_Z) -> UNDEFINED,
        (LOGIC_0, UNDEFINED) -> LOGIC_1,
        (LOGIC_1, UNDEFINED) -> UNDEFINED,
        (LOGIC_0, LOGIC_0) -> LOGIC_1,
        (LOGIC_0, LOGIC_1) -> LOGIC_1,
        (LOGIC_1, LOGIC_0) -> LOGIC_1,
        (LOGIC_1, LOGIC_1) -> LOGIC_0,
    );

    test_binary_gate(
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_binary_gate(
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_nor_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> UNDEFINED,
        (HIGH_Z, LOGIC_1) -> LOGIC_0,
        (UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1) -> LOGIC_0,
        (LOGIC_0, HIGH_Z) -> UNDEFINED,
        (LOGIC_1, HIGH_Z) -> LOGIC_0,
        (LOGIC_0, UNDEFINED) -> UNDEFINED,
        (LOGIC_1, UNDEFINED) -> LOGIC_0,
        (LOGIC_0, LOGIC_0) -> LOGIC_1,
        (LOGIC_0, LOGIC_1) -> LOGIC_0,
        (LOGIC_1, LOGIC_0) -> LOGIC_0,
        (LOGIC_1, LOGIC_1) -> LOGIC_0,
    );

    test_binary_gate(
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_binary_gate(
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_xnor_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> UNDEFINED,
        (HIGH_Z, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0, HIGH_Z) -> UNDEFINED,
        (LOGIC_1, HIGH_Z) -> UNDEFINED,
        (LOGIC_0, UNDEFINED) -> UNDEFINED,
        (LOGIC_1, UNDEFINED) -> UNDEFINED,
        (LOGIC_0, LOGIC_0) -> LOGIC_1,
        (LOGIC_0, LOGIC_1) -> LOGIC_0,
        (LOGIC_1, LOGIC_0) -> LOGIC_0,
        (LOGIC_1, LOGIC_1) -> LOGIC_1,
    );

    test_binary_gate(
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_binary_gate(
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_not_gate() {
    const TEST_DATA: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,
    );

    test_unary_gate(
        SimulatorBuilder::add_not_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_unary_gate(
        SimulatorBuilder::add_not_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_buffer() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> HIGH_Z,
        (HIGH_Z, LOGIC_1) -> HIGH_Z,
        (UNDEFINED, LOGIC_0) -> HIGH_Z,
        (UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0, HIGH_Z) -> UNDEFINED,
        (LOGIC_1, HIGH_Z) -> UNDEFINED,
        (LOGIC_0, UNDEFINED) -> UNDEFINED,
        (LOGIC_1, UNDEFINED) -> UNDEFINED,
        (LOGIC_0, LOGIC_0) -> HIGH_Z,
        (LOGIC_0, LOGIC_1) -> LOGIC_0,
        (LOGIC_1, LOGIC_0) -> HIGH_Z,
        (LOGIC_1, LOGIC_1) -> LOGIC_1,
    );

    for width in [LogicWidth::MIN, LogicWidth::MAX] {
        let mut builder = SimulatorBuilder::default();

        let input = builder.add_wire(width);
        let enable = builder.add_wire(LogicWidth::MIN);
        let output = builder.add_wire(width);
        let _gate = builder.add_buffer(input, enable, output).unwrap();

        let mut sim = builder.build();

        for (i, test_data) in TEST_DATA.iter().enumerate() {
            sim.set_wire_base_drive(input, test_data.input_a);
            sim.set_wire_base_drive(enable, test_data.input_b);

            match sim.run_sim(2) {
                SimulationRunResult::Ok => {}
                SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
                SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
            }

            let output_state = sim.get_wire_state(output);

            assert!(
                output_state.eq_width(&test_data.output, width),
                "[TEST {i}]  expected: {}  actual: {}",
                test_data.output.display_string(width),
                output_state.display_string(width),
            );
        }
    }
}

#[test]
fn test_add() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0, 0) -> 0,
        (0, 1) -> 1,
        (1, 0) -> 1,
        (1, 1) -> 2,
        (0, {u32::MAX}) -> {u32::MAX},
        ({u32::MAX}, 0) -> {u32::MAX},
        (1, {u32::MAX}) -> 0,
        ({u32::MAX}, 1) -> 0,
        ({u32::MAX}, {u32::MAX}) -> {u32::MAX - 1},
    );

    test_binary_gate(SimulatorBuilder::add_add, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_sub() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0, 0) -> 0,
        (0, 1) -> {u32::MAX},
        (1, 0) -> 1,
        (1, 1) -> 0,
        (0, {u32::MAX}) -> 1,
        ({u32::MAX}, 0) -> {u32::MAX},
        ({u32::MAX}, {u32::MAX}) -> 0,
    );

    test_binary_gate(SimulatorBuilder::add_sub, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_mul() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0, 0) -> 0,
        (0, 1) -> 0,
        (1, 0) -> 0,
        (1, 1) -> 1,
        (0, {u32::MAX}) -> 0,
        ({u32::MAX}, 0) -> 0,
        (1, {u32::MAX}) -> {u32::MAX},
        ({u32::MAX}, 1) -> {u32::MAX},
        ({u32::MAX}, {u32::MAX}) -> 1,
    );

    test_binary_gate(SimulatorBuilder::add_mul, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_div() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0, 0) -> UNDEFINED,
        (0, 1) -> 0,
        (1, 0) -> UNDEFINED,
        (1, 1) -> 1,
        (0, {u32::MAX}) -> 0,
        ({u32::MAX}, 0) -> UNDEFINED,
        (1, {u32::MAX}) -> 0,
        ({u32::MAX}, 1) -> {u32::MAX},
        ({u32::MAX}, {u32::MAX}) -> 1,
    );

    test_binary_gate(SimulatorBuilder::add_div, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_rem() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0, 0) -> UNDEFINED,
        (0, 1) -> 0,
        (1, 0) -> UNDEFINED,
        (1, 1) -> 0,
        (0, {u32::MAX}) -> 0,
        ({u32::MAX}, 0) -> UNDEFINED,
        (1, {u32::MAX}) -> 1,
        ({u32::MAX}, 1) -> 0,
        ({u32::MAX}, {u32::MAX}) -> 0,
    );

    test_binary_gate(SimulatorBuilder::add_rem, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn test_wide_and_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> LOGIC_0,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> LOGIC_0,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> LOGIC_0,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> LOGIC_0,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> LOGIC_0,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> LOGIC_0,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> LOGIC_0,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> LOGIC_0,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_0,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_and_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_and_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_wide_or_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> LOGIC_1,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> LOGIC_1,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> LOGIC_1,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> LOGIC_1,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> LOGIC_1,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> LOGIC_1,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> LOGIC_1,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> LOGIC_1,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> LOGIC_1,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> LOGIC_1,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_1,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> LOGIC_1,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> LOGIC_1,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> LOGIC_1,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> LOGIC_1,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> LOGIC_1,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_or_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_or_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_wide_xor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_0,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_xor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_xor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_wide_nand_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> LOGIC_1,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> LOGIC_1,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> LOGIC_1,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> LOGIC_1,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> LOGIC_1,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> LOGIC_1,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> LOGIC_1,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> LOGIC_1,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_1,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_nand_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_nand_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_wide_nor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> LOGIC_0,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> LOGIC_0,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> LOGIC_0,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> LOGIC_0,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> LOGIC_0,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> LOGIC_0,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> LOGIC_0,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> LOGIC_0,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> LOGIC_0,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> LOGIC_0,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> LOGIC_0,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_0,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> LOGIC_0,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> LOGIC_0,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> LOGIC_0,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> LOGIC_0,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> LOGIC_0,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_nor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_nor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_wide_xnor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
        (HIGH_Z   , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , LOGIC_0  , HIGH_Z) -> UNDEFINED,
        (LOGIC_1  , LOGIC_1  , HIGH_Z) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_0  , LOGIC_1  , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , LOGIC_0  , UNDEFINED) -> UNDEFINED,
        (LOGIC_1  , LOGIC_1  , UNDEFINED) -> UNDEFINED,

        (HIGH_Z   , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_0) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_0) -> LOGIC_1,
        (LOGIC_0  , LOGIC_1  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_0  , LOGIC_0) -> LOGIC_0,
        (LOGIC_1  , LOGIC_1  , LOGIC_0) -> LOGIC_1,

        (HIGH_Z   , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (UNDEFINED, HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, UNDEFINED, LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_0  , LOGIC_1) -> UNDEFINED,
        (HIGH_Z   , LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_0  , LOGIC_1) -> UNDEFINED,
        (UNDEFINED, LOGIC_1  , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_1  , HIGH_Z   , LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_1  , UNDEFINED, LOGIC_1) -> UNDEFINED,
        (LOGIC_0  , LOGIC_0  , LOGIC_1) -> LOGIC_0,
        (LOGIC_0  , LOGIC_1  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_0  , LOGIC_1) -> LOGIC_1,
        (LOGIC_1  , LOGIC_1  , LOGIC_1) -> LOGIC_0,
    );

    test_wide_gate(
        SimulatorBuilder::add_wide_xnor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_wide_xnor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn test_register() {
    let mut builder = SimulatorBuilder::default();

    let data_in = builder.add_wire(LogicWidth::MAX);
    let data_out = builder.add_wire(LogicWidth::MAX);
    let enable = builder.add_wire(LogicWidth::MIN);
    let clock = builder.add_wire(LogicWidth::MIN);
    let register = builder
        .add_register(data_in, data_out, enable, clock)
        .unwrap();

    let mut sim = builder.build();

    struct TestData {
        data_in: LogicState,
        enable: bool,
        clock: bool,
        data_out: LogicState,
    }

    macro_rules! test_data {
        ($(($in:tt, $e:literal, $c:literal) -> $out:tt),* $(,)?) => {
            &[
                $(
                    TestData {
                        data_in: logic_state!($in),
                        enable: $e,
                        clock: $c,
                        data_out: logic_state!($out),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data![
        (HIGH_Z, false, false) -> UNDEFINED,
        (HIGH_Z, false, true) -> UNDEFINED,
        (HIGH_Z, true, false) -> UNDEFINED,
        (HIGH_Z, true, true) -> UNDEFINED,

        (0, false, false) -> UNDEFINED,
        (0, false, true) -> UNDEFINED,
        (0, true, false) -> UNDEFINED,
        (0, true, true) -> 0,

        (1, false, false) -> 0,
        (1, false, true) -> 0,
        (1, true, false) -> 0,
        (1, true, true) -> 1,

        (HIGH_Z, false, false) -> 1,
        (HIGH_Z, false, true) -> 1,
        (HIGH_Z, true, false) -> 1,
        (HIGH_Z, true, true) -> UNDEFINED,

        (0, false, true) -> UNDEFINED,
        (0, true, true) -> UNDEFINED,
        (0, true, false) -> UNDEFINED,
        (0, true, true) -> 0,

        (0, true, false) -> 0,
        (UNDEFINED, true, true) -> UNDEFINED,
        (UNDEFINED, true, false) -> UNDEFINED,
        (0xAA55, true, true) -> 0xAA55,
    ];

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        sim.set_wire_base_drive(data_in, test_data.data_in);
        sim.set_wire_base_drive(enable, LogicState::from_bool(test_data.enable));
        sim.set_wire_base_drive(clock, LogicState::from_bool(test_data.clock));

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data_out);

        assert!(
            output_state.eq_width(&test_data.data_out, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.data_out.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );

        let register_data = sim.get_component_data(register);
        let ComponentData::RegisterValue(register_data) = register_data else {
            panic!("[TEST {i}] invalid component data");
        };

        assert!(
            register_data.eq_width(&output_state, LogicWidth::MAX),
            "[TEST {i}] register data differs from output",
        );
    }
}
