use std::{fs::File, io::{BufWriter, Write}};

use pulldown_cmark::{html, Options, Parser};


static default_options: Options = Options::all();

pub(crate) fn render(markdown: &String) -> String {
    let parser = Parser::new_ext(markdown, default_options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Adapter that turn [`std::io::Write`] to [`std::fmt::Write`]
struct IoWriteAdapter<'a, W: Write>(&'a mut W);

impl<'a, W: Write> std::fmt::Write for IoWriteAdapter<'a, W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
    }
}

pub(crate) fn render_to_file(md: &str, path: &str) -> std::io::Result<()> {
    let parser = Parser::new_ext(md, Options::all());
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let mut adapter = IoWriteAdapter(&mut writer);
    html::write_html_fmt(&mut adapter, parser).unwrap();
    Ok(())
}
