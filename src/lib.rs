use std::{os::fd::RawFd, str::FromStr};
use nix::sys::socket::{
   AddressFamily, Backlog, MsgFlags, SockFlag, SockType, SockaddrIn, accept, bind, listen, recv, recvfrom, send, sendto, setsockopt, socket, sockopt::{Broadcast, ReuseAddr}
};
use nix::unistd::{close};
use std::os::fd::AsRawFd;


pub fn run() {
    let sock_addr = SockaddrIn::from_str("0.0.0.0:3000").unwrap(); 

    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None).unwrap();

    setsockopt(&fd, ReuseAddr, &true).unwrap();
    setsockopt(&fd, Broadcast, &true).unwrap();
    
    bind(fd.as_raw_fd(), &sock_addr).unwrap();

    let mut buf = [0u8; 1024];
            
    loop {
        let (bytes, client) = recvfrom(fd.as_raw_fd(), &mut buf).unwrap();
        

        let client_addr : SockaddrIn = client.unwrap();
    

        println!("Client Address: {:?}, Port: {:?}", &client_addr.ip(), &client_addr.port());
        println!("File descriptor: {:?}", &fd);

        // close(fd.as_raw_fd()).unwrap(); 
    }

}
