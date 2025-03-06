//! An end-to-end example of using the SP1 SDK to generate a proof of CID calculation
//! that can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --file <PATH_TO_FILE> --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --file <PATH_TO_FILE> --system plonk
//! ```

use alloy_sol_types::SolType;
use cid_proof_lib::PublicValuesStruct;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::fs;
use std::path::PathBuf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CID_PROOF_ELF: &[u8] = include_elf!("cid-proof-program");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,

    #[clap(long, required = true)]
    file: PathBuf,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CIDProofFixture {
    cid_bytes: String,
    cid_bytes_remainder: String,
    cid_length: u8,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Check if file exists
    if !args.file.exists() {
        eprintln!("Error: File not found: {}", args.file.display());
        std::process::exit(1);
    }

    // Read the file content
    let file_content = fs::read(&args.file).expect("Failed to read file");
    println!("File size: {} bytes", file_content.len());

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the program.
    let (pk, vk) = client.setup(CID_PROOF_ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&file_content);

    println!("File: {}", args.file.display());
    println!("Proof System: {:?}", args.system);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let public_values = PublicValuesStruct::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = CIDProofFixture {
        cid_bytes: format!("0x{}", hex::encode(public_values.cid_bytes.0)),
        cid_bytes_remainder: format!("0x{}", hex::encode(public_values.cid_bytes_remainder.0)),
        cid_length: public_values.cid_length,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Display the CID information
    println!("CID bytes: {}", fixture.cid_bytes);
    println!("CID bytes remainder: {}", fixture.cid_bytes_remainder);
    println!("CID length: {}", fixture.cid_length);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
