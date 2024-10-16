use std::path::Path;

pub async fn setup() -> anyhow::Result<()> {
    let root_dir = Path::new("..");
    tokio::process::Command::new("task")
        .arg("cluster-create")
        .current_dir(&root_dir)
        .status()
        .await?;
    Ok(())
}

pub async fn teardown() -> anyhow::Result<()> {
    let root_dir = Path::new("..");
    let _ = tokio::process::Command::new("task")
        .arg("cluster-delete")
        .current_dir(&root_dir)
        .status()
        .await?;
    Ok(())
}
