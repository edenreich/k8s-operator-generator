use kube::CustomResourceExt;

fn main() {
    println!(
        "---\n{}\n---\n{}",
        serde_yaml::to_string(&k8s_operator::Cat::crd()).unwrap(),
        serde_yaml::to_string(&k8s_operator::Dog::crd()).unwrap()
    );
}
