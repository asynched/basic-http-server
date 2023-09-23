use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpListener;

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
    fn into_http_status(self) -> (i32, &'static str) {
        match self {
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
        }
    }
}

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct HttpResponse {
    pub status: HttpStatusCode,
    pub body: String,
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
            let mut buffer = [0; 8192];
            stream.read(&mut buffer).unwrap();

            let request = parse_request(&buffer);
            let response = (self.handler)(request);

            let (status_code, status) = response.status.into_http_status();
            let response = format!(
                "HTTP/1.1 {} {}\r\n\r\n{}",
                status_code, status, response.body
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }

        return Ok(());
    }
}

fn parse_request(buffer: &[u8]) -> HttpRequest {
    return HttpRequest {
        body: "".to_string(),
        headers: HashMap::new(),
        method: "".to_string(),
        path: "".to_string(),
    };
}
