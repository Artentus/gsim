use super::super::*;
use crate::import::yosys::*;
use crate::import::*;

#[cfg(test)]
fn test_yosys_import(
    json: &str,
    expected_inputs: &[(&str, NonZeroU8)],
    expected_outputs: &[(&str, NonZeroU8)],
) -> (ModuleConnections, Simulator) {
    let importer = YosysModuleImporter::from_json_str(json).unwrap();
    let mut builder = SimulatorBuilder::default();
    let connections = builder.import_module(&importer).unwrap();

    for &(port_name, port_width) in expected_inputs {
        let wire = *connections
            .inputs
            .get(port_name)
            .expect(&format!("expected input port `{port_name}` to be present"));
        let wire_width = builder.get_wire_width(wire);
        assert_eq!(wire_width, port_width, "input port `{port_name}` has incorrect width;  expected: {port_width}  actual: {wire_width}");
    }

    for &(port_name, port_width) in expected_outputs {
        let wire = *connections
            .outputs
            .get(port_name)
            .expect(&format!("expected output port `{port_name}` to be present"));
        let wire_width = builder.get_wire_width(wire);
        assert_eq!(wire_width, port_width, "output port `{port_name}` has incorrect width;  expected: {port_width}  actual: {wire_width}");
    }

    (connections, builder.build())
}

#[test]
fn simple_and_gate() {
    const JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/import_tests/yosys/simple_and_gate.json"
    ));

    let width = NonZeroU8::new(8).unwrap();
    let (connections, mut sim) =
        test_yosys_import(JSON, &[("a", width), ("b", width)], &[("o", width)]);

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

        (0xAA, 0xAA) -> 0xAA,
        (0x55, 0x55) -> 0x55,
        (0xAA, 0x55) -> 0,
    );

    test_binary_module(
        &mut sim,
        connections.inputs["a"],
        connections.inputs["b"],
        connections.outputs["o"],
        width,
        TEST_DATA,
        10,
    );
}

#[test]
fn program_counter() {
    const JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/import_tests/yosys/program_counter.json"
    ));

    let width = NonZeroU8::new(32).unwrap();
    let (connections, mut sim) = test_yosys_import(
        JSON,
        &[
            ("data_in", width),
            ("inc", width),
            ("load", NonZeroU8::MIN),
            ("enable", NonZeroU8::MIN),
            ("reset", NonZeroU8::MIN),
            ("clk", NonZeroU8::MIN),
        ],
        &[("pc_next", width), ("pc_value", width)],
    );

    struct TestData {
        data_in: u32,
        inc: u32,
        load: bool,
        enable: bool,
        reset: bool,
        clk: bool,
        pc_next: LogicState,
        pc_value: LogicState,
    }

    macro_rules! test_data {
        (@BIT +) => { true };
        (@BIT -) => { false };
        ($(($d:literal, $i:literal, LD $ld:tt, EN $en:tt, RST $rst:tt, CLK $clk:tt) -> ($n:tt, $v:tt)),* $(,)?) => {
            &[
                $(
                    TestData {
                        data_in: $d,
                        inc: $i,
                        load: test_data!(@BIT $ld),
                        enable: test_data!(@BIT $en),
                        reset: test_data!(@BIT $rst),
                        clk: test_data!(@BIT $clk),
                        pc_next: logic_state!($n),
                        pc_value: logic_state!($v),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data!(
        (0, 0, LD-, EN-, RST-, CLK-) -> (0, 0),
        (0, 0, LD-, EN-, RST+, CLK-) -> (0, 0),
        (0, 0, LD-, EN-, RST+, CLK+) -> (0, 0),
        (0, 0, LD-, EN-, RST-, CLK-) -> (0, 0),

        (0, 1, LD-, EN-, RST-, CLK-) -> (0, 0),
        (0, 1, LD-, EN+, RST-, CLK-) -> (1, 0),
        (0, 1, LD-, EN+, RST-, CLK+) -> (2, 1),
        (0, 1, LD-, EN-, RST-, CLK-) -> (1, 1),

        (4, 0, LD-, EN-, RST-, CLK-) -> (1, 1),
        (4, 0, LD+, EN-, RST-, CLK-) -> (1, 1),
        (4, 0, LD+, EN-, RST-, CLK+) -> (1, 1),
        (4, 0, LD-, EN-, RST-, CLK-) -> (1, 1),
        (4, 2, LD+, EN+, RST-, CLK-) -> (4, 1),
        (4, 2, LD+, EN+, RST-, CLK+) -> (4, 4),
        (4, 2, LD-, EN+, RST-, CLK-) -> (6, 4),
        (4, 2, LD-, EN-, RST-, CLK-) -> (4, 4),

        (0, 1, LD-, EN+, RST-, CLK-) -> (5, 4),
        (0, 1, LD-, EN+, RST+, CLK-) -> (0, 4),
        (0, 1, LD-, EN+, RST+, CLK+) -> (0, 0),
        (0, 1, LD-, EN+, RST-, CLK-) -> (1, 0),
    );

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        sim.set_wire_drive(
            connections.inputs["data_in"],
            &LogicState::from_int(test_data.data_in),
        );
        sim.set_wire_drive(
            connections.inputs["inc"],
            &LogicState::from_int(test_data.inc),
        );
        sim.set_wire_drive(
            connections.inputs["load"],
            &LogicState::from_bool(test_data.load),
        );
        sim.set_wire_drive(
            connections.inputs["enable"],
            &LogicState::from_bool(test_data.enable),
        );
        sim.set_wire_drive(
            connections.inputs["reset"],
            &LogicState::from_bool(test_data.reset),
        );
        sim.set_wire_drive(
            connections.inputs["clk"],
            &LogicState::from_bool(test_data.clk),
        );

        match sim.run_sim(50) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let pc_next = sim.get_wire_state(connections.outputs["pc_next"]);
        let pc_value = sim.get_wire_state(connections.outputs["pc_value"]);

        assert!(
            pc_next.eq(&test_data.pc_next, width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.pc_next.display_string(width),
            pc_next.display_string(width),
        );

        assert!(
            pc_value.eq(&test_data.pc_value, width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.pc_value.display_string(width),
            pc_value.display_string(width),
        );
    }
}

#[test]
fn proc_mux() {
    const JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/import_tests/yosys/proc_mux.json"
    ));

    let (connections, mut sim) = test_yosys_import(
        JSON,
        &[
            ("data_in", NonZeroU8::new(3).unwrap()),
            ("select_0", NonZeroU8::MIN),
            ("select_1", NonZeroU8::MIN),
        ],
        &[("data_out", NonZeroU8::MIN)],
    );

    struct TestData {
        data: LogicState,
        select: [LogicState; 2],
        output: LogicState,
    }

    macro_rules! test_data {
        ($(($d:tt, [$($s:tt),+ $(,)?]) -> $o:tt),* $(,)?) => {
            &[
                $(
                    TestData {
                        data: logic_state!($d),
                        select: [$(logic_state!($s)),+],
                        output: logic_state!($o),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data!(
        (0b000, [0, 0]) -> 0,
        (0b000, [1, 0]) -> 0,
        (0b000, [0, 1]) -> 0,

        (0b001, [0, 0]) -> 0,
        (0b001, [1, 0]) -> 1,
        (0b001, [0, 1]) -> 0,

        (0b010, [0, 0]) -> 0,
        (0b010, [1, 0]) -> 0,
        (0b010, [0, 1]) -> 1,

        (0b011, [0, 0]) -> 0,
        (0b011, [1, 0]) -> 1,
        (0b011, [0, 1]) -> 1,

        (0b100, [0, 0]) -> 1,
        (0b100, [1, 0]) -> 0,
        (0b100, [0, 1]) -> 0,

        (0b101, [0, 0]) -> 1,
        (0b101, [1, 0]) -> 1,
        (0b101, [0, 1]) -> 0,

        (0b110, [0, 0]) -> 1,
        (0b110, [1, 0]) -> 0,
        (0b110, [0, 1]) -> 1,

        (0b111, [0, 0]) -> 1,
        (0b111, [1, 0]) -> 1,
        (0b111, [0, 1]) -> 1,
    );

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        sim.set_wire_drive(connections.inputs["data_in"], &test_data.data);
        sim.set_wire_drive(connections.inputs["select_0"], &test_data.select[0]);
        sim.set_wire_drive(connections.inputs["select_1"], &test_data.select[1]);

        match sim.run_sim(4) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output = sim.get_wire_state(connections.outputs["data_out"]);

        assert!(
            output.eq(&test_data.output, NonZeroU8::MIN),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(NonZeroU8::MIN),
            output.display_string(NonZeroU8::MIN),
        );
    }
}
