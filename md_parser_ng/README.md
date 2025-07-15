# MD Parser NG - Archived Experimental Markdown Parsers

## Overview

This crate contains two experimental markdown parsers that were developed as potential
replacements for the legacy parser in r3bl_tui, but ultimately archived in favor of
retaining the mature, battle-tested legacy implementation.

### Parsers Included

1. **NG Parser** (`parse_markdown_ng`): A nom-based parser with virtual array abstraction
   (`AsStrSlice`)
2. **Simple Parser** (`parse_markdown_simple`): Direct string manipulation without nom

## Why Archived?

After extensive benchmarking and analysis, we found:

- The NG parser was 600-5,000x slower than the legacy parser due to virtual array
  abstraction overhead
- The Simple parser achieved performance comparable to the legacy parser (within 25%) but
  didn't justify the migration risk
- The legacy parser remains the best choice due to its maturity, reliability, and
  excellent performance

## Performance Comparison

| Parser | Small Content | Medium Content | Large Content |
| ------ | ------------- | -------------- | ------------- |
| Legacy | ~2.6-24K ns   | ~72-196K ns    | ~196K ns      |
| Simple | ~2-25K ns     | ~76-243K ns    | ~243K ns      |
| NG/nom | ~19K-1.4M ns  | ~44M-685M ns   | ~686M ns      |

## Repository Structure

```
md_parser_ng/
├── src/
│   ├── mod.rs                    # Main library entry point
│   ├── parse_markdown_ng.rs      # NG parser implementation
│   ├── parse_markdown_simple.rs  # Simple parser implementation
│   ├── local_constants.rs        # Local constants and definitions
│   ├── local_types.rs            # Local type definitions and aliases
│   ├── as_str_slice/            # Virtual array abstraction for NG parser
│   ├── block_ng/                # Block parsers for NG
│   ├── extended_ng/             # Extended parsers for NG
│   ├── fragment_ng/             # Fragment parsers for NG
│   ├── standard_ng/             # Standard parsers for NG
│   └── compat_test_data/        # Test data for compatibility testing
├── tests/                       # Integration tests
├── benches/                     # Performance benchmarks
└── docs/                        # Additional documentation
```

## Usage

This crate is archived and not intended for production use. It's maintained for:

- Historical reference
- Learning purposes
- Understanding parser design trade-offs

To use in experiments:

```rust
use md_parser_ng::{parse_markdown_ng, parse_markdown_simple, AsStrSlice, GCString};

// NG Parser
let lines: Vec<GCString> = content.lines().map(GCString::from).collect();
let input = AsStrSlice::from(lines.as_slice());
let result = parse_markdown_ng(input);

// Simple Parser
let lines: Vec<GCString> = content.lines().map(GCString::from).collect();
let result = parse_markdown_simple(&lines);
```

## Key Learnings

1. **Virtual array abstractions can be extremely costly** - The `AsStrSlice` abstraction
   added massive overhead
2. **Parser combinators aren't always the answer** - nom added complexity without
   performance benefits for this use case
3. **Direct string manipulation often wins** - The Simple parser proved this approach can
   be both simple and performant
4. **Migration risk must be justified** - A 25% performance improvement doesn't justify
   rewriting battle-tested code

## Dependencies

Pinned to specific commit for compatibility. This commit does not have `AsStrSlice` or any
other files that are in this `md_parser_ng` crate. However, by the time this crate was
archived, the `r3bl_tui` crate had diverged significantly, and the missing code is in
`local_types.rs` and `local_constants.rs`.

```toml
r3bl_tui = { git = "https://github.com/r3bl-org/r3bl-open-core.git", rev = "fe1182a0f6c40f38852f2204b9895bef546aeed7" }
```

## Documentation

See the `docs/` directory for detailed analysis:

- `ng_parser_virtual_array.md` - Virtual array design and performance analysis
- `ng_parser_simple_drop_nom.md` - Simple parser design without nom
- `parser_strategy_analysis.md` - Comprehensive parser strategy comparison

## License

Apache-2.0

## Build Status

⚠️ **Note**: This archive currently has compilation issues due to API changes and missing
type exports from r3bl_tui. Since this is an archival crate intended for historical
reference, these issues are not being fixed. The code is preserved as-is to show the state
of the experimental parsers at the time of archival.

### Known Compilation Issues

1. Missing type exports from r3bl_tui (MdElement, CharLengthExt, etc.)
2. ~~API changes in struct fields (HeadingData.level vs heading_level)~~ - **Fixed**:
   Updated all occurrences to use `heading_level` field
3. Trait implementation restrictions for external types
4. Missing utility methods and macros

The `missing_types.rs` file contains partial workarounds for some of these issues, but
full compilation would require significant modifications that would alter the historical
accuracy of the archive.

## Note

For production markdown parsing needs, use the legacy parser in the main r3bl_tui crate.
