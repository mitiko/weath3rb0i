// fn package_merge(counts: Vec<u32>, max_len: u8) {
//     let mut symbol_map: Vec<_> = counts
//         .iter()
//         .copied()
//         .enumerate()
//         .filter(|&(_, count)| count != 0)
//         .collect();
//     // sort symbols by counts
//     symbol_map.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
//     let sorted_counts: Vec<_> = symbol_map.iter().map(|&x| x.1).collect();
//     let code_lens = package_merge_sorted(&sorted_counts, max_len);

//     todo!()
//     // do some un-sorting
//     // counts.sort();
// }

// Inspired by
// https://create.stephan-brumme.com/length-limited-prefix-codes/#package-merge
// https://github.com/sellibitze/packagemerge-rs/blob/27adc64e3a8b51b86ea91449c6a4c1971af7c682/src/lib.rs
fn package_merge_sorted(a: &[u32], max_len: u8) -> Vec<u32> {
    assert!(a.len() <= 1 << max_len, "Max length is too small");
    assert!(max_len <= 32, "Max length is too big");

    let mut package_depths: Vec<u32> = vec![0; a.len() * 2 - 1];
    let mut prev: Vec<u32> = a.iter().copied().collect();

    for depth in 1..max_len {
        let mask = 1 << depth; // records at which depth it was packaged
        let mut seq = a.iter().peekable();
        let mut packages = prev.chunks_exact(2).map(|x| x[0] + x[1]).peekable();
        let mut curr = Vec::with_capacity(a.len() + prev.len() / 2 + 2);

        // merge packages with original sequence
        loop {
            // TODO: refactor
            let (next_item, is_package) = match (seq.peek(), packages.peek()) {
                (None, None) => break,
                (_, None) => (seq.next().copied(), false), // merged all packages
                (None, _) => (packages.next(), true),      // merged original sequence
                (Some(&a), Some(b)) => {
                    if a <= b {
                        (seq.next().copied(), false)
                    } else {
                        (packages.next(), true)
                    }
                }
            };
            if is_package {
                package_depths[curr.len()] |= mask;
            }
            curr.push(next_item.unwrap());
        }
        prev = curr; // TODO: mem swap for efficiency
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
        let mut counts = [1, 32, 16, 4, 8, 2, 1];
        counts.sort(); // TODO:
        // assert_eq!(package_merge(&counts, 8), [6, 1, 2, 4, 3, 5, 6]); // TODO:
        assert_eq!(package_merge_sorted(&counts, 8), [6, 6, 5, 4, 3, 2, 1]);
        // assert_eq!(package_merge(&counts, 5), [5, 1, 2, 5, 3, 5, 5]); // TODO:
        assert_eq!(package_merge_sorted(&counts, 5), [5, 5, 5, 5, 3, 2, 1]);
    }

    #[test]
    fn stephan_brumme_example() {
        // let mut counts = [270, 20, 10, 0, 1, 6, 1]; // TODO:
        let counts = [1, 1, 6, 10, 20, 270];
        assert_eq!(package_merge_sorted(&counts, 4), [4, 4, 4, 4, 2, 1]);
    }

    // TODO: enwik8, book1 frequencies

    #[test]
    fn single_symbol() {
        for max_len in [1, 2, 8] {
            assert_eq!(package_merge_sorted(&[1], max_len), [0]);
            assert_eq!(package_merge_sorted(&[10], max_len), [0]);
        }
    }

    #[test]
    fn two_symbols() {
        for max_len in [1, 2, 8] {
            assert_eq!(package_merge_sorted(&[1, 1], max_len), [1, 1]);
            assert_eq!(package_merge_sorted(&[10, 10], max_len), [1, 1]);
            assert_eq!(package_merge_sorted(&[1, 100], max_len), [1, 1]);
        }
    }

    #[test]
    #[should_panic(expected = "Max length is too small")]
    fn max_len_too_small() {
        package_merge_sorted(&[1, 1, 2, 4, 8, 16, 32], 2);
    }

    #[test]
    #[should_panic(expected = "Max length is too big")]
    fn max_len_too_big() {
        package_merge_sorted(&[1, 1, 2, 4, 8, 16, 32], 33);
    }
}
