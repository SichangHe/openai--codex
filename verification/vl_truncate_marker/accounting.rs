use vstd::prelude::*;

verus! {

spec fn saturating_sub(left: int, right: int) -> int {
    if right <= left { left - right } else { 0 }
}

spec fn saturating_add_three_u64(bytes: int) -> int {
    if bytes <= u64::MAX as int - 3 {
        bytes + 3
    } else {
        u64::MAX as int
    }
}

spec fn omitted_bytes(
    total_bytes: int,
    retained_prefix_bytes: int,
    retained_suffix_bytes: int,
) -> int {
    saturating_sub(
        saturating_sub(total_bytes, retained_prefix_bytes),
        retained_suffix_bytes,
    )
}

spec fn approximate_tokens(bytes: int) -> int {
    saturating_add_three_u64(bytes) / 4
}

spec fn token_marker_count(
    total_bytes: int,
    retained_prefix_bytes: int,
    retained_suffix_bytes: int,
) -> int {
    approximate_tokens(omitted_bytes(
        total_bytes,
        retained_prefix_bytes,
        retained_suffix_bytes,
    ))
}

fn saturating_sub_model(left: usize, right: usize) -> (result: usize)
    ensures
        result as int == saturating_sub(left as int, right as int),
{
    if right <= left { left - right } else { 0 }
}

fn removed_byte_count_model(
    total_bytes: usize,
    retained_prefix_bytes: usize,
    retained_suffix_bytes: usize,
) -> (result: usize)
    ensures
        result as int == omitted_bytes(
            total_bytes as int,
            retained_prefix_bytes as int,
            retained_suffix_bytes as int,
        ),
{
    let after_prefix = saturating_sub_model(total_bytes, retained_prefix_bytes);
    saturating_sub_model(after_prefix, retained_suffix_bytes)
}

fn approximate_tokens_model(bytes: u64) -> (result: u64)
    ensures
        result as int == approximate_tokens(bytes as int),
{
    let rounded = if bytes <= u64::MAX - 3 {
        bytes + 3
    } else {
        u64::MAX
    };
    rounded / 4
}

proof fn disjoint_retained_slices_partition_the_input(
    total_bytes: int,
    retained_prefix_bytes: int,
    retained_suffix_bytes: int,
)
    requires
        0 <= retained_prefix_bytes,
        0 <= retained_suffix_bytes,
        retained_prefix_bytes + retained_suffix_bytes <= total_bytes,
    ensures
        omitted_bytes(total_bytes, retained_prefix_bytes, retained_suffix_bytes)
            == total_bytes - retained_prefix_bytes - retained_suffix_bytes,
        omitted_bytes(total_bytes, retained_prefix_bytes, retained_suffix_bytes)
            + retained_prefix_bytes
            + retained_suffix_bytes
            == total_bytes,
{
}

proof fn marker_uses_actual_omitted_bytes(
    total_bytes: int,
    retained_prefix_bytes: int,
    retained_suffix_bytes: int,
)
    ensures
        token_marker_count(total_bytes, retained_prefix_bytes, retained_suffix_bytes)
            == approximate_tokens(omitted_bytes(
                total_bytes,
                retained_prefix_bytes,
                retained_suffix_bytes,
            )),
{
}

proof fn minimized_utf8_witness()
    ensures
        omitted_bytes(6, 0, 0) == 6,
        token_marker_count(6, 0, 0) == 2,
        approximate_tokens(saturating_sub(6, 4)) == 1,
{
}

fn main() {
    let omitted_bytes = removed_byte_count_model(6, 0, 0);
    let marker_tokens = approximate_tokens_model(omitted_bytes as u64);
    assert(omitted_bytes == 6);
    assert(marker_tokens == 2);
}

}
