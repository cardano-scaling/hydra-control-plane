use std::fs::File;

use clap::{arg, command, Parser};
use hydra_control_plane_rpc::{
    model::{
        cluster::KeyEnvelope,
        hydra::{contract::hydra_validator::HydraValidator, tx::input::InputWrapper},
    },
    providers::blockfrost::Blockfrost,
};
use pallas::{
    crypto::key::ed25519::SecretKey,
    ledger::addresses::Address,
    txbuilder::{BuildConway, Output, ScriptKind, StagingTransaction},
};
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    network_id: u8,
    // Seed Input {tx_hash}#{index}
    #[arg(short, long)]
    seed_input: String,

    // Cardano Signing Key file
    #[arg(short = 'k', long)]
    key_file: String,

    // Blockfrost Project ID
    #[arg(short, long)]
    blockfrost_key: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    let mut destination_bytes =
        hex::decode("6a09cb22defaf4a96a6be1ef6c07467ac9923d1750a79214a06c503a")
            .expect("failed to decode address bytes");
    destination_bytes.insert(0, 0b1110000 | args.network_id);
    let destination: Address = Address::from_bytes(destination_bytes.as_slice())
        .expect("Failed to construct destination address");

    let blockfrost = Blockfrost::new(args.blockfrost_key.as_str());
    let admin_key_envelope: KeyEnvelope =
        serde_json::from_reader(File::open(args.key_file).expect("unable to open key file"))
            .expect("unable to parse key file");

    let admin_key: SecretKey = admin_key_envelope
        .try_into()
        .expect("Failed to get secret key from file");

    let seed_input: InputWrapper = args.seed_input.try_into().expect("Failed to parse seed input. Please make sure it uses the following format: {tx_hash}#{index}");

    let seed_input_output = blockfrost
        .get_utxo(
            hex::encode(seed_input.tx_hash.0).as_str(),
            seed_input.txo_index as i32,
        )
        .await
        .map_err(|e| tracing::error!(err = e.to_string(), "Failed to fetch seed input"))
        .unwrap();

    let initial_output = Output::new(destination.clone(), 12386940)
        .set_inline_script(ScriptKind::PlutusV2, HydraValidator::VInitial.into());
    let commit_output = Output::new(destination.clone(), 3866070)
        .set_inline_script(ScriptKind::PlutusV2, HydraValidator::VCommit.into());
    let head_output = Output::new(destination, 55292990)
        .set_inline_script(ScriptKind::PlutusV2, HydraValidator::VHead.into());

    let transaction = StagingTransaction::new()
        .fee(873549)
        .input(seed_input.into())
        .output(initial_output)
        .output(commit_output)
        .output(head_output)
        .output(Output::new(
            seed_input_output.address.0,
            seed_input_output.lovelace - (12386940 + 3866070 + 55292990) - 873549,
        ))
        .build_conway_raw()
        .inspect_err(|e| println!("Transaction build failed: {}", e))
        .expect("Failed to build transaction");

    let signed_transaction = transaction
        .sign(admin_key.into())
        .inspect_err(|e| println!("Failed to sign transaction: {}", e))
        .expect("Failed to sign transaction");

    debug!(
        "Signed transaction: {}",
        hex::encode(signed_transaction.tx_bytes.clone())
    );

    let commit_tx_id = blockfrost
        .submit_transaction(signed_transaction)
        .await
        .expect("Failed to submit commit tx");
    info!("{}", commit_tx_id);
}
