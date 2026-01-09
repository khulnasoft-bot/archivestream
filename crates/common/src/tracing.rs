use opentelemetry::{global, trace::{Tracer, TracerProvider}, KeyValue};
use opentelemetry_sdk::{
    trace::{self as sdktrace, Sampler},
    Resource,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize OpenTelemetry tracing with Jaeger backend
pub fn init_tracing(service_name: &str) -> anyhow::Result<()> {
    // Configure OTLP exporter to send to Jaeger
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            sdktrace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Set up tracing subscriber with OpenTelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info,tower_http=debug", service_name).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    Ok(())
}

/// Shutdown tracing and flush remaining spans
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info, instrument};

    #[instrument]
    fn example_traced_function(param: &str) {
        info!("Processing: {}", param);
    }

    #[test]
    fn test_tracing_init() {
        // This would require a running Jaeger instance
        // Skip in CI, run manually for testing
    }
}
