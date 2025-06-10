use rlibdns::messages::inter::response_codes::ResponseCodes;

pub type ResponseResult<T> = Result<T, ResponseError>;

#[derive(Debug, Clone)]
pub struct ResponseError {
    code: ResponseCodes,
    message: String
}

impl ResponseError {

    pub fn new(code: ResponseCodes, message: &str) -> Self {
        Self {
            code,
            message: message.to_string()
        }
    }

    pub fn get_code(&self) -> ResponseCodes {
        self.code
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}
