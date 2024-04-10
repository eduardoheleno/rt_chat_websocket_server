use std::thread;
use websocket::{sync::Server, OwnedMessage};

fn main() {
	let server = Server::bind("127.0.0.1:8081").unwrap();

	for request in server.filter_map(Result::ok) {
		thread::spawn(|| {
            let client = request.accept().unwrap();
			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

			let (mut receiver, mut sender) = client.split().unwrap();

			for message in receiver.incoming_messages() {
				let message = message.unwrap();
                println!("{:?}", message);

                match message {
                    OwnedMessage::Text(_) => {
                        sender.send_message(&message).unwrap();
                        return;
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
