pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Static Generator demo page</title>
  <link rel="stylesheet" href="https://unpkg.com/@nekohack/normalize.css@1.2.1/dist/index.css">
  </head>

"#;

pub fn render_body(body: &str) -> String {
    format!(
        r#"  <body>
    <header>
      <h1><a href="https://github.com/jiyuujin/static_generator">Static Generator</a> demo page</h1>
    </header>
    <main>
      <section>
        <a href="/">Home</a>
        <br />
        {}
      </section>
    </main>
  </body>"#,
        body
    )
}

pub const FOOTER: &str = r#"

</html>
"#;
