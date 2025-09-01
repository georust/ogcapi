use chrono::{DateTime, Utc};
use geojson::{LineStringType, PointType, PolygonType};
use serde::{Deserialize, Serialize, Serializer, ser};
use serde_json::json;

use super::{crs::Crs, trs::Trs};

/// TemporalPrimitiveGeometry Object
///
/// A [TemporalPrimitiveGeometry](https://docs.ogc.org/is/19-045r3/19-045r3.html#tprimitive) object describes the
/// movement of a geographic feature whose leaf geometry at a time instant is drawn by a primitive geometry such as a
/// point, linestring, and polygon in the two- or three-dimensional spatial coordinate system, or a point cloud in the
/// three-dimensional spatial coordinate system.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TemporalPrimitiveGeometry {
    pub id: Option<String>,
    #[serde(flatten)]
    pub value: Value,
    #[serde(default)]
    pub interpolation: Interpolation,
    pub crs: Option<Crs>,
    pub trs: Option<Trs>,
}

impl TemporalPrimitiveGeometry {
    pub fn new(value: Value) -> Self {
        Self {
            id: None,
            value,
            interpolation: Interpolation::default(),
            crs: Default::default(),
            trs: Default::default(),
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Value {
    ///The type represents a trajectory of a time-parametered 0-dimensional (0D) geometric primitive (Point),
    ///representing a single position at a time position (instant) within its temporal domain. Intuitively a temporal
    ///geometry of a continuous movement of point depicts a set of curves in a spatiotemporal domain.
    ///It supports more complex movements of moving features, as well as linear movement like MF-JSON Trajectory.
    ///For example, the non-linear movement information of people, vehicles, or hurricanes can be shared as a
    ///TemporalPrimitiveGeometry object with the "MovingPoint" type.
    MovingPoint {
        #[serde(flatten)]
        dt_coords: DateTimeCoords<DateTime<Utc>, PointType>,
        #[serde(flatten)]
        base_representation: Option<BaseRepresentation>,
    },
    ///The type represents the prism of a time-parametered 1-dimensional (1D) geometric primitive (LineString), whose
    ///leaf geometry at a time position is a 1D linear object in a particular period. Intuitively a temporal geometry
    ///of a continuous movement of curve depicts a set of surfaces in a spatiotemporal domain. For example, the
    ///movement information of weather fronts or traffic congestion on roads can be shared as a
    ///TemporalPrimitiveGeometry object with the "MovingLineString" type.
    MovingLineString {
        #[serde(flatten)]
        dt_coords: DateTimeCoords<DateTime<Utc>, LineStringType>,
    },
    ///The type represents the prism of a time-parameterized 2-dimensional (2D) geometric primitive (Polygon), whose
    ///leaf geometry at a time position is a 2D polygonal object in a particular period. The list of points are in
    ///counterclockwise order. Intuitively a temporal geometry of a continuous movement of polygon depicts a set of
    ///volumes in a spatiotemporal domain. For example, the changes of flooding areas or the movement information of
    ///air pollution can be shared as a TemporalPrimitiveGeometry object with the "MovingPolygon" type.
    MovingPolygon {
        #[serde(flatten)]
        dt_coords: DateTimeCoords<DateTime<Utc>, PolygonType>,
    },
    ///The type represents the prism of a time-parameterized point cloud whose leaf geometry at a time position is a
    ///set of points in a particular period. Intuitively a temporal geometry of a continuous movement of point set
    ///depicts a set of curves in a spatiotemporal domain. For example, the tacking information by using Light
    ///Detection and Ranging (LiDAR) can be shared as a TemporalPrimitiveGeometry object with the "MovingPointCloud"
    ///type.
    MovingPointCloud {
        #[serde(flatten)]
        dt_coords: DateTimeCoords<DateTime<Utc>, Vec<PointType>>,
    },
}

impl TryFrom<(Vec<chrono::DateTime<Utc>>, Vec<PointType>)> for Value {
    type Error = String;
    fn try_from(value: (Vec<chrono::DateTime<Utc>>, Vec<PointType>)) -> Result<Self, Self::Error> {
        let dt_coords = DateTimeCoordsUnchecked{
            datetimes: value.0, 
            coordinates: value.1
        }.try_into()?;
        Ok(Self::MovingPoint { dt_coords, base_representation: None })
    }
}

impl<A, B> TryFrom<(Vec<A>, Vec<B>)> for DateTimeCoords<A, B>{
    type Error = &'static str;
    fn try_from(value: (Vec<A>, Vec<B>)) -> Result<Self, Self::Error> {
        DateTimeCoordsUnchecked{
            datetimes: value.0, 
            coordinates: value.1
        }.try_into()
    }
}

#[derive(Deserialize)]
struct DateTimeCoordsUnchecked<A, B> {
    datetimes: Vec<A>,
    coordinates: Vec<B>,
}

#[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[serde(try_from = "DateTimeCoordsUnchecked<A,B>")]
pub struct DateTimeCoords<A, B> {
    datetimes: Vec<A>,
    coordinates: Vec<B>,
}

impl<A,B> DateTimeCoords<A, B> {
    pub fn append(&mut self, other: &mut Self)  {
            self.datetimes.append(&mut other.datetimes);
            self.coordinates.append(&mut other.coordinates);
    }

    pub fn datetimes(&self) -> &[A] {
        self.datetimes.as_slice()
    }

    pub fn coordinates(&self) -> &[B] {
        self.coordinates.as_slice()
    }
}

impl<A,B> Serialize for DateTimeCoords<A, B>{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.coordinates.len() != self.datetimes.len() {
             Err(ser::Error::custom("coordinates and datetimes must be of same length"))
        }else{
            let value = json!(self);
            value.serialize(serializer) 
        }
        
    }
}

impl<A, B> TryFrom<DateTimeCoordsUnchecked<A, B>>
    for DateTimeCoords<A, B>
{
    type Error = &'static str;

    fn try_from(
        value: DateTimeCoordsUnchecked<A, B>,
    ) -> Result<Self, Self::Error> {
        if value.coordinates.len() != value.datetimes.len() {
            Err("coordinates and datetimes must be of same length")
        }else{
            Ok(Self{
                datetimes: value.datetimes, 
                coordinates: value.coordinates
            })
        }
    }
}

///MF-JSON Prism separates out translational motion and rotational motion. The "interpolation" member is default and
///represents the translational motion of the geometry described by the "coordinates" value. Its value is a MotionCurve
///object described by one of predefined five motion curves (i.e., "Discrete", "Step", "Linear", "Quadratic", and
///"Cubic") or a URL (e.g., "<http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/motioncurve>")
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BaseRepresentation {
    base: Base,
    orientations: Vec<Orientation>,
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
pub struct Orientation {
    ///The "scales" member has a array value of numbers along the x, y and z axis in order as three scale factors.
    scales: [f64; 3],
    ///the "angles" member has a JSON array value of numbers along the x, y and z axis in order as Euler angles in degree.
    ///Angles are defined according to the right-hand rule; a positive value represents a rotation that appears clockwise
    ///when looking in the positive direction of the axis and a negative value represents a counter-clockwise rotation.
    angles: [f64; 3],
}
#[cfg(test)]
mod tests {

    use geojson::JsonObject;

    use super::*;

    #[test]
    fn from_json_object() {
        let mut coordinates = vec![];
        let mut datetimes = vec![];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let moving_point = Value::MovingPoint {
            dt_coords: (datetimes, coordinates).try_into().unwrap(),
            base_representation: None,
        };
        let jo: JsonObject = serde_json::from_str(
            r#"{
                "type": "MovingPoint",
                "coordinates": [[0.0, 0.0],[0.0, 1.0],[0.0, 2.0]],
                "datetimes": ["1970-01-01T00:00:00Z", "1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"]
            }"#,
        )
        .unwrap();
        assert_eq!(moving_point, serde_json::from_value(jo.into()).unwrap());
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
            TemporalPrimitiveGeometry::new(Value::MovingPoint {
            dt_coords: (datetimes, coordinates).try_into().unwrap(),
            base_representation: None,
            });
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
            Orientation {
                scales: [1.0, 1.0, 1.0],
                angles: [0.0, 0.0, 0.0],
            },
            Orientation {
                scales: [1.0, 1.0, 1.0],
                angles: [0.0, 355.0, 0.0],
            },
            Orientation {
                scales: [1.0, 1.0, 1.0],
                angles: [0.0, 0.0, 330.0],
            },
        ];
        for i in 0..3 {
            coordinates.push(vec![0., i as f64]);
            datetimes.push(DateTime::from_timestamp(i, 0).unwrap());
        }
        let geometry: TemporalPrimitiveGeometry = TemporalPrimitiveGeometry::new(
            Value::MovingPoint{
                dt_coords: (datetimes, coordinates).try_into().unwrap(),
                base_representation: Some(BaseRepresentation{
                    base: Base{
                        r#type: "glTF".to_string(), 
                        href: "http://www.opengis.net/spec/movingfeatures/json/1.0/prism/example/car3dmodel.gltf".to_string()
                    }, 
                    orientations
                }),
            });
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
