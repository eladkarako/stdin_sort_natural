use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::{self, BufReader, BufWriter, IsTerminal, Read, Write},
};

fn find_ascii_digit_run_end(source_bytes: &[u8], start_index: usize) -> usize {
    let mut current_index = start_index;

    while current_index < source_bytes.len() && source_bytes[current_index].is_ascii_digit() {
        current_index += 1;
    }

    return current_index;
}

fn find_first_nonzero_index(source_bytes: &[u8], start_index: usize, end_index: usize) -> usize {
    let mut current_index = start_index;

    while current_index < end_index && source_bytes[current_index] == b'0' {
        current_index += 1;
    }

    return current_index;
}

#[derive(Clone, Copy, Debug)]
struct DigitRunDescription {
    run_end_index: usize,
    significant_start_index: usize,
    significant_digit_count: usize,
    original_digit_count: usize,
}

fn describe_ascii_digit_run(source_bytes: &[u8], run_start_index: usize) -> DigitRunDescription {
    let run_end_index = find_ascii_digit_run_end(source_bytes, run_start_index);

    let is_run_start_zero = source_bytes[run_start_index] == b'0';

    let significant_start_index = if is_run_start_zero {
        find_first_nonzero_index(source_bytes, run_start_index, run_end_index)
    } else {
        run_start_index
    };

    let significant_digit_count = run_end_index - significant_start_index;
    let original_digit_count = run_end_index - run_start_index;

    return DigitRunDescription {
        run_end_index,
        significant_start_index,
        significant_digit_count,
        original_digit_count,
    };
}

fn compare_ascii_digit_slices(
    left_bytes: &[u8],
    left_start_index: usize,
    right_bytes: &[u8],
    right_start_index: usize,
    digit_count: usize,
) -> Ordering {
    const DIGIT_COMPARISON_BLOCK_SIZE: usize = 16;

    let mut block_offset = 0usize;

    while block_offset + DIGIT_COMPARISON_BLOCK_SIZE <= digit_count {
        for within_block_offset in 0..DIGIT_COMPARISON_BLOCK_SIZE {
            let left_digit_index = left_start_index + block_offset + within_block_offset;
            let right_digit_index = right_start_index + block_offset + within_block_offset;

            let left_digit = left_bytes[left_digit_index];
            let right_digit = right_bytes[right_digit_index];

            if left_digit != right_digit {
                return left_digit.cmp(&right_digit);
            }
        }

        block_offset += DIGIT_COMPARISON_BLOCK_SIZE;
    }

    while block_offset < digit_count {
        let left_digit_index = left_start_index + block_offset;
        let right_digit_index = right_start_index + block_offset;

        let left_digit = left_bytes[left_digit_index];
        let right_digit = right_bytes[right_digit_index];

        if left_digit != right_digit {
            return left_digit.cmp(&right_digit);
        }

        block_offset += 1;
    }

    return Ordering::Equal;
}

fn compare_described_digit_runs(
    left_bytes: &[u8],
    left_run: DigitRunDescription,
    right_bytes: &[u8],
    right_run: DigitRunDescription,
) -> Ordering {
    let left_significant_digit_count = left_run.significant_digit_count;
    let right_significant_digit_count = right_run.significant_digit_count;

    if left_significant_digit_count == 0 && right_significant_digit_count == 0 {
        let left_original_digit_count = left_run.original_digit_count;
        let right_original_digit_count = right_run.original_digit_count;

        return left_original_digit_count.cmp(&right_original_digit_count);
    }

    if left_significant_digit_count != right_significant_digit_count {
        return left_significant_digit_count.cmp(&right_significant_digit_count);
    }

    let left_significant_start_index = left_run.significant_start_index;
    let right_significant_start_index = right_run.significant_start_index;

    let significant_digit_count = left_run.significant_digit_count;

    let digit_comparison_order = compare_ascii_digit_slices(
        left_bytes,
        left_significant_start_index,
        right_bytes,
        right_significant_start_index,
        significant_digit_count,
    );

    if digit_comparison_order != Ordering::Equal {
        return digit_comparison_order;
    }

    let left_original_digit_count = left_run.original_digit_count;
    let right_original_digit_count = right_run.original_digit_count;

    return left_original_digit_count.cmp(&right_original_digit_count);
}

pub fn compare_natural_order_bytes_without_allocation(
    left_bytes: &[u8],
    right_bytes: &[u8],
) -> Ordering {
    let mut left_index = 0usize;
    let mut right_index = 0usize;

    while left_index < left_bytes.len() && right_index < right_bytes.len() {
        let left_byte = left_bytes[left_index];
        let right_byte = right_bytes[right_index];

        let is_left_byte_is_digit = left_byte.is_ascii_digit();
        let is_right_byte_is_digit = right_byte.is_ascii_digit();

        if is_left_byte_is_digit && is_right_byte_is_digit {
            let left_digit_run = describe_ascii_digit_run(left_bytes, left_index);
            let right_digit_run = describe_ascii_digit_run(right_bytes, right_index);

            let run_order = compare_described_digit_runs(
                left_bytes,
                left_digit_run,
                right_bytes,
                right_digit_run,
            );

            if run_order != Ordering::Equal {
                return run_order;
            }

            let next_left_index = left_digit_run.run_end_index;
            let next_right_index = right_digit_run.run_end_index;

            left_index = next_left_index;
            right_index = next_right_index;

            continue;
        }

        if !is_left_byte_is_digit && !is_right_byte_is_digit {
            let left_folded_byte = ascii_lowercase_byte(left_byte);
            let right_folded_byte = ascii_lowercase_byte(right_byte);

            let non_digit_order = left_folded_byte.cmp(&right_folded_byte);

            if non_digit_order != Ordering::Equal {
                return non_digit_order;
            }

            left_index += 1;
            right_index += 1;

            continue;
        }

        // One side is a digit; the other side is not.
        if is_left_byte_is_digit {
            return Ordering::Less;
        } else {
            return Ordering::Greater;
        };
    }

    return left_bytes.len().cmp(&right_bytes.len());
}

fn trim_space_and_tab(b: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < b.len() && (b[start] == b' ' || b[start] == b'\t') {
        start += 1;
    }

    let mut end = b.len();
    while end > start && (b[end - 1] == b' ' || b[end - 1] == b'\t') {
        end -= 1;
    }

    return &b[start..end];
}

fn ascii_lowercase_byte(byte: u8) -> u8 {
    if (b'A'..=b'Z').contains(&byte) {
        return byte + (b'a' - b'A');
    } else {
        return byte;
    }
}

fn hash_ascii_lowercase_into_hasher(hasher: &mut DefaultHasher, source_bytes: &[u8]) {
    for &byte in source_bytes {
        ascii_lowercase_byte(byte).hash(hasher);
    }

    return;
}

fn compute_hash_key(source_bytes: &[u8], ignore_case_ascii: bool) -> u64 {
    let mut hasher = DefaultHasher::new();

    if ignore_case_ascii {
        hash_ascii_lowercase_into_hasher(&mut hasher, source_bytes);
    } else {
        source_bytes.hash(&mut hasher);
    }

    return hasher.finish();
}

fn dedup_lines_ignore_ascii_case<'a>(lines: &mut Vec<&'a [u8]>) {
    let mut deduped: Vec<&'a [u8]> = Vec::with_capacity(lines.len());

    let mut prev_hash: Option<u64> = None;
    for &line in lines.iter() {
        let h = compute_hash_key(line, true); // ignore ASCII case
        if prev_hash.map_or(false, |ph| ph == h) {
            continue;
        }
        deduped.push(line);
        prev_hash = Some(h);
    }

    *lines = deduped;
    return;
}

fn main() -> io::Result<()> {
    if io::stdin().is_terminal() {
        // STDIN is not a pipe. it is an interactive TTY. no EOF will arrive automatically, so the app will just hang.
        return Ok(());
    }

    let mut stdin_handle = BufReader::new(io::stdin().lock());
    let mut input_bytes = Vec::<u8>::new();
    stdin_handle.read_to_end(&mut input_bytes)?;
    input_bytes.retain(|&byte| byte != b'\r' && byte != b'\x0B'); // cleanup

    // split to lines, trim line from both ends, remove empty lines.
    let mut lines: Vec<&[u8]> = input_bytes
        .split(|&separator_byte| separator_byte == b'\n')
        .map(|line| {
            return trim_space_and_tab(line);
        })
        .filter(|s| !s.is_empty())
        .collect();

    lines.sort_by(|left, right| compare_natural_order_bytes_without_allocation(left, right));
    dedup_lines_ignore_ascii_case(&mut lines);

    let mut stdout_handle = BufWriter::new(io::stdout().lock());
    let eol_bytes: &[u8] = if cfg!(windows) { b"\r\n" } else { b"\n" };

    for (line_index, line_bytes) in lines.iter().enumerate() {
        if line_index > 0 {
            stdout_handle.write_all(eol_bytes)?;
        }
        stdout_handle.write_all(line_bytes)?;
    }

    stdout_handle.flush()?;
    return Ok(());
}
