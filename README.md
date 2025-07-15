# r3bl-open-core-archive

All the crates that are no longer maintained in
[r3bl-open-core](https://github.com/r3bl-org/r3bl-open-core/) mono repo are moved here for
posterity.

Links will be provided in the
[CHANGELOG.md](https://github.com/r3bl-org/r3bl-open-core/blob/main/CHANGELOG.md) of the mono repo,
to keep track of when crates are moved out into this archive repo.

If you need to, you can add a dependency to this repo in your `Cargo.toml`.
Here's an example.

```rust
r3bl_simple_logger = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_redux = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_rs_utils_macro = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_ansi_color = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_test_fixtures = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_log = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_script = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_tuify = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_terminal_async = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_core = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
r3bl_md_parser_ng = { git = "https://github.com/r3bl-org/r3bl-open-core-archive" }
```

You can submit PRs if you need any changes made to the crates in this repo.
