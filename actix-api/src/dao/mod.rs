use std::env;

use deadpool_postgres::{Pool, tokio_postgres::NoTls, Config, Manager, ManagerConfig, RecyclingMethod, Runtime};


pub fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap()
}

fn get_db_config() -> deadpool_postgres::Config {
    let mut config = deadpool_postgres::Config::new();
    config.user = Some(env::var("DB_USER").unwrap());
    config.password = Some(env::var("DB_PASSWORD").unwrap());
    config.dbname = Some(env::var("DB_NAME").unwrap());
    config.host = Some(env::var("DB_HOSTNAME").unwrap());

    config.manager =
       Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    config
}

pub fn create_pool() -> Result<Pool, String> {
    let pool = get_db_config()
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|err| err.to_string()).unwrap();
    Ok(pool)
}
