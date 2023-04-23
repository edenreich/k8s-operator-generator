use kube::{
    api::{Api, ListParams},
    Client,
};
use tokio::time::{sleep, Duration};
use k8s_operator::Cat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let cats: Api<Cat> = Api::default_namespaced(client);

    loop {
        let lp = ListParams::default().timeout(10);
        let cat_list = cats.list(&lp).await?;

        for cat in cat_list.items {
            println!("Found cat {:?}", cat.metadata.name);
        }

        sleep(Duration::from_secs(5)).await;
    }
}
