pub mod serde;

pub trait RdfNode {
    fn id(&self) -> Option<&str>;
}
