use roundtable::prelude::*;
use roundtable::rtdb::{Header, Table};
use std::io::{Cursor, Seek};

#[test]
fn header_validation() {
    roundtable::datapoint! {
        struct Foo {
            a: i32,
            b: i32,
        }
    }

    let foo = Foo::default();
    let h = Header::new(&foo, 10, 2000, 30);
    let mut buf = Cursor::new(vec![]);
    h.write_out(&mut buf).unwrap();
    let mut new = Header::default();
    buf.rewind().unwrap();
    new.read_in(&mut buf).unwrap();
    assert_eq!(h, new);
    new.validate(&foo).unwrap();
}

#[test]
fn time_errors() {
    let dp_count = 10;
    let t_start = 1000;
    let t_step = 100;
    let mut buf = Cursor::new(vec![]);
    let opts = Options::default()
        .fwd_skip_mode(FwdSkipMode::Zeroed)
        .max_fwd_skip(3);
    let mut t = Table::new(opts, dp_count, t_start, t_step, &1_i64, buf).unwrap();
    assert_eq!(t.insert(999, &1).unwrap_err(), Error::UpdateTooEarly);
    assert_eq!(t.insert(9999, &1).unwrap_err(), Error::UpdateTooLate);
    t.insert(1100, &100).unwrap();
    t.insert(1200, &200).unwrap();
    t.insert(1400, &400).unwrap();
    assert_eq!(t.insert(1900, &900).unwrap_err(), Error::UpdateTooLate);
    t.insert(1800, &800).unwrap();
    t.insert(2100, &210).unwrap();
    assert_eq!(t.get(999).unwrap_err(), Error::OutOfRangePast);
    assert_eq!(t.get(1100).unwrap_err(), Error::OutOfRangePast);
    assert_eq!(t.get(2100).unwrap(), &210);
    assert_eq!(t.get(2101).unwrap_err(), Error::OutOfRangeFuture);
}

#[test]
fn boundary_values() {
    let dp_count = 10;
    let t_start = 0;
    let t_step = 100;
    let mut buf = Cursor::new(vec![]);
    let opts = Options::default()
        .fwd_skip_mode(FwdSkipMode::Zeroed)
        .max_fwd_skip(7);
    let mut t = Table::new(opts, dp_count, t_start, t_step, &0_i64, buf).unwrap();
    t.insert(150, &1).unwrap();
    t.insert(250, &2).unwrap();
    t.insert(350, &3).unwrap();
    assert_eq!(t.get(0).unwrap(), &0);
    assert_eq!(t.get(99).unwrap(), &0);
    assert_eq!(t.get(100).unwrap(), &1);
    assert_eq!(t.get(199).unwrap(), &1);
    assert_eq!(t.get(200).unwrap(), &2);
    assert_eq!(t.get(299).unwrap(), &2);
    assert_eq!(t.get(300).unwrap(), &3);
    t.insert(950, &9).unwrap();
    t.insert(1050, &10).unwrap();
    t.insert(1100, &11).unwrap();
    assert_eq!(t.get(900).unwrap(), &9);
    assert_eq!(t.get(999).unwrap(), &9);
    assert_eq!(t.get(1000).unwrap(), &10);
    assert_eq!(t.get(1099).unwrap(), &10);
}

#[test]
fn zeroed() {
    let dp_count = 10;
    let t_start = 100;
    let t_step = 10;
    let mut buf = Cursor::new(vec![]);
    let opts = Options::default().fwd_skip_mode(FwdSkipMode::Zeroed);
    let mut t = Table::new(opts, dp_count, t_start, t_step, &0_i32, buf).unwrap();
    t.insert(110, &1).unwrap();
    t.insert(120, &2).unwrap();
    t.insert(150, &5).unwrap();
    assert_eq!(t.get(100).unwrap(), &0);
    assert_eq!(t.get(110).unwrap(), &1);
    assert_eq!(t.get(120).unwrap(), &2);
    assert_eq!(t.get(130).unwrap(), &0);
    assert_eq!(t.get(140).unwrap(), &0);
    assert_eq!(t.get(150).unwrap(), &5);
}

#[test]
fn nearest() {
    let dp_count = 30;
    let t_start = 0;
    let t_step = 10;
    let mut buf = Cursor::new(vec![]);
    let opts = Options::default()
        .fwd_skip_mode(FwdSkipMode::Nearest)
        .max_fwd_skip(8);
    let mut t = Table::new(opts, dp_count, t_start, t_step, &0_i32, buf).unwrap();
    t.insert(10, &1).unwrap();
    t.insert(30, &3).unwrap();
    t.insert(60, &6).unwrap();
    t.insert(100, &10).unwrap();
    t.insert(150, &15).unwrap();
    assert_eq!(t.get(10).unwrap(), &1);
    assert_eq!(t.get(20).unwrap(), &3);
    assert_eq!(t.get(30).unwrap(), &3);
    assert_eq!(t.get(40).unwrap(), &3);
    assert_eq!(t.get(50).unwrap(), &6);
    assert_eq!(t.get(60).unwrap(), &6);
    assert_eq!(t.get(70).unwrap(), &6);
    assert_eq!(t.get(80).unwrap(), &10);
    assert_eq!(t.get(90).unwrap(), &10);
    assert_eq!(t.get(100).unwrap(), &10);
    assert_eq!(t.get(110).unwrap(), &10);
    assert_eq!(t.get(120).unwrap(), &10);
    assert_eq!(t.get(130).unwrap(), &15);
    assert_eq!(t.get(140).unwrap(), &15);
    assert_eq!(t.get(150).unwrap(), &15);
}
