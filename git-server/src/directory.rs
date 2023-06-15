use actix_web::{HttpRequest, HttpResponse};
use askama_escape::{escape, Html};
use percent_encoding::{utf8_percent_encode, CONTROLS};
use std::io;
use std::{fmt::Write, path::Path};

pub fn directory_listing(
    dir: impl AsRef<Path>,
    req: &HttpRequest,
) -> Result<HttpResponse, io::Error> {
    let index_of = format!("Index of {}", req.path());
    let mut body = String::new();

    for entry in dir.as_ref().read_dir()? {
        let entry = entry.unwrap();
        let p = match entry.path().strip_prefix(&dir) {
            // Ok(p) if cfg!(windows) => dir.join(p).to_string_lossy().replace('\\', "/"),
            Ok(p) => dir.as_ref().join(p).to_string_lossy().into_owned(),
            Err(_) => continue,
        };

        // if file is a directory, add '/' to the end of the name
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let _ = write!(
                    body,
                    "<li><a href=\"{}\">{}/</a></li>",
                    utf8_percent_encode(&p, CONTROLS),
                    escape(&entry.file_name().to_string_lossy(), Html),
                );
            } else {
                let _ = write!(
                    body,
                    "<li><a href=\"{}\">{}</a></li>",
                    utf8_percent_encode(&p, CONTROLS),
                    escape(&entry.file_name().to_string_lossy(), Html),
                );
            }
        } else {
            continue;
        }
    }

    let html = format!(
        "<html>\
        <head><title>{}</title></head>\
        <body><h1>{}</h1>\
        <ul>\
        {}\
        </ul></body>\n</html>",
        index_of, index_of, body
    );
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
