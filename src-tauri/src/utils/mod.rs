pub mod db_manager;
pub mod repo_manager;

pub fn generate_cuid() -> String {
    cuid2::create_id()
}