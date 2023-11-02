use crate::*;

#[test]
fn simple_gate() {
    let mut builder = SimulatorBuilder::default();

    let a = builder.add_wire(NonZeroU8::MIN).unwrap();
    let b = builder.add_wire(NonZeroU8::MIN).unwrap();
    let o = builder.add_wire(NonZeroU8::MIN).unwrap();
    builder.add_and_gate(&[a, b], o).unwrap();

    let mut dot = Vec::new();
    builder.write_dot(&mut dot, false).unwrap();
    let dot = String::from_utf8(dot).unwrap();

    const EXPECTED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/dot_export_tests/simple_gate.dot"
    ));

    assert_eq!(dot, EXPECTED);
}
