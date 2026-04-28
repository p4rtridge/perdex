pub mod serde;

pub trait RdfNode {
    /// Returns the RDF identifier of this node, if it has one.
    fn id(&self) -> Option<&str>;
}
