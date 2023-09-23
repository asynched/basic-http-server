mod server;

use server::http::{HttpResponseBuilder, HttpServer, HttpStatusCode};

fn main() {
    let addr = "127.0.0.1:3000";

    let server = HttpServer::new(|request| {
        let response = HttpResponseBuilder::new();

        println!("{:?}", request);

        response
            .status(HttpStatusCode::Ok)
            .header("Content-Type", "text/html")
            .header("X-Powered-By", "rust/basic-http-server")
            .body("<h1>Hello, world!</h1>")
            .build()
    });

    println!("Server is starting on address: {}", addr);

    match server.listen(addr) {
        Ok(()) => (),
        Err(err) => println!("Error: {}", err),
    };
}
