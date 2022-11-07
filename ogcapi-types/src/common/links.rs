use url::Url;

use super::{link_rel::SELF, Link};

#[doc(hidden)]
pub type Links = Vec<Link>;

#[doc(hidden)]
pub trait Linked {
    fn get_base_url(&mut self) -> Option<Url>;

    fn resolve_relative_links(&mut self);

    fn insert_or_update(&mut self, other: &[Link]);
}

impl Linked for Links {
    fn get_base_url(&mut self) -> Option<Url> {
        self.iter()
            .find(|l| l.rel == SELF)
            .and_then(|l| l.href.parse().ok())
    }

    fn resolve_relative_links(&mut self) {
        if let Some(base) = self.get_base_url() {
            for link in self.iter_mut() {
                if link.href.starts_with("http") || link.href.starts_with('/') {
                    continue;
                }

                match base.join(&link.href) {
                    Ok(url) => link.href = url.to_string(),
                    Err(_) => {
                        log::error!(
                            "Unable to resolve link `{}` with base `{}`",
                            link.href, base
                        )
                    }
                }
            }
        }
    }

    fn insert_or_update(&mut self, others: &[Link]) {
        for link in others {
            self.iter_mut()
                .find(|l| l.rel == link.rel)
                .map(|l| l.href = link.href.to_owned())
                .unwrap_or_else(|| self.push(link.to_owned()));
        }
    }
}
