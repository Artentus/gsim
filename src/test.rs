use crate::*;

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
    ($(($a:ident, $b:ident) -> $o:ident),* $(,)?) => {
        &[
            $(
                BinaryGateTestData {
                    input_a: LogicState::$a,
                    input_b: LogicState::$b,
                    output: LogicState::$o,
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
    ($($i:ident -> $o:ident),* $(,)?) => {
        &[
            $(
                UnaryGateTestData {
                    input: LogicState::$i,
                    output: LogicState::$o,
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
    ($(($($i:ident),+) -> $o:ident),* $(,)?) => {
        &[
            $(
                WideGateTestData {
                    inputs: &[$(LogicState::$i),+],
                    output: LogicState::$o,
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
