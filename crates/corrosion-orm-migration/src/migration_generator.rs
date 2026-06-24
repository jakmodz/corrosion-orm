use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};
use chrono::Local;
use corrosion_orm::model::snapshot::ModelsSnapshot;

use crate::{
    diff_engine::{self, DiffEngine, render_down_code, render_up_code},
    driver_util::get_dialect,
};

pub async fn create_migration(
    name: Option<&String>,
    old_snapshot: &Option<ModelsSnapshot>,
    new_snapshot: &ModelsSnapshot,
) -> Result<String> {
    let time_stamp = Local::now().format("%Y%m%d%H%M%S").to_string();

    let default_name = "auto_migration".to_string();
    let raw_name = name.unwrap_or(&default_name);
    let sanitize = |s: &str| {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    };

    let migration_name = format!("m{}_{}", time_stamp, sanitize(raw_name));
    let migration_file_name = format!("{}.rs", migration_name);
    let src_path = Path::new("./src");
    let migration_path = src_path.join(&migration_file_name);

    let dialect = get_dialect();
    let (up_code, down_code) = if let Some(old) = old_snapshot {
        let ops = DiffEngine::diff(old.models(), new_snapshot.models());
        (
            render_up_code(&ops, dialect),
            render_down_code(&ops, dialect),
        )
    } else {
        let ops: Vec<_> = new_snapshot
            .models()
            .iter()
            .map(|m| diff_engine::MigrationOp::CreateTable(m.clone()))
            .collect();
        (render_up_code(&ops, dialect), String::new())
    };

    let raw_template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/MIGRATION_TEMPLATE.rs"
    ));

    let tpl = raw_template
        .replace("{{migration_name}}", &migration_name)
        .replace("{{up_code}}", &up_code)
        .replace("{{down_code}}", &down_code);

    let mut file = File::create_new(&migration_path).with_context(|| {
        format!(
            "failed to create file for migration {}",
            migration_file_name
        )
    })?;
    file.write_all(tpl.as_bytes())?;
    eprintln!("Created migration: {}", migration_path.display());
    Ok(migration_name)
}
