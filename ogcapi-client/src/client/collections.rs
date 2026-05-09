use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use ogcapi_types::common::{
    Collection, Collections, Link,
    link_rel::{DATA, NEXT},
};

use crate::client::BoxFuture;
use crate::{Client, Error};

impl Client {
    /// Create a new collection.
    pub async fn create_collection(&self, collection: &Collection) -> Result<String, Error> {
        let url = self.endpoint.join("collections")?;
        self.post(url.as_str(), collection).await
    }

    /// Returns a single collection by id.
    pub async fn collection(&self, id: &str) -> Result<Collection, Error> {
        let url = self.endpoint.join(&format!("collections/{id}"))?;
        self.fetch::<Collection>(url.as_str()).await
    }

    /// Update a collection.
    pub async fn update_collection(&self, collection: &Collection) -> Result<(), Error> {
        let id = collection.id.as_str();
        let url = self.endpoint.join(&format!("collections/{id}"))?;
        self.put(url.as_str(), collection).await
    }

    /// Returns a single collection by id.
    pub async fn delete_collection(&self, id: &str) -> Result<(), Error> {
        let url = self.endpoint.join(&format!("collections/{id}"))?;
        self.delete(url.as_str()).await
    }

    /// Returns an async paginating iterator over collections.
    pub async fn collections(&self) -> Result<CollectionsStream, Error> {
        let root = self.root().await?;

        if let Some(link) = root.links.iter().find(|l| l.rel == DATA) {
            let page = self
                .fetch::<ogcapi_types::common::Collections>(&link.href)
                .await?;
            Ok(CollectionsStream {
                client: self.clone(),
                collections: page.collections.into_iter(),
                links: page.links,
                pending: None,
            })
        } else {
            Err(Error::ClientError(
                "No link found with relation `data`!".to_string(),
            ))
        }
    }
}

/// Async paginating iterator over collections. Implements [`Stream`].
pub struct CollectionsStream {
    client: Client,
    collections: <Vec<Collection> as IntoIterator>::IntoIter,
    links: Vec<Link>,
    pending: Option<BoxFuture<Collections>>,
}

impl Stream for CollectionsStream {
    type Item = Result<Collection, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if let Some(fut) = &mut this.pending {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Ok(page)) => {
                        this.collections = page.collections.into_iter();
                        this.links = page.links;
                        this.pending = None;
                    }
                    Poll::Ready(Err(err)) => {
                        this.pending = None;
                        this.links = Vec::new();
                        return Poll::Ready(Some(Err(err)));
                    }
                }
            }

            if let Some(value) = this.collections.next() {
                return Poll::Ready(Some(Ok(value)));
            }

            if let Some(link) = this.links.iter().find(|l| l.rel == NEXT) {
                let href = link.href.clone();
                let client = this.client.clone();
                this.pending = Some(Box::pin(async move {
                    client
                        .fetch::<ogcapi_types::common::Collections>(&href)
                        .await
                }));
            } else {
                return Poll::Ready(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use futures_util::TryStreamExt;

    use super::*;

    #[tokio::test]
    async fn collection() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let collection = client.collection("ch.swisstopo.swissalti3d").await.unwrap();
        assert_eq!("ch.swisstopo.swissalti3d", collection.id);
    }

    #[tokio::test]
    async fn collections() {
        let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
        let client = Client::new(endpoint).unwrap();
        let collections: Vec<_> = client
            .collections()
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap();
        // for c in &collections {
        //     println!("{}", c.id);
        // }
        assert!(!collections.is_empty());
    }
}
