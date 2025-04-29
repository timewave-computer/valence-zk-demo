use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    //signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeaconHeaderSummary {
    //root: String,
    //canonical: bool,
    pub header: SignedBeaconBlockHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String, // Hex-encoded 32 bytes (0x-prefixed)
    pub state_root: String,  // Hex-encoded 32 bytes (0x-prefixed)
    pub body_root: String,   // Hex-encoded 32 bytes (0x-prefixed)
}