use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use psutil::{cpu, memory, network};

static COLOR_NORMAL: &str = "#b0b846";
static COLOR_WARN: &str = "#e9b143";
static COLOR_ERROR: &str = "#f2594b";

const THRESHOLD_ERROR: f32 = 90.0;
const THRESHOLD_WARN: f32 = 30.0;

/// System monitor for tmux status line
#[derive(Parser, Debug)]
#[command(name = "tmux-sysstat")]
struct Opt {
    /// Update interval in seconds
    #[arg(short, long, default_value = "1")]
    interval: u64,

    /// Run continuously, outputting stats every <interval> seconds
    #[arg(short, long)]
    daemon: bool,
}

fn get_color_by_usage(usage: f32) -> &'static str {
    match usage {
        u if u >= THRESHOLD_ERROR => COLOR_ERROR,
        u if u > THRESHOLD_WARN => COLOR_WARN,
        _ => COLOR_NORMAL,
    }
}

fn collect_and_print(
    cpu_counter: &mut cpu::CpuPercentCollector,
    net_collector: &mut network::NetIoCountersCollector,
    prev_net: &mut network::NetIoCounters,
    interval: u64,
) -> Result<()> {
    let start_time = Instant::now();
    thread::sleep(Duration::from_secs(interval));
    let elapsed_secs = start_time.elapsed().as_secs().max(1);

    let mem_usage = memory::virtual_memory()?.percent();
    let cur_net = net_collector.net_io_counters()?;

    let diff_sent = cur_net.bytes_sent().saturating_sub(prev_net.bytes_sent());
    let diff_recv = cur_net.bytes_recv().saturating_sub(prev_net.bytes_recv());
    *prev_net = cur_net;

    let uplink_speed = ByteSize(diff_sent / elapsed_secs);
    let downlink_speed = ByteSize(diff_recv / elapsed_secs);
    let cpu_percent = cpu_counter.cpu_percent()?;

    let cpu_color = get_color_by_usage(cpu_percent);
    let mem_color = get_color_by_usage(mem_usage);

    println!(
        "#[fg=#45403d]#[default]#[fg=#e2cca9,bg=#45403d] \
         CPU: #[fg={}]{:>.0}%#[fg=default] \
         MEM: #[fg={}]{:.0}%#[fg=default] \
         #[fg=#5a524c]#[default]#[fg=#e2cca9,bg=#5a524c] \
         \u{2193}  #[fg=#b0b846]{}/s#[fg=default] \
         \u{2191}  #[fg=#b0b846]{}/s#[fg=default]",
        cpu_color, cpu_percent, mem_color, mem_usage, downlink_speed, uplink_speed
    );

    std::io::stdout().flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Opt::parse();

    let mut cpu_counter = cpu::CpuPercentCollector::new()?;
    let mut net_collector = network::NetIoCountersCollector::default();
    let mut prev_net = net_collector.net_io_counters()?;

    if args.daemon {
        loop {
            collect_and_print(&mut cpu_counter, &mut net_collector, &mut prev_net, args.interval)?;
        }
    } else {
        collect_and_print(&mut cpu_counter, &mut net_collector, &mut prev_net, args.interval)?;
    }
    Ok(())
}
