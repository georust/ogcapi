use chrono::{DateTime, Utc};
use geojson::{JsonObject, JsonValue, LineStringType, PointType, PolygonType};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Error},
};

use crate::common::Crs;

use super::{json_utils, trs::Trs};

/// TemporalPrimitiveGeometry Object
///
/// A [TemporalPrimitiveGeometry](https://docs.ogc.org/is/19-045r3/19-045r3.html#tprimitive) object describes the
/// movement of a geographic feature whose leaf geometry at a time instant is drawn by a primitive geometry such as a
/// point, linestring, and polygon in the two- or three-dimensional spatial coordinate system, or a point cloud in the
/// three-dimensional spatial coordinate system.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TemporalPrimitiveGeometry {
    #[serde(flatten)]
    pub value: Value,
    #[serde(default)]
    pub interpolation: Interpolation,
    // FIXME apparently specification of moving features CRS and TRS is different to common::CRS ?!
    pub crs: Option<Crs>,
    pub trs: Option<Trs>,
    pub foreign_members: Option<JsonObject>,
}

impl TemporalPrimitiveGeometry {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            interpolation: Interpolation::default(),
            crs: Default::default(),
            trs: Default::default(),
            foreign_members: Default::default(),
        }
    }
}

impl<V> From<V> for TemporalPrimitiveGeometry
where
    V: Into<Value>,
{
    fn from(v: V) -> TemporalPrimitiveGeometry {
        TemporalPrimitiveGeometry::new(v.into())
    }
}

///The value specifies the variants of a TemporalPrimitiveGeometry object with constraints on the interpretation of the
///array value of the "coordinates" member, the same-length "datetimes" array member and the optional members "base" and
///"orientations".
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    ///The type represents a trajectory of a time-parametered 0-dimensional (0D) geometric primitive (Point),
    ///representing a single position at a time position (instant) within its temporal domain. Intuitively a temporal
    ///geometry of a continuous movement of point depicts a set of curves in a spatiotemporal domain.
    ///It supports more complex movements of moving features, as well as linear movement like MF-JSON Trajectory.
    ///For example, the non-linear movement information of people, vehicles, or hurricanes can be shared as a
    ///TemporalPrimitiveGeometry object with the "MovingPoint" type.
    MovingPoint(Vec<(DateTime<Utc>, PointType)>),
    ///The type represents the prism of a time-parametered 1-dimensional (1D) geometric primitive (LineString), whose
    ///leaf geometry at a time position is a 1D linear object in a particular period. Intuitively a temporal geometry
    ///of a continuous movement of curve depicts a set of surfaces in a spatiotemporal domain. For example, the
    ///movement information of weather fronts or traffic congestion on roads can be shared as a
    ///TemporalPrimitiveGeometry object with the "MovingLineString" type.
    MovingLineString(Vec<(DateTime<Utc>, LineStringType)>),
    ///The type represents the prism of a time-parameterized 2-dimensional (2D) geometric primitive (Polygon), whose
    ///leaf geometry at a time position is a 2D polygonal object in a particular period. The list of points are in
    ///counterclockwise order. Intuitively a temporal geometry of a continuous movement of polygon depicts a set of
    ///volumes in a spatiotemporal domain. For example, the changes of flooding areas or the movement information of
    ///air pollution can be shared as a TemporalPrimitiveGeometry object with the "MovingPolygon" type.
    MovingPolygon(Vec<(DateTime<Utc>, PolygonType)>),
    ///The type represents the prism of a time-parameterized point cloud whose leaf geometry at a time position is a
    ///set of points in a particular period. Intuitively a temporal geometry of a continuous movement of point set
    ///depicts a set of curves in a spatiotemporal domain. For example, the tacking information by using Light
    ///Detection and Ranging (LiDAR) can be shared as a TemporalPrimitiveGeometry object with the "MovingPointCloud"
    ///type.
    MovingPointCloud(Vec<(DateTime<Utc>, Vec<PointType>)>),
    ///The constraints on the "base" and "orientation" members are represented in the additional variant "BaseRepresentation"
    ///where a 3D Model given as "Base" is moved along a trajectory of "MovingPoint" and rotated and scaled according to the
    ///"orientations" member.
    BaseRepresentation(Base, Vec<(DateTime<Utc>, PointType, Orientations)>),
}

///MF-JSON Prism separates out translational motion and rotational motion. The "interpolation" member is default and
///represents the translational motion of the geometry described by the "coordinates" value. Its value is a MotionCurve
///object described by one of predefined five motion curves (i.e., "Discrete", "Step", "Linear", "Quadratic", and
///"Cubic") or a URL (e.g., "http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/motioncurve")
///
///See [7.2.10 MotionCurve Objects](https://docs.ogc.org/is/19-045r3/19-045r3.html#interpolation)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum Interpolation {
    ///The positions are NOT connected. The position is valid only at the time instant in datetimes
    Discrete,
    ///This method is realized as a jump from one position to the next at the end of a subinterval. The curve is not
    ///continuous but would be useful for representing an accident or event. This interpolation requires at least two
    ///positions.
    Step,
    ///This method is the default value of the "interpolation" member. It connects straight lines between positions.
    ///The position with respect to time is constructed from linear splines that are two–positions interpolating
    ///polynomials. Therefore, this interpolation also requires at least two positions.
    #[default]
    Linear,
    ///This method interpolates the position at time t by using a piecewise quadratic spline on each interval [t_{-1},t]
    ///with first-order parametric continuity. Between consecutive positions, piecewise quadratic splines are constructed
    ///from the following parametric equations in terms of the time variable. This method results in a curve of a
    ///temporal trajectory that is continuous and has a continuous first derivative at the positions in coordinates
    ///except the two end positions. For this interpolation, at least three leaves at particular times are required.
    Quadratic,
    ///This method interpolates the position at time t by using a Catmull–Rom (cubic) spline on each interval [t_{-1},t].
    ///
    ///See [7.2.10 MotionCurve Objects](https://docs.ogc.org/is/19-045r3/19-045r3.html#interpolation)
    Cubic,
    ///If applications need to define their own interpolation methods, the "interpolation" member in the 
    ///TemporalPrimitiveGeometry object has a URL to a JSON array of parametric equations defined on a set of intervals of parameter t-value.
    ///
    ///See [7.2.10.2 URLs for user-defined parametric curve](https://docs.ogc.org/is/19-045r3/19-045r3.html#_7_2_10_2_urls_for_user_defined_parametric_curve)
    Url(String),
}

///The 3D model represents a base geometry of a 3D shape, and the combination of the "base" and "orientations" members 
///represents a 3D temporal geometry of the MF_RigidTemporalGeometry type in ISO 19141.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Base {
    ///The "type" member has a JSON string to represent a 3D File format such as STL, OBJ, PLY, and glTF.
    r#type: String,
    ///The "href" member has a URL to address 3D model data.
    href: String,
}

///Orientations represents rotational motion of the base representation of a member named "base"
///as a transform matrix of the base representation at each time of the elements in "datetimes".
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Orientations {
    ///The "scales" member has a array value of numbers along the x, y and z axis in order as three scale factors.
    scales: ScaleType,
    ///the "angles" member has a JSON array value of numbers along the x, y and z axis in order as Euler angles in degree. 
    ///Angles are defined according to the right-hand rule; a positive value represents a rotation that appears clockwise 
    ///when looking in the positive direction of the axis and a negative value represents a counter-clockwise rotation.
    angles: AngleType,
}
type ScaleType = Vec<f64>;
type AngleType = Vec<f64>;

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::MovingPoint(_) => "MovingPoint",
            Value::MovingLineString(_) => "MovingLineString",
            Value::MovingPolygon(_) => "MovingPolygon",
            Value::MovingPointCloud(_) => "MovingPointCloud",
            Value::BaseRepresentation(_, _) => "MovingPoint",
        }
    }

    fn unzip(
        &self,
    ) -> (
        Vec<&DateTime<Utc>>,
        Vec<geojson::Value>,
        Option<(&Base, Vec<&Orientations>)>,
    ) {
        match self {
            Value::MovingPoint(x) => {
                let (datetimes, coordinates): (Vec<&DateTime<Utc>>, Vec<geojson::Value>) = x
                    .iter()
                    .map(|(a, b)| (a, geojson::Value::Point(b.to_vec())))
                    .unzip();
                (datetimes, coordinates, None)
            }
            Value::MovingLineString(x) => {
                let (datetimes, coordinates): (Vec<&DateTime<Utc>>, Vec<geojson::Value>) = x
                    .iter()
                    .map(|(a, b)| (a, geojson::Value::LineString(b.to_vec())))
                    .unzip();
                (datetimes, coordinates, None)
            }
            Value::MovingPolygon(x) => {
                let (datetimes, coordinates): (Vec<&DateTime<Utc>>, Vec<geojson::Value>) = x
                    .iter()
                    .map(|(a, b)| (a, geojson::Value::Polygon(b.to_vec())))
                    .unzip();
                (datetimes, coordinates, None)
            }
            Value::MovingPointCloud(x) => {
                let (datetimes, coordinates): (Vec<&DateTime<Utc>>, Vec<geojson::Value>) = x
                    .iter()
                    .map(|(a, b)| (a, geojson::Value::MultiPoint(b.to_vec())))
                    .unzip();
                (datetimes, coordinates, None)
            }
            Value::BaseRepresentation(base, x) => {
                let (datetimes, coordinates): (Vec<&DateTime<Utc>>, Vec<geojson::Value>) = x
                    .iter()
                    .map(|(a, b, _)| (a, geojson::Value::Point(b.to_vec())))
                    .unzip();
                let orientations: Vec<&Orientations> =
                    x.iter().map(|(_, _, orientations)| orientations).collect();
                (datetimes, coordinates, Some((base, orientations)))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;

        let mut val = JsonObject::deserialize(deserializer)?;

        Value::try_from(&mut val).map_err(|e| D::Error::custom(e.to_string()))
    }
}

impl TryFrom<&mut JsonObject> for Value {
    type Error = serde_json::Error;

    fn try_from(object: &mut JsonObject) -> Result<Self, Self::Error> {
        let res = &*json_utils::expect_type(object)?;
        let coordinates = json_utils::expect_named_vec(object, "coordinates")?;
        let dt = json_utils::expect_named_vec(object, "datetimes")?;
        let base: Option<Base> = object
            .remove("base")
            .map(serde_json::from_value)
            .transpose()?;
        if coordinates.len() != dt.len() {
            Err(serde_json::Error::invalid_length(
                dt.len(),
                &"coordinates and datetimes must be of same length!",
            ))?;
        }
        let datetimes = json_utils::deserialize_iter::<DateTime<Utc>>(dt);
        match res {
            "MovingPoint" if base.is_some() => {
                let orientations = json_utils::expect_named_vec(object, "orientations")?;
                if coordinates.len() != orientations.len() {
                    Err(serde_json::Error::invalid_length(
                        coordinates.len(),
                        &"orientations, coordinates and datetimes must be of same length!",
                    ))?;
                }
                Ok(Value::BaseRepresentation(
                    base.unwrap(),
                    datetimes
                        .zip(json_utils::deserialize_iter::<PointType>(coordinates))
                        .zip(json_utils::deserialize_iter::<Orientations>(orientations))
                        .map(|((dt, coord), orientations)| (dt, coord, orientations))
                        .collect(),
                ))
            }
            "MovingPoint" => Ok(Value::MovingPoint(
                datetimes
                    .zip(json_utils::deserialize_iter::<PointType>(coordinates))
                    .collect(),
            )),
            "MovingLineString" => Ok(Value::MovingLineString(
                datetimes
                    .into_iter()
                    .zip(json_utils::deserialize_iter::<LineStringType>(coordinates))
                    .collect(),
            )),
            "MovingPolygon" => Ok(Value::MovingPolygon(
                datetimes
                    .into_iter()
                    .zip(json_utils::deserialize_iter::<PolygonType>(coordinates))
                    .collect(),
            )),
            "MovingPointCloud" => Ok(Value::MovingPointCloud(
                datetimes
                    .into_iter()
                    .zip(json_utils::deserialize_iter::<Vec<PointType>>(coordinates))
                    .collect(),
            )),
            unknown_variant => Err(serde_json::Error::unknown_variant(
                unknown_variant,
                &[
                    "MovingPoint",
                    "MovingLineString",
                    "MovingPolygon",
                    "MovingPointCloud",
                ],
            )),
        }
    }
}

impl TryFrom<JsonValue> for Value {
    type Error = serde_json::Error;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        if let JsonValue::Object(mut obj) = value {
            Self::try_from(&mut obj)
        } else {
            Err(serde_json::Error::invalid_type(
                de::Unexpected::Other("type"),
                &"object",
            ))
        }
    }
}

impl<'a> From<&'a Value> for JsonValue {
    fn from(value: &'a Value) -> JsonValue {
        serde_json::to_value(value).unwrap()
    }
}

impl<'a> From<&'a Value> for JsonObject {
    fn from(value: &'a Value) -> JsonObject {
        let (datetimes, coordinates, base_rep) = value.unzip();
        let mut map = JsonObject::new();
        map.insert(
            String::from("type"),
            // The unwrap() should never panic, because &str always serializes to JSON
            serde_json::to_value(value.type_name()).unwrap(),
        );
        map.insert(
            String::from("coordinates"),
            // The unwrap() should never panic, because coordinates contains only JSON-serializable types
            serde_json::to_value(coordinates).unwrap(),
        );
        map.insert(
            String::from("datetimes"),
            // The unwrap() should never panic, because Value contains only JSON-serializable types
            serde_json::to_value(datetimes).unwrap(),
        );
        if let Some(base_rep) = base_rep {
            map.insert(
                String::from("base"),
                // The unwrap() should never panic, because Base contains only JSON-serializable types
                serde_json::to_value(base_rep.0).unwrap(),
            );
            map.insert(
                String::from("orientations"),
                // The unwrap() should never panic, because Orientations contains only JSON-serializable types
                serde_json::to_value(base_rep.1).unwrap(),
            );
        }
        map
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_object: JsonObject = self.into();
        json_object.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_json_object() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let moving_point = Value::MovingPoint(datetimes.into_iter().zip(coordinates).collect());
        let mut jo: JsonObject = serde_json::from_str(
            r#"{
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
            }"#,
        )
        .unwrap();
        assert_eq!(moving_point, Value::try_from(&mut jo).unwrap());
    }

    #[test]
    fn from_json_value() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let moving_point = Value::MovingPoint(datetimes.into_iter().zip(coordinates).collect());
        let jv: JsonValue = serde_json::from_str(
            r#"{
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
            }"#,
        )
        .unwrap();
        assert_eq!(moving_point, Value::try_from(jv).unwrap());
    }

    #[test]
    fn moving_geometry_from_json_value() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let geometry: TemporalPrimitiveGeometry =
            Value::MovingPoint(datetimes.into_iter().zip(coordinates).collect()).into();
        let deserialized_geometry: TemporalPrimitiveGeometry = serde_json::from_str(
            r#"{
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
            }"#,
        )
        .unwrap();
        assert_eq!(geometry, deserialized_geometry);
    }

    #[test]
    fn invalid_moving_geometry_from_json_value() {
        let geometry_too_few_datetimes: Result<TemporalPrimitiveGeometry, serde_json::Error> =
            serde_json::from_str(
                r#"{
                    "type": "MovingPoint",
                    "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                    "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z"]
                }"#,
            );
        assert!(geometry_too_few_datetimes.is_err());

        let geometry_too_few_coordinates: Result<TemporalPrimitiveGeometry, serde_json::Error> =
            serde_json::from_str(
                r#"{
                    "type": "MovingPoint",
                    "coordinates": [[0.0, 0.0],[0.0, 1.0]],
                    "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
                }"#,
            );
        assert!(geometry_too_few_coordinates.is_err())
    }

    #[test]
    fn moving_base_rep_geometry_from_json_value() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        let orientations = vec![
            Orientations {
                scales: vec![1.0, 1.0, 1.0],
                angles: vec![0.0, 0.0, 0.0],
            },
            Orientations {
                scales: vec![1.0, 1.0, 1.0],
                angles: vec![0.0, 355.0, 0.0],
            },
            Orientations {
                scales: vec![1.0, 1.0, 1.0],
                angles: vec![0.0, 0.0, 330.0],
            },
        ];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let geometry: TemporalPrimitiveGeometry =
            Value::BaseRepresentation(
                Base{r#type: "glTF".to_string(), href: "http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/car3dmodel.gltf".to_string()}, 
                datetimes.into_iter().zip(coordinates).zip(orientations).map(|((a,b), c)| (a,b,c)).collect()).into();
        let deserialized_geometry: TemporalPrimitiveGeometry = serde_json::from_str(
        r#"{ 
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"],
                "interpolation": "Linear",
                "base": {
                  "type": "glTF",
                  "href": "http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/car3dmodel.gltf"
                },
                "orientations": [
                  {"scales":[1,1,1], "angles":[0,0,0]},
                  {"scales":[1,1,1], "angles":[0,355,0]},
                  {"scales":[1,1,1], "angles":[0,0,330]}
                ]
            }"#
        )
        .unwrap();
        assert_eq!(geometry, deserialized_geometry);
    }
}
