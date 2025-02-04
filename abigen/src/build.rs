use std::path::{Path, PathBuf};
use std::str;

use crate::{generate_abi_code, generate_abi_code_from_bytes, normalize_path};
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Abigen<'a> {
    /// The path where to find the source of the ABI JSON for the contract whose bindings
    /// are being generated.
    abi_path: PathBuf,
    /// The bytes of the ABI for the contract whose bindings are being generated.
    bytes: Option<&'a [u8]>,
    
    /// The name of the contract whose bindings are being generated.
    contract_name: String,

    /// The hex encoded 20 byte address of the contract whose bindings are being generated.
    /// If this is not None, the generated code filter events by this contract address.
    contract_address: Option<String>,

    /// The extension of the abi code.
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
    extended_event_attribute: Vec<String>,
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
            extended_event_attribute: vec![],
        }
    }

    pub fn extended_event_import(&self) -> &Vec<String> {
        &self.extended_event_import
    }

    pub fn extended_event_derive(&self) -> &Vec<String> {
        &self.extended_event_derive
    }

    pub fn extended_event_attribute(&self) -> &Vec<String> {
        &self.extended_event_attribute
    }

    pub fn extend_event_derive(&mut self, derive: &str) {
        self.extended_event_derive.push(derive.to_string());
    }

    pub fn extend_event_import(&mut self, import: &str) {
        self.extended_event_import.push(import.to_string());
    }

    pub fn extend_event_attribute(&mut self, attribute: &str) {
        self.extended_event_attribute.push(attribute.to_string());
    }

}

impl<'a> Abigen<'a> {
    /// Creates a new builder for the given contract name and where the ABI JSON file can be found
    /// at `path`, which is relative to the your crate's root directory (where `Cargo.toml` file is located).
    pub fn new<S: AsRef<str>>(contract_name: S,contract_address:Option<String>, path: S) -> Result<Self, anyhow::Error> {
        let path = normalize_path(path.as_ref()).context("normalize path")?;

        Ok( Self {
            contract_name: contract_name.as_ref().to_string(),
            contract_address: contract_address,
            abi_path: path,
            bytes: None,
            extension: None,
        })
    }

    pub fn add_extension(mut self, extension: AbiExtension) -> Self {
        self.extension = Some(extension);
        self
    }

    /// Creates a new builder for the given contract name and where the ABI bytes can be found
    /// at 'abi_bytes'.
    pub fn from_bytes<S: AsRef<str>>(
        _contract_name: S,
        _contract_address:Option<String>,
        abi_bytes: &'a [u8],
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            abi_path: "".parse()?,
            contract_name: _contract_name.as_ref().to_string(),
            contract_address: _contract_address,
            bytes: Some(abi_bytes),
            extension: None,
        })
    }

    pub fn generate(&self) -> Result<GeneratedBindings, anyhow::Error> {
        let item = match &self.bytes {
            None => {
                generate_abi_code(
                    self.abi_path.to_string_lossy(),
                    self.contract_name.clone(),
                    self.contract_address.clone(),
                     self.extension.clone()
                    ).context("generating abi code")?
            }
            Some(bytes) => {
                generate_abi_code_from_bytes(
                    bytes,
                    self.contract_name.clone(), 
                    self.contract_address.clone(),
                    self.extension.clone()
                ).context("generating abi code")?
            }
        };

        // FIXME: We wrap into a fake module because `syn::parse2(file)` doesn't like it when there is
        // no wrapping statement. Below that we remove the first and last line of the generated code
        // which fixes the problem.
        //

        let file = syn::parse_file(&item.to_string()).context("parsing generated code")?;

        let code = prettyplease::unparse(&file);

        Ok(GeneratedBindings { code })
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
