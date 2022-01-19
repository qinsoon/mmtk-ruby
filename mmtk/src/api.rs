// All functions here are extern function. There is no point for marking them as unsafe.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use libc::c_char;
use mmtk::util::constants::MIN_OBJECT_SIZE;
use mmtk::util::{VMWorkerThread, VMMutatorThread, VMThread};
use std::ffi::CStr;
use mmtk::memory_manager;
use mmtk::AllocationSemantics;
use mmtk::util::{ObjectReference, Address};
use mmtk::scheduler::GCWorker;
use mmtk::Mutator;
use mmtk::MMTK;
use crate::Ruby;
use crate::SINGLETON;
use crate::abi;

#[no_mangle]
pub extern "C" fn mmtk_init_binding(heap_size: usize, upcalls: *const abi::RubyUpcalls) {
    // # Safety
    // Casting `SINGLETON` as mutable is safe because `gc_init` will only be executed once by a single thread during startup.
    #[allow(clippy::cast_ref_to_mut)]
    let singleton_mut = unsafe { &mut *(&*SINGLETON as *const MMTK<Ruby> as *mut MMTK<Ruby>) };
    memory_manager::gc_init(singleton_mut, heap_size);

    unsafe {
        crate::UPCALLS = upcalls;
    }
}

#[no_mangle]
pub extern "C" fn mmtk_start_control_collector(tls: VMWorkerThread) {
    memory_manager::start_control_collector(&SINGLETON, tls);
}

#[no_mangle]
pub extern "C" fn mmtk_bind_mutator(tls: VMMutatorThread) -> *mut Mutator<Ruby> {
    Box::into_raw(memory_manager::bind_mutator(&SINGLETON, tls))
}

#[no_mangle]
pub extern "C" fn mmtk_destroy_mutator(mutator: *mut Mutator<Ruby>) {
    memory_manager::destroy_mutator(unsafe { Box::from_raw(mutator) })
}

#[no_mangle]
pub extern "C" fn mmtk_alloc(mutator: *mut Mutator<Ruby>, size: usize,
                    align: usize, offset: isize, semantics: AllocationSemantics) -> Address {
    let clamped_size = size.max(MIN_OBJECT_SIZE);
    memory_manager::alloc::<Ruby>(unsafe { &mut *mutator }, clamped_size, align, offset, semantics)
}

#[no_mangle]
pub extern "C" fn mmtk_post_alloc(mutator: *mut Mutator<Ruby>, refer: ObjectReference,
                                        bytes: usize, semantics: AllocationSemantics) {
    memory_manager::post_alloc::<Ruby>(unsafe { &mut *mutator }, refer, bytes, semantics)
}

#[no_mangle]
pub extern "C" fn mmtk_will_never_move(object: ObjectReference) -> bool {
    !object.is_movable()
}

#[no_mangle]
pub extern "C" fn mmtk_start_worker(tls: VMWorkerThread, worker: &'static mut GCWorker<Ruby>, mmtk: &'static MMTK<Ruby>) {
    memory_manager::start_worker::<Ruby>(tls, worker, mmtk)
}

#[no_mangle]
pub extern "C" fn mmtk_initialize_collection(tls: VMThread) {
    memory_manager::initialize_collection(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn mmtk_enable_collection() {
    memory_manager::enable_collection(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_used_bytes() -> usize {
    memory_manager::used_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_free_bytes() -> usize {
    memory_manager::free_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_total_bytes() -> usize {
    memory_manager::total_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_is_live_object(object: ObjectReference) -> bool{
    object.is_live()
}

#[no_mangle]
pub extern "C" fn mmtk_is_mapped_object(object: ObjectReference) -> bool {
    object.is_mapped()
}

#[no_mangle]
pub extern "C" fn mmtk_is_mapped_address(address: Address) -> bool {
    address.is_mapped()
}

#[no_mangle]
pub extern "C" fn mmtk_modify_check(object: ObjectReference) {
    memory_manager::modify_check(&SINGLETON, object)
}

#[no_mangle]
pub extern "C" fn mmtk_handle_user_collection_request(tls: VMMutatorThread) {
    memory_manager::handle_user_collection_request::<Ruby>(&SINGLETON, tls);
}

#[no_mangle]
pub extern "C" fn mmtk_add_weak_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_weak_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn mmtk_add_soft_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_soft_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn mmtk_add_phantom_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_phantom_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn mmtk_harness_begin(tls: VMMutatorThread) {
    memory_manager::harness_begin(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn mmtk_harness_end(_tls: VMMutatorThread) {
    memory_manager::harness_end(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_process(name: *const c_char, value: *const c_char) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    memory_manager::process(&SINGLETON, name_str.to_str().unwrap(), value_str.to_str().unwrap())
}

#[no_mangle]
pub extern "C" fn mmtk_starting_heap_address() -> Address {
    memory_manager::starting_heap_address()
}

#[no_mangle]
pub extern "C" fn mmtk_last_heap_address() -> Address {
    memory_manager::last_heap_address()
}

#[no_mangle]
pub extern "C" fn mmtk_register_finalizable(reff: ObjectReference) {
    crate::binding().finalizer_processor.register_finalizable(reff);
}

#[no_mangle]
pub extern "C" fn mmtk_poll_finalizable(include_live: bool) -> ObjectReference {
    crate::binding().finalizer_processor.poll_finalizable(include_live).unwrap_or_else(|| {
        unsafe { Address::zero().to_object_reference() }
    })
}
