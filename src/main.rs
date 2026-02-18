use anyhow::Result;
use bytesize::ByteSize;
use psutil::{cpu, memory, network};
use std::thread;
use std::time::{Duration, Instant};
use structopt::StructOpt;

// Color palette
static COLOR_NORMAL: &str = "#b0b846";
static COLOR_WARN: &str = "#e9b143";
static COLOR_ERROR: &str = "#f2594b";

// Threshold constants
const THRESHOLD_ERROR: f32 = 90.0;
const THRESHOLD_WARN: f32 = 30.0;

/// System monitor for tmux status line
#[derive(StructOpt, Debug)]
#[structopt(name = "tmux-sysstat")]
struct Opt {
    /// Update interval in seconds
    #[structopt(short, long, default_value = "1")]
    interval: u64,
}

/// Get color based on usage percentage
fn get_color_by_usage(usage: f32) -> &'static str {
    match usage {
        u if u >= THRESHOLD_ERROR => COLOR_ERROR,
        u if u > THRESHOLD_WARN => COLOR_WARN,
        _ => COLOR_NORMAL,
    }
}

fn main() -> Result<()> {
    let args = Opt::from_args();

    let mut cpu_counter = cpu::CpuPercentCollector::new()?;
    let mut net_io_counters_collector = network::NetIoCountersCollector::default();
    let prev_net_io_counters = net_io_counters_collector.net_io_counters()?;

    let start_time = Instant::now();
    thread::sleep(Duration::from_secs(args.interval));
    let elapsed_secs = start_time.elapsed().as_secs().max(1);

    let current_memory_usage = memory::virtual_memory()?.percent();
    let diff = net_io_counters_collector.net_io_counters()? - prev_net_io_counters;

    // Calculate network speeds using actual elapsed time
    let uplink_speed = ByteSize(diff.bytes_sent() / elapsed_secs);
    let downlink_speed = ByteSize(diff.bytes_recv() / elapsed_secs);
    let cpu_percent = cpu_counter.cpu_percent()?;

    // Determine colors based on usage
    let cpu_color = get_color_by_usage(cpu_percent);
    let mem_color = get_color_by_usage(current_memory_usage);

    // Build output string
    let output = format!(
        "#[fg=#45403d]#[default]#[fg=#e2cca9,bg=#45403d] \
         CPU: #[fg={}]{:>.0}%#[fg=default] \
         MEM: #[fg={}]{:.0}%#[fg=default] \
         #[fg=#5a524c]#[default]#[fg=#e2cca9,bg=#5a524c] \
         ↓  #[fg=#b0b846]{}/s#[fg=default] \
         ↑  #[fg=#b0b846]{}/s#[fg=default]",
        cpu_color, cpu_percent, mem_color, current_memory_usage, downlink_speed, uplink_speed
    );

    println!("{}", output);
    Ok(())
}
