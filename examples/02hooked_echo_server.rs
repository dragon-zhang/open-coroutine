use libc as c;
use std::ffi::c_void;
use std::io::Error;
use std::mem;
use std::mem::size_of;
use std::os::raw::c_int;
use std::thread;
use std::time::Duration;
use libfiber::fiber::Fiber;
use libfiber::libfiber::{ACL_FIBER_ATTR, size_t};
use libfiber::scheduler::{EventMode, Scheduler};

fn echo_client(fiber: &Fiber, arg: Option<*mut c_void>) {
    let client_socket = arg.unwrap() as c_int;
    unsafe {
        let mut buf = [0u8; 4096];
        loop {
            // let mut pollfd = c::pollfd {
            //     fd: client_socket,
            //     events: c::POLLIN,
            //     revents: 0,
            // };
            // let n = c::poll(&mut pollfd, 1, 3000);
            // if n <= 0 {
            //     break;
            // }

            let n = c::read(client_socket, &mut buf as *mut _ as *mut c_void, buf.len());
            if n <= 0 {
                eprintln!("read error fd:{}, {}", client_socket, Error::last_os_error());
                break;
            }

            let recv_str = String::from_utf8_lossy(&buf[0..n as usize]);
            println!("fiber-{} receive {}", fiber.get_id(), recv_str);
            let n = c::write(client_socket, &recv_str as *const _ as *const c_void, recv_str.len());
            if n < 0 {
                eprintln!("write failed !");
                break;
            }
            if recv_str.starts_with("end") {
                println!("End tcp");
                c::close(client_socket);
            }
        }
        c::close(client_socket);
    }
}

fn fiber_accept(fiber: &Fiber, arg: Option<*mut c_void>) {
    unsafe {
        let socket = c::socket(c::AF_INET, c::SOCK_STREAM, c::IPPROTO_TCP);
        if socket < 0 {
            panic!("last OS error: {:?}", Error::last_os_error());
        }
        if c::setsockopt(socket, c::SOL_SOCKET, c::SO_REUSEADDR,
                         1 as *const c_void, size_of::<i32> as c::socklen_t) > 0 {
            panic!("setsockopt failed !");
        }

        let serv_addr = c::sockaddr_in {
            sin_len: 0,
            sin_family: c::AF_INET as u8,
            sin_port: 9898u16.to_be(),
            sin_addr: c::in_addr {
                s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be()
            },
            sin_zero: mem::zeroed(),
        };
        let result = c::bind(socket, &serv_addr as *const c::sockaddr_in as *const c::sockaddr, mem::size_of_val(&serv_addr) as u32);
        if result < 0 {
            eprintln!("last OS error: {:?}", Error::last_os_error());
            c::close(socket);
        }
        if c::listen(socket, 128) < 0 {
            panic!("listen failed !");
        };
        println!("fiber-{} listen ok !", fiber.get_id());

        loop {
            let mut cliaddr: c::sockaddr_storage = mem::zeroed();
            let mut len = mem::size_of_val(&cliaddr) as u32;
            let client_socket = c::accept(socket, &mut cliaddr as *mut c::sockaddr_storage as *mut c::sockaddr, &mut len);
            if client_socket < 0 {
                eprintln!("last OS error: {:?}", Error::last_os_error());
                break;
            }
            Fiber::new(echo_client, Some(client_socket as *mut c_void), 128000);
        }

        c::close(socket);
    }
}

fn main() {
    Fiber::new(fiber_accept, None, 327680);
    let scheduler = Scheduler::new(EventMode::Kernel);
    scheduler.start();
}