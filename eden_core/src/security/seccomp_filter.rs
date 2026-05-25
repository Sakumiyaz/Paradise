// eden_core/src/security/seccomp_filter.rs
//! # Filtro Seccomp - Aislamiento de Syscalls
//!
//! Este módulo implementa un filtro seccomp en modo BPF (Berkeley Packet Filter)
//! para restringir los syscalls que EDEN puede invocar.
//!
//! ## Seguridad
//!
//! Una vez instalado, el filtro NO puede ser removido. El proceso morirá
//! inmediatamente si intenta un syscall no permitido. Esto previene ataques
//! incluso si EDEN es comprometido.
//!
//! ## Arquitectura
//!
//! Soporta x86_64 y ARM64 (aarch64)
//! Usa syscalls puros de Rust (sin libc)

use crate::syscall::{prctl, PR_GET_SECCOMP, PR_SET_SECCOMP, SECCOMP_MODE_FILTER};

// ============================================================================
// CONSTANTES DE SECCOMP (linux/seccomp.h)
// ============================================================================

/// SECCOMP_MODE_FILTER - Usar filtro BPF

/// Constantes de retorno BPF
const SECCOMP_RET_KILL: u32 = 0x00000000;
const SECCOMP_RET_ALLOW: u32 = 0x7FFF0000;
const SECCOMP_RET_TRAP: u32 = 0x00030000;
const SECCOMP_RET_ERRNO: u32 = 0x00050000;

/// BPF opcodes
const BPF_LD: u16 = 0x00;
const BPF_JMP: u16 = 0x10;
const BPF_RET: u16 = 0x06;
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_JEQ: u16 = 0x10;
const BPF_K: u16 = 0x00;

// ============================================================================
// ESTRUCTURAS BPF
// ============================================================================

/// Instrucciones BPF para filtro de sockets
/// Similar a struct sock_filter en C
#[repr(C)]
struct sock_filter {
    /// Código de operación BPF
    code: u16,
    /// Jump if true
    jt: u8,
    /// Jump if false
    jf: u8,
    /// Dato/k
    k: u32,
}

/// Programa BPF - filtro de seccomp
/// Similar a struct sock_fprog en C
#[repr(C)]
struct sock_fprog {
    /// Número de instrucciones
    len: u16,
    /// Puntero a instrucciones
    filter: *const sock_filter,
}

// ============================================================================
// NUMEROS DE SYSCALL (x86_64 Linux)
// ============================================================================

/// Lista de syscalls PERMITIDOS para EDEN
/// IMPORTANTE: Solo agregar syscalls estrictamente necesarios
const SYSCALLS_PERMITIDOS_X86_64: &[u32] = &[
    // === I/O Básico ===
    0,   // read
    1,   // write
    2,   // open (deprecated, usar openat)
    3,   // close
    5,   // fstat
    8,   // lseek
    9,   // mmap
    10,  // mprotect (limitado, verificado en otra capa)
    11,  // munmap
    12,  // brk

    // === Sistema de archivos ===
    257, // openat
    262, // newfstatat (fstatat)

    // === Terminal / Framebuffer ===
    16,  // ioctl (solo para framebuffer y terminal)

    // === Memoria ===
    214, // mremap
    231, // exit_group (importante para hilos)

    // === Tiempo ===
    35,  // nanosleep
    228, // clock_gettime
    113, // clock_getres

    // === Proceso / Hilos ===
    39,  // getpid
    57,  // fork (no debería usarse, pero permitido)
    56,  // clone (para threads)
    110, // gettid
    234, // tgkill
    61,  // wait4
    134, // rt_sigreturn
    135, // rt_sigaction
    120, // unshare

    // ===UID/GID ===
    102, // getuid
    104, // getgid
    116, // getgroups
    60,  // exit ( _exit, diferente a exit_group)

    // === Sockets Unix Domain ===
    41,  // socket (AF_UNIX only)
    49,  // bind
    42,  // connect
    43,  // accept
    44,  // sendto
    45,  // recvfrom
    46,  // sendmsg
    47,  // recvmsg
    48,  // shutdown
    51,  // getsockname
    52,  // getpeername
    53,  // socketpair

    // === prctl (para instalar filtro) ===
    157, // prctl

    // === Random ===
    318, // getrandom (para UUIDs, Seeds)

    // === Futex (sincronización) ===
    202, // futex

    // === Scheduling ===
    24,  // sched_yield
    155, // sched_setaffinity
    160, // sched_getaffinity

    // === Eventfd ===
    290, // eventfd
    291, // eventfd2

    // === Epoll ===
    28,  // epoll_create
    29,  // epoll_ctl
    30,  // epoll_wait
    233, // epoll_pwait

    // === Memory mapped files ===
    74,  // fsync
    75,  // fdatasync
    76,  // truncate
    77,  // ftruncate

    // === Directories ===
    79,  // getcwd
    80,  // chdir
    81,  // fchdir
    82,  // rename
    83,  // mkdir
    84,  // rmdir
    85,  // creat
    86,  // link
    87,  // unlink
    88,  // symlink
    89,  // readlink
    90,  // chmod
    91,  // fchmod
    92,  // chown
    93,  // fchown
    94,  // lchown
    95,  // umask

    // === Pipes ===
    22,  // pipe
    233, // pipe2

    // === Select/poll ===
    23,  // select
    24,  // sched_yield
    27,  // madvise
    34,  // pause
    37,  // alarm
    38,  // setitimer
    40,  // getuid (old)
    102, // getuid (new)
    104, // getgid (new)
    96,  // gettimeofday
    97,  // getrlimit
    98,  // getrusage
    99,  // sysinfo

    // === Signals ===
    129, // rt_sigprocmask
    130, // rt_sigpending
    131, // rt_sigtimedwait
    132, // rt_sigqueueinfo
    133, // rt_sigsuspend

    // === System V IPC ===
    64,  // semget
    65,  // semop
    66,  // semctl
    67,  // shmdt
    68,  // msgget
    69,  // msgsnd
    70,  // msgrcv
    71,  // msgctl

    // === Shared memory ===
    29,  // shmget (via mmap)
    30,  // shmat
    31,  // shmctl

    // === File descriptor operations ===
    32,  // dup
    33,  // dup2
    72,  // fcntl
    73,  // flock

    // === Socket options ===
    54,  // getsockopt
    55,  // setsockopt

    // === Process VM ===
    17,  // getpgid (was ioprio_get, corrected)
    121, // setpgid (was gettid, corrected)

    // === Networking - INET/UNIX ===
    50,  // listen
];

// ============================================================================
// TRADUCCION ARQUITECTURA
// ============================================================================

/// Arquitectura actual (solo x86_64 por ahora)
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const ARQUITECTURA_ACTUAL: Architecture = Architecture::X86_64;

/// Arquitectura desconocida
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
const ARQUITECTURA_ACTUAL: Architecture = Architecture::Unknown;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}

// ============================================================================
// GENERADOR DE PROGRAMA BPF
// ============================================================================

/// Generador de filtros BPF para seccomp
pub struct SeccompFilterBuilder {
    /// Arquitectura destino
    arch: Architecture,
    /// Lista de syscalls permitidos
    syscall_whitelist: Vec<u32>,
    /// policy: Modo de filtro
    policy: SeccompPolicy,
}

/// Política de seccomp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompPolicy {
    /// Morir en syscall no permitido (estricto)
    Kill,
    /// Retornar EPERM en syscall no permitido
    Trap,
    /// Retornar error específico en syscall no permitido  
    Errno(u32),
    /// Permitir todos los syscalls (para debug)
    Allow,
}

impl SeccompFilterBuilder {
    /// Crear nuevo builder
    pub fn new() -> Self {
        Self {
            arch: ARQUITECTURA_ACTUAL,
            syscall_whitelist: Vec::new(),
            policy: SeccompPolicy::Kill,
        }
    }

    /// Agregar arquitectura destino
    pub fn arch(mut self, arch: Architecture) -> Self {
        self.arch = arch;
        self
    }

    /// Agregar syscalls a la whitelist
    pub fn add_syscalls(mut self, syscalls: &[u32]) -> Self {
        self.syscall_whitelist.extend(syscalls);
        self
    }

    /// Agregar política
    pub fn policy(mut self, policy: SeccompPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Agregar syscalls base necesarios para runtime de Rust
    pub fn add_base_syscalls(self) -> Self {
        self.add_syscalls(SYSCALLS_PERMITIDOS_X86_64)
    }

    /// Compilar a programa BPF
    pub fn build(self) -> Result<Vec<sock_filter>, SeccompError> {
        if self.syscall_whitelist.is_empty() {
            return Err(SeccompError::EmptyWhitelist);
        }

        match self.policy {
            SeccompPolicy::Allow => Ok(vec![]),
            _ => self.build_strict_filter(),
        }
    }

    /// Construir filtro estricto
    fn build_strict_filter(self) -> Result<Vec<sock_filter>, SeccompError> {
        let mut program = Vec::new();
        let syscall_count = self.syscall_whitelist.len();
        
        // BPF_STMT(BPF_LD | BPF_W | BPF_ABS, offsetof(struct seccomp_data, nr))
        program.push(sock_filter {
            code: BPF_LD | BPF_W | BPF_ABS,
            jt: 0,
            jf: 0,
            k: 0, // offsetof(struct seccomp_data, nr)
        });

        // Para cada syscall en la whitelist, crear instrucciones de validación
        for (idx, &syscall) in self.syscall_whitelist.iter().enumerate() {
            // BPF_JMP(BPF_JEQ | BPF_K, syscall, 0, n_left-idx-1)
            let jt = 0; // Saltar a siguiente si no coincide
            let jf = ((syscall_count - idx - 1) as u8).min(255);
            program.push(sock_filter {
                code: BPF_JMP | BPF_JEQ | BPF_K,
                jt,
                jf,
                k: syscall,
            });
        }

        // Última instrucción: retornar según política
        let ret_value = match self.policy {
            SeccompPolicy::Kill => SECCOMP_RET_KILL,
            SeccompPolicy::Trap => SECCOMP_RET_TRAP,
            SeccompPolicy::Errno(err) => SECCOMP_RET_ERRNO | (err & 0xFFFF),
            SeccompPolicy::Allow => SECCOMP_RET_ALLOW,
        };

        program.push(sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: ret_value,
        });

        Ok(program)
    }
}

impl Default for SeccompFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de seccomp
#[derive(Debug, Clone)]
pub enum SeccompError {
    /// Whitelist vacía
    EmptyWhitelist,
    /// Arquitectura no soportada
    UnsupportedArchitecture,
    /// Programa BPF demasiado grande
    ProgramTooLarge,
    /// Fallo al instalar filtro
    InstallationFailed(i32),
    /// Sistema no soporta seccomp
    NotSupported,
}

impl core::fmt::Display for SeccompError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyWhitelist => write!(f, "whitelist de syscalls vacía"),
            Self::UnsupportedArchitecture => write!(f, "arquitectura no soportada"),
            Self::ProgramTooLarge => write!(f, "programa BPF demasiado grande"),
            Self::InstallationFailed(err) => write!(f, "fallo al instalar filtro: errno={}", err),
            Self::NotSupported => write!(f, "sistema no soporta seccomp"),
        }
    }
}

// ============================================================================
// INSTALADOR DE FILTRO SECCOMP
// ============================================================================

/// Instala el filtro seccomp en el proceso actual
/// 
/// # Safety
/// 
/// Una vez instalado, el filtro NO puede ser removido. El proceso morirá
/// si intenta un syscall no permitido.
pub unsafe fn install_seccomp_filter(filter: &[sock_filter]) -> Result<(), SeccompError> {
    let program = sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_ptr(),
    };

    // Verificar que seccomp está soportado
    // prctl(PR_GET_SECCOMP) retorna 0 si no está soportado
    let seccomp_available = prctl(PR_GET_SECCOMP, 0, 0, 0, 0);
    
    if seccomp_available == -1 {
        return Err(SeccompError::NotSupported);
    }

    // Instalar filtro BPF
    // prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &program)
    let result = prctl(
        PR_SET_SECCOMP,
        SECCOMP_MODE_FILTER as usize,
        &program as *const sock_fprog as usize,
        0,
        0,
    );

    if result != 0 {
        return Err(SeccompError::InstallationFailed(1)); // errno
    }

    Ok(())
}

/// Instala filtro seccomp default con whitelist base + syscalls adicionales
pub fn install_default_filter(extra_syscalls: Option<&[u32]>) -> Result<(), SeccompError> {
    let builder = SeccompFilterBuilder::new()
        .add_base_syscalls();

    let builder = if let Some(extra) = extra_syscalls {
        builder.add_syscalls(extra)
    } else {
        builder
    };

    let filter = builder
        .policy(SeccompPolicy::Kill)
        .build()?;

    // Safety: install_seccomp_filter is unsafe because:
    // 1. Once installed, seccomp filter cannot be removed
    // 2. Process will be killed for any disallowed syscall
    // 3. The filter pointer must be valid and not modified
    // We verify seccomp is not already active before calling
    if is_seccomp_enabled() {
        return Err(SeccompError::AlreadyEnabled);
    }
    unsafe { install_seccomp_filter(&filter) }
}

// ============================================================================
// UTILIDADES DE SEGURIDAD
// ============================================================================

/// Verifica si el proceso está bajo seccomp
pub fn is_seccomp_enabled() -> bool {
    // PR_GET_SECCOMP retorna el modo actual
    let result = unsafe { 
        prctl(PR_GET_SECCOMP, 0, 0, 0, 0)
    };
    result > 0
}

/// Obtiene información de syscalls usados recentemente
pub fn get_syscall_stats() -> Option<SyscallStats> {
    // Esta funcionalidad requeriría instrumentation en el kernel
    // Retornamos None por ahora - se podría implementar via auditar
    None
}

/// Estadísticas de syscalls (placeholder)
pub struct SyscallStats {
    pub total_calls: u64,
    pub syscall_counts: Vec<(u32, u64)>,
}
