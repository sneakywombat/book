use aya::{include_bytes_aligned, programs::Lsm, Bpf, Btf};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use structopt::StructOpt;
use log::{info,error};

fn main() {
    env_logger::init();
    if let Err(e) = try_main() {
        error!("error: {:#}", e);
    }
}

#[derive(Debug, StructOpt)]
struct Opt {}

fn try_main() -> Result<(), anyhow::Error> {
    // command-line options a currently unused
    let _opt = Opt::from_args();

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/myapp"
    ))?;
    let btf = Btf::from_sys_fs()?;
    let program: &mut Lsm =
        bpf.program_mut("task_alloc").unwrap().try_into()?;
    program.load("task_alloc", &btf)?;
    program.attach()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    info!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(500))
    }
    info!("Exiting...");

    Ok(())
}
