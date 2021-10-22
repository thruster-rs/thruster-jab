# Thruster Jab
Because this is a dependency injection library, and shots are injected, and shots are called jabs in the UK.

## What is this?
Thruster Jab is a dependency injection library for your service injection needs. It works using dynamic dispatch via `Box<dyn Any>`. If you're looking to make something that will need mocks in order to test (say, an API that makes external calls, or a service that is very expensive) then Jab might be helpful to you.

## Example
Although built for [thruster](https://github.com/thruster-rs/thruster), Jab can be used independently. A simple example would be:

```rs
struct A(i32);

trait C {}

impl C for A {}

let mut jab = JabDI::default();

let a = A(0);

provide!(jab, dyn C, a);
let something = fetch!(jab, dyn C); // This is the original a that we passed in, now as a C trait.
```

A slightly longer, but more "copy pasta" example (taken from our tests because I'm lazy) would be:

```rs
// Just making two structs that impl two traits
#[derive(Debug, PartialEq)]
struct A(i32);

#[derive(Debug, PartialEq)]
struct B(i32);

trait C {
    fn valc(&self) -> i32;
}
trait D {
    fn vald(&self) -> i32;
}

impl C for A {
    fn valc(&self) -> i32 {
        self.0
    }
}

impl D for B {
    fn vald(&self) -> i32 {
        self.0
    }
}

// This is the good part
let mut jab = JabDI::default();

let a = A(0);
let b = B(1);

provide!(jab, dyn C, a);
provide!(jab, dyn D, b);

assert_eq!(
    0,
    fetch!(jab, dyn C).valc(),
    "it should correctly find struct A for trait C"
);

assert_eq!(
    1,
    fetch!(jab, dyn D).vald(),
    "it should correctly find struct B for trait D"
);
```

For use with actual tests, check out our `hello_world.rs` example.
