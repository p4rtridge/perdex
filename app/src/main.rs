use app::error::AppError;
use error_stack::{Report, ResultExt};
use tokio::runtime;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn main() -> Result<(), Report<AppError>> {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(STACK_SIZE)
        .build()
        .change_context(AppError)?;

    rt.block_on(app::run())
}
