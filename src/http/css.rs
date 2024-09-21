use picoserve::response::StatusCode;

#[derive(Clone, Copy)]
pub struct Css<'a>(&'a str);

impl<'a> picoserve::response::Content for Css<'a> {
    async fn write_content<W: picoserve::io::Write>(self, writer: W) -> Result<(), W::Error> {
        self.0.as_bytes().write_content(writer).await
    }
    fn content_type(&self) -> &'static str {
        "text/css"
    }
    fn content_length(&self) -> usize {
        self.0.as_bytes().len()
    }
}

impl<'a> Css<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self(content)
    }
}

mod generated {
    use super::Css;

    include!(concat!(core::env!("OUT_DIR"), "/css.generated.rs"));
}

pub fn get_resource(name: &str) -> impl picoserve::response::IntoResponse {
    if let Some((_, css)) = generated::RESOURCES.iter().find(|(n, _)| *n == name) {
        picoserve::response::Response::new(StatusCode::OK, *css)
    } else {
        picoserve::response::Response::new(StatusCode::NOT_FOUND, Css::new(""))
    }
}
