pub mod chat;
pub mod lifecycle;
pub mod onboarding;
pub mod workspace;

// Re-exporting modules allows for a clean wildcard import in main.rs 
// while maintaining logical modularity.
pub use chat::*;
pub use lifecycle::*;
pub use onboarding::*;
pub use workspace::*;