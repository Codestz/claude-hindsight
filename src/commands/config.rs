//! Configuration management commands

use crate::config::Config;
use crate::error::Result;

/// Show current configuration
pub fn show_config() -> Result<()> {
    let config = Config::load()?;

    // Validate
    config.validate()?;

    let toml_string = toml::to_string_pretty(&config)
        .map_err(|e| crate::error::HindsightError::Config(format!("Failed to serialize: {}", e)))?;

    println!("Current configuration:\n");
    println!("{}", toml_string);
    println!("\nConfig file: {:?}", Config::config_path()?);

    Ok(())
}

/// Reset configuration to defaults
pub fn reset_config() -> Result<()> {
    println!("Resetting configuration to defaults...");
    Config::reset()?;
    println!("✓ Configuration reset successfully");
    println!("  Location: {:?}", Config::config_path()?);
    Ok(())
}

/// Validate configuration file
pub fn validate_config() -> Result<()> {
    let config = Config::load()?;

    match config.validate() {
        Ok(()) => {
            println!("✓ Configuration is valid");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Configuration validation failed:");
            eprintln!("  {}", e);
            Err(e)
        }
    }
}

/// Open configuration file in default editor
pub fn edit_config() -> Result<()> {
    let config_path = Config::config_path()?;

    // Ensure config exists
    if !config_path.exists() {
        println!("Config file doesn't exist. Creating default...");
        Config::default().save()?;
    }

    // Get editor from environment or use default
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    println!("Opening config in {}...", editor);
    println!("  Location: {:?}", config_path);

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    if !status.success() {
        return Err(crate::error::HindsightError::Config(format!(
            "Editor {} exited with error",
            editor
        )));
    }

    // Validate after edit
    println!("\nValidating configuration...");
    validate_config()?;

    Ok(())
}
