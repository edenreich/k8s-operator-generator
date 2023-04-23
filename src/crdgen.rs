use kube::CustomResourceExt;

fn main() {
    println!("{}", serde_yaml::to_string(&k8s_operator::Cat::crd()).unwrap());
}
