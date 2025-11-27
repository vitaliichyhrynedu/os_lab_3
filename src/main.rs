use os_lab_3::{
    kernel::{AccessResult, Kernel},
    memory::FRAME_COUNT,
    paging::{Clock, PageReplacementPolicy, Random, Vpn},
    process::{Pid, Process, Request},
};

struct SimConfig {
    process_count: usize,
    pages_per_process: usize,
    working_set_size: usize,
    duration: usize,
}

const CONFIG: SimConfig = SimConfig {
    process_count: 16,
    pages_per_process: 16_384,
    working_set_size: 512,
    duration: 100_000,
};

#[derive(Default)]
struct SimStats {
    policy_name: String,
    accesses: u64,
    hits: u64,
    misses: u64,
    swap_outs: u64,
    swap_ins: u64,
}

impl SimStats {
    fn hit_rate(&self) -> f64 {
        if self.accesses == 0 {
            0.0
        } else {
            (self.hits as f64 / self.accesses as f64) * 100.0
        }
    }

    fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }
}

fn main() {
    print_header();

    let stats_random = run_simulation(Random, "Random");
    print_report(&stats_random);

    let stats_clock = run_simulation(Clock::new(), "Clock");
    print_report(&stats_clock);
}

fn run_simulation<P: PageReplacementPolicy>(policy: P, name: &str) -> SimStats {
    println!("Running `{}` policy simulation...", name);

    let mut kernel = Kernel::new(policy);

    for i in 0..CONFIG.process_count {
        let process = Process::new(
            Pid(i),
            CONFIG.pages_per_process,
            CONFIG.working_set_size,
            CONFIG.duration * 2,
            1_024,
        );
        kernel.pm.spawn_process(process);
    }

    let mut stats = SimStats {
        policy_name: name.to_string(),
        ..Default::default()
    };

    for _ in 0..CONFIG.duration {
        for pid_idx in 0..kernel.pm.process_count() {
            let pid = Pid(pid_idx);

            let request = kernel
                .pm
                .get_mut_process(pid)
                .expect("Process scheduled but not found")
                .request();

            match request {
                Request::MemoryReference { vpn, operation } => {
                    stats.accesses += 1;
                    match kernel.access_memory(pid, Vpn(vpn), operation) {
                        AccessResult::Hit => stats.hits += 1,
                        AccessResult::Miss => stats.misses += 1,
                    }
                }
                Request::Termination => {
                    //
                }
            }
        }
    }

    stats.swap_outs = kernel.mm.stats.swap_out_count as u64;
    stats.swap_ins = kernel.mm.stats.swap_in_count as u64;

    stats
}

fn print_header() {
    println!("# Operating Systems: Lab 3\n");
    print_row_header("## Conditions");
    print_row("Frame count", &FRAME_COUNT);
    print_row("Process count", &CONFIG.process_count);
    print_row("Pages per process", &CONFIG.pages_per_process);
    print_row("Working set size", &CONFIG.working_set_size);
    print_row("Simulation duration", &CONFIG.duration);
    println!();
}

fn print_report(stats: &SimStats) {
    print_row_header(&format!("## Stats for the `{}` policy", stats.policy_name));
    print_row("Memory accesses", &stats.accesses);
    print_row("Page hits", &stats.hits);
    print_row("Page faults", &stats.misses);
    print_row("Swap outs", &stats.swap_outs);
    print_row("Swap ins", &stats.swap_ins);
    print_row("Hit rate", &format!("{:.2}%", stats.hit_rate()));
    print_row("Miss rate", &format!("{:.2}%", stats.miss_rate()));
    println!();
}

fn print_row_header(title: &str) {
    println!("{}", title);
    println!("| {:<20} | {:<20} |", "Metric", "Value");
    println!("| {:-<20} | {:-<20} |", "-", "-");
}

fn print_row(label: &str, value: &dyn std::fmt::Display) {
    println!("| {:<20} | {:<20} |", label, value);
}
