mod server;

use server::http::{HttpResponse, HttpServer, HttpStatusCode};

fn main() {
    let addr = "127.0.0.1:3000";

    let server = HttpServer::new(|_request| {
        return HttpResponse {
            status: HttpStatusCode::Ok,
            body: "Hello, world!".to_string(),
        };
    });

    println!("Server is starting on address: {}", addr);

    match server.listen(addr) {
        Ok(()) => (),
        Err(err) => println!("Error: {}", err),
    };
}
