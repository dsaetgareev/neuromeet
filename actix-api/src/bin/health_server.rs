use deadpool_postgres::Pool;
use sec_api::dao::create_pool;
use tokio_schedule::Job;
use tracing::{info, level_filters::LevelFilter};
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use url::Url;



#[tokio::main]
async fn main() -> Result<(), ()> {
    dotenv().ok();
    let _ = init_logs("health_server");

    info!("start health_server");

    let pool = create_pool().unwrap();
    let every_30_seconds = tokio_schedule::every(3).seconds() // by default chrono::Local timezone
        .perform(move || {
        let value = pool.clone();
        async move { 
            println!("Every minute at 00'th and 30'th second");
            check_conferences(&value).await.unwrap();
        }
        });

    let handle = tokio::spawn(every_30_seconds);
    handle.await.unwrap();

    Ok(())
}

async fn check_conferences(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT conference, user_id, 
                    FROM conferences 
                    WHERE status = 'active' 
                    AND ((EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) * 1000)::BIGINT - end_time) > 300000;", 
        &[]
    ).await?;
    
    for row in rows {
        let conference: String = row.get(0);
        let user_id: String = row.get(1);
        let td: String = row.get(2);
        info!("Found expired conference: conference={}, user_id={}", conference, td);
        
        // client.execute(
        //     "UPDATE conferences SET status = 'completed' WHERE id = $1", 
        //     &[&id]
        // ).await?;
    }
    Ok(())
}


fn init_logs(app_name: &str) -> Result<(), ()> {
    let loki_url_str = std::env::var("LOKI_URL").expect("LOKI_URL env var must be defined");
    let loki_url = Url::parse(&loki_url_str).unwrap();

    let (layer, task) = tracing_loki::builder()
       .label("application", app_name)
       .unwrap()
       .extra_field("pid", format!("{}", std::process::id()))
       .unwrap()
       .extra_field("thread", format!("{:?}", std::thread::current().id()))
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