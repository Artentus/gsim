use clap::*;
use directories::ProjectDirs;
use gsim::import::*;
use gsim::*;
use reedline_repl_rs::{Repl, Result};
use std::fmt::Write;
use std::path::PathBuf;

const APP_NAME: &str = "Gsim CLI";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Yosys,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Import format
    #[arg(short, long, required = true, value_name = "FORMAT")]
    format: Format,

    // Import file
    #[arg(short, long, required = true, value_name = "FILE")]
    input: PathBuf,
}

struct Context {
    sim: Simulator,
    ports: ModuleConnections,
}

const DRIVE_INPUT_ARG: &str = "input";
const DRIVE_STATE_ARG: &str = "state";
const EVAL_MAX_STEPS_ARG: &str = "max-steps";

fn main() {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.input).unwrap();

    let mut builder = SimulatorBuilder::default();
    let ports = match args.format {
        Format::Yosys => {
            let importer = gsim::import::yosys::YosysModuleImporter::from_json_str(&json).unwrap();
            builder.import_module(&importer).unwrap()
        }
    };

    let context = Context {
        sim: builder.build(),
        ports,
    };

    let mut repl = Repl::new(context)
        .with_name(APP_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description(env!("CARGO_PKG_DESCRIPTION"))
        .with_stop_on_ctrl_c(true)
        .with_stop_on_ctrl_d(true)
        .with_command(Command::new("list"), list)
        .with_command(
            Command::new("drive")
                .arg(Arg::new(DRIVE_INPUT_ARG).required(true))
                .arg(Arg::new(DRIVE_STATE_ARG).required(true)),
            drive,
        )
        .with_command(
            Command::new("eval").arg(Arg::new(EVAL_MAX_STEPS_ARG).value_parser(value_parser!(u64))),
            eval,
        )
        .with_command(Command::new("quit"), quit);

    if let Some(proj_dirs) = ProjectDirs::from("", "", APP_NAME) {
        repl = repl.with_history(proj_dirs.config_local_dir().to_owned(), 100);
    }

    repl.run().unwrap();
}

#[inline]
const fn const_max<const N: usize>(v: [usize; N]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < v.len() {
        if v[i] > max {
            max = v[i];
        }
        i += 1;
    }
    max
}

fn list(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let mut result = String::new();

    const NAME_HEADER: &str = "Name";
    const KIND_HEADER: &str = "Kind";
    const WIDTH_HEADER: &str = "Width";
    const STATE_HEADER: &str = "State";

    const INPUT_KIND: &str = "input";
    const OUTPUT_KIND: &str = "output";
    const KIND_WIDTH: usize = const_max([KIND_HEADER.len(), INPUT_KIND.len(), OUTPUT_KIND.len()]);

    let mut name_width = NAME_HEADER.len();
    let mut state_width = STATE_HEADER.len();
    for (input_name, &input_wire) in &context.ports.inputs {
        let input_width = context.sim.get_wire_width(input_wire);
        name_width = name_width.max(input_name.chars().count());
        state_width = state_width.max(input_width.get() as usize);
    }
    for (output_name, &output_wire) in &context.ports.outputs {
        let output_width = context.sim.get_wire_width(output_wire);
        name_width = name_width.max(output_name.chars().count());
        state_width = state_width.max(output_width.get() as usize);
    }

    writeln!(
        result,
        "{NAME_HEADER:<name_width$}    {KIND_HEADER:<kind_width$}    {WIDTH_HEADER:<width_width$}    {STATE_HEADER:<state_width$}",
        name_width = name_width,
        kind_width = KIND_WIDTH,
        width_width = WIDTH_HEADER.len(),
        state_width = state_width,
    ).unwrap();

    for (input_name, &input_wire) in &context.ports.inputs {
        let input_width = context.sim.get_wire_width(input_wire);
        let input_state = context.sim.get_wire_drive(input_wire);

        writeln!(
            result,
            "{input_name:<name_width$}    {INPUT_KIND:<kind_width$}    {input_width:<width_width$}    {input_state:<state_width$}",
            input_state = input_state.display_string(input_width),
            name_width = name_width,
            kind_width = KIND_WIDTH,
            width_width = WIDTH_HEADER.len(),
            state_width = state_width,
        ).unwrap();
    }

    for (output_name, &output_wire) in &context.ports.outputs {
        let output_width = context.sim.get_wire_width(output_wire);
        let output_state = context.sim.get_wire_state(output_wire);

        writeln!(
            result,
            "{output_name:<name_width$}    {OUTPUT_KIND:<kind_width$}    {output_width:<width_width$}    {output_state:<state_width$}",
            output_state = output_state.display_string(output_width),
            name_width = name_width,
            kind_width = KIND_WIDTH,
            width_width = WIDTH_HEADER.len(),
            state_width = state_width,
        ).unwrap();
    }

    Ok(Some(result))
}

fn drive(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let input_name: &str = args.get_one::<String>(DRIVE_INPUT_ARG).unwrap();
    let new_state: &str = args.get_one::<String>(DRIVE_STATE_ARG).unwrap();

    let Some(&input_wire) = context.ports.inputs.get(input_name) else {
        println!("Input port '{input_name}' does not exist");
        return Ok(None);
    };

    let new_state = if new_state.starts_with('d') {
        u32::from_str_radix(&new_state[1..], 10)
            .ok()
            .map(LogicState::from_int)
    } else if new_state.starts_with('h') {
        u32::from_str_radix(&new_state[1..], 16)
            .ok()
            .map(LogicState::from_int)
    } else {
        LogicState::parse(new_state)
    };

    let Some(new_state) = new_state else {
        println!("Error parsing new state");
        return Ok(None);
    };

    context.sim.set_wire_drive(input_wire, &new_state);

    let input_width = context.sim.get_wire_width(input_wire);
    let result = format!(
        "Driving input `{input_name}' to '{}'",
        new_state.display_string(input_width)
    );

    Ok(Some(result))
}

fn eval(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    context.sim.run_sim(
        args.try_get_one(EVAL_MAX_STEPS_ARG)
            .unwrap()
            .copied()
            .unwrap_or(10000),
    );

    let mut result = String::new();

    const NAME_HEADER: &str = "Name";
    const STATE_HEADER: &str = "State";

    let mut name_width = NAME_HEADER.len();
    let mut state_width = STATE_HEADER.len();
    for (output_name, &output_wire) in &context.ports.outputs {
        let output_width = context.sim.get_wire_width(output_wire);
        name_width = name_width.max(output_name.chars().count());
        state_width = state_width.max(output_width.get() as usize);
    }

    writeln!(
        result,
        "{NAME_HEADER:<name_width$}    {STATE_HEADER:<state_width$}",
        name_width = name_width,
        state_width = state_width,
    )
    .unwrap();

    for (output_name, &output_wire) in &context.ports.outputs {
        let output_width = context.sim.get_wire_width(output_wire);
        let output_state = context.sim.get_wire_state(output_wire);

        writeln!(
            result,
            "{output_name:<name_width$}    {output_state:<state_width$}",
            output_state = output_state.display_string(output_width),
            name_width = name_width,
            state_width = state_width,
        )
        .unwrap();
    }

    Ok(Some(result))
}

fn quit(_args: ArgMatches, _context: &mut Context) -> Result<Option<String>> {
    std::process::exit(0)
}
