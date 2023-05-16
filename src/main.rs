use hyper::service::{make_service_fn, service_fn};
use hyper::Method;
use hyper::{Body, Request, Response, Server, StatusCode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use url::Url;
use tokio::fs;

#[derive(Deserialize)]
struct GithubLink {
    link: String,
}


#[derive(Serialize)]
struct CodeLines {
    lines: Vec<String>,
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/fetch_code") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let github_link: GithubLink = serde_json::from_slice(&whole_body).unwrap();
            let code = fetch_code_from_github(github_link.link).await;
            let json = serde_json::to_string(&CodeLines { lines: code }).unwrap();
            Ok(Response::new(Body::from(json)))
        },
        (&Method::GET, "/.well-known/ai-plugin.json") => {
            match fs::read_to_string("./src/static/ai-plugin.json").await {
                Ok(contents) => Ok(Response::new(Body::from(contents))),
                Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("404 - Not Found")).unwrap()),
            }
        },
        (&Method::GET, "/openapi.yaml") => {
            match fs::read_to_string("./src/static/openapi.yaml").await {
                Ok(contents) => Ok(Response::new(Body::from(contents))),
                Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("404 - Not Found")).unwrap()),
            }
        },
        (&Method::GET, "/logo.png") => {
            match fs::read("./src/static/logo.png").await {
                Ok(contents) => Ok(Response::new(Body::from(contents))),
                Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("404 - Not Found")).unwrap()),
            }
        },
        _ => {
            let not_found = "Route not found\n";
            Ok(Response::builder()
                .status(404)
                .body(not_found.into())
                .unwrap())
        }
    }
}

fn parse_numbers(num: &str) -> usize {
    num
        .chars()
        .filter(|a| a.is_digit(10))
        .collect::<String>()
        .parse::<usize>()
        .unwrap()
}

async fn fetch_code_from_github(link: String) -> Vec<String> {
    let url = Url::parse(&link).unwrap();
    let path_parts: Vec<&str> = url.path_segments().unwrap().collect();
    let user = path_parts[0];
    let repo = path_parts[1];
    let file_path = &path_parts[3..].join("/");
    let line_numbers: Option<Vec<usize>> = match url.fragment() {
        Some(fragment) => Some(fragment.split('-').map(|n| parse_numbers(n)).collect()),
        None => None,
    };

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

#[tokio::main]
async fn main() {
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_digits() {
        assert_eq!(111, parse_numbers("L111"));
    }
}
