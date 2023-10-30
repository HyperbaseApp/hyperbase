use std::sync::mpsc::{channel, Receiver, Sender};

use anyhow::Result;
use lettre::{
    message::MessageBuilder, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};
use tokio::sync::Mutex;

pub struct Mailer {
    message_builder: MessageBuilder,
    smtp_transport: SmtpTransport,
    channel_receiver: Mutex<Receiver<MailPayload>>,
}

impl Mailer {
    pub fn new(
        smtp_host: &str,
        smtp_username: &str,
        smtp_password: &str,
        sender_name: &str,
        sender_email: &str,
    ) -> (Self, Sender<MailPayload>) {
        let (sender, receiver) = channel::<MailPayload>();

        (
            Self {
                message_builder: Message::builder()
                    .from(format!("{sender_name} <{sender_email}>").parse().unwrap()),
                smtp_transport: SmtpTransport::relay(smtp_host)
                    .unwrap()
                    .credentials(Credentials::new(
                        smtp_username.to_string(),
                        smtp_password.to_string(),
                    ))
                    .build(),
                channel_receiver: Mutex::new(receiver),
            },
            sender,
        )
    }

    pub fn send_mail(&self, payload: MailPayload) -> Result<()> {
        self.smtp_transport.send(
            &self
                .message_builder
                .to_owned()
                .to(payload.to.parse()?)
                .subject(payload.subject)
                .body(payload.body)?,
        )?;
        Ok(())
    }

    pub async fn run(self) -> Result<()> {
        Ok(tokio::spawn((|| async {
            let channel_receiver = self.channel_receiver;
            let message_builder = self.message_builder;
            let smtp_transport = self.smtp_transport;

            loop {
                match channel_receiver.lock().await.recv() {
                    Ok(payload) => {
                        let mailbox = payload.to.parse();
                        if let Err(err) = mailbox {
                            eprintln!("{err}");
                            continue;
                        }
                        let mailbox = mailbox.unwrap();

                        let message = message_builder
                            .to_owned()
                            .to(mailbox)
                            .subject(payload.subject)
                            .body(payload.body);
                        if let Err(err) = message {
                            eprintln!("{err}");
                            continue;
                        }
                        let message = message.unwrap();

                        let res = smtp_transport.send(&message);
                        if let Err(err) = res {
                            eprintln!("{err}");
                            continue;
                        }
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        break;
                    }
                }
            }
        })())
        .await?)
    }
}

pub struct MailPayload {
    to: String,
    subject: String,
    body: String,
}

impl MailPayload {
    pub fn new(to: &str, subject: &str, body: &str) -> Self {
        Self {
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
        }
    }
}
