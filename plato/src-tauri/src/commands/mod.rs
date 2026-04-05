/**
 * Commands Module Map
 * Explicitly exposes sub-modules within the commands directory.
 * This ensures the main application entry point can resolve the 
 * workspace and onboarding logic across the IPC bridge.
 */
pub mod workspace;
pub mod onboarding;