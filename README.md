# dynamodule

dynamodule is an experimental dynamic OOP framework for Rust.

The current focus is run-time code updates, but this library also serves as a
playground for dynamic OOP techniques generally.  Suggestions and pull requests
are more than welcome!

This is for research use only!  There are probably lots of ways to screw up and
corrupt memory badly.  This code really stresses Rust's type / trait system,
and will often ICE the compiler or worse.

## Overview

Traits define interfaces.

```rust
trait Vehicle {
    fn go_somewhere(&self) -> String;
}
interface!(Vehicle);
```

Trait impls, paired with a constructor, are the basis for classes.

```rust
#[derive(Copy)]
struct Bicycle(&'static str);

constructor!(Bicycle as Vehicle,
    fn new(color: &'static str) -> ... {
        Bicycle(color)
    }
);

impl Vehicle for Bicycle {
    fn go_somewhere(&self) -> String {
        format!("{} bicycle has a basket and a bell that rings", self.color)
    }
}
```

A class is created at runtime, based on some specific `impl`.  The class can
then be instantiated.

```rust
let vehicle: Class<Vehicle, _> = Class::of::<Bicycle>();
let bike: Instance<Vehicle> = vehicle.new("red");
assert_eq!(bike.go_somewhere().as_slice(),
    "red bicycle has a basket and a bell that rings");
```

The point of this indirection is that we can switch out the methods at runtime,
modifying the behavior of existing objects.  The new methods can come from a
dynamically loaded library.

```rust
struct Motorcycle { ... }

impl Vehicle for Motorcycle { ... }

#[no_mangle]
pub unsafe fn install_plugin(cls: &Class<Vehicle, &'static str>) {
    cls.override_methods(&Class::of::<Motorcycle>());
}
```

We compile this as a dylib crate, then load it on demand.

```rust
let path = Path::new("motorcycle.so");
let lib = DynamicLibrary::open(Some(&path)).unwrap();
let install: unsafe fn(&Class<Vehicle, &'static str>)
    = lib.symbol("install_plugin").unwrap();
install(&vehicle);

assert_eq!(bike.go_somewhere().as_slice(),
    "red motorcycle has 600 cc's of whoop-ass");
```

That's the theory, anyway!  Here it's *really* important that `Bicycle` and
`Motorcycle` have the same in-memory representation.  Future work will explore
forward-compatible `self` types.

## Virtual destructors

Instance types can have destructors:

```rust
trait Thing { }
interface!(Thing);

static COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

struct Foo; impl Thing for Foo { }

constructor!(Foo as Thing,
    fn new(_x: ()) -> ... { Foo }
);

impl Drop for Foo {
    fn drop(&mut self) {
        COUNT.fetch_add(1, Ordering::SeqCst);
    }
}
```

dynamodule supports these as "virtual destructors" with dynamic dispatch:

```rust
let cls: Class<Thing, _> = Class::of::<Foo>();

assert_eq!(COUNT.load(Ordering::SeqCst), 0);

{
    let ins: Instance<Thing> = cls.new(());
    let _ = ins;
}

assert_eq!(COUNT.load(Ordering::SeqCst), 1);
```

Actually, this bypasses Rust's built-in virtual destructors for boxed trait
objects, which have some issues.  It also means we can override the destructor:

```rust
unsafe {
    cls.override_methods(&Class::of::<Nothing>());
}

{
    let ins: Instance<Thing> = cls.new(());
    let _ = ins;
}

assert_eq!(COUNT.load(Ordering::SeqCst), 1);
```

## Other examples

I don't have a plugin example working yet, but you can see a single-crate
example in `tests/vehicle.rs`.
