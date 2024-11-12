use hydra_control_plane_operator::custom_resource::HydraDoomNode;
use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&HydraDoomNode::crd()).unwrap())
}
