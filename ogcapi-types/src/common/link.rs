use serde::{Deserialize, Serialize};

/// Hyperlink to enable Hypermedia Access
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Link {
    /// Supplies the URI to a remote resource (or resource fragment).
    pub href: String,
    /// The type or semantics of the relation.
    pub rel: String,
    /// A hint indicating what the media type of the result of dereferencing
    /// the link should be.
    pub r#type: Option<String>,
    /// A hint indicating what the language of the result of dereferencing the
    /// link should be.
    pub hreflang: Option<String>,
    /// Used to label the destination of a link such that it can be used as a
    /// human-readable identifier.
    pub title: Option<String>,
    pub length: Option<i64>,
}

impl Link {
    /// Constructs a new Link with the given href and link relation
    pub fn new(href: impl ToString, rel: impl ToString) -> Link {
        Link {
            href: href.to_string(),
            rel: rel.to_string(),
            r#type: None,
            hreflang: None,
            title: None,
            length: None,
        }
    }

    /// Sets the media type of the Link and returns the Value
    pub fn mediatype(mut self, mime: impl ToString) -> Link {
        self.r#type = Some(mime.to_string());
        self
    }

    /// Sets the language of the Link and returns the Value
    pub fn language(mut self, language: impl ToString) -> Link {
        self.hreflang = Some(language.to_string());
        self
    }

    /// Sets the title of the Link and returns the Value
    pub fn title(mut self, title: impl ToString) -> Link {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the length of the reference resource by the Link and returns the Value
    pub fn length(mut self, length: i64) -> Link {
        self.length = Some(length);
        self
    }
}
