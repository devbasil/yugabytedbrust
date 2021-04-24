use derive_more::{Display, Error};


#[non_exhaustive]
pub struct CustomStatusMessage;
impl CustomStatusMessage {
    /// DEFAULT User Request has failed required filter validation
    pub const USER_REQUEST_FAILED: &'static str = "FAILED";
    /// DEFAULT User Request has failed authenitcation
    pub const USER_REQUEST_DENIED: &'static str = "DENIED";
    /// DEFAULT User request was successfuly processed
    pub const USER_REQUEST_SUCCESS:  &'static str = "SUCCESS";   
}

#[non_exhaustive]
pub struct ServiceCustomCodes;
impl ServiceCustomCodes {
    /// DEFAULT jwt access denied code
    pub const JWT_DENIED_CODE: &'static str = "444-PS-DJWT";
    /// DEFAULT internal server error code for user reference
    pub const INTERNAL_SERVER_ERROR_CODE: &'static str = "444-PS-INT";
    /// Use this if you don't want to tell the user the nature of the error because of security concern
    pub const INTERNAL_MASKING_ERROR_CODE: &'static str = "444-PS-MSK";
    /// DEFAULT internal bad request error code for user reference
    pub const BAD_REQUEST_ERROR_CODE: &'static str = "444-PS-DBR";
    /// DEFAULT Unsupported Media Type error code for user reference
    pub const UNSUPPORTED_MEDIA_TYPE_ERROR_CODE: &'static str = "444-PS-UNSM";
}


#[derive(Debug, Display, Error)]
pub enum UserErrorMessages {
    /// DEFAULT internal server error message for user reference
    #[display(fmt = "Please try again later REFERENCE CODE: {}", ServiceCustomCodes::INTERNAL_SERVER_ERROR_CODE)]
    InternalServerError,
    /// DEFAULT Unsupported media request message for user reference
    #[display(fmt = "Unsupported media content REFERENCE CODE: {}", ServiceCustomCodes::UNSUPPORTED_MEDIA_TYPE_ERROR_CODE)]
    UnsupportedMediaType,
    /// DEFAULT bad request message for user reference
    #[display(fmt = "Bad request REFERENCE CODE: {}", ServiceCustomCodes::BAD_REQUEST_ERROR_CODE)]
    BadClientData,
    #[display(fmt = "timeout")]
    Timeout,
    ///DEAFULT user field input error
    #[display(fmt = "Input error for reference check: {}", reason)]
    InputError{reason: String},
    ///DEFAULT jwt access denied message
    #[display(fmt = "Access Denied, reference code: {}", ServiceCustomCodes::JWT_DENIED_CODE)]  
    JwtAccessTokenError,
    ///DEFAULT masking message: Use this message if the origin of the error is unclear
    #[display(fmt = "Process terminated, REFERENCE CODE: {}", ServiceCustomCodes::INTERNAL_MASKING_ERROR_CODE)]  
    ForMaskingError
}