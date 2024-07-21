use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// OGC Symbology Conceptual Model: Core Part

/// Extension type (Literal = Value);
type Extension = Map<String, Value>;

#[derive(Serialize, Deserialize, Debug)]
struct Meta {
    name: Option<ParameterValue>,
    title: ParameterValue,
    r#abstract: Option<ParameterValue>,
}
/// The Style class organizes rules of symbolizing instructions
#[derive(Serialize, Deserialize, Debug)]
struct Style {
    #[serde(flatten)]
    meta: Meta,
    rule: Vec<Rule>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rule {
    #[serde(flatten)]
    meta: Meta,
    symbolizer: Vec<Symbolizer>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Symbolizer {
    #[serde(flatten)]
    meta: Meta,
    uom: Vec<UOM>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParameterValue {
    language: Vec<String>, // IETF RFC 4646
    #[serde(flatten)]
    extension: Option<Extension>,
}

/// Unit of measures
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
enum UOM {
    // portrayal units
    Pixel,
    Millimeter,
    Inch,
    Percentage,
    // ground units:
    Meter,
    Foot,
}

#[derive(Serialize, Deserialize, Debug)]
struct Color {
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Fill {
    uom: Option<UOM>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stroke {
    uom: Option<UOM>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Graphic {
    uom: Option<UOM>,
    graphic_size: Option<GraphicSize>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphicSize {
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Label {
    uom: Option<UOM>,
    label_text: Option<GraphicSize>,
    font: Option<Font>,
    fill: Option<Fill>,
    #[serde(flatten)]
    extension: Option<Extension>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Font {
    uom: Option<UOM>,
    font_family: Vec<ParameterValue>,
    font_size: Option<ParameterValue>,
    font_weigth: Option<ParameterValue>,
    font_style: Option<ParameterValue>,
    #[serde(flatten)]
    extension: Option<Extension>,
}
