# BUGS

- The transform commands are also changing the file extensions. For examples, `smv title` should change `cargo.toml` into `Cargo.toml` but it produces `CARGO.TOML` instead.
- The title case command is using the `.` in filenames as the delimiter on where to transform. For example, `smv title` should change `CARGO.TOML` into `Cargo.TOML` but it produces `CARGO.toml` instead.