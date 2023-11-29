Gsim is a digital circuit simulation engine optimized for high simulation speed.


### Usage Example

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
