# Press any button to continue

Small crate that gives an easy access to the classic Windows "Press any key to
continue" console prompt.

There is currently no way to implement this using Rust's standard library. The
closest you can get is to read one character from stdin but the user has to
press "ENTER" to do that so you essentially get "Press ENTER to continue..."
doing like this:

```rust
println!("Press ENTER to continue...");
let buffer = &mut [0u8];
std::io::stdin().read_exact(buffer).unwrap();
```

This crate provides only one method called `wait` which progresses on any
keypress.

```rust
fn main() {
    println!("Hello world!");
    press_btn_continue::wait("Press any key to continue...").unwrap();
}
```

## Compatibility

As of now this library only compiles on Windows but I'd be happy to add support
for other platforms as well later on.

## Dependencies

There are no external dependencies. I try to keep this library as lightweight
and transparent as possible (easy to review, and adds very little to compile
times).
