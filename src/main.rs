use std::{
    collections::HashMap, net::{
        SocketAddr,
        TcpStream
    }, process::exit, sync::{
        Arc,
        Mutex
    }, thread
};
use websocket::{sync::{Server, Writer}, OwnedMessage};

struct UserInfo {
    username: String,
    ip: SocketAddr,
    sender: Writer<TcpStream>
}

type UserPool = Arc<Mutex<HashMap<SocketAddr, UserInfo>>>;

fn main() {
	let server = Server::bind("127.0.0.1:8081").unwrap();
    let user_pool: UserPool = Arc::new(Mutex::new(HashMap::new()));

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

			let (mut receiver, sender) = client.split().unwrap();

            let user_info = UserInfo { username: username_string.to_owned(), ip, sender };
			println!("Connection from {}", user_info.ip);

            let mut locked_user_pool = up_ref.lock().unwrap();
            locked_user_pool.insert(ip, user_info);
            drop(locked_user_pool);

			for message in receiver.incoming_messages() {
                let message = message.expect("Error on fetching message");

                match message {
                    OwnedMessage::Text(m) => {
                        println!("message: {m}");
                        let message_with_username = format!("{};{}", m, username_string);
                        let to_send_message = OwnedMessage::Text(message_with_username);

                        let mut locked_user_pool = up_ref.lock().unwrap();
                        locked_user_pool.iter_mut().for_each(|user_info| {
                            user_info.1.sender.send_message(&to_send_message).expect("Could not send dataframe");
                        });

                        drop(locked_user_pool);
                    }
                    OwnedMessage::Close(_) => {
                        let mut locked_user_pool = up_ref.lock().unwrap();
                        locked_user_pool.remove(&ip);
                        drop(locked_user_pool);

                        println!("closed connection from {ip}");
                        return;
                    }
                    _ => {
                        eprintln!("unsupported type of message");
                        exit(1);
                    }
                }
			}
		});
	}
}
