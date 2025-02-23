mod error;
use crate::error::{CompilationError, DownloadError, PrettyError};

use std::{
    env::{current_dir, temp_dir},
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use clap::{value_parser, Parser};
use error::PrettyInvalidArgs;

const DEFAULT_FILE_NAME: &str = "pretty";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CLI {
    /// file to be compiled
    #[arg(value_parser=value_parser!(PathBuf))]
    input_path: Option<std::path::PathBuf>,

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

    // the output file (default: "pretty.pdf")
    #[arg(short = 'o', long = "output", value_parser=value_parser!(PathBuf), default_value="pretty.pdf")]
    output_path: Option<std::path::PathBuf>,
}

fn initialize(config_dir: &PathBuf) -> Result<(), PrettyError> {
    // prepare latex templates
    let template_style = include_str!("../templates/anton.sty");
    let template = include_str!("../templates/template.tex");

    fs::create_dir_all(&config_dir)?;

    let template_style_path = config_dir.clone().join("anton.sty");
    let template_path = config_dir.clone().join("template.tex");

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

fn compile_markdown(
    markdown_file_path: &PathBuf,
    pdf_file_path: &PathBuf,
    config_dir: &PathBuf,
) -> Result<(), CompilationError> {
    // execute pandoc
    if cfg!(target_os = "linux") {
        let path = markdown_file_path
            .to_str()
            .ok_or_else(|| CompilationError::FileNotFound)?;
        let output_path = pdf_file_path
            .to_str()
            .ok_or_else(|| CompilationError::InvalidOutputPath)?;

        let template_path = config_dir.clone().join("template.tex");

        let cmd = format!(
            "pandoc \"{}\" -f markdown -t pdf --template=\"{}\" --pdf-engine=xelatex -o {}",
            path,
            template_path.display(),
            output_path
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

fn process_output_path(output_path: PathBuf) -> Result<(PathBuf, String), PrettyError> {
    let output_dir: PathBuf;
    let output_file_name: String;

    // check type of output_path
    if output_path.is_dir() {
        output_dir = output_path.to_path_buf();
        output_file_name = String::from(DEFAULT_FILE_NAME);
    } else {
        output_file_name = match output_path.file_stem() {
            Some(name) => String::from(name.to_str().unwrap_or_else(|| DEFAULT_FILE_NAME)),
            None => String::from(DEFAULT_FILE_NAME),
        };
        output_dir = match output_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => current_dir()?,
        };
    }

    // check if the output_dir exists
    if !output_dir.exists() {
        return Err(PrettyError::NonExistantPath(output_path.to_path_buf()));
    }

    Ok((output_dir, output_file_name))
}

#[tokio::main]
async fn main() -> Result<(), PrettyError> {
    let args = CLI::parse();

    // config directory: <config_dir>/pretty
    let mut config_dir = dirs::config_dir().ok_or_else(|| PrettyError::ConfigDirNotFound)?;
    config_dir.push("pretty");

    // output path
    let (output_dir, output_file_name) = match args.output_path {
        Some(path) => process_output_path(path),
        None => Err(PrettyError::InvalidInput(PrettyInvalidArgs::OutputPath)),
    }?;
    let pdf_path = output_dir.clone().join(output_file_name.clone() + ".pdf");

    // initialize the config directory
    // we are abusing this to create the physical latex files in the filesystem
    initialize(&config_dir)?;

    // decide on the path to take
    if args.hedgedoc {
        // remote source: Use `domain` and `document_id` to download and compile
        let tmp_dir = temp_dir();
        let tmp_file = tmp_dir.clone().join("pretty.md");

        // check that `domain` and `document_id` are given
        if let Some(some_domain) = args.domain {
            if let Some(some_document_id) = args.document_id {
                let url = format!("{}/{}/download", some_domain, some_document_id);

                download(&url, &tmp_file).await?;
                compile_markdown(&tmp_file, &pdf_path, &config_dir)?;

                if args.keep {
                    // copy tmp_file over to output dir
                    let keep_file = output_dir.clone().join(output_file_name.clone() + ".md");

                    std::fs::copy(&tmp_file, &keep_file)
                        .map_err(|err| PrettyError::Copy(err.to_string()))?;

                    println!("Copied markdown file to {}", keep_file.display());
                }
            } else {
                println!("No document id given.");
            }
        } else {
            println!("No domain of hedgedoc instance given.");
        }
    } else {
        // local source: Use `path` as the markdown file to be compiled
        if let Some(markdown_path) = args.input_path {
            compile_markdown(&markdown_path, &pdf_path, &config_dir)?;
        } else {
            return Err(PrettyError::InvalidInput(PrettyInvalidArgs::InputPath));
        }
    }

    // show output pdf
    if args.show {
        open::that(pdf_path)?;
    }

    Ok(())
}
