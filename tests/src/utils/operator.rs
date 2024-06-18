use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn deploy() -> anyhow::Result<(), anyhow::Error> {
    let _ = tokio::process::Command::new("task")
        .arg("package")
        .status()
        .await?;

    let mut child = tokio::process::Command::new("task")
        .arg("deploy-operator")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to deploy the operator");

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
