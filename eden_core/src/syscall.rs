//! # Pure Rust Syscall Interface
//!
//! This module provides direct syscall access without libc dependency.
//! Supports both x86_64 and aarch64 Linux.
#![allow(dead_code)]
#![allow(non_snake_case)]

/// mmap syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_mmap: usize = 9;

#[cfg(target_arch = "aarch64")]
pub const SYS_mmap: usize = 222;

/// mprotect syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_mprotect: usize = 10;

#[cfg(target_arch = "aarch64")]
pub const SYS_mprotect: usize = 226;

/// munmap syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_munmap: usize = 11;

#[cfg(target_arch = "aarch64")]
pub const SYS_munmap: usize = 215;

/// prctl syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_prctl: usize = 157;

#[cfg(target_arch = "aarch64")]
pub const SYS_prctl: usize = 167;

/// fsync syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_fsync: usize = 74;

#[cfg(target_arch = "aarch64")]
pub const SYS_fsync: usize = 82;

/// exit syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_exit: usize = 60;

#[cfg(target_arch = "aarch64")]
pub const SYS_exit: usize = 93;

/// gettid syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_gettid: usize = 186;

#[cfg(target_arch = "aarch64")]
pub const SYS_gettid: usize = 178;

/// rt_sigaction syscall number (for signal handling)
#[cfg(target_arch = "x86_64")]
pub const SYS_rt_sigaction: usize = 134;

#[cfg(target_arch = "aarch64")]
pub const SYS_rt_sigaction: usize = 134;

/// ioctl syscall number
#[cfg(target_arch = "x86_64")]
pub const SYS_ioctl: usize = 16;

#[cfg(target_arch = "aarch64")]
pub const SYS_ioctl: usize = 29;

// ============================================================================
// Memory Protection Constants
// ============================================================================

/// Protection: read only
pub const PROT_READ: usize = 1;

/// Protection: write only
pub const PROT_WRITE: usize = 2;

/// Protection: execute only
pub const PROT_EXEC: usize = 4;

// ============================================================================
// Memory Map Flags
// ============================================================================

/// Share this mapping
pub const MAP_SHARED: usize = 0x01;

/// Share this mapping anonymously (not file-backed)
pub const MAP_ANONYMOUS: usize = 0x20;

/// Map failed constant
pub const MAP_FAILED: usize = usize::MAX;

// ============================================================================
// prctl Constants
// ============================================================================

/// Get seccomp status
pub const PR_GET_SECCOMP: u32 = 16;

/// Set seccomp mode
pub const PR_SET_SECCOMP: u32 = 22;

/// Set no new privileges
pub const PR_SET_NO_NEW_PRIVS: u32 = 38;

/// SECCOMP_MODE_FILTER - Use BPF filter
pub const SECCOMP_MODE_FILTER: u32 = 1;

// ============================================================================
// Signal Constants
// ============================================================================

/// Ignore signal
pub const SIG_IGN: usize = 1;

/// Default signal handling
pub const SIG_DFL: usize = 0;

/// Interrupt signal (SIGINT)
pub const SIGINT: u32 = 2;

// ============================================================================
// ioctl Constants
// ============================================================================

/// Get variable screen info
pub const FBIOGET_VSCREENINFO: u64 = 0x4600;

/// Get fixed screen info
pub const FBIOGET_FSCREENINFO: u64 = 0x4602;

// ============================================================================
// Syscall Functions
// ============================================================================

/// Execute a syscall with 6 arguments
#[inline(always)]
pub unsafe fn syscall6(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> isize {
    let ret: isize;
    
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "syscall",
        inout("rax") n => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        in("r9") a6
    );
    
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!(
        "svc #0",
        inout("x8") n => ret,
        in("x0") a1,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        in("x4") a5,
        in("x5") a6
    );
    
    ret
}

/// Execute a syscall with 5 arguments
#[inline(always)]
pub unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    syscall6(n, a1, a2, a3, a4, a5, 0)
}

/// Execute a syscall with 4 arguments
#[inline(always)]
pub unsafe fn syscall4(n: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    syscall6(n, a1, a2, a3, a4, 0, 0)
}

/// Execute a syscall with 3 arguments
#[inline(always)]
pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> isize {
    syscall6(n, a1, a2, a3, 0, 0, 0)
}

/// Execute a syscall with 2 arguments
#[inline(always)]
pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> isize {
    syscall6(n, a1, a2, 0, 0, 0, 0)
}

/// Execute a syscall with 1 argument
#[inline(always)]
pub unsafe fn syscall1(n: usize, a1: usize) -> isize {
    syscall6(n, a1, 0, 0, 0, 0, 0)
}

/// Execute a syscall with 0 arguments
#[inline(always)]
pub unsafe fn syscall0(n: usize) -> isize {
    syscall6(n, 0, 0, 0, 0, 0, 0)
}

/// ioctl - device control
///
/// # Safety
/// Same safety requirements as ioctl(2)
pub unsafe fn ioctl(fd: i32, request: u64, arg: usize) -> i32 {
    syscall3(SYS_ioctl, fd as usize, request as usize, arg) as i32
}

// ============================================================================
// Wrapped Syscall Functions
// ============================================================================

/// mmap - map memory
///
/// # Safety
/// Same safety requirements as mmap(2)
pub unsafe fn mmap(
    addr: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: i64,
    offset: i64,
) -> *mut u8 {
    let ret = syscall6(
        SYS_mmap,
        addr,
        len,
        prot,
        flags,
        fd as usize,
        offset as usize,
    );
    if ret < 0 {
        return MAP_FAILED as *mut u8;
    }
    ret as *mut u8
}

/// mprotect - set memory protection
///
/// # Safety
/// Same safety requirements as mprotect(2)
pub unsafe fn mprotect(addr: usize, len: usize, prot: usize) -> i32 {
    syscall3(SYS_mprotect, addr, len, prot) as i32
}

/// munmap - unmap memory
///
/// # Safety
/// Same safety requirements as munmap(2)
pub unsafe fn munmap(addr: usize, len: usize) -> i32 {
    syscall2(SYS_munmap, addr, len) as i32
}

/// prctl - process control
pub unsafe fn prctl(option: u32, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> i32 {
    syscall5(SYS_prctl, option as usize, arg2, arg3, arg4, arg5) as i32
}

/// fsync - synchronize file
pub unsafe fn fsync(fd: i32) -> i32 {
    syscall1(SYS_fsync, fd as usize) as i32
}

/// rt_sigaction - set signal handler
///
/// # Safety
/// Same safety requirements as sigaction(2)
pub unsafe fn rt_sigaction(sig: usize, handler: usize, _oldact: usize) -> i32 {
    // sigaction syscall: rt_sigaction(signum, act, oldact)
    // We only use it to ignore signals via SIG_IGN
    syscall3(SYS_rt_sigaction, sig, handler, 0) as i32
}

/// signal - set signal handler (simple interface)
///
/// This is a wrapper around the legacy signal syscall for simple use cases.
/// For SIG_IGN, we can pass the value directly.
///
/// # Safety
/// Same safety requirements as signal(2)
pub unsafe fn signal(sig: usize, handler: usize) -> i32 {
    #[cfg(target_arch = "x86_64")]
    {
        // The legacy signal syscall is syscall 8 on x86_64
        syscall2(8, sig, handler) as i32
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Use rt_sigaction on aarch64 (syscall 134)
        rt_sigaction(sig, handler, 0)
    }
}

/// Exit the current process
pub fn exit(code: i32) -> ! {
    // Use std::process::exit for clean exit
    std::process::exit(code)
}

// ============================================================================
// Directory Functions (Pure Rust replacement for dirs)
// ============================================================================

/// Get the home directory using environment variables
pub fn home_dir() -> Option<std::path::PathBuf> {
    // Try HOME first (Unix)
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            // Fallback for Windows
            std::env::var_os("USERPROFILE")
                .map(std::path::PathBuf::from)
        })
}

/// Get the cache directory
pub fn cache_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            home_dir().map(|h| h.join(".cache"))
        })
}

/// Get the data directory
pub fn data_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            home_dir().map(|h| h.join(".local").join("share"))
        })
}
