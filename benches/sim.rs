use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use gsim::*;
use std::num::NonZeroU8;

fn generate_sim(first: bool) -> Simulator {
    use rand::distributions::Uniform;
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(0);
    let drive_dist = Uniform::new(0, 3);
    let comp_dist = Uniform::new(0, 8);

    let mut builder = SimulatorBuilder::default();
    let mut wires = Vec::new();

    for _ in 0..100 {
        let wire = builder.add_wire(NonZeroU8::MIN).unwrap();
        let drive = match drive_dist.sample(&mut rng) {
            0 => LogicState::HIGH_Z,
            1 => LogicState::LOGIC_0,
            2 => LogicState::LOGIC_1,
            _ => unreachable!(),
        };
        builder.set_wire_drive(wire, &drive).unwrap();
        wires.push(wire);
    }

    for _ in 0..1000000 {
        let output = builder.add_wire(NonZeroU8::MIN).unwrap();
        match comp_dist.sample(&mut rng) {
            0 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_and_gate(&[input_a, input_b], output).unwrap();
            }
            1 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_or_gate(&[input_a, input_b], output).unwrap();
            }
            2 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_xor_gate(&[input_a, input_b], output).unwrap();
            }
            3 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_nand_gate(&[input_a, input_b], output).unwrap();
            }
            4 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_nor_gate(&[input_a, input_b], output).unwrap();
            }
            5 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_xnor_gate(&[input_a, input_b], output).unwrap();
            }
            6 => {
                let input = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_not_gate(input, output).unwrap();
            }
            7 => {
                let input = *wires.choose(&mut rng).unwrap();
                let enable = *wires.choose(&mut rng).unwrap();
                let _id = builder.add_buffer(input, enable, output).unwrap();
            }
            _ => unreachable!(),
        }
        wires.push(output);
    }

    let sim = builder.build();

    if first {
        let stats = sim.stats();

        println!();
        println!();
        println!("Wires: {} ({})", stats.wire_count, stats.wire_alloc_size);
        println!("    Width alloc: {}", stats.wire_width_alloc_size);
        println!("    Drive alloc: {}", stats.wire_drive_alloc_size);
        println!("    State alloc: {}", stats.wire_state_alloc_size);
        println!(
            "Components: {} + {} ({} + {})",
            stats.small_component_count,
            stats.large_component_count,
            stats.component_alloc_size,
            stats.large_component_alloc_size
        );
        println!("    Width alloc: {}", stats.output_width_alloc_size);
        println!("    State alloc: {}", stats.output_state_alloc_size);
        println!(
            "Total memory: {}",
            stats.wire_alloc_size
                + stats.wire_width_alloc_size
                + stats.wire_drive_alloc_size
                + stats.wire_state_alloc_size
                + stats.component_alloc_size
                + stats.large_component_alloc_size
                + stats.output_width_alloc_size
                + stats.output_state_alloc_size
        );
    }

    sim
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut first = true;

    c.bench_function("sim", |b| {
        b.iter_batched(
            || {
                let sim = generate_sim(first);
                first = false;
                sim
            },
            |mut sim| {
                sim.reset();
                let result = sim.run_sim(u64::MAX);
                assert!(matches!(result, SimulationRunResult::Ok));
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
