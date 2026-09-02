use super::History;
use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    helpers::RotatingBuffer,
    models::Model,
    u8,
};

#[derive(Clone)]
pub struct ACHistory<M: Model> {
    pos: usize,
    bits: u64,
    probs: RotatingBuffer<u16, 64>,
    processed_bits: usize,
    model: M,
}

impl<M: Model> ACHistory<M> {
    pub fn new(model: M) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            model,
        }
    }
}

impl<M: Model> History for ACHistory<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
        self.processed_bits = 0;
    }

    fn hash(&mut self, max_bits: u8) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter::new(max_bits);
        for i in 0..self.pos.min(64) {
            let bit = u8!((self.bits >> i) & 1);
            let p = self.probs[i];
            if ac.encode(bit, p, &mut writer).is_err() {
                self.processed_bits = i + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

/// Selects which history bits are worth encoding into the compressed hash.
#[derive(Clone, Copy, Debug)]
pub enum HistorySkipCondition {
    /// Skips predictions near 0.5
    Probability { lower: f64, upper: f64 },
    /// Encode when bit cost is within the inclusive range
    Entropy { lower: f64, upper: f64 },
    /// Encode when bit cost is outside the range
    EntropyExclusive { lower: f64, upper: f64 },
}

impl HistorySkipCondition {
    /// Returns whether the bit should be encoded.
    #[inline]
    pub fn evaluate(&self, bit: u8, prediction: u16) -> bool {
        let prediction = f64::from(prediction) / 65536.0;
        let probability = if bit == 0 {
            1.0 - prediction
        } else {
            prediction
        };

        match *self {
            Self::Probability { lower, upper } => {
                if bit == 0 {
                    prediction > upper
                } else {
                    prediction < lower
                }
            }
            Self::Entropy { lower, upper } => {
                let entropy = -probability.log2();
                entropy >= lower && entropy <= upper
            }
            Self::EntropyExclusive { lower, upper } => {
                let entropy = -probability.log2();
                entropy < lower || entropy > upper
            }
        }
    }
}

const SKIP_HISTORY_BITS: usize = 1024;
const SKIP_HISTORY_WORDS: usize = SKIP_HISTORY_BITS / u64::BITS as usize;

/// An entropy-coded history that only includes bits selected by a
/// [`HistorySkipCondition`]. Bits are kept in LSB order, with the newest bit
/// at index zero.
#[derive(Clone)]
pub struct ACHistorySkip<M: Model> {
    pos: usize,
    bits: [u64; SKIP_HISTORY_WORDS],
    probs: RotatingBuffer<u16, SKIP_HISTORY_BITS>,
    processed_bits: usize,
    condition: HistorySkipCondition,
    model: M,
}

impl<M: Model> ACHistorySkip<M> {
    pub fn new(model: M, condition: HistorySkipCondition) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: [0; SKIP_HISTORY_WORDS],
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            condition,
            model,
        }
    }
}

impl<M: Model> History for ACHistorySkip<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        for word in (1..self.bits.len()).rev() {
            self.bits[word] = (self.bits[word] << 1) | (self.bits[word - 1] >> 63);
        }
        self.bits[0] = (self.bits[0] << 1) | u64::from(bit);
        self.pos += 1;
        self.processed_bits = 0;
    }

    fn hash(&mut self, max_bits: u8) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter::new(max_bits);

        for i in 0..self.pos.min(SKIP_HISTORY_BITS) {
            let bit = u8!((self.bits[i / 64] >> (i % 64)) & 1);
            let p = self.probs[i];
            if !self.condition.evaluate(bit, p) {
                continue;
            }
            if ac.encode(bit, p, &mut writer).is_err() {
                self.processed_bits = i + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

/// An entropy-coded history whose bits are arranged in MSB-first byte order.
///
/// `ACHistory` visits the history one bit at a time, starting with the most
/// recently received bit. That makes the bits within every byte appear in
/// reverse order. This variant starts with the partial (most recent) byte,
/// but visits the bits in that byte in their original order. It then visits
/// each preceding byte, also MSB first.
#[derive(Clone)]
pub struct ACHistoryMSB<M: Model> {
    pos: usize,
    bits: u64,
    probs: RotatingBuffer<u16, 64>,
    processed_bits: usize,
    model: M,
}

impl<M: Model> ACHistoryMSB<M> {
    pub fn new(model: M) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            model,
        }
    }
}

/// Return the index in the rotating history for the `ordinal`th bit in the
/// MSB-first view of the history.
///
/// `partial_bits` is the number of bits in the current partial byte and is in
/// the range `0..=7`. A value of zero means that the most recent byte is
/// complete. The rotating buffer is indexed from the newest bit: index zero
/// is the last bit received. The current partial byte and every preceding
/// byte are therefore read in descending buffer-index order.
#[inline]
fn history_index_msb(ordinal: usize, partial_bits: usize) -> usize {
    if ordinal < partial_bits {
        return partial_bits - ordinal - 1;
    }
    let previous = ordinal - partial_bits;
    let byte = previous / 8;
    let bit = previous % 8;
    partial_bits + byte * 8 + 7 - bit
}

impl<M: Model> History for ACHistoryMSB<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
        self.processed_bits = 0;
    }

    fn hash(&mut self, max_bits: u8) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter::new(max_bits);
        let partial_bits = self.pos % 8;

        for ordinal in 0..self.pos.min(64) {
            let index = history_index_msb(ordinal, partial_bits);
            // With an unaligned history, the next complete MSB-first byte
            // can begin before the 64-bit raw window. Do not read beyond the
            // rotating buffer just to fill the output to 64 bits.
            if index >= 64 {
                break;
            }
            let bit = u8!((self.bits >> index) & 1);
            let p = self.probs[index];
            if ac.encode(bit, p, &mut writer).is_err() {
                self.processed_bits = ordinal + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

/// An entropy-coded history that uses LSB-first order for the current
/// partial byte and MSB-first order for all preceding complete bytes.
///
/// The byte boundary determines where the ordering changes: bits in the
/// current incomplete byte are read from newest to oldest, while complete
/// bytes before it are read MSB-first.
#[derive(Clone)]
pub struct ACHistoryByteBoundary<M: Model> {
    pos: usize,
    bits: u64,
    probs: RotatingBuffer<u16, 64>,
    processed_bits: usize,
    model: M,
}

impl<M: Model> ACHistoryByteBoundary<M> {
    pub fn new(model: M) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            model,
        }
    }
}

/// Return the index in the rotating history for the `ordinal`th bit in the
/// LSB-partial/MSB-previous-bytes view of the history.
#[inline]
fn history_index_lsb_msb(ordinal: usize, partial_bits: usize) -> usize {
    if ordinal < partial_bits {
        return ordinal;
    }
    let previous = ordinal - partial_bits;
    let byte = previous / 8;
    let bit = previous % 8;
    partial_bits + byte * 8 + 7 - bit
}

impl<M: Model> History for ACHistoryByteBoundary<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
        self.processed_bits = 0;
    }

    fn hash(&mut self, max_bits: u8) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter::new(max_bits);
        let partial_bits = self.pos % 8;

        for ordinal in 0..self.pos.min(64) {
            let index = history_index_lsb_msb(ordinal, partial_bits);
            // With an unaligned history, the next complete MSB-first byte
            // can begin before the 64-bit raw window.
            if index >= 64 {
                break;
            }
            let bit = u8!((self.bits >> index) & 1);
            let p = self.probs[index];
            if ac.encode(bit, p, &mut writer).is_err() {
                self.processed_bits = ordinal + 1;
                break;
            }
        }

        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
}

impl EntropyWriter {
    fn new(max_bits: u8) -> Self {
        Self { state: 0, max_bits, rev_bits: 0, idx: 0 }
    }
}

impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        debug_assert!(self.idx <= self.max_bits);
        use std::io::{Error, ErrorKind};
        let bit = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            if self.idx == self.max_bits {
                return Err(Error::from(ErrorKind::Other));
            }

            self.state = (self.state >> 1) | (u32::from(bit) << 31);
            self.idx += 1;
            Ok(())
        };

        write_bit_raw(bit)?;
        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            write_bit_raw(bit ^ 1)?;
        }

        Ok(())
    }

    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn flush(&mut self, _padding: u32) -> std::io::Result<()> {
        self.write_bit(1)?;
        debug_assert!(self.rev_bits == 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_index_msb_partial() {
        // One bit in the current byte, followed by two complete preceding
        // bytes: current, then 8..1, then 16..9.
        let indices = (0..17)
            .map(|i| super::history_index_msb(i, 1))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![0, 8, 7, 6, 5, 4, 3, 2, 1, 16, 15, 14, 13, 12, 11, 10, 9]
        );

        // Two bits in the current byte: current bits are 1,0, followed by
        // the preceding byte 9..2.
        let indices = (0..10)
            .map(|i| super::history_index_msb(i, 2))
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![1, 0, 9, 8, 7, 6, 5, 4, 3, 2]);
    }

    #[test]
    fn history_index_msb_aligned() {
        let indices = (0..17)
            .map(|i| super::history_index_msb(i, 0))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23]
        );
    }

    #[test]
    fn history_index_lsb_msb_partial() {
        let indices = (0..17)
            .map(|i| super::history_index_lsb_msb(i, 1))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![0, 8, 7, 6, 5, 4, 3, 2, 1, 16, 15, 14, 13, 12, 11, 10, 9]
        );

        let indices = (0..10)
            .map(|i| super::history_index_lsb_msb(i, 2))
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![0, 1, 9, 8, 7, 6, 5, 4, 3, 2]);
    }

    #[test]
    fn history_index_lsb_msb_aligned() {
        let indices = (0..17)
            .map(|i| super::history_index_lsb_msb(i, 0))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23]
        );
    }

    #[test]
    fn history_skip_conditions_evaluate_observed_bit_cost() {
        let probability = super::HistorySkipCondition::Probability { lower: 0.4, upper: 0.6 };
        assert!(probability.evaluate(1, 16_384));
        assert!(!probability.evaluate(1, 32_768));
        assert!(!probability.evaluate(1, 49_152));
        assert!(probability.evaluate(0, 49_152));
        assert!(!probability.evaluate(0, 32_768));
        assert!(!probability.evaluate(0, 16_384));

        let entropy = super::HistorySkipCondition::Entropy { lower: 1.0, upper: f64::INFINITY };
        assert!(entropy.evaluate(1, 16_384));
        assert!(!entropy.evaluate(1, 49_152));

        let exclusive = super::HistorySkipCondition::EntropyExclusive { lower: 0.5, upper: 1.0 };
        assert!(exclusive.evaluate(1, 49_152));
        assert!(!exclusive.evaluate(1, 32_768));
        assert!(exclusive.evaluate(1, 16_384));
    }
}
