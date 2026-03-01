//! Implementation of the `clean` command
//!
//! Resets Hindsight state — database only, or the entire config directory.

use crate::error::Result;
use std::io::{self, Write};

pub fn run(db: bool, all: bool, yes: bool) -> Result<()> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        crate::error::HindsightError::Config("Could not determine config directory".to_string())
    })?;

    let hindsight_dir = config_dir.join("claude-hindsight");

    // Determine target: --all takes precedence; otherwise delete DB only
    // (--db is the default when neither flag is given)
    let _ = db; // explicit flag accepted for clarity, but "not --all" = DB only
    let (target, description) = if all {
        (
            hindsight_dir.clone(),
            format!("entire directory: {}", hindsight_dir.display()),
        )
    } else {
        let db_path = hindsight_dir.join("sessions.db");
        (
            db_path.clone(),
            format!("database: {}", db_path.display()),
        )
    };

    if !target.exists() {
        println!("Nothing to clean — {} does not exist.", target.display());
        return Ok(());
    }

    // Confirm unless --yes / -y
    if !yes {
        print!("This will delete {}.\nContinue? [y/N] ", description);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Delete
    if target.is_dir() {
        std::fs::remove_dir_all(&target)?;
    } else {
        std::fs::remove_file(&target)?;
    }

    println!("Deleted {}.", description);
    println!("\nNext step: run 'hindsight init' to rebuild.");

    Ok(())
}
