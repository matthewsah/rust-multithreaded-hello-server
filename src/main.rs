use multithreaded_hello::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn handle_connection(mut stream: TcpStream) {
    let buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut stream);

    let request_line: String = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"), // case 1
        "GET /sleep HTTP/1.1" => {
            // case 2
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        } // case 3
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents: String = fs::read_to_string(filename).unwrap();
    let length: usize = contents.len();
    let response: String = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool: ThreadPool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream: TcpStream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    // We’ll limit the number of threads in the pool to a small number to
    // protect us from Denial of Service (DoS) attacks; if we had our program
    // create a new thread for each request as it came in, someone making 10
    // million requests to our server could create havoc by using up all our
    // server’s resources and grinding the processing of requests to a halt.

    // Rather than spawning unlimited threads, then, we’ll have a fixed number
    // of threads waiting in the pool. Requests that come in are sent to the
    // pool for processing. The pool will maintain a queue of incoming
    // requests. Each of the threads in the pool will pop off a request from
    // this queue, handle the request, and then ask the queue for another
    // request. With this design, we can process up to N requests
    // concurrently, where N is the number of threads. If each thread is
    // responding to a long-running request, subsequent requests can still
    // back up in the queue, but we’ve increased the number of long-running
    // requests we can handle before reaching that point.
}
