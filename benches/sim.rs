use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use gsim::*;

fn generate_sim() -> Simulator {
    use rand::distributions::Uniform;
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(0);
    let drive_dist = Uniform::new(0, 3);
    let comp_dist = Uniform::new(0, 8);

    let mut sim = Simulator::new();
    let mut wires = Vec::new();

    for _ in 0..100 {
        let wire = sim.add_wire(LogicWidth::MIN);
        let drive = match drive_dist.sample(&mut rng) {
            0 => LogicState::HIGH_Z,
            1 => LogicState::LOGIC_0,
            2 => LogicState::LOGIC_1,
            _ => unreachable!(),
        };
        sim.set_wire_base_drive(wire, drive);
        wires.push(wire);
    }

    for _ in 0..1000000 {
        let output = sim.add_wire(LogicWidth::MIN);
        match comp_dist.sample(&mut rng) {
            0 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_and_gate(input_a, input_b, output).unwrap();
            }
            1 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_or_gate(input_a, input_b, output).unwrap();
            }
            2 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_xor_gate(input_a, input_b, output).unwrap();
            }
            3 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_nand_gate(input_a, input_b, output).unwrap();
            }
            4 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_nor_gate(input_a, input_b, output).unwrap();
            }
            5 => {
                let input_a = *wires.choose(&mut rng).unwrap();
                let input_b = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_xnor_gate(input_a, input_b, output).unwrap();
            }
            6 => {
                let input = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_not_gate(input, output).unwrap();
            }
            7 => {
                let input = *wires.choose(&mut rng).unwrap();
                let enable = *wires.choose(&mut rng).unwrap();
                let _id = sim.add_buffer(input, enable, output).unwrap();
            }
            _ => unreachable!(),
        }
        wires.push(output);
    }

    sim
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sim", |b| {
        b.iter_batched(
            generate_sim,
            |mut sim| {
                let result = sim.run_sim(u64::MAX);
                assert!(matches!(result, SimulationRunResult::Ok));
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
