use kube::CustomResourceExt;

fn main() {
    print!(
        "---\n{}\n---\n{}",
        serde_yaml::to_string(&k8s_operator::types::cat::Cat::crd()).unwrap(),
        serde_yaml::to_string(&k8s_operator::types::dog::Dog::crd()).unwrap(),
    );
}
