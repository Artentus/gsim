use crate::*;

struct BinaryGateTestData {
    input_a: LogicState,
    input_b: LogicState,
    output: LogicState,
}

fn test_binary_gate<F>(add_gate: F, width: LogicWidth, test_data: &[BinaryGateTestData])
where
    F: FnOnce(&mut Simulator, WireId, WireId, WireId) -> AddComponentResult,
{
    let mut simulator = Simulator::default();

    let input_a = simulator.add_wire(width);
    let input_b = simulator.add_wire(width);
    let output = simulator.add_wire(width);
    let _gate = add_gate(&mut simulator, input_a, input_b, output).unwrap();

    for (i, test_data) in test_data.iter().enumerate() {
        simulator.set_wire_base_drive(input_a, test_data.input_a);
        simulator.set_wire_base_drive(input_b, test_data.input_b);

        let mut sim_result = simulator.begin_sim();
        loop {
            match sim_result {
                SimulationStepResult::Unchanged => break,
                SimulationStepResult::Changed => sim_result = simulator.step_sim(),
                SimulationStepResult::Err(err) => panic!("{err:?}"),
            }
        }

        let output_state = simulator.get_wire_state(output);

        println!(
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(width),
            output_state.display_string(width),
        );

        assert!(output_state.eq_width(&test_data.output, width));
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

fn test_unary_gate<F>(add_gate: F, width: LogicWidth, test_data: &[UnaryGateTestData])
where
    F: FnOnce(&mut Simulator, WireId, WireId) -> AddComponentResult,
{
    let mut simulator = Simulator::default();

    let input = simulator.add_wire(width);
    let output = simulator.add_wire(width);
    let _gate = add_gate(&mut simulator, input, output).unwrap();

    for (i, test_data) in test_data.iter().enumerate() {
        simulator.set_wire_base_drive(input, test_data.input);

        let mut sim_result = simulator.begin_sim();
        loop {
            match sim_result {
                SimulationStepResult::Unchanged => break,
                SimulationStepResult::Changed => sim_result = simulator.step_sim(),
                SimulationStepResult::Err(err) => panic!("{err:?}"),
            }
        }

        let output_state = simulator.get_wire_state(output);

        println!(
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(width),
            output_state.display_string(width),
        );

        assert!(output_state.eq_width(&test_data.output, width));
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

    test_binary_gate(Simulator::add_and_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_and_gate, LogicWidth::MAX, TEST_DATA);
}

#[test]
fn test_or_gate() {
    const TEST_DATA: &[BinaryGateTestData] = binary_gate_test_data!(
        (HIGH_Z, HIGH_Z) -> UNDEFINED,
        (HIGH_Z, UNDEFINED) -> UNDEFINED,
        (UNDEFINED, HIGH_Z) -> UNDEFINED,
        (UNDEFINED, UNDEFINED) -> UNDEFINED,
        (HIGH_Z, LOGIC_0) -> UNDEFINED,
        (HIGH_Z, LOGIC_1) -> LOGIC_1,
        (UNDEFINED, LOGIC_0) -> UNDEFINED,
        (UNDEFINED, LOGIC_1) -> LOGIC_1,
        (LOGIC_0, HIGH_Z) -> UNDEFINED,
        (LOGIC_1, HIGH_Z) -> LOGIC_1,
        (LOGIC_0, UNDEFINED) -> UNDEFINED,
        (LOGIC_1, UNDEFINED) -> LOGIC_1,
        (LOGIC_0, LOGIC_0) -> LOGIC_0,
        (LOGIC_0, LOGIC_1) -> LOGIC_1,
        (LOGIC_1, LOGIC_0) -> LOGIC_1,
        (LOGIC_1, LOGIC_1) -> LOGIC_1,
    );

    test_binary_gate(Simulator::add_or_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_or_gate, LogicWidth::MAX, TEST_DATA);
}

#[test]
fn test_xor_gate() {
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
        (LOGIC_0, LOGIC_0) -> LOGIC_0,
        (LOGIC_0, LOGIC_1) -> LOGIC_1,
        (LOGIC_1, LOGIC_0) -> LOGIC_1,
        (LOGIC_1, LOGIC_1) -> LOGIC_0,
    );

    test_binary_gate(Simulator::add_xor_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_xor_gate, LogicWidth::MAX, TEST_DATA);
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

    test_binary_gate(Simulator::add_nand_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_nand_gate, LogicWidth::MAX, TEST_DATA);
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

    test_binary_gate(Simulator::add_nor_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_nor_gate, LogicWidth::MAX, TEST_DATA);
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

    test_binary_gate(Simulator::add_xnor_gate, LogicWidth::MIN, TEST_DATA);
    test_binary_gate(Simulator::add_xnor_gate, LogicWidth::MAX, TEST_DATA);
}

#[test]
fn test_not_gate() {
    const TEST_DATA: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_0,
    );

    test_unary_gate(Simulator::add_not_gate, LogicWidth::MIN, TEST_DATA);
    test_unary_gate(Simulator::add_not_gate, LogicWidth::MAX, TEST_DATA);
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
        let mut simulator = Simulator::default();

        let input = simulator.add_wire(width);
        let enable = simulator.add_wire(LogicWidth::MIN);
        let output = simulator.add_wire(width);
        let _gate = simulator.add_buffer(input, enable, output).unwrap();

        for (i, test_data) in TEST_DATA.iter().enumerate() {
            simulator.set_wire_base_drive(input, test_data.input_a);
            simulator.set_wire_base_drive(enable, test_data.input_b);

            let mut sim_result = simulator.begin_sim();
            loop {
                match sim_result {
                    SimulationStepResult::Unchanged => break,
                    SimulationStepResult::Changed => sim_result = simulator.step_sim(),
                    SimulationStepResult::Err(err) => panic!("{err:?}"),
                }
            }

            let output_state = simulator.get_wire_state(output);

            println!(
                "[TEST {i}]  expected: {}  actual: {}",
                test_data.output.display_string(width),
                output_state.display_string(width),
            );

            assert!(output_state.eq_width(&test_data.output, width));
        }
    }
}
