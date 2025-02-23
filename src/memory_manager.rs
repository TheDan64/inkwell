use llvm_sys::prelude::LLVMBool;

/// A trait for user-defined memory management in MCJIT.
///
/// Implementors can override how LLVM's MCJIT engine allocates memory for code
/// and data sections. This is sometimes needed for:
/// - custom allocators,
/// - sandboxed or restricted environments,
/// - capturing stack map sections (e.g., for garbage collection),
/// - or other specialized JIT memory management requirements.
///
/// # StackMap and GC Integration
///
/// By examining the `section_name` argument in [`McjitMemoryManager::allocate_data_section`],
/// you can detect sections such as `.llvm_stackmaps` (on ELF) or `__llvm_stackmaps`
/// (on Mach-O). Recording the location of these sections may be useful for
/// custom garbage collectors. For more information, refer to the [LLVM
/// StackMaps documentation](https://llvm.org/docs/StackMaps.html#stack-map-section).
///
/// Typically, on Darwin (Mach-O), the stack map section name is `__llvm_stackmaps`,
/// and on Linux (ELF), it is `.llvm_stackmaps`.
pub trait McjitMemoryManager: std::fmt::Debug {
    /// Allocates a block of memory for a code section.
    ///
    /// # Parameters
    ///
    /// * `size` - The size in bytes for the code section.
    /// * `alignment` - The required alignment in bytes.
    /// * `section_id` - A numeric ID that LLVM uses to identify this section.
    /// * `section_name` - A name for this section, if provided by LLVM.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the allocated memory. Implementors must ensure it is
    /// at least `size` bytes long and meets `alignment` requirements.
    fn allocate_code_section(
        &mut self,
        size: libc::uintptr_t,
        alignment: libc::c_uint,
        section_id: libc::c_uint,
        section_name: &str,
    ) -> *mut u8;

    /// Allocates a block of memory for a data section.
    ///
    /// # Parameters
    ///
    /// * `size` - The size in bytes for the data section.
    /// * `alignment` - The required alignment in bytes.
    /// * `section_id` - A numeric ID that LLVM uses to identify this section.
    /// * `section_name` - A name for this section, if provided by LLVM.
    /// * `is_read_only` - Whether this data section should be read-only.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the allocated memory. Implementors must ensure it is
    /// at least `size` bytes long and meets `alignment` requirements.
    fn allocate_data_section(
        &mut self,
        size: libc::uintptr_t,
        alignment: libc::c_uint,
        section_id: libc::c_uint,
        section_name: &str,
        is_read_only: bool,
    ) -> *mut u8;

    /// Finalizes memory permissions for all allocated sections.
    ///
    /// This is called once all sections have been allocated. Implementors can set
    /// permissions such as making code sections executable or data sections
    /// read-only.
    ///
    /// # Errors
    ///
    /// If any error occurs (for example, failing to set page permissions),
    /// return an `Err(String)`. This error is reported back to LLVM as a C string.
    fn finalize_memory(&mut self) -> Result<(), String>;

    /// Cleans up or deallocates resources before the memory manager is destroyed.
    ///
    /// This is called when LLVM has finished using the memory manager. Any
    /// additional allocations or references should be released here if needed.
    fn destroy(&mut self);
}

/// Holds a boxed `McjitMemoryManager` and passes it to LLVM as an opaque pointer.
///
/// LLVM calls into the adapter using the extern "C" function pointers defined below.
#[derive(Debug)]
pub struct MemoryManagerAdapter {
    pub memory_manager: Box<dyn McjitMemoryManager>,
}

// ------ Extern "C" Adapters ------

/// Adapter for `allocate_code_section`.
///
/// Called by LLVM with a raw pointer (`opaque`). Casts back to `MemoryManagerAdapter`
/// and delegates to `allocate_code_section`.
pub(crate) extern "C" fn allocate_code_section_adapter(
    opaque: *mut libc::c_void,
    size: libc::uintptr_t,
    alignment: libc::c_uint,
    section_id: libc::c_uint,
    section_name: *const libc::c_char,
) -> *mut u8 {
    let adapter = unsafe { &mut *(opaque as *mut MemoryManagerAdapter) };
    let sname = unsafe { c_str_to_str(section_name) };
    adapter
        .memory_manager
        .allocate_code_section(size, alignment, section_id, sname)
}

/// Adapter for `allocate_data_section`.
///
/// Note that `LLVMBool` is `0` for false, and `1` for true. We check `!= 0` to
/// interpret it as a bool.
pub(crate) extern "C" fn allocate_data_section_adapter(
    opaque: *mut libc::c_void,
    size: libc::uintptr_t,
    alignment: libc::c_uint,
    section_id: libc::c_uint,
    section_name: *const libc::c_char,
    is_read_only: LLVMBool,
) -> *mut u8 {
    let adapter = unsafe { &mut *(opaque as *mut MemoryManagerAdapter) };
    let sname = unsafe { c_str_to_str(section_name) };
    adapter
        .memory_manager
        .allocate_data_section(size, alignment, section_id, sname, is_read_only != 0)
}

/// Adapter for `finalize_memory`.
///
/// If an error is returned, the message is converted into a C string and set in `err_msg_out`.
pub(crate) extern "C" fn finalize_memory_adapter(
    opaque: *mut libc::c_void,
    err_msg_out: *mut *mut libc::c_char,
) -> libc::c_int {
    let adapter = unsafe { &mut *(opaque as *mut MemoryManagerAdapter) };
    match adapter.memory_manager.finalize_memory() {
        Ok(()) => 0,
        Err(e) => {
            let cstring = std::ffi::CString::new(e).unwrap_or_default();
            unsafe {
                *err_msg_out = cstring.into_raw();
            }
            1
        },
    }
}

/// Adapter for `destroy`.
///
/// Called when LLVM is done with the memory manager. Calls `destroy` and drops
/// the adapter to free resources.
pub(crate) extern "C" fn destroy_adapter(opaque: *mut libc::c_void) {
    // Re-box to drop the adapter and its contents.
    // SAFETY: `opaque` must have been allocated by Box<MemoryManagerAdapter>.
    let mut adapter = unsafe { Box::from_raw(opaque as *mut MemoryManagerAdapter) };

    // Clean up user-defined resources
    adapter.memory_manager.destroy();

    // Dropping `adapter` automatically frees the memory
}

/// Converts a raw C string pointer to a Rust `&str`.
///
/// # Safety
///
/// The caller must ensure `ptr` points to a valid, null-terminated UTF-8 string.
/// If the string is invalid UTF-8 or `ptr` is null, an empty string is returned.
unsafe fn c_str_to_str<'a>(ptr: *const libc::c_char) -> &'a str {
    if ptr.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap_or("")
    }
}
