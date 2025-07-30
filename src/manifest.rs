use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use path_slash::PathExt;

use crate::haxe;

#[derive(clap::Subcommand)]
pub enum Cli {
    // WIP: hidden since I never finished this subcommand
    #[command(hide = true)]
    // TODO: Figure out what's the current utility of this subcommand
    /// Manage mm2 asset manifest
    Manifest {
        #[command(subcommand)]
        command: Command,
    },
}

#[derive(clap::Subcommand)]
pub enum Command {
    Generate {
        path: PathBuf,

        #[arg(short, long)]
        output: PathBuf,
    },
}

pub fn run(Cli::Manifest { command }: Cli) {
    let Command::Generate { path, output } = command;

    let initial_working_dir = std::env::current_dir().unwrap();

    let mut manifest_values = vec![];

    std::env::set_current_dir(path).unwrap();
    for dir in ["assets", "libraries"] {
        visit_files(dir, &mut |file_path| {
            let file_type = match get_manifest_file_type(file_path) {
                Ok(x) => x,
                Err(err) => {
                    eprintln!("{err}");
                    return Ok(());
                }
            };

            let slash_file_path = Cow::Owned(file_path.to_slash_lossy().into_owned());
            let file = haxe::Value::String(slash_file_path);
            let file_type = haxe::Value::String(file_type.into());

            manifest_values.push(haxe::Value::Struct {
                fields: [
                    ("path".into(), file.clone()),
                    ("type".into(), file_type),
                    ("id".into(), file),
                ]
                .into(),
            });

            Ok(())
        })
        .unwrap();
    }

    let data = haxe::to_string(&[haxe::Value::Array(manifest_values)]);
    std::env::set_current_dir(initial_working_dir).unwrap();
    std::fs::write(output, data).unwrap();
}

fn get_manifest_file_type(path: &Path) -> Result<&'static str, Cow<'static, str>> {
    let Some(ext) = path.extension().map(|ext| ext.to_string_lossy()) else {
        return Err(format!("\"{path:?}\" has no extension").into());
    };

    match ext.as_ref() {
        "ogg" => {
            let parent = path.components().nth_back(1).and_then(|p| {
                let Component::Normal(path) = p else {
                    return None;
                };

                path.to_str()
            });

            match parent {
                Some("music") => Ok("MUSIC"),
                Some("effects") => Ok("SOUND"),
                Some(p) => Err(format!("Invalid parent {p:?} for music file {path:?}").into()),
                None => Err(
                    format!("Sound file {path:?} must have parent to determine file type").into(),
                ),
            }
        }

        "bik" => Ok("BINARY"),

        "otf" | "ttf" => Ok("FONT"),

        "jpg" | "png" => Ok("IMAGE"),

        "csv" | "dat" | "json" | "strings" | "txt" | "version" => Ok("TEXT"),

        ext => Err(format!("Invalid ext {ext}: no known file type").into()),
    }
}

fn visit_files(
    dir: impl AsRef<Path>,
    visitor: &mut impl FnMut(&Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let dir = dir.as_ref();

    if !dir.is_dir() {
        visitor(dir)?;
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        visit_files(entry.path(), visitor)?;
    }

    Ok(())
}
