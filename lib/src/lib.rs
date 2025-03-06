use alloy_sol_types::{private::FixedBytes, sol};
use cid::Cid;
use multihash::{Code, MultihashDigest};

// Constants
pub const RAW: u64 = 0x55; // Raw codec

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        bytes32 cid_bytes; // First 32 bytes of the CID
        bytes16 cid_bytes_remainder; // Remaining bytes of the CID if any
        uint8 cid_length; // Actual length of the CID in bytes
    }
}

/// Calculate CID v1 using SHA-256 for a given file content
pub fn calculate_cid(content: &[u8]) -> (Cid, [u8; 48]) {
    // Calculate SHA-256 hash of the content
    let hash = Code::Sha2_256.digest(content);

    // Create CID v1 with the RAW codec
    let cid = Cid::new_v1(RAW, hash);

    // Convert CID to bytes for storing in PublicValuesStruct
    let mut cid_bytes = [0u8; 48]; // Allocate enough space for the CID

    // Convert the CID to bytes
    let cid_raw_bytes = cid.to_bytes();
    for (i, b) in cid_raw_bytes.iter().enumerate() {
        if i < cid_bytes.len() {
            cid_bytes[i] = *b;
        }
    }

    (cid, cid_bytes)
}

/// Extract structured public values from CID bytes
pub fn extract_public_values(cid_bytes: [u8; 48]) -> PublicValuesStruct {
    let mut first_32 = [0u8; 32];
    let mut remainder = [0u8; 16];

    // Copy first 32 bytes
    for i in 0..32 {
        first_32[i] = cid_bytes[i];
    }

    // Copy remainder (up to 16 more bytes)
    for i in 0..16 {
        if i + 32 < cid_bytes.len() {
            remainder[i] = cid_bytes[i + 32];
        }
    }

    // Find actual length by counting non-zero bytes
    let mut length = 0u8;
    for i in 0..cid_bytes.len() {
        if cid_bytes[i] != 0 {
            length = (i + 1) as u8;
        }
    }

    // Convert the arrays to FixedBytes types
    let cid_bytes_fixed: FixedBytes<32> = first_32.into();
    let cid_bytes_remainder_fixed: FixedBytes<16> = remainder.into();

    PublicValuesStruct {
        cid_bytes: cid_bytes_fixed,
        cid_bytes_remainder: cid_bytes_remainder_fixed,
        cid_length: length,
    }
}
