fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/metrics_service.proto")?;
    tonic_build::compile_protos("proto/admin_service.proto")?;
    Ok(())
}
