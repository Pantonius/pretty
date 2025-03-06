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

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    input_path: Option<PathBuf>,

    #[serde(default)]
    output_dir: PathBuf,

    #[serde(default)]
    output_file_name: String,

    #[serde(default)]
    config_dir: PathBuf,

    #[serde(default)]
    config_file: PathBuf,

    #[serde(default)]
    title: String,

    #[serde(default)]
    font: String,

    author: Option<String>,
    logo: Option<String>,

    #[serde(default)]
    toc_title: String,

    toc_subtitle: Option<String>,

    #[serde(default)]
    show: bool,

    #[serde(default)]
    keep: bool,

    domain: Option<String>,
    document_id: Option<String>,

    #[serde(default)]
    hedgedoc: bool,
}

trait Mergable {
    fn merge(self: &Self, other: &Self) -> Self;
}

impl Mergable for String {
    fn merge(self: &Self, other: &Self) -> Self {
        if self.is_empty() {
            other.to_string()
        } else {
            self.to_string()
        }
    }
}

impl Mergable for PathBuf {
    fn merge(self: &Self, other: &Self) -> Self {
        if *self == PathBuf::new() {
            other.to_path_buf()
        } else {
            self.to_path_buf()
        }
    }
}

impl Mergable for bool {
    fn merge(self: &Self, other: &Self) -> Self {
        *self || *other
    }
}

impl<T: Clone> Mergable for Option<T> {
    fn merge(self: &Self, other: &Self) -> Self {
        self.clone().or_else(|| other.clone())
    }
}

impl Mergable for Config {
    fn merge(self: &Self, other: &Self) -> Self {
        Self {
            input_path: (&self.input_path).merge(&other.input_path),
            output_dir: (&self.output_dir).merge(&other.output_dir),
            output_file_name: (&self.output_file_name).merge(&other.output_file_name),
            config_file: (&self.config_file).merge(&other.config_file),
            config_dir: (&self.config_dir).merge(&other.config_dir),
            title: (&self.title).merge(&other.title),
            font: (&self.font).merge(&other.font),
            author: (&self.author).merge(&other.author),
            logo: (&self.logo).merge(&other.logo),
            toc_title: (&self.toc_title).merge(&other.toc_title),
            toc_subtitle: (&self.toc_subtitle).merge(&other.toc_subtitle),
            show: (&self.show).merge(&other.show),
            keep: (&self.keep).merge(&other.keep),
            domain: (&self.domain).merge(&other.domain),
            document_id: (&self.document_id).merge(&other.document_id),
            hedgedoc: (&self.hedgedoc).merge(&other.hedgedoc),
        }
    }
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

        if let Ok(false) = standard_template_path.try_exists() {
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
            author: None,
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

    /// Loads the config from a pretty.yaml file in the current working directory or the
    /// pretty subdirectory of the config folder of the system
    pub fn load_config(&mut self) {
        let cwd = current_dir().unwrap_or_default();

        if let Ok(conf) = LibConfig::builder()
            .add_source(config::File::with_name(
                self.config_file.to_str().get_or_insert(""),
            ))
            .add_source(
                config::File::with_name(cwd.join("pretty").to_str().get_or_insert(""))
                    .required(false),
            )
            .build()
        {
            let pretty_conf = conf.try_deserialize::<Config>().unwrap_or_default();

            *self = self.merge(&pretty_conf);
        }
    }

    /// Returns the configured config directory
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    /// Sets the input path (should be a markdown file)
    pub fn set_input_path(&mut self, input_path: PathBuf) {
        self.input_path = Some(input_path);
    }

    /// Returns the configured input path
    pub fn get_input_path(&self) -> Option<PathBuf> {
        self.input_path.clone()
    }

    /// Sets the ouput path (may be a directory or a file)
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

    /// Returns the ouput path for the pdf file
    pub fn get_output_pdf(&self) -> OsString {
        self.output_dir
            .clone()
            .join(&self.output_file_name)
            .with_extension("pdf")
            .into_os_string()
    }

    /// Returns the ouput path for the md file
    pub fn get_output_md(&self) -> OsString {
        self.output_dir
            .clone()
            .join(&self.output_file_name)
            .with_extension("md")
            .into_os_string()
    }

    /// Returns the configured document title
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    /// Returns the configured author
    pub fn get_author(&self) -> String {
        self.author.clone().unwrap_or_default()
    }

    /// Returns the configured title for the table of contents
    pub fn get_toc_title(&self) -> String {
        self.toc_title.clone()
    }

    /// Returns the configured font for the pdf document
    pub fn get_font(&self) -> String {
        self.font.clone()
    }

    /// Sets the domain of the remote source (should be a HedgeDoc instance)
    pub fn set_domain(&mut self, domain: &String) {
        self.domain = Some(String::from(domain));
    }

    /// Returns the configured domain for a remote source
    pub fn get_domain(&self) -> Option<String> {
        self.domain.clone()
    }

    /// Sets the document id that should be downloaded
    pub fn set_document_id(&mut self, document_id: &String) {
        self.document_id = Some(String::from(document_id));
    }

    /// Returns the configured document id for the document that should be downloaded from a
    /// remote source
    pub fn get_document_id(&self) -> Option<String> {
        self.document_id.clone()
    }

    /// Sets whether or not the markdown content should be downloaded from a remote hedgedoc
    /// instance
    pub fn set_hedgedoc(&mut self, hedgedoc: bool) {
        self.hedgedoc = hedgedoc;
    }

    /// Returns whether or not the markdown content should be downloaded from a remote hedgedoc
    /// instance
    pub fn is_hedgedoc(&self) -> bool {
        self.hedgedoc
    }

    /// Sets whether or not to keep the downloaded markdown file from a remote source
    pub fn set_keep(&mut self, keep: bool) {
        self.keep = keep;
    }

    /// Returns whether or not the downloaded markdown file should be kept
    pub fn should_keep(&self) -> bool {
        self.keep
    }

    /// Sets whether or not the compiled pdf document should be opened after compilation
    pub fn set_show(&mut self, show: bool) {
        self.show = show;
    }

    /// Returns whether or not the compiled pdf document should be opened after compilation
    pub fn should_show(&self) -> bool {
        self.show
    }
}
