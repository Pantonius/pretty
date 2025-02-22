mod error;
use crate::error::{CompilationError, DownloadError, PrettyError};

use std::{
    env::{self, temp_dir},
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use clap::{value_parser, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {
    /// file to be compiled
    #[arg(value_parser=value_parser!(PathBuf))]
    path: Option<std::path::PathBuf>,

    /// whether to open the output or not
    #[arg(short = 's', long = "show")]
    show: bool,

    /// whether the input source is a HedgeDoc instance
    #[arg(short = 'H')]
    hedgedoc: bool,

    /// the domain of the source instance (e.g. HedgeDoc); For instance: https://demo.hedgedoc.org
    #[arg(short = 'D', long = "domain")]
    domain: Option<String>,

    /// defines the ID of the hedgedoc document on the configured instance
    #[arg(short = 'I', long = "document_id")]
    document_id: Option<String>,

    /// whether to keep the downloaded markdown (in case a source has been selected)
    #[arg(short = 'k', long = "keep")]
    keep: bool,
}

fn initialize(ewd: &PathBuf) -> Result<(), PrettyError> {
    // prepare latex templates
    let template_style = include_str!("../templates/anton.sty");
    let template = include_str!("../templates/template.tex");

    let base_path = PathBuf::from("~/.config/pretty");
    fs::create_dir_all(&base_path)?;

    let template_style_path = base_path.clone().join("anton.sty");
    let template_path = base_path.clone().join("template.tex");

    let mut template_style_file = File::create(&template_style_path)?;
    template_style_file.write_all(template_style.as_bytes())?;

    let mut template_file = File::create(&template_path)?;
    template_file.write_all(template.as_bytes())?;

    Ok(())
}

async fn download(url: &String, output_file_path: &PathBuf) -> Result<(), DownloadError> {
    let res = reqwest::get(url).await?;

    if !res.status().is_success() {
        return Err(DownloadError::StatusCode(res.status()));
    }

    let body = res.text().await?;

    // save to file
    let mut file = File::create(output_file_path)?;
    file.write_all(body.as_bytes())?;

    Ok(())
}

fn compile_markdown(markdown_file_path: &PathBuf) -> Result<(), CompilationError> {
    // execute pandoc
    if cfg!(target_os = "linux") {
        let path = markdown_file_path
            .to_str()
            .ok_or_else(|| CompilationError::FileNotFound)?;

        let cmd = format!(
            "pandoc \"{}\" -f markdown -t pdf --template=\"~/.config/pretty/template.tex\" --pdf-engine=xelatex -o {}",
            path, "pretty.pdf"
        );
        println!("Executing: {}", cmd);
        let output = Command::new("sh").arg("-c").arg(cmd).output()?;

        if !output.status.success() {
            return Err(CompilationError::Pandoc(format!(
                "Unexpected exit: {}",
                output.status.to_string()
            )));
        }
    } else {
        return Err(CompilationError::OSUnsupported);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PrettyError> {
    let args = CLI::parse();
    let tmp_dir = temp_dir();
    let tmp_file = tmp_dir.clone().join("some.md");
    let ewd = env::current_exe().unwrap_or_else(|_| {
        return tmp_dir.clone();
    });

    initialize(&ewd)?;

    // decide on the path to take
    if args.hedgedoc {
        // remote source: Use `domain` and `document_id` to download and compile

        // check that `domain` and `document_id` are given
        if let Some(some_domain) = args.domain {
            if let Some(some_document_id) = args.document_id {
                let url = format!("{}/{}/download", some_domain, some_document_id);

                download(&url, &tmp_file).await?;
                compile_markdown(&tmp_file)?;
            } else {
                println!("No document id given.");
            }
        } else {
            println!("No domain of hedgedoc instance given.");
        }
    } else {
        // local source: Use `path` as the markdown file to be compiled
        println!("Local");
    }

    Ok(())
}
