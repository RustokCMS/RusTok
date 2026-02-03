use rustok_iggy::config::{IggyConfig, IggyMode, SerializationFormat};
use rustok_iggy::transport::IggyTransport;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
#[ignore = "Integration test requires running iggy backend"]
async fn test_iggy_transport_lifecycle() -> TestResult<()> {
    let config = IggyConfig {
        mode: IggyMode::Embedded,
        serialization: SerializationFormat::Json,
        ..IggyConfig::default()
    };

    let transport = IggyTransport::new(config).await?;
    transport.shutdown().await?;

    Ok(())
}
