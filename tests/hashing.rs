use roundtable::prelude::*;

#[test]
fn arrays_and_structs_are_the_same() {
    roundtable::datapoint! {
        struct Foo {
            a: u8,
            b: u8,
            c: u8,
            d: u8,
            e: u8,
            f: u8,
            g: u8,
            h: u8,
        }

        struct Bar {
            a: u8,
            b: u8,
            c: [u8; 4],
            g: u8,
            h: u8,

        }

        struct Baz {
            a: [u8; 2],
            b: [u8; 2],
            c: [u8; 2],
            d: [u8; 2],
        }
    }

    const QUX: [u8; 8] = [0; 8];
    let foo: Foo = Foo::default();
    let bar: Bar = Bar::default();
    let baz: Baz = Baz::default();
    assert_eq!(QUX.get_hash(), foo.get_hash());
    assert_eq!(QUX.get_hash(), bar.get_hash());
    assert_eq!(QUX.get_hash(), baz.get_hash());
}

#[test]
fn different_arrays_are_different() {
    const FOO: [u8; 12] = [0; 12];
    const BAR: [u8; 180] = [0; 180];
    const BAZ: [u8; 181] = [0; 181];
    const QUX: [i32; 3] = [0; 3];
    assert_ne!(QUX.get_hash(), FOO.get_hash());
    assert_ne!(QUX.get_hash(), BAR.get_hash());
    assert_ne!(QUX.get_hash(), BAZ.get_hash());
    assert_ne!(FOO.get_hash(), BAR.get_hash());
    assert_ne!(FOO.get_hash(), BAZ.get_hash());
    assert_ne!(BAR.get_hash(), BAZ.get_hash());
}

#[test]
fn different_structs_are_different() {
    roundtable::datapoint! {
        struct Foo {
            a: u8,
            b: u8,
            c: u8,
            d: u8,
            e: u8,
            f: u8,
            g: u8,
            h: u8,
            i: u8,
            j: u8,
            k: u8,
        }

        struct Bar {
            a: i8,
            b: u8,
            c: u8,
            d: u8,
            e: u8,
            f: u8,
            g: u8,
            h: u8,
            i: u8,
            j: u8,
            k: u8,
        }

        struct Baz {
            a: u8,
            b: u8,
            c: u8,
            d: u8,
            e: u8,
            f: u8,
            g: f32,
            h: u8,
            i: u8,
            j: u8,
            k: u8,
        }

    }

    let foo: Foo = Foo::default();
    let bar: Bar = Bar::default();
    let baz: Baz = Baz::default();
    assert_ne!(foo.get_hash(), bar.get_hash());
    assert_ne!(bar.get_hash(), baz.get_hash());
    assert_ne!(baz.get_hash(), foo.get_hash());
}
