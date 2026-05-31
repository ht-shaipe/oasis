import type { Tool } from './types';

// ── Tool Icons ─────────────────────────────────────────────────────────────

export const TOOL_ICONS = {
    CSV_STATS: '/assets/icons/CsvStats.svg',
    CSV_SPLIT: '/assets/icons/CsvSplit.svg',
    CSV_CONVERT: '/assets/icons/CsvConvert.svg',
    EXCEL_MOVE: '/assets/icons/ExcelMove.svg',
    JSON_CONVERT: '/assets/icons/JsonConvert.svg',
    JSON_MERGE: '/assets/icons/JsonMerge.svg',
    NETWORK_SCAN: '/assets/icons/NetworkScan.svg',
} as const;

// ── Tool Definitions ───────────────────────────────────────────────────────

export const TOOLS: Tool[] = [
    { id: 'csv-stats', icon: TOOL_ICONS.CSV_STATS, labelKey: 'toolbox.csvStats' },
    { id: 'csv-split', icon: TOOL_ICONS.CSV_SPLIT, labelKey: 'toolbox.csvSplit' },
    { id: 'csv-convert', icon: TOOL_ICONS.CSV_CONVERT, labelKey: 'toolbox.formatConvert' },
    { id: 'excel-move', icon: TOOL_ICONS.EXCEL_MOVE, labelKey: 'toolbox.excelMove' },
    { id: 'json-convert', icon: TOOL_ICONS.JSON_CONVERT, labelKey: 'toolbox.jsonConvert' },
    { id: 'json-merge', icon: TOOL_ICONS.JSON_MERGE, labelKey: 'toolbox.jsonMerge' },
    { id: 'network-scan', icon: TOOL_ICONS.NETWORK_SCAN, labelKey: 'toolbox.networkScan' },
];

// ── Sidebar Configuration ───────────────────────────────────────────────────

export const SIDEBAR_CONFIG = {
    MIN_WIDTH: 140,
    MAX_WIDTH: 360,
    DEFAULT_WIDTH: 160,
    STORAGE_KEY: 'toolbox-sidebar-width',
} as const;

// ── Default Values ─────────────────────────────────────────────────────────

export const DEFAULTS = {
    CSV_SPLIT_PARTS: 2,
    CONVERT_FORMAT: 'json',
    JSON_OUTPUT_FORMAT: 'csv',
    EXCEL_SUFFIXES: '.pdf,.docx,.xlsx',
    SCAN_IP_RANGE: '192.168.1.1-254',
    SCAN_PORTS: '80,443,22,8080',
    SCAN_TIMEOUT: 1000,
} as const;
