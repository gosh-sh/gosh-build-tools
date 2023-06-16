use actix_web::{HttpRequest, HttpResponse};
use askama_escape::{escape, Html};
use percent_encoding::{utf8_percent_encode, CONTROLS};
use std::io;
use std::{fmt::Write, path::Path};

pub fn directory_listing(
    directory: impl AsRef<Path>,
    req: &HttpRequest,
) -> Result<HttpResponse, io::Error> {
    let index_of = format!("Index of {}", req.path());

    let req_path_dir = req.path().trim_end_matches('/');
    let mut body = String::new();
    for entry in directory.as_ref().read_dir()? {
        let entry = entry?;

        // if file is a directory, add '/' to the end of the name
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        if let Ok(entry_path) = entry.path().strip_prefix(&directory) {
            let lossy_path = entry_path.to_string_lossy();
            let url = format!("{req_path_dir}/{lossy_path}");
            let encoded_url = utf8_percent_encode(&url, CONTROLS);
            let escaped_html = escape(&lossy_path, Html);

            if metadata.is_dir() {
                let _ = writeln!(
                    body,
                    "<li><a href=\"{encoded_url}/\">{escaped_html}/</a></li>",
                );
            } else {
                let _ = writeln!(
                    body,
                    "<li><a href=\"{encoded_url}\">{escaped_html}</a></li>",
                );
            }
        } else {
            continue;
        }
    }

    let html = format!(
        "\
<html>
<head><title>{}</title></head>\
<body><h1>{}</h1>\
<hr>
<ul>\
{}\
</ul>
<hr>
</body>
</html>\
",
        index_of, index_of, body
    );
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
