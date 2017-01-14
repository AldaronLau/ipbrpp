use std::time::Duration;
use std::thread;
use std::net;
use std::env;

const PACKET_HEADER : &'static str = "PlopGrizzlyipbrP";

fn socket(listen_on: net::SocketAddr) -> net::UdpSocket {
	let attempt = net::UdpSocket::bind(listen_on);
	match attempt {
		Ok(socket) => socket,
		Err(err) => panic!("Could not bind: {}", err)
	}
}

fn read_message(socket: net::UdpSocket) -> (Vec<u8>, std::net::SocketAddr) {
	let mut buf: [u8; 256] = [0; 256];
	let result = socket.recv_from(&mut buf);
	drop(socket);
	match result {
		Ok((amt, src)) => (Vec::from(&buf[0..amt]), src),
		Err(err) => panic!("Read error: {}", err)
	}
}

pub fn send_message(send_addr: net::SocketAddr, target: net::SocketAddr,
	data: &Vec<u8>)
{
	let socket1 = socket(send_addr);
	socket1.set_broadcast(true).unwrap();
	println!("Sending data to {}", target);
	let result = socket1.send_to(&data, target);
	drop(socket1);
	match result {
		Ok(amt) => println!("Sent {} bytes", amt),
		Err(err) => panic!("Write error: {}", err)
	}
}

pub fn listen(listen_on: net::SocketAddr)
	-> thread::JoinHandle<(Vec<u8>, std::net::SocketAddr)>
{
	thread::spawn(move || { read_message(socket(listen_on)) })
}

fn server() {
	let ip = net::Ipv4Addr::new(0, 0, 0, 0);
	let listen_addr = net::SocketAddrV4::new(ip, 8141);

	loop {
		let future = listen(net::SocketAddr::V4(listen_addr));
		let (received, client_ip) = future.join().unwrap();
		let rs = String::from_utf8(received).unwrap();
		if rs.len() < 17 {
			println!("Too Small Packet");
			continue;
		}
		let id = &rs[..16];
		let un = &rs[16..];

		if *id != *PACKET_HEADER {
			println!("Incorrect Packet Header: {}", id);
		} else{
			println!("{} online: \"{}\"", client_ip, un);
		}
	}
}

fn broadcast(what: String) {
	let ip = net::Ipv4Addr::new(255, 255, 255, 255); // All on subnet
	let send_addr = net::SocketAddrV4::new(ip, 8142);
	let message: Vec<u8> = what.into_bytes();

	loop {
		send_message(net::SocketAddr::V4(send_addr),
			net::SocketAddr::V4(net::SocketAddrV4::new(ip, 8141)),
			&message);
		thread::sleep(Duration::from_secs(4));
	}
}

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() != 2 {
		println!("Running server....(listening for connections)");
		println!("To run a client: ipbrpp nickname");
		server();
		return;
	}
	println!("Starting with nickname: \"{}\"", args[1]);

	// Start sending data
	broadcast(PACKET_HEADER.to_owned() + &args[1][..]);
}
