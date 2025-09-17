pub fn package_merge(counts: &[u32], max_len: u8) -> Vec<u8> {
    let mut symbol2count: Vec<_> = counts
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, count)| count != 0)
        .collect();
    // sort symbols by counts
    symbol2count.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
    let sorted_counts: Vec<_> = symbol2count.iter().map(|&x| x.1).collect();

    assert!(sorted_counts.len() != 0, "No symbols provided");
    assert!(max_len <= 32, "Max length is too big"); // can be 64 for quad words
    assert!(
        sorted_counts.len() <= 1 << max_len,
        "Max length is too small"
    );
    let sorted_code_lens = package_merge_sorted(&sorted_counts, max_len);

    // un-sort the code lens to their symbols
    let mut code_lens = vec![0; counts.len()];
    symbol2count
        .iter()
        .map(|x| x.0)
        .zip(sorted_code_lens)
        .for_each(|(sym, code_len)| code_lens[sym] = code_len);

    code_lens
}

// Inspired by
// https://create.stephan-brumme.com/length-limited-prefix-codes/#package-merge
// https://github.com/sellibitze/packagemerge-rs/blob/27adc64e3a8b51b86ea91449c6a4c1971af7c682/src/lib.rs
fn package_merge_sorted(a: &[u32], max_len: u8) -> Vec<u8> {
    let mut package_depths: Vec<u32> = vec![0; a.len() * 2 - 1];
    let mut curr: Vec<u32> = a.iter().copied().collect();
    let mut next = Vec::with_capacity(a.len() * 2 - 1);

    for depth in 1..max_len {
        let mut seq = a.iter().peekable(); // always merge with the initial counts
        let mut packages = curr.chunks_exact(2).map(|x| x[0] + x[1]).peekable();

        // merge packages from curr with initial sequence
        loop {
            next.push(match (seq.peek(), packages.peek()) {
                (None, None) => break,
                (Some(&a), Some(b)) if a < b => seq.next().copied().unwrap(),
                (_, None) => seq.next().copied().unwrap(),
                _ => {
                    package_depths[next.len()] |= 1 << depth;
                    packages.next().unwrap()
                }
            });
        }

        std::mem::swap(&mut curr, &mut next);
        next.clear();
    }

    let mut code_lens = vec![0; a.len()];
    let (mut depth, mut relevant_symbols) = (max_len, a.len() * 2 - 2);
    while relevant_symbols > 0 && depth > 0 {
        depth -= 1;
        let mut sym = 0;
        for i in 0..relevant_symbols {
            if package_depths[i] & (1 << depth) == 0 {
                code_lens[sym] += 1;
                sym += 1; // move to the next non-packaged symbol
            }
        }
        relevant_symbols = (relevant_symbols - sym) << 1;
    }
    code_lens
}

// TODO: write tests
pub fn canonical(code_lens: &[u8]) -> Vec<(u16, u8)> {
    let mut symbol2code_lens: Vec<_> = code_lens
        .iter()
        .enumerate()
        .filter(|(_, &x)| x != 0)
        .collect();
    symbol2code_lens.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

    let max_len = code_lens
        .iter()
        .reduce(|acc, x| acc.max(x))
        .map(|&x| usize::from(x))
        .unwrap_or(0);

    let mut count_lens = vec![0; max_len + 1];
    symbol2code_lens
        .iter()
        .map(|x| x.1)
        .for_each(|&code_len| count_lens[usize::from(code_len)] += 1);

    let mut codes: Vec<u16> = vec![0; max_len + 1];
    for i in 0..max_len {
        codes[i + 1] = (codes[i] + count_lens[i]) << 1;
    }

    let mut res = vec![(0, 0); code_lens.len()];
    for (sym, &code_len) in symbol2code_lens {
        res[sym] = (codes[usize::from(code_len)], code_len);
        codes[usize::from(code_len)] = codes[usize::from(code_len)].wrapping_add(1);
    }
    res
}

fn package_merge_canonical(_counts: &[u32], _max_len: u8) -> Vec<(u16, u8)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sellibitze_example() {
        let counts = [1, 32, 16, 4, 8, 2, 1];
        assert_eq!(package_merge(&counts, 8), [6, 1, 2, 4, 3, 5, 6]);
        assert_eq!(package_merge(&counts, 5), [5, 1, 2, 5, 3, 5, 5]);
    }

    #[test]
    fn stephan_brumme_example() {
        let counts = [270, 20, 10, 0, 1, 6, 1];
        assert_eq!(package_merge(&counts, 4), [1, 2, 4, 0, 4, 4, 4]);
        let counts = [10, 20, 270, 0, 1, 6, 1];
        assert_eq!(package_merge(&counts, 4), [4, 2, 1, 0, 4, 4, 4]);
    }

    #[test]
    fn book1() {
        let book1_counts = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16622, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 125551, 832, 2468, 0, 0, 0, 1, 6470, 43, 40, 1, 691, 10296, 3955, 7170, 0,
            98, 240, 185, 184, 151, 96, 87, 85, 85, 82, 220, 762, 498, 5, 498, 759, 0, 967, 1463,
            580, 269, 444, 413, 575, 977, 2899, 253, 45, 413, 565, 502, 856, 693, 14, 245, 850,
            1966, 103, 64, 753, 5, 416, 0, 0, 0, 0, 0, 0, 0, 47836, 9132, 12685, 26623, 72431,
            12237, 12303, 37561, 37007, 468, 4994, 23078, 14044, 40919, 44795, 9332, 520, 32889,
            36788, 50027, 16031, 5382, 14071, 861, 11986, 264, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let code_lens = [
            12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0,
            0, 0, 0, 0, 3, 10, 8, 0, 0, 0, 12, 7, 12, 12, 12, 10, 6, 7, 7, 0, 12, 12, 12, 12, 12,
            12, 12, 12, 12, 12, 12, 10, 10, 12, 10, 10, 0, 10, 9, 10, 11, 11, 11, 10, 10, 8, 11,
            12, 11, 10, 10, 10, 10, 12, 12, 10, 9, 12, 12, 10, 12, 11, 0, 0, 0, 0, 0, 0, 0, 4, 6,
            6, 5, 3, 6, 6, 4, 4, 11, 7, 5, 6, 4, 4, 6, 10, 5, 5, 4, 6, 7, 6, 10, 6, 11, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(package_merge(&book1_counts, 12), code_lens);
    }

    #[test]
    fn single_symbol() {
        for max_len in [1, 2, 8] {
            assert_eq!(package_merge(&[1], max_len), [0]);
            assert_eq!(package_merge(&[10], max_len), [0]);
        }
    }

    #[test]
    fn two_symbols() {
        for max_len in [1, 2, 8] {
            assert_eq!(package_merge(&[1, 1], max_len), [1, 1]);
            assert_eq!(package_merge(&[10, 10], max_len), [1, 1]);
            assert_eq!(package_merge(&[1, 100], max_len), [1, 1]);
        }
    }

    #[test]
    #[should_panic(expected = "No symbols provided")]
    fn no_symbols() {
        assert_eq!(package_merge(&[], 8), &[]);
    }

    #[test]
    #[should_panic(expected = "Max length is too big")]
    fn max_len_too_big() {
        package_merge(&[1, 1, 2, 4, 8, 16, 32], 33);
    }

    #[test]
    #[should_panic(expected = "Max length is too small")]
    fn max_len_too_small() {
        package_merge(&[1, 1, 2, 4, 8, 16, 32], 2);
    }

    #[test]
    fn check_canonical_sorted() {
        let code_lens = [2, 2, 2, 3, 3];
        let codes = canonical(&code_lens);
        assert_eq!(codes, [(0, 2), (1, 2), (2, 2), (6, 3), (7, 3)]);
    }

    #[test]
    fn check_canonical_unsorted() {
        let code_lens = [2, 3, 2, 3, 2];
        let codes = canonical(&code_lens);
        assert_eq!(codes, [(0, 2), (6, 3), (1, 2), (7, 3), (2, 2)]);
    }

    #[test]
    fn check_canonical_zeroes() {
        let code_lens = [
            7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 3, 7, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 5, 7, 7, 6, 4, 7, 7, 5, 5, 7, 7, 6, 7, 5, 5, 7, 7, 6, 5,
            5, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let codes = canonical(&code_lens);
        for (code, len) in codes {
            assert!(code <= (1 << len));
        }
    }

    #[test]
    fn test_canonical_max_len() {
        let counts: Vec<u32> = (0..=17).map(|x| 1 << x).collect();
        let codes = package_merge(&counts, 16);
        let canonical_codes = canonical(&codes);
        let expected = [
            (0b1111111111111100, 16),
            (0b1111111111111101, 16),
            (0b1111111111111110, 16),
            (0b1111111111111111, 16),
            (0b11111111111110, 14),
            (0b1111111111110, 13),
            (0b111111111110, 12),
            (0b11111111110, 11),
            (0b1111111110, 10),
            (0b111111110, 9),
            (0b11111110, 8),
            (0b1111110, 7),
            (0b111110, 6),
            (0b11110, 5),
            (0b1110, 4),
            (0b110, 3),
            (0b10, 2),
            (0b0, 1),
        ];
        assert_eq!(canonical_codes, expected);
    }
}
