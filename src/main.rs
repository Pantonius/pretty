mod config;
mod error;
use crate::error::{CompilationError, DownloadError, PrettyError};

use std::{env::temp_dir, fs::File, io::Write, path::PathBuf, process::Command};

use clap::{value_parser, Parser};
use config::Config;

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

    /// the domain of the source instance (e.g. HedgeDoc)
    #[arg(short = 'D', long = "domain")]
    domain: Option<String>,

    /// defines the ID of the hedgedoc document on the configured instance
    #[arg(short = 'I', long = "document_id")]
    document_id: Option<String>,

    /// whether to keep the downloaded markdown (in case a source has been selected)
    #[arg(short = 'k', long = "keep")]
    keep: bool,

    /// the output file or directory
    #[arg(short = 'o', long = "output", value_parser=value_parser!(PathBuf), default_value="pretty.pdf")]
    output_path: Option<std::path::PathBuf>,
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

fn compile_markdown(config: &Config) -> Result<(), CompilationError> {
    // execute pandoc
    if cfg!(target_os = "linux") {
        let path = config
            .get_input_path()
            .ok_or_else(|| CompilationError::FileNotFound)?;
        let output_path = config.get_output_pdf();
        let output_path = output_path
            .to_str()
            .ok_or_else(|| CompilationError::InvalidOutputPath)?;

        let template_path = config.get_config_dir().join("template.tex");

        let cmd = format!(
            "pandoc \"{}\" -f markdown -t pdf --template=\"{}\" -V mainfont=\"{}\" -V title:\"{}\" -V toc-title:\"{}\" --pdf-engine=xelatex -o {}",
            path.display(),
            template_path.display(),
            config.get_font(),
            config.get_title(),
            config.get_toc_title(),
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

#[tokio::main]
async fn main() -> Result<(), PrettyError> {
    let args: CLI = CLI::parse();
    let mut config: Config = Config::default();
    config.load_config();

    // input path
    if let Some(path) = args.input_path {
        config.set_input_path(path);
    }

    // output path
    if let Some(path) = args.output_path {
        config.set_output_path(path)?;
    }

    // headgedoc
    if let Some(ref domain) = args.domain {
        config.set_domain(domain);
    }
    if let Some(ref document_id) = args.document_id {
        config.set_document_id(document_id);
    }

    // flags
    config.set_hedgedoc(args.hedgedoc);
    config.set_keep(args.keep);
    config.set_show(args.show);

    // decide on the path to take
    if config.is_hedgedoc() {
        // remote source: Use `domain` and `document_id` to download and compile
        let tmp_dir = temp_dir();
        let tmp_file = tmp_dir.clone().join("pretty.md");

        // check that `domain` and `document_id` are given
        if let Some(some_domain) = config.get_domain() {
            if let Some(some_document_id) = config.get_document_id() {
                let url = format!("{}/{}/download", some_domain, some_document_id);

                download(&url, &tmp_file).await?;

                config.set_input_path(tmp_file.clone());
                compile_markdown(&config)?;

                if config.should_keep() {
                    // copy tmp_file over to output dir
                    let keep_file = config.get_output_md();

                    std::fs::copy(&tmp_file, &keep_file)
                        .map_err(|err| PrettyError::Copy(err.to_string()))?;

                    println!("Copied markdown file to {:?}", keep_file);
                }
            } else {
                println!("No document id given.");
            }
        } else {
            println!("No domain of hedgedoc instance given.");
        }
    } else {
        // local source: Use `path` as the markdown file to be compiled
        compile_markdown(&config)?;
    }

    // show output pdf
    if config.should_show() {
        open::that(&config.get_output_pdf())?;
    }

    Ok(())
}
