// returns max code length
fn canon_huff_in_place(sorted_histogram: &mut [u32]) -> u32 {
    debug_assert!(sorted_histogram.iter().all(|&x| x != 0), "All entries of histogram must be non-zero.");
    debug_assert!(sorted_histogram.windows(2).all(|x| x[0] <= x[1]), "Histogram must be sorted.");
    debug_assert!(sorted_histogram.len() >= 1, "Histogram cannot be empty or trivial."); // TODO: remove triviality restriction

    let len = sorted_histogram.len();
    let a = sorted_histogram;

    let mut leaf = 0;
    let mut root = 0;
    for i in 0..len {
        // first child
        if leaf >= len || (root < i && a[root] < a[leaf]) {
            a[i] = a[root];
            a[root] = u32::try_from(i).unwrap();
            root += 1;
        }
        else {
            a[i] = a[leaf];
            leaf += 1;
        }

        // second child
        if leaf >= len || (root < i && a[root] < a[leaf]) {
            a[i] += a[root];
            a[root] = u32::try_from(i).unwrap();
            root += 1;
        }
        else {
            a[i] += a[leaf];
            leaf += 1;
        }
    }

    // phase 2
    a[len - 2] = 0;
    for j in (0..(len - 2)).rev() {
        a[j] = a[usize::try_from(a[j]).unwrap()] + 1;
    }

    let mut avail = 1;
    let mut used = 0;
    let mut depth = 0;

    let mut root2 = isize::try_from(len - 2).unwrap();
    let mut next = isize::try_from(len - 1).unwrap();
    while avail > 0 {
        while root2 >= 0 && a[usize::try_from(root2).unwrap()] == depth {
            used += 1;
            root2 -= 1;
        }
        while avail > used {
            a[usize::try_from(next).unwrap()] = depth;
            next -= 1;
            avail -= 1;
        }

        avail = 2 * used;
        depth += 1;
        used = 0;
    }

    a[0]
}

pub fn huff_in_place(histogram: &mut [u32]) -> u32 {
    let mut mapping: Vec<_> = histogram
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, count)| count != 0)
        .collect();

    mapping.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
    let mut sorted_histogram: Vec<_> = mapping.iter().map(|x| x.1).collect();
    dbg!(&sorted_histogram);
    let max = canon_huff_in_place(&mut sorted_histogram);
    mapping
        .into_iter()
        .map(|(i, _)| i)
        .zip(sorted_histogram)
        .for_each(|(i, len)| histogram[i] = len);
    max
}
