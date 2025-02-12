# `tagid-derive` - Procedural macro for deriving `Label` in the `tagid` crate

The `tagid-derive` crate provides a procedural macro to derive the `Label` trait from the `tagid` crate. 
This simplifies the implementation of label-based type identification.

## Installation
Add the following to your `Cargo.toml`:

```toml
[dependencies]
tagid = "0.2"
tagid-derive = "0.2"
```

## Usage
### Deriving `Label`

The `Label` trait provides a compile-time string label for a type. 
To derive it, simply annotate your struct or enum with `#[derive(Label)]`:

```rust
use tagid::Label;

#[derive(Label)]
struct Order;

assert_eq!(Order::label(), "Order");
```

Now, `Order::label()` will return a unique string label for the type:

This is useful for logging, serialization, or any scenario where type-based labeling is needed.

### License
This project is licensed under the MIT License. See LICENSE for details.
