use chrono::naive::NaiveDateTime;
use slug::slugify;

#[derive(Debug)]
pub struct Article {
    id: i32,
    slug: String,
    description: String,
    body: String,
    created_at: NaiveDateTime,
    updated_at: Option<NaiveDateTime>,
    author: i32,
}

impl Article {
    pub fn create() -> Self {
        unimplemented!();
    }

    pub fn get() -> std::io::Result<Option<Self>> {
        unimplemented!();
    }

    pub fn update(&mut self) -> std::io::Result<()> {
        unimplemented!();
    }

    pub fn delete(&mut self) -> std::io::Result<()> {
        unimplemented!();
    }
}
