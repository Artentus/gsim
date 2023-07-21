use super::*;

#[test]
fn and_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(
        SimulatorBuilder::add_and_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_and_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn or_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(SimulatorBuilder::add_or_gate, LogicWidth::MIN, TEST_DATA, 2);
    test_wide_gate(SimulatorBuilder::add_or_gate, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn xor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn nand_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn nor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn xnor_gate() {
    const TEST_DATA: &[WideGateTestData] = wide_gate_test_data!(
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

    test_wide_gate(
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn not_gate() {
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
fn buffer() {
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
                output_state.eq(test_data.output, width),
                "[TEST {i}]  expected: {}  actual: {}",
                test_data.output.display_string(width),
                output_state.display_string(width),
            );
        }
    }
}

#[test]
fn slice() {
    struct TestData {
        input: LogicState,
        offset: LogicOffset,
        output: LogicState,
    }

    macro_rules! test_data {
        ($(([$($i:tt),+], $offset:literal) -> [$($o:tt),+]),* $(,)?) => {
            &[
                $(
                    TestData {
                        input: bits!($($i),+),
                        offset: unsafe { LogicOffset::new_unchecked($offset) },
                        output: bits!($($o),+),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data!(
        ([Z, Z], 0) -> [Z],
        ([X, Z], 0) -> [Z],
        ([0, Z], 0) -> [Z],
        ([1, Z], 0) -> [Z],

        ([Z, X], 0) -> [X],
        ([X, X], 0) -> [X],
        ([0, X], 0) -> [X],
        ([1, X], 0) -> [X],

        ([Z, 0], 0) -> [0],
        ([X, 0], 0) -> [0],
        ([0, 0], 0) -> [0],
        ([1, 0], 0) -> [0],

        ([Z, 1], 0) -> [1],
        ([X, 1], 0) -> [1],
        ([0, 1], 0) -> [1],
        ([1, 1], 0) -> [1],

        ([Z, Z], 1) -> [Z],
        ([X, Z], 1) -> [X],
        ([0, Z], 1) -> [0],
        ([1, Z], 1) -> [1],

        ([Z, X], 1) -> [Z],
        ([X, X], 1) -> [X],
        ([0, X], 1) -> [0],
        ([1, X], 1) -> [1],

        ([Z, 0], 1) -> [Z],
        ([X, 0], 1) -> [X],
        ([0, 0], 1) -> [0],
        ([1, 0], 1) -> [1],

        ([Z, 1], 1) -> [Z],
        ([X, 1], 1) -> [X],
        ([0, 1], 1) -> [0],
        ([1, 1], 1) -> [1],
    );

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input = builder.add_wire(unsafe { LogicWidth::new_unchecked(2) });
        let output = builder.add_wire(LogicWidth::MIN);
        let _gate = builder.add_slice(input, test_data.offset, output).unwrap();

        let mut sim = builder.build();

        sim.set_wire_base_drive(input, test_data.input);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq(test_data.output, LogicWidth::MIN),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(LogicWidth::MIN),
            output_state.display_string(LogicWidth::MIN),
        );
    }
}

#[test]
fn merge() {
    macro_rules! test_data {
        ($(($([$($i:tt),+]),+) -> [$($o:tt),+]),* $(,)?) => {
            &[
                $(
                    WideGateTestData {
                        inputs: &[$(bits!($($i),+)),+],
                        output: bits!($($o),+),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[WideGateTestData] = test_data!(
        ([Z], [Z]) -> [Z, Z],
        ([Z], [X]) -> [X, Z],
        ([Z], [0]) -> [0, Z],
        ([Z], [1]) -> [1, Z],

        ([X], [Z]) -> [Z, X],
        ([X], [X]) -> [X, X],
        ([X], [0]) -> [0, X],
        ([X], [1]) -> [1, X],

        ([0], [Z]) -> [Z, 0],
        ([0], [X]) -> [X, 0],
        ([0], [0]) -> [0, 0],
        ([0], [1]) -> [1, 0],

        ([1], [Z]) -> [Z, 1],
        ([1], [X]) -> [X, 1],
        ([1], [0]) -> [0, 1],
        ([1], [1]) -> [1, 1],
    );

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|&drive| {
                let wire = builder.add_wire(LogicWidth::MIN);
                builder.set_wire_base_drive(wire, drive);
                wire
            })
            .collect();
        let output_width = LogicWidth::new(test_data.inputs.len() as u8).unwrap();
        let output = builder.add_wire(output_width);
        let _gate = builder.add_merge(&inputs, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq(test_data.output, output_width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(output_width),
            output_state.display_string(output_width),
        );
    }
}

#[test]
fn add() {
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
fn sub() {
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
fn mul() {
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
fn div() {
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
fn rem() {
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
fn left_shift() {
    const TEST_DATA_32: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (1, 0) -> 1,
        (1, 1) -> 2,
        (1, 2) -> 4,
        (1, 31) -> 0x80000000,

        (1, 32) -> 0,
        (1, 33) -> 0,
        (1, 63) -> 0,
        (1, 64) -> 0,

        (0x55, 0) -> 0x55,
        (0x55, 1) -> 0xAA,
    );

    test_binary_gate(
        SimulatorBuilder::add_left_shift,
        LogicWidth::MAX,
        TEST_DATA_32,
        2,
    );

    const TEST_DATA_16: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (1, 0) -> 1,
        (1, 1) -> 2,
        (1, 2) -> 4,
        (1, 15) -> 0x8000,

        (1, 16) -> 0,
        (1, 17) -> 0,
        (1, 31) -> 0,
        (1, 32) -> 0,

        (0x55, 0) -> 0x55,
        (0x55, 1) -> 0xAA,
    );

    test_binary_gate(
        SimulatorBuilder::add_left_shift,
        LogicWidth::new(16).unwrap(),
        TEST_DATA_16,
        2,
    );
}

#[test]
fn logical_right_shift() {
    const TEST_DATA_32: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0x80000000, 0) -> 0x80000000,
        (0x80000000, 1) -> 0x40000000,
        (0x80000000, 2) -> 0x20000000,
        (0x80000000, 31) -> 1,

        (0x80000000, 32) -> 0,
        (0x80000000, 33) -> 0,
        (0x80000000, 63) -> 0,
        (0x80000000, 64) -> 0,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_binary_gate(
        SimulatorBuilder::add_logical_right_shift,
        LogicWidth::MAX,
        TEST_DATA_32,
        2,
    );

    const TEST_DATA_16: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0x8000, 0) -> 0x8000,
        (0x8000, 1) -> 0x4000,
        (0x8000, 2) -> 0x2000,
        (0x8000, 15) -> 1,

        (0x8000, 16) -> 0,
        (0x8000, 17) -> 0,
        (0x8000, 31) -> 0,
        (0x8000, 32) -> 0,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_binary_gate(
        SimulatorBuilder::add_logical_right_shift,
        LogicWidth::new(16).unwrap(),
        TEST_DATA_16,
        2,
    );
}

#[test]
fn arithmetic_right_shift() {
    const TEST_DATA_32: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0x80000000, 0) -> 0x80000000,
        (0x80000000, 1) -> 0xC0000000,
        (0x80000000, 2) -> 0xE0000000,
        (0x80000000, 31) -> 0xFFFFFFFF,

        (0x80000000, 32) -> 0,
        (0x80000000, 33) -> 0,
        (0x80000000, 63) -> 0,
        (0x80000000, 64) -> 0,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_binary_gate(
        SimulatorBuilder::add_arithmetic_right_shift,
        LogicWidth::MAX,
        TEST_DATA_32,
        2,
    );

    const TEST_DATA_16: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, 0) -> UNDEFINED,
        (UNDEFINED, 0) -> UNDEFINED,
        (0, HIGH_Z) -> UNDEFINED,
        (0, UNDEFINED) -> UNDEFINED,

        (0x8000, 0) -> 0x8000,
        (0x8000, 1) -> 0xC000,
        (0x8000, 2) -> 0xE000,
        (0x8000, 15) -> 0xFFFF,

        (0x8000, 16) -> 0,
        (0x8000, 17) -> 0,
        (0x8000, 31) -> 0,
        (0x8000, 32) -> 0,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_binary_gate(
        SimulatorBuilder::add_arithmetic_right_shift,
        LogicWidth::new(16).unwrap(),
        TEST_DATA_16,
        2,
    );
}

#[test]
fn wide_and_gate() {
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
        SimulatorBuilder::add_and_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_and_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn wide_or_gate() {
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

    test_wide_gate(SimulatorBuilder::add_or_gate, LogicWidth::MIN, TEST_DATA, 2);
    test_wide_gate(SimulatorBuilder::add_or_gate, LogicWidth::MAX, TEST_DATA, 2);
}

#[test]
fn wide_xor_gate() {
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
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_xor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn wide_nand_gate() {
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
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_nand_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn wide_nor_gate() {
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
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_nor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn wide_xnor_gate() {
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
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MIN,
        TEST_DATA,
        2,
    );
    test_wide_gate(
        SimulatorBuilder::add_xnor_gate,
        LogicWidth::MAX,
        TEST_DATA,
        2,
    );
}

#[test]
fn adder() {
    struct TestData {
        input_a: LogicState,
        input_b: LogicState,
        carry_in: LogicState,
        output: LogicState,
        carry_out: LogicState,
    }

    macro_rules! test_data {
        ($(($a:tt, $b:tt, $ci:tt) -> ($o:tt, $co:tt)),* $(,)?) => {
            &[
                $(
                    TestData {
                        input_a: logic_state!($a),
                        input_b: logic_state!($b),
                        carry_in: logic_state!($ci),
                        output: logic_state!($o),
                        carry_out: logic_state!($co),
                    },
                )*
            ]
        };
    }

    const TEST_DATA_1: &[TestData] = test_data!(
        (HIGH_Z, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, 0) -> (UNDEFINED, UNDEFINED),

        (UNDEFINED, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, 0) -> (UNDEFINED, UNDEFINED),

        (0, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (0, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, 0, 0) -> (0, 0),

        (1, 0, 0) -> (1, 0),
        (0, 1, 0) -> (1, 0),
        (0, 0, 1) -> (1, 0),
        (1, 1, 0) -> (2, 0),
        (1, 0, 1) -> (2, 0),
        (0, 1, 1) -> (2, 0),
        (1, 1, 1) -> (3, 0),

        (0xFFFFFFFF, 1, 0) -> (0, 1),
        (1, 0xFFFFFFFF, 0) -> (0, 1),
        (0xFFFFFFFF, 0, 1) -> (0, 1),
        (0, 0xFFFFFFFF, 1) -> (0, 1),
        (0xFFFFFFFF, 1, 1) -> (1, 1),
        (1, 0xFFFFFFFF, 1) -> (1, 1),

        (0xFFFFFFFF, 0xFFFFFFFF, 0) -> (0xFFFFFFFE, 1),
        (0xFFFFFFFF, 0xFFFFFFFF, 0) -> (0xFFFFFFFE, 1),
        (0xFFFFFFFF, 0xFFFFFFFF, 1) -> (0xFFFFFFFF, 1),
        (0xFFFFFFFF, 0xFFFFFFFF, 1) -> (0xFFFFFFFF, 1),
    );

    let mut builder = SimulatorBuilder::default();

    let input_a = builder.add_wire(LogicWidth::MAX);
    let input_b = builder.add_wire(LogicWidth::MAX);
    let carry_in = builder.add_wire(LogicWidth::MIN);
    let output = builder.add_wire(LogicWidth::MAX);
    let carry_out = builder.add_wire(LogicWidth::MIN);
    let _adder = builder
        .add_adder(input_a, input_b, carry_in, output, carry_out)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_1.iter().enumerate() {
        sim.set_wire_base_drive(input_a, test_data.input_a);
        sim.set_wire_base_drive(input_b, test_data.input_b);
        sim.set_wire_base_drive(carry_in, test_data.carry_in);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);
        let carry_out_state = sim.get_wire_state(carry_out);

        assert!(
            output_state.eq(test_data.output, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );

        assert!(
            carry_out_state.eq(test_data.carry_out, LogicWidth::MIN),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.carry_out.display_string(LogicWidth::MIN),
            carry_out_state.display_string(LogicWidth::MIN),
        );
    }

    const TEST_DATA_2: &[TestData] = test_data!(
        (HIGH_Z, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0, 0) -> (UNDEFINED, UNDEFINED),

        (UNDEFINED, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0, 0) -> (UNDEFINED, UNDEFINED),

        (0, HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (0, 0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, 0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, 0, 0) -> (0, 0),

        (1, 0, 0) -> (1, 0),
        (0, 1, 0) -> (1, 0),
        (0, 0, 1) -> (1, 0),
        (1, 1, 0) -> (2, 0),
        (1, 0, 1) -> (2, 0),
        (0, 1, 1) -> (2, 0),
        (1, 1, 1) -> (3, 0),

        (0xFFFF, 1, 0) -> (0, 1),
        (1, 0xFFFF, 0) -> (0, 1),
        (0xFFFF, 0, 1) -> (0, 1),
        (0, 0xFFFF, 1) -> (0, 1),
        (0xFFFF, 1, 1) -> (1, 1),
        (1, 0xFFFF, 1) -> (1, 1),

        (0xFFFF, 0xFFFF, 0) -> (0xFFFE, 1),
        (0xFFFF, 0xFFFF, 0) -> (0xFFFE, 1),
        (0xFFFF, 0xFFFF, 1) -> (0xFFFF, 1),
        (0xFFFF, 0xFFFF, 1) -> (0xFFFF, 1),
    );

    let mut builder = SimulatorBuilder::default();

    const WIDTH: LogicWidth = unsafe { LogicWidth::new_unchecked(16) };
    let input_a = builder.add_wire(WIDTH);
    let input_b = builder.add_wire(WIDTH);
    let carry_in = builder.add_wire(LogicWidth::MIN);
    let output = builder.add_wire(WIDTH);
    let carry_out = builder.add_wire(LogicWidth::MIN);
    let _adder = builder
        .add_adder(input_a, input_b, carry_in, output, carry_out)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_2.iter().enumerate() {
        sim.set_wire_base_drive(input_a, test_data.input_a);
        sim.set_wire_base_drive(input_b, test_data.input_b);
        sim.set_wire_base_drive(carry_in, test_data.carry_in);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);
        let carry_out_state = sim.get_wire_state(carry_out);

        assert!(
            output_state.eq(test_data.output, WIDTH),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH),
            output_state.display_string(WIDTH),
        );

        assert!(
            carry_out_state.eq(test_data.carry_out, LogicWidth::MIN),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.carry_out.display_string(LogicWidth::MIN),
            carry_out_state.display_string(LogicWidth::MIN),
        );
    }
}

#[test]
fn multiplier() {
    struct TestData {
        input_a: LogicState,
        input_b: LogicState,
        output_low: LogicState,
        output_high: LogicState,
    }

    macro_rules! test_data {
        ($(($a:tt, $b:tt) -> ($l:tt, $h:tt)),* $(,)?) => {
            &[
                $(
                    TestData {
                        input_a: logic_state!($a),
                        input_b: logic_state!($b),
                        output_low: logic_state!($l),
                        output_high: logic_state!($h),
                    },
                )*
            ]
        };
    }

    const TEST_DATA_1: &[TestData] = test_data!(
        (HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, 0) -> (0, 0),

        (1, 0) -> (0, 0),
        (0, 1) -> (0, 0),
        (1, 1) -> (1, 0),

        (0xFFFFFFFF, 0) -> (0, 0),
        (0, 0xFFFFFFFF) -> (0, 0),
        (0xFFFFFFFF, 1) -> (0xFFFFFFFF, 0),
        (1, 0xFFFFFFFF) -> (0xFFFFFFFF, 0),

        (0xFFFFFFFF, 2) -> (0xFFFFFFFE, 1),
        (0xFFFFFFFF, 0xFFFFFFFF) -> (1, 0xFFFFFFFE),
    );

    let mut builder = SimulatorBuilder::default();

    let input_a = builder.add_wire(LogicWidth::MAX);
    let input_b = builder.add_wire(LogicWidth::MAX);
    let output_low = builder.add_wire(LogicWidth::MAX);
    let output_high = builder.add_wire(LogicWidth::MAX);
    let _adder = builder
        .add_multiplier(input_a, input_b, output_low, output_high)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_1.iter().enumerate() {
        sim.set_wire_base_drive(input_a, test_data.input_a);
        sim.set_wire_base_drive(input_b, test_data.input_b);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_low_state = sim.get_wire_state(output_low);
        let output_high_state = sim.get_wire_state(output_high);

        assert!(
            output_low_state.eq(test_data.output_low, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output_low.display_string(LogicWidth::MAX),
            output_low_state.display_string(LogicWidth::MAX),
        );

        assert!(
            output_high_state.eq(test_data.output_high, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output_high.display_string(LogicWidth::MAX),
            output_high_state.display_string(LogicWidth::MAX),
        );
    }

    const TEST_DATA_2: &[TestData] = test_data!(
        (HIGH_Z, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (HIGH_Z, 0) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (UNDEFINED, 0) -> (UNDEFINED, UNDEFINED),
        (0, HIGH_Z) -> (UNDEFINED, UNDEFINED),
        (0, UNDEFINED) -> (UNDEFINED, UNDEFINED),
        (0, 0) -> (0, 0),

        (1, 0) -> (0, 0),
        (0, 1) -> (0, 0),
        (1, 1) -> (1, 0),

        (0xFFFF, 0) -> (0, 0),
        (0, 0xFFFF) -> (0, 0),
        (0xFFFF, 1) -> (0xFFFF, 0),
        (1, 0xFFFF) -> (0xFFFF, 0),

        (0xFFFF, 2) -> (0xFFFE, 1),
        (0xFFFF, 0xFFFF) -> (1, 0xFFFE),
    );

    let mut builder = SimulatorBuilder::default();

    const WIDTH: LogicWidth = unsafe { LogicWidth::new_unchecked(16) };
    let input_a = builder.add_wire(WIDTH);
    let input_b = builder.add_wire(WIDTH);
    let output_low = builder.add_wire(WIDTH);
    let output_high = builder.add_wire(WIDTH);
    let _adder = builder
        .add_multiplier(input_a, input_b, output_low, output_high)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_2.iter().enumerate() {
        sim.set_wire_base_drive(input_a, test_data.input_a);
        sim.set_wire_base_drive(input_b, test_data.input_b);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_low_state = sim.get_wire_state(output_low);
        let output_high_state = sim.get_wire_state(output_high);

        assert!(
            output_low_state.eq(test_data.output_low, WIDTH),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output_low.display_string(WIDTH),
            output_low_state.display_string(WIDTH),
        );

        assert!(
            output_high_state.eq(test_data.output_high, WIDTH),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output_high.display_string(WIDTH),
            output_high_state.display_string(WIDTH),
        );
    }
}

#[test]
fn multiplexer() {
    struct TestData {
        inputs: &'static [LogicState],
        select: LogicState,
        output: LogicState,
    }

    macro_rules! test_data {
        ($(([$($i:tt),+ $(,)?], $s:tt) -> $o:tt),* $(,)?) => {
            &[
                $(
                    TestData {
                        inputs: &[$(logic_state!($i)),+],
                        select: logic_state!($s),
                        output: logic_state!($o),
                    },
                )*
            ]
        };
    }

    const TEST_DATA_1: &[TestData] = test_data!(
        ([HIGH_Z, HIGH_Z], HIGH_Z) -> UNDEFINED,
        ([HIGH_Z, HIGH_Z], UNDEFINED) -> UNDEFINED,
        ([HIGH_Z, UNDEFINED], HIGH_Z) -> UNDEFINED,
        ([HIGH_Z, UNDEFINED], UNDEFINED) -> UNDEFINED,
        ([UNDEFINED, HIGH_Z], HIGH_Z) -> UNDEFINED,
        ([UNDEFINED, HIGH_Z], UNDEFINED) -> UNDEFINED,
        ([UNDEFINED, UNDEFINED], HIGH_Z) -> UNDEFINED,
        ([UNDEFINED, UNDEFINED], UNDEFINED) -> UNDEFINED,

        ([HIGH_Z, HIGH_Z], 0) -> UNDEFINED,
        ([HIGH_Z, HIGH_Z], 1) -> UNDEFINED,

        ([HIGH_Z, UNDEFINED], 0) -> UNDEFINED,
        ([HIGH_Z, UNDEFINED], 1) -> UNDEFINED,

        ([UNDEFINED, HIGH_Z], 0) -> UNDEFINED,
        ([UNDEFINED, HIGH_Z], 1) -> UNDEFINED,

        ([UNDEFINED, UNDEFINED], 0) -> UNDEFINED,
        ([UNDEFINED, UNDEFINED], 1) -> UNDEFINED,

        ([0x55, 0xAA], 0) -> 0x55,
        ([0x55, 0xAA], 1) -> 0xAA,

        ([1, 2, 3, 4], 0) -> 1,
        ([1, 2, 3, 4], 1) -> 2,
        ([1, 2, 3, 4], 2) -> 3,
        ([1, 2, 3, 4], 3) -> 4,
    );

    for (i, test_data) in TEST_DATA_1.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|&drive| {
                let wire = builder.add_wire(LogicWidth::MAX);
                builder.set_wire_base_drive(wire, drive);
                wire
            })
            .collect();
        let select = builder.add_wire(LogicWidth::new(inputs.len().ilog2() as u8).unwrap());
        builder.set_wire_base_drive(select, test_data.select);
        let output = builder.add_wire(LogicWidth::MAX);
        let _mux = builder.add_multiplexer(&inputs, select, output).unwrap();

        let mut sim = builder.build();
        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output);

        assert!(
            output_state.eq(test_data.output, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );
    }
}

#[test]
fn register() {
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
            output_state.eq(test_data.data_out, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.data_out.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );

        let register_data = sim.get_component_data(register);
        let ComponentData::RegisterValue(register_data) = register_data else {
            panic!("[TEST {i}] invalid component data");
        };

        assert!(
            register_data.eq(output_state, LogicWidth::MAX),
            "[TEST {i}] register data differs from output",
        );
    }
}

#[test]
fn horizontal_and_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_0,
        LOGIC_1 -> LOGIC_1,

        0b1111 -> 1,

        0b1110 -> 0,
        0b1101 -> 0,
        0b1011 -> 0,
        0b0111 -> 0,

        0b1100 -> 0,
        0b1010 -> 0,
        0b0110 -> 0,

        0b1000 -> 0,
        0b0100 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_and_gate,
        LogicWidth::new(4).unwrap(),
        TEST_DATA_EVEN,
        2,
    );

    const TEST_DATA_ODD: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_0,
        LOGIC_1 -> LOGIC_1,

        0b11111 -> 1,

        0b11110 -> 0,
        0b11101 -> 0,
        0b11011 -> 0,
        0b10111 -> 0,
        0b01111 -> 0,

        0b11100 -> 0,
        0b11010 -> 0,
        0b10110 -> 0,
        0b01110 -> 0,

        0b11000 -> 0,
        0b10100 -> 0,
        0b01100 -> 0,

        0b10000 -> 0,
        0b01000 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_and_gate,
        LogicWidth::new(5).unwrap(),
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn horizontal_or_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_0,
        LOGIC_1 -> LOGIC_1,

        0b0000 -> 0,

        0b0001 -> 1,
        0b0010 -> 1,
        0b0100 -> 1,
        0b1000 -> 1,

        0b0011 -> 1,
        0b0101 -> 1,
        0b1001 -> 1,

        0b0111 -> 1,
        0b1011 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_or_gate,
        LogicWidth::new(4).unwrap(),
        TEST_DATA_EVEN,
        2,
    );

    const TEST_DATA_ODD: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_0,
        LOGIC_1 -> LOGIC_1,

        0b00000 -> 0,

        0b00001 -> 1,
        0b00010 -> 1,
        0b00100 -> 1,
        0b01000 -> 1,
        0b10000 -> 1,

        0b00011 -> 1,
        0b00101 -> 1,
        0b01001 -> 1,
        0b10001 -> 1,

        0b00111 -> 1,
        0b01011 -> 1,
        0b10011 -> 1,

        0b01111 -> 1,
        0b10111 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_or_gate,
        LogicWidth::new(5).unwrap(),
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn horizontal_nand_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,

        0b1111 -> 0,

        0b1110 -> 1,
        0b1101 -> 1,
        0b1011 -> 1,
        0b0111 -> 1,

        0b1100 -> 1,
        0b1010 -> 1,
        0b0110 -> 1,

        0b1000 -> 1,
        0b0100 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_nand_gate,
        LogicWidth::new(4).unwrap(),
        TEST_DATA_EVEN,
        2,
    );

    const TEST_DATA_ODD: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,

        0b11111 -> 0,

        0b11110 -> 1,
        0b11101 -> 1,
        0b11011 -> 1,
        0b10111 -> 1,
        0b01111 -> 1,

        0b11100 -> 1,
        0b11010 -> 1,
        0b10110 -> 1,
        0b01110 -> 1,

        0b11000 -> 1,
        0b10100 -> 1,
        0b01100 -> 1,

        0b10000 -> 1,
        0b01000 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_nand_gate,
        LogicWidth::new(5).unwrap(),
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn horizontal_nor_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,

        0b0000 -> 1,

        0b0001 -> 0,
        0b0010 -> 0,
        0b0100 -> 0,
        0b1000 -> 0,

        0b0011 -> 0,
        0b0101 -> 0,
        0b1001 -> 0,

        0b0111 -> 0,
        0b1011 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_nor_gate,
        LogicWidth::new(4).unwrap(),
        TEST_DATA_EVEN,
        2,
    );

    const TEST_DATA_ODD: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,

        0b00000 -> 1,

        0b00001 -> 0,
        0b00010 -> 0,
        0b00100 -> 0,
        0b01000 -> 0,
        0b10000 -> 0,

        0b00011 -> 0,
        0b00101 -> 0,
        0b01001 -> 0,
        0b10001 -> 0,

        0b00111 -> 0,
        0b01011 -> 0,
        0b10011 -> 0,

        0b01111 -> 0,
        0b10111 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_nor_gate,
        LogicWidth::new(5).unwrap(),
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn comparator() {
    test_comparator(SimulatorBuilder::add_compare_equal, |a, b| a == b);
    test_comparator(SimulatorBuilder::add_compare_not_equal, |a, b| a != b);

    test_comparator(SimulatorBuilder::add_compare_less_than, |a, b| a < b);
    test_comparator(SimulatorBuilder::add_compare_greater_than, |a, b| a > b);
    test_comparator(SimulatorBuilder::add_compare_less_than_or_equal, |a, b| {
        a <= b
    });
    test_comparator(
        SimulatorBuilder::add_compare_greater_than_or_equal,
        |a, b| a >= b,
    );

    test_signed_comparator(SimulatorBuilder::add_compare_less_than_signed, |a, b| a < b);
    test_signed_comparator(SimulatorBuilder::add_compare_greater_than_signed, |a, b| {
        a > b
    });
    test_signed_comparator(
        SimulatorBuilder::add_compare_less_than_or_equal_signed,
        |a, b| a <= b,
    );
    test_signed_comparator(
        SimulatorBuilder::add_compare_greater_than_or_equal_signed,
        |a, b| a >= b,
    );
}

#[test]
fn ram() {
    let mut builder = SimulatorBuilder::default();

    const ADDR_WIDTH: LogicWidth = unsafe { LogicWidth::new_unchecked(2) };
    let write_addr = builder.add_wire(ADDR_WIDTH);
    let data_in = builder.add_wire(LogicWidth::MAX);
    let read_addr = builder.add_wire(ADDR_WIDTH);
    let data_out = builder.add_wire(LogicWidth::MAX);
    let write = builder.add_wire(LogicWidth::MIN);
    let clock = builder.add_wire(LogicWidth::MIN);
    let ram = builder
        .add_ram(write_addr, data_in, read_addr, data_out, write, clock)
        .unwrap();

    let mut sim = builder.build();

    struct TestData {
        write_addr: LogicSizeInteger,
        data_in: LogicState,
        read_addr: LogicSizeInteger,
        write: bool,
        clock: bool,
        data_out: LogicState,
    }

    macro_rules! test_data {
        ($(($aw:literal, $in:tt, $ar:literal, $w:literal, $c:literal) -> $out:tt),* $(,)?) => {
            &[
                $(
                    TestData {
                        write_addr: $aw,
                        data_in: logic_state!($in),
                        read_addr: $ar,
                        write: $w,
                        clock: $c,
                        data_out: logic_state!($out),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data![
        (0, HIGH_Z, 0, false, false) -> UNDEFINED,
        (0, HIGH_Z, 0, false, true) -> UNDEFINED,
        (0, HIGH_Z, 0, true, false) -> UNDEFINED,
        (0, HIGH_Z, 0, true, true) -> UNDEFINED,

        (0, 0, 0, false, false) -> UNDEFINED,
        (0, 0, 0, false, true) -> UNDEFINED,
        (0, 0, 0, true, false) -> UNDEFINED,
        (0, 0, 0, true, true) -> 0,

        (0, 1, 0, false, false) -> 0,
        (0, 1, 0, false, true) -> 0,
        (0, 1, 0, true, false) -> 0,
        (0, 1, 0, true, true) -> 1,

        (0, HIGH_Z, 0, false, false) -> 1,
        (0, HIGH_Z, 0, false, true) -> 1,
        (0, HIGH_Z, 0, true, false) -> 1,
        (0, HIGH_Z, 0, true, true) -> UNDEFINED,

        (0, 0, 0, false, true) -> UNDEFINED,
        (0, 0, 0, true, true) -> UNDEFINED,
        (0, 0, 0, true, false) -> UNDEFINED,
        (0, 0, 0, true, true) -> 0,

        (0, 0, 0, true, false) -> 0,
        (0, UNDEFINED, 0, true, true) -> UNDEFINED,
        (0, UNDEFINED, 0, true, false) -> UNDEFINED,
        (0, 0xAA55, 0, true, true) -> 0xAA55,

        (0, 0, 1, false, false) -> UNDEFINED,
        (0, 0, 2, false, false) -> UNDEFINED,
        (0, 0, 3, false, false) -> UNDEFINED,

        (0, 0, 0, true, true) -> 0,
        (0, 0, 0, true, false) -> 0,
        (1, 1, 1, true, true) -> 1,
        (1, 1, 1, true, false) -> 1,
        (2, 2, 2, true, true) -> 2,
        (2, 2, 2, true, false) -> 2,
        (3, 3, 3, true, true) -> 3,
        (3, 3, 3, true, false) -> 3,

        (0, 0, 0, false, false) -> 0,
        (0, 0, 1, false, false) -> 1,
        (0, 0, 2, false, false) -> 2,
        (0, 0, 3, false, false) -> 3,
    ];

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        sim.set_wire_base_drive(write_addr, LogicState::from_int(test_data.write_addr));
        sim.set_wire_base_drive(data_in, test_data.data_in);
        sim.set_wire_base_drive(read_addr, LogicState::from_int(test_data.read_addr));
        sim.set_wire_base_drive(write, LogicState::from_bool(test_data.write));
        sim.set_wire_base_drive(clock, LogicState::from_bool(test_data.clock));

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data_out);

        assert!(
            output_state.eq(test_data.data_out, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.data_out.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );

        let mem_data = sim.get_component_data(ram);
        let ComponentData::MemoryBlock(mem_data) = mem_data else {
            panic!("[TEST {i}] invalid component data");
        };

        assert!(
            mem_data
                .read(test_data.read_addr as usize)
                .eq(output_state, LogicWidth::MAX),
            "[TEST {i}] memory data differs from output",
        );
    }
}

#[test]
fn rom() {
    let mut builder = SimulatorBuilder::default();

    const ADDR_WIDTH: LogicWidth = unsafe { LogicWidth::new_unchecked(2) };
    let addr = builder.add_wire(ADDR_WIDTH);
    let data = builder.add_wire(LogicWidth::MAX);
    let rom = builder.add_rom(addr, data).unwrap();

    let mem_data = builder.get_component_data(rom);
    let ComponentData::MemoryBlock(mut mem_data) = mem_data else {
        panic!("[TEST] invalid component data");
    };

    mem_data.write(0, LogicState::from_int(1));
    mem_data.write(1, LogicState::from_int(2));
    mem_data.write(2, LogicState::from_int(3));
    mem_data.write(3, LogicState::from_int(4));

    let mut sim = builder.build();

    const TEST_DATA: &[UnaryGateTestData] = unary_gate_test_data![
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,

        0 -> 1,
        1 -> 2,
        2 -> 3,
        3 -> 4,
    ];

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        sim.set_wire_base_drive(addr, test_data.input);

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data);

        assert!(
            output_state.eq(test_data.output, LogicWidth::MAX),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(LogicWidth::MAX),
            output_state.display_string(LogicWidth::MAX),
        );
    }
}
