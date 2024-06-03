#[derive(Debug)]
pub enum ProgramError {
    ArgumentMissing,
    PathMissing,
    InvalidAmountOfArguments,
    InvalidFilePath,
    InvalidFileFormat,
    ErrorWhileReadingFile,
}

impl ProgramError {
    /// Returns the error message for the ProgramError
    ///
    /// # Returns
    ///
    /// * &str - The error message
    ///
    /// # Examples
    ///
    /// ```
    /// use rgrep::program_error::*;
    ///
    /// let error = ProgramError::ArgumentMissing;
    ///
    /// assert_eq!(error.message(), "Invalid arguments: regex and path missing");
    /// ```
    ///
    pub fn message(&self) -> &str {
        match self {
            // Arguments Errors
            ProgramError::ArgumentMissing => "Invalid arguments: regex and path missing",
            ProgramError::PathMissing => "Invalid arguments: path missing",
            ProgramError::InvalidAmountOfArguments => "Invalid amount of arguments",
            // File Reading Errors
            ProgramError::InvalidFilePath => "Invalid file path",
            ProgramError::InvalidFileFormat => "Invalid file format",
            ProgramError::ErrorWhileReadingFile => "An error occurred while reading file",
        }
    }
}
