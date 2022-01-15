use uuid::Uuid;

pub fn generate_container_id() -> Uuid {
    Uuid::new_v4()
}

pub fn generate_image_id() -> String {
    format!("polis-{}", &Uuid::new_v4().to_string()[..8])
}
