use crate::id_lists::IdInternal;
use crate::SimulatorData;
use std::num::{NonZeroU16, NonZeroU8};

#[derive(Debug, Clone, Copy)]
enum TimescaleUnit {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
}

impl std::fmt::Display for TimescaleUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TimescaleUnit::Seconds => "s",
            TimescaleUnit::Milliseconds => "ms",
            TimescaleUnit::Microseconds => "us",
            TimescaleUnit::Nanoseconds => "ns",
            TimescaleUnit::Picoseconds => "ps",
        };

        f.write_str(s)
    }
}

/// The timescale in a VCD file
#[derive(Debug, Clone, Copy)]
pub struct Timescale {
    unit: TimescaleUnit,
    value: NonZeroU16,
}

impl std::default::Default for Timescale {
    fn default() -> Self {
        Self {
            unit: TimescaleUnit::Nanoseconds,
            value: NonZeroU16::MIN,
        }
    }
}

impl std::fmt::Display for Timescale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

#[allow(missing_docs)]
impl Timescale {
    pub const fn seconds(seconds: NonZeroU16) -> Self {
        Self {
            unit: TimescaleUnit::Seconds,
            value: seconds,
        }
    }

    pub const fn milliseconds(milliseconds: NonZeroU16) -> Self {
        Self {
            unit: TimescaleUnit::Milliseconds,
            value: milliseconds,
        }
    }

    pub const fn microseconds(microseconds: NonZeroU16) -> Self {
        Self {
            unit: TimescaleUnit::Microseconds,
            value: microseconds,
        }
    }

    pub const fn nanoseconds(nanoseconds: NonZeroU16) -> Self {
        Self {
            unit: TimescaleUnit::Nanoseconds,
            value: nanoseconds,
        }
    }

    pub const fn picoseconds(picoseconds: NonZeroU16) -> Self {
        Self {
            unit: TimescaleUnit::Picoseconds,
            value: picoseconds,
        }
    }
}

pub(crate) fn write_vcd_header<VCD: std::io::Write>(
    data: &SimulatorData,
    vcd: &mut VCD,
    timescale: Timescale,
) -> std::io::Result<()> {
    use cow_utils::CowUtils;

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let now = chrono::Local::now().format("%A, %B %e %Y, %X");
    writeln!(vcd, "$version Gsim {VERSION} $end")?;
    writeln!(vcd, "$date {now} $end")?;
    writeln!(vcd, "$timescale {timescale} $end")?;
    writeln!(vcd, "$scope module SIM $end")?;
    for (&wire_id, wire_name) in &data.wire_names {
        let wire_name = wire_name.cow_replace(char::is_whitespace, "_");
        let wire_width = data.get_wire_width(wire_id);
        let ident = wire_id.to_u32();
        if wire_width > NonZeroU8::MIN {
            writeln!(
                vcd,
                "    $var wire {wire_width} W{ident} {wire_name}[{wire_width}] $end",
            )?;
        } else {
            writeln!(vcd, "    $var wire {wire_width} W{ident} {wire_name} $end")?;
        }
    }
    writeln!(vcd, "$upscope $end")?;
    writeln!(vcd, "$enddefinitions $end")?;

    Ok(())
}

pub(crate) fn trace_vcd<VCD: std::io::Write>(
    data: &SimulatorData,
    vcd: &mut VCD,
    time: u64,
) -> std::io::Result<()> {
    writeln!(vcd, "#{time}")?;
    for &wire_id in data.wire_names.keys() {
        let wire_width = data.get_wire_width(wire_id);
        let wire_state = data.get_wire_state(wire_id);
        let ident = wire_id.to_u32();
        if wire_width > NonZeroU8::MIN {
            writeln!(vcd, "b{} W{ident}", wire_state.display_string(wire_width))?;
        } else {
            writeln!(vcd, "{} W{ident}", wire_state.get_bit_state(0))?;
        }
    }

    Ok(())
}
