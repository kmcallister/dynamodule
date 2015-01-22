#![deny(warnings)]
#![allow(unstable)]

#[macro_use] extern crate dynamodule;

use dynamodule::{Class, Instance};

// OOP without vehicle examples is like FP without the Fibonacci sequence.
//
// So here's an interface for vehicles.
trait Vehicle {
    fn go_somewhere(&self) -> String;
}
interface!(Vehicle);

// Cars are boring.  Let's ride bikes!  Specifically, let's define two distinct
// types implementing the Vehicle trait.
macro_rules! vehicle_impl { ($Vehicle:ident) => {
    struct $Vehicle {
        color: &'static str,
    }

    constructor!($Vehicle as Vehicle,
        fn new(color: &'static str) -> ... {
            $Vehicle {
                color: color,
            }
        });
}}
vehicle_impl!(Bicycle);
vehicle_impl!(Motorcycle);

// The two kinds of bikes have different advantages.

impl Vehicle for Bicycle {
    fn go_somewhere(&self) -> String {
        format!("{} bicycle has a basket and a bell that rings", self.color)
    }
}

impl Vehicle for Motorcycle {
    fn go_somewhere(&self) -> String {
        format!("{} motorcycle has 600 cc's of whoop-ass", self.color)
    }
}

#[test]
fn drive_it_like_you_stole_it() {
    let vehicle: Class<Vehicle, _> = Class::of::<Bicycle>();
    let bike: Instance<Vehicle> = vehicle.new("red");

    assert_eq!("red bicycle has a basket and a bell that rings",
        bike.go_somewhere().as_slice());

    // Try out a live upgrade!  It's *really* important that Bicycle and
    // Motorcycle have the same in-memory representation.

    unsafe {
        vehicle.override_methods(&Class::of::<Motorcycle>());
    }

    assert_eq!("red motorcycle has 600 cc's of whoop-ass",
        bike.go_somewhere().as_slice());

    assert_eq!("blue motorcycle has 600 cc's of whoop-ass",
        vehicle.new("blue").go_somewhere().as_slice());
}
