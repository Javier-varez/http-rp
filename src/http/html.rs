use picoserve::response::StatusCode;

#[derive(Copy, Clone)]
pub struct Html<'a>(&'a str);

impl<'a> picoserve::response::Content for Html<'a> {
    async fn write_content<W: picoserve::io::Write>(self, writer: W) -> Result<(), W::Error> {
        self.0.as_bytes().write_content(writer).await
    }

    fn content_type(&self) -> &'static str {
        "text/html"
    }

    fn content_length(&self) -> usize {
        self.0.as_bytes().len()
    }
}

impl<'a> Html<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self(content)
    }
}

mod generated {
    use super::Html;
    pub(super) static RESOURCES: [(&'static str, Html); 1] = [(
        "index.html",
        Html::new(include_str!("../../html/index.html")),
    )];
}

pub fn get_resource(mut name: &str) -> impl picoserve::response::IntoResponse {
    if name == "" {
        name = "index.html"
    }

    if let Some((_, html)) = generated::RESOURCES.iter().find(|(n, _)| *n == name) {
        picoserve::response::Response::new(StatusCode::OK, *html)
    } else {
        picoserve::response::Response::new(StatusCode::NOT_FOUND, Html::new(""))
    }
}
