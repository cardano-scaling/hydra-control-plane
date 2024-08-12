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
pub const SCRIPT_ADDRESS: &str = "addr_test1wzlfx944w3z2gzva8afmvuhuc27pfpae3ey55ttckcgrekq3ynvmt";
pub const SCRIPT_CBOR: &str = "59151d01000033232323232323232323232322322323232323232323232232322533301453301549010d48656c6c6f2c20576f726c64210013232323232323232533301c3014301e3754016264a66603a602a603e6ea80044cc00800cdd7181198101baa001153301e49013c65787065637420566572696669636174696f6e4b657943726564656e7469616c287061796d656e745f63726564656e7469616c29203d2061646d696e0016302230233023301f375402a264a66603a6024603e6ea80204c8c8c8c94ccc084c064c08cdd5000899192999811980d98129baa00113253330243370e900218131baa0011323232323253330293021302b3754002264a6660546464646464646464646464646464646464646464646464646464646464a66608ea66608e66e21200030493754609a022294454ccc11cccc11ccdd78068032504a229444ccc11ccdd78078042504a229404c8c8c94ccc1294ccc128010528899982519baf00d0054a094452889919192999826982298279baa00113253304f301d37500022a6609e603a6ea0cdc1003824899b880013370400e0926eb4c14cc140dd50008a9982724813c65787065637420536f6d6528646966666572656e636529203d206d6174682e737172742864785f73717561726564202b2064795f7371756172656429001632533304d3371000290000a60103d87a80001533304d33712002900109818198291ba80014bd7009818198291ba8323330010010023370666e000092002480108894ccc140cdc48010008801099980180180099b833370000266e0c01000520044bd7019b8033019337026eb4c148004dd69829001240086603266e04dd6982918298009bad3052305300248010c138dd5182898271baa002304d375460a0609a6ea8008c13cc140c130dd50031827182798259baa00c301202b33710900018241baa304c304d002304b001304b304b304b0023049001304900230470013047002304500130453041375403260866088608860880046084002608400460800026080004607c002607c00260726ea80bcc004004894ccc0e800452000133700900119801001181e8009800800911299981a99b88001480005200015333035302d00114800854c8ccc0d8c0b8cdc3001240082666008008600200666e0c009200413001333004004300100333706601000490021b820022373266004002911003001001222533333303a00213232323232323300c0020013371491010128000025333036337100069007099b80483c80400c54ccc0d8cdc4001a410004266e00cdc0241002800690068a9981ba4929576861742061726520796f7520646f696e673f204e6f2049206d65616e2c20736572696f75736c792e001653330390011337149101035b5d2900004133714911035b5f2000375c6076607866601000266074980102415d003303a375266e2922010129000044bd70111981e26103422c20003303c375266601001000466e28dd7180c0009bae30100014bd701bac3037002375a606a0026466ec0dd4181a8009ba730360013754004264a66606e002266e292201027b7d00002133714911037b5f2000375c6072607464646600200200644a6660740022006266446607a98103422c20003303d3752666012012607400466e292201023a2000333009009303b002337146eb8c064004dd71808800a5eb80c0f0004cc008008c0f4004cc0e13010342207d0033038375200497ae03756004264a66606e002266e29221025b5d00002133714911035b5f2000375c6072607466600c00266070980102415d0033038375200497ae0223303a4c0103422c20003303a375266600c00c00466e28dd7180b0009bae300e0014bd701bac002133006375a0040022646466e2922102682700001323330010013006371a00466e292201012700003222533303633710004900008008991919199803003180580299b8b33700004a66607266e2000920141481805206e3371666e000054ccc0e4cdc4000a4028290300a40dc00866e18009202033706002901019b8e004002375c0046e0120012223233001001004225333036001100413300330380013300200230390012232330010010032253330303028001133714910101300000315333030337100029000099b8a489012d003300200233702900000089980299b8400148050cdc599b803370a002900a240c00066002002444a66605a66e2400920001001133300300333708004900a19b8b3370066e140092014481800044cc03c0400045281bae302f302c37540022a6605492014665787065637420566572696669636174696f6e4b657943726564656e7469616c287061796d656e745f63726564656e7469616c29203d206f6c645f646174756d2e6f776e657200163001302b37540424605c605e00266030002046a66666605a00220022a6604c0442c2a6604c0442c2a6604c0442c2a6604c0442c6054604e6ea800454cc0952412765787065637420496e6c696e65446174756d286461746129203d206f75747075745f646174756d00163029302a302a302637546052604c6ea800454cc09124016a65787065637420536f6d65287363726970745f6f757470757429203d0a202020202020202020206c6973742e66696e64286f7574707574732c20666e286f757470757429207b206f75747075742e61646472657373203d3d207363726970745f61646472657373207d2900163300400923375e6008604c6ea8004008c09cc090dd51813981418121baa3027302437540022a660449214365787065637420536f6d65287363726970745f696e70757429203d207472616e73616374696f6e2e66696e645f696e70757428696e707574732c206f75745f726566290016323300300a23375e6006604a6ea8004008c098c08cdd5005918130009119198008008019129998130008a60103d87a8000132325333024300500213007330290024bd70099802002000981500118140009ba54800054cc0792411f657870656374205370656e64286f75745f72656629203d20707572706f7365001637586044603e6ea802c88c8cc00400400c894ccc08c004528099299981019b8f375c604c00400829444cc00c00c004c098004dd618101810981098109810981098108011bac301f001301f301f0023758603a00260326ea8c070008c06cc070004c05cdd50008a4c2a6602a9211856616c696461746f722072657475726e65642066616c73650013656325333013300b00115333017301637540042930a9980a0088b0a9998099804000899299980c0008a9980a8090b099299980c980e00109924c66010002464a66602e601e60326ea80044c94ccc07000454cc064058584c8c94ccc07800454cc06c060584c8c94ccc08000454cc074068584c8c94ccc08800454cc07c070584c8c94ccc09000454cc084078584c8c94ccc09800454cc08c080584c8c94ccc0a000454cc094088584c8c94ccc0a800454cc09c090584c8c94ccc0b000454cc0a4098584c8c94ccc0b800454cc0ac0a0584c94ccc0bcc0c8008526153302c02916325333333033001153302c02916153302c02916153302c029161375a0022a660580522c6060002606000464a6666660620022a6605404e2c2a6605404e2c2a6605404e2c26eb400454cc0a809c58c0b8004c0b8008c94cccccc0bc00454cc0a00945854cc0a00945854cc0a0094584dd68008a998140128b181600098160011929999998168008a998130118b0a998130118b0a998130118b09bad001153302602316302a001302a00232533333302b0011533024021161533024021161533024021161375a0022a660480422c6050002605000464a6666660520022a6604403e2c2a6604403e2c2a6604403e2c26eb400454cc08807c58c098004c098008c94cccccc09c00454cc0800745854cc0800745854cc080074584dd68008a9981000e8b181200098120011929999998128008a9980f00d8b0a9980f00d8b0a9980f00d8b09bad001153301e01b1630220013022002325333333023001153301c01916153301c01916153301c019161375a0022a660380322c6040002604000464a6666660420022a6603402e2c2a6603402e2c2a6603402e2c26eb400454cc06805c58c078004c068dd50008a9980c00a8b299999980f00088008a9980b80a0b0a9980b80a0b0a9980b80a0b0a9980b80a0b0a9980b0098b19299999980e8008a9980b0098b0a9980b0098b09bac001153301601316153301601316301a001301637540042a660280222c60286ea80054cccccc064004400454cc04803c5854cc04803c5854cc04803c5854cc04803c58cc004020038894ccc040c020c048dd5001099299980a8008a998090010b09919299980b8008a9980a0020b09919299980c8008a9980b0030b09919299980d8008a9980c0040b09919299980e8008a9980d0050b09919299980f8008a9980e0060b0991929998108008a9980f0070b0991929998118008a998100080b09929998121813801099191919191924ca66604a603a604e6ea80204c94ccc0a800454cc09c05c584c8c94ccc0b000454cc0a4064584c8c94ccc0b800454cc0ac06c584c8c94ccc0c000454cc0b4074584c94ccc0c4c0d0008526153302e01e16325333333035001132533303230310011533302e3023303000114a22a66605c604c6060002294054cc0bc07c5854cc0bc07c58dd50008a9981700f0b0a9981700f0b0a9981700f0b0a9981700f0b181900098190011929999998198008a9981600e0b0a9981600e0b0a9981600e0b09bad001153302c01c1630300013030002325333333031001153302a01a16153302a01a16153302a01a161375a0022a660540342c605c002605c00464a66666605e0022a660500302c2a660500302c2a660500302c26eb400454cc0a006058c0b0004c0a0dd50040a9981300b0b1980b804929999998168008a9981300b0b0a9981300b0b0a9981300b0b09bad0011533026016163301600a23301a533333302c00110011533025015161533025015161533025015161533025015160155333022301a30243754016264a66604e0022a660480282c26464a6660520022a6604c02c2c26464a6660560022a660500302c26464a66605a0022a660540342c26464a66605e0022a660580382c264a666060606600426464646493198130030101981280380f9981180400f19299981698128008a99981898181baa00a149854cc0b80785854ccc0b4c08800454ccc0c4c0c0dd50050a4c2a6605c03c2c2a66605a66e1d200400115333031303037540142930a9981700f0b0a9981700f0b18171baa009153302d01d16325333333034001153302d01d16153302d01d16153302d01d161375a0022a6605a03a2c6062002606200464a66666606400220022a660560362c2a660560362c2a660560362c2a660560362c605e002605e00464a66666606000220022a660520322c2a660520322c2a660520322c2a660520322c605a002605a00464a66666605c00220022a6604e02e2c2a6604e02e2c2a6604e02e2c2a6604e02e2c6056002605600464a66666605800220022a6604a02a2c2a6604a02a2c2a6604a02a2c2a6604a02a2c6052002604a6ea802c54cc08c04c58cc054030048cc05003404454cc08404458c94cccccc0a00044c94ccc094c09000454ccc084c058c08c0045288a999810980c98118008a5015330220121615330220121637540022a660420222c2a660420222c2a660420222c2a660420222c604a002604a00464a66666604c00220022a6603e01e2c2a6603e01e2c2a6603e01e2c2a6603e01e2c6046002604600464a6666660480022a6603a01a2c2a6603a01a2c26eb000454cc0740345854cc07403458c084004c084008c94cccccc08800454cc06c02c5854cc06c02c584dd60008a9980d8058b0a9980d8058b180f800980f80119299999981000088008a9980c8048b0a9980c8048b0a9980c8048b0a9980c8048b180e800980e80119299999980f00088008a9980b8038b0a9980b8038b0a9980b8038b0a9980b8038b180d800980d80119299999980e00088008a9980a8028b0a9980a8028b0a9980a8028b0a9980a8028b180c800980c80119299999980d000899299980b980b0008a9998099804180a8008a5115333013300b301500114a02a660280082c2a660280082c6ea800454cc04c00c5854cc04c00c5854cc04c00c5854cc04c00c58c05c004c04cdd50010a998088008b11191980080080191299980a8008a4c2646600600660320046006602e0024464a66601e600e002264a6660280022a660220062c264a66602a60300042930a998090020b19299999980c8008a998090020b0a998090020b0a998090020b0a998090020b09bae0013016001301237540062a66601e6008002264a6660280022a660220062c264a66602a60300042930a998090020b19299999980c8008a998090020b0a998090020b0a998090020b0a998090020b09bae0013016001301237540062a660200042c60206ea8008dc3a400444a6660186008601c6ea80084c94ccc04400454cc038008584c8c94ccc04c00454cc040010584c94ccc050c05c0084c9265333010300830123754006264a66602a0022a6602400c2c26464a66602e0022a660280102c26464a6660320022a6602c0142c264a666034603a0042930a9980b8058b19299999980f0008a9980b8058b0a9980b8058b0a9980b8058b09bad001153301700b16301b001301b00232533333301c0011533015009161533015009161533015009161375a0022a6602a0122c6032002603200464a6666660340022a6602600e2c2a6602600e2c2a6602600e2c26eb400454cc04c01c58c05c004c04cdd50018a998088028b0a998088028b19299999980c0008a998088028b0a998088028b0a998088028b09bad001153301100516301500130150023253333330160011001153300f00316153300f00316153300f00316153300f003163013001300f37540042a6601a0022c44a6660166006601a6ea80084c94ccc04000454cc034008584c8c94ccc04800454cc03c010584c8c94ccc05000454cc044018584c94ccc054c0600085261533012007163253333330190011533012007161533012007161533012007161375a0022a6602400e2c602c002602c00464a66666602e0022a6602000a2c2a6602000a2c2a6602000a2c26eb400454cc04001458c050004c050008c94cccccc05400454cc03800c5854cc03800c5854cc03800c584dd68008a998070018b180900098071baa002153300c00116370e9000299999980780088008a998040030b0a998040030b0a998040030b0a998040030b1bad001490121657870656374206e65775f646174756d3a2047616d6544617461203d20646174610049011272656465656d65723a2052656465656d6572004901136f6c645f646174756d3a2047616d6544617461005734ae7155ceaab9e5573eae815d0aba25748981051a00da33600001";
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
