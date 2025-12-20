use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vanity_core::VanityGenerator;
use vanity_wallet::EthereumVanityGenerator;

fn benchmark_search(c: &mut Criterion) {
    // Benchmark searching for a very simple prefix (1 char)
    // This tests the raw throughput of key generation + hashing + matching
    let generator = EthereumVanityGenerator::new("a", "", false);

    c.bench_function("generate_and_match_1_char", |b| {
        b.iter(|| {
            // We search for just 1 successful match per iteration
            // This includes RNG + fake work
            // Ideally we'd benchmark just the `generate` step but our API bundles them.
            // For a benchmark, we need deterministic or statisical significance.
            // Since `search` is random, benchmarking it directly is noisy.
            // Instead, let's benchmark the internal `generate` flow if possible,
            // or just benchmark a very easy search.
            let _ = generator.generate();
        })
    });
}

fn benchmark_key_generation(c: &mut Criterion) {
    c.bench_function("raw_key_generation", |b| {
        b.iter(|| {
            // Setup similar to the loop body
            use k256::ecdsa::{SigningKey, VerifyingKey};
            use rand::{rngs::OsRng, RngCore};
            use sha3::{Digest, Keccak256};

            let mut rng = OsRng;
            let mut bytes = [0u8; 32];
            rng.fill_bytes(&mut bytes);

            let signing_key = SigningKey::from_bytes(&bytes.into()).unwrap();
            let verifying_key = VerifyingKey::from(&signing_key);
            let encoded_point = verifying_key.to_encoded_point(false);
            let public_key_bytes = &encoded_point.as_bytes()[1..];

            let mut hasher = Keccak256::new();
            hasher.update(public_key_bytes);
            let hash = hasher.finalize();

            black_box(hash);
        })
    });
}

criterion_group!(benches, benchmark_search, benchmark_key_generation);
criterion_main!(benches);
