pub use substreams_ethereum_core::scalar;
pub use substreams_ethereum_core::{block_view, pb, rpc, Event, Function, NULL_ADDRESS};
pub use substreams_ethereum_derive::EthabiContract;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use getrandom;

/// Builder struct for generating type-safe bindings Rust code directly in your project from a contract's ABI.
/// This is equivalent to code generated by macro [use_contract].
///
/// # Example
///
/// Running the code below will generate a file called `erc721.rs` containing the
/// bindings inside, which exports an `erc` struct, along with all its events. Put into a
/// `build.rs` file this will generate the bindings during `cargo build`.
///
/// ```no_run
///     use anyhow::{Ok, Result};
///     use substreams_ethereum::Abigen;
///
///     fn main() -> Result<(), anyhow::Error> {
///         Abigen::new("ERC721", "abi/erc721.json")?
///             .generate()?
///             .write_to_file("src/abi/erc721.rs")?;
///
///         Ok(())
///     }
/// ```
pub use substreams_ethereum_abigen::build::Abigen;
pub use substreams_ethereum_abigen::build::AbiExtension;
pub use substreams_ethereum_abigen::build::EventExtension;

/// This macro can be used to import an Ethereum ABI file in JSON format and generate all the
/// required bindings for ABI decoding/encoding in Rust, targetting `substreams` developer
/// experience. You prefer to have the code generated directly, check out [Abigen].
///
/// ```no_run
///     use substreams_ethereum::use_contract;
///
///     use_contract!(erc721, "./examples/abi/erc721.json");
/// ```
///
/// This invocation will generate the following code (signatures only for consiscness):
///
/// ```rust
/// mod erc721 {
///     pub mod events {
///         #[derive(Debug, Clone, PartialEq)]
///         pub struct Transfer {
///             pub from: Vec<u8>,
///             pub to: Vec<u8>,
///             pub token_id: ethabi::Uint,
///         }
///
///         impl Transfer {
///             pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
///                // ...
///                # todo!()
///             }
///
///             pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Transfer, String> {
///                // ...
///                # todo!()
///             }
///         }
///
///         // ... Other events ...
///     }
/// }
/// ```
#[macro_export]
macro_rules! use_contract {
    ($module: ident, $path: expr) => {
        #[allow(dead_code)]
        #[allow(missing_docs)]
        #[allow(unused_imports)]
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub mod $module {
            #[derive(substreams_ethereum::EthabiContract)]
            #[ethabi_contract_options(path = $path)]
            struct _Dummy;
        }
    };
}

/// The `init` macro registers a custom get random function in the system which is required
/// because `ethabi` that we rely on for ABI decoding/encoding primitives use it somewhere in
/// its transitive set of dependencies and causes problem in `wasm32-unknown-unknown` target.
///
/// This macro must be invoked in the root crate so you must have the `substreams_ethereum::init!()`
/// call in your `lib.rs` of your Substreams.
///
/// In addition, you need to have `getrandom = { version = "0.2", features = ["custom"] }` dependency
/// in your Substreams `Cargo.toml` file:
///
/// ```toml
/// # Required so that ethabi > ethereum-types build correctly under wasm32-unknown-unknown
/// [target.wasm32-unknown-unknown.dependencies]
/// getrandom = { version = "0.2", features = ["custom"] }
///```
#[macro_export]
macro_rules! init {
    () => {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        $crate::getrandom::register_custom_getrandom!($crate::getrandom_unavailable);
    };
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
const GETRANDOM_UNVAILABLE_IN_SUBSTREAMS: u32 = getrandom::Error::CUSTOM_START + 5545;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn getrandom_unavailable(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let code = std::num::NonZeroU32::new(GETRANDOM_UNVAILABLE_IN_SUBSTREAMS).unwrap();
    Err(getrandom::Error::from(code))
}
