use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::web::{self, JsonConfig, PathConfig, QueryConfig};
use actix_web::{post, App, HttpServer, Responder, ResponseError};
use tokio::fs;

use oj::cli;
use oj::config::Config;
use oj::db::{connection, migration};
use oj::error::ApiError;
use oj::judger;
use oj::routes;

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load env from the `.env` file, but it's ok if this file is missing
    dotenvy::dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info"))
        .format_timestamp_micros()
        .init();

    let args = cli::parse_args();

    log::info!(
        "Starting OJ with config file [{}] and flush-data [{}]",
        args.config,
        args.flush_data
    );

    let config_json = fs::read_to_string(args.config).await?;
    let config = match Config::new(&config_json) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };

    let pool = connection::connection_pool();
    migration::initialize_database(args.flush_data, &pool);

    let (adder, waiter) = judger::create_judger(pool.clone(), &config);

    let Config {
        server,
        problem_map,
        language_map,
    } = config;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().add(("Access-Control-Allow-Origin", "*")))
            .app_data(JsonConfig::default().error_handler(ApiError::invalid_argument))
            .app_data(PathConfig::default().error_handler(ApiError::invalid_argument))
            .app_data(QueryConfig::default().error_handler(ApiError::invalid_argument))
            .app_data(web::Data::new(problem_map.clone()))
            .app_data(web::Data::new(language_map.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(adder.clone()))
            .service(routes::jobs::routes())
            .service(routes::users::routes())
            .service(routes::contests::routes())
            .service(routes::problems::routes())
            .service(routes::languages::routes())
            // DO NOT REMOVE: used in automatic testing
            .service(exit)
            .default_service(web::to(|| async {
                ApiError::not_found("Route").error_response()
            }))
    })
    .bind((server.bind_address, server.bind_port))?
    .run()
    .await?;

    waiter.wait().await;

    Ok(())
}
