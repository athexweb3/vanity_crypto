use crate::{Address, PrivateKey};

/// Defines the interface for a vanity address generator.
/// This allows us to swap implementations (CPU vs GPU, Eth vs BTC) easily.
pub trait VanityGenerator {
    /// Generates a keypair that matches the given predicate.
    /// This is a blocking operation intended to be run in a separate thread/task.
    fn generate(&self) -> (PrivateKey, Address);
}
