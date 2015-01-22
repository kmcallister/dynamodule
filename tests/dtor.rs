#![deny(warnings)]
#![allow(unstable)]

#[macro_use] extern crate dynamodule;

use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use dynamodule::{Class, Instance};

trait Thing { }
interface!(Thing);

static COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

struct Foo(usize);
impl Thing for Foo { }

constructor!(Foo as Thing,
    fn new(amount: usize) -> ... {
        Foo(amount)
    }
);

impl Drop for Foo {
    fn drop(&mut self) {
        COUNT.fetch_add(self.0, Ordering::SeqCst);
    }
}

struct Nothing(usize);
impl Thing for Nothing { }

constructor!(Nothing as Thing,
    fn new(_amount: usize) -> ... {
        Nothing(0)
    }
);

impl Drop for Nothing {
    fn drop(&mut self) {
        // nothing! absolutely nothing!
    }
}

#[test]
fn destroy() {
    let cls: Class<Thing, _> = Class::of::<Foo>();

    let get_count = |&:| COUNT.load(Ordering::SeqCst);

    assert_eq!(get_count(), 0);

    {
        let ins: Instance<Thing> = cls.new(7);
        let _ = ins;
    }

    assert_eq!(get_count(), 7);

    unsafe {
        cls.override_methods(&Class::of::<Nothing>());
    }

    {
        let ins: Instance<Thing> = cls.new(4);
        let _ = ins;
    }

    // Destructor overridden.
    assert_eq!(get_count(), 7);
}
