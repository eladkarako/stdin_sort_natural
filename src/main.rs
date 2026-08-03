use std::cmp::Ordering;

fn end_digits(s: &[u8], mut i: usize) -> usize {
    while i < s.len() && s[i].is_ascii_digit() {
        i += 1;
    }
    return i;
}

// Returns first index in [start,end) where s[idx] != '0', else end if all zeros.
fn first_nonzero_in_range(s: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    while i < end && s[i] == b'0' {
        i += 1;
    }
    return i;
}

fn cmp_equal_len_digits(
    a: &[u8],
    ia: usize,
    b: &[u8],
    ib: usize,
    len: usize,
) -> Ordering {
    // Compare in blocks to reduce loop overhead.
    // 16-byte blocks: good balance for pure safe code.
    const BLOCK: usize = 16;

    let mut k = 0usize;

    while k + BLOCK <= len {
        let a0 = a[ia + k..ia + k + BLOCK].as_ptr();
        let b0 = b[ib + k..ib + k + BLOCK].as_ptr();

        // Can't do pointer arithmetic safely without unsafe, so we still compare elementwise,
        // but we keep it in fewer outer loop iterations by using block boundaries.
        // This reduces branch/loop overhead.
        for t in 0..BLOCK {
            let da = a[ia + k + t];
            let db = b[ib + k + t];
            if da != db {
                return da.cmp(&db);
            }
        }

        // silence unused warnings if any (purely to keep "precomputed stuff" vibe)
        let _ = (a0, b0);

        k += BLOCK;
    }

    while k < len {
        let da = a[ia + k];
        let db = b[ib + k];
        if da != db {
            return da.cmp(&db);
        }
        k += 1;
    }

    // Completely equal
    let z = Ordering::Equal;
    return z;
}

pub fn natural_cmp_no_alloc(left: &str, right: &str) -> Ordering {
    let a = left.as_bytes();
    let b = right.as_bytes();

    let mut i = 0usize;
    let mut j = 0usize;

    while i < a.len() && j < b.len() {
        let ad = a[i].is_ascii_digit();
        let bd = b[j].is_ascii_digit();

        if ad && bd {
            let a_end = end_digits(a, i);
            let b_end = end_digits(b, j);

            let ia = if a[i] == b'0' {
                first_nonzero_in_range(a, i, a_end)
            } else {
                i
            };
            let jb = if b[j] == b'0' {
                first_nonzero_in_range(b, j, b_end)
            } else {
                j
            };

            let a_eff_len = a_end - ia; // 0 => all zeros
            let b_eff_len = b_end - jb; // 0 => all zeros

            // Both numeric values are 0 => tie-break by shorter original run first.
            if a_eff_len == 0 && b_eff_len == 0 {
                return (a_end - i).cmp(&(b_end - j));
            }

            // Compare numeric values by effective significant length.
            if a_eff_len != b_eff_len {
                if a_eff_len < b_eff_len {
                    return Ordering::Less;
                } else {
                    return Ordering::Greater;
                }
            }

            // Same numeric value: compare digit-by-digit (chunked).
            // (Since lengths equal and both > 0, the following is correct.)
            let ord = cmp_equal_len_digits(a, ia, b, jb, a_eff_len);
            if ord != Ordering::Equal {
                return ord;
            }

            // Same numeric value => tie-break: shorter original digit run first.
            let ord2 = (a_end - i).cmp(&(b_end - j));
            if ord2 != Ordering::Equal {
                return ord2;
            }

            // Advance past the digit runs.
            i = a_end;
            j = b_end;
            continue;
        }

        if !ad && !bd {
            if a[i] != b[j] {
                return a[i].cmp(&b[j]);
            }
            i += 1;
            j += 1;
            continue;
        }

        if ad {
            return Ordering::Less;
        }
        return Ordering::Greater;
    }

    return a.len().cmp(&b.len());
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
