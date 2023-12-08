use substreams_ethereum::{Abigen, AbiExtension, EventExtension};

fn main() -> Result<(), anyhow::Error> {
    let abis = vec!["tests"];

    for abi in abis {
        // All `path` arguments is relative to crate's Cargo.toml directory, in this example, it's 'abigen'
        let in_path = format!("abi/{}.json", abi);
        let out_path = format!("src/abi/{}.rs", abi);

        let abigen = Abigen::new(abi, None, &in_path)?;
        let mut event_extension = EventExtension::new();
        let extension = AbiExtension::new(event_extension);
        abigen.add_extension(extension).generate()?.write_to_file(&out_path)?;
    }

    Ok(())
}
