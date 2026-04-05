pub mod chat;
pub mod lifecycle;
pub mod onboarding;
pub mod workspace;

// Re-exporting modules establishes a clear public API boundary 
// for main.rs while preventing monolith file structures.
pub use chat::*;
pub use lifecycle::*;
pub use onboarding::*;
pub use workspace::*;