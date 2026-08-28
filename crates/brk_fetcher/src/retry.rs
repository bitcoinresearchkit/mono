use std::{thread::sleep, time::Duration};

use tracing::warn;

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
            warn!("Request failed with a permanent network error; skipping retries: {e}");
            return res;
        }

        if i == retries {
            return res;
        }

        if let Err(error) = &res {
            warn!("Request failed; retrying in {sleep_in_s}s: {error}");
        }
        sleep(Duration::from_secs(sleep_in_s));

        i += 1;
    }
}
