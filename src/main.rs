use std::cmp::Ordering;

fn end_digits(s: &[u8], mut i: usize) -> usize {
    while i < s.len() && s[i].is_ascii_digit() {
        i += 1;
    }
    return i;
}

fn first_nonzero_in_range(s: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    while i < end && s[i] == b'0' {
        i += 1;
    }
    return i;
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

            let ia = first_nonzero_in_range(a, i, a_end);
            let jb = first_nonzero_in_range(b, j, b_end);

            let a_eff_len = a_end - ia; // 0 means the whole run was zeros
            let b_eff_len = b_end - jb;

            // Fast path: both numeric values are 0 (all digits were '0')
            if a_eff_len == 0 && b_eff_len == 0 {
                return (a_end - i).cmp(&(b_end - j)); // tie-break: shorter run first
            }

            // Numeric compare by effective significant length (after trimming leading zeros).
            if a_eff_len != b_eff_len {
                if a_eff_len < b_eff_len {
                    return Ordering::Less;
                } else {
                    return Ordering::Greater;
                }
            }

            // Now both eff lens are equal and > 0 => compare digits.
            let mut k = 0usize;
            while k < a_eff_len {
                let da = a[ia + k];
                let db = b[jb + k];
                if da != db {
                    return da.cmp(&db);
                }
                k += 1;
            }

            // Same numeric value => tie-break: shorter original digit run first ("1" < "01")
            let ord = (a_end - i).cmp(&(b_end - j));
            if ord != Ordering::Equal {
                return ord;
            }

            // Advance past the digit runs and continue.
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

    let ord = a.len().cmp(&b.len());
    return ord;
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
