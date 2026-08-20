use std::{collections::BTreeSet, fs, path::Path};

use vecdb::{
    AnyStoredVec, AnyVec, Database, ImportableVec, MutableVec, ReadableVec, Stamp, Version,
    WritableVec, ZeroCopyVec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = fs::remove_dir_all("raw");

    let version = Version::TWO;

    let database = Database::open(Path::new("raw"))?;

    let mut options = (&database, "vec", version).into();

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(1, 2), vec![1]);
        assert_eq!(vec.collect_range(2, 3), vec![2]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        vec.write()?;

        assert!(vec.header().stamp() == Stamp::new(0));
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        vec.mut_header().update_stamp(Stamp::new(100));

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.collect_range(0, 6), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);

        vec.push(21);
        vec.push(22);

        assert_eq!(vec.stored_len(), 21);
        assert_eq!(vec.pushed_len(), 2);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(20, 23), vec![20, 21, 22]);
        assert!(vec.collect_range(23, 24).is_empty());

        vec.write()?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.stored_len(), 23);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 23), vec![20, 21, 22]);

        vec.truncate_if_needed(14)?;

        assert_eq!(vec.stored_len(), 14);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 14);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert!(vec.collect_range(20, 21).is_empty());

        assert_eq!(
            vec.collect_signed_range(Some(-5), None),
            vec![9, 10, 11, 12, 13]
        );

        vec.push(vec.len() as u32);
        let all = vec.collect();
        assert_eq!(*all.last().unwrap(), 14);

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.write()?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        let all = vec.collect();
        assert_eq!(*all.last().unwrap(), 14);

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.reset()?;

        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 0);

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.pushed_len(), 21);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 21);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        let reader = vec.reader();
        assert_eq!(vec.take(10, &reader), Some(10));
        assert_eq!(vec.holes(), &BTreeSet::from([10]));
        assert_eq!(vec.get_with_reader(10, &reader), None);
        drop(reader);

        vec.write()?;

        assert!(vec.holes() == &BTreeSet::from([10]));
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert!(vec.holes() == &BTreeSet::from([10]));

        let reader = vec.reader();
        assert!(vec.get_with_reader(10, &reader).is_none());
        drop(reader);

        vec.update(10, 10)?;
        vec.update(0, 10)?;

        let reader = vec.reader();
        assert_eq!(vec.holes(), &BTreeSet::new());
        assert_eq!(vec.get_with_reader(0, &reader), Some(10));
        assert_eq!(vec.get_with_reader(10, &reader), Some(10));
        drop(reader);

        vec.write()?;
    }

    options = options.with_saved_stamped_changes(10);

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.truncate_if_needed(10)?;

        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4, 6, 7, 8, 9, 21]);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.rollback()?;

        assert_eq!(vec.stamp(), Stamp::new(1));

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.rollback()?;

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write(Stamp::new(0))?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;

        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4, 6, 7, 8, 9, 21]);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        let _ = vec.rollback_before(Stamp::new(1))?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write(Stamp::new(0))?;

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );
    }

    {
        let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;
        dbg!(("0", vec.prev_holes(), vec.updated()));

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.stamp(), Stamp::new(1));

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        vec.stamped_write_with_changes(Stamp::new(2))?;

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        let _ = vec.rollback_before(Stamp::new(1))?;

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
        assert_eq!(vec.stamp(), Stamp::new(0));

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        assert_eq!(vec.stamp(), Stamp::new(0));
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.stamp(), Stamp::new(2));

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        dbg!("-----------------------------------------------------------------------");

        let _ = vec.rollback_before(Stamp::new(1))?;

        dbg!((
            vec.stored_len(),
            vec.real_stored_len(),
            vec.pushed(),
            vec.prev_updated(),
            vec.updated(),
            vec.prev_holes(),
            vec.holes(),
        ));

        assert_eq!(vec.stamp(), Stamp::new(0));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(0))?;

        let vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;
        dbg!(("0", vec.prev_holes(), vec.updated()));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    // {
    //     let mut vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;

    //     dbg!(("0", vec.stamp(), vec.stored_len()));

    //     vec.truncate_if_needed(10)?;
    //     let reader = vec.reader();
    //     vec.take(5, &reader);
    //     vec.update(3, 5)?;
    //     vec.push(21);
    //     drop(reader);

    //     vec.stamped_write_with_changes(Stamp::new(1))?;
    //     assert_eq!(vec.stamp(), Stamp::new(1));

    //     dbg!(("1", vec.stamp(), vec.stored_len()));

    //     let reader = vec.reader();
    //     vec.take(0, &reader);
    //     vec.update(1, 5)?;
    //     vec.push(5);
    //     vec.push(6);
    //     vec.push(7);
    //     drop(reader);

    //     vec.stamped_write_with_changes(Stamp::new(2))?;

    //     assert_eq!(
    //         vec.collect_holed(),
    //         vec![
    //             None,
    //             Some(5),
    //             Some(2),
    //             Some(5),
    //             Some(4),
    //             None,
    //             Some(6),
    //             Some(7),
    //             Some(8),
    //             Some(9),
    //             Some(21),
    //             Some(5),
    //             Some(6),
    //             Some(7)
    //         ]
    //     );

    //     vec.rollback_before(Stamp::new(1))?;
    //     dbg!(("roll", vec.stamp(), vec.stored_len()));
    //     assert_eq!(vec.stamp(), Stamp::new(0));

    //     vec.truncate_if_needed(10)?;
    //     let reader = vec.reader();
    //     vec.take(5, &reader);
    //     vec.update(3, 5)?;
    //     vec.push(21);
    //     drop(reader);

    //     let reader = vec.reader();
    //     vec.take(0, &reader);
    //     vec.update(1, 5)?;
    //     vec.push(5);
    //     vec.push(6);
    //     vec.push(7);
    //     drop(reader);

    //     assert_eq!(
    //         vec.collect_holed(),
    //         vec![
    //             None,
    //             Some(5),
    //             Some(2),
    //             Some(5),
    //             Some(4),
    //             None,
    //             Some(6),
    //             Some(7),
    //             Some(8),
    //             Some(9),
    //             Some(21),
    //             Some(5),
    //             Some(6),
    //             Some(7)
    //         ]
    //     );

    //     dbg!(1);

    //     vec.rollback_before(Stamp::new(1))?;
    //     dbg!(1);

    //     assert_eq!(vec.stamp(), Stamp::new(0));

    //     assert_eq!(
    //         vec.collect(),
    //         vec![
    //             10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    //         ]
    //     );

    //     vec.stamped_write_with_changes(Stamp::new(0))?;

    //     let vec: MutableVec<ZeroCopyVec<usize, u32>> = MutableVec::forced_import_with(options)?;
    //     dbg!(("0", vec.prev_holes(), vec.updated()));

    //     assert_eq!(
    //         vec.collect(),
    //         vec![
    //             10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    //         ]
    //     );
    // }

    Ok(())
}

// fn main() {}
