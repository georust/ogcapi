use url::Url;

use super::{link_rel::SELF, Link, Linked};

#[doc(hidden)]
pub type Links = Vec<Link>;

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
                        eprintln!(
                            "Unable to resolve link `{}` with base `{}`",
                            link.href, base
                        )
                    }
                }
            }
        }
    }
}

#[test]
fn join() {
    let base: url::Url = "http://ogcapi/collections/collection?pi=3".parse().unwrap();
    println!("{}", base.join("../items").unwrap());
}
