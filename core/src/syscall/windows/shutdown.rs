use crate::net::EventLoops;
use crate::syscall::set_errno;
use once_cell::sync::Lazy;
use std::ffi::c_int;
use windows_sys::Win32::Networking::WinSock::{SOCKET, WINSOCK_SHUTDOWN_HOW, SD_RECEIVE, SD_SEND, SD_BOTH, WSAEINVAL};

#[must_use]
pub extern "system" fn shutdown(
    fn_ptr: Option<&extern "system" fn(SOCKET, WINSOCK_SHUTDOWN_HOW) -> c_int>,
    fd: SOCKET,
    how: WINSOCK_SHUTDOWN_HOW,
) -> c_int {
    static CHAIN: Lazy<ShutdownSyscallFacade<NioShutdownSyscall<RawShutdownSyscall>>> =
        Lazy::new(Default::default);
    CHAIN.shutdown(fn_ptr, fd, how)
}

trait ShutdownSyscall {
    extern "system" fn shutdown(
        &self,
        fn_ptr: Option<&extern "system" fn(SOCKET, WINSOCK_SHUTDOWN_HOW) -> c_int>,
        fd: SOCKET,
        how: WINSOCK_SHUTDOWN_HOW,
    ) -> c_int;
}

impl_facade!(ShutdownSyscallFacade, ShutdownSyscall, shutdown(fd: SOCKET, how: WINSOCK_SHUTDOWN_HOW) -> c_int);

#[repr(C)]
#[derive(Debug, Default)]
struct NioShutdownSyscall<I: ShutdownSyscall> {
    inner: I,
}

impl<I: ShutdownSyscall> ShutdownSyscall for NioShutdownSyscall<I> {
    extern "system" fn shutdown(
        &self,
        fn_ptr: Option<&extern "system" fn(SOCKET, WINSOCK_SHUTDOWN_HOW) -> c_int>,
        fd: SOCKET,
        how: WINSOCK_SHUTDOWN_HOW,
    ) -> c_int {
        {
            let fd = fd.try_into().expect("overflow");
            _ = match how {
                SD_RECEIVE => EventLoops::del_read_event(fd),
                SD_SEND => EventLoops::del_write_event(fd),
                SD_BOTH => EventLoops::del_event(fd),
                _ => {
                    set_errno(WSAEINVAL.try_into().expect("overflow"));
                    return -1;
                }
            };
        }
        self.inner.shutdown(fn_ptr, fd, how)
    }
}

impl_raw!(RawShutdownSyscall, ShutdownSyscall, windows_sys::Win32::Networking::WinSock,
    shutdown(fd: SOCKET, how: WINSOCK_SHUTDOWN_HOW) -> c_int
);
