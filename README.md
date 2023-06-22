# About

Gsim is a work-in-progress digital circuit simulation engine optimized for high simulation speed.  
This repository contains the simulation backend in form of a Rust library, a circuit editor using this engine can be found at https://github.com/Artentus/gsim-gui.


# Usage Example

```rust
use gsim::*;

pub fn main() {
    let mut builder = SimulatorBuilder::default();

    // Add wires and components to the simulation
    let wire_width = LogicWidth::new(1).unwrap();
    let input_a = builder.add_wire(wire_width);
    let input_b = builder.add_wire(wire_width);
    let output = builder.add_wire(wire_width);
    // The gate ID is not usefull to us because we don't intend on reading its data
    let _gate = builder.add_and_gate(input_a, input_b, output).unwrap();

    // Create the simulation
    let mut sim = builder.build();

    // Manually drive the input wires
    sim.set_wire_base_drive(input_a, LogicState::from_bool(true));
    sim.set_wire_base_drive(input_b, LogicState::from_bool(false));

    // Run the simulation
    const MAX_STEPS: u64 = 2;
    match sim.run_sim(MAX_STEPS) {
        SimulationRunResult::Ok => {}
        SimulationRunResult::MaxStepsReached => panic!("simulation did not settle within allowed steps"),
        SimulationRunResult::Err(err) => panic!("simulation error: {err:?}"),
    }

    // Make sure we got the expected result
    let output_state = sim.get_wire_state(output);
    assert!(output_state.eq(LogicState::from_bool(false), wire_width));
}
```


# Contributing

Contributions are always welcome, but please follow these steps before submitting a PR:

- Run `cargo fmt` using the default Rust formatting style
- Run `cargo clippy` and make sure there are no warnings in your code (warnings that existed before are ok)
- Run `cargo test` to make sure you didn't break anything
- Run `cargo bench` before and after to ensure your changes didn't cause a performance regression
- Consider writing a test if applicable to your change (e.g. you added a new component type)
