use crate::{Client, Error};
use ogcapi_types::processes::{Execute, Output, Response, Results, StatusInfo, TransmissionMode};
use std::collections::HashMap;

impl Client {
    #[cfg(feature = "processes")]
    pub async fn execute(
        &self,
        process_id: &str,
        execute: &Execute,
    ) -> Result<ProcessResponseBody, Error> {
        let url = format!("{}processes/{}/execution", self.endpoint, process_id);

        let response = self
            .client
            .post(url)
            .json(execute)
            .send()
            .await
            .and_then(|rsp| rsp.error_for_status())?;

        match response.status().as_u16() {
            200 => match execute.response {
                Response::Raw => {
                    if execute.outputs.len() == 1 {
                        let (_k, v) = execute.outputs.iter().next().unwrap();
                        match v.transmission_mode {
                            TransmissionMode::Value => Ok(ProcessResponseBody::Requested {
                                outputs: execute.outputs.clone(),
                                parts: vec![response.bytes().await?.to_vec()],
                            }),
                            TransmissionMode::Reference => todo!(),
                        }
                    } else {
                        unimplemented!()
                    }
                }
                Response::Document => Ok(ProcessResponseBody::Results(
                    response.json::<Results>().await?,
                )),
            },
            201 => Ok(ProcessResponseBody::StatusInfo(
                response.json::<StatusInfo>().await?,
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ProcessResponseBody {
    Requested {
        outputs: HashMap<String, Output>,
        parts: Vec<Vec<u8>>,
    },
    Results(Results),
    Empty(String),
    StatusInfo(StatusInfo),
}

#[cfg(test)]
#[cfg(feature = "greeter")]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "needs running demo service"]
    async fn execute_greeter() {
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

        let response = client.execute(Greeter {}.id(), &execute).await.unwrap();

        let ProcessResponseBody::Requested {
            outputs: _outputs,
            parts,
        } = response
        else {
            panic!()
        };

        assert_eq!(
            String::from_utf8(parts[0].clone()).unwrap(),
            "Hello, client!\n"
        )
    }
}
