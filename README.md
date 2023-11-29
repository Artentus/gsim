[<img alt="crates.io" src="https://img.shields.io/crates/v/gsim?style=for-the-badge&logo=rust" height="20">](https://crates.io/crates/gsim)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/gsim?style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/gsim)
<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Artentus/gsim/tests.yml?branch=master&style=for-the-badge" height="20">

# About

Gsim is a digital circuit simulation engine optimized for high simulation speed.  
This repository contains the simulation backend in form of a Rust library, a circuit editor using this engine can be found at https://github.com/Artentus/gsim-gui.  
The library also implements a C-API as well a a Python module.


# Usage Example

```rust
use gsim::*;
use std::num::NonZeroU8;

pub fn main() {
    let mut builder = SimulatorBuilder::default();

    // Add wires and components to the simulation
    let wire_width = NonZeroU8::new(1).unwrap();
    let input_a = builder.add_wire(wire_width).unwrap();
    let input_b = builder.add_wire(wire_width).unwrap();
    let output = builder.add_wire(wire_width).unwrap();
    // The gate ID is not usefull to us because we don't intend on reading its data
    let _gate = builder.add_and_gate(&[input_a, input_b], output).unwrap();

    // Create the simulation
    let mut sim = builder.build();

    // Manually drive the input wires
    sim.set_wire_drive(input_a, &LogicState::from_bool(true)).unwrap();
    sim.set_wire_drive(input_b, &LogicState::from_bool(false)).unwrap();

    // Run the simulation
    const MAX_STEPS: u64 = 2;
    match sim.run_sim(MAX_STEPS) {
        SimulationRunResult::Ok => {}
        SimulationRunResult::MaxStepsReached => panic!("simulation did not settle within allowed steps"),
        SimulationRunResult::Err(err) => panic!("simulation error: {err:?}"),
    }

    // Make sure we got the expected result
    let output_state = sim.get_wire_state(output).unwrap();
    assert!(output_state.eq(&LogicState::from_bool(false), wire_width));
}
```


# Contributing

Contributions are always welcome, but please follow these steps before submitting a PR:

- Run `cargo fmt` using the default Rust formatting style
- Run `cargo clippy` and make sure there are no warnings in your code (warnings that existed before are ok)
- Run `cargo test` to make sure you didn't break anything
- Run `cargo bench` before and after to ensure your changes didn't cause a performance regression
- Consider writing a test if applicable to your change (e.g. you added a new component type)
