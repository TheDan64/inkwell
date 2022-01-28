use std::{
    error::Error,
    ffi::CStr,
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

use either::Either;
use libc::c_char;
#[llvm_versions(12.0..=latest)]
use llvm_sys::error::LLVMCreateStringError;
use llvm_sys::error::{
    LLVMConsumeError, LLVMDisposeErrorMessage, LLVMErrorRef, LLVMErrorTypeId, LLVMGetErrorMessage,
    LLVMGetErrorTypeId,
};

#[llvm_versions(12.0..=latest)]
use crate::orc2::lljit;
use crate::support::to_c_str;

/// An LLVM Error.
#[derive(Debug)]
pub struct LLVMError {
    error: LLVMErrorRef,
    handled: bool,
}

impl LLVMError {
    pub(crate) fn new(error: LLVMErrorRef) -> Result<(), Self> {
        if error.is_null() {
            return Ok(());
        }
        Err(LLVMError {
            error,
            handled: false,
        })
    }
    // Null type id == success
    pub fn get_type_id(&self) -> LLVMErrorTypeId {
        // FIXME: Don't expose LLVMErrorTypeId
        unsafe { LLVMGetErrorTypeId(self.error) }
    }

    /// Returns the error message of the error. This consumes the error
    /// and makes the error unusable afterwards.
    /// ```
    /// # #[cfg(not(feature = "llvm11-0"))] {
    /// use std::ffi::{CString, CStr};
    /// use inkwell::error::LLVMError;
    ///
    /// let error = LLVMError::new_string_error("llvm error");
    /// assert_eq!(*error.get_message(), *CString::new("llvm error").unwrap().as_c_str());
    /// # }
    /// ```
    pub fn get_message(mut self) -> LLVMErrorMessage {
        self.handled = true;
        unsafe { LLVMErrorMessage::new(LLVMGetErrorMessage(self.error)) }
    }

    /// Creates a new StringError with the given message.
    /// ```
    /// use inkwell::error::LLVMError;
    ///
    /// let error = LLVMError::new_string_error("string error");
    /// ```
    #[llvm_versions(12.0..=latest)]
    pub fn new_string_error(message: &str) -> Self {
        let error = unsafe { LLVMCreateStringError(to_c_str(message).as_ptr()) };
        LLVMError {
            error,
            handled: false,
        }
    }
}

impl Drop for LLVMError {
    fn drop(&mut self) {
        if !self.handled {
            unsafe { LLVMConsumeError(self.error) }
        }
    }
}

#[llvm_versions(12.0..=latest)]
impl<S> From<Either<LLVMError, S>> for LLVMError
where S: ToString {
    fn from(err: Either<LLVMError, S>) -> Self {
        match err {
            Either::Left(err) => err,
            Either::Right(message) => LLVMError::new_string_error(&message.to_string()),
        }
    }
}

/// An owned LLVM Error Message.
#[derive(Eq)]
pub struct LLVMErrorMessage {
    pub(crate) ptr: *const c_char,
}

impl LLVMErrorMessage {
    pub(crate) unsafe fn new(ptr: *const c_char) -> Self {
        LLVMErrorMessage { ptr }
    }

    /// This is a convenience method for creating a Rust `String`,
    /// however; it *will* reallocate. `LLVMErrorMessage` should be used
    /// as much as possible to save memory since it is allocated by
    /// LLVM. It's essentially a `CString` with a custom LLVM
    /// deallocator
    /// ```
    /// # #[cfg(not(feature = "llvm11-0"))] {
    /// use inkwell::error::{LLVMError, LLVMErrorMessage};
    ///
    /// let error = LLVMError::new_string_error("error");
    /// let error_msg = error.get_message().to_string();
    /// assert_eq!(error_msg, "error");
    /// # }
    /// ```
    /// The example does not work on LLVM version 11.
    pub fn to_string(&self) -> String {
        (*self).to_string_lossy().into_owned()
    }
}

impl Deref for LLVMErrorMessage {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.ptr) }
    }
}

impl Debug for LLVMErrorMessage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl Display for LLVMErrorMessage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.deref().to_string_lossy())
    }
}

impl PartialEq for LLVMErrorMessage {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Error for LLVMErrorMessage {
    fn description(&self) -> &str {
        self.to_str()
            .expect("Could not convert LLVMErrorMessage to str (likely invalid unicode)")
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Drop for LLVMErrorMessage {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeErrorMessage(self.ptr as *mut _);
        }
    }
}
