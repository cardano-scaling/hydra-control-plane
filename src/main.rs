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
pub const SCRIPT_ADDRESS: &str = "addr_test1wr6tahvg0jj27ur5trmvydcpn269r2jzeadtck2aawcdsuqjfrgeu";
pub const SCRIPT_CBOR: &str = "5905a70100003232323232323223232323232323232322322533300e32323232323232325333016301430173754016264a66602e602a60306ea80044cc00800cdd7180e180c9baa00116301b301c301c3018375402a264a66602e602460306ea80204c8c8c94ccc068c060c06cdd500089919299980e180d180e9baa001132533301d3015301e37540022646464a666040603c60426ea80044c94ccc084c8c8c8c8c8c8c8c8c8c8c94ccc0b14ccc0b0cdc424000605a6ea8c0c402c5288a99981619981619baf0070024a0944528899981619baf0090044a09445280991929998171816000899299981798168008a511533302f302a00114a22940c0bcdd50010a9998171814800899299981798138008a511533302f302a00114a22940c0bcdd5001099299981798138008a511533302f302d00114a22940c0bcdd500118171baa3032302f375400e6062605c6ea8004c0c0c0c4008c0bc004c0bc008c0b4004c0b4c0a4dd500518159816001181500098150011814000981400098119baa02013300c00d00114a06eb8c094c088dd50008b1812181298109baa01e30150013022301f37540022c604260446044603c6ea8c084c078dd50008b19802004119baf3004301e3754002004603e60386ea8c07cc080c070dd5180f980e1baa00116323300300923375e6006603a6ea8004008c078c06cdd50051180f00091191980080080191299980f0008a60103d87a800013232533301d300500213374a90001981080125eb804cc010010004c088008c08000458c06cc060dd500591191980080080191299980e0008a50132533301a3371e6eb8c07c008010528899801801800980f8009bac3019301a301a301a301a301a301a00237586030002603060300046eb0c058004c048dd5180a801180a180a80098081baa00114984d958c94ccc034c02c00454ccc040c03cdd50010a4c2c2a66601a601000226464a666024602a004264932999807980698081baa00113232323232323232533301a301d002132498cc0400048c94ccc064c05c00454ccc070c06cdd50010a4c2c2a66603260280022a66603860366ea80085261616301937540022c6eb0c06c004c06c008dd6980c800980c8011bad30170013017002375a602a00260226ea80045858c04c004c03cdd50010b18069baa00130010092533300a3008300b37540022646464646464646464646464a666032603800426464646464931980980291bad0013301200623016001533301830163019375400e26464646464646464a666046604c004264646464931810002180f802980e803192999810980f8008a99981218119baa00814985854ccc084c07000454ccc090c08cdd50040a4c2c2a66604260320022a66604860466ea802052616163021375400e2c604800260480046044002604400460400026040004603c00260346ea801c58c044020c04002458dd6180d000980d0011bac3018001301800230160013016002301400130140023012001301200232533300f300e0011533300c3007300d00114a22a6660186014601a00229405858dd5180800098061baa00116370e90021119198008008019129998070008a4c26466006006602400460066020002464a666010600c00226464a66601a60200042930b1bae300e001300a37540042a666010600600226464a66601a60200042930b1bae300e001300a37540042c60106ea8004dc3a40044a66600a6006600c6ea80044c8c8c8c94ccc030c03c0084c92653330093007300a37540062646464646464646464646464a66603060360042930b1bad30190013019002375a602e002602e0046eb4c054004c054008dd6980980098098011bad30110013011002375a601e00260166ea800c5858dd698068009806801180580098039baa00116253330043002300537540022646464646464a66601a60200042930b1bad300e001300e002375a601800260180046eb4c028004c018dd50008b1b87480015cd2ab9d5573caae7d5d02ba157441";
struct MyState {
    state: HydraNodesState,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ttl_minutes: u64,
    nodes: Vec<NodeConfig>,
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    #[serde(default = "localhost")]
    local_url: String,
    max_players: usize,
    remote_url: Option<String>,
    admin_key_file: PathBuf,
    persisted: bool,
}

fn localhost() -> String {
    "ws://127.0.0.1:4001".to_string()
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
                    warn!("Node not found: ${:?}", authority);
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
                    HydraEventMessage::SnapshotConfirmed(snapshot_confirmed) => node
                        .stats
                        .calculate_stats(snapshot_confirmed.confirmed_transactions),

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
