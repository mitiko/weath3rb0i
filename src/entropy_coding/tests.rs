use std::{fs::File, io::BufWriter};

use super::{
    ac_io::{ACReader, ACWriter},
    ACRead, ACWrite, ArithmeticCoder,
};

fn compress(filename: &str, input: &[u8], probabilities: &[u16]) -> Vec<u8> {
    let file = File::create(filename).unwrap();
    let writer = ACWriter::new(BufWriter::new(file));
    let mut ac = ArithmeticCoder::new_coder(writer);

    let mut reader = ACReader::new(input);
    for &prob in probabilities {
        let bit = reader.read_bit().unwrap();
        ac.encode(bit, prob).unwrap();
    }

    ac.flush().unwrap();
    let compressed = std::fs::read(filename).unwrap();
    std::fs::remove_file(filename).unwrap();
    compressed
}

fn decompress(filename: &str, input: &[u8], probabilities: &[u16]) -> Vec<u8> {
    let reader = ACReader::new(input);
    let mut ac = ArithmeticCoder::new_decoder(reader).unwrap();

    let file = File::create(filename).unwrap();
    let mut writer = ACWriter::new(BufWriter::new(file));
    for &prob in probabilities {
        let bit = ac.decode(prob).unwrap();
        writer.write_bit(bit).unwrap();
    }

    // flush always appends \x00 to end, because it's aligned
    writer.flush(0).unwrap();
    let mut decompressed = std::fs::read(filename).unwrap();
    assert_eq!(decompressed.pop(), Some(0x00));
    std::fs::remove_file(filename).unwrap();
    decompressed
}

#[test]
fn best_model_zeroes() {
    let block_size = 1 << 15;
    let input = [0x00].repeat(block_size);
    let probabilities = [0].repeat(block_size * 8);
    let compressed = compress("best_model_zeroes.bin", &input, &probabilities);
    let decompressed = decompress("best_model_zeroes.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // only 0s will always compress to 0 bytes (and flush will add 1)
    assert_eq!(compressed.len(), 1);
}

#[test]
fn best_model_ones() {
    let block_size = 1 << 15;
    let input = [0xff].repeat(block_size);
    let probabilities = [u16::MAX].repeat(block_size * 8);
    let compressed = compress("best_model_ones.bin", &input, &probabilities);
    let decompressed = decompress("best_model_ones.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    assert_eq!(compressed.len(), 1);

    let block_size = 1 << 16;
    let input = [0xff].repeat(block_size);
    let probabilities = [u16::MAX].repeat(block_size * 8);
    let compressed = compress("best_model_ones.bin", &input, &probabilities);
    let decompressed = decompress("best_model_ones.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // loss is 2^-16 bits/bit, so at 2^16 bytes, we get an extra byte
    assert_eq!(compressed.len(), 2);
}

#[test]
fn best_model_alternating() {
    let block_size = 1024;
    let input = [0x55].repeat(block_size);
    let probabilities = [0, u16::MAX].repeat(block_size * 8 / 2);
    let compressed = compress("best_model_alternating.bin", &input, &probabilities);
    let decompressed = decompress("best_model_alternating.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    assert_eq!(compressed.len(), 1);
}

#[test]
fn worst_model_zeroes() {
    let block_size = 16;
    let input = [0x00].repeat(block_size);
    let probabilities = [u16::MAX].repeat(block_size * 8);
    let compressed = compress("worst_model_zeroes.bin", &input, &probabilities);
    let decompressed = decompress("worst_model_zeroes.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // loss is (1 - 2^-16) = (2^16 - 1)/(2^16) bits/bit
    assert_eq!(compressed.len(), 16 * block_size + 1);
}

#[test]
fn worst_model_ones() {
    let block_size = 16;
    let input = [0xff].repeat(block_size);
    let probabilities = [0].repeat(block_size * 8);
    let compressed = compress("worst_model_ones.bin", &input, &probabilities);
    let decompressed = decompress("worst_model_ones.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // TODO: Optimize zeroes bias
    // practical loss is a little more than (1 - 2^-16) bits/bit
    assert_eq!(compressed.len(), 32 * block_size + 1);
}

#[test]
fn worst_model_alternating() {
    let block_size = 16;
    let input = [0x55].repeat(block_size);
    let probabilities = [u16::MAX, 0].repeat(block_size * 8 / 2);
    let compressed = compress("worst_model_alternating.bin", &input, &probabilities);
    let decompressed = decompress("worst_model_alternating.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // TODO: Optimize zeroes bias
    // half the time it gets the bias, half the time it does not
    // 24 = (32 + 16) / 2
    assert_eq!(compressed.len(), 24 * block_size + 1);
}

#[test]
fn no_model() {
    let block_size = 128;
    let input = [0xaa, 0x55].repeat(block_size / 2);
    let probabilities = [1 << 15].repeat(block_size * 8);
    let compressed = compress("no_model.bin", &input, &probabilities);
    let decompressed = decompress("no_model.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // flush always adds one byte to aligned sequences
    assert!(compressed.len() == block_size + 1);
}

#[test]
fn half_good_model() {
    let block_size = 128;
    let input = [0x55].repeat(block_size);
    let probabilities = [1 << 15, u16::MAX].repeat(block_size * 8 / 2);
    let compressed = compress("half_good_model.bin", &input, &probabilities);
    let decompressed = decompress("half_good_model.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // cross entropy is 1/2 * 1 + 1/2 * 2^-16 ~ 1/2
    assert_eq!(compressed.len(), block_size / 2);
}

#[test]
fn half_bad_model() {
    let block_size = 128;
    let input = [0x55].repeat(block_size);
    let probabilities = [1 << 15, 0].repeat(block_size * 8 / 2);
    let compressed = compress("half_bad_model.bin", &input, &probabilities);
    let decompressed = decompress("half_bad_model.tmp", &compressed, &probabilities);
    assert_eq!(input, decompressed);
    // half the time it gets a 16x penalty, half the time it writes the bit
    // flush always adds one byte to aligned sequences
    assert_eq!(compressed.len(), 16 * block_size + block_size / 2 + 1);
}

// TODO: Fuzzing tests
