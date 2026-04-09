use crate::context::Context;
use crate::module::Module;
use crate::memory_buffer::MemoryBuffer;
use std::thread;

/// A piece of work to be executed in an isolated LLVM context.
pub trait CompilerJob: Send + 'static {
    fn execute<'ctx>(self: Box<Self>, context: &'ctx Context) -> Module<'ctx>;
}

/// A pool designed for compiling isolated closures inside LLVM without violating thread-safety rules.
#[derive(Debug)]
pub struct CompilerPool;

impl CompilerPool {
    /// Executes a list of tasks concurrently across multiple threads.
    /// Each task receives its own isolated `LLVMContext` and returns a `Module`.
    /// The modules are serialized to Bitcode as pure bytes (`Vec<u8>`), sent back across the thread boundary,
    /// and parsed/linked perfectly back into the `master_module`.
    pub fn execute_and_link<'ctx>(
        master_context: &'ctx Context,
        master_module: &Module<'ctx>,
        jobs: Vec<Box<dyn CompilerJob>>,
    ) -> Result<(), String> {
        let max_concurrency = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        // Process jobs in bounded batches to avoid OS thread exhaustion
        let mut all_bytes = Vec::with_capacity(jobs.len());
        
        let mut pending_jobs: Vec<Box<dyn CompilerJob>> = jobs.into_iter().collect();

        while !pending_jobs.is_empty() {
            let batch_size = std::cmp::min(pending_jobs.len(), max_concurrency);
            let mut handles = Vec::with_capacity(batch_size);

            for job in pending_jobs.drain(..batch_size) {
                let handle = thread::spawn(move || {
                    let local_context = Context::create();
                    let local_module = job.execute(&local_context);

                    // Serialize the LLVM module to bitcode and copy it into a standard safe `Vec<u8>`.
                    // The byte buffer is Send-able across threads without worrying about LLVM object ownership.
                    let mem_buffer = local_module.write_bitcode_to_memory();
                    mem_buffer.as_slice().to_vec()
                });
                handles.push(handle);
            }

            for handle in handles {
                match handle.join() {
                    Ok(byte_code) => {
                        all_bytes.push(byte_code);
                    }
                    Err(_) => {
                        return Err("A thread in the CompilerPool unexpectedly panicked.".to_string());
                    }
                }
            }
        }

        // Link all successfully serialized modules into the master module atomically.
        for byte_code in all_bytes {
            // Turn bytes back to MemoryBuffer in main thread Context.
            // create_from_memory_range_copy assumes trailing NUL byte exists, which as_slice() includes!
            let mem_buffer = MemoryBuffer::create_from_memory_range_copy(&byte_code, "concurrent_module");
            
            let parsed_module = Module::parse_bitcode_from_buffer(&mem_buffer, master_context)
                .map_err(|e| e.to_string())?;
                
            master_module.link_in_module(parsed_module).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
