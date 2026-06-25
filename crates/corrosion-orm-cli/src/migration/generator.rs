use anyhow::{Context, Result};
use std::{fs, path::Path};

pub async fn init_migrations(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    render_mod_rs(path).await?;
    Ok(())
}
async fn render_mod_rs(path: &Path) -> Result<()> {
    fn write_resource(path: &Path, filename: &str, contents: &str) -> Result<()> {
        let out = path.join(filename);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&out)
            .with_context(|| format!("refusing to overwrite {}", out.display()))?;
        std::io::Write::write_all(&mut file, contents.as_bytes())
            .with_context(|| format!("failed to write {}", out.display()))?;
        Ok(())
    }

    const README: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/README_MIGRATIONS.md"
    ));
    write_resource(path, "README.md", README)?;

    let src_path = path.join("src");
    fs::create_dir_all(&src_path)
        .with_context(|| format!("failed to create {}", src_path.display()))?;

    const CARGO_TOML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/EXAMPLE_TOML.toml"
    ));
    write_resource(path, "Cargo.toml", CARGO_TOML)?;
    const MAIN_MIGRATION: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/MAIN_MIGRATION.rs"
    ));
    write_resource(&src_path, "main.rs", MAIN_MIGRATION)?;

    const MIGRATOR: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/MIGRATOR.rs"
    ));
    write_resource(&src_path, "migrator.rs", MIGRATOR)?;

    const BUILD_MIGRATOR: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/BUILD_MIGRATOR.rs"
    ));
    write_resource(path, "build.rs", BUILD_MIGRATOR)?;
    Ok(())
}
