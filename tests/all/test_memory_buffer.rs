use std::path::Path;

use inkwell::memory_buffer::MemoryBuffer;

#[test]
fn test_memory_buffer() {
    let buffer = b"mem";
    let memory_buffer = MemoryBuffer::create_from_memory_range(buffer, "mem_buf");

    assert_eq!(memory_buffer.as_slice(), buffer);
}

#[test]
fn test_memory_buffer_copied() {
    let buffer = b"mem";
    let memory_buffer_copied = MemoryBuffer::create_from_memory_range_copy(buffer, "mem_buf_copied");

    assert_ne!(memory_buffer_copied.as_slice().as_ptr(), buffer.as_ptr() as *const _);
    assert_eq!(memory_buffer_copied.as_slice(), buffer);

    let memory_buffer = MemoryBuffer::create_from_memory_range(memory_buffer_copied.as_slice(), "mem_buf");

    assert_eq!(memory_buffer_copied.as_slice(), memory_buffer.as_slice());
}

#[test]
fn test_memory_buffer_file() {
    let memory_buffer = MemoryBuffer::create_from_file(Path::new("./LICENSE")).unwrap();

    assert_eq!(memory_buffer.as_slice()[memory_buffer.get_size() - 1], b'\n'); // new line character
}
