pub trait Repository {
    fn insert_one(&self);
    fn insert_many(&self);
    fn find_one(&self);
    fn find_many(&self);
    fn update_one(&self);
    fn update_many(&self);
    fn delete_one(&self);
    fn delete_many(&self);
}
