//! Graceful process shutdown coordination for BRK.

use std::{
    mem,
    process::exit,
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    thread,
};

use libc::{
    SA_RESTART, SIGINT, SIGTERM, c_int, pipe, read, sigaction, sigemptyset, write as LibcWrite,
};
use log::info;
use parking_lot::{Mutex, RwLock};

mod guard;

pub use guard::ExitGuard;

static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGNAL_PIPE: AtomicI32 = AtomicI32::new(-1);

type CleanupCallback = Box<dyn Fn() + Send + Sync>;

extern "C" fn signal_handler(_sig: c_int) {
    if SIGNAL_RECEIVED.swap(true, Ordering::Relaxed) {
        const MSG: &[u8] = b"Shutdown already pending...\n";
        unsafe { LibcWrite(2, MSG.as_ptr().cast(), MSG.len()) };
    } else {
        const MSG: &[u8] = b"Signal received, shutdown pending...\n";
        unsafe { LibcWrite(2, MSG.as_ptr().cast(), MSG.len()) };
        let fd = SIGNAL_PIPE.load(Ordering::Relaxed);
        unsafe { LibcWrite(fd, b"x".as_ptr().cast(), 1) };
    }
}

/// Graceful shutdown coordinator for ensuring data consistency during program exit.
///
/// On first signal, a background thread acquires the write lock (waiting only for the
/// current critical section to finish), runs cleanup callbacks, and exits.
/// On subsequent signals, reports that shutdown is already pending and continues waiting.
#[derive(Default, Clone)]
pub struct Exit {
    lock: Arc<RwLock<()>>,
    cleanup_callbacks: Arc<Mutex<Vec<CleanupCallback>>>,
}

impl Exit {
    /// Creates a shutdown coordinator without installing signal handlers.
    pub fn new() -> Self {
        Self::default()
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
    /// Panics if called more than once per process, or if pipe creation or `sigaction` fails.
    pub fn set_ctrlc_handler(&self) {
        assert!(
            !HANDLER_INSTALLED.swap(true, Ordering::Relaxed),
            "the ctrl-c handler is already installed"
        );

        let mut fds = [0i32; 2];
        assert!(
            unsafe { pipe(fds.as_mut_ptr()) } == 0,
            "failed to create pipe"
        );

        let read_fd = fds[0];
        SIGNAL_PIPE.store(fds[1], Ordering::Relaxed);

        unsafe {
            let mut action: sigaction = mem::zeroed();
            action.sa_sigaction = signal_handler as *const () as usize;
            sigemptyset(&raw mut action.sa_mask);
            action.sa_flags = SA_RESTART;

            assert!(
                sigaction(SIGINT, &action, ptr::null_mut()) == 0,
                "failed to install SIGINT handler"
            );
            assert!(
                sigaction(SIGTERM, &action, ptr::null_mut()) == 0,
                "failed to install SIGTERM handler"
            );
        }

        let lock = self.lock.clone();
        let callbacks = self.cleanup_callbacks.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 1];
            unsafe { read(read_fd, buf.as_mut_ptr().cast(), 1) };

            let _guard = lock.write();
            for callback in callbacks.lock().iter() {
                callback();
            }
            info!("Exiting...");
            exit(0);
        });
    }
}
