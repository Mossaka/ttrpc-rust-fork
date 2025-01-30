use opentelemetry::{
    propagation::{Extractor, Injector},
    Context as OtelContext,
};
use std::collections::HashMap;

pub struct MetadataInjector<'a>(pub &'a mut HashMap<String, Vec<String>>);
pub struct MetadataExtractor<'a>(pub &'a HashMap<String, Vec<String>>);

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

pub fn inject_context(otel_ctx: &OtelContext, metadata: &mut HashMap<String, Vec<String>>) {
    let mut injector = MetadataInjector(metadata);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(otel_ctx, &mut injector);
    });
}

pub fn extract_context(metadata: &HashMap<String, Vec<String>>) -> OtelContext {
    let extractor = MetadataExtractor(metadata);
    opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}
