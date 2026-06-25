use std::{env, fs, io::Write, path::PathBuf};

fn is_migration_file(name: &str) -> bool {
    if !name.ends_with(".rs") {
        return false;
    }

    let Some(stem) = name.strip_suffix(".rs") else {
        return false;
    };

    if stem == "main" || stem == "migrator" || stem == "registry" || stem == "lib" {
        return false;
    }

    stem.starts_with('m')
        && stem
            .chars()
            .nth(1)
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
}

fn main() {
    println!("cargo:rerun-if-changed=src");

    let src_dir = PathBuf::from("src");
    let mut modules: Vec<String> = fs::read_dir(&src_dir)
        .expect("failed to read src directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();
            if !is_migration_file(&name) {
                return None;
            }
            name.strip_suffix(".rs").map(|s| s.to_string())
        })
        .collect();

    modules.sort();

    let mut modules_out = String::new();
    let mut list_out = String::from("vec![\n");

    for module in &modules {
        modules_out.push_str(&format!(
            "mod {} {{ include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/{}.rs\")); }}\n",
            module, module
        ));
        list_out.push_str(&format!(
            "    Box::new(crate::{}::GeneratedMigration) as Box<dyn corrosion_orm_migration::MigrationTrait>,\n",
            module
        ));
    }

    list_out.push_str("]\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let mut modules_file = fs::File::create(out_dir.join("migration_modules.rs"))
        .expect("failed to create migration_modules.rs");
    modules_file
        .write_all(modules_out.as_bytes())
        .expect("failed to write migration_modules.rs");

    let mut list_file = fs::File::create(out_dir.join("migration_list.rs"))
        .expect("failed to create migration_list.rs");
    list_file
        .write_all(list_out.as_bytes())
        .expect("failed to write migration_list.rs");
}
