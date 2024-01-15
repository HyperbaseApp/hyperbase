use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl InsertOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(Deserialize)]
pub struct FindOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl FindOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl UpdateOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct DeleteOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl DeleteOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct FindManyFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl FindManyFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}
