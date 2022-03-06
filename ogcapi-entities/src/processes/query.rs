use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProcessQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use crate::processes::ProcessQuery;

    #[test]
    fn query() {
        let query = ProcessQuery {
            limit: Some(100),
            offset: None,
        };
        let query_string = serde_qs::to_string(&query).unwrap();
        assert_eq!("limit=100", query_string);
        assert_eq!(
            query,
            serde_qs::from_str::<ProcessQuery>(&query_string).unwrap()
        );
    }
}
