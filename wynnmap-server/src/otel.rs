use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    metrics::SdkMeterProvider,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::{
    SCHEMA_URL, attribute::DEPLOYMENT_ENVIRONMENT_NAME, resource::SERVICE_VERSION,
};
use tracing::Level;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::OtelConfig;

fn resource(conf: &OtelConfig) -> Resource {
    Resource::builder()
        .with_service_name(env!("CARGO_PKG_NAME"))
        .with_schema_url(
            [
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, conf.env_name.clone()),
            ],
            SCHEMA_URL,
        )
        .build()
}

pub(crate) fn init(conf: &OtelConfig) {
    let trace_provider = init_trace_provider(conf);
    let metrics_provider = init_metrics_provider(conf);

    global::set_meter_provider(metrics_provider);

    let tracer = trace_provider.tracer("wynnmap");

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            Level::INFO,
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();
}

fn init_trace_provider(conf: &OtelConfig) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&*conf.endpoint)
        .build()
        .expect("Failed to initialize otel exporter");

    SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource(conf))
        .with_batch_exporter(exporter)
        .build()
}

fn init_metrics_provider(conf: &OtelConfig) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&*conf.endpoint)
        .build()
        .expect("Failed to initialize otel metrics");

    SdkMeterProvider::builder()
        .with_resource(resource(conf))
        .with_periodic_exporter(exporter)
        .build()
}
