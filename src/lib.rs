use nix::sys::socket::{
    bind, sendto, setsockopt, socket,
    sockopt::{Broadcast, ReuseAddr},
    AddressFamily, MsgFlags, SockFlag, SockType, SockaddrIn,
};
use std::fs;
use std::os::fd::AsRawFd;
use std::thread::sleep;
use std::{str::FromStr, time::Duration};

const WAV_HEADER_SIZE: usize = 44;

pub fn run() {
    let sock_addr = SockaddrIn::from_str("0.0.0.0:3000").unwrap();

    let fd = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .unwrap();

    setsockopt(&fd, ReuseAddr, &true).unwrap();
    setsockopt(&fd, Broadcast, &true).unwrap();

    bind(fd.as_raw_fd(), &sock_addr).unwrap();

    let broadcast = SockaddrIn::from_str("127.0.0.1:3001").unwrap();

    let file_bytes = fs::read("source/652040__jibey__electric-guitar-riff-ratm-style.wav").unwrap();

    if file_bytes.len() < WAV_HEADER_SIZE {
        panic!("File too small to be a valid WAV file");
    }

    let h = &file_bytes[..WAV_HEADER_SIZE];

    let sample_rate = u32::from_le_bytes([h[24], h[25], h[26], h[27]]);
    let channels = u16::from_le_bytes([h[22], h[23]]);
    let bits_per_sample = u16::from_le_bytes([h[34], h[35]]);

    println!(
        "WAV Format: {} Hz, {} channels, {}-bit",
        sample_rate, channels, bits_per_sample
    );

    assert_eq!(channels, 2, "stereo only");
    assert_eq!(bits_per_sample, 24, "24-bit only");
    assert_eq!(sample_rate, 44100, "44.1kHz only");

    let audio_bytes = &file_bytes[WAV_HEADER_SIZE..];

    println!("Starting audio stream to {:?}...", broadcast);
    println!("Audio data size: {} bytes", audio_bytes.len());

    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let bytes_per_ms = (sample_rate as usize * channels as usize * bytes_per_sample) / 1000;

    // Use 2ms packets for smoother streaming and less burstiness
    // 264 bytes/ms * 2ms = 528 bytes per packet
    let packet_duration_ms = 2u64;
    let chunk_bytes = bytes_per_ms * packet_duration_ms as usize;
    let packet_interval = Duration::from_millis(packet_duration_ms);

    println!(
        "Chunk size: {} bytes ({} ms), Target: {} kb/s",
        chunk_bytes,
        packet_duration_ms,
        (bytes_per_ms * 8)
    );

    let mut loop_count = 0u32;

    loop {
        loop_count += 1;
        let loop_start = std::time::Instant::now();
        let mut packet_idx: u64 = 0;
        let mut packets_sent: usize = 0;
        let mut bytes_sent: usize = 0;

        for chunk in audio_bytes.chunks(chunk_bytes) {
            // Calculate exact time this packet should be sent
            let target_time = loop_start + packet_interval * packet_idx as u32;

            // Busy-wait for precise timing (sleep is not accurate enough)
            while std::time::Instant::now() < target_time {
                std::thread::yield_now();
            }

            // Send the packet
            match sendto(fd.as_raw_fd(), chunk, &broadcast, MsgFlags::empty()) {
                Ok(n) => {
                    packets_sent += 1;
                    bytes_sent += n;
                }
                Err(e) => {
                    eprintln!("Send error at packet {}: {:?}", packet_idx, e);
                }
            }

            packet_idx += 1;
        }

        let elapsed = loop_start.elapsed();
        let actual_rate = (bytes_sent * 8) as f64 / elapsed.as_secs_f64() / 1000.0;
        println!(
            "Loop {} complete: {} packets, {} bytes in {:?} ({:.0} kb/s)",
            loop_count, packets_sent, bytes_sent, elapsed, actual_rate
        );
    }
}
