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
    ordering: HistoryBitOrder,
    model: M,
}

/// Controls the order in which an [`ACHistory`] visits its raw history.
#[derive(Clone, Copy, Debug)]
pub enum HistoryBitOrder {
    /// Visit every bit newest-first.
    LSB,
    /// Visit the current partial byte MSB-first, then older bytes MSB-first.
    MSB,
    /// Visit the current partial byte newest-first, then older bytes MSB-first.
    Boundary,
}

impl<M: Model> ACHistory<M> {
    pub fn new(model: M, ordering: HistoryBitOrder) -> Self {
        const HALF: u16 = 1 << 15;
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(HALF),
            processed_bits: 0,
            ordering,
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
        let partial_bits = self.pos % 8;
        for ordinal in 0..self.pos.min(64) {
            let index = self.ordering.history_index(ordinal, partial_bits);
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

impl HistoryBitOrder {
    #[inline]
    fn history_index(self, ordinal: usize, partial_bits: usize) -> usize {
        match self {
            Self::LSB => ordinal,
            Self::MSB => {
                if ordinal < partial_bits {
                    return partial_bits - ordinal - 1;
                }
                let previous = ordinal - partial_bits;
                let byte = previous / 8;
                let bit = previous % 8;
                partial_bits + byte * 8 + 7 - bit
            }
            Self::Boundary => {
                if ordinal < partial_bits {
                    return ordinal;
                }
                let previous = ordinal - partial_bits;
                let byte = previous / 8;
                let bit = previous % 8;
                partial_bits + byte * 8 + 7 - bit
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntropyWriter {
    pub state: u32,
    pub max_bits: u8,
    pub rev_bits: u16,
    pub idx: u8,
}

impl EntropyWriter {
    pub fn new(max_bits: u8) -> Self {
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
            .map(|i| super::HistoryBitOrder::MSB.history_index(i, 1))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![0, 8, 7, 6, 5, 4, 3, 2, 1, 16, 15, 14, 13, 12, 11, 10, 9]
        );

        // Two bits in the current byte: current bits are 1,0, followed by
        // the preceding byte 9..2.
        let indices = (0..10)
            .map(|i| super::HistoryBitOrder::MSB.history_index(i, 2))
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![1, 0, 9, 8, 7, 6, 5, 4, 3, 2]);
    }

    #[test]
    fn history_index_msb_aligned() {
        let indices = (0..17)
            .map(|i| super::HistoryBitOrder::MSB.history_index(i, 0))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23]
        );
    }

    #[test]
    fn history_index_lsb_msb_partial() {
        let indices = (0..17)
            .map(|i| super::HistoryBitOrder::Boundary.history_index(i, 1))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![0, 8, 7, 6, 5, 4, 3, 2, 1, 16, 15, 14, 13, 12, 11, 10, 9]
        );

        let indices = (0..10)
            .map(|i| super::HistoryBitOrder::Boundary.history_index(i, 2))
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![0, 1, 9, 8, 7, 6, 5, 4, 3, 2]);
    }

    #[test]
    fn history_index_lsb_msb_aligned() {
        let indices = (0..17)
            .map(|i| super::HistoryBitOrder::Boundary.history_index(i, 0))
            .collect::<Vec<_>>();
        assert_eq!(
            indices,
            vec![7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23]
        );
    }
}
