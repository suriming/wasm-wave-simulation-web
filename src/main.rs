use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}


fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let check_format = request_line.split(" ").nth(1).unwrap();
    if check_format.ends_with(".wasm") {
        // Serve the wasm file
        let file_path = request_line.split(" ").nth(1).unwrap();
        let (_, file_path) = file_path.split_at(1);
        println!("{}", file_path);
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read(file_path).unwrap();
        let length = contents.len();
        
        let response = format!(
            "{status_line}\r\nContent-Type: application/wasm\r\nContent-Length: {length}\r\n\r\n",
            status_line = status_line,
            length = length,
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(&contents).unwrap();
        // stream.write_all(response.as_bytes()).unwrap();
    }

    else if request_line.starts_with("GET /pkg/") {
        // Serve the file from the pkg folder
        let file_path = request_line.split(" ").nth(1).unwrap();
        let (_, file_path) = file_path.split_at(1);
        println!("{}", file_path);
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string(file_path).unwrap();
        let length = contents.len();
        println!("requestline: {}", request_line);
        
        if file_path.ends_with(".wasm"){
            let response = format!(
                "{status_line}\r\nContent-Type: application/wasm\r\nContent-Length: {length}\r\n\r\n{contents}"        
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        else {
            let response = format!(
                "{status_line}\r\nContent-Type: application/javascript\r\nContent-Length: {length}\r\n\r\n{contents}"
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    } else if request_line == "GET / HTTP/1.1" {
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("index.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    }
}