use anyhow::Result;
use bytesize::ByteSize;
use psutil::{cpu, memory, network};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

static COLOR_NORMAL: &str = "#b0b846";
static COLOR_WARN: &str = "#e9b143";
static COLOR_ERROR: &str = "#f2594b";
// static COLOR_BG0: &str = "#f2594b";
// static COLOR_FG0: &str = "#f2594b";
// static SEPRATOR: &str = "";

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag. The name of the
    // argument will be, by default, based on the name of the field.
    /// Activate debug mode
    #[structopt(short, long, default_value = "1")]
    interval: u64,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
}
fn main() -> Result<()> {
    let args = Opt::from_args();

    let mut cpu_counter = cpu::CpuPercentCollector::new()?;

    let mut net_io_counters_collector = network::NetIoCountersCollector::default();
    let prev_net_io_counters = net_io_counters_collector.net_io_counters()?;

    thread::sleep(Duration::from_secs(args.interval));
    let current_memory_usage = memory::virtual_memory()?.percent();
    let diff = net_io_counters_collector.net_io_counters()? - prev_net_io_counters;
    let uplink_speed = ByteSize(diff.bytes_sent() / args.interval);
    let downlink_speed = ByteSize(diff.bytes_recv() / args.interval);
    let percent = cpu_counter.cpu_percent()?;

    // prepare for result output
    let mut output_str = String::new();
    let cpu_color = match percent {
        i if i >= 90. => COLOR_ERROR,
        i if i > 30. && i < 90. => COLOR_WARN,
        _ => COLOR_NORMAL,
    };
    let mem_color = match current_memory_usage {
        i if i >= 90. => COLOR_ERROR,
        i if i > 30. && i < 90. => COLOR_WARN,
        _ => COLOR_NORMAL,
    };
    output_str.push_str(&format!(
        "#[fg=#45403d]#[default]#[fg=#e2cca9,bg=#45403d] CPU: #[fg={}]{:>.0}%#[fg=default] MEM: #[fg={}]{:.0}%#[fg=default] ",
        cpu_color, percent, mem_color, current_memory_usage
    ));
    output_str.push_str(&format!(
        "#[fg=#5a524c]#[default]#[fg=#e2cca9,bg=#5a524c]↓  #[fg=#b0b846]{}/s#[fg=default] ↑  #[fg=#b0b846]{}/s#[fg=default]",
        downlink_speed, uplink_speed
    ));
    println!("{}", output_str);
    Ok(())
}
