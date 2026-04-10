use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;

#[cfg(feature = "stac")]
use ogcapi_types::stac::SearchParams;
use ogcapi_types::{
    common::{Link, link_rel::NEXT},
    features::{Feature, FeatureCollection},
};

use crate::{Client, Error, client::BoxFuture};

impl Client {
    /// Create a new item.
    pub async fn create_item(
        &self,
        collection_id: &str,
        feature: &Feature,
    ) -> Result<String, Error> {
        let url = self
            .endpoint
            .join(&format!("collections/{collection_id}/items"))?;
        self.post(url.as_str(), feature).await
    }

    /// Fetch a single item of a collection.
    pub async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Feature, Error> {
        let input = format!("collections/{collection_id}/items/{feature_id}");
        let url = self.endpoint.join(&input)?;
        self.fetch::<Feature>(url.as_str()).await
    }

    /// Update an item of a collection.
    pub async fn update_item(&self, collection_id: &str, feature: &Feature) -> Result<(), Error> {
        let input = format!("collections/{collection_id}/items");
        let url = self.endpoint.join(&input)?;
        self.put(url.as_str(), feature).await
    }

    /// Returns a single collection by id.
    pub async fn delete_item(&self, collection_id: &str, feature_id: &str) -> Result<(), Error> {
        let input = format!("collections/{collection_id}/items/{feature_id}");
        let url = self.endpoint.join(&input)?;
        self.delete(url.as_str()).await
    }

    /// Returns an async paginating iterator over items in a collection.
    pub async fn items(&self, id: &str) -> Result<ItemsStream, Error> {
        let url = self.endpoint.join(&format!("collections/{id}/items"))?;
        let page = self.fetch::<FeatureCollection>(url.as_str()).await?;
        Ok(ItemsStream {
            client: self.clone(),
            items: page.features.into_iter(),
            links: page.links,
            pending: None,
        })
    }

    /// Returns an async paginating iterator over items in a collection,
    /// filtered by the given query parameters (bbox, datetime, limit, etc.).
    pub async fn items_with_query(
        &self,
        id: &str,
        query: &ogcapi_types::features::Query,
    ) -> Result<ItemsStream, Error> {
        let base = self.endpoint.join(&format!("collections/{id}/items"))?;
        let qs = serde_qs::to_string(query)?;
        let url = if qs.is_empty() {
            base.to_string()
        } else {
            format!("{base}?{qs}")
        };
        let page = self.fetch::<FeatureCollection>(&url).await?;
        Ok(ItemsStream {
            client: self.clone(),
            items: page.features.into_iter(),
            links: page.links,
            pending: None,
        })
    }

    /// Searches STAC items with the given parameters.
    #[cfg(feature = "stac")]
    pub async fn search(&self, params: SearchParams) -> Result<ItemsStream, Error> {
        let url = self
            .endpoint
            .join(&format!("search?{}", serde_qs::to_string(&params)?))?;
        let page = self.fetch::<FeatureCollection>(url.as_str()).await?;
        Ok(ItemsStream {
            client: self.clone(),
            items: page.features.into_iter(),
            links: page.links,
            pending: None,
        })
    }
}

/// Async paginating iterator over items. Implements [`Stream`].
pub struct ItemsStream {
    client: Client,
    items: <Vec<Feature> as IntoIterator>::IntoIter,
    links: Vec<Link>,
    pending: Option<BoxFuture<FeatureCollection>>,
}

impl Stream for ItemsStream {
    type Item = Result<Feature, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if let Some(fut) = &mut this.pending {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Ok(page)) => {
                        this.items = page.features.into_iter();
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

            if let Some(value) = this.items.next() {
                return Poll::Ready(Some(Ok(value)));
            }

            if let Some(link) = this.links.iter().find(|l| l.rel == NEXT) {
                let href = link.href.clone();
                let client = this.client.clone();
                this.pending = Some(Box::pin(async move {
                    client.fetch::<FeatureCollection>(&href).await
                }));
            } else {
                return Poll::Ready(None);
            }
        }
    }
}
