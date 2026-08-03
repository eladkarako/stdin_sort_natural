use std::cmp::Ordering;
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
#[derive(Clone, Copy, Debug)]
struct DigitRun {
    sig_start: usize,
    value_len: usize,
    original_len: usize,
}
fn classify_digit_run(bytes: &[u8], run_start: usize) -> DigitRun {
    let run_end = scan_digit_run_end(bytes, run_start);
    let sig_start = if bytes[run_start] == b'0' {
        first_nonzero_index(bytes, run_start, run_end)
    } else {
        run_start
    };
    let value_len = run_end - sig_start;
    DigitRun {
        sig_start,
        value_len,
        original_len: run_end - run_start,
    }
}
fn compare_digit_runs(left: &[u8], right: &[u8], i: usize, j: usize) -> Ordering {
    let a = classify_digit_run(left, i);
    let b = classify_digit_run(right, j);
    if a.value_len == 0 && b.value_len == 0 {
        return a.original_len.cmp(&b.original_len);
    }
    if a.value_len != b.value_len {
        return a.value_len.cmp(&b.value_len);
    }
    let ord = compare_digits_same_length(left, a.sig_start, right, b.sig_start, a.value_len);
    if ord != Ordering::Equal {
        return ord;
    }
    a.original_len.cmp(&b.original_len)
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
            let ord = compare_digit_runs(a, b, i, j);
            if ord != Ordering::Equal {
                return ord;
            }
            i = scan_digit_run_end(a, i);
            j = scan_digit_run_end(b, j);
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
fn main() {
    let mut v = vec![
        "file2.txt",
        "file10.txt",
        "אאא",
        "גגגג",
        "בבב",
        "파일2",
        "파일10",
        "あ2",
        "あ10",
        "a9",
        "a02",
    ];
    v.sort_by(|x, y| natural_cmp_no_alloc(x, y));
    for s in v {
        println!("{s}");
    }
}
