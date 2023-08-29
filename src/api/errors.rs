use warp::hyper::StatusCode;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: StatusCode,
    pub message: ApiErrorType,
    pub id: String,
    pub route: String,
}
#[derive(Debug, Clone)]
pub enum ApiErrorType {
    Generic(String),
    InvalidSignature,
    DBInsertionFailed,
    CacheInsertionFailed,
    CuckooFilterInsertionFailed,
    CuckooFilterLookupFailed
}

impl ApiError {
    pub fn new(code: StatusCode, message: ApiErrorType, id: String, route: String) -> Self {
        ApiError {
            code,
            message,
            id,
            route,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl warp::reject::Reject for ApiError {}

impl std::fmt::Display for ApiErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            ApiErrorType::Generic(message) => write!(f, "Generic error: {message}"),
            ApiErrorType::InvalidSignature => write!(f, "Invalid signature"),
            ApiErrorType::DBInsertionFailed => write!(f, "DB insertion failed"),
            ApiErrorType::CacheInsertionFailed => write!(f, "Cache insertion failed"),
            ApiErrorType::CuckooFilterInsertionFailed => write!(f, "Cuckoo filter insertion failed"),
            ApiErrorType::CuckooFilterLookupFailed => write!(f, "Cuckoo filter lookup failed")
        }
    }
}