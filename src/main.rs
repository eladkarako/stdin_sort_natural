use std::cmp::Ordering;
use std::collections::HashSet;

/// Returns the index one past the end of the ASCII digit run starting at `start`.
///
/// Example: for "abc123def" with `start` at the '1', returns the index after '3'.
fn scan_digit_run_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start;

    // Walk forward while bytes are ASCII digits.
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    i
}

/// Finds the first non-'0' byte in `bytes[start..end]`.
///
/// Precondition: `bytes[start] == b'0'`.
fn first_nonzero_index(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;

    // Skip leading zeros.
    while i < end && bytes[i] == b'0' {
        i += 1;
    }

    i
}

/// Cached metadata for a digit run.
///
/// A digit run is a contiguous sequence of ASCII digits.
#[derive(Clone, Copy, Debug)]
struct DigitRun {
    /// Index one past the end of the digit run.
    run_end: usize,

    /// Index of the first significant digit (skips leading '0' if present).
    significant_start: usize,

    /// Number of significant digits: `run_end - significant_start`.
    /// If all digits are '0', this becomes 0.
    value_len: usize,

    /// Total digit run length: `run_end - run_start`.
    original_len: usize,
}

/// Classify a digit run starting at `run_start`.
fn classify_digit_run(bytes: &[u8], run_start: usize) -> DigitRun {
    // Compute the end of the digit run once.
    let run_end = scan_digit_run_end(bytes, run_start);

    // Significant start: skip leading zeros only if the run starts with '0'.
    let significant_start = if bytes[run_start] == b'0' {
        first_nonzero_index(bytes, run_start, run_end)
    } else {
        run_start
    };

    // Significant length: if all digits are zeros, this is 0.
    let value_len = run_end - significant_start;

    DigitRun {
        run_end,
        significant_start,
        value_len,
        original_len: run_end - run_start,
    }
}

/// Compare two digit slices that have the same length (`len`) lexicographically.
///
/// Both slices are expected to already point at the start of the significant digits.
fn compare_digits_same_length(
    a: &[u8],
    a_start: usize,
    b: &[u8],
    b_start: usize,
    len: usize,
) -> Ordering {
    const BLOCK: usize = 16;
    let mut k = 0usize;

    // Chunked comparison to reduce bounds checks / allow better CPU behavior.
    while k + BLOCK <= len {
        for t in 0..BLOCK {
            let da = a[a_start + k + t];
            let db = b[b_start + k + t];
            if da != db {
                return da.cmp(&db);
            }
        }
        k += BLOCK;
    }

    // Tail comparison.
    while k < len {
        let da = a[a_start + k];
        let db = b[b_start + k];
        if da != db {
            return da.cmp(&db);
        }
        k += 1;
    }

    Ordering::Equal
}

/// Compare two digit runs using a "natural sort" rule:
/// - First compare by number of significant digits (after trimming leading zeros).
/// - If equal, compare significant digits lexicographically.
/// - If significant digits are identical (e.g., both represent 2), then
///   compare by original run length so "02" > "2" and "002" > "02" (stable-ish ordering).
fn compare_digit_runs(a: &[u8], a_run: DigitRun, b: &[u8], b_run: DigitRun) -> Ordering {
    // If both are all-zeros (significant_len == 0), compare by run length.
    if a_run.value_len == 0 && b_run.value_len == 0 {
        return a_run.original_len.cmp(&b_run.original_len);
    }

    // Different number of significant digits => larger numeric value has more digits.
    if a_run.value_len != b_run.value_len {
        return a_run.value_len.cmp(&b_run.value_len);
    }

    // Same significant length => lexicographic compare of significant digits.
    let ord = compare_digits_same_length(
        a,
        a_run.significant_start,
        b,
        b_run.significant_start,
        a_run.value_len,
    );

    if ord != Ordering::Equal {
        return ord;
    }

    // Same significant digits => tie-break by original length (leading zeros).
    a_run.original_len.cmp(&b_run.original_len)
}

/// Natural (human-friendly) ordering for strings without extra allocations.
///
/// Rules:
/// - Non-digit characters are compared byte-by-byte.
/// - Digit runs are compared by numeric value:
///   - more significant digits wins
///   - then lexicographic by digits
///   - then tie-break using run length (handles leading zeros deterministically)
pub fn natural_cmp_no_alloc(left: &str, right: &str) -> Ordering {
    let a = left.as_bytes();
    let b = right.as_bytes();

    let mut i = 0usize;
    let mut j = 0usize;

    // Two-pointer scan over bytes.
    while i < a.len() && j < b.len() {
        let a_is_digit = a[i].is_ascii_digit();
        let b_is_digit = b[j].is_ascii_digit();

        // Both are digit runs: classify once per run and compare.
        if a_is_digit && b_is_digit {
            let a_run = classify_digit_run(a, i);
            let b_run = classify_digit_run(b, j);

            let ord = compare_digit_runs(a, a_run, b, b_run);
            if ord != Ordering::Equal {
                return ord;
            }

            // Skip whole digit runs (no per-char stepping).
            i = a_run.run_end;
            j = b_run.run_end;
            continue;
        }

        // Both are non-digits: compare single bytes.
        if !a_is_digit && !b_is_digit {
            let ord = a[i].cmp(&b[j]);
            if ord != Ordering::Equal {
                return ord;
            }
            i += 1;
            j += 1;
            continue;
        }

        // One side is digit and the other is not: digit sorts before non-digit.
        return if a_is_digit {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }

    // If one string ended, shorter string sorts first.
    a.len().cmp(&b.len())
}

/// Normalize a line by:
/// 1) removing all occurrences of '\u{000B}' (vertical tab), '\r', and '\n'
/// 2) trimming leading/trailing spaces and tabs (' ' and '\t')
/// 3) returning `None` for empty results
///
/// This uses `String::retain` for the removal in-place.
fn normalize_line_owned(mut s: String) -> Option<String> {
    // In-place removal of control chars anywhere in the string.
    s.retain(|c| c != '\u{000B}' && c != '\r' && c != '\n');

    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut end = bytes.len();

    // Trim leading spaces/tabs.
    while start < end {
        match bytes[start] {
            b' ' | b'\t' => start += 1,
            _ => break,
        }
    }

    // Trim trailing spaces/tabs.
    while end > start {
        match bytes[end - 1] {
            b' ' | b'\t' => end -= 1,
            _ => break,
        }
    }

    let trimmed = &s[start..end];
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Create a dedup key for the already-normalized line.
///
/// If `ignore_case` is true, uses Unicode lowercasing via `to_lowercase()`.
fn dedup_key_alloc(s: &str, ignore_case: bool) -> String {
    if ignore_case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}

/// Normalize, deduplicate (keeping the first occurrence), and sort using natural order.
///
/// - `lines`: any iterable of items convertible to `&str`
/// - `ignore_case`: when true, dedup is case-insensitive (sorting is case-sensitive)
pub fn process_and_sort_lines<I>(lines: I, ignore_case: bool) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();

    for raw in lines {
        let raw = raw.as_ref();

        // Need an owned String for retain-based removal.
        let normalized = normalize_line_owned(raw.to_string());
        let Some(normalized) = normalized else {
            continue;
        };

        // Dedup key allocation (required with this simple approach).
        let key = dedup_key_alloc(&normalized, ignore_case);

        // Keep the first occurrence only.
        if seen.insert(key) {
            out.push(normalized);
        }
    }

    out.sort_by(|a, b| natural_cmp_no_alloc(a, b));
    out
}

fn main() {
    let raw_lines = vec![
        " file10.txt\n",
        "\tfile2.txt\r",
        "file2.txt\n",
        "  FILE10.TXT  ",
        "   ",
        "אאא\n",
        "גגגג",
        "בבב\n",
        "파일10\n",
        "파일2\n",
        "あ10\n",
        "あ2\n",
        "a02\n",
        "a2\n",
    ];

    let processed = process_and_sort_lines(raw_lines, true);

    for s in processed {
        println!("{s}");
    }
}
