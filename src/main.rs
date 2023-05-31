use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Deserialize)]
struct GithubLink {
    link: String,
}

#[derive(Serialize)]
struct CodeLines {
    lines: Vec<String>,
}

async fn fetch_code_from_github(link: String) -> Vec<String> {
    let url = Url::parse(&link).unwrap();
    let path_parts: Vec<&str> = url.path_segments().unwrap().collect();
    let user = path_parts[0];
    let repo = path_parts[1];
    let file_path = &path_parts[3..].join("/");
    let line_numbers: Option<Vec<usize>> = url
        .fragment()
        .map(|fragment| fragment.split('-').map(parse_numbers).collect());

    let client = Client::new();
    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        user, repo, file_path
    );
    let text = client
        .get(&raw_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let code: Vec<String> = match line_numbers.as_deref() {
        Some([line]) => vec![lines[line - 1].clone()],
        Some([start_line, end_line]) => lines[start_line - 1..*end_line].to_vec(),
        None => lines,
        _ => panic!("TODO: non-exhaustive pattern match. please fix"),
    };

    code
}

async fn fetch_pr_from_github(link: String) -> Vec<String> {
    let url = Url::parse(&link).unwrap();
    let path_parts: Vec<&str> = url.path_segments().unwrap().collect();
    let user = path_parts[0];
    let repo = path_parts[1];
    let pull_number = &path_parts[3];

    let client = Client::new();
    let raw_url = format!(
        "https://patch-diff.githubusercontent.com/raw/{}/{}/pull/{}.patch",
        user, repo, pull_number
    );
    let text = client
        .get(&raw_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    text.lines().map(|s| s.to_string()).collect()
}

fn parse_numbers(num: &str) -> usize {
    num.chars()
        .filter(|a| a.is_ascii_digit())
        .collect::<String>()
        .parse::<usize>()
        .unwrap()
}

async fn fetch_code(info: web::Json<GithubLink>) -> impl Responder {
    let code = fetch_code_from_github(info.link.clone()).await;
    let json = serde_json::to_string(&CodeLines { lines: code }).unwrap();
    HttpResponse::Ok().body(json)
}

async fn fetch_pr(info: web::Json<GithubLink>) -> impl Responder {
    let code = fetch_pr_from_github(info.link.clone()).await;
    let json = serde_json::to_string(&CodeLines { lines: code }).unwrap();
    HttpResponse::Ok().body(json)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/fetch_code", web::post().to(fetch_code))
            .route("/fetch_pr", web::post().to(fetch_pr))
            .route("/health-check", web::get().to(health_check))
            .service(fs::Files::new("/", "./src/static/").use_hidden_files())
    })
    .bind(addr)?
    .run()
    .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_digits() {
        assert_eq!(111, parse_numbers("L111"));
    }
}
