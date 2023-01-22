use lol_html::{doc_comments, rewrite_str, Settings};
use once_cell::sync::Lazy;

use crate::SERVER_ID;

static AUTO_REFRESH_SCRIPT: Lazy<String> = Lazy::new(|| {
    format!(
        r#"
<script>
    (() => {{
        const protocol = window.location.protocol === 'https' ? 'wss' : 'ws';
        const wsUrl = `${{protocol}}://${{window.location.host}}/_refresh`;
        const serverId = '{}';

        const connectWs = () => {{
            const ws = new WebSocket(wsUrl);
            ws.addEventListener('open', () => {{
                const invId = setInterval(() => {{
                    try {{
                        ws.send(serverId);
                    }} catch(e) {{
                        // do nothing if errored.
                    }}
                }}, 1000);
                ws.addEventListener('error', () => {{
                    clearInterval(invId);
                }});
            }});
            ws.addEventListener('close', () => {{
                setTimeout(connectWs, 1000);
            }});
            ws.addEventListener('message', (e) => {{
                if (e.data === 'restart') {{
                    window.location.reload();
                }}
            }});
        }};

        connectWs();
    }})();
</script>"#,
        SERVER_ID.as_str()
    )
});

pub(crate) fn add_refresh_script(html_s: &str) -> String {
    rewrite_str(
        html_s,
        Settings {
            document_content_handlers: vec![doc_comments!(|c| {
                if c.text() == "%STELLATION_BODY%" {
                    c.after(
                        AUTO_REFRESH_SCRIPT.as_str(),
                        lol_html::html_content::ContentType::Html,
                    );
                }
                Ok(())
            })],
            ..Default::default()
        },
    )
    .expect("failed to render html")
}
