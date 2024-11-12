use std::fs::File;

use clap::{arg, Parser};
use futures_util::future::join_all;
use hydra_control_plane_rpc::{
    model::{
        cluster::KeyEnvelope,
        hydra::tx::{
            commit::CommitTx, head_parameters::HeadParameters, init::InitTx, input::InputWrapper,
            output::OutputWrapper, script_registry::NetworkScriptRegistry,
        },
    },
    providers::blockfrost::Blockfrost,
};
use pallas::{
    crypto::{hash::Hash, key::ed25519::SecretKey},
    ledger::addresses::Address,
    txbuilder::{Input, Output},
};
use tracing::debug;

// CLI to open a Hydra Head
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Network ID
    #[arg(short, long, default_value_t = 0)]
    network_id: u8,

    // Seed Input {tx_hash}#{index}
    #[arg(short, long)]
    seed_input: String,

    // Participant address
    #[arg(short, long)]
    participant: String,

    // Contestation period in seconds
    #[arg(short, long, default_value_t = 60000)]
    contestation_period: u64,

    // Party, hydra verification key hash
    #[arg(short = 'u', long)]
    party_verification_file: String,

    // Commit Inputs
    #[arg(short='i', long, num_args=1..)]
    commit_inputs: Vec<String>,

    // Cardano Signing Key file
    #[arg(short = 'k', long)]
    cardano_key_file: String,

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
    let blockfrost = Blockfrost::new(args.blockfrost_key.as_str());
    let admin_key_envelope: KeyEnvelope = serde_json::from_reader(
        File::open(args.cardano_key_file).expect("unable to open key file"),
    )
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

    let party_key_envelope: KeyEnvelope = serde_json::from_reader(
        File::open(args.party_verification_file).expect("unable to open party key file"),
    )
    .expect("unable to parse party key file");
    let party: Vec<u8> = party_key_envelope
        .try_into()
        .expect("Failed to get party verification key from file");

    println!("Building init transaction...");

    let participant_hash = match Address::from_bech32(args.participant.as_str())
        .expect("Failed to parse bech32 participant address")
    {
        Address::Shelley(address) => address.payment().as_hash().as_ref().to_vec(),
        Address::Byron(_) => panic!("Byron addresses are not supported"),
        Address::Stake(_) => panic!("Stake addresses are not supported"),
    };

    let init_tx = InitTx {
        network_id: args.network_id,
        seed_input,
        participants: vec![participant_hash.clone()],
        parameters: HeadParameters {
            contestation_period: args.contestation_period as i64,
            parties: vec![party.clone()],
        },
    };

    let head_id = init_tx.get_head_id().expect("Failed to get head ID");

    let built_init_tx = init_tx
        .to_tx(Output::new(
            Address::from_bech32(
                seed_input_output
                    .address
                    .to_bech32()
                    .expect("failed to parse address")
                    .as_str(),
            )
            .expect("failed to parse address"),
            seed_input_output.lovelace - 9000000,
        ))
        .expect("Failed to build tx");

    println!("Tx bytes: {}", hex::encode(built_init_tx.tx_bytes.clone()));

    let built_init_tx = built_init_tx
        .sign(admin_key.clone().into())
        .expect("Failed to sign tx");

    debug!(
        "Submitting init tx: {}",
        hex::encode(built_init_tx.tx_bytes.clone().0)
    );

    let init_tx_id = blockfrost
        .submit_transaction(built_init_tx)
        .await
        .expect("Failed to submit init tx");

    println!("Submitted init tx: {}", init_tx_id);
    println!("Committing funds...");

    let commit_inputs: Vec<(InputWrapper, OutputWrapper)> = join_all(args.commit_inputs.into_iter().map(|input| async {
        let input: InputWrapper = input.try_into().expect("Failed to parse commit input. Please make sure it uses the following format: {tx_hash}#{index}");
        let input_tx_hash = hex::encode(input.tx_hash.0);
        let output = blockfrost.get_utxo(input_tx_hash.as_str(), input.txo_index as i32).await.expect("Failed to fetch commit input output");
        (input, output.into())
    })).await;

    let commit_tx = CommitTx {
        network_id: args.network_id,
        script_registry: NetworkScriptRegistry::Preprod.into(),
        head_id: head_id.clone(),
        party,
        initial_input: (
            Input::new(
                Hash::from(
                    hex::decode(init_tx_id.clone())
                        .expect("failed to decode init tx id")
                        .as_slice(),
                ),
                1,
            )
            .into(),
            init_tx.make_initial_output(Hash::from(head_id.as_slice()), participant_hash.clone()),
            Hash::from(participant_hash.as_slice()),
        ),
        blueprint_tx: vec![(
            Input::new(
                Hash::from(
                    hex::decode(init_tx_id)
                        .expect("failed to decode init tx id")
                        .as_slice(),
                ),
                2,
            )
            .into(),
            Output::new(
                Address::from_bech32(args.participant.as_str())
                    .expect("failed to parse participant address"),
                seed_input_output.lovelace - 9000000 - 1875229,
            )
            .into(),
        )],
        fee: 1875229,
        commit_inputs,
    };

    let built_commit_tx = commit_tx
        .build_tx()
        .expect("Failed to build commit transaction")
        .sign(admin_key.into())
        .expect("Failed to sign commit tx");

    println!(
        "Signed commit tx: {}",
        hex::encode(built_commit_tx.tx_bytes.clone())
    );

    let commit_tx_id = blockfrost
        .submit_transaction(built_commit_tx)
        .await
        .expect("Failed to submit commit tx");
    println!("Submitted commit tx: {}", commit_tx_id);
}
