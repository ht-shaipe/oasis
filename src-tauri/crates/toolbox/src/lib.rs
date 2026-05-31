pub mod commands;
pub mod csv_convert;
pub mod csv_split;
pub mod csv_stats;
pub mod excel_move;
pub mod json_convert;
pub mod json_merge;
pub mod network_scan;

#[macro_export]
macro_rules! toolbox_handlers {
	() => {
		oasis_toolbox::commands::csv_scan_dir,
		oasis_toolbox::commands::csv_split_file,
		oasis_toolbox::commands::csv_convert_file,
		oasis_toolbox::commands::excel_move_preview,
		oasis_toolbox::commands::excel_move_apply,
		oasis_toolbox::commands::json_convert_file,
		oasis_toolbox::commands::json_convert_batch,
		oasis_toolbox::commands::json_merge_files,
		oasis_toolbox::commands::network_scan_ports
	};
}
