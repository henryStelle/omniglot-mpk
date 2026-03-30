use std::any::Any;
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

// pub unsafe fn with_mock_rt_lib<'a, ID: OGID + 'a, A: omniglot::rt::mock::MockRtAllocator, R>(
//     brand: ID,
//     allocator: A,
//     f: impl FnOnce(
//         LibAddRt<ID, omniglot::rt::mock::MockRt<ID, A>, omniglot::rt::mock::MockRt<ID, A>>,
//         AllocScope<
//             <omniglot::rt::mock::MockRt<ID, A> as omniglot::rt::OGRuntime>::AllocTracker<'a>,
//             ID,
//         >,
//         AccessScope<ID>,
//     ) -> R,
// ) -> R {
//     // This is unsafe, as it instantiates a runtime that can be used to run
//     // foreign functions without memory protection:
//     let (rt, alloc, access) =
//         unsafe { omniglot::rt::mock::MockRt::new(false, false, allocator, brand) };

//     // Create a "bound" runtime, which implements the LibOAdd API:
//     let bound_rt = LibAddRt::new(rt).unwrap();

//     // Run the provided closure:
//     f(bound_rt, alloc, access)
// }

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

/// Moves a value into a PKey-protected memory region.
/// This effectively "hands off" the data to the sandbox.
unsafe fn move_to_pkey_memory<T>(value: T, pkey: i32) -> *mut T {
    // 1. Allocate exactly one page of memory (usually 4KB)
    let page_size = libc::sysconf(libc::_SC_PAGESIZE) as usize;

    let ptr = libc::mmap(
        ptr::null_mut(),
        page_size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
    );

    if ptr == libc::MAP_FAILED {
        panic!("Failed to allocate memory for PKey wrapping");
    }

    // 2. Tag this specific page with the Sandbox PKey (e.g., PKey 3)
    if libc::syscall(
        libc::SYS_pkey_mprotect,
        ptr,
        page_size,
        libc::PROT_READ | libc::PROT_WRITE,
        pkey,
    ) != 0
    {
        panic!("Failed to set PKey on the allocated page");
    }

    // 3. Move the value into the new PKey-protected home
    let typed_ptr = ptr as *mut T;
    ptr::write(typed_ptr, value);

    typed_ptr
}

fn main() {
    env_logger::init();

    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        with_mpkrt_lib(brand, |lib, mut alloc, mut access| {
            install_segv_handler();
            // // 1. Create a Box on the DEFAULT heap (lives at 0x55...)
            // let mut host_box = Box::new(5);

            // // 2. Create memory on your "FOREIGN" heap (the one mapped with PKey 3)
            // // Use the move_to_pkey_memory helper we wrote earlier
            // unsafe {
            //     let foreign_ptr = move_to_pkey_memory(10i32, 3);

            //     log::debug!("Host Box Address: {:p}", host_box.as_ref());
            //     log::debug!("Foreign Ptr Address: {:p}", foreign_ptr);

            //     // TEST A: Pass the Host Box (Will likely fail with MAPERR)
            //     let aa =
            //         lib.evil_cannot_deref_ptr_add(host_box.as_mut(), 2, &mut alloc, &mut access);
            //     dbg!(*aa.unwrap().assume_valid());

            //     // TEST B: Pass the Foreign Ptr (Should succeed with your WRPKRU C-code)
            //     let ret = lib.evil_cannot_deref_ptr_add(foreign_ptr, 2, &mut alloc, &mut access);

            //     let result_ptr = ret.unwrap().assume_valid();
            //     log::debug!("Success! Value is: {}", *result_ptr);
            // }
            // INSTEAD OF: let mut hmm = Box::new(5);
            // DO THIS:
            // unsafe {
            //     // unsafe fn set_pkru(mask: u32) {
            //     //     std::arch::asm!(
            //     //         "wrpkru",
            //     //         in("eax") mask,
            //     //         in("edx") 0,
            //     //         in("ecx") 0,
            //     //     );
            //     // }
            //     let pkey_ptr = move_to_pkey_memory(5i32, 3);

            //     // set_pkru(0x0); // This opens ALL keys (0-15) for testing.

            //     // 2. Call the function
            //     let ret = lib.cannot_deref_ptr_add(pkey_ptr, 2, &mut alloc, &mut access);

            //     // 3. RESTRICT PKey 3 again (Set bits 6 and 7 to 1)
            //     // set_pkru(0xC0);

            //     let a = ret.unwrap().assume_valid();
            //     write_str("Success! Result: ");
            //     write_int(*a);
            // }

            // 1. add
            // let ret = lib
            //     .add(1, 2, &mut alloc, &mut access)
            //     .unwrap()
            //     .validate()
            //     .unwrap();
            // dbg!(ret);

            // 2. ptr_add
            // unsafe {
            //     let ret = lib
            //         .add_ptr(1, 2, &mut alloc, &mut access)
            //         .unwrap()
            //         .assume_valid();
            //     println!("box_ptr: {:#?}", Box::from_raw(ret));
            // }

            // 3. segfaulting pass in mem
            // let mut rust_mem = Box::new(5);
            // log::debug!("Box address: {:p}", rust_mem.as_ref());
            // let ret = lib.cannot_deref_ptr_add(rust_mem.as_mut(), 2, &mut alloc, &mut access);

            // 5. evil add
            let mut rust_mem = Box::new(5);
            let ret = lib.evil_cannot_deref_ptr_add(rust_mem.as_mut(), 2, &mut alloc, &mut access);
            let ptr = ret.unwrap().validate().unwrap();
            unsafe {
                println!("Evil add result: {}", *ptr);
            }
        });
    });
}

use libc::{sigaction, siginfo_t, SA_SIGINFO, SIGBUS, SIGSEGV};
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

        if stack_ptr != libc::MAP_FAILED {
            let ss = libc::stack_t {
                ss_sp: stack_ptr,
                ss_flags: 0,
                ss_size: stack_size,
            };
            // register the alternate stack
            libc::sigaltstack(&ss, ptr::null_mut());
        }

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
    }
}
