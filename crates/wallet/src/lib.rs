pub mod bitcoin;
pub mod ethereum;

pub use bitcoin::{BitcoinAddressType, BitcoinVanityGenerator};
pub use cosmos::CosmosVanityGenerator;
pub use ethereum::EthereumVanityGenerator;
pub use solana::SolanaVanityGenerator;
pub use ton::TonVanityGenerator;

pub mod cosmos;
pub mod solana;
pub mod ton;
