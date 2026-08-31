use std::{thread, time::Duration};

use brk_exit::Exit;

fn main() {
    let exit = Exit::new();
    exit.register_cleanup(|| {
        eprintln!("[cleanup] flushing data...");
    });
    exit.set_ctrlc_handler();

    eprintln!("Running... press Ctrl+C to test signal handling");
    let mut i = 1_u64;
    loop {
        let _lock = exit.lock();
        eprintln!("  tick {i}");
        thread::sleep(Duration::from_secs(1));
        i = i.wrapping_add(1);
    }
}
