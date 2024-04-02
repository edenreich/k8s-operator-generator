fn main() {}

#[cfg(test)]
mod test {
    use anyhow::{Error, Ok};
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use k8s_operator::{add_finalizer, types::cat::Cat};
    use kube::{
        api::{Api, ObjectMeta},
        Client,
    };
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    async fn setup_cluster() -> anyhow::Result<(), Error> {
        let _ = Command::new("task").arg("cluster-create").status();
        Ok(())
    }

    async fn setup_client() -> Client {
        let client = Client::try_default()
            .await
            .expect("Failed to create client");
        client
    }

    async fn deploy_operator() -> Result<(), Error> {
        let _ = Command::new("task").arg("package").status().await?;

        let mut child = Command::new("task")
            .arg("deploy-operator")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to apply the CRD");

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        while let Some(line) = stdout_reader.next_line().await? {
            println!("Stdout: {}", line);
        }

        while let Some(line) = stderr_reader.next_line().await? {
            eprintln!("Stderr: {}", line);
        }

        let _ = child.wait().await?;

        Ok(())
    }

    async fn teardown_cluster() {
        let _ = Command::new("task")
            .arg("cluster-delete")
            .spawn()
            .expect("Failed to execute command");
    }

    #[tokio::test]
    async fn test_cat_crds_exist() -> anyhow::Result<(), Error> {
        setup_cluster().await?;
        deploy_operator().await?;
        let client = setup_client().await;

        let crds: Api<CustomResourceDefinition> = Api::all(client.clone());
        let params = kube::api::ListParams {
            field_selector: Some("metadata.name=cats.example.com".to_string()),
            ..Default::default()
        };
        let crds_list = crds.list(&params).await?;

        teardown_cluster().await;

        assert_eq!(
            crds_list.items.len(),
            1,
            "CRDs for cats.example.com not found"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_add_finalizer() -> anyhow::Result<(), Error> {
        setup_cluster().await?;
        deploy_operator().await?;
        let client = setup_client().await;
        let api: Api<Cat> = Api::namespaced(client.clone(), "default");
        let mut resource = Cat {
            metadata: ObjectMeta {
                name: Some("test-cat".to_string()),
                ..Default::default()
            },
            spec: Default::default(),
            status: Default::default(),
        };

        // deploy the resource
        match api.get("test-cat").await {
            std::result::Result::Ok(_) => {}
            Err(_) => {
                api.create(&Default::default(), &resource).await?;
            }
        }

        // add finalizer
        add_finalizer(&mut resource, api.clone()).await?;

        // get the resource
        let cat = api.get("test-cat").await?;

        // check if the finalizer is added
        assert_eq!(
            cat.metadata.finalizers,
            Some(vec!["finalizers.example.com".to_string()])
        );

        teardown_cluster().await;

        Ok(())
    }
}
