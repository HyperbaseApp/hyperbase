use super::base::BaseModel;

pub trait AdminModel {
    fn table_name(&self) -> &str {
        "_admins"
    }

    fn base_model(&self) -> &dyn BaseModel;
    fn email(&self) -> &str;
    fn password_hash(&self) -> &str;
}
