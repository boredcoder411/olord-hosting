use actix_web::{get, post, App, Error, HttpResponse, HttpServer, HttpRequest, Responder};
use actix_files as fs;
use actix_web::http::header::{ContentDisposition, DispositionType};
use std::path::PathBuf;
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use std::io::Write;
use uuid::Uuid;
use std::fs::File;

#[get("/{filename:.*}")]
async fn index(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = fs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[post("/upload")]
async fn upload(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = item.expect("Failed to read field");

        // Generate a UUID for the filename
        let file_id = Uuid::new_v4();
        let file_path = format!("./uploads/{}.olord", file_id);

        // Create a new file to save the data
        let mut file = File::create(file_path).expect("Failed to create file");

        // Write the file content to disk
        while let Some(chunk) = field.next().await {
            let data = chunk.expect("Failed to read chunk");
            file.write_all(&data).expect("Failed to write to file");
        }
    }
    HttpResponse::Ok().body("File uploaded successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create uploads directory if it doesn't exist
    let upload_dir = PathBuf::from("./uploads");
    std::fs::create_dir_all(&upload_dir).expect("Failed to create upload directory");

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(upload)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

