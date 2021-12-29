// Copyright 2021 Chay Nabors.

use std::env;
use std::error::Error;
use std::io::Write;
use std::path::Path;

use log::info;

fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = "OUT_DIR";

    println!("cargo:rerun-if-changed=\"client.json\"");
    println!("cargo:rerun-if-changed=\"content\"");
    println!("cargo:rerun-if-env-changed=\"{}\"", target_dir);

    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}: {}] {}", record.level(), record.metadata().target(), record.args()))
        .filter(None, log::LevelFilter::Info)
        .filter(Some("gfx_backend_dx11"), log::LevelFilter::Warn)
        .filter(Some("gfx_backend_vulkan"), log::LevelFilter::Warn)
        .filter(Some("wgpu_core"), log::LevelFilter::Warn)
        .init();

    let env_var = env::var(target_dir)?;
    let output_path = Path::new(&env_var).join("..").join("..").join("..");

    info!("Copying config");
    copy_config(&output_path)?;
    info!("Copied config");

    info!("Copying content");
    copy_content(&output_path)?;
    info!("Copied content");

    Ok(())
}

fn copy_config(output_path: &Path) -> Result<(), Box<dyn Error>> {
    let config = Path::new("client.json");
    if config.exists() {
        let output_path = output_path.clone().join("client.json");
        std::fs::copy(config, output_path)?;
    }

    Ok(())
}

fn copy_content(output_path: &Path) -> Result<(), Box<dyn Error>> {
    let content = Path::new("content");
    if content.exists() {
        info!("Creating content directory");
        let output_path = output_path.clone().join("content");
        if let Err(e) = std::fs::create_dir(&output_path) {
            match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => return Err(Box::new(e)),
            }
        }

        info!("Changing directory permissions");
        let mut perms = std::fs::metadata(&output_path)?.permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(&output_path, perms)?;

        info!("Reading content paths to array");
        let mut files = vec![];
        for file in std::fs::read_dir(content)? {
            files.push(file?);
        }

        info!("Copying content to destination");
        for file in files {
            let output_path = output_path.clone().join(file.file_name());
            std::fs::copy(file.path(), &output_path)?;
        }
    }

    Ok(())
}
