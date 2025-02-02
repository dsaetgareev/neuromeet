use std::net::ToSocketAddrs;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use tracing::{error, info, level_filters::LevelFilter};

use sec_api::webtransport::{self, Certs};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use url::Url;

async fn health_responder() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[actix_rt::main]
async fn main() {
    dotenv().ok();

    let _  = init_logs("webtransport_server");

    let health_listen = std::env::var("HEALTH_LISTEN_URL")
        .expect("expected HEALTH_LISTEN_URL to be set")
        .to_socket_addrs()
        .expect("expected HEALTH_LISTEN_URL to be a valid socket address")
        .next()
        .expect("expected HEALTH_LISTEN_URL to be a valid socket address");

    let opt = webtransport::WebTransportOpt {
        listen: std::env::var("LISTEN_URL")
            .expect("expected LISTEN_URL to be set")
            .to_socket_addrs()
            .expect("expected LISTEN_URL to be a valid socket address")
            .next()
            .expect("expected LISTEN_URL to be a valid socket address"),
        certs: Certs {
            key: std::env::var("KEY_PATH")
                .expect("expected KEY_PATH to be set")
                .into(),
            cert: std::env::var("CERT_PATH")
                .expect("expected CERT_PATH to be set")
                .into(),
        },
    };

    let listen = opt.listen;
    actix_rt::spawn(async move {
        info!("Starting http server: {:?}", listen);
        let server =
            HttpServer::new(|| App::new().route("/healthz", web::get().to(health_responder)))
                .bind(&health_listen)
                .unwrap();
        if let Err(e) = server.run().await {
            error!("http server error: {}", e);
        }
    });

    let _ = actix_rt::spawn(async move {
        webtransport::start(opt).await.unwrap();
    })
    .await;
}

fn init_logs(app_name: &str) -> Result<(), ()> {
    let loki_url_str = std::env::var("LOKI_URL").expect("LOKI_URL env var must be defined");
    let loki_url = Url::parse(&loki_url_str).unwrap();

    let (layer, task) = tracing_loki::builder()
       .label("application", app_name)
       .unwrap()
       .extra_field("pid", format!("{}", std::process::id()))
       .unwrap()
       .build_url(loki_url.clone())
       .unwrap();

   let filter = EnvFilter::builder()
       .with_default_directive(LevelFilter::INFO.into())
       .parse("")
       .unwrap();

   tracing_subscriber::registry()
       .with(filter)
       .with(layer)
       .with(tracing_subscriber::fmt::Layer::new())
       .init();

   tokio::spawn(task);
   Ok(())
}