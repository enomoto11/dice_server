use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::OnceLock;

use futures::TryStreamExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use log::info;
use mongodb::bson::{doc, DateTime as BsonDateTime};
use mongodb::{options::ClientOptions, Client};
use opentelemetry::global::{self, BoxedTracer};
use opentelemetry::trace::{Span, SpanKind, Status, Tracer};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout::SpanExporter;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Debug, Serialize, Deserialize)]
struct TestDoc {
    signage_id: i32,
    date: BsonDateTime,
    budgets: Vec<i32>,
}

async fn roll_dice(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let random_number = rand::rng().random_range(1..=6);
    Ok(Response::new(Full::new(Bytes::from(
        random_number.to_string(),
    ))))
}

async fn mongo_test(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // MongoDBに接続
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("testdb");
    let col = db.collection::<TestDoc>("testcol");
    // クエリ条件
    use mongodb::bson::{doc, DateTime as BsonDateTime};

    let start = BsonDateTime::parse_rfc3339_str("2025-05-19T00:00:00Z").unwrap();
    let end = BsonDateTime::parse_rfc3339_str("2025-05-23T23:59:59Z").unwrap();

    let filter = doc! {
        "date": { "$gte": start, "$lte": end },
        "signage_id": { "$gte": 7300, "$lte": 7400 }
    };
    let mut cursor = col.find(filter, None).await.unwrap();
    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.unwrap() {
        info!("MongoDBから取得: {:?}", doc);
        results.push(doc);
    }
    let body = serde_json::to_string(&results).unwrap();
    Ok(Response::new(Full::new(Bytes::from(body))))
}

async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let tracer = get_tracer();

    let mut span = tracer
        .span_builder(format!("{} {}", req.method(), req.uri().path()))
        .with_kind(SpanKind::Server)
        .start(tracer);

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rolldice") => roll_dice(req).await,
        (&Method::GET, "/mongo-test") => mongo_test(req).await,
        _ => {
            span.set_status(Status::Ok);
            Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap())
        }
    }
}

fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("dice_server"))
}

fn init_tracer_provider() {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr).await?;
    init_tracer_provider();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
