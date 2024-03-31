
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::process::Command;
use actix_files as fs;
use std::error::Error as stdError;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

//use actix_web::HttpRequest;
use actix_web::Error;

use actix_web::http::StatusCode;
use std::cell::RefCell;
use std::fs::read_to_string;
use ssr_rs::Ssr;


//config
#[derive(serde::Deserialize)]
pub struct Settings{
	pub database: DatabaseSettings,
	pub application_port: u16,
	pub password: String

}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: String,
	pub port: u16,
	pub host: String,
	pub database_name: String

}

impl DatabaseSettings {
	pub fn connection_string(&self) -> String {
		format!("postgres://{}:{}@{}:{}",self.username, self.password, self.host, self.port)
	}
}

// get config
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
	let settings = config::Config::builder()
		.add_source(config::File::new("/app/configuration.yaml", config::FileFormat::Yaml))
		.build()?;
	settings.try_deserialize::<Settings>()
}


// index
async fn index() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("/app/www/index.html")?)
}


// status
async fn status() -> String {
    "Server is up and running.".to_string()
}

//-- ssr --
thread_local! {
    static SSR: RefCell<Ssr<'static, 'static>> = RefCell::new(
            Ssr::from(
                read_to_string("./app/www/index.js").unwrap(),
                "SSR"
                ).unwrap()
            )
}

async fn ssr_index() -> HttpResponse {
    let result = SSR.with(|ssr| ssr.borrow_mut().render_to_string(None).unwrap());

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(result)
}



// ------API------

// UPDATE
async fn update(req_body: String) -> impl Responder {
	if req_body == "kekw" {
		println!("update...");
		let mut cmd = Command::new("bash");
		let out = cmd.arg("-c").arg("update-www.sh").output().expect("failed to execute update");
		println!("{:?}", out);
	}
    HttpResponse::Ok()
}

// submit
async fn submit(req_body: String) -> impl Responder {

	//get config
	let configuration = match get_configuration() {
		Ok(c) => c,
		Err(_) => return HttpResponse::BadRequest(),
	};

	let url = configuration.database.connection_string();
	
	match add_customer(req_body, url).await {
		Ok(()) => HttpResponse::Ok(),
		Err(_) => HttpResponse::BadRequest(),
	};

	HttpResponse::Ok()
}

// DATABASE postgres
async fn add_customer(c_string: String, url: String) -> Result<(), Box<dyn stdError>> {

	let pool = match sqlx::postgres::PgPool::connect(&url).await {
		Ok(p) => p,
		Err(e) => return Err(Box::new(e)),
	};


	let parts = c_string.split("|");
	let data: Vec<&str> = parts.collect();


	let query = "INSERT INTO kunde (Kundennummer, Name, Email, Nachricht, Status) VALUES ($1, $2, $3, $4, $5)";
	match sqlx::query(query)
		.bind("0".to_string())
		.bind(&data[0].to_string())		
		.bind(&data[1].to_string())
		.bind(&data[2].to_string())
		.bind("nix".to_string())
		.execute(&pool).await {
			Ok(_) => Ok(()),
			Err(e) => Err(Box::new(e)),
		}

}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
	
	Ssr::create_platform();

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    
	builder.set_certificate_chain_file("cert.pem").unwrap();

	HttpServer::new(|| {
		App::new()
			.route("/status", web::get().to(status))
			.route("/", web::get().to(index))
			.route("/submit", web::put().to(submit))
			.route("/update", web::get().to(update))
			.service(fs::Files::new("/", "/app/www"))
			.default_service(web::get().to(index))
	    
    })
    .bind_openssl("0.0.0.0:8000", builder)?
    .run()
    .await
}



