use std::cmp::Ordering;

fn run_end_digit(s: &[u8], mut i: usize) -> usize {
    while i < s.len() && s[i].is_ascii_digit() {
        i += 1;
    }
    return i;
}

// Compares the digit run starting at (i0, j0).
// Returns (ordering, i_after_run, j_after_run).
fn cmp_digit_run(
    a: &[u8],
    i0: usize,
    b: &[u8],
    j0: usize,
) -> (Ordering, usize, usize) {
    let a_end = run_end_digit(a, i0);
    let b_end = run_end_digit(b, j0);

    let mut ia = i0;
    while ia < a_end && a[ia] == b'0' {
        ia += 1;
    }
    let mut jb = j0;
    while jb < b_end && b[jb] == b'0' {
        jb += 1;
    }

    let a_eff_len = a_end - ia;
    let b_eff_len = b_end - jb;

    let ord = if a_eff_len != b_eff_len {
        if a_eff_len < b_eff_len {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    } else {
        // same numeric value (ignoring leading zeros)
        let mut k = 0usize;
        while k < a_eff_len {
            let da = a[ia + k];
            let db = b[jb + k];
            if da != db {
                let r = da.cmp(&db);
                return (r, a_end, b_end);
            }
            k += 1;
        }
        // same numeric value => shorter original digit run first
        (a_end - i0).cmp(&(b_end - j0))
    };

    return (ord, a_end, b_end);
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
            let (ord, i2, j2) = cmp_digit_run(a, i, b, j);
            if ord != Ordering::Equal {
                return ord;
            }
            i = i2;
            j = j2;
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
