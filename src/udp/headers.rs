use std::fmt;
use std::mem::size_of;

#[derive(PartialEq, Eq, Debug)]
pub enum PacketHeader {
    Disc,
    GET,
    GETACK,
    TCPReceiverExistence,
    UDPReceiverExistence,
    StopWaitData,
    StopWaitACK,
    StopWaitNAK,
    GoBackN,
    SRepeat,
    Unrecognized,
}

impl PacketHeader {
    // So apparently const functions work without even enabling the feature!
    // These three have an \n in the front because they're going to be
    // Parsed as strings, but the others should be treated as raw bytes.
    pub const fn discovery() -> &'static str {
        "DISC\n"
    }
    pub const fn ack() -> &'static str {
        "ACK\n"
    }
    pub const fn get() -> &'static str {
        "GET\n"
    }
    pub const fn stop_and_wait_data() -> &'static str {
        "SWD"
    }
    pub const fn stop_and_wait_ack() -> &'static str {
        "SWA"
    }
    pub const fn stop_and_wait_nak() -> &'static str {
        "SWN"
    }
    pub const fn go_back_n() -> &'static str {
        "GBN"
    }
    pub const fn selective_repeat() -> &'static str {
        "SR"
    }
    pub const fn tcp_get() -> &'static str {
        "TCPGET"
    }

    // TODO: Check each packet header for UDP, only the first packet for TCP.
    pub fn packet_type(packet_str: &str) -> PacketHeader {
        // Doing this repetitive work because the following PR has not been merged as of today:
        // https://github.com/rust-lang/rfcs/pull/2920
        const DISCOVERY: &'static str = PacketHeader::discovery();
        const GET: &'static str = PacketHeader::get();
        const ACK: &'static str = PacketHeader::ack();
        const TCP_GET: &'static str = PacketHeader::tcp_get();
        const STOP_AND_WAIT_DATA: &'static str = PacketHeader::stop_and_wait_data();
        const STOP_AND_WAIT_ACK: &'static str = PacketHeader::stop_and_wait_ack();
        const STOP_AND_WAIT_NAK: &'static str = PacketHeader::stop_and_wait_nak();
        const GO_BACK_N: &'static str = PacketHeader::go_back_n();
        const SELECTIVE_REPEAT: &'static str = PacketHeader::selective_repeat();
        let header_str = packet_str.lines().next().unwrap_or("");
        let header = [header_str, "\n"].join("");
        if header.starts_with(DISCOVERY) {
            PacketHeader::Disc
        } else if header.starts_with(GET) {
            PacketHeader::GET
        } else if header.starts_with(ACK) {
            PacketHeader::GETACK
        } else if header.starts_with(TCP_GET) {
            PacketHeader::TCPReceiverExistence
        } else if header.starts_with(STOP_AND_WAIT_DATA) {
            PacketHeader::StopWaitData
        } else if header.starts_with(STOP_AND_WAIT_ACK) {
            PacketHeader::StopWaitACK
        } else if header.starts_with(STOP_AND_WAIT_NAK) {
            PacketHeader::StopWaitNAK
        } else if header.starts_with(GO_BACK_N) {
            PacketHeader::GoBackN
        } else if header.starts_with(SELECTIVE_REPEAT) {
            PacketHeader::SRepeat
        } else {
            PacketHeader::Unrecognized
        }
    }
}

impl fmt::Display for PacketHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_str: &'static str = "";
        if self == &PacketHeader::Disc {
            display_str = PacketHeader::discovery();
        } else if self == &PacketHeader::GET {
            display_str = PacketHeader::discovery();
        } else if self == &PacketHeader::GETACK {
            display_str = PacketHeader::ack();
        } else if self == &PacketHeader::TCPReceiverExistence {
            display_str = PacketHeader::tcp_get();
        } else if self == &PacketHeader::StopWaitData {
            display_str = PacketHeader::stop_and_wait_data();
        } else if self == &PacketHeader::StopWaitACK {
            display_str = PacketHeader::stop_and_wait_ack();
        } else if self == &PacketHeader::StopWaitNAK {
            display_str = PacketHeader::stop_and_wait_nak();
        } else if self == &PacketHeader::GoBackN {
            display_str = PacketHeader::go_back_n();
        } else if self == &PacketHeader::SRepeat {
            display_str = PacketHeader::selective_repeat();
        } else {
            display_str = PacketHeader::discovery();
        }
        write!(f, "{}", display_str)
    }
}

pub struct TCPHeader {
    pub conn_type: PacketHeader,
    pub udp_get_port: u16,
    pub file_name: String,
}

impl TCPHeader {
    pub fn new(conn_type: PacketHeader, udp_get_port: u16, file_name: String) -> TCPHeader {
        TCPHeader {
            conn_type,
            udp_get_port,
            file_name,
        }
    }

    pub fn from_string(packet: String) -> TCPHeader {
        let mut packet_lines = packet.lines();
        let packet_type = packet_lines.next().unwrap();
        let conn_type = PacketHeader::packet_type(&packet_type);
        let udp_get_port = packet_lines.next().unwrap().parse::<u16>().unwrap_or(0);
        let file_name = packet_lines.next().unwrap_or("").to_string();
        TCPHeader::new(conn_type, udp_get_port, file_name)
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.conn_type, self.udp_get_port, self.file_name
        )
    }
}

pub enum StdinHeader {
    LIST,
    GET,
}

impl StdinHeader {
    pub fn get() -> &'static str {
        "get"
    }
    pub fn list() -> &'static str {
        "list"
    }
}

pub struct StopAndWaitHeader {
    header_size: u16,
    get_port: u16,
    rdt_port: u16,
    file_name: String,
}

impl StopAndWaitHeader {
    pub fn new(
        header_size: u16,
        get_port: u16,
        rdt_port: u16,
        file_name: String,
    ) -> StopAndWaitHeader {
        StopAndWaitHeader {
            header_size,
            get_port,
            rdt_port,
            file_name,
        }
    }

    fn u16_from_bytes(buf: &[u8]) -> u16 {
        let byte_str = std::str::from_utf8(buf).unwrap();
        byte_str.parse::<u16>().unwrap()
    }

    pub fn from_string(buf: &[u8]) -> StopAndWaitHeader {
        let size = size_of::<u16>();
        let header_size = StopAndWaitHeader::u16_from_bytes(&buf[..size]);
        let header_usize: usize = header_size.into();
        let get_port = StopAndWaitHeader::u16_from_bytes(&buf[size..size * 2]);
        let rdt_port = StopAndWaitHeader::u16_from_bytes(&buf[size * 2..size * 3]);
        let file_name = std::str::from_utf8(&buf[size * 3..header_usize])
            .unwrap()
            .to_string();
        StopAndWaitHeader::new(header_size, get_port, rdt_port, file_name)
    }

    pub fn packet_size(file_name: String) -> usize {
        size_of::<u16>() * 3 + file_name.as_bytes().len()
    }
}
