fn package_merge(counts: &[u32], max_len: u8) -> Vec<u8> {
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
    assert!(max_len <= 32, "Max length is too big");
    assert!(
        sorted_counts.len() <= 1 << max_len,
        "Max length is too small"
    );

    let sorted_code_lens = package_merge_sorted(&sorted_counts, max_len);
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
    let mut prev: Vec<u32> = a.iter().copied().collect();
    let mut curr = Vec::with_capacity(a.len() * 2 - 1);

    for depth in 1..max_len {
        let mask = 1 << depth; // records at which depth it was packaged
        let mut seq = a.iter().peekable(); // always merge with the initial counts
        let mut packages = prev.chunks_exact(2).map(|x| x[0] + x[1]).peekable();
        curr.clear(); //

        // merge iteration
        loop {
            let is_package = match (packages.peek(), seq.peek()) {
                (None, None) => break,
                (None, _) => false,
                (_, None) => true,
                (Some(a), Some(&b)) => a <= b,
            };
            let next_item = if is_package {
                package_depths[curr.len()] |= mask;
                packages.next().unwrap()
            } else {
                seq.next().copied().unwrap()
            };
            curr.push(next_item);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let mut code_lens = vec![0; a.len()];
    let mut relevant_symbols = a.len() * 2 - 2;
    for depth in (0..max_len).rev() {
        if relevant_symbols == 0 {
            break;
        }
        let mask = 1 << depth;
        let mut packaged = 0;
        for sym in 0..relevant_symbols {
            // if it hasn't been packaged, we increase it's code length
            if package_depths[sym] & mask == 0 {
                code_lens[sym - packaged] += 1;
            } else {
                packaged += 1;
            }
        }
        relevant_symbols = packaged * 2;
    }
    code_lens
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
    #[should_panic(expected = "Max length is too small")]
    fn max_len_too_small() {
        package_merge(&[1, 1, 2, 4, 8, 16, 32], 2);
    }

    #[test]
    #[should_panic(expected = "Max length is too big")]
    fn max_len_too_big() {
        package_merge(&[1, 1, 2, 4, 8, 16, 32], 33);
    }

    #[test]
    #[should_panic(expected = "No symbols provided")]
    fn no_symbols() {
        assert_eq!(package_merge(&[], 8), &[]);
    }
}
