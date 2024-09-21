use picoserve::response::StatusCode;

#[derive(Clone, Copy)]
pub struct Js<'a>(&'a str);

impl<'a> picoserve::response::Content for Js<'a> {
    async fn write_content<W: picoserve::io::Write>(self, writer: W) -> Result<(), W::Error> {
        self.0.as_bytes().write_content(writer).await
    }

    fn content_type(&self) -> &'static str {
        "text/javascript"
    }

    fn content_length(&self) -> usize {
        self.0.as_bytes().len()
    }
}

impl<'a> Js<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self(content)
    }
}

mod generated {
    use super::Js;

    include!(concat!(core::env!("OUT_DIR"), "/js.generated.rs"));
}

pub fn get_resource(name: &str) -> impl picoserve::response::IntoResponse {
    if let Some((_, js)) = generated::RESOURCES.iter().find(|(n, _)| *n == name) {
        picoserve::response::Response::new(StatusCode::OK, *js)
    } else {
        picoserve::response::Response::new(StatusCode::NOT_FOUND, Js::new(""))
    }
}
