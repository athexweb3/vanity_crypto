pub mod bitcoin;
pub mod ethereum;

pub use bitcoin::{BitcoinAddressType, BitcoinVanityGenerator};
pub use ethereum::EthereumVanityGenerator;
