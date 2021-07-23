// use actix_web::{get, middleware, web::Path, App, HttpServer, Result};
// use std::fs::read_to_string;
// use std::time::Instant;

// #[get("/{package}")]
// async fn package(package: Path<String>) -> Result<String> {
//     let now = Instant::now();
//     read_to_string(format!("packages/{}.json", package.to_string())).unwrap();
//     println!("{}", now.elapsed().as_secs_f32());
//     Ok(read_to_string(format!("packages/{}.json", package.to_string())).unwrap())
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .wrap(middleware::Compress::default())
//             .service(package)
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }
use actix_files as fs;
use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .service(fs::Files::new("/", ".").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
