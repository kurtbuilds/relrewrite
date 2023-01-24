use std::fs;
use std::path::{Path};
use anyhow::{anyhow, Result};
use walkdir::WalkDir;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
struct AbsoluteToRelative {
    #[clap(long)]
    dry: bool,
    path: String,
}

#[derive(Parser)]
struct RelativeToAbsolute {
    #[clap(long)]
    dry: bool,
    fpath: String,
}

#[derive(Subcommand)]
enum Command {
    AbsoluteToRelative(AbsoluteToRelative),
    RelativeToAbsolute(RelativeToAbsolute),
}

fn rewrite_absolute_import_to_relative(mut contents: String, depth: usize) -> String {
    let mut offset = 0usize;
    while offset < contents.len() {
        if let Some(m) = (&contents[offset..]).find("from \"@/") {
            let start = offset + m + 6;
            let end = offset + m + 7;
            let relative_path_prefix = "../".repeat(depth);
            let relative_path_prefix = &relative_path_prefix[..(3 * depth - 1)];
            contents.replace_range(start..end, relative_path_prefix);
            offset = end;
        } else if let Some(m) = (&contents[offset..]).find("from \"@\"") {
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

fn rewrite_absolute_import_to_relative_for_path(path: &Path, root: &Path) -> Result<()> {
    let contents = fs::read_to_string(path)?;
    let depth = path.strip_prefix(root).unwrap().components().count() - 1;
    let contents = rewrite_absolute_import_to_relative(contents, depth);
    fs::write(path, contents)?;
    Ok(())
}

fn rewrite_absolute_to_relative(opt: AbsoluteToRelative) -> Result<()> {
    let js_files = WalkDir::new(&opt.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().strip_prefix(&opt.path).unwrap().starts_with("node_modules"))
        .filter(|e| e.path().extension().map(|e| e == "js").unwrap_or(false));
    let root_path = Path::new(&opt.path);
    for entry in js_files {
        rewrite_absolute_import_to_relative_for_path(entry.path(), root_path)?;
    }
    Ok(())
}

fn relative_to_absolute_contents(mut contents: String) -> String {
    let mut offset = 0usize;
    while offset < contents.len() {
        if let Some(m) = (&contents[offset..]).find("from \"../../lib-ts") {
            let start = offset + m + 6;
            let end = offset + m + 6 + 12;
            contents.replace_range(start..end, "@bs/lib");
            offset = end;
        } else {
            break;
        }
    }
    contents
}

fn relative_to_absolute(opt: RelativeToAbsolute) -> Result<()> {
    let contents = fs::read_to_string(&opt.fpath)?;
    let contents = relative_to_absolute_contents(contents);
    fs::write(&opt.fpath, contents)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::AbsoluteToRelative(abs) => {
            rewrite_absolute_to_relative(abs)
        }
        Command::RelativeToAbsolute(rel) => {
            if [".ts", ".tsx"].into_iter().any(|ext| rel.fpath.ends_with(ext)) {
                relative_to_absolute(rel)
            } else {
                Ok(())
            }
        }
    }
}