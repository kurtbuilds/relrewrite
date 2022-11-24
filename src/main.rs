use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result};
use walkdir::WalkDir;
use clap::{Parser};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    path: String,
}

fn rewrite_contents(mut contents: String, depth: usize) -> String {
    let mut offset = 0usize;
    while offset < contents.len() {
        if let Some(m) = (&contents[offset..]).find("from \"@") {
            let start = offset + m + 6;
            let end = offset + m + 7;
            let relative_path_prefix = "../".repeat(depth);
            let relative_path_prefix = &relative_path_prefix[..(3 * depth - 1)];
            contents.replace_range(start..end, relative_path_prefix);
            offset = end;
        } else {
            break;
        }
    }
    contents
}

fn rewrite_file(path: &Path, root: &Path) -> Result<()> {
    let contents = fs::read_to_string(path)?;
    let depth = path.strip_prefix(root).unwrap().components().count() - 1;
    let contents = rewrite_contents(contents, depth);
    fs::write(path, contents)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let js_files = WalkDir::new(&cli.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().strip_prefix(&cli.path).unwrap().starts_with("node_modules"))
        .filter(|e| e.path().extension().map(|e| e == "js").unwrap_or(false));
    let root_path = Path::new(&cli.path);
    for entry in js_files {
        rewrite_file(entry.path(), root_path)?;
    }
    Ok(())
}