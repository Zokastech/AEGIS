// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Context windows in **bytes**, realigned to UTF‑8 boundaries.
//! Otherwise `str::get(lo..hi)` may be `None` (split inside a multibyte character)
//! and recognizers skip detection.

/// Returns `text[lo..hi]` after shifting `lo` and `hi` to character boundaries.
pub(crate) fn byte_window_utf8<'a>(text: &'a str, lo: usize, hi: usize) -> &'a str {
    let n = text.len();
    let lo = lo.min(n);
    let mut hi = hi.min(n);
    if lo >= hi {
        return "";
    }
    let mut lo2 = lo;
    while lo2 < n && !text.is_char_boundary(lo2) {
        lo2 += 1;
    }
    while hi > lo2 && !text.is_char_boundary(hi) {
        hi -= 1;
    }
    if lo2 >= hi {
        return "";
    }
    text.get(lo2..hi).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn misaligned_lo_skips_to_char() {
        let t = "a—b"; // em dash U+2014 = 3 bytes ; octets 0=a, 1–3=—, 4=b
        assert_eq!(t.len(), 5);
        // lo=2 lands mid em dash → window realigned
        let w = byte_window_utf8(t, 2, 5);
        assert_eq!(w, "b");
    }
}
