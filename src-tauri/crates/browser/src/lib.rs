pub mod cdp_launcher;
pub mod commands;

#[macro_export]
macro_rules! browser_handlers {
	() => {
		oasis_browser::commands::find_chrome_path,
		oasis_browser::commands::launch_chrome_cdp
	};
}
