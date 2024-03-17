use kube::CustomResourceExt;

fn main() {
    let crds = vec![
        k8s_operator::types::cat::Cat::crd(),
        k8s_operator::types::dog::Dog::crd(),
    ];

    for crd in crds {
        match serde_yaml::to_string(&crd) {
            Ok(yaml) => print!("---\n{}", yaml),
            Err(e) => eprintln!("Error serializing CRD to YAML: {}", e),
        }
    }
}
