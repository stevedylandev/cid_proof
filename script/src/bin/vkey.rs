use sp1_sdk::{include_elf, HashableKey, Prover, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CID_PROOF_ELF: &[u8] = include_elf!("cid-proof-program");

fn main() {
    let prover = ProverClient::builder().cpu().build();
    let (_, vk) = prover.setup(CID_PROOF_ELF);
    println!("{}", vk.bytes32());
}
