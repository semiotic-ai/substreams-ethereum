use std::path::{Path, PathBuf};

use crate::{generate_abi_code, normalize_path};
use anyhow::Context;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Abigen {
    /// The path where to fin the source of the ABI JSON for the contract whose bindings
    /// are being generated.
    contract_name: &'static str,
    abi_path: PathBuf,
    extension: Option<AbiExtension>,
}

#[derive(Debug, Clone)]
pub struct AbiExtension {
    event_extension: EventExtension,
}

#[derive(Debug, Clone)]
pub struct EventExtension {
    extended_event_derive: Vec<String>,
    extended_event_import: Vec<String>,
}

impl AbiExtension {
    pub fn new(event_extension: EventExtension) -> Self {
        Self { event_extension }
    }

    pub fn event_extension(&self) -> EventExtension {
        self.event_extension.clone()
    }
}

impl EventExtension {
    pub fn new() -> Self {
        Self {
            extended_event_derive: vec![],
            extended_event_import: vec![],
        }
    }

    pub fn extended_event_import(&self) -> &Vec<String> {
        &self.extended_event_import
    }

    pub fn extended_event_derive(&self) -> &Vec<String> {
        &self.extended_event_derive
    }

    pub fn extend_event_derive(&mut self, derive: &str) {
        self.extended_event_derive.push(derive.to_string());
    }

    pub fn extend_event_import(&mut self, import: &str) {
        self.extended_event_import.push(import.to_string());
    }
}

impl Abigen {
    /// Creates a new builder for the given contract name and where the ABI JSON file can be found
    /// at `path`, which is relative to the your crate's root directory (where `Cargo.toml` file is located).
    pub fn new<S: AsRef<str>>(contract_name: &'static str, path: S) -> Result<Self, anyhow::Error> {
        let path = normalize_path(path.as_ref()).context("normalize path")?;

        Ok(Self {
            contract_name,
            abi_path: path,
            extension: None,
        })
    }

    pub fn add_extension(mut self, extension: AbiExtension) -> Self {
        self.extension = Some(extension);
        self
    }

    pub fn generate(&self) -> Result<GeneratedBindings, anyhow::Error> {
        let item = generate_abi_code(self.abi_path.to_string_lossy(), self.contract_name, self.extension.clone())
            .context("generating abi code")?;

        // FIXME: We wrap into a fake module because `syn::parse2(file)` doesn't like it when there is
        // no wrapping statement. Below that we remove the first and last line of the generated code
        // which fixes the problem.
        //

        let file = syn::parse_file(&item.to_string()).context("parsing generated code")?;

        let code = prettyplease::unparse(&file);

        Ok(GeneratedBindings {
            code,
        })
    }
}

pub struct GeneratedBindings {
    code: String,
}

impl GeneratedBindings {
    pub fn write_to_file<P: AsRef<Path>>(&self, p: P) -> Result<(), anyhow::Error> {
        let path = normalize_path(p.as_ref()).context("normalize path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directories for {}", parent.to_string_lossy()))?
        }

        std::fs::write(path, &self.code)
            .with_context(|| format!("writing file {}", p.as_ref().to_string_lossy()))
    }
}
