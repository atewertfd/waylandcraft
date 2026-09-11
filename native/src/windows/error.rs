use thiserror::Error;

#[derive(Debug, Error)]
pub enum WindowsError {
    #[error(transparent)]
    Jni(#[from] jni::errors::Error),
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error("{0}")]
    Null(&'static str),
    #[error("Null WLC instance handle given. Function: {0}")]
    NullInstancePtr(&'static str),
    #[error("Null window handle given. Function: {0}")]
    NullWindowPtr(&'static str),
    #[error("Width cannot be below 1")]
    NonPositiveWidth,
    #[error("Height cannot be below 1")]
    NonPositiveHeight,
    #[error("Unknown pointer button {0} received")]
    UnknownPointerButton(i32),
    #[error("Unknown scroll direction {0} received")]
    UnknownScrollDirection(i32),
    #[error("Unknown keyboard state {0} received")]
    UnknownKeyboardState(i32),
    #[error("{0}")]
    Message(String),
}

impl WindowsError {
    pub fn message(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }
}
