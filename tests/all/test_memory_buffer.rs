use inkwell::memory_buffer::MemoryBuffer;

#[test]
fn test_memory_buffer() {
    let buffer = c"mem";
    let memory_buffer = MemoryBuffer::create_from_memory_range(buffer, "mem");

    assert_eq!(memory_buffer.as_slice().as_ptr(), buffer.as_ptr() as *const _);
    assert_eq!(memory_buffer.get_size(), 4);
}

#[test]
fn test_memory_buffer_copied() {
    let buffer = c"mem";
    let memory_buffer = MemoryBuffer::create_from_memory_range_copy(buffer, "mem");

    assert_ne!(memory_buffer.as_slice().as_ptr(), buffer.as_ptr() as *const _);
    assert_eq!(memory_buffer.get_size(), 4);
}
