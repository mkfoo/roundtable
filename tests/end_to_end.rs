use roundtable as rt;
use rt::prelude::*;
use std::collections::BTreeMap;

#[test]
fn in_memory() {
    let test_data = BTreeMap::from([
        (47030060, 10000001),
        (47030189, 10000002),
        (47030318, 10000003),
        (47030447, 10000004),
        (47030576, 10000005),
        (47030705, 10000006),
        (47030834, 10000007),
        (47030963, 10000008),
        (47031092, 10000009),
    ]);
    let t_start = 47029931_u64;
    let t_step = 129_u64;
    let t_total = 1161_u64;
    let opts = Options::new(t_start, t_step, t_total);
    let mut tab = rt::create::in_memory(opts, 10000000_i64).unwrap();

    for (t, v) in test_data.iter() {
        tab.insert(*t, v).unwrap();
    }

    let buf = tab.into_inner().into_inner();
    let mut tab2 = rt::load::from_buffer(opts, buf).unwrap();

    assert_eq!((47030060, &10000001), tab2.first().unwrap());
    assert_eq!((47031092, &10000009), tab2.last().unwrap());

    for (t, v) in test_data.iter() {
        assert_eq!(v, tab2.get(*t).unwrap());
    }
}

#[test]
fn in_file() {
    let test_data = BTreeMap::from([
        (47030060, 10000001),
        (47030189, 10000002),
        (47030318, 10000003),
        (47030447, 10000004),
        (47030576, 10000005),
        (47030705, 10000006),
        (47030834, 10000007),
        (47030963, 10000008),
        (47031092, 10000009),
    ]);
    let t_start = 47029931_u64;
    let t_step = 129_u64;
    let t_total = 3888_u64;
    let opts = Options::new(t_start, t_step, t_total).overwrite(true);

    let mut tab = rt::create::in_file(opts, 10000000_i32, "test.rtdb").unwrap();

    for (t, v) in test_data.iter() {
        tab.insert(*t, v).unwrap();
    }

    let file = tab.into_inner();
    file.sync_all().unwrap();
    std::mem::drop(file);
    let mut tab2 = rt::load::from_file(opts, "test.rtdb").unwrap();

    assert_eq!((47029931, &10000000), tab2.first().unwrap());
    assert_eq!((47031092, &10000009), tab2.last().unwrap());

    for (t, v) in test_data.iter() {
        assert_eq!(v, tab2.get(*t).unwrap());
    }
}
