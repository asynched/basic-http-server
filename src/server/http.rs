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

impl Into<(i32, &'static str)> for HttpStatusCode {
    fn into(self) -> (i32, &'static str) {
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
            let mut buffer = [0; 8192];
            stream.read(&mut buffer).unwrap();

            let request = parse_request(&buffer);
            let response = (self.handler)(request);

            let (status_code, status) = response.status.into();

            let mut headers = String::new();

            for (key, value) in response.headers {
                headers.push_str(&format!("{}: {}\r\n", key, value));
            }

            let response = format!(
                "HTTP/1.1 {} {}\r\n{}\r\n{}",
                status_code, status, headers, response.body
            );

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }

        return Ok(());
    }
}

fn parse_request(_buffer: &[u8]) -> HttpRequest {
    return HttpRequest {
        body: "".to_string(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        path: "".to_string(),
    };
}
