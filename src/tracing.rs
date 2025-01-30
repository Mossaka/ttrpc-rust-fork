/// Tracing functionality using OpenTelemetry.
///
/// Inspired by https://github.com/containerd/otelttrpc
///
/// This module provides context propogation utilities for injecting and extracting tracing
/// context into/from `context::metadata`, and starting client and server traces.
///
/// # Usage
///
/// ```rust
/// use opentelemetry::Context as OtelContext;
/// use crate::tracing::{start_client_trace, start_server_trace};
///
/// let (client_ctx, client_metadata) = start_client_trace("client_trace", "service_name", "method_name");
///
/// let (server_ctx, server_metadata) = start_server_trace("server_trace", "service_name", "method_name", &client_metadata);
/// ```
use opentelemetry::{
    global::tracer,
    propagation::{Extractor, Injector},
    trace::{TraceContextExt, Tracer as _},
    Context as OtelContext, KeyValue,
};
use std::collections::HashMap;

use crate::context::{from_pb, to_pb};

/// start_client_trace starts a new trace with the given name, service, and method for the client.
/// It returns the new context and metadata to be sent with the request.
/// The metadata could be sent with the request to the server.
/// The context could be used to get the span.
pub fn start_client_trace(
    name: &str,
    service: &str,
    method: &str,
) -> (OtelContext, Vec<crate::proto::KeyValue>) {
    let tracer = tracer("ttrpc");
    let cx = OtelContext::current_with_span(tracer.start(name.to_string()));

    cx.span().set_attributes([
        KeyValue::new("rpc.system", "ttrpc"),
        KeyValue::new("rpc.service", service.to_string()),
        KeyValue::new("rpc.method", method.to_string()),
    ]);

    let _guard = cx.clone().attach();

    let mut metadata = HashMap::new();
    inject_context(&cx, &mut metadata);

    (cx, to_pb(metadata))
}

/// start_server_trace starts a new trace with the given name, service, and method for the server.
/// It returns the new context and metadata to be sent with the response.
/// The metadata could be sent with the response to the client.
/// The context could be used to get the span.
pub fn start_server_trace(
    name: &str,
    service: &str,
    method: &str,
    metadata: &Vec<crate::proto::KeyValue>,
) -> (OtelContext, HashMap<String, Vec<String>>) {
    let parent_cx = extract_context(&from_pb(metadata));
    let tracer = tracer("ttrpc");
    let cx =
        OtelContext::current_with_span(tracer.start_with_context(name.to_string(), &parent_cx));

    cx.span().set_attributes([
        KeyValue::new("rpc.system", "ttrpc"),
        KeyValue::new("rpc.service", service.to_string()),
        KeyValue::new("rpc.method", method.to_string()),
    ]);

    let _guard = cx.clone().attach();

    let mut extracted_metadata = HashMap::new();
    inject_context(&cx, &mut extracted_metadata);

    (cx, extracted_metadata)
}

struct MetadataInjector<'a>(pub &'a mut HashMap<String, Vec<String>>);
struct MetadataExtractor<'a>(pub &'a HashMap<String, Vec<String>>);

impl Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), vec![value]);
    }
}

impl Extractor for MetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.first()).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

fn inject_context(otel_ctx: &OtelContext, metadata: &mut HashMap<String, Vec<String>>) {
    let mut injector = MetadataInjector(metadata);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(otel_ctx, &mut injector);
    });
}

fn extract_context(metadata: &HashMap<String, Vec<String>>) -> OtelContext {
    let extractor = MetadataExtractor(metadata);
    opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}
