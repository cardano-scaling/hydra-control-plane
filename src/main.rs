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
pub const SCRIPT_ADDRESS: &str = "addr_test1wrn8uqxhpnk43wj8l40qtgqsetmjfwgegeszjkl5near33gn3ceas";
pub const SCRIPT_CBOR: &str = "59143401000033232323232323232323232322322323232323232323232232322533301453301549010d48656c6c6f2c20576f726c64210013232323232323232533301c3014301e3754016264a66603a602a603e6ea80044cc00800cdd7181198101baa001153301e49013c65787065637420566572696669636174696f6e4b657943726564656e7469616c287061796d656e745f63726564656e7469616c29203d2061646d696e0016302230233023301f375402a264a66603a6024603e6ea80204c8c8c8c94ccc084c064c08cdd5000899192999811980d98129baa00113253330243370e900218131baa0011323232323253330293021302b3754002264a666054646464646464646464646464646464646464646464646464646464a66608aa66608a66e21200030473754609601e294454ccc114ccc114cdd78058022504a229444ccc114cdd78068032504a229404cc88c8c94cc12924010a6d61705f6368616e676500153304a30183330490034c0103d87a80004c0103d87980001533304900314a22646464a6660986088609c6ea80044c94cc138c070dd40008a99827180e1ba833704010090266e20004cdc10040241bad3052304f37540022a6609a92013c65787065637420536f6d6528646966666572656e636529203d206d6174682e737172742864785f73717561726564202b2064795f7371756172656429001632533304c3371000290000a60103d87a80001533304c33712002900109817998289ba80014bd7009817998289ba8323330010010023370666e000092002480108894ccc13ccdc48010008801099980180180099b833370000266e0c01000520044bd7019b8033018337026eb4c144004dd69828801240086603066e04dd6982898290009bad3051305200248010c134dd5182818269baa002304c3754609e60986ea8008c138c13cc12cdd50029826982718251baa00b30100293330453375e01000294128982518259825982580118248009824801182380098238011822800982298209baa01930433044304430440023042001304200230400013040002303e001303e0013039375405e600200244a66607400229000099b8048008cc008008c0f4004c0040048894ccc0d4cdc4000a4000290000a99981a98168008a40042a6466606c605c66e1800920041333004004300100333706004900209800999802002180080199b83300800248010dc100111b99330020014881003001001222533333303a00213232323232323300c0020013371491010128000025333036337100069007099b80483c80400c54ccc0d8cdc4001a410004266e00cdc0241002800690068a9981ba4929576861742061726520796f7520646f696e673f204e6f2049206d65616e2c20736572696f75736c792e001653330390011337149101035b5d2900004133714911035b5f2000375c6076607866601000266074980102415d003303a375266e2922010129000044bd70111981e26103422c20003303c375266601001000466e28dd7180c0009bae30100014bd701bac3037002375a606a0026466ec0dd4181a8009ba730360013754004264a66606e002266e292201027b7d00002133714911037b5f2000375c6072607464646600200200644a6660740022006266446607a98103422c20003303d3752666012012607400466e292201023a2000333009009303b002337146eb8c064004dd71808800a5eb80c0f0004cc008008c0f4004cc0e13010342207d0033038375200497ae03756004264a66606e002266e29221025b5d00002133714911035b5f2000375c6072607466600c00266070980102415d0033038375200497ae0223303a4c0103422c20003303a375266600c00c00466e28dd7180b0009bae300e0014bd701bac002133006375a0040022646466e2922102682700001323330010013006371a00466e292201012700003222533303633710004900008008991919199803003180580299b8b33700004a66607266e2000920141481805206e3371666e000054ccc0e4cdc4000a4028290300a40dc00866e18009202033706002901019b8e004002375c0046e0120012223233001001004225333036001100413300330380013300200230390012232330010010032253330303028001133714910101300000315333030337100029000099b8a489012d003300200233702900000089980299b8400148050cdc599b803370a002900a240c00066002002444a66605a66e2400920001001133300300333708004900a19b8b3370066e140092014481800044cc03c0400045281bae302f302c37540022a6605492014665787065637420566572696669636174696f6e4b657943726564656e7469616c287061796d656e745f63726564656e7469616c29203d206f6c645f646174756d2e6f776e657200163001302b37540424605c605e00266030002046a66666605a00220022a6604c0442c2a6604c0442c2a6604c0442c2a6604c0442c6054604e6ea800454cc0952412765787065637420496e6c696e65446174756d286461746129203d206f75747075745f646174756d00163029302a302a302637546052604c6ea800454cc09124016a65787065637420536f6d65287363726970745f6f757470757429203d0a202020202020202020206c6973742e66696e64286f7574707574732c20666e286f757470757429207b206f75747075742e61646472657373203d3d207363726970745f61646472657373207d2900163300400923375e6008604c6ea8004008c09cc090dd51813981418121baa3027302437540022a660449214365787065637420536f6d65287363726970745f696e70757429203d207472616e73616374696f6e2e66696e645f696e70757428696e707574732c206f75745f726566290016323300300a23375e6006604a6ea8004008c098c08cdd5005918130009119198008008019129998130008a60103d87a8000132325333024300500213007330290024bd70099802002000981500118140009ba54800054cc0792411f657870656374205370656e64286f75745f72656629203d20707572706f7365001637586044603e6ea802c88c8cc00400400c894ccc08c004528099299981019b8f375c604c00400829444cc00c00c004c098004dd618101810981098109810981098108011bac301f001301f301f0023758603a00260326ea8c070008c06cc070004c05cdd50008a4c2a6602a9211856616c696461746f722072657475726e65642066616c73650013656325333013300b00115333017301637540042930a9980a0088b0a9998099804000899299980c0008a9980a8090b099299980c980e00109924c66010002464a66602e601e60326ea80044c94ccc07000454cc064058584c8c94ccc07800454cc06c060584c8c94ccc08000454cc074068584c8c94ccc08800454cc07c070584c8c94ccc09000454cc084078584c8c94ccc09800454cc08c080584c8c94ccc0a000454cc094088584c8c94ccc0a800454cc09c090584c8c94ccc0b000454cc0a4098584c8c94ccc0b800454cc0ac0a0584c94ccc0bcc0c8008526153302c02916325333333033001153302c02916153302c02916153302c029161375a0022a660580522c6060002606000464a6666660620022a6605404e2c2a6605404e2c2a6605404e2c26eb400454cc0a809c58c0b8004c0b8008c94cccccc0bc00454cc0a00945854cc0a00945854cc0a0094584dd68008a998140128b181600098160011929999998168008a998130118b0a998130118b0a998130118b09bad001153302602316302a001302a00232533333302b0011533024021161533024021161533024021161375a0022a660480422c6050002605000464a6666660520022a6604403e2c2a6604403e2c2a6604403e2c26eb400454cc08807c58c098004c098008c94cccccc09c00454cc0800745854cc0800745854cc080074584dd68008a9981000e8b181200098120011929999998128008a9980f00d8b0a9980f00d8b0a9980f00d8b09bad001153301e01b1630220013022002325333333023001153301c01916153301c01916153301c019161375a0022a660380322c6040002604000464a6666660420022a6603402e2c2a6603402e2c2a6603402e2c26eb400454cc06805c58c078004c068dd50008a9980c00a8b299999980f00088008a9980b80a0b0a9980b80a0b0a9980b80a0b0a9980b80a0b0a9980b0098b19299999980e8008a9980b0098b0a9980b0098b09bac001153301601316153301601316301a001301637540042a660280222c60286ea80054cccccc064004400454cc04803c5854cc04803c5854cc04803c5854cc04803c58cc004020038894ccc040c020c048dd5001099299980a8008a998090010b09919299980b8008a9980a0020b09919299980c8008a9980b0030b09919299980d8008a9980c0040b09919299980e8008a9980d0050b09919299980f8008a9980e0060b0991929998108008a9980f0070b09929998111812801099191919191924ca6660466036604a6ea80184c94ccc0a000454cc094054584c8c94ccc0a800454cc09c05c584c8c94ccc0b000454cc0a4064584c94ccc0b4c0c0008526153302a01a16325333333031001153302a01a16153302a01a16153302a01a161375a0022a660540342c605c002605c00464a66666605e0022a660500302c2a660500302c2a660500302c26eb400454cc0a006058c0b0004c0b0008c94cccccc0b400454cc0980585854cc0980585854cc098058584dd68008a9981300b0b181500098131baa006153302401416330150072533333302b0011533024014161533024014161533024014161375a0022a660480282c66028010466030a66666605400220022a660460262c2a660460262c2a660460262c2a660460262c026a666040603060446ea80244c94ccc09400454cc088048584c8c94ccc09c00454cc090050584c8c94ccc0a400454cc098058584c8c94ccc0ac00454cc0a0060584c94ccc0b0c0bc0084c8c8c8c9263302200401c3302100501b3301f00601a32533302930210011533302d302c37540102930a9981500d0b0a999814980f0008a99981698161baa008149854cc0a80685854ccc0a4cdc3a40080022a66605a60586ea8020526153302a01a16153302a01a16302a375400e2a660520322c64a66666606000220022a660520322c2a660520322c2a660520322c2a660520322c605a002605a00464a66666605c00220022a6604e02e2c2a6604e02e2c2a6604e02e2c2a6604e02e2c6056002605600464a66666605800220022a6604a02a2c2a6604a02a2c2a6604a02a2c2a6604a02a2c6052002605200464a66666605400220022a660460262c2a660460262c2a660460262c2a660460262c604e00260466ea802454cc08404458cc04c028040cc04802c03c54cc07c03c58c94cccccc098004400454cc07c03c5854cc07c03c5854cc07c03c5854cc07c03c58c08c004c08c008c94cccccc09000454cc0740345854cc074034584dd60008a9980e8068b0a9980e8068b181080098108011929999998110008a9980d8058b0a9980d8058b09bac001153301b00b16153301b00b16301f001301f0023253333330200011001153301900916153301900916153301900916153301900916301d001301d00232533333301e0011001153301700716153301700716153301700716153301700716301b001301b00232533333301c00110011533015005161533015005161533015005161533015005163019001301900232533333301a00113253330173016001153330133008301500114a22a6660266016602a002294054cc0500105854cc05001058dd50008a998098018b0a998098018b0a998098018b0a998098018b180b80098099baa00215330110011622323300100100322533301500114984c8cc00c00cc064008c00cc05c00488c94ccc03cc01c0044c94ccc05000454cc04400c584c94ccc054c0600085261533012004163253333330190011533012004161533012004161533012004161533012004161375c002602c00260246ea800c54ccc03cc0100044c94ccc05000454cc04400c584c94ccc054c0600085261533012004163253333330190011533012004161533012004161533012004161533012004161375c002602c00260246ea800c54cc04000858c040dd50011b8748008894ccc030c010c038dd500109929998088008a998070010b0991929998098008a998080020b099299980a180b80109924ca666020601060246ea800c4c94ccc05400454cc048018584c8c94ccc05c00454cc050020584c8c94ccc06400454cc058028584c94ccc068c074008526153301700b1632533333301e001153301700b16153301700b16153301700b161375a0022a6602e0162c6036002603600464a6666660380022a6602a0122c2a6602a0122c2a6602a0122c26eb400454cc05402458c064004c064008c94cccccc06800454cc04c01c5854cc04c01c5854cc04c01c584dd68008a998098038b180b80098099baa0031533011005161533011005163253333330180011533011005161533011005161533011005161375a0022a6602200a2c602a002602a00464a66666602c00220022a6601e0062c2a6601e0062c2a6601e0062c2a6601e0062c6026002601e6ea800854cc03400458894ccc02cc00cc034dd500109929998080008a998068010b0991929998090008a998078020b09919299980a0008a998088030b099299980a980c0010a4c2a6602400e2c64a6666660320022a6602400e2c2a6602400e2c2a6602400e2c26eb400454cc04801c58c058004c058008c94cccccc05c00454cc0400145854cc0400145854cc040014584dd68008a998080028b180a000980a00119299999980a8008a998070018b0a998070018b0a998070018b09bad001153300e003163012001300e37540042a660180022c6e1d2000533333300f0011001153300800616153300800616153300800616153300800616375a002920121657870656374206e65775f646174756d3a2047616d6544617461203d20646174610049011272656465656d65723a2052656465656d6572004901136f6c645f646174756d3a2047616d6544617461005734ae7155ceaab9e5573eae815d0aba25748981051a00da33600001";
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
