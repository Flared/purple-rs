use lazy_static::lazy_static;
use purple_rs::*;
use std::ffi::{CStr, CString};

pub struct PurplePrpl;

pub mod status {
    use lazy_static::lazy_static;
    use std::ffi::CString;
    lazy_static! {
        pub static ref ONLINE_ID: CString = CString::new("online").unwrap();
        pub static ref ONLINE_NAME: CString = CString::new("Online").unwrap();
        pub static ref OFFLINE_ID: CString = CString::new("offline").unwrap();
        pub static ref OFFLINE_NAME: CString = CString::new("Offline").unwrap();
    }
}

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
}

impl purple_rs::PrplPlugin for PurplePrpl {
    type Plugin = Self;

    fn new() -> Self {
        Self
    }

    fn register(&self, context: RegisterContext<Self>) -> RegisterContext<Self> {
        let info = purple_rs::PrplInfo {
            id: "prpl-example".into(),
            name: "Prpl Example".into(),
            version: "0.1".into(),
            summary: "Example protocol implementation".into(),
            description: "Example protocol implementation".into(),
            author: "Israel Halle <israel.halle@flare.systems>".into(),
            homepage: "https://github.com/Flared/purple_rs-rs".into(),
        };

        context
            .with_info(info)
            .enable_login()
            .enable_close()
            .enable_list_icon()
            .enable_status_types()
    }
}

impl purple_rs::LoginHandler for PurplePrpl {
    fn login(&mut self, _account: &mut Account) {
        println!("login");
        purple_rs::task::spawn(async {
            println!("before sleep");
            purple_rs::task::sleep(1000).await;
            println!("after sleep");
            purple_rs::task::sleep(1000).await;
            println!("after second sleep");
        })
    }
}

impl purple_rs::CloseHandler for PurplePrpl {
    fn close(&mut self, _connection: &mut Connection) {}
}

impl purple_rs::ListIconHandler for PurplePrpl {
    fn list_icon(_account: &mut Account) -> &'static CStr {
        &ICON_FILE
    }
}

impl purple_rs::StatusTypeHandler for PurplePrpl {
    fn status_types(_account: &mut Account) -> Vec<StatusType> {
        vec![
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_AVAILABLE,
                Some(&status::ONLINE_ID),
                Some(&status::ONLINE_NAME),
                true,
            ),
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_OFFLINE,
                Some(&status::OFFLINE_ID),
                Some(&status::OFFLINE_NAME),
                true,
            ),
        ]
    }
}

purple_prpl_plugin!(PurplePrpl);
