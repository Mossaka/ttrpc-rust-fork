use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use std::sync::Arc;
use std::thread;
use ttrpc::error::Result;
use ttrpc::Server;

mod protocols;
use protocols::sync::hello::{HelloRequest, HelloResponse};
use protocols::sync::hello_ttrpc;
mod utils;

struct HelloService;

impl hello_ttrpc::HelloService for HelloService {
    fn say_hello(&self, _ctx: &::ttrpc::TtrpcContext, req: HelloRequest) -> Result<HelloResponse> {
        let mut response = HelloResponse::new();
        response.reply = format!("Hello, {}!", req.greeting);
        Ok(response)
    }
}

fn main() {
    simple_logging::log_to_stderr(log::LevelFilter::Trace);
    global::set_text_map_propagator(TraceContextPropagator::new());
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    global::set_tracer_provider(provider.clone());

    let hello_service = hello_ttrpc::create_hello_service(Arc::new(HelloService {}));

    utils::remove_if_sock_exist(utils::SOCK_ADDR).unwrap();
    let mut server = Server::new()
        .bind(utils::SOCK_ADDR)
        .unwrap()
        .register_service(hello_service);

    server.start().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })
        .expect("Error setting Ctrl-C handler");
        println!("Server is running on {}", utils::SOCK_ADDR);
    });

    rx.recv().unwrap();

    provider
        .shutdown()
        .expect("TracerProvider should shutdown successfully");
}
