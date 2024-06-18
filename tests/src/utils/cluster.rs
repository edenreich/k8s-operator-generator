pub async fn setup() -> anyhow::Result<()> {
    tokio::process::Command::new("task")
        .arg("cluster-create")
        .status()
        .await?;
    Ok(())
}

pub async fn teardown() -> anyhow::Result<()> {
    let _ = tokio::process::Command::new("task")
        .arg("cluster-delete")
        .status()
        .await?;
    Ok(())
}
