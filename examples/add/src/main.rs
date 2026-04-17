use std::any::Any;
use std::env;
use std::ffi::CString;
use std::os::fd::AsRawFd;

// Prelude:
use omniglot::id::OGID;
use omniglot::markers::{AccessScope, AllocScope};

// Auto-generated bindings, so doesn't follow Rust conventions at all:
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(improper_ctypes)] // TODO: fix this by wrapping functions with u128s
pub mod libadd {
    include!(concat!(env!("OUT_DIR"), "/libogadd_bindings.rs"));
}

// These are the Omniglot wrapper types / traits generated.
use libadd::{LibAdd, LibAddRt};

pub fn with_mpkrt_lib<ID: OGID, R>(
    brand: ID,
    f: impl for<'a> FnOnce(
        LibAddRt<ID, omniglot_mpk::OGMPKRuntime<ID>, omniglot_mpk::OGMPKRuntime<ID>>,
        AllocScope<
            <omniglot_mpk::OGMPKRuntime<ID> as omniglot::rt::OGRuntime>::AllocTracker<'a>,
            ID,
        >,
        AccessScope<ID>,
    ) -> R,
) -> R {
    let (rt, alloc, access) = omniglot_mpk::OGMPKRuntime::new(
        [CString::new(concat!(env!("OUT_DIR"), "/libadd.so")).unwrap()].into_iter(),
        brand,
        //Some(GLOBAL_PKEY_ALLOC.get_pkey()),
        None,
        false,
    );

    // Create a "bound" runtime, which implements the LibPng API:
    let bound_rt = LibAddRt::new(rt).unwrap();

    // Run the provided closure:
    f(bound_rt, alloc, access)
}

fn main() {
    env_logger::init();

    // try and cast the last argument to an integer, which we'll use as the test id
    // if that fails, error and exit
    let test_id = match env::args().last().map(|s| s.parse::<u8>()) {
        Some(Ok(id)) => id,
        _ => {
            log::error!("Usage: {} <test_id>", env::args().next().unwrap());
            std::process::exit(1);
        }
    };

    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        with_mpkrt_lib(brand, |lib, mut alloc, mut access| {
            install_segv_handler();

            match test_id {
                1 => {
                    log::info!("Running test 1: simple add");
                    // This is a simple test to verify that basic computation and pass-by-value works correctly

                    let ret = lib
                        .add(1, 2, &mut alloc, &mut access)
                        .unwrap()
                        .validate()
                        .unwrap();
                    assert_eq!(ret, 3);
                }
                2 => {
                    log::info!("Running test 2: add pointer result");
                    // This tests the add_ptr function, which returns a pointer
                    // ensuring that rust can read from the FFI memory correctly

                    // Note: this is known to leak memory, since the library allocates but never frees
                    // as rust can't free FFI-allocated memor
                    unsafe {
                        let ret = lib
                            .add_ptr(1, 2, &mut alloc, &mut access)
                            .unwrap()
                            .assume_valid();
                        assert_eq!(*ret, 3);
                    }
                }
                3 => {
                    // 3. segfaulting pass in mem
                    log::info!("Running test 3: segfaulting pass in mem");
                    // The FFI should not be able to access the rust heap, so this should cause a segfault
                    let mut rust_mem = Box::new(5);
                    log::debug!("Box address: {:p}", rust_mem.as_ref());
                    let ret =
                        lib.cannot_deref_ptr_add(rust_mem.as_mut(), 2, &mut alloc, &mut access);
                    unreachable!("Expected a segfault, but function returned: {:?}", ret);
                }
                4 => {
                    log::info!("Running test 4: evil add with Rust heap pointer");
                    // This is the "evil" test, which tries to pass a Rust heap pointer, but the library modifies the MPK
                    // to bypass the protections
                    let mut rust_mem = Box::new(5);
                    let ret = lib.evil_cannot_deref_ptr_add(
                        rust_mem.as_mut(),
                        2,
                        &mut alloc,
                        &mut access,
                    );
                    let ptr = ret.unwrap().validate().unwrap();
                    unsafe {
                        assert_eq!(*ptr, 7);
                    }
                }
                5 => {
                    log::info!("Running test 5: intentionally allow FFI to access specific Rust heap memory");
                    // This test demonstrates how the Rust code can intentionally allow the FFI to access specific heap
                    lib.rt()
                        .allocate_stacked_t_mut(&mut alloc, |val_ref, mut alloc| {
                            val_ref.write(5, &mut access);

                            let ret = lib.cannot_deref_ptr_add(
                                val_ref.as_ptr() as *mut std::os::raw::c_int,
                                2,
                                &mut alloc,
                                &mut access,
                            );
                            let ptr = ret.unwrap().validate().unwrap();
                            unsafe {
                                assert_eq!(*ptr, 7);
                            }
                        });
                }
                _ => {
                    log::error!("Unknown test_id: {}", test_id);
                    std::process::exit(1);
                }
            }
        });
    });
}

use libc::{sigaction, siginfo_t, SA_SIGINFO, SIGBUS, SIGSEGV};
use omniglot::rt::OGRuntime;
use std::mem::zeroed;
use std::ptr;

// Linux-stable si_code values
const SEGV_MAPERR: i32 = 1;
const SEGV_ACCERR: i32 = 2;
const SEGV_PKUERR: i32 = 4;

// Minimal async-signal-safe string write
unsafe fn write_str(s: &str) {
    libc::write(2, s.as_ptr() as *const _, s.len());
}

// Minimal async-signal-safe integer printer
unsafe fn write_int(n: i32) {
    let mut buf = [0u8; 16];
    let mut i = 15;
    buf[i] = b'\n'; // Append newline

    let mut un = if n < 0 { -(n as i64) as u64 } else { n as u64 };

    if un == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while un > 0 {
            i -= 1;
            buf[i] = b'0' + (un % 10) as u8;
            un /= 10;
        }
    }
    if n < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    libc::write(2, buf[i..].as_ptr() as *const _, 16 - i);
}

extern "C" fn segv_handler(sig: i32, info: *mut siginfo_t, _ucontext: *mut libc::c_void) {
    unsafe {
        if info.is_null() {
            write_str("SIGNAL: info is null (cannot extract si_code)\n");
            libc::exit(1);
        }

        let si = &*info;
        let code = si.si_code;

        // dbg!(sig);
        if sig == SIGBUS {
            write_str("SIGBUS: Bus error (bad alignment or missing page). si_code = ");
            write_int(code);
            libc::exit(1);
        }

        match code {
            SEGV_PKUERR => {
                write_str("SIGSEGV: MPK violation\n");
                dbg!((*info).si_addr());
            }
            SEGV_ACCERR => write_str("SIGSEGV: protection fault (mprotect / permissions)\n"),
            SEGV_MAPERR => write_str("SIGSEGV: address not mapped\n"),
            _ => {
                write_str("SIGSEGV: unknown cause, si_code = ");
                write_int(code);
            }
        }

        libc::exit(1);
    }
}

pub fn install_segv_handler() {
    unsafe {
        // 1. Setup an alternate signal stack.
        // This ensures the kernel can deliver the signal even if the
        // thread's normal stack is protected by an MPK or corrupted.
        let stack_size = libc::MINSIGSTKSZ + 8192;
        let stack_ptr = libc::mmap(
            ptr::null_mut(),
            stack_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );

        if stack_ptr == libc::MAP_FAILED {
            log::error!("Failed to allocate memory for signal stack");
            libc::exit(1);
        }

        let ss = libc::stack_t {
            ss_sp: stack_ptr,
            ss_flags: 0,
            ss_size: stack_size,
        };
        // register the alternate stack
        libc::sigaltstack(&ss, ptr::null_mut());

        // 2. Install the signal handler
        let mut sa: sigaction = zeroed();

        // CRITICAL: Add libc::SA_ONSTACK to use the memory we just mapped!
        sa.sa_flags = SA_SIGINFO | libc::SA_ONSTACK;
        sa.sa_sigaction = segv_handler as usize;

        libc::sigemptyset(&mut sa.sa_mask);

        // Catch both SEGV and BUS errors
        if sigaction(SIGSEGV, &sa, ptr::null_mut()) != 0 {
            libc::exit(1);
        }
        if sigaction(SIGBUS, &sa, ptr::null_mut()) != 0 {
            libc::exit(1);
        }

        log::info!("Custom SIGSEGV/SIGBUS handler installed with alternate stack");
    }
}
