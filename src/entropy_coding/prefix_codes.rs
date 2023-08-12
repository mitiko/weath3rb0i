// returns max code length
fn canon_huff_in_place(sorted_histogram: &mut [u32]) -> u32 {
    debug_assert!(sorted_histogram.iter().all(|&x| x != 0), "All entries of histogram must be non-zero.");
    debug_assert!(sorted_histogram.windows(2).all(|x| x[0] <= x[1]), "Histogram must be sorted.");
    debug_assert!(sorted_histogram.len() >= 1, "Histogram cannot be empty or trivial."); // TODO: remove triviality restriction
    debug_assert!(sorted_histogram.len() <= usize::try_from(u32::MAX).unwrap_or(usize::MAX), "Histogram must be at most 2^32 elements");

    let (len, a) = (sorted_histogram.len(), sorted_histogram);

    let mut leaf_range = 0..len;
    let mut node_range = 0..0;
    let mut x_leaf = 0;
    let mut x_root = 0;

    for i in 0..len {
        node_range.end = i;

        let leafs_exist = !leaf_range.is_empty();
        let prefer_leafs = || node_range.is_empty() || a[leaf_range.start] <= a[node_range.start]; // lazy
        let use_leafs = leafs_exist && prefer_leafs();
        let x_cond = x_leaf == len || (x_root < i && a[x_root] < a[x_leaf]);
        assert!(leafs_exist == !(x_leaf == len));
        // if use_leafs != x_cond {
        //     dbg!(i);
        //     dbg!(&leaf_range);
        //     dbg!(&node_range);
        //     dbg!(x_leaf);
        //     dbg!(x_root);
        //     dbg!(leafs_exist);
        //     dbg!(prefer_leafs);
        //     dbg!(!(x_leaf == len));
        //     dbg!(x_root < i);
        //     dbg!(a[x_root] < a[x_leaf]);
        //     dbg!(x_cond);
        // }
        assert!(use_leafs == !x_cond);

        if use_leafs {
            a[i] = a[leaf_range.start];

            assert!(x_leaf == leaf_range.start);
            x_leaf += 1;
            leaf_range.start += 1;
        }
        else {
            a[i] = a[node_range.start];
            a[node_range.start] = u32::try_from(i).unwrap();

            assert!(x_root == node_range.start);
            x_root += 1;
            node_range.start += 1;
        }

        let leafs_exist = !leaf_range.is_empty();
        // if leaf_range.start == 82 {
        //     dbg!(leafs_exist);
        //     println!("coq");
        //     let prefer_leafs = node_range.is_empty() || a[leaf_range.start] <= a[node_range.start];
        //     println!("{}", prefer_leafs);
        // }
        let prefer_leafs = || node_range.is_empty() || a[leaf_range.start] <= a[node_range.start]; // lazy
        let use_leafs = leafs_exist && prefer_leafs();
        let x_cond = x_leaf == len || (x_root < i && a[x_root] < a[x_leaf]);
        // assert!(leafs_exist == !(x_leaf == len));
        // if use_leafs != x_cond {
        //     dbg!(i);
        //     dbg!(&leaf_range);
        //     dbg!(&node_range);
        //     dbg!(x_leaf);
        //     dbg!(x_root);
        //     dbg!(leafs_exist);
        //     dbg!(prefer_leafs);
        //     dbg!(!(x_leaf == len));
        //     dbg!(x_root < i);
        //     dbg!(a[x_root] < a[x_leaf]);
        //     dbg!(x_cond);
        // }
        assert!(use_leafs == !x_cond);

        if use_leafs {
            a[i] += a[leaf_range.start];

            assert!(x_leaf == leaf_range.start);
            x_leaf += 1;
            leaf_range.start += 1;
        }
        else {
            a[i] += a[node_range.start];
            a[node_range.start] = u32::try_from(i).unwrap();

            assert!(x_root == node_range.start);
            x_root += 1;
            node_range.start += 1;
        }
    }
    /*
    for i in 0..len {
        let leaf = singleton_weights[0];
        let root = node_weights[0];

        if singleton_weights.is_empty() || (!node_weigts.is_empty() && node_weigts[0] < singleton_weights[0]) {
            parent_nodes.push(node_weights[0]);
            node_weights[0] = i;
            node_weights = node_weights[1..];
        }
        else {
            parent_nodes.push(singleton_weights[0]);
            leaf_weights = leaf_weights[1..];
        }

        if singleton_weights.is_empty() || (!node_weigts.is_empty() && node_weigts[0] < singleton_weights[0]) {
            parent_nodes.last() += a[root];

        }
    }

     */
    // let mut range = 1..10;
    // println!("range: {:?}", range);
    // range.end += 1;
    // println!("range: {:?}", range);
    // range.start += 1;
    // println!("range: {:?}", range);


    // let mut leaf = 0;
    // let mut root = 0;
    // for i in 0..len {

        // for x in [false, true] {
        //     let c = leaf == len || (root < i && a[root] < a[leaf]);
        //     a[i] = if x { 0 } else { a[i] };
        //     a[i] += if c { a[root] } else { a[leaf] };
        //     if c { a[root] = u32::try_from(i).unwrap(); }
        //     if c { root += 1 } else { leaf += 1 };
        // }
        // first child

        // x_leaf == len         => leaf_range.is_empty()
        // x_root < i            => !node_range.is_empty()
        // a[x_root] < a[x_leaf] => a[node_range.start] <= a[leaf_range.start]
        // let use_nodes = leaf_range.is_empty() || (!node_range.is_empty() && a[node_range.start] <= a[leaf_range.start])
        // let use_leafs = !use_nodes = !leaf_range.is_empty() && !(node_range.is_empty() && a[node_range.start] <= a[leaf_range.start])
        // let use_leafs = leafs_exist && (!node_range.is_empty() || !(a[node_range.start] <= a[leaf_range.start]))
        //



        // let singleton_weights = &a[leaf..];
        // let node_weigts = &a[root..i];
        // let parent_nodes = &a[..root];
        // if singleton_weights.is_empty() || (!node_weigts.is_empty() && node_weigts[0] < singleton_weights[0]) {
        // if !singleton_weights.is_empty() && !(!node_weights.is_empty() && node_weights[0] < singleton_weights[0])) {
        //     B
        // }
        // else {
        //     A
        // }

        // if !singleton_weights.is_empty() && !(!node_weights.is_empty() && !(node_weights[0] >= singleton_weights[0])) {
        //     B
        // }
        // else {
        //     A
        // }

        // if (node_weights.is_empty() || node_weights[0] >= singleton_weights[0]) && !singleton_weights.is_empty() {
        //     B
        // }
        // else {
        //     A
        // }

        // debug_assert!(!(singleton_weights.is_empty() && node_weights.is_empty()));
        // let leafs_exist = !singleton_weights.is_empty();
        // let prefer_leafs = node_weights.is_empty() || singleton_weights[0] <= node_weights[0];
        // let use_leafs = prefer_leafs && leafs_exist;



        // let cond1 = leaf >= len || (root < i && a[root] < a[leaf]);
        // let diff1 = i - root;
        // if leaf > len {
        //     println!("EUR!!");
        // }
    //     if leaf != len {
    //         if root >= i {

    //         }
    //     }
    //     if leaf == len || (root < i && a[root] < a[leaf]) {
    //         a[i] = a[root];
    //         a[root] = u32::try_from(i).unwrap();
    //         root += 1;
    //     }
    //     else {
    //         a[i] = a[leaf];
    //         leaf += 1;
    //     }

    //     // second child
    //     // let cond2 = leaf >= len || (root < i && a[root] < a[leaf]);
    //     // let diff2 = i - root;
    //     if leaf == len || (root < i && a[root] < a[leaf]) {
    //         a[i] += a[root];
    //         a[root] = u32::try_from(i).unwrap();
    //         root += 1;
    //     }
    //     else {
    //         a[i] += a[leaf];
    //         leaf += 1;
    //     }

    //     // if cond1 != cond2 {
    //     //     println!("err - {}", i);
    //     // }
    //     // let ddiff = diff1 - diff2;
    //     // if !(ddiff == 0 || ddiff == 1) {
    //     //     println!("d1 = {}, d2 = {}, dd = {}", diff1, diff2, ddiff);
    //     // }
    //     // if !(diff1 == 0 || diff2 == 1) {
    //     //     println!("d1 = {}, d2 = {}", diff1, diff2);
    //     // }
    // }

    // for i in 0..len {
    //     debug_assert!(!(singleton_weights.is_empty() && node_weights.is_empty()));

    //     if singleton_weights.is_empty() || (!node_weights.is_empty() && node_weights[0] < singleton_weights[0]) {

    //     }
    //     else {

    //     }

    //     // if !singleton_weights.is_empty() 
    // }

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
