// Collection of data, published or curated by a single agent, and available for access or download in one or more formats [DCAT]
struct Dataset {
    // collection: Collection,
    formats: Vec<String>,
    distributions: Vec<Distribution>,
    agent: String,
}

/// Represents an accessible form of a dataset [DCAT]
struct Distribution {
    form: String,
}
