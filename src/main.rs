use axum::{http::StatusCode, routing::get_service, Router};
use std::{fs, io, net::SocketAddr, path::Path, thread, time::Duration};
use tower_http::services::ServeDir;

mod template;

const CONTENT_DIR: &str = "content";
const PUBLIC_DIR: &str = "public";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    build_website(CONTENT_DIR, PUBLIC_DIR).expect("Building website");

    tokio::task::spawn_blocking(move || {
        let mut hotwatch = hotwatch::Hotwatch::new().expect("Failed");
        hotwatch
            .watch(CONTENT_DIR, |_| {
                build_website(CONTENT_DIR, PUBLIC_DIR).expect("Building website");
            })
            .expect("Failed");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    let app = Router::new().nest(
        "/",
        get_service(ServeDir::new(PUBLIC_DIR)).handle_error(|error: io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

/**
 * ウェブサイトをビルドする
 * 1. public ディレクトリ全体を削除する
 * 2. ディレクトリ内全ての md ファイルを処理する
 * 3. public ディレクトリ内の HTML ファイルにレンダリングする
 */
fn build_website(content_dir: &str, output_dir: &str) -> Result<(), anyhow::Error> {
    // public ディレクトリ全体を削除する
    let _ = fs::remove_dir_all(output_dir);

    let markdown_files: Vec<String> = walkdir::WalkDir::new(content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().display().to_string().ends_with(".md"))
        .map(|e| e.path().display().to_string())
        .collect();
    let mut html_files = Vec::with_capacity(markdown_files.len());

    for file in &markdown_files {
        let mut html = template::HEADER.to_owned();
        let markdown_file = fs::read_to_string(&file)?;
        let parser =
            pulldown_cmark::Parser::new_ext(&markdown_file, pulldown_cmark::Options::all());

        let mut body = String::new();

        // ディレクトリ内全ての md ファイルを処理する
        pulldown_cmark::html::push_html(&mut body, parser);

        html.push_str(template::render_body(&body).as_str());
        html.push_str(template::FOOTER);

        let html_file = file
            .replace(content_dir, output_dir)
            .replace(".md", ".html");
        let folder = Path::new(&html_file).parent().unwrap();
        let _ = fs::create_dir_all(folder);

        // public ディレクトリ内の HTML ファイルにレンダリングする
        fs::write(&html_file, html)?;

        html_files.push(html_file);
    }

    write_index(html_files, output_dir)?;

    Ok(())
}

/**
 * リンクを書き込む
 */
fn write_index(files: Vec<String>, output_dir: &str) -> Result<(), anyhow::Error> {
    let mut html = template::HEADER.to_owned();
    let body = files
        .into_iter()
        .map(|file| {
            let file = file.trim_start_matches(output_dir);
            let title = file.trim_start_matches("/").trim_end_matches(".html");
            format!(r#"<a href="{}">{}</a>"#, file, title)
        })
        .collect::<Vec<String>>()
        .join("<br />\n");

    html.push_str(template::render_body(&body).as_str());
    html.push_str(template::FOOTER);

    let index_path = Path::new(&output_dir).join("index.html");

    fs::write(index_path, html)?;

    Ok(())
}
