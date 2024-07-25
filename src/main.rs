use std::path::PathBuf;

use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::global::global;
use routes::head::head;
use routes::heads::heads;
use routes::new_game::new_game;
use tokio::{
    spawn,
    sync::mpsc::{self, error::TryRecvError, UnboundedReceiver, UnboundedSender},
};

use model::{
    hydra::{
        hydra_message::{HydraData, HydraEventMessage},
        state::HydraNodesState,
    },
    node::Node,
};
use serde::Deserialize;

#[macro_use]
extern crate rocket;

mod model;
mod routes;

// this is a temporary way to store the script address
pub const SCRIPT_ADDRESS: &str = "addr_test1wp096khk46y6mxmnl0pqe446kdlzswsjpyd67ju6gs9sldqjkl4wx";
pub const SCRIPT_CBOR: &str = "59026701000032323232323232322223253330073232323232323232323232323253330143370e9000180980089919191980080080291299980d8008a5013232533301a3371e00400a29444cc010010004c078008dd7180e0009bae30190013012001163017301830110103758602c002602c002602a00260280026026002602400260220026020002601e002600e0026018002600a00429309b2b199119299980499b87480000044c8c8c8c8c8c8c8c94ccc050c0580084c8c8c926323300100100422533301800114984c8cc00c00cc06c008c03cc064004c94ccc04ccdc3a40000022646464646464a666038603c0042646493180a00219299980d19b874800000454ccc074c060018526161533301a3370e90010008a99980e980c0030a4c2c2a66603466e1d20040011533301d301800614985858c06001458dd6980e000980e001180d000980d001180c00098088028b180880219299980919b87480000044c8c94ccc05cc06400852616375c602e002602000c2a66602466e1d20020011323253330173019002149858dd7180b80098080030b18080028b1bac3014001301400230120013012002301000130100023370e900118059baa300e001300700216300700123253330083370e90000008991919192999807980880109924c64a66601a66e1d20000011323232323232323232323232533301c301e002149858dd6980e000980e0011bad301a001301a002375a603000260300046eb4c058004c058008dd6980a000980a0011bad3012001300b00416300b00316375a601e002601e004601a002600c0042c600c0020064600a6ea80048c00cdd5000ab9a5573aaae7955cfaba157441";
struct MyState {
    state: HydraNodesState,
    config: Config,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
    max_players: u64,
    nodes: Vec<NodeConfig>,
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    #[serde(default = "localhost")]
    connection_url: String,
    admin_key_file: PathBuf,
    persisted: bool,
}

fn localhost() -> String {
    "ws://127.0.0.1:4001".to_string()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().expect("invalid config");

    let (tx, rx): (UnboundedSender<HydraData>, UnboundedReceiver<HydraData>) =
        mpsc::unbounded_channel();

    let mut nodes = vec![];
    for node in &config.nodes {
        let node = Node::try_new(&node, &tx).await.expect("failed to connect");
        nodes.push(node);
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
        .manage(MyState {
            state: hydra_state,
            config,
        })
        .mount("/", routes![new_game, heads, head, global])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}

async fn update(state: HydraNodesState, mut rx: UnboundedReceiver<HydraData>) {
    loop {
        match rx.try_recv() {
            Ok(data) => match data {
                HydraData::Received { message, authority } => {
                    let mut state_guard = state.state.write().await;
                    let nodes = &mut state_guard.nodes;
                    let node = nodes
                        .iter_mut()
                        .find(|n| n.connection_info.to_authority() == authority);
                    if let None = node {
                        warn!("Node not found: ${:?}", authority);
                        continue;
                    }
                    let node = node.unwrap();
                    match message {
                        HydraEventMessage::HeadIsOpen(head_is_open) => {
                            if let None = node.head_id {
                                info!(
                                    "updating node {:?} with head_id {:?}",
                                    node.connection_info.to_authority(),
                                    head_is_open.head_id
                                );
                                node.head_id = Some(head_is_open.head_id.to_string());
                            }
                        }
                        HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => node
                            .stats
                            .calculate_stats(snapshot_confirmed.confirmed_transactions),
                        HydraEventMessage::TxValid(tx) => match node.add_transaction(tx) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("failed to add transaction {:?}", e);
                            }
                        },
                        _ => {} //println!("Unhandled message: {:?}", message),
                    }
                }
                HydraData::Send(_) => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                warn!("mpsc disconnected");
                break;
            }
        }
    }
}
