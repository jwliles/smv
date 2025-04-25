# SMV Build Issues and Workarounds

## Procedural Macro Compilation Issue

### Problem Description

When building SMV with nightly Rust, you may encounter the following error:

```
error: cannot produce proc-macro for `clap_derive v3.2.25` as the target `x86_64-unknown-linux-gnu` does not support these crate types
```

This error occurs when trying to compile the `clap_derive` crate, which is a procedural macro dependency of the `clap` crate used for command-line argument parsing. 

The issue arises despite the following conditions:
- Using a platform (x86_64-unknown-linux-gnu) that fully supports procedural macros
- Having the necessary Rust components installed (rust-std, rustc, cargo, etc.)
- Using a recent version of Rust (1.85+, nightly)

### Root Cause Analysis

This problem appears to be related to an unusual interaction between:
1. The use of Rust 2024 edition with older Rust toolchains
2. The proc-macro compilation process
3. Potentially the host environment configuration

The project was originally developed with Rust 2024 edition (released in February 2025). When attempting to build with Rust 1.65.0 or even newer versions that don't fully support all 2024 edition features, proc-macro compilation fails.

The exact cause involves:
- Edition incompatibility: Rust 2024 edition features being used in proc-macro code
- Toolchain limitations: Older Rust toolchains not supporting newer proc-macro features
- Target constraints: The `x86_64-unknown-linux-gnu` target potentially having limitations with proc-macros on certain toolchain versions

### Workaround Implemented

To work around this issue, we've modified the project to:
1. Restore the Rust 2024 edition in Cargo.toml (the original edition)
2. Switch from clap's derive-based API to its builder-based API
3. Keep using the nightly Rust toolchain
4. Maintain the same functionality while avoiding proc-macro dependencies

This approach provides compatibility with the 2024 edition features while circumventing the proc-macro compilation issues. It eliminates the need for procedural macros during compilation while preserving all the command-line argument parsing capabilities.

### Alternative Approaches (Not Implemented)

Other potential solutions that were considered:
1. Reinstalling the Rust toolchain
2. Modifying system libraries or environment variables
3. Using a different crate for command-line argument parsing
4. Building with a different target triple

### If You Encounter This Issue

If you encounter this issue when building from source:

1. Ensure you're using the nightly Rust toolchain:
   ```bash
   rustup default nightly
   ```

2. If the issue persists, try reinstalling the Rust toolchain:
   ```bash
   rustup update
   rustup toolchain install nightly --force
   ```

3. Check for any missing system dependencies related to compiler toolchains

4. The workaround implemented in this project should allow successful builds without requiring additional configuration

### References

This issue appears to be relatively uncommon, with limited documentation available online. If you have more information about this error or find a definitive solution, please consider submitting a pull request to update this document.

## Last Updated

This document was last updated on April 24, 2025.