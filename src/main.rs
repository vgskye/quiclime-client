use std::{
    net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

use gtk::prelude::*;
use quinn::{Connection, Endpoint};
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    loading_widgets::LoadingWidgets,
    view, RelmApp, RelmWidgetExt,
};
use tokio::io::AsyncReadExt;

mod proto;

struct App {
    domain: String,
    connection: Connection,
}

#[derive(Debug)]
enum Msg {
    CopyURL,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    set_label: &format!("Domain: {}", model.domain),
                    set_margin_all: 5,
                },

                gtk::Button {
                    set_label: "Copy URL",
                    connect_clicked => Msg::CopyURL,
                },
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some("e4mc standalone"),
                set_default_size: (300, 75),

                // This will be removed automatically by
                // LoadingWidgets when the full view has loaded
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    // Initialize the component.
    async fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker_response: proto::BrokerResponse =
            reqwest::get("https://broker.e4mc.link/getBestRelay")
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

        let mut client =
            Endpoint::client(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into()).unwrap();

        let mut roots = rustls::RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs")
        {
            roots.add(&rustls::Certificate(cert.0)).unwrap();
        }
        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        config.alpn_protocols = vec![b"quiclime".to_vec()];

        client.set_default_client_config(quinn::ClientConfig::new(Arc::new(config)));

        let connection = client
            .connect(
                (broker_response.host.as_str(), broker_response.port)
                    .to_socket_addrs()
                    .unwrap()
                    .find(|addr| addr.is_ipv4())
                    .unwrap(),
                &broker_response.host,
            )
            .unwrap()
            .await
            .unwrap();

        let (mut send_control, mut recv_control) = connection.open_bi().await.unwrap();

        let request =
            serde_json::to_vec(&proto::ServerboundControlMessage::RequestDomainAssignment).unwrap();
        send_control
            .write_all(&[request.len() as u8])
            .await
            .unwrap();
        send_control.write_all(&request).await.unwrap();

        let mut buf = vec![0u8; recv_control.read_u8().await.unwrap() as _];
        recv_control.read_exact(&mut buf).await.unwrap();
        let response: proto::ClientboundControlMessage = serde_json::from_slice(&buf).unwrap();

        let domain = if let proto::ClientboundControlMessage::DomainAssignmentComplete { domain } =
            response
        {
            domain
        } else {
            panic!("server didn't response with DomainAssignmentComplete")
        };

        let model = App { domain, connection };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        match msg {
            Msg::CopyURL => {
                root.clipboard().set_text(&self.domain);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("link.e4mc.standalone");
    app.run_async::<App>(0);
}
