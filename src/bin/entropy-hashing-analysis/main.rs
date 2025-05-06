use std::collections::HashMap;
use weath3rb0i::{
    history::{ACHistory, History, HuffHistory, RawHistory},
    models::ac_hash::StationaryModel,
};

fn main() -> std::io::Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    let skip_stats = true;
    let skip_locations = true;

    // for contexts of size o0-o24 (bitwise) =>
    // get stats for level 2 history of
    // most common, most predictable, and most expensive contexts
    if !skip_stats {
        let raw_history_stats = get_stats_for(RawHistory::new(), "raw")?;
        let huff_history_stats = get_stats_for(HuffHistory::new(&buf, 12, 12), "huffman")?;
        let ac_history_stats = get_stats_for(ACHistory::new(24, StationaryModel::new(&buf)), "ac")?;
        let json = serde_json::json!({
            "raw_history": raw_history_stats,
            "huff_history": huff_history_stats,
            "ac_history": ac_history_stats,
        });
        let json = serde_json::to_string_pretty(&json).unwrap();
        std::fs::write("stats.json", json.as_bytes())?;
    }

    // find all locations where the huffman history hashes to X for
    // most common, most predictable, and most expensive contexts
    // so that we can decompress the context & see what is being hashed
    if !skip_locations {
        let raw_history_locs = get_locations_for(RawHistory::new(), "raw")?;
        let huff_history_locs = get_locations_for(HuffHistory::new(&buf, 12, 12), "huffman")?;
        let ac_history_locs =
            get_locations_for(ACHistory::new(16, StationaryModel::new(&buf)), "ac")?;
        let json = serde_json::json!({
            "raw_history": raw_history_locs,
            "huff_history": huff_history_locs,
            "ac_history": ac_history_locs,
        });
        let json = serde_json::to_string_pretty(&json).unwrap();
        std::fs::write("locations.json", json.as_bytes())?;
    }

    Ok(())
}

fn get_locations_for<H: History + Clone>(
    h: H,
    history_name: &str,
) -> std::io::Result<serde_json::Value> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    let max_ctx = 32;
    let max_locations = 100;

    let mut stats = serde_json::Value::Array(Vec::new());
    for bits in [4, 8, 12, 16] {
        println!("[{history_name}] Finding important contexts of o{bits} ...");

        let mask = (1 << bits) - 1;
        let mut history = MaskedHistory::new(h.clone(), mask);
        let entropy_counts = get_entropy(&buf, history.clone());

        // get the contexts which are important
        let mut v = entropy_counts
            .iter()
            .enumerate()
            .map(|(ctx, &(entropy, count))| (ctx as u32, entropy, count, entropy * count as f64))
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let most_predictable_ctx = v
            .iter()
            .filter(|x| x.1 != 0.0)
            .map(|x| x.0)
            .take(max_ctx)
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.2.cmp(&b.2).reverse());
        let most_common_ctx = v.iter().map(|x| x.0).take(max_ctx).collect::<Vec<_>>();
        v.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap().reverse());
        let most_expensive_ctx = v.iter().map(|x| x.0).take(max_ctx).collect::<Vec<_>>();

        println!("[{history_name}] Collecting locations for contexts of o{bits} ...");

        // maps context to locations where it appears, capped at max_locations
        let mut ctx_map: HashMap<u32, Vec<usize>> = most_common_ctx
            .iter()
            .chain(most_expensive_ctx.iter())
            .chain(most_predictable_ctx.iter())
            .map(|&ctx| (ctx, Vec::new()))
            .collect();

        // run history again & store the locations of the important contexts
        let mut pos = 0;
        for &byte in buf.iter() {
            for i in (0..8).rev() {
                let hash = history.hash();
                if let Some(locations) = ctx_map.get_mut(&hash) {
                    if locations.len() < max_locations {
                        locations.push(pos);
                    }
                }
                let bit = (byte >> i) & 1;
                pos += 1;
                history.update(bit);
            }
        }

        let obj = serde_json::json!({
            "mask": mask,
            "most_predictable_ctx": most_predictable_ctx,
            "most_common_ctx": most_common_ctx,
            "most_expensive_ctx": most_expensive_ctx,
            "locations": ctx_map,
        });
        stats.as_array_mut().unwrap().push(obj);
    }

    Ok(stats)
}

fn get_stats_for<H: History + Clone>(
    h: H,
    history_name: &str,
) -> std::io::Result<serde_json::Value> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    let max_stats = 32;

    // calculate entropy at context for different level 1 histories
    let mut stats = serde_json::Value::Array(Vec::new());
    for bits in 0..=24 {
        let mask = (1 << bits) - 1;
        let history = MaskedHistory::new(h.clone(), mask);
        // for each context, this stores entropy & bit count of level 2 history
        let entropy_counts = get_entropy(&buf, history);

        // meta structure to store everything
        let mut v = entropy_counts
            .iter()
            .enumerate()
            .map(|(ctx, &(entropy, count))| (ctx, entropy, count, entropy * count as f64))
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let most_predictable = v
            .iter()
            .filter(|x| x.1 != 0.0)
            .take(max_stats)
            .copied()
            .collect::<Vec<_>>();
        v.sort_by(|a, b| a.2.cmp(&b.2).reverse());
        let most_common = v.iter().take(max_stats).copied().collect::<Vec<_>>();
        v.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap().reverse());
        let most_expensive = v.iter().take(max_stats).copied().collect::<Vec<_>>();

        let obj = serde_json::json!({
            "mask": mask,
            "most_predictable": most_predictable,
            "most_common": most_common,
            "most_expensive": most_expensive,
        });
        stats.as_array_mut().unwrap().push(obj);

        let weighted_entropy = weighted_sum(entropy_counts);
        println!(
            "[{history_name}] Bitwise order-{} has entropy: {} bits/bit",
            bits, weighted_entropy
        );
    }
    Ok(stats)
}

// Calculates history at each byte boundary, then calculates entropy for the level 2 history of the context
fn get_entropy(buf: &[u8], mut history: impl History) -> Vec<(f64, u32)> {
    let mut calculators = Vec::new();

    for &byte in buf.iter() {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;

            let hash = history.hash();
            if hash >= calculators.len() as u32 {
                let new_size = if hash.is_power_of_two() {
                    hash << 1
                } else {
                    hash.next_power_of_two()
                };
                calculators.resize(new_size as usize, EntropyCalculator::new());
            }
            calculators[hash as usize].update(bit);

            history.update(bit);
        }
    }

    calculators
        .into_iter()
        .map(|ec| (ec.entropy(), ec.total()))
        .collect()
}

fn weighted_sum(entropy_counts: Vec<(f64, u32)>) -> f64 {
    let total_count = entropy_counts.iter().map(|&(_, count)| count).sum::<u32>() as f64;
    entropy_counts
        .iter()
        .map(|&(entropy, count)| entropy * (count as f64 / total_count))
        .sum()
}

#[derive(Clone)]
struct MaskedHistory<H: History> {
    history: H,
    mask: u32,
}

impl<H: History> MaskedHistory<H> {
    fn new(history: H, mask: u32) -> Self {
        Self { history, mask }
    }
}

impl<H: History> History for MaskedHistory<H> {
    fn update(&mut self, bit: u8) {
        self.history.update(bit);
    }

    fn hash(&mut self) -> u32 {
        self.history.hash() & self.mask
    }
}

#[derive(Clone)]
struct EntropyCalculator {
    data: [u16; 2],
}

impl EntropyCalculator {
    pub fn new() -> Self {
        Self { data: [0; 2] }
    }

    pub fn entropy(&self) -> f64 {
        let c0 = f64::from(self.data[0]);
        let c1 = f64::from(self.data[1]);
        if c0 + c1 == 0.0 {
            return 0.0;
        }
        if c0 == 0.0 || c1 == 0.0 {
            return 2.0 / (c0 + c1);
        }
        let p = c1 / (c0 + c1);
        -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
    }

    pub fn total(&self) -> u32 {
        u32::from(self.data[0]) + u32::from(self.data[1])
    }

    pub fn update(&mut self, bit: u8) {
        self.data[usize::from(bit)] += 1;
        if self.data[usize::from(bit)] == u16::MAX {
            self.data[0] = (self.data[0] >> 1) + (self.data[0] & 1);
            self.data[1] = (self.data[1] >> 1) + (self.data[1] & 1);
        }
    }
}
