use super::*;

const WIDTH_1: BitWidth = bit_width!(1);
const WIDTH_2: BitWidth = bit_width!(2);
const WIDTH_4: BitWidth = bit_width!(4);
const WIDTH_5: BitWidth = bit_width!(5);
const WIDTH_16: BitWidth = bit_width!(16);
const WIDTH_32: BitWidth = bit_width!(32);
const WIDTH_33: BitWidth = bit_width!(33);
const WIDTH_64: BitWidth = bit_width!(64);
const WIDTH_128: BitWidth = bit_width!(128);

#[test]
fn and_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0) -> logic_0,
            (high_z, logic_1) -> undefined,
            (undefined, logic_0) -> logic_0,
            (undefined, logic_1) -> undefined,
            (logic_0, high_z) -> logic_0,
            (logic_1, high_z) -> undefined,
            (logic_0, undefined) -> logic_0,
            (logic_1, undefined) -> undefined,
            (logic_0, logic_0) -> logic_0,
            (logic_0, logic_1) -> logic_0,
            (logic_1, logic_0) -> logic_0,
            (logic_1, logic_1) -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_and_gate, width, test_data, 2);
    }
}

#[test]
fn or_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z)       -> undefined,
            (high_z, undefined)    -> undefined,
            (undefined, high_z)    -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0)      -> undefined,
            (high_z, logic_1)      -> logic_1,
            (undefined, logic_0)   -> undefined,
            (undefined, logic_1)   -> logic_1,
            (logic_0, high_z)      -> undefined,
            (logic_1, high_z)      -> logic_1,
            (logic_0, undefined)   -> undefined,
            (logic_1, undefined)   -> logic_1,
            (logic_0, logic_0)     -> logic_0,
            (logic_0, logic_1)     -> logic_1,
            (logic_1, logic_0)     -> logic_1,
            (logic_1, logic_1)     -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_or_gate, width, test_data, 2);
    }
}

#[test]
fn xor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z)       -> undefined,
            (high_z, undefined)    -> undefined,
            (undefined, high_z)    -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0)      -> undefined,
            (high_z, logic_1)      -> undefined,
            (undefined, logic_0)   -> undefined,
            (undefined, logic_1)   -> undefined,
            (logic_0, high_z)      -> undefined,
            (logic_1, high_z)      -> undefined,
            (logic_0, undefined)   -> undefined,
            (logic_1, undefined)   -> undefined,
            (logic_0, logic_0)     -> logic_0,
            (logic_0, logic_1)     -> logic_1,
            (logic_1, logic_0)     -> logic_1,
            (logic_1, logic_1)     -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_xor_gate, width, test_data, 2);
    }
}

#[test]
fn nand_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0) -> logic_1,
            (high_z, logic_1) -> undefined,
            (undefined, logic_0) -> logic_1,
            (undefined, logic_1) -> undefined,
            (logic_0, high_z) -> logic_1,
            (logic_1, high_z) -> undefined,
            (logic_0, undefined) -> logic_1,
            (logic_1, undefined) -> undefined,
            (logic_0, logic_0) -> logic_1,
            (logic_0, logic_1) -> logic_1,
            (logic_1, logic_0) -> logic_1,
            (logic_1, logic_1) -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_nand_gate, width, test_data, 2);
    }
}

#[test]
fn nor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0) -> undefined,
            (high_z, logic_1) -> logic_0,
            (undefined, logic_0) -> undefined,
            (undefined, logic_1) -> logic_0,
            (logic_0, high_z) -> undefined,
            (logic_1, high_z) -> logic_0,
            (logic_0, undefined) -> undefined,
            (logic_1, undefined) -> logic_0,
            (logic_0, logic_0) -> logic_1,
            (logic_0, logic_1) -> logic_0,
            (logic_1, logic_0) -> logic_0,
            (logic_1, logic_1) -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_nor_gate, width, test_data, 2);
    }
}

#[test]
fn xnor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, logic_0) -> undefined,
            (high_z, logic_1) -> undefined,
            (undefined, logic_0) -> undefined,
            (undefined, logic_1) -> undefined,
            (logic_0, high_z) -> undefined,
            (logic_1, high_z) -> undefined,
            (logic_0, undefined) -> undefined,
            (logic_1, undefined) -> undefined,
            (logic_0, logic_0) -> logic_1,
            (logic_0, logic_1) -> logic_0,
            (logic_1, logic_0) -> logic_0,
            (logic_1, logic_1) -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_xnor_gate, width, test_data, 2);
    }
}

#[test]
fn wide_and_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> logic_0,
            (high_z   , logic_1  , high_z) -> undefined,
            (undefined, logic_0  , high_z) -> logic_0,
            (undefined, logic_1  , high_z) -> undefined,
            (logic_0  , high_z   , high_z) -> logic_0,
            (logic_1  , high_z   , high_z) -> undefined,
            (logic_0  , undefined, high_z) -> logic_0,
            (logic_1  , undefined, high_z) -> undefined,
            (logic_0  , logic_0  , high_z) -> logic_0,
            (logic_0  , logic_1  , high_z) -> logic_0,
            (logic_1  , logic_0  , high_z) -> logic_0,
            (logic_1  , logic_1  , high_z) -> undefined,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> logic_0,
            (high_z   , logic_1  , undefined) -> undefined,
            (undefined, logic_0  , undefined) -> logic_0,
            (undefined, logic_1  , undefined) -> undefined,
            (logic_0  , high_z   , undefined) -> logic_0,
            (logic_1  , high_z   , undefined) -> undefined,
            (logic_0  , undefined, undefined) -> logic_0,
            (logic_1  , undefined, undefined) -> undefined,
            (logic_0  , logic_0  , undefined) -> logic_0,
            (logic_0  , logic_1  , undefined) -> logic_0,
            (logic_1  , logic_0  , undefined) -> logic_0,
            (logic_1  , logic_1  , undefined) -> undefined,

            (high_z   , high_z   , logic_0) -> logic_0,
            (high_z   , undefined, logic_0) -> logic_0,
            (undefined, high_z   , logic_0) -> logic_0,
            (undefined, undefined, logic_0) -> logic_0,
            (high_z   , logic_0  , logic_0) -> logic_0,
            (high_z   , logic_1  , logic_0) -> logic_0,
            (undefined, logic_0  , logic_0) -> logic_0,
            (undefined, logic_1  , logic_0) -> logic_0,
            (logic_0  , high_z   , logic_0) -> logic_0,
            (logic_1  , high_z   , logic_0) -> logic_0,
            (logic_0  , undefined, logic_0) -> logic_0,
            (logic_1  , undefined, logic_0) -> logic_0,
            (logic_0  , logic_0  , logic_0) -> logic_0,
            (logic_0  , logic_1  , logic_0) -> logic_0,
            (logic_1  , logic_0  , logic_0) -> logic_0,
            (logic_1  , logic_1  , logic_0) -> logic_0,

            (high_z   , high_z   , logic_1) -> undefined,
            (high_z   , undefined, logic_1) -> undefined,
            (undefined, high_z   , logic_1) -> undefined,
            (undefined, undefined, logic_1) -> undefined,
            (high_z   , logic_0  , logic_1) -> logic_0,
            (high_z   , logic_1  , logic_1) -> undefined,
            (undefined, logic_0  , logic_1) -> logic_0,
            (undefined, logic_1  , logic_1) -> undefined,
            (logic_0  , high_z   , logic_1) -> logic_0,
            (logic_1  , high_z   , logic_1) -> undefined,
            (logic_0  , undefined, logic_1) -> logic_0,
            (logic_1  , undefined, logic_1) -> undefined,
            (logic_0  , logic_0  , logic_1) -> logic_0,
            (logic_0  , logic_1  , logic_1) -> logic_0,
            (logic_1  , logic_0  , logic_1) -> logic_0,
            (logic_1  , logic_1  , logic_1) -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_and_gate, width, test_data, 2);
    }
}

#[test]
fn wide_or_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> undefined,
            (high_z   , logic_1  , high_z) -> logic_1,
            (undefined, logic_0  , high_z) -> undefined,
            (undefined, logic_1  , high_z) -> logic_1,
            (logic_0  , high_z   , high_z) -> undefined,
            (logic_1  , high_z   , high_z) -> logic_1,
            (logic_0  , undefined, high_z) -> undefined,
            (logic_1  , undefined, high_z) -> logic_1,
            (logic_0  , logic_0  , high_z) -> undefined,
            (logic_0  , logic_1  , high_z) -> logic_1,
            (logic_1  , logic_0  , high_z) -> logic_1,
            (logic_1  , logic_1  , high_z) -> logic_1,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> undefined,
            (high_z   , logic_1  , undefined) -> logic_1,
            (undefined, logic_0  , undefined) -> undefined,
            (undefined, logic_1  , undefined) -> logic_1,
            (logic_0  , high_z   , undefined) -> undefined,
            (logic_1  , high_z   , undefined) -> logic_1,
            (logic_0  , undefined, undefined) -> undefined,
            (logic_1  , undefined, undefined) -> logic_1,
            (logic_0  , logic_0  , undefined) -> undefined,
            (logic_0  , logic_1  , undefined) -> logic_1,
            (logic_1  , logic_0  , undefined) -> logic_1,
            (logic_1  , logic_1  , undefined) -> logic_1,

            (high_z   , high_z   , logic_0) -> undefined,
            (high_z   , undefined, logic_0) -> undefined,
            (undefined, high_z   , logic_0) -> undefined,
            (undefined, undefined, logic_0) -> undefined,
            (high_z   , logic_0  , logic_0) -> undefined,
            (high_z   , logic_1  , logic_0) -> logic_1,
            (undefined, logic_0  , logic_0) -> undefined,
            (undefined, logic_1  , logic_0) -> logic_1,
            (logic_0  , high_z   , logic_0) -> undefined,
            (logic_1  , high_z   , logic_0) -> logic_1,
            (logic_0  , undefined, logic_0) -> undefined,
            (logic_1  , undefined, logic_0) -> logic_1,
            (logic_0  , logic_0  , logic_0) -> logic_0,
            (logic_0  , logic_1  , logic_0) -> logic_1,
            (logic_1  , logic_0  , logic_0) -> logic_1,
            (logic_1  , logic_1  , logic_0) -> logic_1,

            (high_z   , high_z   , logic_1) -> logic_1,
            (high_z   , undefined, logic_1) -> logic_1,
            (undefined, high_z   , logic_1) -> logic_1,
            (undefined, undefined, logic_1) -> logic_1,
            (high_z   , logic_0  , logic_1) -> logic_1,
            (high_z   , logic_1  , logic_1) -> logic_1,
            (undefined, logic_0  , logic_1) -> logic_1,
            (undefined, logic_1  , logic_1) -> logic_1,
            (logic_0  , high_z   , logic_1) -> logic_1,
            (logic_1  , high_z   , logic_1) -> logic_1,
            (logic_0  , undefined, logic_1) -> logic_1,
            (logic_1  , undefined, logic_1) -> logic_1,
            (logic_0  , logic_0  , logic_1) -> logic_1,
            (logic_0  , logic_1  , logic_1) -> logic_1,
            (logic_1  , logic_0  , logic_1) -> logic_1,
            (logic_1  , logic_1  , logic_1) -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_or_gate, width, test_data, 2);
    }
}

#[test]
fn wide_xor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> undefined,
            (high_z   , logic_1  , high_z) -> undefined,
            (undefined, logic_0  , high_z) -> undefined,
            (undefined, logic_1  , high_z) -> undefined,
            (logic_0  , high_z   , high_z) -> undefined,
            (logic_1  , high_z   , high_z) -> undefined,
            (logic_0  , undefined, high_z) -> undefined,
            (logic_1  , undefined, high_z) -> undefined,
            (logic_0  , logic_0  , high_z) -> undefined,
            (logic_0  , logic_1  , high_z) -> undefined,
            (logic_1  , logic_0  , high_z) -> undefined,
            (logic_1  , logic_1  , high_z) -> undefined,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> undefined,
            (high_z   , logic_1  , undefined) -> undefined,
            (undefined, logic_0  , undefined) -> undefined,
            (undefined, logic_1  , undefined) -> undefined,
            (logic_0  , high_z   , undefined) -> undefined,
            (logic_1  , high_z   , undefined) -> undefined,
            (logic_0  , undefined, undefined) -> undefined,
            (logic_1  , undefined, undefined) -> undefined,
            (logic_0  , logic_0  , undefined) -> undefined,
            (logic_0  , logic_1  , undefined) -> undefined,
            (logic_1  , logic_0  , undefined) -> undefined,
            (logic_1  , logic_1  , undefined) -> undefined,

            (high_z   , high_z   , logic_0) -> undefined,
            (high_z   , undefined, logic_0) -> undefined,
            (undefined, high_z   , logic_0) -> undefined,
            (undefined, undefined, logic_0) -> undefined,
            (high_z   , logic_0  , logic_0) -> undefined,
            (high_z   , logic_1  , logic_0) -> undefined,
            (undefined, logic_0  , logic_0) -> undefined,
            (undefined, logic_1  , logic_0) -> undefined,
            (logic_0  , high_z   , logic_0) -> undefined,
            (logic_1  , high_z   , logic_0) -> undefined,
            (logic_0  , undefined, logic_0) -> undefined,
            (logic_1  , undefined, logic_0) -> undefined,
            (logic_0  , logic_0  , logic_0) -> logic_0,
            (logic_0  , logic_1  , logic_0) -> logic_1,
            (logic_1  , logic_0  , logic_0) -> logic_1,
            (logic_1  , logic_1  , logic_0) -> logic_0,

            (high_z   , high_z   , logic_1) -> undefined,
            (high_z   , undefined, logic_1) -> undefined,
            (undefined, high_z   , logic_1) -> undefined,
            (undefined, undefined, logic_1) -> undefined,
            (high_z   , logic_0  , logic_1) -> undefined,
            (high_z   , logic_1  , logic_1) -> undefined,
            (undefined, logic_0  , logic_1) -> undefined,
            (undefined, logic_1  , logic_1) -> undefined,
            (logic_0  , high_z   , logic_1) -> undefined,
            (logic_1  , high_z   , logic_1) -> undefined,
            (logic_0  , undefined, logic_1) -> undefined,
            (logic_1  , undefined, logic_1) -> undefined,
            (logic_0  , logic_0  , logic_1) -> logic_1,
            (logic_0  , logic_1  , logic_1) -> logic_0,
            (logic_1  , logic_0  , logic_1) -> logic_0,
            (logic_1  , logic_1  , logic_1) -> logic_1,
        );

        test_wide_gate(SimulatorBuilder::add_xor_gate, width, test_data, 2);
    }
}

#[test]
fn wide_nand_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> logic_1,
            (high_z   , logic_1  , high_z) -> undefined,
            (undefined, logic_0  , high_z) -> logic_1,
            (undefined, logic_1  , high_z) -> undefined,
            (logic_0  , high_z   , high_z) -> logic_1,
            (logic_1  , high_z   , high_z) -> undefined,
            (logic_0  , undefined, high_z) -> logic_1,
            (logic_1  , undefined, high_z) -> undefined,
            (logic_0  , logic_0  , high_z) -> logic_1,
            (logic_0  , logic_1  , high_z) -> logic_1,
            (logic_1  , logic_0  , high_z) -> logic_1,
            (logic_1  , logic_1  , high_z) -> undefined,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> logic_1,
            (high_z   , logic_1  , undefined) -> undefined,
            (undefined, logic_0  , undefined) -> logic_1,
            (undefined, logic_1  , undefined) -> undefined,
            (logic_0  , high_z   , undefined) -> logic_1,
            (logic_1  , high_z   , undefined) -> undefined,
            (logic_0  , undefined, undefined) -> logic_1,
            (logic_1  , undefined, undefined) -> undefined,
            (logic_0  , logic_0  , undefined) -> logic_1,
            (logic_0  , logic_1  , undefined) -> logic_1,
            (logic_1  , logic_0  , undefined) -> logic_1,
            (logic_1  , logic_1  , undefined) -> undefined,

            (high_z   , high_z   , logic_0) -> logic_1,
            (high_z   , undefined, logic_0) -> logic_1,
            (undefined, high_z   , logic_0) -> logic_1,
            (undefined, undefined, logic_0) -> logic_1,
            (high_z   , logic_0  , logic_0) -> logic_1,
            (high_z   , logic_1  , logic_0) -> logic_1,
            (undefined, logic_0  , logic_0) -> logic_1,
            (undefined, logic_1  , logic_0) -> logic_1,
            (logic_0  , high_z   , logic_0) -> logic_1,
            (logic_1  , high_z   , logic_0) -> logic_1,
            (logic_0  , undefined, logic_0) -> logic_1,
            (logic_1  , undefined, logic_0) -> logic_1,
            (logic_0  , logic_0  , logic_0) -> logic_1,
            (logic_0  , logic_1  , logic_0) -> logic_1,
            (logic_1  , logic_0  , logic_0) -> logic_1,
            (logic_1  , logic_1  , logic_0) -> logic_1,

            (high_z   , high_z   , logic_1) -> undefined,
            (high_z   , undefined, logic_1) -> undefined,
            (undefined, high_z   , logic_1) -> undefined,
            (undefined, undefined, logic_1) -> undefined,
            (high_z   , logic_0  , logic_1) -> logic_1,
            (high_z   , logic_1  , logic_1) -> undefined,
            (undefined, logic_0  , logic_1) -> logic_1,
            (undefined, logic_1  , logic_1) -> undefined,
            (logic_0  , high_z   , logic_1) -> logic_1,
            (logic_1  , high_z   , logic_1) -> undefined,
            (logic_0  , undefined, logic_1) -> logic_1,
            (logic_1  , undefined, logic_1) -> undefined,
            (logic_0  , logic_0  , logic_1) -> logic_1,
            (logic_0  , logic_1  , logic_1) -> logic_1,
            (logic_1  , logic_0  , logic_1) -> logic_1,
            (logic_1  , logic_1  , logic_1) -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_nand_gate, width, test_data, 2);
    }
}

#[test]
fn wide_nor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> undefined,
            (high_z   , logic_1  , high_z) -> logic_0,
            (undefined, logic_0  , high_z) -> undefined,
            (undefined, logic_1  , high_z) -> logic_0,
            (logic_0  , high_z   , high_z) -> undefined,
            (logic_1  , high_z   , high_z) -> logic_0,
            (logic_0  , undefined, high_z) -> undefined,
            (logic_1  , undefined, high_z) -> logic_0,
            (logic_0  , logic_0  , high_z) -> undefined,
            (logic_0  , logic_1  , high_z) -> logic_0,
            (logic_1  , logic_0  , high_z) -> logic_0,
            (logic_1  , logic_1  , high_z) -> logic_0,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> undefined,
            (high_z   , logic_1  , undefined) -> logic_0,
            (undefined, logic_0  , undefined) -> undefined,
            (undefined, logic_1  , undefined) -> logic_0,
            (logic_0  , high_z   , undefined) -> undefined,
            (logic_1  , high_z   , undefined) -> logic_0,
            (logic_0  , undefined, undefined) -> undefined,
            (logic_1  , undefined, undefined) -> logic_0,
            (logic_0  , logic_0  , undefined) -> undefined,
            (logic_0  , logic_1  , undefined) -> logic_0,
            (logic_1  , logic_0  , undefined) -> logic_0,
            (logic_1  , logic_1  , undefined) -> logic_0,

            (high_z   , high_z   , logic_0) -> undefined,
            (high_z   , undefined, logic_0) -> undefined,
            (undefined, high_z   , logic_0) -> undefined,
            (undefined, undefined, logic_0) -> undefined,
            (high_z   , logic_0  , logic_0) -> undefined,
            (high_z   , logic_1  , logic_0) -> logic_0,
            (undefined, logic_0  , logic_0) -> undefined,
            (undefined, logic_1  , logic_0) -> logic_0,
            (logic_0  , high_z   , logic_0) -> undefined,
            (logic_1  , high_z   , logic_0) -> logic_0,
            (logic_0  , undefined, logic_0) -> undefined,
            (logic_1  , undefined, logic_0) -> logic_0,
            (logic_0  , logic_0  , logic_0) -> logic_1,
            (logic_0  , logic_1  , logic_0) -> logic_0,
            (logic_1  , logic_0  , logic_0) -> logic_0,
            (logic_1  , logic_1  , logic_0) -> logic_0,

            (high_z   , high_z   , logic_1) -> logic_0,
            (high_z   , undefined, logic_1) -> logic_0,
            (undefined, high_z   , logic_1) -> logic_0,
            (undefined, undefined, logic_1) -> logic_0,
            (high_z   , logic_0  , logic_1) -> logic_0,
            (high_z   , logic_1  , logic_1) -> logic_0,
            (undefined, logic_0  , logic_1) -> logic_0,
            (undefined, logic_1  , logic_1) -> logic_0,
            (logic_0  , high_z   , logic_1) -> logic_0,
            (logic_1  , high_z   , logic_1) -> logic_0,
            (logic_0  , undefined, logic_1) -> logic_0,
            (logic_1  , undefined, logic_1) -> logic_0,
            (logic_0  , logic_0  , logic_1) -> logic_0,
            (logic_0  , logic_1  , logic_1) -> logic_0,
            (logic_1  , logic_0  , logic_1) -> logic_0,
            (logic_1  , logic_1  , logic_1) -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_nor_gate, width, test_data, 2);
    }
}

#[test]
fn wide_xnor_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = wide_gate_test_data!(width;
            (high_z   , high_z   , high_z) -> undefined,
            (high_z   , undefined, high_z) -> undefined,
            (undefined, high_z   , high_z) -> undefined,
            (undefined, undefined, high_z) -> undefined,
            (high_z   , logic_0  , high_z) -> undefined,
            (high_z   , logic_1  , high_z) -> undefined,
            (undefined, logic_0  , high_z) -> undefined,
            (undefined, logic_1  , high_z) -> undefined,
            (logic_0  , high_z   , high_z) -> undefined,
            (logic_1  , high_z   , high_z) -> undefined,
            (logic_0  , undefined, high_z) -> undefined,
            (logic_1  , undefined, high_z) -> undefined,
            (logic_0  , logic_0  , high_z) -> undefined,
            (logic_0  , logic_1  , high_z) -> undefined,
            (logic_1  , logic_0  , high_z) -> undefined,
            (logic_1  , logic_1  , high_z) -> undefined,

            (high_z   , high_z   , undefined) -> undefined,
            (high_z   , undefined, undefined) -> undefined,
            (undefined, high_z   , undefined) -> undefined,
            (undefined, undefined, undefined) -> undefined,
            (high_z   , logic_0  , undefined) -> undefined,
            (high_z   , logic_1  , undefined) -> undefined,
            (undefined, logic_0  , undefined) -> undefined,
            (undefined, logic_1  , undefined) -> undefined,
            (logic_0  , high_z   , undefined) -> undefined,
            (logic_1  , high_z   , undefined) -> undefined,
            (logic_0  , undefined, undefined) -> undefined,
            (logic_1  , undefined, undefined) -> undefined,
            (logic_0  , logic_0  , undefined) -> undefined,
            (logic_0  , logic_1  , undefined) -> undefined,
            (logic_1  , logic_0  , undefined) -> undefined,
            (logic_1  , logic_1  , undefined) -> undefined,

            (high_z   , high_z   , logic_0) -> undefined,
            (high_z   , undefined, logic_0) -> undefined,
            (undefined, high_z   , logic_0) -> undefined,
            (undefined, undefined, logic_0) -> undefined,
            (high_z   , logic_0  , logic_0) -> undefined,
            (high_z   , logic_1  , logic_0) -> undefined,
            (undefined, logic_0  , logic_0) -> undefined,
            (undefined, logic_1  , logic_0) -> undefined,
            (logic_0  , high_z   , logic_0) -> undefined,
            (logic_1  , high_z   , logic_0) -> undefined,
            (logic_0  , undefined, logic_0) -> undefined,
            (logic_1  , undefined, logic_0) -> undefined,
            (logic_0  , logic_0  , logic_0) -> logic_1,
            (logic_0  , logic_1  , logic_0) -> logic_0,
            (logic_1  , logic_0  , logic_0) -> logic_0,
            (logic_1  , logic_1  , logic_0) -> logic_1,

            (high_z   , high_z   , logic_1) -> undefined,
            (high_z   , undefined, logic_1) -> undefined,
            (undefined, high_z   , logic_1) -> undefined,
            (undefined, undefined, logic_1) -> undefined,
            (high_z   , logic_0  , logic_1) -> undefined,
            (high_z   , logic_1  , logic_1) -> undefined,
            (undefined, logic_0  , logic_1) -> undefined,
            (undefined, logic_1  , logic_1) -> undefined,
            (logic_0  , high_z   , logic_1) -> undefined,
            (logic_1  , high_z   , logic_1) -> undefined,
            (logic_0  , undefined, logic_1) -> undefined,
            (logic_1  , undefined, logic_1) -> undefined,
            (logic_0  , logic_0  , logic_1) -> logic_0,
            (logic_0  , logic_1  , logic_1) -> logic_1,
            (logic_1  , logic_0  , logic_1) -> logic_1,
            (logic_1  , logic_1  , logic_1) -> logic_0,
        );

        test_wide_gate(SimulatorBuilder::add_xnor_gate, width, test_data, 2);
    }
}

#[test]
fn not_gate() {
    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = unary_gate_test_data!(width;
            high_z -> undefined,
            undefined -> undefined,
            logic_0 -> logic_1,
            logic_1 -> logic_0,
        );

        test_unary_gate(SimulatorBuilder::add_not_gate, width, test_data, 2);
    }
}

#[test]
fn buffer() {
    macro_rules! buffer_test_data {
        ($width:expr; $(($a:tt, $b:tt) -> $o:tt),* $(,)?) => {
            &[
                $(
                    BinaryGateTestData {
                        input_a: logic_state!($width; $a),
                        input_b: logic_state!(WIDTH_1; $b),
                        output: logic_state!($width; $o),
                    },
                )*
            ]
        };
    }

    for width in [WIDTH_1, WIDTH_32, WIDTH_33, WIDTH_64] {
        let test_data = buffer_test_data!(width;
            (high_z, high_z) -> high_z,
            (undefined, high_z) -> high_z,
            (logic_0, high_z) -> high_z,
            (logic_1, high_z) -> high_z,

            (high_z, undefined) -> undefined,
            (undefined, undefined) -> undefined,
            (logic_0, undefined) -> undefined,
            (logic_1, undefined) -> undefined,

            (high_z, logic_0) -> high_z,
            (undefined, logic_0) -> high_z,
            (logic_0, logic_0) -> high_z,
            (logic_1, logic_0) -> high_z,

            (high_z, logic_1) -> undefined,
            (undefined, logic_1) -> undefined,
            (logic_0, logic_1) -> logic_0,
            (logic_1, logic_1) -> logic_1,
        );

        for (i, test_data) in test_data.iter().enumerate() {
            let mut builder = SimulatorBuilder::default();

            let input = builder.add_wire(width).unwrap();
            builder.set_wire_drive(input, &test_data.input_a).unwrap();
            let enable = builder.add_wire(WIDTH_1).unwrap();
            builder.set_wire_drive(enable, &test_data.input_b).unwrap();
            let output = builder.add_wire(width).unwrap();
            let _gate = builder.add_buffer(input, enable, output).unwrap();

            let mut sim = builder.build();

            match sim.run_sim(2) {
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
}

#[test]
fn add() {
    for width in [WIDTH_16, WIDTH_32, WIDTH_64] {
        let test_data = binary_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, 0) -> undefined,
            (undefined, 0) -> undefined,
            (0, high_z) -> undefined,
            (0, undefined) -> undefined,

            (0, 0) -> 0,
            (0, 1) -> 1,
            (1, 0) -> 1,
            (1, 1) -> 2,
            (0, {u64::MAX}) -> {u64::MAX},
            ({u64::MAX}, 0) -> {u64::MAX},
            (1, {u64::MAX}) -> 0,
            ({u64::MAX}, 1) -> 0,
            ({u64::MAX}, {u64::MAX}) -> {u64::MAX - 1},
        );

        test_binary_gate(SimulatorBuilder::add_add, width, test_data, 2);
    }
}

#[test]
fn sub() {
    for width in [WIDTH_16, WIDTH_32, WIDTH_64] {
        let test_data = binary_gate_test_data!(width;
            (high_z, high_z) -> undefined,
            (high_z, undefined) -> undefined,
            (undefined, high_z) -> undefined,
            (undefined, undefined) -> undefined,
            (high_z, 0) -> undefined,
            (undefined, 0) -> undefined,
            (0, high_z) -> undefined,
            (0, undefined) -> undefined,

            (0, 0) -> 0,
            (0, 1) -> {u64::MAX},
            (1, 0) -> 1,
            (1, 1) -> 0,
            (0, {u64::MAX}) -> 1,
            ({u64::MAX}, 0) -> {u64::MAX},
            ({u64::MAX}, {u64::MAX}) -> 0,
        );

        test_binary_gate(SimulatorBuilder::add_sub, width, test_data, 2);
    }
}

/*
#[test]
fn neg() {
    const TEST_DATA: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,

        0 -> 0,
        1 -> LOGIC_1,
        LOGIC_1 -> 1,
    );

    test_unary_gate(SimulatorBuilder::add_neg, WIDTH_16, TEST_DATA, 2);
    test_unary_gate(SimulatorBuilder::add_neg, WIDTH_32, TEST_DATA, 2);
    test_unary_gate(SimulatorBuilder::add_neg, WIDTH_33, TEST_DATA, 2);
    test_unary_gate(SimulatorBuilder::add_neg, WIDTH_64, TEST_DATA, 2);
}

#[test]
fn mul() {
    let test_data: &[BinaryGateTestData] = binary_gate_test_data!(
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
        ({u32::MAX}, {u32::MAX}) -> [1, u32::MAX - 1],
        ([u32::MAX, u32::MAX], [u32::MAX, u32::MAX]) -> [1, 0, u32::MAX - 1, u32::MAX],
        ([0x658c0c38, 0xd50cebfb], [0x901cfad8, 0xc0083189]) -> [0x4838ff40, 0x2201c171, 0xe109006d, 0x9fd0829d],
    );

    test_binary_gate(SimulatorBuilder::add_mul, WIDTH_16, test_data, 2);
    test_binary_gate(SimulatorBuilder::add_mul, WIDTH_32, test_data, 2);
    test_binary_gate(SimulatorBuilder::add_mul, WIDTH_33, test_data, 2);
    test_binary_gate(SimulatorBuilder::add_mul, WIDTH_64, test_data, 2);
    test_binary_gate(SimulatorBuilder::add_mul, WIDTH_128, test_data, 2);
}

#[test]
fn slice() {
    struct TestData {
        input: LogicState,
        offset: u8,
        output: LogicState,
    }

    macro_rules! test_data {
        ($(([$($i:tt),+], $offset:literal) -> [$($o:tt),+]),* $(,)?) => {
            &[
                $(
                    TestData {
                        input: bits!($($i),+),
                        offset: $offset,
                        output: bits!($($o),+),
                    },
                )*
            ]
        };
    }

    let test_data = test_data!(
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

    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let input = builder.add_wire(WIDTH_2).unwrap();
        let output = builder.add_wire(WIDTH_1).unwrap();
        let _gate = builder.add_slice(input, test_data.offset, output).unwrap();

        let mut sim = builder.build();

        sim.set_wire_drive(input, &test_data.input).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_1),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_1),
            output_state.display_string(WIDTH_1),
        );
    }
}

#[test]
fn merge() {
    struct TestData {
        inputs: Vec<LogicState>,
        output: LogicState,
    }

    macro_rules! test_data {
        ($(($([$($i:tt),+]),+) -> [$($o:tt),+]),* $(,)?) => {
            &[
                $(
                    TestData {
                        inputs: vec![$(bits!($($i),+)),+],
                        output: bits!($($o),+),
                    },
                )*
            ]
        };
    }

    let test_data = test_data!(
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

    for (i, test_data) in test_data.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|drive| {
                let wire = builder.add_wire(WIDTH_1).unwrap();
                builder.set_wire_drive(wire, drive).unwrap();
                wire
            })
            .collect();
        let output_width = NonZeroU8::new(test_data.inputs.len() as u8).unwrap();
        let output = builder.add_wire(output_width).unwrap();
        let _gate = builder.add_merge(&inputs, output).unwrap();

        let mut sim = builder.build();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, output_width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(output_width),
            output_state.display_string(output_width),
        );
    }
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

        (1, 32) -> 1,
        (1, 33) -> 2,
        (1, 63) -> 0x80000000,
        (1, 64) -> 1,

        (0x55, 0) -> 0x55,
        (0x55, 1) -> 0xAA,
    );

    test_shifter(SimulatorBuilder::add_left_shift, WIDTH_32, TEST_DATA_32, 2);

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

        (1, 16) -> 1,
        (1, 17) -> 2,
        (1, 31) -> 0x8000,
        (1, 32) -> 1,

        (0x55, 0) -> 0x55,
        (0x55, 1) -> 0xAA,
    );

    test_shifter(SimulatorBuilder::add_left_shift, WIDTH_16, TEST_DATA_16, 2);
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

        (0x80000000, 32) -> 0x80000000,
        (0x80000000, 33) -> 0x40000000,
        (0x80000000, 63) -> 1,
        (0x80000000, 64) -> 0x80000000,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_shifter(
        SimulatorBuilder::add_logical_right_shift,
        WIDTH_32,
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

        (0x8000, 16) -> 0x8000,
        (0x8000, 17) -> 0x4000,
        (0x8000, 31) -> 1,
        (0x8000, 32) -> 0x8000,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_shifter(
        SimulatorBuilder::add_logical_right_shift,
        WIDTH_16,
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

        (0x80000000, 32) -> 0x80000000,
        (0x80000000, 33) -> 0xC0000000,
        (0x80000000, 63) -> 0xFFFFFFFF,
        (0x80000000, 64) -> 0x80000000,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_shifter(
        SimulatorBuilder::add_arithmetic_right_shift,
        WIDTH_32,
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

        (0x8000, 16) -> 0x8000,
        (0x8000, 17) -> 0xC000,
        (0x8000, 31) -> 0xFFFF,
        (0x8000, 32) -> 0x8000,

        (0xAA, 0) -> 0xAA,
        (0xAA, 1) -> 0x55,
    );

    test_shifter(
        SimulatorBuilder::add_arithmetic_right_shift,
        WIDTH_16,
        TEST_DATA_16,
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

    let input_a = builder.add_wire(WIDTH_32).unwrap();
    let input_b = builder.add_wire(WIDTH_32).unwrap();
    let carry_in = builder.add_wire(WIDTH_1).unwrap();
    let output = builder.add_wire(WIDTH_32).unwrap();
    let carry_out = builder.add_wire(WIDTH_1).unwrap();
    let _adder = builder
        .add_adder(input_a, input_b, carry_in, output, carry_out)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_1.iter().enumerate() {
        sim.set_wire_drive(input_a, &test_data.input_a).unwrap();
        sim.set_wire_drive(input_b, &test_data.input_b).unwrap();
        sim.set_wire_drive(carry_in, &test_data.carry_in).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();
        let carry_out_state = sim.get_wire_state(carry_out).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_32),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_32),
            output_state.display_string(WIDTH_32),
        );

        assert!(
            carry_out_state.eq(&test_data.carry_out, WIDTH_1),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.carry_out.display_string(WIDTH_1),
            carry_out_state.display_string(WIDTH_1),
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

    let input_a = builder.add_wire(WIDTH_16).unwrap();
    let input_b = builder.add_wire(WIDTH_16).unwrap();
    let carry_in = builder.add_wire(WIDTH_1).unwrap();
    let output = builder.add_wire(WIDTH_16).unwrap();
    let carry_out = builder.add_wire(WIDTH_1).unwrap();
    let _adder = builder
        .add_adder(input_a, input_b, carry_in, output, carry_out)
        .unwrap();

    let mut sim = builder.build();

    for (i, test_data) in TEST_DATA_2.iter().enumerate() {
        sim.set_wire_drive(input_a, &test_data.input_a).unwrap();
        sim.set_wire_drive(input_b, &test_data.input_b).unwrap();
        sim.set_wire_drive(carry_in, &test_data.carry_in).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();
        let carry_out_state = sim.get_wire_state(carry_out).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_16),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_16),
            output_state.display_string(WIDTH_16),
        );

        assert!(
            carry_out_state.eq(&test_data.carry_out, WIDTH_1),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.carry_out.display_string(WIDTH_1),
            carry_out_state.display_string(WIDTH_1),
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

    const TEST_DATA: &[TestData] = test_data!(
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

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|drive| {
                let wire = builder.add_wire(WIDTH_32).unwrap();
                builder.set_wire_drive(wire, drive).unwrap();
                wire
            })
            .collect();
        let select = builder
            .add_wire(NonZeroU8::new(inputs.len().ilog2() as u8).unwrap())
            .unwrap();
        builder.set_wire_drive(select, &test_data.select).unwrap();
        let output = builder.add_wire(WIDTH_32).unwrap();
        let _mux = builder.add_multiplexer(&inputs, select, output).unwrap();

        let mut sim = builder.build();
        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_32),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_32),
            output_state.display_string(WIDTH_32),
        );
    }
}

#[test]
fn priority_decoder() {
    struct TestData {
        inputs: &'static [LogicState],
        output: LogicState,
    }

    macro_rules! test_data {
        ($([$($i:tt),+ $(,)?] -> $o:tt),* $(,)?) => {
            &[
                $(
                    TestData {
                        inputs: &[$(logic_state!($i)),+],
                        output: logic_state!($o),
                    },
                )*
            ]
        };
    }

    const TEST_DATA: &[TestData] = test_data!(
        [HIGH_Z] -> UNDEFINED,
        [UNDEFINED] -> UNDEFINED,
        [0] -> 0,
        [1] -> 1,

        [HIGH_Z, HIGH_Z] -> UNDEFINED,
        [HIGH_Z, UNDEFINED] -> UNDEFINED,
        [HIGH_Z, 0] -> UNDEFINED,
        [HIGH_Z, 1] -> UNDEFINED,

        [UNDEFINED, HIGH_Z] -> UNDEFINED,
        [UNDEFINED, UNDEFINED] -> UNDEFINED,
        [UNDEFINED, 0] -> UNDEFINED,
        [UNDEFINED, 1] -> UNDEFINED,

        [0, HIGH_Z] -> UNDEFINED,
        [0, UNDEFINED] -> UNDEFINED,
        [0, 0] -> 0,
        [0, 1] -> 2,

        [1, HIGH_Z] -> 1,
        [1, UNDEFINED] -> 1,
        [1, 0] -> 1,
        [1, 1] -> 1,

        [HIGH_Z, HIGH_Z, HIGH_Z] -> UNDEFINED,
        [HIGH_Z, HIGH_Z, UNDEFINED] -> UNDEFINED,
        [HIGH_Z, HIGH_Z, 0] -> UNDEFINED,
        [HIGH_Z, HIGH_Z, 1] -> UNDEFINED,
        [HIGH_Z, UNDEFINED, HIGH_Z] -> UNDEFINED,
        [HIGH_Z, UNDEFINED, UNDEFINED] -> UNDEFINED,
        [HIGH_Z, UNDEFINED, 0] -> UNDEFINED,
        [HIGH_Z, UNDEFINED, 1] -> UNDEFINED,
        [HIGH_Z, 0, HIGH_Z] -> UNDEFINED,
        [HIGH_Z, 0, UNDEFINED] -> UNDEFINED,
        [HIGH_Z, 0, 0] -> UNDEFINED,
        [HIGH_Z, 0, 1] -> UNDEFINED,
        [HIGH_Z, 1, HIGH_Z] -> UNDEFINED,
        [HIGH_Z, 1, UNDEFINED] -> UNDEFINED,
        [HIGH_Z, 1, 0] -> UNDEFINED,
        [HIGH_Z, 1, 1] -> UNDEFINED,

        [UNDEFINED, HIGH_Z, HIGH_Z] -> UNDEFINED,
        [UNDEFINED, HIGH_Z, UNDEFINED] -> UNDEFINED,
        [UNDEFINED, HIGH_Z, 0] -> UNDEFINED,
        [UNDEFINED, HIGH_Z, 1] -> UNDEFINED,
        [UNDEFINED, UNDEFINED, HIGH_Z] -> UNDEFINED,
        [UNDEFINED, UNDEFINED, UNDEFINED] -> UNDEFINED,
        [UNDEFINED, UNDEFINED, 0] -> UNDEFINED,
        [UNDEFINED, UNDEFINED, 1] -> UNDEFINED,
        [UNDEFINED, 0, HIGH_Z] -> UNDEFINED,
        [UNDEFINED, 0, UNDEFINED] -> UNDEFINED,
        [UNDEFINED, 0, 0] -> UNDEFINED,
        [UNDEFINED, 0, 1] -> UNDEFINED,
        [UNDEFINED, 1, HIGH_Z] -> UNDEFINED,
        [UNDEFINED, 1, UNDEFINED] -> UNDEFINED,
        [UNDEFINED, 1, 0] -> UNDEFINED,
        [UNDEFINED, 1, 1] -> UNDEFINED,

        [0, HIGH_Z, HIGH_Z] -> UNDEFINED,
        [0, HIGH_Z, UNDEFINED] -> UNDEFINED,
        [0, HIGH_Z, 0] -> UNDEFINED,
        [0, HIGH_Z, 1] -> UNDEFINED,
        [0, UNDEFINED, HIGH_Z] -> UNDEFINED,
        [0, UNDEFINED, UNDEFINED] -> UNDEFINED,
        [0, UNDEFINED, 0] -> UNDEFINED,
        [0, UNDEFINED, 1] -> UNDEFINED,
        [0, 0, HIGH_Z] -> UNDEFINED,
        [0, 0, UNDEFINED] -> UNDEFINED,
        [0, 0, 0] -> 0,
        [0, 0, 1] -> 3,
        [0, 1, HIGH_Z] -> 2,
        [0, 1, UNDEFINED] -> 2,
        [0, 1, 0] -> 2,
        [0, 1, 1] -> 2,

        [1, HIGH_Z, HIGH_Z] -> 1,
        [1, HIGH_Z, UNDEFINED] -> 1,
        [1, HIGH_Z, 0] -> 1,
        [1, HIGH_Z, 1] -> 1,
        [1, UNDEFINED, HIGH_Z] -> 1,
        [1, UNDEFINED, UNDEFINED] -> 1,
        [1, UNDEFINED, 0] -> 1,
        [1, UNDEFINED, 1] -> 1,
        [1, 0, HIGH_Z] -> 1,
        [1, 0, UNDEFINED] -> 1,
        [1, 0, 0] -> 1,
        [1, 0, 1] -> 1,
        [1, 1, HIGH_Z] -> 1,
        [1, 1, UNDEFINED] -> 1,
        [1, 1, 0] -> 1,
        [1, 1, 1] -> 1,
    );

    for (i, test_data) in TEST_DATA.iter().enumerate() {
        let mut builder = SimulatorBuilder::default();

        let inputs: Vec<_> = test_data
            .inputs
            .iter()
            .map(|drive| {
                let wire = builder.add_wire(WIDTH_1).unwrap();
                builder.set_wire_drive(wire, drive).unwrap();
                wire
            })
            .collect();
        let output_width = NonZeroU8::new((inputs.len() + 1).clog2() as u8).unwrap();
        let output = builder.add_wire(output_width).unwrap();
        let _decoder = builder.add_priority_decoder(&inputs, output).unwrap();

        let mut sim = builder.build();
        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, output_width),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(output_width),
            output_state.display_string(output_width),
        );
    }
}

#[test]
fn register() {
    let mut builder = SimulatorBuilder::default();

    let data_in = builder.add_wire(WIDTH_32).unwrap();
    let data_out = builder.add_wire(WIDTH_32).unwrap();
    let enable = builder.add_wire(WIDTH_1).unwrap();
    let clock = builder.add_wire(WIDTH_1).unwrap();
    let register = builder
        .add_register(data_in, data_out, enable, clock, ClockPolarity::Rising)
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
        sim.set_wire_drive(data_in, &test_data.data_in).unwrap();
        sim.set_wire_drive(enable, &LogicState::from_bool(test_data.enable))
            .unwrap();
        sim.set_wire_drive(clock, &LogicState::from_bool(test_data.clock))
            .unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data_out).unwrap();

        assert!(
            output_state.eq(&test_data.data_out, WIDTH_32),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.data_out.display_string(WIDTH_32),
            output_state.display_string(WIDTH_32),
        );

        let register_data = sim.get_component_data(register).unwrap();
        let ComponentData::RegisterValue(register_data) = register_data else {
            panic!("[TEST {i}] invalid component data");
        };

        assert!(
            register_data.read().eq(&output_state, WIDTH_32),
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
        WIDTH_4,
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
        WIDTH_5,
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
        WIDTH_4,
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
        WIDTH_5,
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn horizontal_xor_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_0,
        LOGIC_1 -> LOGIC_0,

        0b0000 -> 0,

        0b0001 -> 1,
        0b0010 -> 1,
        0b0100 -> 1,
        0b1000 -> 1,

        0b0011 -> 0,
        0b0101 -> 0,
        0b1001 -> 0,

        0b0111 -> 1,
        0b1011 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_xor_gate,
        WIDTH_4,
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

        0b00011 -> 0,
        0b00101 -> 0,
        0b01001 -> 0,
        0b10001 -> 0,

        0b00111 -> 1,
        0b01011 -> 1,
        0b10011 -> 1,

        0b01111 -> 0,
        0b10111 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_xor_gate,
        WIDTH_5,
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
        WIDTH_4,
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
        WIDTH_5,
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
        WIDTH_4,
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
        WIDTH_5,
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn horizontal_xnor_gate() {
    const TEST_DATA_EVEN: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> UNDEFINED,
        UNDEFINED -> UNDEFINED,
        LOGIC_0 -> LOGIC_1,
        LOGIC_1 -> LOGIC_1,

        0b0000 -> 1,

        0b0001 -> 0,
        0b0010 -> 0,
        0b0100 -> 0,
        0b1000 -> 0,

        0b0011 -> 1,
        0b0101 -> 1,
        0b1001 -> 1,

        0b0111 -> 0,
        0b1011 -> 0,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_xnor_gate,
        WIDTH_4,
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

        0b00011 -> 1,
        0b00101 -> 1,
        0b01001 -> 1,
        0b10001 -> 1,

        0b00111 -> 0,
        0b01011 -> 0,
        0b10011 -> 0,

        0b01111 -> 1,
        0b10111 -> 1,
    );

    test_horizontal_gate(
        SimulatorBuilder::add_horizontal_xnor_gate,
        WIDTH_5,
        TEST_DATA_ODD,
        2,
    );
}

#[test]
fn compare_equal() {
    test_comparator(SimulatorBuilder::add_compare_equal, |a, b| a == b);
}

#[test]
fn compare_not_equal() {
    test_comparator(SimulatorBuilder::add_compare_not_equal, |a, b| a != b);
}

#[test]
fn compare_less_than() {
    test_comparator(SimulatorBuilder::add_compare_less_than, |a, b| a < b);
}

#[test]
fn compare_greater_than() {
    test_comparator(SimulatorBuilder::add_compare_greater_than, |a, b| a > b);
}

#[test]
fn compare_less_than_or_equal() {
    test_comparator(SimulatorBuilder::add_compare_less_than_or_equal, |a, b| {
        a <= b
    });
}

#[test]
fn compare_greater_than_or_equal() {
    test_comparator(
        SimulatorBuilder::add_compare_greater_than_or_equal,
        |a, b| a >= b,
    );
}

#[test]
fn compare_less_than_signed() {
    test_signed_comparator(SimulatorBuilder::add_compare_less_than_signed, |a, b| a < b);
}

#[test]
fn compare_greater_than_signed() {
    test_signed_comparator(SimulatorBuilder::add_compare_greater_than_signed, |a, b| {
        a > b
    });
}

#[test]
fn compare_less_than_or_equal_signed() {
    test_signed_comparator(
        SimulatorBuilder::add_compare_less_than_or_equal_signed,
        |a, b| a <= b,
    );
}

#[test]
fn compare_greater_than_or_equal_signed() {
    test_signed_comparator(
        SimulatorBuilder::add_compare_greater_than_or_equal_signed,
        |a, b| a >= b,
    );
}

#[test]
fn zero_extend() {
    let test_data: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> {% 0, Z },
        UNDEFINED -> {% 0, X },
        0 -> {% 0, 0 },
        1 -> {% 0, 1 },
    );

    let mut builder = SimulatorBuilder::default();

    let input = builder.add_wire(WIDTH_1).unwrap();
    let output = builder.add_wire(WIDTH_2).unwrap();
    let _extend = builder.add_zero_extend(input, output).unwrap();

    let mut sim = builder.build();

    for (i, test_data) in test_data.iter().enumerate() {
        sim.set_wire_drive(input, &test_data.input).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_2),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_2),
            output_state.display_string(WIDTH_2),
        );
    }
}

#[test]
fn sign_extend() {
    let test_data: &[UnaryGateTestData] = unary_gate_test_data!(
        HIGH_Z -> {% Z, Z },
        UNDEFINED -> {% X, X },
        0 -> {% 0, 0 },
        1 -> {% 1, 1 },
    );

    let mut builder = SimulatorBuilder::default();

    let input = builder.add_wire(WIDTH_1).unwrap();
    let output = builder.add_wire(WIDTH_2).unwrap();
    let _extend = builder.add_sign_extend(input, output).unwrap();

    let mut sim = builder.build();

    for (i, test_data) in test_data.iter().enumerate() {
        sim.set_wire_drive(input, &test_data.input).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(output).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_2),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_2),
            output_state.display_string(WIDTH_2),
        );
    }
}

#[test]
fn ram() {
    let mut builder = SimulatorBuilder::default();

    const ADDR_WIDTH: NonZeroU8 = WIDTH_2;
    let write_addr = builder.add_wire(ADDR_WIDTH).unwrap();
    let data_in = builder.add_wire(WIDTH_32).unwrap();
    let read_addr = builder.add_wire(ADDR_WIDTH).unwrap();
    let data_out = builder.add_wire(WIDTH_32).unwrap();
    let write = builder.add_wire(WIDTH_1).unwrap();
    let clock = builder.add_wire(WIDTH_1).unwrap();
    let ram = builder
        .add_ram(
            write_addr,
            data_in,
            read_addr,
            data_out,
            write,
            clock,
            ClockPolarity::Rising,
        )
        .unwrap();

    let mut sim = builder.build();

    struct TestData {
        write_addr: u32,
        data_in: LogicState,
        read_addr: u32,
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
        sim.set_wire_drive(write_addr, &LogicState::from_int(test_data.write_addr))
            .unwrap();
        sim.set_wire_drive(data_in, &test_data.data_in).unwrap();
        sim.set_wire_drive(read_addr, &LogicState::from_int(test_data.read_addr))
            .unwrap();
        sim.set_wire_drive(write, &LogicState::from_bool(test_data.write))
            .unwrap();
        sim.set_wire_drive(clock, &LogicState::from_bool(test_data.clock))
            .unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data_out).unwrap();

        assert!(
            output_state.eq(&test_data.data_out, WIDTH_32),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.data_out.display_string(WIDTH_32),
            output_state.display_string(WIDTH_32),
        );

        let mem_data = sim.get_component_data(ram).unwrap();
        let ComponentData::MemoryBlock(mem_data) = mem_data else {
            panic!("[TEST {i}] invalid component data");
        };

        assert!(
            mem_data
                .read(test_data.read_addr as usize)
                .unwrap()
                .eq(&output_state, WIDTH_32),
            "[TEST {i}] memory data differs from output",
        );
    }
}

#[test]
fn rom() {
    let mut builder = SimulatorBuilder::default();

    const ADDR_WIDTH: NonZeroU8 = WIDTH_2;
    let addr = builder.add_wire(ADDR_WIDTH).unwrap();
    let data = builder.add_wire(WIDTH_32).unwrap();
    let rom = builder.add_rom(addr, data).unwrap();

    let mem_data = builder.get_component_data_mut(rom).unwrap();
    let ComponentData::MemoryBlock(mut mem_data) = mem_data else {
        panic!("[TEST] invalid component data");
    };

    mem_data.write(0, &LogicState::from_int(1)).unwrap();
    mem_data.write(1, &LogicState::from_int(2)).unwrap();
    mem_data.write(2, &LogicState::from_int(3)).unwrap();
    mem_data.write(3, &LogicState::from_int(4)).unwrap();

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
        sim.set_wire_drive(addr, &test_data.input).unwrap();

        match sim.run_sim(2) {
            SimulationRunResult::Ok => {}
            SimulationRunResult::MaxStepsReached => panic!("[TEST {i}] exceeded max steps"),
            SimulationRunResult::Err(err) => panic!("[TEST {i}] {err:?}"),
        }

        let output_state = sim.get_wire_state(data).unwrap();

        assert!(
            output_state.eq(&test_data.output, WIDTH_32),
            "[TEST {i}]  expected: {}  actual: {}",
            test_data.output.display_string(WIDTH_32),
            output_state.display_string(WIDTH_32),
        );
    }
}
*/
