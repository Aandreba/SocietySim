use spirv_builder::{MetadataPrintout, SpirvBuilder, Capability};

const CRATE: &str = "gpu";
const TARGET: &str = "spirv-unknown-vulkan1.1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new(CRATE, TARGET)
        .print_metadata(MetadataPrintout::Full)
        .capability(Capability::Int8)
        .capability(Capability::Int16)
        //.capability(Capability::VariablePointersStorageBuffer)
        .build()?;
    Ok(())
}