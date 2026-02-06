pub mod error;
/// Module API - Contient tous les types publics exposés par l'API
/// Ces types peuvent être réutilisés dans une bibliothèque cliente
pub mod requests;
pub mod responses;
pub mod result;

// Re-exports pour faciliter l'utilisation
pub use error::ErrorResponse;
pub use requests::*;
pub use responses::*;
pub use result::AppResponse;

// Type alias optionnel pour les résultats des handlers
#[allow(unused_imports)]
pub use result::AppResult;
