use anyhow::Result;
use tokio::runtime::Runtime;

/// Manually create a tokio runtime
pub fn create_runtime() -> Result<Runtime> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("tokio-chat-worker")
        // TODO: loom?
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack for deep recursion
        // TODO: enable specific features
        .enable_all()
        .build()?;
    Ok(runtime)
}
