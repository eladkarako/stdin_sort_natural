use std::cmp::Ordering;
use std::collections::HashSet;

fn scan_digit_run_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i
}
fn first_nonzero_index(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    while i < end && bytes[i] == b'0' {
        i += 1;
    }
    i
}
#[derive(Clone, Copy, Debug)]
struct DigitRun {
    run_end: usize,
    significant_start: usize,
    value_len: usize,
    original_len: usize,
}
fn classify_digit_run(bytes: &[u8], run_start: usize) -> DigitRun {
    let run_end = scan_digit_run_end(bytes, run_start);
    let significant_start = if bytes[run_start] == b'0' {
        first_nonzero_index(bytes, run_start, run_end)
    } else {
        run_start
    };
    let value_len = run_end - significant_start;
    DigitRun {
        run_end,
        significant_start,
        value_len,
        original_len: run_end - run_start,
    }
}
fn compare_digits_same_length(
    a: &[u8],
    a_start: usize,
    b: &[u8],
    b_start: usize,
    len: usize,
) -> Ordering {
    const BLOCK: usize = 16;
    let mut k = 0usize;
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
fn compare_digit_runs(a: &[u8], a_run: DigitRun, b: &[u8], b_run: DigitRun) -> Ordering {
    if a_run.value_len == 0 && b_run.value_len == 0 {
        return a_run.original_len.cmp(&b_run.original_len);
    }
    if a_run.value_len != b_run.value_len {
        return a_run.value_len.cmp(&b_run.value_len);
    }
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
    a_run.original_len.cmp(&b_run.original_len)
}
pub fn natural_cmp_no_alloc(left: &str, right: &str) -> Ordering {
    let a = left.as_bytes();
    let b = right.as_bytes();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < a.len() && j < b.len() {
        let a_is_digit = a[i].is_ascii_digit();
        let b_is_digit = b[j].is_ascii_digit();
        if a_is_digit && b_is_digit {
            let a_run = classify_digit_run(a, i);
            let b_run = classify_digit_run(b, j);
            let ord = compare_digit_runs(a, a_run, b, b_run);
            if ord != Ordering::Equal {
                return ord;
            }
            i = a_run.run_end;
            j = b_run.run_end;
            continue;
        }
        if !a_is_digit && !b_is_digit {
            let ord = a[i].cmp(&b[j]);
            if ord != Ordering::Equal {
                return ord;
            }
            i += 1;
            j += 1;
            continue;
        }
        return if a_is_digit {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    a.len().cmp(&b.len())
}
fn normalize_line_owned(mut s: String) -> Option<String> {
    s.retain(|c| c != '\u{000B}' && c != '\r' && c != '\n');
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut end = bytes.len();
    while start < end {
        match bytes[start] {
            b' ' | b'\t' => start += 1,
            _ => break,
        }
    }
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
fn dedup_key_alloc(s: &str, ignore_case: bool) -> String {
    if ignore_case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}
pub fn process_and_sort_lines<I>(lines: I, ignore_case: bool) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for raw in lines {
        let raw = raw.as_ref();
        let normalized = normalize_line_owned(raw.to_string());
        let Some(normalized) = normalized else {
            continue;
        };
        let key = dedup_key_alloc(&normalized, ignore_case);
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
