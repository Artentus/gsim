use super::super::*;
use crate::import::yosys::*;

#[cfg(test)]
fn test_yosys_import(
    json: &str,
    expected_inputs: &[(&str, LogicWidth)],
    expected_outputs: &[(&str, LogicWidth)],
) -> (ModuleConnections, Simulator) {
    let importer = YosysModuleImporter::from_json_str(json).unwrap();
    let mut builder = SimulatorBuilder::default();
    let connections = dbg!(builder.import_module(&importer).unwrap());

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

    let width = LogicWidth::new(8).unwrap();
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
