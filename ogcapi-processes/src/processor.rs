use anyhow::{Error, Result};
use ogcapi_types::processes::{Execute, ExecuteResults, Process};
use std::{future::Future, pin::Pin, sync::Arc};

/// Trait for defining and executing a [`Process`]
#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    /// Input type for this process once the [`Execute`] payload has been parsed and validated.
    type Input: Send + 'static;
    /// Output type for this process once the [`Execute`] payload has been executed.
    type Output: Send + 'static + TryInto<ExecuteResults, Error = Error>;

    /// Returns the process id (must be unique)
    fn id(&self) -> &'static str;

    /// Returns the process version
    fn version(&self) -> &'static str;

    /// Returns the Process description
    async fn process(&self) -> Result<Process>;

    /// Parses and validates the [`Execute`] payload before execution
    async fn parse(&self, execute: Execute) -> Result<Self::Input>;

    /// Executes the Process and returns [`ExecuteResults`]
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Object-safe execution adapter for a typed [`Processor`].
#[async_trait::async_trait]
pub trait DynProcessor: Send + Sync {
    /// Returns the process id (must be unique)
    fn id(&self) -> &'static str;

    /// Returns the process version
    fn version(&self) -> &'static str;

    /// Returns the Process description
    async fn process(&self) -> Result<Process>;

    /// Prepares the [`Execute`] payload for execution, returning a future that will execute the process.
    async fn prepare(self: Arc<Self>, execute: Execute) -> Result<ProcessExecution>;
}

pub type ProcessExecution = Pin<Box<dyn Future<Output = Result<ExecuteResults>> + Send>>;

#[async_trait::async_trait]
impl<T> DynProcessor for T
where
    T: Processor + 'static,
{
    fn id(&self) -> &'static str {
        <Self as Processor>::id(self)
    }

    fn version(&self) -> &'static str {
        <Self as Processor>::version(self)
    }

    async fn process(&self) -> Result<Process> {
        <Self as Processor>::process(self).await
    }

    async fn prepare(
        self: Arc<Self>,
        execute: Execute,
    ) -> Result<Pin<Box<dyn Future<Output = Result<ExecuteResults>> + Send>>> {
        let input = self.parse(execute).await?;

        Ok(Box::pin(
            async move { self.execute(input).await?.try_into() },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ogcapi_types::processes::{
        ExecuteResult, Format, InlineOrRefData, Input, InputValueNoObject, Output, TransmissionMode,
    };
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Clone, Debug, Deserialize)]
    struct TestInput {
        name: String,
    }

    #[derive(Debug, PartialEq)]
    struct TestOutput {
        greeting: String,
    }

    impl TryFrom<TestOutput> for ExecuteResults {
        type Error = Error;

        fn try_from(value: TestOutput) -> Result<Self, Self::Error> {
            let mut results = ExecuteResults::new();
            results.insert(
                "greeting".to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(Format {
                            media_type: Some("text/plain".to_string()),
                            encoding: Some("utf8".to_string()),
                            schema: None,
                        }),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        value.greeting,
                    )),
                },
            );
            Ok(results)
        }
    }

    #[derive(Clone, Debug)]
    struct TestProcessor;

    #[async_trait::async_trait]
    impl Processor for TestProcessor {
        type Input = TestInput;
        type Output = TestOutput;

        fn id(&self) -> &'static str {
            "test"
        }

        fn version(&self) -> &'static str {
            "1.0.0"
        }

        async fn process(&self) -> anyhow::Result<ogcapi_types::processes::Process> {
            panic!("not used in this unit test")
        }

        async fn parse(&self, execute: Execute) -> Result<Self::Input> {
            let value = serde_json::to_value(execute.inputs)?;
            Ok(serde_json::from_value(value)?)
        }

        async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
            assert_eq!(input.name, "Ada");
            Ok(TestOutput {
                greeting: format!("Hello, {}!", input.name),
            })
        }
    }

    #[tokio::test]
    async fn prepare_fails_before_spawn_on_invalid_payload() {
        let processor: Arc<dyn DynProcessor> = Arc::new(TestProcessor);

        let mut execute = Execute::default();
        execute.inputs.insert(
            "wrong".to_string(),
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String("Ada".to_string()),
            )),
        );

        let invalid = processor.prepare(execute).await;
        assert!(invalid.is_err());
    }

    #[tokio::test]
    async fn prepare_allows_spawn_after_valid_payload() {
        let processor: Arc<dyn DynProcessor> = Arc::new(TestProcessor);
        let mut execute = Execute::default();
        execute.inputs.insert(
            "name".to_string(),
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String("Ada".to_string()),
            )),
        );
        let future = processor.prepare(execute).await.unwrap();

        let results = future.await.unwrap();

        assert_eq!(results.len(), 1);

        assert_eq!(
            results["greeting"].data,
            InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                "Hello, Ada!".to_string()
            ),)
        );
    }
}
