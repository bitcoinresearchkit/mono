use std::{sync::mpsc, thread, time::Duration};

use super::*;

#[test]
fn pipeline_update_waits_for_readers_and_stays_closed_until_published() {
    let gate = Publication::default();
    let read = gate.try_read().unwrap();
    let writer_gate = gate.clone();
    let (started_tx, started_rx) = mpsc::channel();
    let (closed_tx, closed_rx) = mpsc::channel();

    let writer = thread::spawn(move || {
        started_tx.send(()).unwrap();
        writer_gate.begin_update();
        closed_tx.send(()).unwrap();
        writer_gate
    });

    started_rx.recv().unwrap();
    assert!(closed_rx.try_recv().is_err());

    drop(read);
    closed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let gate = writer.join().unwrap();
    assert!(gate.try_read().is_none());

    gate.finish_update();
    assert!(gate.try_read().is_some());
}

#[test]
fn timed_read_stops_waiting_at_its_deadline() {
    let gate = Publication::default();
    gate.begin_update();

    assert!(gate.read_for(Duration::from_millis(10)).is_none());

    gate.finish_update();
    assert!(gate.read_for(Duration::ZERO).is_some());
}

#[test]
fn timed_read_wakes_when_update_finishes() {
    let gate = Publication::default();
    gate.begin_update();
    let reader_gate = gate.clone();
    let (started_tx, started_rx) = mpsc::channel();

    let reader = thread::spawn(move || {
        started_tx.send(()).unwrap();
        reader_gate.read_for(Duration::from_secs(1))
    });

    started_rx.recv().unwrap();
    thread::sleep(Duration::from_millis(10));
    gate.finish_update();

    assert!(reader.join().unwrap().is_some());
}
