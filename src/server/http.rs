use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum HttpStatusCode {
    Ok,
    Created,
    Accepted,
    NoContent,
    MovedPermanently,
    Found,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
}

impl HttpStatusCode {
    fn to_http_status(self) -> (i32, &'static str) {
        return match self {
            HttpStatusCode::Ok => (200, "OK"),
            HttpStatusCode::Created => (201, "CREATED"),
            HttpStatusCode::Accepted => (202, "ACCEPTED"),
            HttpStatusCode::NoContent => (204, "NO CONTENT"),
            HttpStatusCode::MovedPermanently => (301, "MOVED PERMANENTLY"),
            HttpStatusCode::Found => (302, "FOUND"),
            HttpStatusCode::NotModified => (304, "NOT MODIFIED"),
            HttpStatusCode::BadRequest => (400, "BAD REQUEST"),
            HttpStatusCode::Unauthorized => (401, "UNAUTHORIZED"),
            HttpStatusCode::Forbidden => (403, "FORBIDDEN"),
            HttpStatusCode::NotFound => (404, "NOT FOUND"),
            HttpStatusCode::MethodNotAllowed => (405, "METHOD NOT ALLOWED"),
            HttpStatusCode::InternalServerError => (500, "INTERNAL SERVER ERROR"),
            HttpStatusCode::NotImplemented => (501, "NOT IMPLEMENTED"),
            HttpStatusCode::BadGateway => (502, "BAD GATEWAY"),
            HttpStatusCode::ServiceUnavailable => (503, "SERVICE UNAVAILABLE"),
        };
    }
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        return match method {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "TRACE" => Ok(HttpMethod::Trace),
            _ => Err(()),
        };
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct HttpResponse {
    pub status: HttpStatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ToString for HttpResponse {
    fn to_string(&self) -> String {
        let (status_code, status) = self.status.clone().to_http_status();

        let mut headers = String::new();

        for (key, value) in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", key, value));
        }

        let response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n{}",
            status_code, status, headers, self.body
        );

        return response;
    }
}

pub struct HttpResponseBuilder {
    status: HttpStatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponseBuilder {
    pub fn new() -> HttpResponseBuilder {
        return HttpResponseBuilder {
            status: HttpStatusCode::Ok,
            headers: HashMap::new(),
            body: "".to_string(),
        };
    }

    pub fn status(mut self, status: HttpStatusCode) -> HttpResponseBuilder {
        self.status = status;
        return self;
    }

    pub fn header(mut self, key: &str, value: &str) -> HttpResponseBuilder {
        self.headers.insert(key.to_string(), value.to_string());
        return self;
    }

    pub fn body(mut self, body: &str) -> HttpResponseBuilder {
        self.body = body.to_string();
        return self;
    }

    pub fn build(self) -> HttpResponse {
        return HttpResponse {
            status: self.status,
            headers: self.headers,
            body: self.body,
        };
    }
}

pub struct HttpServer {
    handler: Box<dyn Fn(HttpRequest) -> HttpResponse>,
}

impl HttpServer {
    pub fn new(handler: impl Fn(HttpRequest) -> HttpResponse + 'static) -> HttpServer {
        return HttpServer {
            handler: Box::new(handler),
        };
    }

    pub fn listen(&self, addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;

        while let Ok((mut stream, _)) = listener.accept() {
            let Ok(request) = parse_request(&mut stream) else {
                continue;
            };

            let response = (self.handler)(request);

            if let Ok(_) = stream.write(response.to_string().as_bytes()) {
                match stream.flush() {
                    Err(err) => println!("Error: {}", err),
                    _ => (),
                };
            }
        }

        return Ok(());
    }
}

enum ParseError {
    Unknown,
}

fn parse_request(stream: &mut TcpStream) -> Result<HttpRequest, ParseError> {
    let mut buffer = [0; 2048];

    stream.read(&mut buffer).unwrap();

    let source = String::from_utf8_lossy(&buffer);

    let mut lines = source.split("\r\n");

    let Some(first_line) = lines.next() else {
        return Err(ParseError::Unknown);
    };

    let mut parts = first_line.split_whitespace();

    let Some(Ok(method)) = parts.next().map(|method| method.parse::<HttpMethod>()) else {
        return Err(ParseError::Unknown);
    };

    let Some(path) = parts.next().map(|path| path.to_string()) else {
        return Err(ParseError::Unknown);
    };

    let mut headers = HashMap::new();

    for line in lines.clone() {
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, ": ");

        let key = parts.next().unwrap().to_string();
        let value = parts.next().unwrap().to_string();

        headers.insert(key, value);
    }

    let content_length = headers
        .get("Content-Length")
        .map(|value| value.parse::<usize>())
        .unwrap_or(Ok(0))
        .unwrap();

    let body = lines
        .skip(headers.len() + 1)
        .collect::<Vec<&str>>()
        .join("\r\n");

    let body = body.trim_matches(char::from(0));
    let left_to_read = content_length - body.len();

    if left_to_read == 0 {
        return Ok(HttpRequest {
            method,
            path,
            headers,
            body: body.to_string(),
        });
    }

    let mut buffer = vec![0; left_to_read];

    stream.read(&mut buffer).unwrap();

    let body = format!("{}{}", body, String::from_utf8_lossy(&buffer));

    return Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    });
}
