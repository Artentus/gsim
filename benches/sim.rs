use criterion::{Criterion, criterion_group, criterion_main};
use gsim::*;

fn generate_sim() -> Simulator {
    use rand::distributions::Uniform;
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(0);
    let drive_dist = Uniform::new(0, 3);
    let comp_dist = Uniform::new(0, 8);

    let mut builder = SimulatorBuilder::default();
    let mut wires = Vec::new();

    for _ in 0..100 {
        let wire = builder.add_wire(BitWidth::MIN).unwrap();
        let drive = match drive_dist.sample(&mut rng) {
            0 => LogicState::high_z(BitWidth::MIN),
            1 => LogicState::logic_0(BitWidth::MIN),
            2 => LogicState::logic_1(BitWidth::MIN),
            _ => unreachable!(),
        };
        builder.set_wire_drive(wire, &drive).unwrap();
        wires.push(wire);
    }

    for _ in 0..1000000 {
        let output = builder.add_wire(BitWidth::MIN).unwrap();
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

    builder.build()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut sim = generate_sim();

    let stats = sim.stats();
    println!();
    println!();
    println!("Wires: {} ({})", stats.wire_count, stats.wire_alloc_size);
    println!("    State alloc: {}", stats.wire_state_alloc_size);
    println!(
        "Components: {} ({})",
        stats.component_count, stats.component_alloc_size,
    );
    println!("    State alloc: {}", stats.output_state_alloc_size);
    println!("Total memory: {}", stats.total_alloc_size());

    c.bench_function("sim", |b| {
        b.iter(|| {
            sim.reset();
            let result = sim.run_sim(u64::MAX);
            assert!(matches!(result, SimulationRunResult::Ok));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
