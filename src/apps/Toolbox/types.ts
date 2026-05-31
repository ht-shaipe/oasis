// ── Tool Types ─────────────────────────────────────────────────────────────

export interface Tool {
    id: string;
    icon: string;
    labelKey: string;
}

export interface ToolState {
    activeTool: string;
    sidebarWidth: number;
    isDragging: boolean;
}

// ── CSV Stats Types ───────────────────────────────────────────────────────

export interface CsvStatsResult {
    entries: Array<{ path: string; lines: number }>;
    total: number;
}

// ── CSV Split Types ───────────────────────────────────────────────────────

export interface CsvSplitParams {
    inputPath: string;
    outputDir: string;
    parts: number;
}

// ── CSV Convert Types ─────────────────────────────────────────────────────

export interface ConvertParams {
    input_path: string;
    output_path: string;
    format: 'csv' | 'json' | 'sql';
}

// ── Excel Move Types ─────────────────────────────────────────────────────

export interface ExcelMoveParams {
    excelPath: string;
    colHeader: string;
    colIndex: number;
    inputDir: string;
    suffixes: string[];
    outputDir: string;
}

export interface ExcelPreviewItem {
    status: string;
    file_name: string;
    base: string;
}

export interface ExcelPreviewResult {
    found: number;
    total: number;
    missing: number;
    duplicate: number;
    items: ExcelPreviewItem[];
}

// ── JSON Convert Types ───────────────────────────────────────────────────

export interface JsonConvertParams {
    input_path: string;
    output_path: string;
    output_format: 'csv' | 'excel';
    json_path: string;
    fields: string[];
}

// ── JSON Merge Types ─────────────────────────────────────────────────────

export interface JsonMergeParams {
    inputDir: string;
    outputPath: string;
    jsonPath: string;
}

// ── Network Scan Types ───────────────────────────────────────────────────

export interface NetworkScanParams {
    ipRange: string;
    portsStr: string;
    timeoutMs: number;
    showClosed: boolean;
}

export interface NetworkScanResult {
    format_text: string;
}
