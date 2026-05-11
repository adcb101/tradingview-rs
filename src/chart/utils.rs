use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub enum ExtendValue {
    Right,
    Left,
    Both,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub enum YLocValue {
    Price,
    AboveBar,
    BelowBar,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub enum LabelStyleValue {
    None,
    XCross,
    Cross,
    TriangleUp,
    TriangleDown,
    Flag,
    Circle,
    ArrowUp,
    ArrowDown,
    LabelUp,
    LabelDown,
    LabelLeft,
    LabelRight,
    LabelLowerLeft,
    LabelLowerRight,
    LabelUpperLeft,
    LabelUpperRight,
    LabelCenter,
    Square,
    Diamond,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub enum LineStyleValue {
    Solid,
    Dotted,
    Dashed,
    ArrowLeft,
    ArrowRight,
    ArrowBoth,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub enum BoxStyleValue {
    Solid,
    Dotted,
    Dashed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicLabel {
    pub id: u64,
    pub x: Option<f64>,
    pub y: f64,
    pub y_loc: YLocValue,
    pub text: String,
    pub style: LabelStyleValue,
    pub color: u32,
    pub text_color: u32,
    pub size: String,
    pub text_align: String,
    pub tool_tip: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct GraphicLine {
    pub id: u64,
    pub x1: Option<f64>,
    pub y1: f64,
    pub x2: Option<f64>,
    pub y2: f64,
    pub extend: ExtendValue,
    pub style: LineStyleValue,
    pub color: u32,
    pub width: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicBox {
    pub id: u64,
    pub x1: Option<f64>,
    pub y1: f64,
    pub x2: Option<f64>,
    pub y2: f64,
    pub color: u32,
    pub bg_color: u32,
    pub extend: ExtendValue,
    pub style: BoxStyleValue,
    pub width: f64,
    pub text: String,
    pub text_size: String,
    pub text_color: u32,
    pub text_v_align: String,
    pub text_h_align: String,
    pub text_wrap: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicData {
    pub labels: Vec<GraphicLabel>,
    pub lines: Vec<GraphicLine>,
    pub boxes: Vec<GraphicBox>,
    pub tables: Vec<GraphicTable>,
    pub polygons: Vec<GraphicPolygon>,
    pub horiz_lines: Vec<GraphicHorizline>,
    pub horiz_hists: Vec<GraphicHorizHist>,
}

/// A cell within a table graphic.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TableCell {
    pub id: u64,
    pub text: String,
    pub width: f64,
    pub height: f64,
    pub text_color: u32,
    pub text_h_align: String,
    pub text_v_align: String,
    pub text_size: String,
    pub bg_color: u32,
}

/// A table graphic drawing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicTable {
    pub id: u64,
    pub position: String,
    pub rows: u64,
    pub columns: u64,
    pub bg_color: u32,
    pub frame_color: u32,
    pub frame_width: f64,
    pub border_color: u32,
    pub border_width: f64,
}

impl GraphicTable {
    /// Builds the cell matrix for this table from raw cell data.
    pub fn build_cells(&self, raw_cells: &serde_json::Map<String, serde_json::Value>) -> Vec<Vec<TableCell>> {
        let mut matrix: Vec<Vec<TableCell>> = Vec::new();
        for cell_val in raw_cells.values() {
            let tid = cell_val.get("tid").and_then(|v| v.as_u64()).unwrap_or(0);
            if tid != self.id {
                continue;
            }
            let row = cell_val.get("row").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let col = cell_val.get("col").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            while matrix.len() <= row {
                matrix.push(Vec::new());
            }
            while matrix[row].len() <= col {
                matrix[row].push(TableCell {
                    id: 0,
                    text: String::new(),
                    width: 0.0,
                    height: 0.0,
                    text_color: 0,
                    text_h_align: String::new(),
                    text_v_align: String::new(),
                    text_size: String::new(),
                    bg_color: 0,
                });
            }
            matrix[row][col] = TableCell {
                id: cell_val.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                text: cell_val.get("t").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                width: cell_val.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0),
                height: cell_val.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                text_color: cell_val.get("tc").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                text_h_align: cell_val.get("tha").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                text_v_align: cell_val.get("tva").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                text_size: cell_val.get("ts").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                bg_color: cell_val.get("bgc").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            };
        }
        matrix
    }
}

/// A point on a polygon graphic.
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct GraphicPoint {
    pub index: Option<f64>,
    pub level: f64,
}

/// A polygon graphic drawing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicPolygon {
    pub id: u64,
    pub points: Vec<GraphicPoint>,
}

/// A horizontal line graphic drawing.
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct GraphicHorizline {
    pub id: u64,
    pub level: f64,
    pub start_index: Option<f64>,
    pub end_index: Option<f64>,
    pub extend_right: bool,
    pub extend_left: bool,
}

/// A horizontal histogram graphic drawing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GraphicHorizHist {
    pub id: u64,
    pub price_low: f64,
    pub price_high: f64,
    pub first_bar_time: Option<f64>,
    pub last_bar_time: Option<f64>,
    pub rate: Vec<f64>,
}

fn translate_extend(value: &str) -> ExtendValue {
    match value {
        "r" => ExtendValue::Right,
        "l" => ExtendValue::Left,
        "b" => ExtendValue::Both,
        "n" => ExtendValue::None,
        _ => ExtendValue::None,
    }
}

fn translate_y_loc(value: &str) -> YLocValue {
    match value {
        "pr" => YLocValue::Price,
        "ab" => YLocValue::AboveBar,
        "bl" => YLocValue::BelowBar,
        _ => YLocValue::Price,
    }
}

fn translate_label_style(value: &str) -> LabelStyleValue {
    match value {
        "n" => LabelStyleValue::None,
        "xcr" => LabelStyleValue::XCross,
        "cr" => LabelStyleValue::Cross,
        "tup" => LabelStyleValue::TriangleUp,
        "tdn" => LabelStyleValue::TriangleDown,
        "flg" => LabelStyleValue::Flag,
        "cir" => LabelStyleValue::Circle,
        "aup" => LabelStyleValue::ArrowUp,
        "adn" => LabelStyleValue::ArrowDown,
        "lup" => LabelStyleValue::LabelUp,
        "ldn" => LabelStyleValue::LabelDown,
        "llf" => LabelStyleValue::LabelLeft,
        "lrg" => LabelStyleValue::LabelRight,
        "llwlf" => LabelStyleValue::LabelLowerLeft,
        "llwrg" => LabelStyleValue::LabelLowerRight,
        "luplf" => LabelStyleValue::LabelUpperLeft,
        "luprg" => LabelStyleValue::LabelUpperRight,
        "lcn" => LabelStyleValue::LabelCenter,
        "sq" => LabelStyleValue::Square,
        "dia" => LabelStyleValue::Diamond,
        _ => LabelStyleValue::None,
    }
}

fn translate_line_style(value: &str) -> LineStyleValue {
    match value {
        "sol" => LineStyleValue::Solid,
        "dot" => LineStyleValue::Dotted,
        "dsh" => LineStyleValue::Dashed,
        "al" => LineStyleValue::ArrowLeft,
        "ar" => LineStyleValue::ArrowRight,
        "ab" => LineStyleValue::ArrowBoth,
        _ => LineStyleValue::Solid,
    }
}

fn translate_box_style(value: &str) -> BoxStyleValue {
    match value {
        "sol" => BoxStyleValue::Solid,
        "dot" => BoxStyleValue::Dotted,
        "dsh" => BoxStyleValue::Dashed,
        _ => BoxStyleValue::Solid,
    }
}

pub fn graphics_parser(data: &Value) -> GraphicData {
    let indexes = data
        .get("indexes")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    let raw_graphic = data.get("graphic").unwrap_or(&Value::Null);

    // Parse labels
    let labels = if let Some(dwglabels) = raw_graphic.get("dwglabels").and_then(|v| v.as_object()) {
        dwglabels
            .values()
            .filter_map(|l| {
                Some(GraphicLabel {
                    id: l.get("id")?.as_u64()?,
                    x: l.get("x")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    y: l.get("y")?.as_f64()?,
                    y_loc: translate_y_loc(l.get("yl")?.as_str()?),
                    text: l.get("t")?.as_str()?.to_string(),
                    style: translate_label_style(l.get("st")?.as_str()?),
                    color: l.get("ci")?.as_u64()? as u32,
                    text_color: l.get("tci")?.as_u64()? as u32,
                    size: l.get("sz")?.as_str()?.to_string(),
                    text_align: l.get("ta")?.as_str()?.to_string(),
                    tool_tip: l.get("tt")?.as_str()?.to_string(),
                })
            })
            .collect()
    } else {
        vec![]
    };

    // Parse lines
    let lines = if let Some(dwglines) = raw_graphic.get("dwglines").and_then(|v| v.as_object()) {
        dwglines
            .values()
            .filter_map(|l| {
                Some(GraphicLine {
                    id: l.get("id")?.as_u64()?,
                    x1: l
                        .get("x1")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    y1: l.get("y1")?.as_f64()?,
                    x2: l
                        .get("x2")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    y2: l.get("y2")?.as_f64()?,
                    extend: translate_extend(l.get("ex")?.as_str()?),
                    style: translate_line_style(l.get("st")?.as_str()?),
                    color: l.get("ci")?.as_u64()? as u32,
                    width: l.get("w")?.as_f64()?,
                })
            })
            .collect()
    } else {
        vec![]
    };

    // Parse boxes
    let boxes = if let Some(dwgboxes) = raw_graphic.get("dwgboxes").and_then(|v| v.as_object()) {
        dwgboxes
            .values()
            .filter_map(|b| {
                Some(GraphicBox {
                    id: b.get("id")?.as_u64()?,
                    x1: b
                        .get("x1")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    y1: b.get("y1")?.as_f64()?,
                    x2: b
                        .get("x2")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    y2: b.get("y2")?.as_f64()?,
                    color: b.get("c")?.as_u64()? as u32,
                    bg_color: b.get("bc")?.as_u64()? as u32,
                    extend: translate_extend(b.get("ex")?.as_str()?),
                    style: translate_box_style(b.get("st")?.as_str()?),
                    width: b.get("w")?.as_f64()?,
                    text: b.get("t")?.as_str()?.to_string(),
                    text_size: b.get("ts")?.as_str()?.to_string(),
                    text_color: b.get("tc")?.as_u64()? as u32,
                    text_v_align: b.get("tva")?.as_str()?.to_string(),
                    text_h_align: b.get("tha")?.as_str()?.to_string(),
                    text_wrap: b.get("tw")?.as_str()?.to_string(),
                })
            })
            .collect()
    } else {
        vec![]
    };

    // Parse tables
    let tables = if let Some(dwgtables) = raw_graphic.get("dwgtables").and_then(|v| v.as_object()) {
        dwgtables
            .values()
            .filter_map(|t| {
                let table = GraphicTable {
                    id: t.get("id")?.as_u64()?,
                    position: t.get("pos").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    rows: t.get("rows").and_then(|v| v.as_u64()).unwrap_or(0),
                    columns: t.get("cols").and_then(|v| v.as_u64()).unwrap_or(0),
                    bg_color: t.get("bgc").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    frame_color: t.get("frmc").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    frame_width: t.get("frmw").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    border_color: t.get("brdc").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    border_width: t.get("brdw").and_then(|v| v.as_f64()).unwrap_or(0.0),
                };
                Some(table)
            })
            .collect()
    } else {
        vec![]
    };

    // Parse polygons
    let polygons = if let Some(polygons_val) = raw_graphic.get("polygons").and_then(|v| v.as_object()) {
        polygons_val
            .values()
            .filter_map(|p| {
                Some(GraphicPolygon {
                    id: p.get("id")?.as_u64()?,
                    points: p
                        .get("points")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .map(|pt| GraphicPoint {
                                    index: pt
                                        .get("index")
                                        .and_then(|x| x.as_u64())
                                        .and_then(|idx| indexes.get(idx as usize))
                                        .and_then(|v| v.as_f64()),
                                    level: pt.get("level").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
            })
            .collect()
    } else {
        vec![]
    };

    // Parse horizontal lines
    let horiz_lines = if let Some(horiz_lines_val) = raw_graphic.get("horizlines").and_then(|v| v.as_object()) {
        horiz_lines_val
            .values()
            .filter_map(|h| {
                Some(GraphicHorizline {
                    id: h.get("id")?.as_u64()?,
                    level: h.get("level").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    start_index: h
                        .get("startIndex")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    end_index: h
                        .get("endIndex")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    extend_right: h.get("extendRight").and_then(|v| v.as_bool()).unwrap_or(false),
                    extend_left: h.get("extendLeft").and_then(|v| v.as_bool()).unwrap_or(false),
                })
            })
            .collect()
    } else {
        vec![]
    };

    // Parse horizontal histograms
    let horiz_hists = if let Some(hhists) = raw_graphic.get("hhists").and_then(|v| v.as_object()) {
        hhists
            .values()
            .filter_map(|h| {
                Some(GraphicHorizHist {
                    id: h.get("id")?.as_u64()?,
                    price_low: h.get("priceLow").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    price_high: h.get("priceHigh").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    first_bar_time: h
                        .get("firstBarTime")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    last_bar_time: h
                        .get("lastBarTime")
                        .and_then(|x| x.as_u64())
                        .and_then(|idx| indexes.get(idx as usize))
                        .and_then(|v| v.as_f64()),
                    rate: h
                        .get("rate")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
                        .unwrap_or_default(),
                })
            })
            .collect()
    } else {
        vec![]
    };

    GraphicData {
        labels,
        lines,
        boxes,
        tables,
        polygons,
        horiz_lines,
        horiz_hists,
    }
}
