//! haygeom - Geographic mapping plugin for Hayashi
//!
//! Provides functionality for rendering geographic maps from WKT geometry data.
//! Supports multiple layers (polygons, points, lines) with independent styling.

use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin};
use hayashi_plugin_sdk::arrow::array::{Array, ArrayRef, StructArray};
use hayashi_plugin_sdk::value::HayashiValue;
use std::collections::HashMap;

// Exposes dynamic library C ABI deallocation hooks
hayashi_plugin!();

mod wkt;
use wkt::{parse_wkt, Geometry};

/// Layer type for map rendering
#[derive(Debug, Clone)]
enum LayerType {
    Polygon,
    Point,
    Line,
}

/// Map layer configuration
#[derive(Debug, Clone)]
struct MapLayer {
    layer_type: LayerType,
    geometries: Vec<Geometry>,
    fill: String,
    stroke: String,
    stroke_width: f64,
    point_size: f64,
}

/// Create a new map with specified dimensions
#[hayashi_fn]
fn map_impl(
    width: f64,
    height: f64
) -> HashMap<String, HayashiValue> {
    let mut map_dict = HashMap::new();
    map_dict.insert("width".to_string(), HayashiValue::Float(width));
    map_dict.insert("height".to_string(), HayashiValue::Float(height));
    map_dict.insert("background".to_string(), HayashiValue::Str("white".to_string()));
    map_dict.insert("layers".to_string(), HayashiValue::List(vec![]));

    map_dict
}

/// Add a layer to the map
#[hayashi_fn]
fn add_layer_impl(
    mut map: HashMap<String, HayashiValue>,
    data: ArrayRef,
    options: HashMap<String, HayashiValue>
) -> Result<HashMap<String, HayashiValue>, String> {

    // Parse options
    let layer_type_str = options.get("type")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("polygon");

    let layer_type = match layer_type_str {
        "polygon" => LayerType::Polygon,
        "point" => LayerType::Point,
        "line" => LayerType::Line,
        _ => return Err(format!("unknown layer type: {}", layer_type_str)),
    };

    let fill = options.get("fill")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "#2D3E50".to_string());

    let stroke = options.get("stroke")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "none".to_string());

    let stroke_width = options.get("stroke_width")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(0.5);

    let point_size = options.get("size")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(5.0);

    // Parse geometries from data
    let struct_arr = data.as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| "DataFrame must be an Arrow StructArray".to_string())?;

    let geom_col_name = "geometry";
    let geom_col_idx = struct_arr.fields().iter()
        .position(|f| f.name() == geom_col_name)
        .ok_or_else(|| format!("Column '{}' not found in DataFrame", geom_col_name))?;

    let geom_strings = extract_column_string(struct_arr, geom_col_idx)?;

    let mut geometries = Vec::new();
    for wkt in &geom_strings {
        match parse_wkt(wkt) {
            Ok(g) => geometries.push(g),
            Err(e) => return Err(format!("Failed to parse WKT '{}': {}", wkt, e)),
        }
    }

    // Create layer
    let layer = MapLayer {
        layer_type,
        geometries,
        fill,
        stroke,
        stroke_width,
        point_size,
    };

    // Add to layers list
    let layers = map.get("layers")
        .and_then(|v| match v {
            HayashiValue::List(l) => Some(l.clone()),
            _ => None,
        })
        .unwrap_or_else(|| vec![]);

    let mut layer_dict = HashMap::new();
    layer_dict.insert("type".to_string(), HayashiValue::Str(layer_type_str.to_string()));
    layer_dict.insert("fill".to_string(), HayashiValue::Str(layer.fill.clone()));
    layer_dict.insert("stroke".to_string(), HayashiValue::Str(layer.stroke.clone()));
    layer_dict.insert("stroke_width".to_string(), HayashiValue::Float(layer.stroke_width));
    layer_dict.insert("size".to_string(), HayashiValue::Float(layer.point_size));
    layer_dict.insert("geometries".to_string(), HayashiValue::List(vec![])); // Placeholder
    let mut new_layers = layers;
    new_layers.push(HayashiValue::Dict(layer_dict));

    // Return updated map
    map.insert("layers".to_string(), HayashiValue::List(new_layers));

    Ok(map)
}

/// Render the map to SVG
#[hayashi_fn]
fn render_impl(
    map: HashMap<String, HayashiValue>
) -> Result<String, String> {

    let width = map.get("width")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(800.0);

    let height = map.get("height")
        .and_then(|v| match v {
            HayashiValue::Float(f) => Some(*f),
            HayashiValue::Int(i) => Some(*i as f64),
            _ => None,
        })
        .unwrap_or(600.0);

    let background = map.get("background")
        .and_then(|v| match v {
            HayashiValue::Str(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("white");

    // For now, return a placeholder SVG
    let mut svg = format!(r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#, width, height, width, height);
    svg = format!(r#"{}<rect width="100%" height="100%" fill="{}"/>"#, svg, background);
    svg.push_str("</svg>");

    Ok(svg)
}

/// Extract a string column from an Arrow StructArray
fn extract_column_string(struct_arr: &StructArray, col_idx: usize) -> Result<Vec<String>, String> {
    let col = struct_arr.column(col_idx);
    let string_array = col.as_any()
        .downcast_ref::<hayashi_plugin_sdk::arrow::array::StringArray>()
        .ok_or_else(|| "Column must be a StringArray".to_string())?;

    string_array.iter()
        .map(|opt: Option<&str>| opt.ok_or_else(|| "null value in column".to_string()).map(|s| s.to_string()))
        .collect()
}
