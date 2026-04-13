use eyre::Result;
use tokio::runtime;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn main() -> Result<()> {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(STACK_SIZE)
        .build()?;

    rt.block_on(app::run())
}
