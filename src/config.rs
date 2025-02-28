use config::Config as LibConfig;
use serde::Deserialize;

use std::{
    env::current_dir,
    ffi::OsString,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::error::PrettyError;

#[derive(Deserialize)]
pub struct Config {
    input_path: Option<PathBuf>,
    output_dir: PathBuf,
    output_file_name: String,
    config_dir: PathBuf,
    config_file: PathBuf,
    title: String,
    font: String,
    logo: Option<String>,
    toc_title: String,
    toc_subtitle: Option<String>,
    show: bool,
    keep: bool,
    domain: Option<String>,
    document_id: Option<String>,
    hedgedoc: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl Config {
    pub fn new() -> Config {
        let default_config = include_str!("../pretty.yaml");
        let standard_template = include_str!("../templates/pandoc/template.tex");

        // get the config directory for pretty
        let config_dir = dirs::config_dir()
            .map(|mut path| {
                path.push("pretty");
                path
            })
            .unwrap_or_default();

        let mut config_file = config_dir.clone();
        config_file.push("pretty.yaml");

        // if it doesn't already exists, create it and put the default pretty.yaml in there
        let _ = fs::create_dir(&config_dir);

        // check for pretty.yaml
        if let Ok(false) = config_file.try_exists() {
            let fs_config_file = File::create(&config_file);
            if let Ok(mut fs_file) = fs_config_file {
                // write default into newly created pretty.yaml
                let _ = fs_file.write_all(default_config.as_bytes());
            }
        }

        // check for template.tex
        let standard_template_path = config_dir.clone().join("template.tex");

        if let Ok(false) = config_file.try_exists() {
            let standard_template_file = File::create(&standard_template_path);
            if let Ok(mut fs_file) = standard_template_file {
                let _ = fs_file.write_all(standard_template.as_bytes());
            }
        }

        Self {
            config_dir,
            config_file,
            input_path: None,
            output_dir: PathBuf::default(),
            output_file_name: String::from("pretty"),
            title: String::from("Pretty Document"),
            font: String::from("Ubuntu"),
            logo: None,
            toc_title: String::from("Table of Contents"),
            toc_subtitle: None,
            show: false,
            keep: false,
            domain: None,
            hedgedoc: false,
            document_id: None,
        }
    }

    pub fn load_config(&mut self) {
        let cwd = current_dir().unwrap_or_default().join("pretty");

        if let Ok(conf) = LibConfig::builder()
            .add_source(config::File::with_name(
                self.config_file.to_str().get_or_insert(""),
            ))
            .add_source(config::File::with_name(cwd.to_str().get_or_insert("")).required(false))
            .build()
        {
            println!("{:#?}", conf);
            *self = conf.try_deserialize::<Config>().unwrap_or_default();
        }
    }

    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    pub fn set_input_path(&mut self, input_path: PathBuf) {
        self.input_path = Some(input_path);
    }

    pub fn get_input_path(&self) -> Option<PathBuf> {
        self.input_path.clone()
    }

    pub fn set_output_path(&mut self, output_path: PathBuf) -> Result<(), PrettyError> {
        // check type of output_path
        if output_path.is_dir() {
            self.output_dir = output_path.to_path_buf();
        } else {
            if let Some(file_name) = output_path.file_stem() {
                if let Some(file_name) = file_name.to_str() {
                    self.output_file_name = String::from(file_name);
                }
            }

            self.output_dir = match output_path.parent() {
                Some(dir) => dir.to_path_buf(),
                None => current_dir()?,
            };
        }

        Ok(())
    }

    pub fn get_output_pdf(&self) -> OsString {
        self.output_dir
            .clone()
            .join(&self.output_file_name)
            .with_extension("pdf")
            .into_os_string()
    }

    pub fn get_output_md(&self) -> OsString {
        self.output_dir
            .clone()
            .join(&self.output_file_name)
            .with_extension("md")
            .into_os_string()
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_toc_title(&self) -> String {
        self.toc_title.clone()
    }

    pub fn get_font(&self) -> String {
        self.font.clone()
    }

    pub fn set_domain(&mut self, domain: &String) {
        self.domain = Some(String::from(domain));
    }

    pub fn get_domain(&self) -> Option<String> {
        self.domain.clone()
    }

    pub fn set_document_id(&mut self, document_id: &String) {
        self.document_id = Some(String::from(document_id));
    }

    pub fn get_document_id(&self) -> Option<String> {
        self.document_id.clone()
    }

    pub fn set_hedgedoc(&mut self, hedgedoc: bool) {
        self.hedgedoc = hedgedoc;
    }

    pub fn is_hedgedoc(&self) -> bool {
        self.hedgedoc
    }

    pub fn set_keep(&mut self, keep: bool) {
        self.keep = keep;
    }

    pub fn should_keep(&self) -> bool {
        self.keep
    }

    pub fn set_show(&mut self, show: bool) {
        self.show = show;
    }

    pub fn should_show(&self) -> bool {
        self.show
    }
}
