use std::{thread::sleep, time::Duration};

use tracing::info;

pub fn default_retry<T>(function: impl Fn(usize) -> brk_error::Result<T>) -> brk_error::Result<T> {
    retry(function, 5, 6)
}

fn retry<T>(
    function: impl Fn(usize) -> brk_error::Result<T>,
    sleep_in_s: u64,
    retries: usize,
) -> brk_error::Result<T> {
    let mut i = 0;

    loop {
        let res = function(i);

        if res.is_ok() {
            return res;
        }

        // Check if error is permanent (blocked endpoint, DNS failure, etc.)
        // If so, fail immediately without retrying
        if let Err(ref e) = res
            && e.is_network_permanently_blocked()
        {
            info!("Permanent network error detected (blocked/unreachable), skipping retries");
            return res;
        }

        if i == retries {
            return res;
        }

        info!("Failed, waiting {sleep_in_s} seconds...");
        sleep(Duration::from_secs(sleep_in_s));

        i += 1;
    }
}
