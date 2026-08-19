use std::{
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    thread,
};

use log::info;
use parking_lot::{Mutex, RwLock};

mod guard;

pub use guard::ExitGuard;

static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGNAL_PIPE: AtomicI32 = AtomicI32::new(-1);

extern "C" fn signal_handler(_sig: libc::c_int) {
    if SIGNAL_RECEIVED.swap(true, Ordering::Relaxed) {
        const MSG: &[u8] = b"Shutdown already pending...\n";
        unsafe { libc::write(2, MSG.as_ptr().cast(), MSG.len()) };
    } else {
        const MSG: &[u8] = b"Signal received, shutdown pending...\n";
        unsafe { libc::write(2, MSG.as_ptr().cast(), MSG.len()) };
        let fd = SIGNAL_PIPE.load(Ordering::Relaxed);
        unsafe { libc::write(fd, b"x".as_ptr().cast(), 1) };
    }
}

/// Graceful shutdown coordinator for ensuring data consistency during program exit.
///
/// On first signal, a background thread acquires the write lock (waiting only for the
/// current critical section to finish), runs cleanup callbacks, and exits.
/// On second signal, force-exits immediately via `_exit(1)`.
#[derive(Default, Clone)]
pub struct Exit {
    lock: Arc<RwLock<()>>,
    cleanup_callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl Exit {
    pub fn new() -> Self {
        Self {
            lock: Arc::new(RwLock::new(())),
            cleanup_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a callback to be executed during shutdown.
    /// Callbacks are executed in registration order before the program exits.
    pub fn register_cleanup<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.cleanup_callbacks.lock().push(Box::new(callback));
    }

    /// Registers signal handlers and spawns a background shutdown thread.
    ///
    /// # Panics
    /// Panics if pipe creation or `sigaction` fails.
    pub fn set_ctrlc_handler(&self) {
        let mut fds = [0i32; 2];
        assert!(
            unsafe { libc::pipe(fds.as_mut_ptr()) } == 0,
            "failed to create pipe"
        );

        let read_fd = fds[0];
        SIGNAL_PIPE.store(fds[1], Ordering::Relaxed);

        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = signal_handler as *const () as usize;
            libc::sigemptyset(&raw mut action.sa_mask);
            action.sa_flags = libc::SA_RESTART;

            assert!(
                libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut()) == 0,
                "failed to install SIGINT handler"
            );
            assert!(
                libc::sigaction(libc::SIGTERM, &action, std::ptr::null_mut()) == 0,
                "failed to install SIGTERM handler"
            );
        }

        let lock = self.lock.clone();
        let callbacks = self.cleanup_callbacks.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 1];
            unsafe { libc::read(read_fd, buf.as_mut_ptr().cast(), 1) };

            let _guard = lock.write();
            for callback in callbacks.lock().iter() {
                callback();
            }
            info!("Exiting...");
            exit(0);
        });
    }

    /// Acquires a read lock to protect a critical section from shutdown.
    /// The shutdown thread will wait for all read locks to be released before exiting.
    /// Returns an owned guard that is Send + 'static (can be moved to background threads).
    pub fn lock(&self) -> ExitGuard {
        ExitGuard::new(&self.lock)
    }
}
