// Security module for path validation and access control
//
// This module provides utilities to ensure that file system operations
// are restricted to configured safe directories, preventing path traversal
// attacks and unauthorized access.

pub mod path_validator;

pub use path_validator::{validate_path, PathSecurityError};
