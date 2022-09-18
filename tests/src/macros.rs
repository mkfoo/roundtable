#![allow(non_snake_case)]
use roundtable::prelude::*;
use std::io::{Cursor, Seek};

#[test]
fn primitives() {
    roundtable::datapoint! {
        struct Foo {
            b: i8,
            B: u8,
            h: i16,
            H: u16,
            i: i32,
            I: u32,
            l: i64,
            L: u64,
            q: i128,
            Q: u128,
            f: f32,
            d: f64,
        }
    }

    const ORIG: Foo = Foo {
        b: 1,
        B: 1,
        h: 2,
        H: 2,
        i: 4,
        I: 4,
        l: 8,
        L: 8,
        q: 16,
        Q: 16,
        f: 4.0,
        d: 8.0,
    };

    let mut buf = Cursor::new(vec![]);
    let size = ORIG.get_size();
    assert_eq!(size, 74);
    ORIG.write_out(&mut buf).unwrap();
    buf.rewind().unwrap();
    let mut new = Foo::default();
    new.read_in(&mut buf).unwrap();
    assert_eq!(ORIG, new);
    assert_eq!(ORIG.get_hash(), new.get_hash());
}

#[test]
fn arrays() {
    roundtable::datapoint! {
        struct Foo {
            a: [i32; 6],
            b: [f32; 2],
            c: [u32; 8],
            d: [i8; 4],
            e: [u8; 4],
        }
    }

    const ORIG: Foo = Foo {
        a: [6i32; 6],
        b: [2f32; 2],
        c: [8u32; 8],
        d: [4i8; 4],
        e: [4u8; 4],
    };

    let mut buf = Cursor::new(vec![]);
    let size = ORIG.get_size();
    assert_eq!(size, 72);
    ORIG.write_out(&mut buf).unwrap();
    buf.rewind().unwrap();
    let mut new = Foo::default();
    new.read_in(&mut buf).unwrap();
    assert_eq!(ORIG, new);
    assert_eq!(ORIG.get_hash(), new.get_hash());
}

#[test]
fn multiple_structs() {
    roundtable::datapoint! {
        struct Foo {
            a: i32,
            b: f32,
        }

        struct Bar {
            a: i64,
            b: f64,
        }

        struct Baz {
            a: i128,
            b: u128,
        }
    }

    const FOO: Foo = Foo { a: 1, b: 2.0 };
    const BAR: Bar = Bar { a: 1, b: 2.0 };
    const BAZ: Baz = Baz { a: 1, b: 2 };
    assert_eq!(FOO.get_size(), 8);
    assert_eq!(BAR.get_size(), 16);
    assert_eq!(BAZ.get_size(), 32);
}

#[test]
fn nested_structs() {
    roundtable::datapoint! {
        struct Foo {
            a: i32,
        }

        struct Bar {
            a: Foo,
            b: u32,
        }

        struct Baz {
            a: Bar,
            b: f32,
        }

        struct Qux {
            a: Baz,
            b: Baz,
        }
    }

    const QUX: Qux = Qux {
        a: Baz {
            a: Bar {
                a: Foo { a: 1 },
                b: 1,
            },
            b: 1.0,
        },
        b: Baz {
            a: Bar {
                a: Foo { a: 1 },
                b: 1,
            },
            b: 1.0,
        },
    };

    assert_eq!(QUX.get_size(), 24);
    let mut buf = Cursor::new(vec![]);
    QUX.write_out(&mut buf).unwrap();
    buf.rewind().unwrap();
    let mut new = Qux::default();
    new.read_in(&mut buf).unwrap();
    assert_eq!(QUX, new);
    assert_eq!(QUX.get_hash(), new.get_hash());
}
