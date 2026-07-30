# truncation-marker verification

production flow

- `truncate_with_byte_estimate`
  - gets UTF-8 `&str` prefix and suffix slices from `split_string`
  - passes their actual lengths to `removed_byte_count`
- `removed_byte_count`
  - applies two saturating subtractions
  - returns the actual omitted-byte count when the retained slices are disjoint
- `removed_units`
  - sends token-mode omitted bytes to `approx_tokens_from_byte_count`
  - keeps Codex's saturating `ceil(bytes / 4)` approximation

proof

- `accounting.rs`
  - models both production saturating subtractions
  - models production's saturating addition before division by four
  - proves disjoint retained lengths partition the input
  - proves the token marker consumes the actual omitted-byte result
  - proves the minimized witness yields two actual-byte tokens and one nominal-budget token

executable binding

- `production_accounting_chain_matches_verified_equation`
  - calls the three production accounting functions together
  - covers every valid partition through 32 bytes
  - covers invalid and maximum-value inputs to bind saturation semantics
- `split_string_slices_are_utf8_aligned_disjoint_and_accounted`
  - calls the real splitter across ASCII and one- through four-byte UTF-8 inputs and all nearby budgets
  - checks both returned `&str` boundaries, slice origin, disjointness, and the production omitted-byte result
- `truncate_middle_tokens_counts_bytes_removed_after_utf8_rounding`
  - calls the public production entry point on the minimized witness

verification

- staged command
  - `/ssd1/sichangheagent/vl_experiments/openai-codex-truncate-marker-prep-20260729/tools/verus-x86-linux/verus accounting.rs`
