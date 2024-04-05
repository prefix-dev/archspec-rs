fn main() -> Result<(), archspec::cpu::UnsupportedMicroarchitecture> {
    let architecture = archspec::cpu::host()?;

    println!("Current CPU architecture:");
    println!("  Name: {}", architecture.name());
    println!("  Vendor: {}", architecture.vendor());
    println!("  Generation: {}", architecture.generation());
    println!("  Family Name: {}", architecture.family().name());
    println!("  Features: {:?}", architecture.all_features());

    Ok(())
}
