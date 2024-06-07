use crate::constants::{EMAIL, MINUTES, PASSWORD};
use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, response::Response, Error},
    Message, SmtpTransport, Transport,
};

pub fn send_email(email: &str, reference: i32, code: i32) -> Result<Response, Error> {
    let creds = Credentials::new(EMAIL.to_string(), PASSWORD.to_string());
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    let message = Message::builder()
        .from(EMAIL.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(format!(
            "Verification code from 789plates, reference: {reference}"
        ))
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "<p style=\"text-align: center\">This code will expire in {MINUTES} minutes</p>
<p style=\"text-align: center\">Your verification code is:</p>
<h1 style=\"text-align: center; padding: 100px\">{code}</h1>
<p style=\"text-align: center\">please don\'t reply to this email</p>"
        ))
        .unwrap();

    mailer.send(&message)
}
