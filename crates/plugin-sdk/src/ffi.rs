//! FFI 辅助 — 动态库挂件的导出/导入符号定义
//!
//! 每个挂件 cdylib 导出两个符号：
//!   - `widget_manifest_json` → 函数，返回 `*const c_char`（清单 JSON 字符串）
//!   - `widget_factory`       → 函数指针 `WidgetFactoryFn`
//!
//! 宿主通过 `libloading` 加载后，读取这两个符号完成注册。

/// 视图工厂函数签名 - 返回函数指针而不是直接创建实体
///
/// 参数：无
/// 返回：函数指针，该函数接受 window 和 cx 参数并返回 AnyView
///
/// 关键改进：factory 返回一个函数指针，实际的 Entity 创建在调用端完成，
/// 避免 FFI 边界内的状态问题
pub type WidgetFactoryFn = unsafe extern "C" fn() -> WidgetCreateFn;

/// Widget 创建函数指针
///
/// 参数：window 和 app context 指针
/// 返回：AnyView
pub type WidgetCreateFn = unsafe extern "C" fn(*mut gpui::Window, *mut gpui::App) -> gpui::AnyView;

/// 导出符号名 — 清单 JSON 函数
pub const WIDGET_MANIFEST_SYMBOL: &[u8; 21] = b"widget_manifest_json\0";

/// 导出符号名 — 视图工厂函数
pub const WIDGET_FACTORY_SYMBOL: &[u8; 15] = b"widget_factory\0";

/// 简化版导出宏 — 在挂件 crate 的 `lib.rs` 中调用
///
/// ```ignore
/// oasis_export_widget_with_manifest!(CalculatorWidget, r#"{"id":"calculator",...}"#);
/// ```
#[macro_export]
macro_rules! oasis_export_widget_with_manifest {
    ($widget_ty:ty, $manifest_json:literal) => {
        /// 导出清单 JSON — 函数形式（避免 static 的 Sync 问题）
        #[unsafe(no_mangle)]
        pub extern "C" fn widget_manifest_json() -> *const std::ffi::c_char {
            static MANIFEST_JSON: &str = $manifest_json;
            MANIFEST_JSON.as_ptr() as *const std::ffi::c_char
        }

        /// 实际的 widget 创建函数（在调用端被调用）
        #[unsafe(no_mangle)]
        pub extern "C" fn widget_create_impl(
            window: *mut gpui::Window,
            app: *mut gpui::App,
        ) -> gpui::AnyView {
            unsafe {
                let cx = &mut *app;
                // 关键：Entity 创建在调用端的上下文中完成
                cx.new(|cx| <$widget_ty as $crate::Widget>::new(cx)).into()
            }
        }

        /// Factory 函数：返回创建函数的指针
        #[unsafe(no_mangle)]
        pub extern "C" fn widget_factory() -> WidgetCreateFn {
            widget_create_impl
        }
    };
}

/// 调用 dylib 工厂函数的助手
///
/// # Safety
/// 调用者必须确保：
/// - `factory` 是有效的函数指针
/// - `window` 和 `cx` 在调用期间保持有效
pub unsafe fn call_widget_factory(
    factory: WidgetFactoryFn,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui::AnyView {
    // 获取创建函数指针
    let create_fn = factory();
    // 在调用端的上下文中创建实体
    create_fn(window as *mut gpui::Window, cx as *mut gpui::App)
}

