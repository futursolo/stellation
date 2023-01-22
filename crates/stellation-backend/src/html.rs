use bounce::helmet::HelmetTag;
use lol_html::{doc_comments, element, rewrite_str, Settings};

pub(crate) async fn format_html<I, H, B>(html_s: &str, tags: I, head_s: H, body_s: B) -> String
where
    I: IntoIterator<Item = HelmetTag>,
    H: Into<String>,
    B: AsRef<str>,
{
    let mut head_s = head_s.into();
    let body_s = body_s.as_ref();

    let mut html_tag = None;
    let mut body_tag = None;

    for tag in tags.into_iter() {
        match tag {
            HelmetTag::Html { .. } => {
                html_tag = Some(tag);
            }
            HelmetTag::Body { .. } => {
                body_tag = Some(tag);
            }
            _ => {
                let _ = tag.write_static(&mut head_s);
            }
        }
    }

    rewrite_str(
        html_s,
        Settings {
            element_content_handlers: vec![
                element!("html", |h| {
                    if let Some(HelmetTag::Html { attrs }) = html_tag.take() {
                        for (k, v) in attrs {
                            h.set_attribute(k.as_ref(), v.as_ref())?;
                        }
                    }

                    Ok(())
                }),
                element!("body", |h| {
                    if let Some(HelmetTag::Body { attrs }) = body_tag.take() {
                        for (k, v) in attrs {
                            h.set_attribute(k.as_ref(), v.as_ref())?;
                        }
                    }

                    Ok(())
                }),
            ],

            document_content_handlers: vec![doc_comments!(|c| {
                if c.text() == "%STELLATION_HEAD%" {
                    c.replace(&head_s, lol_html::html_content::ContentType::Html);
                }
                if c.text() == "%STELLATION_BODY%" {
                    c.replace(body_s, lol_html::html_content::ContentType::Html);
                }

                Ok(())
            })],
            ..Default::default()
        },
    )
    .expect("failed to render html")
}
