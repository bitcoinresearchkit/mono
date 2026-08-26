use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use fjall::Database;

#[test]
fn db_open() -> fjall::Result<()> {
    let folder = tempfile::tempdir()?;

    {
        let _db = Database::builder(&folder).open()?;
    }

    // DB should not be locked
    {
        let _db = Database::builder(&folder).open()?;
    }

    Ok(())
}

#[test]
fn db_open_with_keyspace() -> fjall::Result<()> {
    let folder = tempfile::tempdir()?;

    {
        let db = Database::builder(&folder).open()?;
        let _keyspace = db.keyspace("hello", Default::default)?;
    }

    // DB should not be locked
    {
        let _db = Database::builder(&folder).open()?;
    }

    Ok(())
}

#[test]
fn different_keyspace_names_open_concurrently() -> fjall::Result<()> {
    const THREADS: usize = 8;

    let folder = tempfile::tempdir()?;
    let database = Database::builder(&folder).open()?;
    let barrier = Arc::new(Barrier::new(THREADS));
    let active = AtomicUsize::new(0);
    let max_active = AtomicUsize::new(0);

    thread::scope(|scope| -> fjall::Result<()> {
        let handles = (0..THREADS)
            .map(|index| {
                let database = database.clone();
                let barrier = Arc::clone(&barrier);
                let active = &active;
                let max_active = &max_active;
                scope.spawn(move || {
                    barrier.wait();
                    database.keyspace(&format!("keyspace_{index}"), || {
                        let current = active.fetch_add(1, Ordering::Relaxed) + 1;
                        max_active.fetch_max(current, Ordering::Relaxed);
                        thread::sleep(Duration::from_millis(10));
                        active.fetch_sub(1, Ordering::Relaxed);
                        Default::default()
                    })
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap()?;
        }
        Ok(())
    })?;

    assert!(max_active.load(Ordering::Relaxed) > 1);
    Ok(())
}

#[test]
fn concurrent_same_name_opens_once() -> fjall::Result<()> {
    const THREADS: usize = 8;

    let folder = tempfile::tempdir()?;
    let database = Database::builder(&folder).open()?;
    let barrier = Arc::new(Barrier::new(THREADS));
    let opens = AtomicUsize::new(0);

    thread::scope(|scope| -> fjall::Result<()> {
        let handles = (0..THREADS)
            .map(|_| {
                let database = database.clone();
                let barrier = Arc::clone(&barrier);
                let opens = &opens;
                scope.spawn(move || {
                    barrier.wait();
                    database.keyspace("shared", || {
                        opens.fetch_add(1, Ordering::Relaxed);
                        thread::sleep(Duration::from_millis(10));
                        Default::default()
                    })
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap()?;
        }
        Ok(())
    })?;

    assert_eq!(opens.load(Ordering::Relaxed), 1);
    Ok(())
}
