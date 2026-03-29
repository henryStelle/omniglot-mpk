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

fn main() {
    env_logger::init();

    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        with_mpkrt_lib(brand, |lib, mut alloc, mut access| {
            log::debug!("hello");
            let ret = lib.add_ptr(1, 2, &mut alloc, &mut access);

            log::debug!("{:#?}", &ret);
            let temp = ret.unwrap();
            log::debug!("{:#?}", &temp);
            let ptr = temp.validate_ref().unwrap();
            log::debug!("{:#?}", &ptr);

            unsafe {
                dbg!(Box::from_raw(*ptr as *mut i32));
            }
        });
    });
}
