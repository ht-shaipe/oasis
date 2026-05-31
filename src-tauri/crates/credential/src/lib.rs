pub mod commands;
pub mod crypto;
pub mod db;
pub mod models;

#[macro_export]
macro_rules! credential_handlers {
	() => {
		oasis_credential::commands::is_master_key_set,
		oasis_credential::commands::setup_master_key,
		oasis_credential::commands::verify_master_key,
		oasis_credential::commands::list_categories,
		oasis_credential::commands::create_category,
		oasis_credential::commands::delete_category,
		oasis_credential::commands::list_credentials,
		oasis_credential::commands::get_credential,
		oasis_credential::commands::create_credential,
		oasis_credential::commands::update_credential,
		oasis_credential::commands::delete_credential,
		oasis_credential::commands::change_master_key
	};
}
