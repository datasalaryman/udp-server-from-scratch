use std::{os::fd::RawFd, str::FromStr, time::Duration};
use nix::sys::socket::{
   AddressFamily, Backlog, MsgFlags, SockFlag, SockType, SockaddrIn, accept, bind, listen, recv, recvfrom, send, sendto, setsockopt, socket, sockopt::{Broadcast, ReuseAddr}
};
use nix::unistd::{close};
use std::os::fd::AsRawFd;
use std::thread::sleep;

pub fn run() {
    let sock_addr = SockaddrIn::from_str("0.0.0.0:3000").unwrap(); 

    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None).unwrap();

    setsockopt(&fd, ReuseAddr, &true).unwrap();
    setsockopt(&fd, Broadcast, &true).unwrap();
    
    bind(fd.as_raw_fd(), &sock_addr).unwrap();

    let broadcast = SockaddrIn::from_str("255.255.255.255:3001").unwrap();

    let buf = "Hello from server\r\n".as_bytes();
            
    loop {
        
        sendto(fd.as_raw_fd(), &buf, &broadcast, MsgFlags::empty()).unwrap();
        sleep(Duration::from_millis(100));
    };

    // close(fd.as_raw_fd()).unwrap(); 


}
