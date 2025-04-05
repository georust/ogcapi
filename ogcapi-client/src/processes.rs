use crate::{Client, Error};

use ogcapi_processes::ProcessResponseBody;
use ogcapi_types::processes::{Execute, Response, Results, StatusInfo, TransmissionMode};

impl Client {
    #[cfg(feature = "processes")]
    pub fn execute(
        &self,
        process_id: &str,
        execute: &Execute,
    ) -> Result<ProcessResponseBody, Error> {
        let url = format!("{}processes/{}/execution", self.endpoint, process_id);

        let response = self
            .client
            .post(url)
            .json(execute)
            // .header("prefer", "respond-async")
            .send()
            .and_then(|rsp| rsp.error_for_status())?;

        dbg!(&response);

        match response.status().as_u16() {
            200 => match execute.response {
                Response::Raw => {
                    if execute.outputs.len() == 1 {
                        let (_k, v) = execute.outputs.iter().next().unwrap();
                        match v.transmission_mode {
                            TransmissionMode::Value => {
                                Ok(ProcessResponseBody::Requested(response.bytes()?.to_vec()))
                            }
                            TransmissionMode::Reference => todo!(),
                        }
                    } else {
                        unimplemented!()
                    }
                }
                Response::Document => Ok(ProcessResponseBody::Results(response.json::<Results>()?)),
            },
            201 => Ok(ProcessResponseBody::StatusInfo(
                response.json::<StatusInfo>()?,
            )),
            204 => match response.headers().get("link").and_then(|l| l.to_str().ok()) {
                Some(s) => Ok(ProcessResponseBody::Empty(s.to_string())),
                None => Err(Error::ServerError(
                    "Missing or malformed `link` header for 204 status response.".to_string(),
                )),
            },
            _ => Err(Error::ServerError(
                "Unspecified success status code.".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    // use ogcapi_processes::gdal_loader::GdalLoaderOutputs;
    use ogcapi_types::processes::Execute;

    use super::*;

    #[test]
    fn execute_greeter() {
        use ogcapi_processes::{
            Processor,
            greeter::{Greeter, GreeterInputs, GreeterOutputs},
        };

        let endpoint = "http://0.0.0.0:8484/";
        let client = Client::new(endpoint).unwrap();

        let input = GreeterInputs {
            name: "client".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            outputs: GreeterOutputs::execute_output(),
            ..Default::default()
        };

        let response = client.execute(Greeter {}.id(), &execute).unwrap();

        let output: GreeterOutputs = response.try_into().unwrap();
        assert_eq!(output.greeting, "Hello, client!\n")
    }

    // #[test]
    // fn execute_gdal_loader() {
    //     use ogcapi_processes::{
    //         Processor,
    //         gdal_loader::{GdalLoader, GdalLoaderInputs},
    //     };

    //     let endpoint = "http://0.0.0.0:8484/";
    //     let client = Client::new(endpoint).unwrap();

    //     let input = GdalLoaderInputs {
    //         input: "/data/ne_10m_railroads_north_america.geojson".to_owned(),
    //         collection: "streets".to_string(),
    //         filter: None,
    //         s_srs: None,
    //         database_url: "postgresql://postgres:password@db:5432/ogcapi".to_string(),
    //     };

    //     let execute = Execute {
    //         inputs: input.execute_input(),
    //         outputs: GdalLoaderOutputs::execute_output(),
    //         ..Default::default()
    //     };

    //     let response = client.execute(GdalLoader {}.id(), &execute).unwrap();

    //     let output: GdalLoaderOutputs = response.try_into().unwrap();
    //     dbg!(output);
    // }
}
