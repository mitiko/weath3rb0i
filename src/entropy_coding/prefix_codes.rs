// returns max code length
fn canon_huff_in_place(sorted_histogram: &mut [u32]) -> u32 {
    debug_assert!(
        sorted_histogram.iter().all(|&x| x != 0),
        "All entries of histogram must be non-zero."
    );
    debug_assert!(
        sorted_histogram.windows(2).all(|x| x[0] <= x[1]),
        "Histogram must be sorted."
    );
    debug_assert!(
        sorted_histogram.len() >= 1,
        "Histogram cannot be empty or trivial."
    ); // TODO: remove triviality restriction
    debug_assert!(
        sorted_histogram.len() <= usize::try_from(u32::MAX).unwrap_or(usize::MAX),
        "Histogram must be at most 2^32 elements"
    );

    let (len, a) = (sorted_histogram.len(), sorted_histogram);
    let (mut leaf_range, mut node_range) = (0..len, 0..0);

    // phase 1 (create Huffman tree)
    for i in 0..len {
        node_range.end = i; // we emit 1 node per loop iteration

        // assign on first iteration, add on second
        for is_left_child in [true, false] {
            let leafs_exist = !leaf_range.is_empty();
            let prefer_leafs =
                || node_range.is_empty() || a[leaf_range.start] <= a[node_range.start]; // lazy
            let use_leafs = leafs_exist && prefer_leafs();

            // TODO: wrong if node_range.end == node_range.start || node_range.end == leaf_range.start
            if is_left_child {
                a[node_range.end] = 0; // reset
            }

            if use_leafs {
                a[node_range.end] += a[leaf_range.start];
                leaf_range.start += 1;
            } else {
                a[node_range.end] += a[node_range.start];
                a[node_range.start] = u32::try_from(i).unwrap(); // pointer
                node_range.start += 1;
            }
        }
    }

    // phase 2 (BFS to find internal node depths)
    let root_index = len - 2; // n leafs => n - 1 internal nodes
    a[root_index] = 0; // depth of root is 0
    for j in (0..root_index).rev() {
        let parent = usize::try_from(a[j]).unwrap();
        a[j] = a[parent] + 1; // depth of child is 1 more than depth of parent
    }
    debug_assert!(
        a.windows(2).all(|x| x[0] <= x[1]),
        "Array of node depths (post phase 2) must be sorted."
    );

    // phase 3 (internal node depths to leaf node depths)
    let mut avail = 1;
    let mut used = 0;
    let mut depth = 0;

    let mut root = isize::try_from(root_index).unwrap();
    let mut next = usize::try_from(len - 1).unwrap();
    while avail > 0 {
        while root >= 0 && a[usize::try_from(root_index).unwrap()] == depth {
            used += 1;
            root -= 1;
        }
        while avail > used {
            a[next] = depth;
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
