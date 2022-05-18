use url::Url;

pub trait Linked {
    fn get_base_url(&mut self) -> Option<Url>;

    fn resolve_relative_links(&mut self);
}
