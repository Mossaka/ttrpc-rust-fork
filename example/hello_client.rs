use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::time::Duration;
use ttrpc::Client;

mod protocols;
mod utils;

use protocols::sync::hello::HelloRequest;
use protocols::sync::hello_ttrpc;

fn call_say_hello(client: &hello_ttrpc::HelloServiceClient) -> ttrpc::error::Result<()> {
    let mut request = HelloRequest::new();
    request.greeting = "World".to_string();
    
    let response = client.say_hello(&request)?;
    println!("Response from server: {}", response.reply);
    Ok(())
}

fn main() {
    simple_logging::log_to_stderr(log::LevelFilter::Trace);
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    global::set_tracer_provider(provider.clone());
    
    let client = Client::connect(utils::SOCK_ADDR).unwrap();
    let hello_client = hello_ttrpc::HelloServiceClient::new(client);
    
    for _ in 0..10 {
        if let Err(e) = call_say_hello(&hello_client) {
            eprintln!("Error calling say_hello: {:?}", e);
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    
    std::thread::sleep(Duration::from_millis(10));
    provider.shutdown().expect("TracerProvider should shutdown successfully");
} 