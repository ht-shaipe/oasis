//! 工具箱首页：按分组展示工具入口。

use gpui::{
    Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled, Window,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    label::Label,
    scroll::ScrollableElement,
    v_flex,
};
use rust_i18n::t;

use super::{ToolId, ToolboxPanel};
use crate::app::app_state::AppSettings;

fn tool_tile(
    id: &'static str,
    icon: IconName,
    label: impl Into<String>,
    icon_color: gpui::Hsla,
    text_color: gpui::Hsla,
    muted: gpui::Hsla,
    entity: Entity<ToolboxPanel>,
    tool: ToolId,
) -> Button {
    let label = label.into();
    let entity = entity.clone();
    Button::new(id)
        .large()
        .outline()
        .rounded_lg()
        .w(px(156.))
        .min_h(px(140.))
        .child(
            v_flex()
                .gap_3()
                .items_center()
                .justify_center()
                .py_4()
                .px_3()
                .w_full()
                .child(Icon::new(icon).with_size(px(44.)).text_color(icon_color))
                .child(
                    Label::new(label)
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(text_color)
                        .text_center()
                        .w_full(),
                )
                .child(
                    Label::new(t!("toolbox.home.card_hint").to_string())
                        .text_xs()
                        .text_color(muted)
                        .text_center()
                        .w_full(),
                ),
        )
        .on_click(move |_, _, cx| {
            let tool_id = match tool {
                super::ToolId::CsvStats => "csv_stats",
                super::ToolId::CsvSplit => "csv_split",
                super::ToolId::CsvExcelConvert => "csv_convert",
                super::ToolId::BatchRename => "batch_rename",
                super::ToolId::ExcelMoveFiles => "excel_move",
                super::ToolId::ApiRequest => "api_request",
                super::ToolId::ApiBatchDownload => "api_batch_download",
                super::ToolId::JsonToCsvExcel => "json_convert",
                super::ToolId::JsonMerge => "json_merge",
                super::ToolId::NetworkScan => "network_scan",
            };
            AppSettings::global_mut(cx).selected_tool = Some(tool_id.to_string());
        })
}

/// 首页：分组展示工具卡片，点击进入对应工具。
pub fn render_home(
    entity: Entity<ToolboxPanel>,
    _window: &mut Window,
    cx: &mut Context<ToolboxPanel>,
) -> impl IntoElement {
    let theme = cx.theme();

    let group_title_csv = t!("toolbox.groups.csv").to_string();
    let tool_csv_stats_label = t!("toolbox.tools.csv_stats").to_string();
    let tool_csv_split_label = t!("toolbox.tools.csv_split").to_string();
    let tool_csv_convert_label = t!("toolbox.tools.csv_convert").to_string();
    let tool_json_convert_label = t!("toolbox.tools.json_convert").to_string();

    let card_csv_stats = tool_tile(
        "tool-home-csv-stats",
        IconName::Folder,
        tool_csv_stats_label,
        theme.blue,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::CsvStats,
    );

    let card_csv_split = tool_tile(
        "tool-home-csv-split",
        IconName::File,
        tool_csv_split_label,
        theme.blue,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::CsvSplit,
    );

    let card_csv_convert = tool_tile(
        "tool-home-csv-convert",
        IconName::File,
        tool_csv_convert_label,
        theme.green,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::CsvExcelConvert,
    );

    let card_json_convert = tool_tile(
        "tool-home-json-convert",
        IconName::File,
        tool_json_convert_label,
        theme.green,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::JsonToCsvExcel,
    );

    let tool_json_merge_label = t!("toolbox.tools.json_merge").to_string();

    let card_json_merge = tool_tile(
        "tool-home-json-merge",
        IconName::File,
        tool_json_merge_label,
        theme.green,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::JsonMerge,
    );

    let group_title_files = t!("toolbox.groups.files").to_string();
    let tool_batch_rename_label = t!("toolbox.tools.batch_rename").to_string();
    let tool_excel_move_label = t!("toolbox.tools.excel_move_files").to_string();

    let card_batch_rename = tool_tile(
        "tool-home-batch-rename",
        IconName::File,
        tool_batch_rename_label,
        theme.yellow,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::BatchRename,
    );

    let card_excel_move = tool_tile(
        "tool-home-excel-move",
        IconName::Folder,
        tool_excel_move_label,
        theme.blue,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::ExcelMoveFiles,
    );

    let group_title_api = t!("toolbox.groups.api").to_string();
    let tool_api_request_label = t!("toolbox.tools.api_request").to_string();
    let tool_batch_download_label = "批量下载".to_string();
    let tool_network_scan_label = "网络扫描".to_string();

    let card_api_request = tool_tile(
        "tool-home-api-request",
        IconName::Globe,
        tool_api_request_label,
        theme.blue,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::ApiRequest,
    );

    let card_batch_download = tool_tile(
        "tool-home-api-batch-download",
        IconName::ArrowRight,
        tool_batch_download_label,
        theme.primary,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::ApiBatchDownload,
    );

    let card_network_scan = tool_tile(
        "tool-home-network-scan",
        IconName::Globe,
        tool_network_scan_label,
        theme.blue,
        theme.foreground,
        theme.muted_foreground,
        entity.clone(),
        ToolId::NetworkScan,
    );

    let group_csv = v_flex()
        .gap_3()
        .child(
            Label::new(group_title_csv)
                .text_base()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground),
        )
        .child(
            gpui::div()
                .w_full()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .p_5()
                .child(
                    h_flex()
                        .gap_4()
                        .flex_wrap()
                        .items_start()
                        .child(card_csv_stats)
                        .child(card_csv_split)
                        .child(card_csv_convert)
                        .child(card_json_convert)
                        .child(card_json_merge),
                ),
        );

    let group_files = v_flex()
        .gap_3()
        .child(
            Label::new(group_title_files)
                .text_base()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground),
        )
        .child(
            gpui::div()
                .w_full()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .p_5()
                .child(
                    h_flex()
                        .gap_4()
                        .flex_wrap()
                        .items_start()
                        .child(card_batch_rename)
                        .child(card_excel_move),
                ),
        );

    let group_api = v_flex()
        .gap_3()
        .child(
            Label::new(group_title_api)
                .text_base()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground),
        )
        .child(
            gpui::div()
                .w_full()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .p_5()
                .child(
                    h_flex()
                        .gap_4()
                        .flex_wrap()
                        .items_start()
                        .child(card_api_request)
                        .child(card_batch_download)
                        .child(card_network_scan),
                ),
        );

    v_flex()
        .size_full()
        .overflow_y_scrollbar()
        .gap_8()
        .p_6()
        .child(
            v_flex()
                .gap_2()
                .child(
                    Label::new(t!("toolbox.home.title").to_string())
                        .text_xl()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground),
                )
                .child(
                    Label::new(t!("toolbox.home.subtitle").to_string())
                        .text_sm()
                        .text_color(theme.muted_foreground),
                ),
        )
        .child(group_csv)
        .child(group_files)
        .child(group_api)
}
