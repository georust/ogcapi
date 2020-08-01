
pub struct ProcessCollection {
    processes: Vec<Process>,
}

pub struct Process {
    id: String,
    title: String,
    version: String,
    job_control_options: Vec<String>,
    output_transmission: Vec<String>,
    inputs: Vec<Input>,
}

pub struct Input {
    id: String,
    title: String,
    input: InputDef,
    min_occurs: usize,
    max_occurs: usize,
}

pub struct Job {
    id: String,
    process: &Process,
    result: Option<Result>

}

pub struct Result {}

pub struct Processes {}

impl Service for Processes {
    fn conformance() -> Conformance {
        Conformance {
            conforms_to: vec![
                "http://www.opengis.net/spec/ogcapi-processes/1.0/conf/core".to_string(),
                "http://www.opengis.net/spec/ogcapi-processes/1.0/conf/oas30".to_string(),
                "http://www.opengis.net/spec/ogcapi-processes/1.0/conf/geojson".to_string(),
            ],
        }
    }

    fn api() -> OpenAPI {
        
    }

}