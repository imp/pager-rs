extern crate libc;

use std::env;
use std::ffi::CString;
use std::ptr;
use std::os::unix::ffi::OsStrExt;

const DEFAULT_PAGER_ENV: &'static str = "PAGER";

#[derive(Debug, Default)]
pub struct Pager {
    env: String,
    ok: bool,
}

impl Pager {
    pub fn new() -> Self {
        Pager {
            env: String::from(DEFAULT_PAGER_ENV),
            ok: true,
        }
    }

    pub fn env(&mut self, env: &str) -> &Self {
        self.env = String::from(env);
        self
    }

    pub fn ok(&self) -> bool {
        self.ok
    }

    pub fn setup(&mut self) {
        if let Some(pager) = getenv(&self.env) {
            let (pager_stdin, main_stdout) = pipe();
            let pid = fork();
            match pid {
                -1 => {
                    // Fork failed
                    close(pager_stdin);
                    close(main_stdout);
                    self.ok = false
                }
                0 => {
                    // I am child
                    dup2(main_stdout, libc::STDOUT_FILENO);
                    close(pager_stdin);
                }
                _ => {
                    // I am parent
                    let argv = vec![pager.as_ptr(), ptr::null()];
                    dup2(pager_stdin, libc::STDIN_FILENO);
                    close(main_stdout);
                    execvp(argv);
                }
            }
        }
    }
}

// In C this would be simple getenv(). Not in Rust though
fn getenv(var: &str) -> Option<CString> {
    let value = env::var_os(&var).unwrap();
    let value = value.as_os_str().as_bytes();
    let value = CString::new(value);
    value.ok()
    // let to_bytes = |x: &OsString| x.as_os_str().as_bytes();
    // let to_bytes = |x: &OsString| x.into::<Vec<u8>>();
    // env::var_os(&self.env).map(to_bytes).and_then(|x| CString::new(x).ok())
}


// Helper wrappers around libc::* API
fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

fn execvp(argv: Vec<*const libc::c_char>) {
    assert!(unsafe { libc::execvp(argv[0], argv.as_ptr()) } > -1);
}

fn dup2(fd1: i32, fd2: i32) {
    assert!(unsafe { libc::dup2(fd1, fd2) } > -1);
}

fn close(fd: i32) {
    assert_eq!(unsafe { libc::close(fd) }, 0);
}

fn pipe() -> (i32, i32) {
    let mut fds = [0; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    (fds[0], fds[1])
}