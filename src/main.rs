use anyhow::{Context, Result};
use model::{
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        state::HydraNodesState,
    },
    node::Node,
};
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::global::global;
use routes::head::head;
use routes::heads::heads;
use routes::new_game::new_game;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::{
    spawn,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

#[macro_use]
extern crate rocket;

mod model;
mod routes;

// this is a temporary way to store the script address
pub const SCRIPT_ADDRESS: &str = "addr_test1wr9eag40fhmq3ecvpy2llnldffv2nk2yufa86t33cmrfevqdkh3vm";
pub const SCRIPT_CBOR: &str = "59068c010000332323232323232232232323232323232322322533300f32323232323232325333017301330183754016264a666030602860326ea80044cc00800cdd7180e980d1baa00116301c301d301d30193754028264a666030602260326ea80204c8c8c8c94ccc070c060c074dd500089919299980f180d180f9baa001132533301f3370e900218101baa00113232325333022301e30233754002264a666046646464646464646464646464646464a666064a66606466e21200030333754606e016294454ccc0c8ccc0c8cdd78038012504a229444ccc0c8cdd78048022504a229404c8c8c8c8c8c94ccc0e0c0d0c0e4dd5000899b88375a607a60746ea8004cdc100301b0b19299981c19b8800148000530103d87a800015333038337120029001098101981e1ba80014bd70098101981e1ba8323330010010023370666e000092002480108894ccc0eccdc48010008801099980180180099b833370000266e0c01000520044bd7019b80337006602666e04dd6981e0011bad303c00448010cc04ccdc09bad303c001375a607800690021980999b81375a6078607a0026eb4c0f0c0f400d2004303c001303737546074606e6ea8c0e8c0ecc0dcdd5002181d000981a9baa30383035375460706072606a6ea801cc03006cc0d8c0dc008c0d4004c0d4008c0cc004c0ccc0bcdd500718189819001181800098180011817000981700098149baa024300100122533302a0011480004cdc02400466004004605a0026002002444a66604c66e20005200014800054ccc098c08800452002153233302730233370c00490020999802002180080199b83002480104c004ccc010010c00400ccdc199b80002480052004370400426601a01c0022940dd7181398121baa00116302630273023375403c602c002604860426ea800458c08cc090c090c080dd5181198101baa001163300400923375e600860406ea8004008c084c078dd518109811180f1baa3021301e37540022c6466006014466ebcc00cc07cdd50008011810180e9baa00b2302000122323300100100322533302000114c0103d87a800013232533301f300500213007330230024bd70099802002000981200118110009ba54800058dd6180e180c9baa00b22323300100100322533301d00114a0264a66603666e3cdd718100010020a511330030030013020001375860346036603660366036603660360046eb0c064004c064c064008dd6180b80098099baa3016002301530160013011375400229309b2b19299980718050008a99980898081baa00214985854ccc038c01c0044c8c94ccc04cc0580084c9263300700125333011300d3012375400226464646464646464a666038603e0042649319808000919299980d980b8008a99980f180e9baa00214985854ccc06cc05000454ccc078c074dd50010a4c2c2c60366ea800458dd6180e800980e8011bad301b001301b002375a603200260320046eb4c05c004c04cdd50008b0b1bac3014001301037540042c601c6ea8004c00402094ccc02cc01cc030dd50008991919191919191919191919299980d180e8010991919191924c6602400a46eb4004cc0440188c0540054ccc064c054c068dd5003899191919191919192999812181380109919191924c603e008603c00a603800c64a666044603c0022a66604a60486ea80205261615333022301b00115333025302437540102930b0a99981119b874801000454ccc094c090dd50040a4c2c2c60446ea801c58c094004c094008c08c004c08c008c084004c084008c07c004c06cdd50038b180800418078048b1bac301b001301b002375860320026032004602e002602e004602a002602a0046026002602600464a666020601e0022a66601a600c601c002294454ccc034c024c0380045280b0b1baa3011001300d37540022c44646600200200644a66602000229309919801801980a001180198090009192999805180300089919299980798090010a4c2c6eb8c040004c030dd50010a999805180180089919299980798090010a4c2c6eb8c040004c030dd50010b18051baa001370e900112999803980198041baa001132323232533300e30110021324994ccc02cc01cc030dd50018991919191919299980a180b8010a4c2c6eb4c054004c054008dd6980980098098011bad3011001300d37540062c2c6eb4c03c004c03c008c034004c024dd50008b12999803180118039baa0011323232323232533300f3012002149858dd6980800098080011bad300e001300e002375a601800260106ea800458dc3a40006eb40055cd2ab9d5573caae7d5d02ba15744981031927100001";
struct MyState {
    state: HydraNodesState,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
    #[serde(default = "default_hosts")]
    hosts: Vec<HostConfig>,
    #[serde(default = "default_nodes")]
    nodes: Vec<NodeConfig>,
}

fn default_nodes() -> Vec<NodeConfig> {
    return vec![];
}
fn default_hosts() -> Vec<HostConfig> {
    return vec![];
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default = "localhost")]
    local_url: String,
    remote_url: Option<String>,
    stats_file_prefix: Option<String>,
    region: String,
    #[serde(default = "default_start_port")]
    start_port: u32,
    #[serde(default = "default_start_port")]
    end_port: u32,

    max_players: usize,
    admin_key_file: PathBuf,
    persisted: bool,
    reserved: bool,
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    #[serde(default = "localhost")]
    local_url: String,
    remote_url: Option<String>,
    #[serde(default = "default_region")]
    region: String,
    port: u32,

    stats_file: Option<String>,

    max_players: usize,
    admin_key_file: PathBuf,
    persisted: bool,
    reserved: bool,
}

fn default_start_port() -> u32 {
    return 4001;
}

fn default_region() -> String {
    "us-east-2".to_string()
}

fn localhost() -> String {
    "ws://127.0.0.1".to_string()
}

#[rocket::main]
async fn main() -> Result<()> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().context("invalid config")?;

    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    let mut nodes = vec![];
    for node in &config.nodes {
        let node = Node::try_new(&node, &tx)
            .await
            .context("failed to construct new node")?;
        nodes.push(node);
    }
    for host in &config.hosts {
        for port in host.start_port..=host.end_port {
            let config = NodeConfig {
                local_url: host.local_url.clone(),
                remote_url: host.remote_url.clone(),
                region: host.region.clone(),
                port,
                stats_file: host
                    .stats_file_prefix
                    .as_ref()
                    .and_then(|prefix| Some(format!("{prefix}-{port}"))),
                admin_key_file: host.admin_key_file.clone(),
                max_players: host.max_players,
                persisted: host.persisted,
                reserved: host.reserved,
            };
            let node = Node::try_new(&config, &tx)
                .await
                .context("failed to construct new node")?;
            nodes.push(node);
        }
    }

    let hydra_state = HydraNodesState::from_nodes(nodes);

    let hydra_state_clone = hydra_state.clone();
    spawn(async move {
        update(hydra_state_clone, rx).await;
    });

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .manage(MyState { state: hydra_state })
        .mount("/", routes![new_game, heads, head, global])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}

async fn update(state: HydraNodesState, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.recv().await {
            Some(HydraData::Received { message, authority }) => {
                let mut state_guard = state.state.write().await;
                let nodes = &mut state_guard.nodes;
                let node = nodes
                    .iter_mut()
                    .find(|n| n.local_connection.to_authority() == authority);
                if let None = node {
                    warn!("Node not found: {}", authority);
                    continue;
                }
                let node = node.unwrap();
                match message {
                    HydraEventMessage::HeadIsOpen(head_is_open) if node.head_id.is_none() => {
                        info!(
                            "updating node {:?} with head_id {:?}",
                            node.local_connection.to_authority(),
                            head_is_open.head_id
                        );
                        node.head_id = Some(head_is_open.head_id.to_string());
                    }
                    HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => {
                        node.stats.calculate_stats(
                            snapshot_confirmed.confirmed_transactions,
                            node.stats_file.clone(),
                        );
                    }

                    HydraEventMessage::TxValid(tx) => match node.add_transaction(tx) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("failed to add transaction {:?}", e);
                        }
                    },
                    _ => {}
                }
            }
            Some(HydraData::Send(_)) => {}
            None => {
                warn!("mpsc disconnected");
                break;
            }
        }
    }
}
