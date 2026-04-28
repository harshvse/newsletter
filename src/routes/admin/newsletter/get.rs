use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};
use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn newsletter_publish_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut error_string = String::new();
    for m in flash_messages.iter() {
        writeln!(error_string, "<p><i>{}</i></p>", m.content()).unwrap()
    }

    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    };
    let idempotency_key = uuid::Uuid::new_v4();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Publish Newsletter</title>
</head>
<body>
{error_string}
<form action="/admin/newsletters" method="post">
<label>Newsletter title
<input
type="text"
placeholder="Enter the title for the newsletter"
name="title"
>
</label>
<br>
<label>Html Content
<textarea
type="text-area"
placeholder="Enter the newsletter content in html"
name="html_content"
>
</textarea>
<label>Text Content
<textarea
type="text-area"
placeholder="Enter the newsletter content in text"
name="text_content"
>
</textarea>
<br>
<input hidden type="text" name="idempotency_key" value="{idempotency_key}">
<button type="submit">Publish Newsletter</button>
</form>
<p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}
