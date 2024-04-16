use std::{net::{SocketAddr, TcpStream}, sync::{Arc, Mutex}, thread};
use websocket::{sync::{Server, Writer}, OwnedMessage};

struct UserInfo {
    username: String,
    ip: SocketAddr,
    sender: Writer<TcpStream>
}

type UserPool = Arc<Mutex<Vec<UserInfo>>>;

fn main() {
	let server = Server::bind("127.0.0.1:8081").unwrap();

    let user_pool: UserPool = Arc::new(Mutex::new(Vec::new()));

	for request in server.filter_map(Result::ok) {
        let up_ref = user_pool.clone();

		thread::spawn(move || {
            let mut headers_iterator = request.request.headers.iter();
            let username_header = headers_iterator.find(|h| h.name() == "username");
            let username_string = match username_header {
                Some(h) => h.value_string(),
                None => {
                    eprintln!("username header not found.");
                    return;
                }
            };

            println!("{username_string}");

            let client = request.accept().unwrap();
			let ip = client.peer_addr().unwrap();

			let (mut receiver, mut sender) = client.split().unwrap();

            let user_info = UserInfo { username: username_string, ip, sender };
			println!("Connection from {}", user_info.ip);

            let mut locked_user_pool = up_ref.lock().unwrap();
            locked_user_pool.push(user_info);
            drop(locked_user_pool);

            // TODO: can't receive messages
			for message in receiver.incoming_messages() {
				let message = message.unwrap();

                match message {
                    OwnedMessage::Text(m) => {
                        println!("message: {m}");
                    }
                    _ => {
                        println!("no data");
                        return;
                    }
                }
			}
		});
	}
}
