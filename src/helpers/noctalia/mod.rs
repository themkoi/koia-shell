use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct ConfigSummary {
    pub bar_thickness: u32,
    pub bar_margin_edge: u32,
}

#[derive(Deserialize, Debug, Default)]
pub struct BarConfig {
    pub thickness: Option<u32>,
    pub margin_edge: Option<u32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct BarSection {
    #[serde(flatten)]
    pub bars: HashMap<String, BarConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct NoctaliaExportedConfig {
    pub bar: Option<BarSection>,
}


fn run_command(cmd: &str, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(cmd).args(args).output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        Err(format!("Command `{} {}` failed", cmd, args.join(" ")).into())
    }
}

pub fn read_config_summary(
    target_bar: Option<&str>,
) -> Result<ConfigSummary, Box<dyn std::error::Error>> {
    let bar_name = target_bar.unwrap_or("default");

    let exported_toml = run_command("noctalia", &["config", "export"])?;
    let config: NoctaliaExportedConfig = toml::from_str(&exported_toml)?;

    let bar_config = config
        .bar
        .as_ref()
        .and_then(|b| b.bars.get(bar_name));

    let bar_thickness = bar_config.and_then(|b| b.thickness).unwrap_or(0);
    let bar_margin_edge = bar_config.and_then(|b| b.margin_edge).unwrap_or(0);

    Ok(ConfigSummary {
        bar_thickness,
        bar_margin_edge,
    })
}

pub fn is_dark_mode() -> Result<bool, Box<dyn std::error::Error>> {
    let output = run_command("noctalia", &["msg", "theme-mode-get"])?;

    match output.trim() {
        "dark" => Ok(true),
        "light" => Ok(false),
        mode => Err(format!("Unknown theme mode: {}", mode).into()),
    }
}