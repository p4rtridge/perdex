use error_stack::Report;
use http::Uri;
use thiserror::Error;

pub mod serde;

pub trait RdfNode {
    /// Returns the RDF identifier of this node, if it has one.
    fn id(&self) -> Option<&str>;
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Authority of `@id` doesn't belong to the originating server")]
    InvalidAuthority,

    #[error("Failed to parse URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
}

/// Validates that the given RDF node has an `@id` that either has no authority or an authority different from the server's.
pub fn validate_rdf_node<T: RdfNode>(
    node: &T,
    server_authority: &str,
) -> Result<(), Report<ValidationError>> {
    if let Some(id) = node.id()
        && Uri::try_from(id)
            .map_err(ValidationError::InvalidUri)?
            .authority()
            .is_none_or(|node_authority| node_authority != server_authority)
    {
        Err(Report::new(ValidationError::InvalidAuthority))
    } else {
        Ok(())
    }
}
