# `tagid` - Typed Unique Identifiers for Rust Entities

[![Crates.io](https://img.shields.io/crates/v/tagid.svg)](https://crates.io/crates/tagid)
[![Docs.rs](https://docs.rs/tagid/badge.svg)](https://docs.rs/tagid)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`tagid` provides a robust system for defining and managing typed unique identifiers in Rust.  
It supports multiple ID generation strategies (CUID, UUID, Snowflake) and integrates seamlessly with  
`serde`, `sqlx`, and other frameworks.

## Features

- **Typed Identifiers**: Define entity-specific IDs with compile-time safety.
- **Multiple ID Generators**:
    - **CUID** (`cuid` feature) - Compact, collision-resistant IDs.
    - **UUID** (`uuid` feature) - Universally unique identifiers.
    - **Snowflake** (`snowflake` feature) - Time-based, distributed IDs.
- **Entity Labeling**: Labels provide contextual meaning to identifiers.
- **Serialization & Database Support**:
    - [`serde`] integration for JSON and binary serialization (`serde` feature).
    - [`sqlx`] integration for database storage (`sqlx` feature).
- **Custom Labeling**: Define custom label formats for entities, which can be useful to disambiguate
  ids in logging.

## Installation

Add `tagid` to your `Cargo.toml`, enabling the desired features:

```toml
[dependencies]
tagid = { version = "0.3.1", features = ["uuid", "serde", "sqlx"] }
```

## Optional Features

| Feature          | Description                                                 |
|------------------|-------------------------------------------------------------|
| `"derive"`       | Enables `#[derive(Label)]` macro for automatic labeling.    |
| `"cuid"`         | Enables the [`CuidGenerator`] for CUID-based IDs.           |
| `"uuid"`         | Enables the [`UuidGenerator`] for UUID-based IDs.           |
| `"snowflake"`    | Enables the [`SnowflakeGenerator`] for distributed IDs.     |
| `"serde"`        | Enables serialization support via `serde`.                  |
| `"sqlx"`         | Enables database integration via `sqlx`.                    |
| `"disintegrate"` | Enables tagid identifiers in `disintegrate`.                |
| `"envelope"`     | Provides an envelope struct for wrapping IDs with metadata. |

## Usage

### Defining an Entity with a Typed ID

```rust
use tagid::{Entity, Id, Label};

#[derive(Label)]
struct User;

impl Entity for User {
    type IdGen = tagid::UuidGenerator;
}

fn main() {
    let user_id = User::next_id();
    println!("User ID: {}", user_id);
}
```

## Labeling System

Labels help associate an identifier with an entity, improving clarity in logs and databases. The `Label` trait provides a way to define a unique label for each entity type.

```rust, ignore
use tagid::{Label, Labeling};

#[derive(Label)]
struct Order;

let order_label = Order::labeler().label();
assert_eq!(order_label, "Order");
```

This ensures that IDs are self-descriptive when displayed, stored, or logged.

## Working with Different ID Types
### Using CUIDs

Enable the `cuid` feature in `Cargo.toml`:

```toml
[dependencies]
tagid = { version = "0.2", features = ["cuid"] }
```

Example usage:

```rust
use tagid::{Entity, Id, Label, CuidGenerator};

#[derive(Label)]
struct Session;

impl Entity for Session {
    type IdGen = CuidGenerator;
}

fn main() {
    let session_id = Session::next_id();
    println!("Session ID: {}", session_id);
}
```

### Using UUIDs

Enable the `uuid` feature:

```toml
[dependencies]
tagid = { version = "0.2", features = ["uuid"] }
```

Example usage:

```rust
use tagid::{Entity, Id, Label, UuidGenerator};

#[derive(Label)]
struct User;

impl Entity for User {
    type IdGen = UuidGenerator;
}

fn main() {
    let user_id = User::next_id();
    println!("User ID: {}", user_id);
}
```

### Using Snowflake IDs

Enable the `snowflake` feature:

```toml
[dependencies]
tagid = { version = "0.2", features = ["snowflake"] }
```

Example usage:

```rust
use tagid::{Entity, Id, Label, snowflake::SnowflakeGenerator};

#[derive(Label)]
struct LogEntry;

impl Entity for LogEntry {
    type IdGen = SnowflakeGenerator;
}

fn main() {
    let log_id = LogEntry::next_id();
    println!("Log ID: {}", log_id);
}
```

## Serialization & Database Integration

### JSON Serialization with `serde`

Enable the `serde` feature in `Cargo.toml`

```toml
[dependencies]
tagid = { version = "0.2", features = ["serde", "derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
```

Then serialize an ID:

```rust
use tagid::{Entity, Id, Label};
use serde::{Serialize, Deserialize};

#[derive(Label, Serialize, Deserialize)]
struct Product;

impl Entity for Product {
    type IdGen = tagid::UuidGenerator;
}

fn main() {
    let product_id = Product::next_id();
    let serialized = serde_json::to_string(&product_id).unwrap();
    println!("Serialized ID: {}", serialized);
}
```

### SQL Database Integration with `sqlx`

Enable `sqlx` support in `Cargo.toml`:

```toml
[dependencies]
tagid = { version = "0.2", features = ["sqlx", "derive"] }
sqlx = { version = "0.7", features = ["postgres"] }
```

Then use `Is<T, ID> in a database model:

```rust
use tagid::{Entity, Id, Label};
use sqlx::FromRow;

#[derive(Label)]
struct Customer;

impl Entity for Customer {
    type IdGen = tagid::UuidGenerator;
}

#[derive(FromRow)]
struct CustomerRecord {
    id: Id<Customer, uuid::Uuid>,
    name: String,
}
```

## Benchmarking

To measure the performance of difference ID generators, run:

```shell
cargo bench
```
## Contributing
Contributions are welcome! Open an issue or submit a pull request on GitHub.

## License
This project is licensed under the MIT License. See the LICENSE file for details.
