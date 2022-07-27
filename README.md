## SmartLink

SmartLink allows you replace your function definitions with function imports at build time, if a certain environment variable is present.

### Example

With smartlink in your crate manifest:

```toml
[package]
name = "demo-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate_type = [ "dylib" ]

[dependencies]
smartlink = { path = "../smartlink", features = [ "show_output" ] }
```
> The `show_output` feature will make smartlink output the linking code during build.

Then, suppose you have this in `src/lib.rs`:
```rust
use smartlink::smartlink;

pub struct Something {
    pub some_field: u8,
}

#[smartlink]
pub fn example_function<'a>(_unused: Something, some_param: &'a dyn std::io::Write) -> &'a dyn std::io::Write {
    // whatever the content of the function
    some_param
}
```

You can build it and output a shared object:

```sh
$ cargo build
```

(FIXME: need to move the shared object to /usr/lib on my machine so that the linker later finds it)

And eventually, in another crate, you need to call that function:

```toml
[package]
name = "demo-program"
version = "0.1.0"
edition = "2021"

[dependencies]
demo-lib = { path = "../demo-lib" }
```
```rust
// src/main.rs
use demo_lib::Something;
use demo_lib::example_function;

fn main() {
    let something = Something {
        some_field: 9,
    };
    let write_implementor = Vec::new();

    example_function(something, &write_implementor);
}
```

You can build and run that:

```sh
$ cargo run
```

But it wouldn't use the shared object; example_function would be compiled into the binary program.

(FIXME: need to remove the `crate_type = [ "dylib" ]` line in `demo-lib/Cargo.toml` here)

You can use smartlink to redirect the call to the shared object:

```sh
$ RUSTFLAGS="-Clink-args=-ldemo-lib" SMARTLINK_NO_IMPL=demo-lib.so cargo run
```

The new function body is shown in the terminal:

```rust
pub fn example_function < 'a >
(_unused : Something, some_param : & 'a dyn std :: io :: Write) -> & 'a dyn
std :: io :: Write
{
    #[link(name = "1", kind = "dylib")] extern "Rust"
    {
        fn example_function_fake_mangling < 'a >
        (_unused : Something, some_param : & 'a dyn std :: io :: Write) -> &
        'a dyn std :: io :: Write ;
    } unsafe { example_function_fake_mangling(_unused, some_param,) }
}
```

> This generated code looks messed up but it is valid.

Then, if the linker can find demo-lib.so, it will end up linking to it.
At runtime, its implementation of `example_function` will be called.
