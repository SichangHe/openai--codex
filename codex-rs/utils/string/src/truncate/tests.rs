use super::approx_tokens_from_byte_count;
use super::removed_byte_count;
use super::removed_units;
use super::split_string;
use super::truncate_middle_chars;
use super::truncate_middle_with_token_budget;
use pretty_assertions::assert_eq;

#[test]
fn split_string_works() {
    assert_eq!(
        split_string(
            "hello world",
            /*beginning_bytes*/ 5,
            /*end_bytes*/ 5
        ),
        (1, "hello", "world")
    );
    assert_eq!(
        split_string("abc", /*beginning_bytes*/ 0, /*end_bytes*/ 0),
        (3, "", "")
    );
}

#[test]
fn split_string_handles_empty_string() {
    assert_eq!(
        split_string("", /*beginning_bytes*/ 4, /*end_bytes*/ 4),
        (0, "", "")
    );
}

#[test]
fn split_string_only_keeps_prefix_when_tail_budget_is_zero() {
    assert_eq!(
        split_string("abcdef", /*beginning_bytes*/ 3, /*end_bytes*/ 0),
        (3, "abc", "")
    );
}

#[test]
fn split_string_only_keeps_suffix_when_prefix_budget_is_zero() {
    assert_eq!(
        split_string("abcdef", /*beginning_bytes*/ 0, /*end_bytes*/ 3),
        (3, "", "def")
    );
}

#[test]
fn split_string_handles_overlapping_budgets_without_removal() {
    assert_eq!(
        split_string("abcdef", /*beginning_bytes*/ 4, /*end_bytes*/ 4),
        (0, "abcd", "ef")
    );
}

#[test]
fn split_string_respects_utf8_boundaries() {
    assert_eq!(
        split_string("😀abc😀", /*beginning_bytes*/ 5, /*end_bytes*/ 5),
        (1, "😀a", "c😀")
    );

    assert_eq!(
        split_string(
            "😀😀😀😀😀",
            /*beginning_bytes*/ 1,
            /*end_bytes*/ 1
        ),
        (5, "", "")
    );
    assert_eq!(
        split_string(
            "😀😀😀😀😀",
            /*beginning_bytes*/ 7,
            /*end_bytes*/ 7
        ),
        (3, "😀", "😀")
    );
    assert_eq!(
        split_string(
            "😀😀😀😀😀",
            /*beginning_bytes*/ 8,
            /*end_bytes*/ 8
        ),
        (1, "😀😀", "😀😀")
    );
}

#[test]
fn truncate_with_token_budget_returns_original_when_under_limit() {
    let s = "short output";
    let limit = 100;
    let (out, original) = truncate_middle_with_token_budget(s, limit);
    assert_eq!(out, s);
    assert_eq!(original, None);
}

#[test]
fn truncate_with_token_budget_reports_truncation_at_zero_limit() {
    let s = "abcdef";
    let (out, original) = truncate_middle_with_token_budget(s, /*max_tokens*/ 0);
    assert_eq!(out, "…2 tokens truncated…");
    assert_eq!(original, Some(2));
}

#[test]
fn truncate_middle_tokens_handles_utf8_content() {
    let s = "😀😀😀😀😀😀😀😀😀😀\nsecond line with text\n";
    let (out, tokens) = truncate_middle_with_token_budget(s, /*max_tokens*/ 8);
    assert_eq!(out, "😀😀😀😀…8 tokens truncated… line with text\n");
    assert_eq!(tokens, Some(16));
}

#[test]
fn truncate_middle_tokens_counts_bytes_removed_after_utf8_rounding() {
    let (out, tokens) = truncate_middle_with_token_budget("€€", /*max_tokens*/ 1);
    assert_eq!(out, "…2 tokens truncated…");
    assert_eq!(tokens, Some(2));
}

#[test]
fn production_accounting_chain_matches_verified_equation() {
    for total_bytes in 0..=32 {
        for retained_prefix_bytes in 0..=total_bytes {
            for retained_suffix_bytes in 0..=(total_bytes - retained_prefix_bytes) {
                let omitted_bytes =
                    removed_byte_count(total_bytes, retained_prefix_bytes, retained_suffix_bytes);
                let expected_omitted_bytes =
                    total_bytes - retained_prefix_bytes - retained_suffix_bytes;

                assert_eq!(omitted_bytes, expected_omitted_bytes);
                assert_eq!(
                    removed_units(/*use_tokens*/ true, omitted_bytes, usize::MAX),
                    approx_tokens_from_byte_count(expected_omitted_bytes)
                );
            }
        }
    }

    for (total_bytes, retained_prefix_bytes, retained_suffix_bytes) in [
        (0, 1, 0),
        (4, 5, 1),
        (4, 3, 2),
        (usize::MAX, usize::MAX, usize::MAX),
    ] {
        let omitted_bytes =
            removed_byte_count(total_bytes, retained_prefix_bytes, retained_suffix_bytes);
        let expected_omitted_bytes = total_bytes
            .saturating_sub(retained_prefix_bytes)
            .saturating_sub(retained_suffix_bytes);

        assert_eq!(omitted_bytes, expected_omitted_bytes);
        assert_eq!(
            removed_units(/*use_tokens*/ true, omitted_bytes, usize::MAX),
            approx_tokens_from_byte_count(expected_omitted_bytes)
        );
    }
}

#[test]
fn split_string_slices_are_utf8_aligned_disjoint_and_accounted() {
    for s in ["", "abcdef", "é€😀", "😀abc😀", "€€"] {
        for beginning_bytes in 0..=s.len().saturating_add(1) {
            for end_bytes in 0..=s.len().saturating_add(1) {
                let (_, prefix, suffix) = split_string(s, beginning_bytes, end_bytes);
                let suffix_start = s.len() - suffix.len();

                assert!(s.is_char_boundary(prefix.len()));
                assert!(s.is_char_boundary(suffix_start));
                assert_eq!(prefix, &s[..prefix.len()]);
                assert_eq!(suffix, &s[suffix_start..]);
                assert!(prefix.len() <= suffix_start);
                assert_eq!(
                    removed_byte_count(s.len(), prefix.len(), suffix.len()),
                    suffix_start - prefix.len()
                );
            }
        }
    }
}

#[test]
fn truncate_middle_bytes_handles_utf8_content() {
    let s = "😀😀😀😀😀😀😀😀😀😀\nsecond line with text\n";
    let out = truncate_middle_chars(s, /*max_bytes*/ 20);
    assert_eq!(out, "😀😀…21 chars truncated…with text\n");
}
