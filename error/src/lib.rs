pub enum Error {
    BadRequest(String),
    Forbidden(String),
    InternalServerError(String),
}
