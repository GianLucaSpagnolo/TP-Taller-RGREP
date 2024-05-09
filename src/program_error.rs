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
