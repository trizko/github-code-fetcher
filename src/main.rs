use hyper::service::{make_service_fn, service_fn};
use hyper::Method;
use hyper::{Body, Request, Response, Server};
use reqwest::Client;
use serde::Deserialize;
use std::convert::Infallible;
use url::Url;

#[derive(Deserialize)]
struct GithubLink {
    link: String,
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/fetch_code") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let github_link: GithubLink = serde_json::from_slice(&whole_body).unwrap();
            let code = fetch_code_from_github(github_link.link).await;
            Ok(Response::new(Body::from(code)))
        }
        _ => {
            let not_found = "Route not found\n";
            Ok(Response::builder()
                .status(404)
                .body(not_found.into())
                .unwrap())
        }
    }
}

fn parse_numbers(t_num: &str) -> usize {
    t_num
        .chars()
        .filter(|a| a.is_digit(10))
        .collect::<String>()
        .parse::<usize>()
        .unwrap()
}

async fn fetch_code_from_github(link: String) -> String {
    let url = Url::parse(&link).unwrap();
    let path_parts: Vec<&str> = url.path_segments().unwrap().collect();
    println!("path_parts: {:?}", path_parts);
    let user = path_parts[0];
    println!("user: {:?}", user);
    let repo = path_parts[1];
    println!("repo: {:?}", repo);
    let file_path = &path_parts[3..].join("/");
    println!("file_path: {:?}", file_path);
    let line_numbers: Vec<&str> = url.fragment().unwrap().split('-').collect();
    println!("line_numbers: {:?}", line_numbers);
    let start_line = parse_numbers(line_numbers[0]);
    println!("start_line: {:?}", start_line);
    let end_line = parse_numbers(line_numbers[1]);
    println!("end_line: {:?}", end_line);

    let client = Client::new();
    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        user, repo, file_path
    );
    println!("raw_url: {:?}", raw_url);
    let text = client
        .get(&raw_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let lines: Vec<&str> = text.lines().collect();
    println!("lines: {:?}", lines);
    let code: Vec<&str> = lines
        [start_line - 1..end_line]
        .to_vec();
    code.join("\n")
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