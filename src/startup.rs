use crate::routes::{health_check, subscribe};
use crate::configuration::{self, DatabaseSettings, Settings};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::{PgPool,postgres::PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;

// A new type to hold the newly built server and its port
pub struct Application {
    port: u16,
    server: Server,
}

impl Application{
    // We have converted the build function into a constructor for
    // `Application`.

    pub async fn build(configuration: Settings) -> Result<Self,std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connec to Postgres.");

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
        );

        let address = format!("{}:{}",configuration.application.host,configuration.application.port);

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener,connection_pool,email_client)?;

        // We "save" the bound part in one of the `Application's` fields
        Ok(Self {
            port,
            server
        })

    }

    pub fn port(&self)-> u16{
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self)-> Result<(),std::io::Error>{
        self.server.await
    }
}


pub async fn get_connection_pool(
    configuration: &DatabaseSettings
) -> Result<PgPool,sqlx::Error>{
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.with_db())
        .await
}

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
        .run();
    Ok(server)
}
