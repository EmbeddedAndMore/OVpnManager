use std::{collections::HashMap, env, path::PathBuf};
use diesel::{deserialize::Queryable, insert_into, Selectable, Insertable};
use log::info;
use russh::{client::Session, *};

use actix_utils::future::{err, ready, Ready};
use actix_web::{
    dev::{self, ServiceResponse},
    error,
    http::{header::ContentType, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers, Logger},
    web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_files as fs;
use actix_web_lab::respond::Html;
use minijinja::path_loader;
use minijinja_autoreload::AutoReloader;
use r2d2;

use serde::{Deserialize, Serialize};
use diesel::RunQueryDsl; 

mod ssh_client;
mod schema;
mod db;

const MAX_SIZE: usize = 262_144; // max payload size is 256k


#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(primary_key(id))]
#[diesel(table_name=schema::hosts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct Host{
    id:i32,
    name:String,
    ip_address: String,
    username: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[diesel(table_name=schema::hosts)]
struct HostCreate {
    name: String,
    ip_address: String,
    username:  String,
    password: String,
}





struct VpnServer{
    id: Option<i32>,
    host_id:i32,
    name: String,
    port: i16,
    subnet: String
}


struct Users{
    id:i32,
    vpn_id:i32,
    username: String,
    is_plant:bool,
    ca_key: String,
    password: String
}



struct MiniJinjaRenderer {
    tmpl_env: web::Data<minijinja_autoreload::AutoReloader>,
}


impl MiniJinjaRenderer {
    fn render(
        &self,
        tmpl: &str,
        ctx: impl Into<minijinja::value::Value>,
    ) -> actix_web::Result<Html> {
        self.tmpl_env
            .acquire_env()
            .map_err(|_| error::ErrorInternalServerError("could not acquire template env"))?
            .get_template(tmpl)
            .map_err(|_| error::ErrorInternalServerError("could not find template"))?
            .render(ctx.into())
            .map(Html)
            .map_err(|err| {
                log::error!("{err}");
                error::ErrorInternalServerError("template error")
            })
    }
}

impl FromRequest for MiniJinjaRenderer {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _pl: &mut dev::Payload) -> Self::Future {
        let tmpl_env = <web::Data<minijinja_autoreload::AutoReloader>>::extract(req)
            .into_inner()
            .unwrap();

        ready(Ok(Self { tmpl_env }))
    }
}


async fn hosts(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    
    tmpl_env.render(
        "hosts.html",
        minijinja::context! {
            text => "Welcome!",
        },
    )
}
async fn add_host(
    pool: web::Data<db::Pool>,
    mut payload: web::Json<HostCreate>,
) -> actix_web::Result<impl Responder> {

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    diesel::insert_into(schema::hosts::table).values(&payload.into_inner()).execute( &mut conn).expect("Error creating new todo");
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::json()).body("{\"response\":\"successful\"}"))
}

async fn test_host_connection(
    tmpl_env: MiniJinjaRenderer,
    mut payload: web::Json<HostCreate>,
) -> actix_web::Result<impl Responder> {
    // payload is a stream of Bytes objects
    println!("{:?}", payload);

    let ssh = ssh_client::Session::connect(
        "root".to_string(),
        "aPRkepaRX7fbLTjxvnxx".to_string(),
        ("49.13.10.131", 22),
    )
    .await;

    match  ssh{
        Ok(mut ssh_session)=>{
            info!("Connected");
            ssh_session.close().await.unwrap();
            Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::json()).body("{\"response\":\"successful\"}"))
        },
        Err(e)=>{
            Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE).content_type(ContentType::json()).body("{\"response\":\"successful\"}"))
        }

    }
    

    

}



async fn servers(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    // if let Some(name) = query.get("name") {
    tmpl_env.render(
        "servers.html",
        minijinja::context! {
            text => "Welcome!",
        },
    )
}

async fn add_server(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    // if let Some(name) = query.get("name") {
    tmpl_env.render(
        "servers.html",
        minijinja::context! {
            text => "Welcome!",
        },
    )
}

async fn users(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    // if let Some(name) = query.get("name") {
    tmpl_env.render(
        "users.html",
        minijinja::context! {
            text => "Welcome!",
        },
    )
}

async fn add_user(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    tmpl_env.render(
        "users.html",
        minijinja::context! {
            text => "Welcome!",
        }
    )
}


async fn check_ssh_connection(
    tmpl_env: MiniJinjaRenderer,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    // if let Some(name) = query.get("name") {
    tmpl_env.render(
        "users.html",
        minijinja::context! {
            text => "Welcome!",
        },
    )
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    

    let pool = db::db_conn_pool();

    // If TEMPLATE_AUTORELOAD is set, then the path tracking is enabled.
    let enable_template_autoreload = env::var("TEMPLATE_AUTORELOAD").as_deref() == Ok("true");

    if enable_template_autoreload {
        log::info!("template auto-reloading is enabled");
    } else {
        log::info!(
            "template auto-reloading is disabled; run with TEMPLATE_AUTORELOAD=true to enable"
        );
    }

    // The closure is invoked every time the environment is outdated to recreate it.
    let tmpl_reloader = AutoReloader::new(move |notifier| {
        let mut env: minijinja::Environment<'static> = minijinja::Environment::new();

        let tmpl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        // if watch_path is never called, no fs watcher is created
        if enable_template_autoreload {
            notifier.watch_path(&tmpl_path, true);
        }

        env.set_loader(path_loader(tmpl_path));

        Ok(env)
    });

    let tmpl_reloader = web::Data::new(tmpl_reloader);

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(tmpl_reloader.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(web::resource("/").route(web::get().to(hosts)))
            .service(web::resource("/hosts").route(web::get().to(hosts)))
            .service(web::resource("/hosts").route(web::post().to(add_host)))
            .service(web::resource("/hosts/connection").route(web::post().to(test_host_connection)))
            .service(web::resource("/servers").route(web::get().to(servers)))
            .service(web::resource("/servers").route(web::post().to(add_server)))
            .service(web::resource("/users").route(web::get().to(users)))
            .service(web::resource("/users").route(web::post().to(add_user)))
            .service(web::resource("/check_ssh_connection").route(web::get().to(users)))
            .service(fs::Files::new("/assets", "./assets").show_files_listing())
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// Error handler for a 404 Page not found error.
fn not_found<B>(svc_res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let res = get_error_response(&svc_res, "Page not found");

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        svc_res.into_parts().0,
        res.map_into_right_body(),
    )))
}


/// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let req = res.request();

    let tmpl_env = MiniJinjaRenderer::extract(req).into_inner().unwrap();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |err: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(err.to_string())
    };

    let ctx = minijinja::context! {
        error => error,
        status_code => res.status().as_str(),
    };

    match tmpl_env.render("error.html", ctx) {
        Ok(body) => body
            .customize()
            .with_status(res.status())
            .respond_to(req)
            .map_into_boxed_body(),

        Err(_) => fallback(error),
    }
}


