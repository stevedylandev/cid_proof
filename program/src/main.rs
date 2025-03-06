//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use cid_proof_lib::{calculate_cid, extract_public_values, PublicValuesStruct};

pub fn main() {
    // Read the file content as input
    let content = sp1_zkvm::io::read_vec();

    // Print for debugging (can be removed in production)
    println!("Processing file of size: {} bytes", content.len());

    // Calculate the CID for the given content
    let (cid, cid_bytes) = calculate_cid(&content);

    // Print CID for debugging
    println!("Calculated CID: {}", cid.to_string());

    // Extract public values to be committed
    let public_values = extract_public_values(cid_bytes);

    // Encode the public values using ABI encoding
    let encoded_values = PublicValuesStruct::abi_encode(&public_values);

    // Commit the encoded values to make them public in the proof
    sp1_zkvm::io::commit_slice(&encoded_values);

    println!("CID proof generated successfully!");
}
