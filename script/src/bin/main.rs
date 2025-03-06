//! An end-to-end example of using the SP1 SDK to generate a proof of CID calculation
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute --file <PATH_TO_FILE>
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove --file <PATH_TO_FILE>
//! ```

use alloy_sol_types::SolType;
use cid_proof_lib::PublicValuesStruct;
use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;
use std::path::PathBuf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CID_PROOF_ELF: &[u8] = include_elf!("cid-proof-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, required = true)]
    file: PathBuf,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

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

    // Setup the inputs - pass file content as input
    let mut stdin = SP1Stdin::new();
    stdin.write(&file_content);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(CID_PROOF_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let public_values = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();

        // Display CID information - use .0 to get the inner array
        let cid_bytes_hex = hex::encode(public_values.cid_bytes.0);
        let cid_remainder_hex = hex::encode(public_values.cid_bytes_remainder.0);

        println!("CID (hex): 0x{}{}", cid_bytes_hex, cid_remainder_hex);
        println!("CID length: {} bytes", public_values.cid_length);

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(CID_PROOF_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");

        // Read the public values from the proof
        let public_values =
            PublicValuesStruct::abi_decode(proof.public_values.as_slice(), true).unwrap();

        // Display CID information - use .0 to get the inner array
        let cid_bytes_hex = hex::encode(public_values.cid_bytes.0);
        let cid_remainder_hex = hex::encode(public_values.cid_bytes_remainder.0);

        println!("CID (hex): 0x{}{}", cid_bytes_hex, cid_remainder_hex);
        println!("CID length: {} bytes", public_values.cid_length);
    }
}
