use std::process::Command;

use anyhow::{anyhow, Result};

pub fn check_installed() -> Result<()> {
    match Command::new("gh").arg("--version").output() {
        Ok(o) if o.status.success() => Ok(()),
        Ok(_) => Err(anyhow!("gh found but returned an error — run `gh auth login`")),
        Err(_) => Err(anyhow!("gh not found — install it from https://cli.github.com")),
    }
}

pub fn run(args: &[String]) -> Result<String> {
    let output = Command::new("gh")
        .args(args)
        .env("NO_COLOR", "1")
        .env("GH_FORCE_TTY", "")
        .output()
        .map_err(|e| anyhow!("failed to run gh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
