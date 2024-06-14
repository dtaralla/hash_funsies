use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher, RandomState};

use ahash::AHasher;
use criterion::{black_box, Criterion, criterion_group, criterion_main};

use hash_funsies::VoxelChunkIndex;

/// Just doesn't do any hashing. Uses the number itself as hashed value.
#[derive(Default)]
struct IdentityHasher(u64);
impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const U128SIZE: usize = std::mem::size_of::<u128>();
        const U64SIZE: usize = std::mem::size_of::<u64>();
        const U32SIZE: usize = std::mem::size_of::<u32>();
        const U16SIZE: usize = std::mem::size_of::<u16>();
        const U8SIZE: usize = std::mem::size_of::<u8>();

        self.0 += match bytes.len() {
            U64SIZE => u64::from_be_bytes(bytes.try_into().unwrap()),
            U32SIZE => u32::from_be_bytes(bytes.try_into().unwrap()) as u64,
            U16SIZE => u16::from_be_bytes(bytes.try_into().unwrap()) as u64,
            U8SIZE => u8::from_be_bytes(bytes.try_into().unwrap()) as u64,

            U128SIZE => {
                let mut sum = 0u64;
                for chunk in bytes.chunks(U64SIZE) {
                    sum += u64::from_be_bytes(chunk.try_into().unwrap());
                }
                sum
            }

            _ => {
                let mut sum = 0u64;
                let mut it = bytes.chunks_exact(U64SIZE);
                for chunk in it.by_ref() {
                    sum += u64::from_be_bytes(chunk.try_into().unwrap());
                }

                {
                    let bytes = it.remainder();
                    let mut result = [0u8; 8];
                    for i in ((U64SIZE - bytes.len())..(U64SIZE - 1)).rev() {
                        result[i] = bytes[i];
                    }
                    sum += u64::from_be_bytes(result);
                }
                sum
            }
        };
    }

    fn write_u8(&mut self, i: u8) {
        self.0 += i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.0 += i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.0 += i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.0 += i;
    }
}

/// Fibonacci hashing - see https://probablydance.com/2018/06/16/fibonacci-hashing-the-optimization-that-the-world-forgot-or-a-better-alternative-to-integer-modulo/
pub struct FibHasher<const N: u8> {
    hash: u64,
}
impl<const N: u8> FibHasher<N> {
    const SHIFT: u8 = 64 - N;
}
impl<const N: u8> Default for FibHasher<N> {
    fn default() -> Self {
        assert!(Self::SHIFT < 64);
        FibHasher { hash: 0 }
    }
}
impl<const N: u8> Hasher for FibHasher<N> {
    fn finish(&self) -> u64 {
        self.hash.wrapping_mul(11400714819323198485) >> Self::SHIFT
    }

    fn write(&mut self, bytes: &[u8]) {
        const U128SIZE: usize = std::mem::size_of::<u128>();
        const U64SIZE: usize = std::mem::size_of::<u64>();
        const U32SIZE: usize = std::mem::size_of::<u32>();
        const U16SIZE: usize = std::mem::size_of::<u16>();
        const U8SIZE: usize = std::mem::size_of::<u8>();

        self.hash += match bytes.len() {
            U64SIZE => u64::from_be_bytes(bytes.try_into().unwrap()),
            U32SIZE => u32::from_be_bytes(bytes.try_into().unwrap()) as u64,
            U16SIZE => u16::from_be_bytes(bytes.try_into().unwrap()) as u64,
            U8SIZE => u8::from_be_bytes(bytes.try_into().unwrap()) as u64,

            U128SIZE => {
                let mut sum = 0u64;
                for chunk in bytes.chunks(U64SIZE) {
                    sum += u64::from_be_bytes(chunk.try_into().unwrap());
                }
                sum
            }

            _ => {
                let mut sum = 0u64;
                let mut it = bytes.chunks_exact(U64SIZE);
                for chunk in it.by_ref() {
                    sum += u64::from_be_bytes(chunk.try_into().unwrap());
                }

                {
                    let bytes = it.remainder();
                    let mut result = [0u8; 8];
                    for i in ((U64SIZE - bytes.len())..(U64SIZE - 1)).rev() {
                        result[i] = bytes[i];
                    }
                    sum += u64::from_be_bytes(result);
                }
                sum
            }
        };
    }

    fn write_u8(&mut self, i: u8) {
        self.hash += i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.hash += i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.hash += i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.hash += i;
    }
}

type CrcHasherBuilder = core::hash::BuildHasherDefault<crc32fast::Hasher>;
type AHashBuilder = core::hash::BuildHasherDefault<AHasher>;
type IdentityHasherBuilder = core::hash::BuildHasherDefault<IdentityHasher>;

// For the Fibonacci Hasher, we need at least size 20 because we're going to hash 200 * 200 * 20
// = 800_000 values, and it would be nice to not have a mapped space full of collisions.
// 2^20 = 1_048_576, which should be a large enough space to hold the amount of keys.
const FIB_SIZE: u8 = 20;
type FibHasherBuilder = core::hash::BuildHasherDefault<FibHasher<FIB_SIZE>>;

pub fn inserts<T: BuildHasher>(
    coords: &Vec<VoxelChunkIndex>,
    bh: T,
) -> HashMap<VoxelChunkIndex, u32, T> {
    let mut hmap = HashMap::with_hasher(bh);
    hmap.reserve(coords.len());

    for &c in coords {
        hmap.insert(c, 0);
    }

    hmap
}

pub fn reads<T: BuildHasher>(coords: &Vec<VoxelChunkIndex>, hm: &HashMap<VoxelChunkIndex, u32, T>) {
    for c in coords {
        black_box(hm.get(c));
    }
}

pub fn hashes<T: BuildHasher>(coords: &[VoxelChunkIndex], build_hasher: T) {
    for c in coords {
        black_box(build_hasher.hash_one(c));
    }
}

const XY_LOW: i32 = -100;
const XY_UP: i32 = 100;
const Z_LOW: i32 = -10;
const Z_UP: i32 = 10;
const NUM_ELEMS: usize = ((XY_UP - XY_LOW) * (XY_UP - XY_LOW) * (Z_UP - Z_LOW)) as usize;

pub fn gen_coords() -> Vec<VoxelChunkIndex> {
    let mut coords = Vec::<VoxelChunkIndex>::with_capacity(NUM_ELEMS);
    for x in XY_LOW..XY_UP {
        for y in XY_LOW..XY_UP {
            for z in Z_LOW..Z_UP {
                coords.push(VoxelChunkIndex::from_coords(x, y, z));
            }
        }
    }

    coords
}

pub fn bench_inserts(c: &mut Criterion) {
    let coords = gen_coords();

    let mut group = c.benchmark_group("Inserts");
    group.sample_size(300);

    group.bench_function("Vanilla", |b| {
        b.iter(|| inserts(&coords, black_box(RandomState::new())))
    });
    group.bench_function("Crc", |b| {
        b.iter(|| inserts(&coords, black_box(CrcHasherBuilder::default())))
    });
    group.bench_function("Fib", |b| {
        b.iter(|| inserts(&coords, black_box(FibHasherBuilder::default())))
    });
    group.bench_function("AHash", |b| {
        b.iter(|| inserts(&coords, black_box(AHashBuilder::default())))
    });
    group.bench_function("Id", |b| {
        b.iter(|| inserts(&coords, black_box(IdentityHasherBuilder::default())))
    });

    group.finish();
}

pub fn bench_reads(c: &mut Criterion) {
    let coords = gen_coords();
    let hm1 = inserts(&coords, RandomState::new());
    let hm2 = inserts(&coords, CrcHasherBuilder::default());
    let hm3 = inserts(&coords, FibHasherBuilder::default());
    let hm4 = inserts(&coords, AHashBuilder::default());
    let hm5 = inserts(&coords, IdentityHasherBuilder::default());

    let mut group = c.benchmark_group("Reads");
    group.sample_size(300);

    group.bench_function("Vanilla", |b| b.iter(|| reads(&coords, black_box(&hm1))));
    group.bench_function("Crc", |b| b.iter(|| reads(&coords, black_box(&hm2))));
    group.bench_function("Fib", |b| b.iter(|| reads(&coords, black_box(&hm3))));
    group.bench_function("AHash", |b| b.iter(|| reads(&coords, black_box(&hm4))));
    group.bench_function("Id", |b| b.iter(|| reads(&coords, black_box(&hm5))));

    group.finish();
}

pub fn bench_hashes(c: &mut Criterion) {
    let coords = gen_coords();

    let mut group = c.benchmark_group("Hashes");
    group.sample_size(300);

    group.bench_function("Vanilla", |b| {
        b.iter(|| hashes(&coords, black_box(RandomState::new())))
    });
    group.bench_function("Crc", |b| {
        b.iter(|| hashes(&coords, black_box(CrcHasherBuilder::default())))
    });
    group.bench_function("Fib", |b| {
        b.iter(|| hashes(&coords, black_box(FibHasherBuilder::default())))
    });
    group.bench_function("AHash", |b| {
        b.iter(|| hashes(&coords, black_box(AHashBuilder::default())))
    });
    group.bench_function("Id", |b| {
        b.iter(|| hashes(&coords, black_box(IdentityHasherBuilder::default())))
    });

    group.finish();
}

criterion_group!(benches, bench_hashes, bench_inserts, bench_reads);
criterion_main!(benches);
