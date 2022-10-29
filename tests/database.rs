use roundtable::error::Error;
use roundtable::prelude::*;
use roundtable::rtdb::{Header, Table};
use std::io::Cursor;

#[test]
fn header_validation() {
    roundtable::datapoint! {
        struct Foo {
            a: i32,
            b: i32,
        }
    }

    let foo = Foo::default();
    let opts = Options::new(0, 1, 5);
    let h = Header::new(&opts, &foo);
    let mut buf = Cursor::new(vec![]);
    h.write_out(&mut buf).unwrap();
    let mut new = Header::default();
    buf.rewind().unwrap();
    new.read_in(&mut buf).unwrap();
    assert_eq!(h, new);
    new.validate(&opts, &foo).unwrap();
}

#[test]
fn partial_load() {
    let buf = Cursor::new(vec![]);
    let opts = Options::new(0, 100, 12000);
    let mut t = Table::new(&opts, &0_i16, buf).unwrap();
    t.insert(100, &1000).unwrap();
    t.insert(200, &2000).unwrap();
    t.insert(300, &3000).unwrap();
    assert_eq!(t.get(100).unwrap(), &1000);
    assert_eq!(t.get(200).unwrap(), &2000);
    assert_eq!(t.get(300).unwrap(), &3000);
    let buf2 = t.into_inner();
    let mut t2 = Table::load(&opts, &0_i16, buf2).unwrap();
    assert_eq!(t2.get(100).unwrap(), &1000);
    assert_eq!(t2.get(200).unwrap(), &2000);
    assert_eq!(t2.get(300).unwrap(), &3000);
    let mut v2 = t2.into_inner().into_inner();
    v2.pop().unwrap();
    let buf3 = Cursor::new(v2);
    assert_eq!(
        Table::load(&opts, &0_i16, buf3).unwrap_err(),
        Error::InvalidStreamLen
    );
}

#[test]
fn full_load() {
    let mut v: Vec<u8> = vec![];
    v.resize(40, 0_u8);
    let buf = Cursor::new(v);
    let opts = Options::new(0, 100, 1000);
    let mut t = Table::new(&opts, &0_i32, buf).unwrap();
    t.insert(100, &10000).unwrap();
    t.insert(200, &20000).unwrap();
    t.insert(300, &30000).unwrap();
    t.insert(400, &40000).unwrap();
    t.insert(500, &50000).unwrap();
    t.insert(600, &60000).unwrap();
    t.insert(700, &70000).unwrap();
    t.insert(800, &80000).unwrap();
    t.insert(900, &90000).unwrap();
    t.insert(1000, &100000).unwrap();
    assert_eq!(t.get(1000).unwrap(), &100000);
    assert_eq!(t.get(200).unwrap(), &20000);
    assert_eq!(t.get(300).unwrap(), &30000);
    let buf2 = t.into_inner();
    let mut t2 = Table::load(&opts, &0_i32, buf2).unwrap();
    assert_eq!(t2.get(1000).unwrap(), &100000);
    assert_eq!(t2.get(200).unwrap(), &20000);
    assert_eq!(t2.get(300).unwrap(), &30000);
    let mut v2 = t2.into_inner().into_inner();
    v2.pop().unwrap();
    let buf3 = Cursor::new(v2);
    assert_eq!(
        Table::load(&opts, &0_i32, buf3).unwrap_err(),
        Error::InvalidStreamLen
    );
}

#[test]
fn time_errors() {
    let t_start = 1000;
    let t_step = 100;
    let t_total = 1000;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Zeroed)
        .max_fwd_skip(3);
    let mut t = Table::new(&opts, &1_i64, buf).unwrap();
    assert_eq!(t.insert(999, &1).unwrap_err(), Error::UpdateTooEarly);
    assert_eq!(t.insert(9999, &1).unwrap_err(), Error::UpdateTooLate);
    t.insert(1100, &100).unwrap();
    t.insert(1200, &200).unwrap();
    t.insert(1400, &400).unwrap();
    assert_eq!(t.insert(1900, &900).unwrap_err(), Error::MaxSkipExceeded);
    t.insert(1800, &800).unwrap();
    t.insert(2100, &210).unwrap();
    println!("lsdj");
    assert_eq!(t.get(999).unwrap_err(), Error::OutOfRangePast);
    assert_eq!(t.get(1100).unwrap_err(), Error::OutOfRangePast);
    assert_eq!(t.get(1199).unwrap_err(), Error::OutOfRangePast);
    assert_eq!(t.get(1200).unwrap(), &200);
    assert_eq!(t.get(2100).unwrap(), &210);
    assert_eq!(t.get(2101).unwrap_err(), Error::OutOfRangeFuture);
}

#[test]
fn boundary_values() {
    let t_start = 0;
    let t_step = 100;
    let t_total = 1000;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Zeroed)
        .max_fwd_skip(7);
    let mut t = Table::new(&opts, &0_i64, buf).unwrap();
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
    let t_start = 100;
    let t_step = 10;
    let t_total = 500;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .max_fwd_skip(8)
        .fwd_skip_mode(FwdSkipMode::Zeroed);
    let mut t = Table::new(&opts, &0_i32, buf).unwrap();
    t.insert(110, &1).unwrap();
    t.insert(120, &2).unwrap();
    t.insert(150, &5).unwrap();
    t.insert(190, &9).unwrap();
    t.insert(240, &14).unwrap();
    assert_eq!(t.get(100).unwrap(), &0);
    assert_eq!(t.get(110).unwrap(), &1);
    assert_eq!(t.get(120).unwrap(), &2);
    assert_eq!(t.get(130).unwrap(), &0);
    assert_eq!(t.get(140).unwrap(), &0);
    assert_eq!(t.get(150).unwrap(), &5);
    assert_eq!(t.get(160).unwrap(), &0);
    assert_eq!(t.get(170).unwrap(), &0);
    assert_eq!(t.get(180).unwrap(), &0);
    assert_eq!(t.get(190).unwrap(), &9);
    assert_eq!(t.get(200).unwrap(), &0);
    assert_eq!(t.get(210).unwrap(), &0);
    assert_eq!(t.get(220).unwrap(), &0);
    assert_eq!(t.get(230).unwrap(), &0);
    assert_eq!(t.get(240).unwrap(), &14);
}

#[test]
fn nearest() {
    let t_start = 0;
    let t_step = 10;
    let t_total = 160;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Nearest)
        .max_fwd_skip(8);
    let mut t = Table::new(&opts, &0_i32, buf).unwrap();
    t.insert(10, &1).unwrap();
    t.insert(30, &3).unwrap();
    t.insert(60, &6).unwrap();
    t.insert(100, &10).unwrap();
    t.insert(140, &14).unwrap();
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
    assert_eq!(t.get(120).unwrap(), &14);
    assert_eq!(t.get(130).unwrap(), &14);
    assert_eq!(t.get(140).unwrap(), &14);
}

#[test]
fn lerp_int() {
    let t_start = 0;
    let t_step = 10;
    let t_total = 160;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Linear)
        .max_fwd_skip(8);
    let mut t = Table::new(&opts, &0_i32, buf).unwrap();
    t.insert(10, &10).unwrap();
    t.insert(40, &40).unwrap();
    t.insert(80, &60).unwrap();
    assert_eq!(t.get(10).unwrap(), &10);
    assert_eq!(t.get(20).unwrap(), &20);
    assert_eq!(t.get(30).unwrap(), &30);
    assert_eq!(t.get(40).unwrap(), &40);
    assert_eq!(t.get(50).unwrap(), &45);
    assert_eq!(t.get(60).unwrap(), &50);
    assert_eq!(t.get(70).unwrap(), &55);
    assert_eq!(t.get(80).unwrap(), &60);
}

#[test]
fn lerp_float() {
    let t_start = 0;
    let t_step = 10;
    let t_total = 160;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Linear)
        .max_fwd_skip(8);
    let mut t = Table::new(&opts, &0.0, buf).unwrap();
    t.insert(40, &1.0).unwrap();
    t.insert(80, &3.0).unwrap();
    assert_eq!(t.get(0).unwrap(), &0.0);
    assert_eq!(t.get(10).unwrap(), &0.25);
    assert_eq!(t.get(20).unwrap(), &0.50);
    assert_eq!(t.get(30).unwrap(), &0.75);
    assert_eq!(t.get(40).unwrap(), &1.0);
    assert_eq!(t.get(50).unwrap(), &1.5);
    assert_eq!(t.get(60).unwrap(), &2.0);
    assert_eq!(t.get(70).unwrap(), &2.5);
    assert_eq!(t.get(80).unwrap(), &3.0);
}

#[test]
fn lerp_array() {
    let t_start = 0;
    let t_step = 10;
    let t_total = 160;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Linear)
        .max_fwd_skip(8);
    let mut t = Table::new(&opts, &[1.0, 2.0, 3.0], buf).unwrap();
    t.insert(40, &[3.0, 4.0, 6.0]).unwrap();
    assert_eq!(t.get(0).unwrap(), &[1.0, 2.0, 3.0]);
    assert_eq!(t.get(10).unwrap(), &[1.5, 2.5, 3.75]);
    assert_eq!(t.get(20).unwrap(), &[2.0, 3.0, 4.5]);
    assert_eq!(t.get(30).unwrap(), &[2.5, 3.5, 5.25]);
    assert_eq!(t.get(40).unwrap(), &[3.0, 4.0, 6.0]);
}

#[test]
fn lerp_struct() {
    roundtable::datapoint! {
        struct Foo {
            a: i32,
            b: [f32; 2],
        }
    }
    let t_start = 0;
    let t_step = 10;
    let t_total = 160;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .fwd_skip_mode(FwdSkipMode::Linear)
        .max_fwd_skip(8);
    let mut t = Table::new(&opts, &Foo::default(), buf).unwrap();
    t.insert(
        40,
        &Foo {
            a: 4,
            b: [2.0, 3.0],
        },
    )
    .unwrap();
    assert_eq!(
        t.get(0).unwrap(),
        &Foo {
            a: 0,
            b: [0.0, 0.0]
        }
    );
    assert_eq!(
        t.get(10).unwrap(),
        &Foo {
            a: 1,
            b: [0.5, 0.75]
        }
    );
    assert_eq!(
        t.get(20).unwrap(),
        &Foo {
            a: 2,
            b: [1.0, 1.5]
        }
    );
    assert_eq!(
        t.get(30).unwrap(),
        &Foo {
            a: 3,
            b: [1.5, 2.25]
        }
    );
    assert_eq!(
        t.get(40).unwrap(),
        &Foo {
            a: 4,
            b: [2.0, 3.0]
        }
    );
}

#[test]
fn first_and_last() {
    let t_start = 42498729;
    let t_step = 193;
    let t_total = 1930;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total)
        .max_fwd_skip(4)
        .fwd_skip_mode(FwdSkipMode::Zeroed);
    let mut t = Table::new(&opts, &0_i32, buf).unwrap();
    t.insert(t_start + 193, &1).unwrap();
    t.insert(t_start + 386, &2).unwrap();
    t.insert(t_start + 579, &3).unwrap();
    assert_eq!(t.first().unwrap(), (t_start, &0));
    assert_eq!(t.last().unwrap(), (t_start + 579, &3));
    t.insert(t_start + 1678, &8).unwrap();
    t.insert(t_start + 1737, &9).unwrap();
    t.insert(t_start + 1930, &10).unwrap();
    assert_eq!(t.first().unwrap(), (t_start + 193, &1));
    assert_eq!(t.last().unwrap(), (t_start + 1930, &10));
    assert_eq!(t.get(t_start + 195).unwrap(), &1);
    assert_eq!(t.get(t_start + 579).unwrap(), &3);
    assert_eq!(t.get(t_start + 1678).unwrap(), &8);
    assert_eq!(t.get(t_start + 1737).unwrap(), &9);
    assert_eq!(t.get(t_start + 1930).unwrap(), &10);
    t.insert(t_start + 2123, &11).unwrap();
    assert_eq!(t.first().unwrap(), (t_start + 386, &2));
    assert_eq!(t.last().unwrap(), (t_start + 2123, &11));
}

#[test]
fn iter() {
    let t_start = 1665232907;
    let t_step = 119;
    let t_total = 11900;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total);
    let mut tab = Table::new(&opts, &0_u64, buf).unwrap();

    for i in 1..105_u64 {
        tab.insert(t_start + t_step * i, &i).unwrap();
    }

    let mut j = 5_u64;

    for (t, v) in tab.iter().unwrap() {
        assert_eq!(t, t_start + t_step * j);
        assert_eq!(v, j);
        j += 1;
    }
}

#[test]
fn range() {
    let t_start = 1665232907;
    let t_step = 119;
    let t_total = 11900;
    let buf = Cursor::new(vec![]);
    let opts = Options::new(t_start, t_step, t_total);
    let mut tab = Table::new(&opts, &0_u64, buf).unwrap();

    for i in 1..105_u64 {
        tab.insert(t_start + t_step * i, &i).unwrap();
    }

    let mut j = 20_u64;

    for (t, v) in tab.range(1665235287, 1665239333).unwrap() {
        assert_eq!(t, t_start + t_step * j);
        assert_eq!(v, j);
        j += 1;
    }
}
