use std::cmp::Ordering;

// The tie-break above is annoying; here’s a cleaner integrated version:
fn cmp_digit_run_clean(a: &[u8], i0: usize, b: &[u8], j0: usize) -> Ordering {
    let mut i = i0;
    while i < a.len() && a[i].is_ascii_digit() { i += 1; }
    let a_end = i;

    let mut j = j0;
    while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    let b_end = j;

    let mut ia = i0;
    while ia < a_end && a[ia] == b'0' { ia += 1; }
    let mut jb = j0;
    while jb < b_end && b[jb] == b'0' { jb += 1; }

    let a_eff_len = a_end - ia;
    let b_eff_len = b_end - jb;

    match a_eff_len.cmp(&b_eff_len) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => {
            for k in 0..a_eff_len {
                let da = a[ia + k];
                let db = b[jb + k];
                if da != db {
                    return da.cmp(&db);
                }
            }
            // same numeric value => shorter original digit run first ("1" < "01")
            (a_end - i0).cmp(&(b_end - j0))
        }
    }
}

pub fn natural_cmp_no_alloc(left: &str, right: &str) -> Ordering {
    let a = left.as_bytes();
    let b = right.as_bytes();

    let mut i = 0usize;
    let mut j = 0usize;

    while i < a.len() && j < b.len() {
        let ad = a[i].is_ascii_digit();
        let bd = b[j].is_ascii_digit();

        match (ad, bd) {
            (true, true) => {
                let ord = cmp_digit_run_clean(a, i, b, j);
                if ord != Ordering::Equal {
                    return ord;
                }
                while i < a.len() && a[i].is_ascii_digit() { i += 1; }
                while j < b.len() && b[j].is_ascii_digit() { j += 1; }
            }
            (false, false) => {
                if a[i] != b[j] {
                    return a[i].cmp(&b[j]);
                }
                i += 1;
                j += 1;
            }
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
        }
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
    //v.join("\r\n");
}
